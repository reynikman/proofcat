//! Fault-injection / chaos-стенд для offload-движка.
//!
//! Надёжность доказываем не happy-path тестом, а симуляцией именно тех сбоев,
//! на которых обычный copy рассыпается: получатель уходит в read-only на
//! середине джоба, диск полон, юникод/эмодзи/длинные имена, симлинки (в т.ч. наружу
//! источника), пустые файлы, смесь «один гигант + триста мелких», тихая порча байта.
//!
//! Всё под `#[cfg(test)]` — ноль стоимости в рантайме, ноль правок движка. Тесты
//! дёргают ровно тот публичный API, что и продакшн.
//!
//! Два инварианта, которые обязаны держаться (иначе весь offload — обман):
//!   1. tee-инвариант: inline-хеш == независимый хеш того, что реально легло в dest
//!      == хеш источника. Читаем один раз, а сойтись должно на диске.
//!   2. MHL реально ловит подмену: порча одного байта в получателе → verify FAIL.

#![cfg(test)]

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::offload::copy_engine::{
    check_available_space, copy_file_multi, CopyControl, CopyEngineConfig,
};
use crate::offload::hash_engine::{hash_bytes, HashAlgorithm, HashResult};
use crate::offload::mhl::verifier::{verify_mhl_path, MhlVerifyOptions};
use crate::offload::orchestrator::{run_offload, OffloadRequest, OffloadSummary};

// ── helpers ────────────────────────────────────────────────────────────────

fn write_file(path: &Path, bytes: &[u8]) {
    if let Some(p) = path.parent() {
        std::fs::create_dir_all(p).unwrap();
    }
    std::fs::write(path, bytes).unwrap();
}

/// Детерминированные псевдослучайные байты без внешних зависимостей (xorshift64).
/// Один seed → одна и та же строка, тесты воспроизводимы.
fn pseudo_bytes(seed: u64, len: usize) -> Vec<u8> {
    let mut x = seed | 1;
    let mut v = Vec::with_capacity(len);
    for _ in 0..len {
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        v.push((x & 0xff) as u8);
    }
    v
}

fn req(src: &Path, dests: &[PathBuf], db: Option<PathBuf>, mhl: bool) -> OffloadRequest {
    OffloadRequest {
        source: src.to_path_buf(),
        destinations: dests.to_vec(),
        algorithms: vec![HashAlgorithm::XXH64, HashAlgorithm::SHA256],
        write_mhl: mhl,
        checkpoint_db: db,
        profile: crate::offload::orchestrator::VerificationProfile::Fast,
        job_id: None,
        small_file_concurrency: 1,
        report_contacts: Vec::new(),
        auto_eject: false,
    }
}

async fn run(request: &OffloadRequest) -> anyhow::Result<OffloadSummary> {
    run_offload(request, None, None, &|_| {}).await
}

fn assert_identical(a: &Path, b: &Path) {
    let ba = std::fs::read(a).unwrap_or_else(|e| panic!("read {a:?}: {e}"));
    let bb = std::fs::read(b).unwrap_or_else(|e| panic!("read {b:?}: {e}"));
    assert_eq!(ba, bb, "byte mismatch {a:?} vs {b:?}");
}

/// (алгоритм → hex) для сравнения хешей независимо от порядка.
fn hash_map(results: &[HashResult]) -> HashMap<HashAlgorithm, String> {
    results
        .iter()
        .map(|r| (r.algorithm, r.hex_digest.clone()))
        .collect()
}

// ── 1. Получатель уходит в read-only на середине джоба ──────────────────────
// Реальный кейс: у одного из двух дисков отвалилась запись. Здоровый диск обязан
// получить полную копию + MHL; больной — попасть в failures, а не уронить джоб.
#[cfg(unix)]
#[tokio::test]
async fn readonly_dest_isolated_healthy_dest_survives() {
    use std::os::unix::fs::PermissionsExt;
    let src = tempfile::tempdir().unwrap();
    let good = tempfile::tempdir().unwrap();
    let bad = tempfile::tempdir().unwrap();

    write_file(&src.path().join("A001/clip1.mov"), &pseudo_bytes(1, 40_000));
    write_file(&src.path().join("A001/clip2.mov"), &pseudo_bytes(2, 20_000));
    write_file(&src.path().join("roll.txt"), b"scene 12 take 3");

    // Больной получатель: dir read-only → AtomicWriter не создаст temp.
    std::fs::set_permissions(bad.path(), std::fs::Permissions::from_mode(0o555)).unwrap();

    let request = req(
        src.path(),
        &[good.path().to_path_buf(), bad.path().to_path_buf()],
        None,
        true,
    );
    let summary = run(&request).await.unwrap();

    // Вернуть запись ДО ассертов, иначе tempdir Drop не подчистит.
    std::fs::set_permissions(bad.path(), std::fs::Permissions::from_mode(0o755)).unwrap();

    // Файл считается failed, если ЛЮБОЙ получатель не смог (текущий контракт).
    assert_eq!(summary.failed, 3, "each file failed on the read-only dest");
    assert_eq!(summary.copied, 0);

    // Но здоровый диск получил байт-в-байт всё + свой ascmhl.
    for rel in ["A001/clip1.mov", "A001/clip2.mov", "roll.txt"] {
        assert_identical(&src.path().join(rel), &good.path().join(rel));
    }
    assert!(
        good.path().join("ascmhl").is_dir(),
        "healthy dest MHL missing"
    );
    assert_eq!(
        summary.mhl_paths.len(),
        1,
        "only healthy dest gets a generation"
    );
}

// ── 2. Диск полон — pre-flight обязан упасть ДО первого байта ────────────────
// ENOSPC на реальной FS не форсируем; проверяем контракт check_available_space:
// требование больше свободного → Err, а не тихий недокоп.
#[tokio::test]
async fn disk_full_preflight_bails() {
    let d = tempfile::tempdir().unwrap();
    let dest = d.path().join("huge.bin");
    let res = check_available_space(&dest, u64::MAX / 2).await;
    assert!(res.is_err(), "must refuse when required > available");
}

// ── 3. Юникод / эмодзи / длинные имена / кириллица в путях ───────────────────
#[tokio::test]
async fn unicode_emoji_long_names_roundtrip() {
    let src = tempfile::tempdir().unwrap();
    let dst = tempfile::tempdir().unwrap();

    let long = format!("{}.mov", "a".repeat(200));
    write_file(&src.path().join("Кадр_01.mov"), &pseudo_bytes(3, 5000));
    write_file(
        &src.path().join("clip 🎬 final.mov"),
        &pseudo_bytes(4, 6000),
    );
    write_file(&src.path().join(&long), &pseudo_bytes(5, 7000));
    write_file(&src.path().join("Días/niño_señal.txt"), "café".as_bytes());

    let request = req(src.path(), &[dst.path().to_path_buf()], None, true);
    let summary = run(&request).await.unwrap();
    assert_eq!(summary.copied, 4);
    assert_eq!(summary.failed, 0);

    for rel in [
        "Кадр_01.mov",
        "clip 🎬 final.mov",
        &long,
        "Días/niño_señal.txt",
    ] {
        assert_identical(&src.path().join(rel), &dst.path().join(rel));
    }

    // MHL с юникод-путями обязан пройти собственную верификацию.
    let report = verify_mhl_path(dst.path(), MhlVerifyOptions::default()).unwrap();
    assert!(
        report.summary.success,
        "MHL verify failed on unicode tree: {:?}",
        report.issues
    );
}

// ── 4. Симлинки в источнике — пропускаем, НЕ следуем (в т.ч. наружу) ─────────
// Безопасность: симлинк на файл вне источника не должен утащить чужие данные
// в копию. DirEntry::metadata() не резолвит симлинк → не файл/не папка → скип.
#[cfg(unix)]
#[tokio::test]
async fn symlinks_skipped_not_followed() {
    let src = tempfile::tempdir().unwrap();
    let dst = tempfile::tempdir().unwrap();
    let outside = tempfile::tempdir().unwrap();
    write_file(&outside.path().join("secret.txt"), b"do not exfiltrate");

    write_file(&src.path().join("real.mov"), &pseudo_bytes(6, 3000));
    // Симлинк на файл вне источника + симлинк на директорию.
    std::os::unix::fs::symlink(
        outside.path().join("secret.txt"),
        src.path().join("link.txt"),
    )
    .unwrap();
    std::os::unix::fs::symlink(outside.path(), src.path().join("link_dir")).unwrap();

    let request = req(src.path(), &[dst.path().to_path_buf()], None, false);
    let summary = run(&request).await.unwrap();

    assert_eq!(summary.total_files, 1, "only the real file is scanned");
    assert_eq!(summary.copied, 1);
    assert!(dst.path().join("real.mov").exists());
    assert!(
        !dst.path().join("link.txt").exists(),
        "symlink must not be copied"
    );
    assert!(
        !dst.path().join("secret.txt").exists(),
        "outside data must not leak"
    );
    assert!(!dst.path().join("link_dir").exists());
}

// ── 5. Пустой файл (0 байт) — валиден, копируется, хешируется ────────────────
#[tokio::test]
async fn empty_file_is_copied_and_hashed() {
    let src = tempfile::tempdir().unwrap();
    let dst = tempfile::tempdir().unwrap();
    write_file(&src.path().join("empty.wav"), b"");
    write_file(&src.path().join("data.bin"), &pseudo_bytes(7, 1000));

    let request = req(src.path(), &[dst.path().to_path_buf()], None, true);
    let summary = run(&request).await.unwrap();
    assert_eq!(summary.copied, 2);
    assert_eq!(summary.failed, 0);

    let e = dst.path().join("empty.wav");
    assert!(e.exists());
    assert_eq!(std::fs::metadata(&e).unwrap().len(), 0);

    let report = verify_mhl_path(dst.path(), MhlVerifyOptions::default()).unwrap();
    assert!(
        report.summary.success,
        "empty-file MHL verify: {:?}",
        report.issues
    );
}

// ── 6. Один гигант (> буфера) + триста мелких — граница буфера и цикл ────────
#[tokio::test]
async fn huge_and_many_tiny_mix_roundtrip() {
    let src = tempfile::tempdir().unwrap();
    let dst = tempfile::tempdir().unwrap();

    // 9 МБ > 4 МБ буфера → минимум три chunk-read.
    write_file(
        &src.path().join("BIG/master.mov"),
        &pseudo_bytes(42, 9 * 1024 * 1024),
    );
    for i in 0..300 {
        write_file(
            &src.path().join(format!("small/f{i:03}.dat")),
            &pseudo_bytes(1000 + i, 1),
        );
    }

    let request = req(src.path(), &[dst.path().to_path_buf()], None, false);
    let summary = run(&request).await.unwrap();
    assert_eq!(summary.total_files, 301);
    assert_eq!(summary.copied, 301);
    assert_eq!(summary.failed, 0);
    assert_identical(
        &src.path().join("BIG/master.mov"),
        &dst.path().join("BIG/master.mov"),
    );
    for i in [0usize, 150, 299] {
        assert_identical(
            &src.path().join(format!("small/f{i:03}.dat")),
            &dst.path().join(format!("small/f{i:03}.dat")),
        );
    }
}

// ── 7. TEE-ИНВАРИАНТ: inline-хеш == хеш файла на диске == хеш источника ──────
// Ядро всего движка. Читаем источник ОДИН раз; сойтись обязано на каждом диске.
#[tokio::test]
async fn tee_hash_matches_bytes_on_every_disk() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("take.r3d");
    let payload = pseudo_bytes(99, 5 * 1024 * 1024 + 777); // не кратно буферу
    write_file(&source, &payload);

    let dests = vec![
        dir.path().join("d1/take.r3d"),
        dir.path().join("d2/take.r3d"),
    ];
    let config = CopyEngineConfig {
        hash_algorithms: vec![HashAlgorithm::XXH64, HashAlgorithm::SHA256],
        ..Default::default()
    };
    let results = copy_file_multi(&source, &dests, &config, &CopyControl::none())
        .await
        .unwrap();

    // Эталон — независимый хеш исходных байт.
    let source_hash = hash_map(&hash_bytes(
        &payload,
        &[HashAlgorithm::XXH64, HashAlgorithm::SHA256],
    ));

    assert_eq!(results.len(), 2);
    for res in &results {
        assert!(res.success && !res.skipped);
        // inline-хеш из потока == хеш источника
        assert_eq!(
            hash_map(&res.hash_results),
            source_hash,
            "inline hash diverged from source"
        );
        // и то, что реально на диске, == тот же хеш
        let on_disk = std::fs::read(&res.dest_path).unwrap();
        let disk_hash = hash_map(&hash_bytes(
            &on_disk,
            &[HashAlgorithm::XXH64, HashAlgorithm::SHA256],
        ));
        assert_eq!(
            disk_hash, source_hash,
            "bytes on disk diverged from hashed stream"
        );
    }
}

// ── 8. Тихая порча байта в получателе — MHL обязан поймать ───────────────────
#[tokio::test]
async fn silent_corruption_caught_by_mhl_verify() {
    let src = tempfile::tempdir().unwrap();
    let dst = tempfile::tempdir().unwrap();
    write_file(&src.path().join("A/clip.mov"), &pseudo_bytes(11, 50_000));
    write_file(&src.path().join("A/audio.wav"), &pseudo_bytes(12, 30_000));

    let request = req(src.path(), &[dst.path().to_path_buf()], None, true);
    run(&request).await.unwrap();

    // Чистая копия → verify PASS.
    let clean = verify_mhl_path(dst.path(), MhlVerifyOptions::default()).unwrap();
    assert!(
        clean.summary.success,
        "fresh copy must verify: {:?}",
        clean.issues
    );

    // Портим один байт (тот же размер — sneaky bit-rot).
    let victim = dst.path().join("A/clip.mov");
    let mut bytes = std::fs::read(&victim).unwrap();
    bytes[0] ^= 0xFF;
    std::fs::write(&victim, &bytes).unwrap();

    let tampered = verify_mhl_path(dst.path(), MhlVerifyOptions::default()).unwrap();
    assert!(!tampered.summary.success, "MHL must reject a flipped byte");
    assert!(
        tampered.summary.failed >= 1,
        "corrupted file must be reported failed"
    );
}

// ── 9. Совпал размер, но другое содержимое — обязан ПЕРЕЗАПИСАТЬ, не skip ────
// «Размер сам по себе никогда не доказательство целостности» (FileConflictPolicy).
#[tokio::test]
async fn same_size_different_content_overwrites() {
    let dir = tempfile::tempdir().unwrap();
    let source = dir.path().join("clip.mov");
    let payload = pseudo_bytes(21, 8192);
    write_file(&source, &payload);

    // Заранее кладём в dest файл ТОГО ЖЕ размера, но другой.
    let dest = dir.path().join("out/clip.mov");
    write_file(&dest, &pseudo_bytes(22, 8192));
    assert_eq!(
        std::fs::metadata(&dest).unwrap().len(),
        payload.len() as u64
    );

    let config = CopyEngineConfig::default(); // SkipIfVerified
    let results = copy_file_multi(
        &source,
        std::slice::from_ref(&dest),
        &config,
        &CopyControl::none(),
    )
    .await
    .unwrap();

    assert!(results[0].success);
    assert!(
        !results[0].skipped,
        "different content must NOT be skipped on size match"
    );
    assert_identical(&source, &dest);
}

// ── 10. Крах на середине → перезапуск не перекопирует уже проверенное ────────
// Симуляция выдёргивания питания/карты: cancel взводится на 2-м файле. Джоб
// падает terminated, 1-й файл уже целиком на диске. Повторный прогон (как после
// перезапуска приложения) пропускает верифицированный 1-й и докопирует остальные —
// без потери данных и без повторного копирования гигабайтов.
//
// В ArchiveMax пропущенные после resume файлы независимо перечитываются и входят
// в итоговую ASC MHL generation. Этот legacy Fast-тест проверяет только безопасное
// продолжение tee-copy без повторной записи готового файла.
#[tokio::test]
async fn crash_midway_then_resume_skips_verified() {
    use std::sync::atomic::{AtomicBool, Ordering};
    let src = tempfile::tempdir().unwrap();
    let dst = tempfile::tempdir().unwrap();
    let dbdir = tempfile::tempdir().unwrap();
    let db = dbdir.path().join("offload.sqlite");

    // Имена дают детерминированный порядок скана: 01 → 02 → 03.
    write_file(&src.path().join("01.mov"), &pseudo_bytes(31, 20_000));
    write_file(&src.path().join("02.mov"), &pseudo_bytes(32, 20_000));
    write_file(&src.path().join("03.mov"), &pseudo_bytes(33, 20_000));

    // ── Прогон 1: «крах» на 2-м файле ──
    let cancel = AtomicBool::new(false);
    let progress = |p: crate::offload::orchestrator::OffloadProgress| {
        if p.phase == "copying" && p.file_index >= 2 {
            cancel.store(true, Ordering::SeqCst);
        }
    };
    let req1 = req(
        src.path(),
        &[dst.path().to_path_buf()],
        Some(db.clone()),
        true,
    );
    let r1 = run_offload(&req1, Some(&cancel), None, &progress).await;
    assert!(
        r1.is_err(),
        "cancel mid-job must surface as terminated error"
    );

    // 1-й файл успел лечь целиком; 3-й — нет. MHL-фаза до краха не дошла.
    assert_identical(&src.path().join("01.mov"), &dst.path().join("01.mov"));
    assert!(
        !dst.path().join("03.mov").exists(),
        "file after crash point must be absent"
    );

    // ── Прогон 2: перезапуск после краха, без cancel ──
    let req2 = req(
        src.path(),
        &[dst.path().to_path_buf()],
        Some(db.clone()),
        true,
    );
    let s2 = run(&req2).await.unwrap();
    assert_eq!(s2.failed, 0, "resume must finish cleanly");
    assert!(
        s2.skipped >= 1,
        "already-verified file 01 must be skipped, not recopied"
    );
    assert_eq!(s2.copied + s2.skipped, 3);
    for rel in ["01.mov", "02.mov", "03.mov"] {
        assert_identical(&src.path().join(rel), &dst.path().join(rel));
    }
    // MHL этой генерации (02,03) обязан пройти собственную верификацию.
    let report = verify_mhl_path(dst.path(), MhlVerifyOptions::default()).unwrap();
    assert!(
        report.summary.success,
        "post-resume MHL: {:?}",
        report.issues
    );
}

// ── Property-тест: рандомное дерево → offload → ASC MHL verify roundtrip ─────
// proptest (сторонний, стандарт фаззинга в Rust) гоняет десятки случайных деревьев
// (глубина / размеры 0..8К / seed) и на каждом требует: все файлы байт-в-байт +
// MHL проходит собственную верификацию. Ловит структурные баги, которые точечные
// тесты пропускают.
mod prop {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig { cases: 48, ..ProptestConfig::default() })]

        #[test]
        fn random_tree_offload_mhl_roundtrip(
            files in proptest::collection::vec((0usize..3usize, 0u64..8192u64, any::<u8>()), 1..12)
        ) {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(async {
                let src = tempfile::tempdir().unwrap();
                let dst = tempfile::tempdir().unwrap();

                // Уникальные пути: d{0..2}/f{idx}.dat (idx уникален → нет коллизий).
                let mut rels = Vec::new();
                for (idx, (sub, size, seed)) in files.iter().enumerate() {
                    let rel = format!("d{sub}/f{idx}.dat");
                    write_file(
                        &src.path().join(&rel),
                        &pseudo_bytes(*seed as u64 + idx as u64 * 131, *size as usize),
                    );
                    rels.push(rel);
                }

                let request = req(src.path(), &[dst.path().to_path_buf()], None, true);
                let summary = run(&request).await.unwrap();
                prop_assert_eq!(summary.failed, 0);
                prop_assert_eq!(summary.copied, rels.len());

                for rel in &rels {
                    let a = std::fs::read(src.path().join(rel)).unwrap();
                    let b = std::fs::read(dst.path().join(rel)).unwrap();
                    prop_assert_eq!(a, b, "byte mismatch for {}", rel);
                }
                let report = verify_mhl_path(dst.path(), MhlVerifyOptions::default()).unwrap();
                prop_assert!(
                    report.summary.success,
                    "MHL roundtrip failed: {:?}",
                    report.issues
                );
                Ok(())
            })?;
        }
    }
}
