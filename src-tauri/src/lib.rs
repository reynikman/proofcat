// ProofCat — локальный инспектор метаданных.
// Зовёт нативные CLI: mediainfo, exiftool, ffprobe. Ничего не шлётся в сеть.

// Offload mode — движок слива карт (портирован из DIT-Pro, MIT — см. NOTICE).
pub mod crash;
pub use offload_core::offload;

use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;
use tauri_plugin_opener::OpenerExt;

const PROOFCAT_LOG_STEM: &str = "proofcat";
const PROOFCAT_LOG_FILE: &str = "proofcat.log";
const LEGACY_META_REPORT_LOG_FILE: &str = "meta-report.log";
const PROOFCAT_EVIDENCE_DIR: &str = "proofcat-evidence";

#[derive(serde::Serialize)]
struct ToolOut {
    ok: bool,
    output: String,
    error: String,
}

#[derive(serde::Serialize)]
struct Report {
    path: String,
    filename: String,
    size_bytes: u64,
    // JSON для построения сводки (не показывается сырым)
    summary_mi_json: String,
    summary_exif_json: String,
    // Читаемый текст для вкладок
    mediainfo: ToolOut,      // чистый (дефолт)
    mediainfo_full: ToolOut, // полный (-f)
    exiftool: ToolOut,
    ffprobe: ToolOut,
}

/// Ищет бинарь в типовых местах (GUI-процесс не видит brew в PATH), иначе — голое имя из PATH.
fn resolve_bin(name: &str) -> String {
    let mut cands: Vec<String> = Vec::new();
    #[cfg(target_os = "macos")]
    {
        cands.push(format!("/opt/homebrew/bin/{name}"));
        cands.push(format!("/usr/local/bin/{name}"));
    }
    #[cfg(target_os = "windows")]
    {
        cands.push(format!("C:\\Program Files\\{name}\\{name}.exe"));
    }
    for c in &cands {
        if Path::new(c).exists() {
            return c.clone();
        }
    }
    name.to_string()
}

/// Разрешённая утилита: программа для запуска + базовые аргументы (для exiftool через perl).
struct Tool {
    program: String,
    base: Vec<String>,
}

/// Предпочитает вшитые бинарники из ресурсов бандла, иначе — системные (dev/brew).
fn resolve_tool(tools_dir: &Option<PathBuf>, name: &str) -> Tool {
    if let Some(dir) = tools_dir {
        // Windows: самодостаточные exe (perl не нужен), плоская раскладка tools\<name>.exe
        #[cfg(target_os = "windows")]
        {
            let p = dir.join(format!("{name}.exe"));
            if p.exists() {
                return Tool {
                    program: p.to_string_lossy().into_owned(),
                    base: vec![],
                };
            }
        }
        // macOS/Linux: раскладка с dylib и exiftool через системный perl
        #[cfg(not(target_os = "windows"))]
        match name {
            "mediainfo" => {
                let p = dir.join("mediainfo");
                if p.exists() {
                    return Tool {
                        program: p.to_string_lossy().into_owned(),
                        base: vec![],
                    };
                }
            }
            "ffprobe" => {
                let p = dir.join("ff/ffprobe");
                if p.exists() {
                    return Tool {
                        program: p.to_string_lossy().into_owned(),
                        base: vec![],
                    };
                }
            }
            "ffmpeg" => {
                let p = dir.join("ff/ffmpeg");
                if p.exists() {
                    return Tool {
                        program: p.to_string_lossy().into_owned(),
                        base: vec![],
                    };
                }
            }
            "exiftool" => {
                let script = dir.join("exiftool/exiftool");
                let lib = dir.join("exiftool/lib");
                if script.exists() {
                    return Tool {
                        program: "/usr/bin/perl".into(),
                        base: vec![
                            format!("-I{}", lib.to_string_lossy()),
                            script.to_string_lossy().into_owned(),
                        ],
                    };
                }
            }
            _ => {}
        }
    }
    Tool {
        program: resolve_bin(name),
        base: vec![],
    }
}

fn run_tool(program: &str, args: &[String]) -> ToolOut {
    let started = std::time::Instant::now();
    match Command::new(program).args(args).output() {
        Ok(o) => {
            let stdout = String::from_utf8_lossy(&o.stdout).into_owned();
            let stderr = String::from_utf8_lossy(&o.stderr).into_owned();
            // Утилиты иногда возвращают ненулевой код, но валидный вывод — считаем ok, если stdout непустой.
            let ok = o.status.success() || !stdout.trim().is_empty();
            log::info!(
                "tool prog={program} exit={:?} out_len={} took_ms={} ok={ok}",
                o.status.code(),
                stdout.len(),
                started.elapsed().as_millis()
            );
            if ok {
                ToolOut {
                    ok: true,
                    output: stdout,
                    error: stderr,
                }
            } else {
                let err = if stderr.trim().is_empty() {
                    format!("exit status {}", o.status)
                } else {
                    stderr
                };
                log::warn!(
                    "tool prog={program} failed: {}",
                    err.chars().take(200).collect::<String>()
                );
                ToolOut {
                    ok: false,
                    output: stdout,
                    error: err,
                }
            }
        }
        Err(e) => {
            log::error!("tool spawn failed prog={program}: {e}");
            ToolOut {
                ok: false,
                output: String::new(),
                error: format!("не удалось запустить `{program}`: {e}"),
            }
        }
    }
}

#[tauri::command]
fn analyze_file(app: tauri::AppHandle, path: String) -> Result<Report, String> {
    let p = Path::new(&path);
    if !p.exists() {
        log::warn!("analyze: file not found {path:?}");
        return Err(format!("Файл не найден: {path}"));
    }
    let filename = p
        .file_name()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let size_bytes = std::fs::metadata(p).map(|m| m.len()).unwrap_or(0);
    log::info!("analyze file={filename:?} size={size_bytes}");

    // Вшитые утилиты из ресурсов бандла; в dev падаем на системные.
    let tools_dir = app.path().resource_dir().ok().map(|d| d.join("tools"));
    let mi = resolve_tool(&tools_dir, "mediainfo");
    let et = resolve_tool(&tools_dir, "exiftool");
    let fp = resolve_tool(&tools_dir, "ffprobe");
    let args = |t: &Tool, extra: &[&str]| -> Vec<String> {
        let mut v = t.base.clone();
        v.extend(extra.iter().map(|s| s.to_string()));
        v
    };

    // JSON — только для сводки-карточек.
    let summary_mi_json = run_tool(&mi.program, &args(&mi, &["--Output=JSON", &path])).output;
    let summary_exif_json =
        run_tool(&et.program, &args(&et, &["-j", "-G", "-struct", &path])).output;

    // Читаемый текст — то, что видно во вкладках.
    let mediainfo = run_tool(&mi.program, &args(&mi, &[&path])); // чистый дефолтный вид
    let mediainfo_full = run_tool(&mi.program, &args(&mi, &["-f", &path])); // полный
    let exiftool = run_tool(&et.program, &args(&et, &["-G1", "-a", "-u", &path]));
    let ffprobe = run_tool(
        &fp.program,
        &args(
            &fp,
            &[
                "-hide_banner",
                "-print_format",
                "json",
                "-show_format",
                "-show_streams",
                &path,
            ],
        ),
    );

    Ok(Report {
        path,
        filename,
        size_bytes,
        summary_mi_json,
        summary_exif_json,
        mediainfo,
        mediainfo_full,
        exiftool,
        ffprobe,
    })
}

/// Нативный диалог выбора файлов (для тех, кто не перетаскивает).
#[tauri::command]
async fn pick_files(app: tauri::AppHandle) -> Vec<String> {
    // Колбэк-форма вместо blocking_pick_files: панель идёт через event-loop главного
    // потока. blocking_* в небандленном dev-бинарнике даёт NSOpenPanel nil + RecvError.
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog().file().pick_files(move |files| {
        let _ = tx.send(files);
    });
    let paths: Vec<String> = rx
        .await
        .ok()
        .flatten()
        .map(|files| {
            files
                .into_iter()
                .filter_map(|f| f.into_path().ok())
                .map(|p| p.to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    log::info!("pick_files -> {} file(s)", paths.len());
    paths
}

/// Сохраняет markdown-отчёт через нативный диалог. Возвращает путь или None (отмена).
#[tauri::command]
async fn save_report(
    app: tauri::AppHandle,
    default_name: String,
    contents: String,
) -> Result<Option<String>, String> {
    // Колбэк-форма вместо blocking_save_file (см. pick_files: избегаем nil/RecvError).
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog()
        .file()
        .add_filter("Markdown", &["md"])
        .set_file_name(&default_name)
        .save_file(move |p| {
            let _ = tx.send(p);
        });
    let picked = rx.await.ok().flatten();

    match picked {
        Some(fp) => {
            let path = fp.into_path().map_err(|e| e.to_string())?;
            std::fs::write(&path, contents).map_err(|e| e.to_string())?;
            log::info!("report saved to {path:?}");
            Ok(Some(path.to_string_lossy().into_owned()))
        }
        None => {
            log::debug!("save cancelled");
            Ok(None)
        }
    }
}

/// Логирование событий из UI (действия, смена языка и т.п.).
#[tauri::command]
fn log_event(level: String, message: String) {
    match level.as_str() {
        "error" => log::error!("[ui] {message}"),
        "warn" => log::warn!("[ui] {message}"),
        _ => log::info!("[ui] {message}"),
    }
}

/// Открывает URL (mailto, папка логов) через системный обработчик.
#[tauri::command]
fn open_url(app: tauri::AppHandle, url: String) -> Result<(), String> {
    log::info!("open_url {}", url.chars().take(60).collect::<String>());
    app.opener()
        .open_url(url, None::<&str>)
        .map_err(|e| e.to_string())
}

#[derive(serde::Serialize)]
struct FeedbackInfo {
    version: String,
    os: String,
    log_dir: String,
    logs: String,
}

/// Current releases write `proofcat.log`. Keep the legacy filename as a
/// read-only fallback so an in-place upgrade does not hide a useful pre-rename
/// diagnostic from the feedback screen.
fn feedback_log_tail(log_dir: &Path) -> String {
    [PROOFCAT_LOG_FILE, LEGACY_META_REPORT_LOG_FILE]
        .into_iter()
        .find_map(|file_name| std::fs::read_to_string(log_dir.join(file_name)).ok())
        .map(|content| {
            if content.len() > 4000 {
                let mut idx = content.len() - 4000;
                while !content.is_char_boundary(idx) {
                    idx += 1;
                }
                format!("…\n{}", &content[idx..])
            } else {
                content
            }
        })
        .unwrap_or_default()
}

fn offload_evidence_file_name(job_id: &str, extension: &str) -> String {
    format!("proofcat-{}.{extension}", job_id.replace(':', "-"))
}

/// True if a replica file lives under this destination volume. Matches by path
/// components (not a raw string prefix) so `/Volumes/CARD` does not swallow
/// `/Volumes/CARD2` when deciding which destination's success gates its evidence.
fn replica_belongs_to_destination(replica_destination: &str, destination_path: &str) -> bool {
    Path::new(replica_destination).starts_with(destination_path)
}

fn destination_completed_successfully(
    summary: &offload::orchestrator::OffloadSummary,
    destination_path: &str,
) -> bool {
    let replicas = summary
        .replicas
        .iter()
        .filter(|replica| replica_belongs_to_destination(&replica.destination, destination_path))
        .collect::<Vec<_>>();
    !replicas.is_empty()
        && replicas.iter().all(|replica| {
            matches!(
                replica.status,
                offload::orchestrator::ReplicaState::Verified
                    | offload::orchestrator::ReplicaState::CopyComplete
                    | offload::orchestrator::ReplicaState::AlreadyMatched
            )
        })
}

async fn write_evidence_file(path: &Path, contents: &str) -> Result<(), String> {
    let mut writer = offload::copy_engine::atomic_writer::AtomicWriter::new(path)
        .await
        .map_err(|error| format!("{error:#}"))?;
    writer
        .write(contents.as_bytes())
        .await
        .map_err(|error| format!("{error:#}"))?;
    writer
        .finalize()
        .await
        .map_err(|error| format!("{error:#}"))
}

/// JSON is the canonical immutable job snapshot. Keep it beside every
/// successful destination so verification evidence travels with the archive.
/// A sidecar write must not change the verified-copy verdict: the durable
/// checkpoint and ASC MHL were already committed by the offload engine.
async fn write_canonical_evidence_to_destinations(
    summary: &offload::orchestrator::OffloadSummary,
) -> Vec<String> {
    let contents = match offload::report::render(summary, offload::report::ReportFormat::Json) {
        Ok(contents) => contents,
        Err(error) => return vec![format!("Could not render canonical JSON: {error:#}")],
    };
    let name = offload_evidence_file_name(&summary.job_id, "json");
    let mut errors = Vec::new();
    for destination in &summary.destination_volumes {
        if !destination_completed_successfully(summary, &destination.path) {
            continue;
        }
        let path = PathBuf::from(&destination.path)
            .join(PROOFCAT_EVIDENCE_DIR)
            .join(&name);
        if let Err(error) = write_evidence_file(&path, &contents).await {
            errors.push(format!("{}: {error}", path.display()));
        }
    }
    errors
}

/// Собирает данные для фидбека: версия, ОС, путь к логам, хвост лог-файла.
#[tauri::command]
fn collect_feedback(app: tauri::AppHandle) -> FeedbackInfo {
    let version = app.package_info().version.to_string();
    let os = format!("{} {}", std::env::consts::OS, std::env::consts::ARCH);
    let dir = app.path().app_log_dir().ok();
    let log_dir = dir
        .as_ref()
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_default();
    let logs = dir.as_deref().map(feedback_log_tail).unwrap_or_default();
    log::info!("feedback collected (logs {} chars)", logs.len());
    FeedbackInfo {
        version,
        os,
        log_dir,
        logs,
    }
}

#[derive(serde::Serialize)]
struct LoudnessOut {
    ok: bool,
    integrated: Option<f64>, // LUFS (EBU R128)
    true_peak: Option<f64>,  // dBTP
    lra: Option<f64>,        // LU
    raw: String,             // ffmpeg ebur128 summary block
    error: String,
}

/// Достаёт число из строки ebur128-сводки вида `I:  -14.5 LUFS`.
/// Ключ должен стоять в начале строки (после trim), а `unit` — присутствовать в ней,
/// чтобы `LRA:` не путался с `LRA low:` / `LRA high:`.
fn parse_metric(text: &str, key: &str, unit: &str) -> Option<f64> {
    for line in text.lines() {
        let l = line.trim();
        if let Some(rest) = l.strip_prefix(key) {
            if !l.contains(unit) {
                continue;
            }
            if let Some(tok) = rest.split_whitespace().next() {
                if let Ok(n) = tok.parse::<f64>() {
                    return Some(n);
                }
            }
        }
    }
    None
}

/// Измеряет громкость по EBU R128 через ffmpeg (фильтр ebur128). Читает весь звук —
/// операция небыстрая, поэтому вызывается кнопкой, а не при анализе.
#[tauri::command]
fn measure_loudness(app: tauri::AppHandle, path: String) -> LoudnessOut {
    let none = |err: String| LoudnessOut {
        ok: false,
        integrated: None,
        true_peak: None,
        lra: None,
        raw: String::new(),
        error: err,
    };
    if !Path::new(&path).exists() {
        return none(format!("Файл не найден: {path}"));
    }
    let tools_dir = app.path().resource_dir().ok().map(|d| d.join("tools"));
    let fm = resolve_tool(&tools_dir, "ffmpeg");
    let mut args = fm.base.clone();
    for a in [
        "-hide_banner",
        "-nostats",
        "-i",
        &path,
        "-map",
        "0:a:0",
        "-af",
        "ebur128=peak=true",
        "-f",
        "null",
        "-",
    ] {
        args.push(a.to_string());
    }
    let started = std::time::Instant::now();
    match Command::new(&fm.program).args(&args).output() {
        Ok(o) => {
            // ebur128 пишет сводку в stderr.
            let stderr = String::from_utf8_lossy(&o.stderr).into_owned();
            let integrated = parse_metric(&stderr, "I:", "LUFS");
            let true_peak = parse_metric(&stderr, "Peak:", "dBFS");
            let lra = parse_metric(&stderr, "LRA:", "LU");
            let raw = match stderr.rfind("Summary:") {
                Some(i) => stderr[i..].trim().to_string(),
                None => stderr
                    .lines()
                    .rev()
                    .take(6)
                    .collect::<Vec<_>>()
                    .into_iter()
                    .rev()
                    .collect::<Vec<_>>()
                    .join("\n"),
            };
            let ok = integrated.is_some();
            log::info!(
                "loudness ok={ok} I={integrated:?} TP={true_peak:?} LRA={lra:?} took_ms={}",
                started.elapsed().as_millis()
            );
            if ok {
                LoudnessOut {
                    ok: true,
                    integrated,
                    true_peak,
                    lra,
                    raw,
                    error: String::new(),
                }
            } else {
                let msg = stderr
                    .lines()
                    .rev()
                    .find(|l| {
                        l.contains("Error")
                            || l.contains("Invalid")
                            || l.contains("matches no streams")
                    })
                    .map(|l| l.trim().to_string())
                    .unwrap_or_else(|| format!("ffmpeg exit {}", o.status));
                none(msg)
            }
        }
        Err(e) => none(format!("не удалось запустить ffmpeg: {e}")),
    }
}

// ======================= checksum (SHA-256) =======================
#[derive(serde::Serialize)]
struct HashOut {
    ok: bool,
    algo: String,
    hash: String,
    error: String,
}

/// SHA-256 файла потоковым чтением (не грузит весь файл в память) — для манифеста сдачи.
/// Медленно на больших файлах, поэтому вызывается кнопкой, а не при анализе.
#[tauri::command]
fn hash_file(path: String) -> HashOut {
    use sha2::{Digest, Sha256};
    use std::io::Read;
    let none = |e: String| HashOut {
        ok: false,
        algo: "SHA-256".into(),
        hash: String::new(),
        error: e,
    };
    let p = Path::new(&path);
    if !p.exists() {
        return none(format!("Файл не найден: {path}"));
    }
    let mut file = match std::fs::File::open(p) {
        Ok(f) => f,
        Err(e) => return none(e.to_string()),
    };
    let mut hasher = Sha256::new();
    let mut buf = vec![0u8; 1 << 20]; // 1 МБ
    let started = std::time::Instant::now();
    loop {
        match file.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => hasher.update(&buf[..n]),
            Err(e) => return none(e.to_string()),
        }
    }
    let hash = format!("{:x}", hasher.finalize());
    log::info!("hash_file done took_ms={}", started.elapsed().as_millis());
    HashOut {
        ok: true,
        algo: "SHA-256".into(),
        hash,
        error: String::new(),
    }
}

// ======================= frame scan (чёрные / замёрзшие кадры) =======================
#[derive(serde::Serialize)]
struct FrameSeg {
    kind: String, // "black" | "freeze"
    start: f64,
    end: Option<f64>,
}
#[derive(serde::Serialize)]
struct ScanOut {
    ok: bool,
    segments: Vec<FrameSeg>,
    error: String,
}

/// Число после ключа `key` в строке лога ffmpeg (напр. `black_start:12.5`).
fn parse_after(s: &str, key: &str) -> Option<f64> {
    let rest = s.strip_prefix(key)?.trim_start();
    let tok = rest.split_whitespace().next()?;
    tok.trim_end_matches(|c: char| !c.is_ascii_digit() && c != '.')
        .parse::<f64>()
        .ok()
}

/// Ищет чёрные (blackdetect) и замёрзшие (freezedetect) сегменты через ffmpeg. Декодирует
/// всё видео — операция небыстрая, поэтому вызывается кнопкой.
#[tauri::command]
fn scan_frames(app: tauri::AppHandle, path: String) -> ScanOut {
    let none = |e: String| ScanOut {
        ok: false,
        segments: vec![],
        error: e,
    };
    if !Path::new(&path).exists() {
        return none(format!("Файл не найден: {path}"));
    }
    let tools_dir = app.path().resource_dir().ok().map(|d| d.join("tools"));
    let fm = resolve_tool(&tools_dir, "ffmpeg");
    let mut args = fm.base.clone();
    for a in [
        "-hide_banner",
        "-nostats",
        "-i",
        &path,
        "-map",
        "0:v:0",
        "-vf",
        "blackdetect=d=0.1:pic_th=0.98,freezedetect=n=-60dB:d=0.3",
        "-an",
        "-f",
        "null",
        "-",
    ] {
        args.push(a.to_string());
    }
    let started = std::time::Instant::now();
    let out = match Command::new(&fm.program).args(&args).output() {
        Ok(o) => o,
        Err(e) => return none(format!("не удалось запустить ffmpeg: {e}")),
    };
    let stderr = String::from_utf8_lossy(&out.stderr);
    let mut segs: Vec<FrameSeg> = Vec::new();
    let mut freeze_starts: Vec<f64> = Vec::new();
    for line in stderr.lines() {
        if let Some(i) = line.find("black_start:") {
            let start = parse_after(&line[i..], "black_start:");
            let end = line
                .find("black_end:")
                .and_then(|j| parse_after(&line[j..], "black_end:"));
            if let Some(st) = start {
                segs.push(FrameSeg {
                    kind: "black".into(),
                    start: st,
                    end,
                });
            }
        }
        if let Some(i) = line.find("freeze_start:") {
            if let Some(st) = parse_after(&line[i..], "freeze_start:") {
                freeze_starts.push(st);
            }
        }
        if let Some(i) = line.find("freeze_end:") {
            if let Some(en) = parse_after(&line[i..], "freeze_end:") {
                let st = freeze_starts.pop().unwrap_or(en);
                segs.push(FrameSeg {
                    kind: "freeze".into(),
                    start: st,
                    end: Some(en),
                });
            }
        }
    }
    // freeze без end — держится до конца файла
    for st in freeze_starts {
        segs.push(FrameSeg {
            kind: "freeze".into(),
            start: st,
            end: None,
        });
    }
    segs.sort_by(|a, b| {
        a.start
            .partial_cmp(&b.start)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    log::info!(
        "scan_frames done segs={} took_ms={}",
        segs.len(),
        started.elapsed().as_millis()
    );
    ScanOut {
        ok: true,
        segments: segs,
        error: String::new(),
    }
}

#[derive(serde::Serialize)]
struct UpdateInfo {
    available: bool,
    version: String, // новая версия (или текущая, если обновлений нет)
    current: String,
    notes: String,
}

/// Тихая проверка обновления. Сеть трогается только при вызове (кнопка / старт).
#[tauri::command]
async fn check_update(app: tauri::AppHandle) -> Result<UpdateInfo, String> {
    use tauri_plugin_updater::UpdaterExt;
    let current = app.package_info().version.to_string();
    let updater = app.updater().map_err(|e| e.to_string())?;
    match updater.check().await {
        Ok(Some(update)) => {
            log::info!("update available: {} (current {current})", update.version);
            Ok(UpdateInfo {
                available: true,
                version: update.version.clone(),
                current,
                notes: update.body.clone().unwrap_or_default(),
            })
        }
        Ok(None) => {
            log::info!("no update (current {current})");
            Ok(UpdateInfo {
                available: false,
                version: current.clone(),
                current,
                notes: String::new(),
            })
        }
        Err(e) => {
            log::warn!("update check failed: {e}");
            Err(e.to_string())
        }
    }
}

/// Скачивает и ставит обновление, затем перезапускает приложение.
#[tauri::command]
async fn install_update(app: tauri::AppHandle) -> Result<(), String> {
    use tauri_plugin_updater::UpdaterExt;
    let updater = app.updater().map_err(|e| e.to_string())?;
    match updater.check().await.map_err(|e| e.to_string())? {
        Some(update) => {
            log::info!("installing update {}", update.version);
            update
                .download_and_install(
                    |_chunk, _total| {},
                    || log::info!("update download finished"),
                )
                .await
                .map_err(|e| e.to_string())?;
            log::info!("update installed, restarting");
            app.restart();
        }
        None => Err("нет доступного обновления".into()),
    }
}

// ======================= DCP (Digital Cinema Package) =======================
// DCP — это ПАПКА (ASSETMAP + CPL.xml + PKL.xml + VOLINDEX + *.mxf), не один файл.
// Rust только извлекает сырые факты; разбор по DCNC и правила DCI/Netflix — в JS.

#[derive(serde::Serialize, Default)]
struct DcpFile {
    name: String,
    kind: String, // ASSETMAP | VOLINDEX | CPL | PKL | MXF | other
    size: u64,
}

#[derive(serde::Serialize, Default)]
struct DcpReel {
    edit_rate: Option<String>,
    fps: Option<f64>,
    duration_frames: Option<i64>,
    duration_sec: Option<f64>,
    aspect: Option<String>,
}

#[derive(serde::Serialize, Default)]
struct DcpCpl {
    filename: String,
    content_title: String,
    annotation: Option<String>,
    issue_date: Option<String>,
    creator: Option<String>,
    standard: String, // "SMPTE" | "Interop" | ""
    encrypted: bool,
    reels: Vec<DcpReel>,
}

#[derive(serde::Serialize, Default)]
struct DcpPicture {
    width: Option<i64>,
    height: Option<i64>,
    frame_rate: Option<String>,
    format: Option<String>,
    color_space: Option<String>,
    bit_depth: Option<String>,
    bit_rate: Option<String>,
}

#[derive(serde::Serialize, Default)]
struct DcpAudio {
    channels: Option<i64>,
    sample_rate: Option<String>,
    bit_depth: Option<String>,
    format: Option<String>,
}

#[derive(serde::Serialize, Default)]
struct DcpReport {
    path: String,
    folder_name: String,
    is_dcp: bool,
    files: Vec<DcpFile>,
    has_assetmap: bool,
    has_volindex: bool,
    has_pkl: bool,
    cpl_count: usize,
    mxf_count: usize,
    total_size: u64,
    cpls: Vec<DcpCpl>,
    picture: Option<DcpPicture>,
    audio: Option<DcpAudio>,
}

/// Текст первого элемента с данным локальным именем (игнорируя namespace).
fn xml_text(doc: &roxmltree::Document, name: &str) -> Option<String> {
    doc.descendants()
        .find(|n| n.is_element() && n.tag_name().name() == name)
        .and_then(|n| n.text())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Собирает DcpCpl из уже распарсенного документа CPL.
fn cpl_from_doc(doc: &roxmltree::Document, path: &Path) -> DcpCpl {
    let ns = doc.root_element().tag_name().namespace().unwrap_or("");
    let standard = if ns.to_lowercase().contains("smpte") {
        "SMPTE"
    } else if ns.contains("digicine") || ns.contains("PROTO") {
        "Interop"
    } else {
        ""
    };
    let encrypted = doc
        .descendants()
        .any(|n| n.is_element() && n.tag_name().name() == "KeyId");

    let mut reels = Vec::new();
    for reel in doc
        .descendants()
        .filter(|n| n.is_element() && n.tag_name().name() == "Reel")
    {
        let pic = reel.descendants().find(|n| {
            n.is_element()
                && matches!(
                    n.tag_name().name(),
                    "MainPicture" | "MainStereoscopicPicture" | "Picture"
                )
        });
        let mut r = DcpReel::default();
        if let Some(p) = pic {
            let child = |nm: &str| {
                p.descendants()
                    .find(|n| n.is_element() && n.tag_name().name() == nm)
                    .and_then(|n| n.text())
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
            };
            r.aspect = child("ScreenAspectRatio");
            let dur = child("Duration").or_else(|| child("IntrinsicDuration"));
            r.duration_frames = dur
                .as_deref()
                .and_then(|s| s.split_whitespace().next())
                .and_then(|s| s.parse::<i64>().ok());
            let edit = child("EditRate").or_else(|| child("FrameRate"));
            if let Some(er) = edit {
                let parts: Vec<f64> = er
                    .split_whitespace()
                    .filter_map(|x| x.parse::<f64>().ok())
                    .collect();
                let fps = match parts.as_slice() {
                    [a, b] if *b != 0.0 => Some(a / b),
                    [a] => Some(*a),
                    _ => None,
                };
                r.edit_rate = Some(er);
                r.fps = fps;
                if let (Some(df), Some(fp)) = (r.duration_frames, fps) {
                    if fp > 0.0 {
                        r.duration_sec = Some(df as f64 / fp);
                    }
                }
            }
        }
        reels.push(r);
    }

    DcpCpl {
        filename: path
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default(),
        content_title: xml_text(doc, "ContentTitleText").unwrap_or_default(),
        annotation: xml_text(doc, "AnnotationText"),
        issue_date: xml_text(doc, "IssueDate"),
        creator: xml_text(doc, "Creator"),
        standard: standard.to_string(),
        encrypted,
        reels,
    }
}

/// Есть ли в каталоге хотя бы один CPL (xml с корнем CompositionPlaylist).
fn dir_has_cpl(dir: &Path) -> bool {
    let rd = match std::fs::read_dir(dir) {
        Ok(r) => r,
        Err(_) => return false,
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.extension()
            .map(|x| x.eq_ignore_ascii_case("xml"))
            .unwrap_or(false)
        {
            if let Ok(xml) = std::fs::read_to_string(&p) {
                if let Ok(doc) = roxmltree::Document::parse(&xml) {
                    if doc.root_element().tag_name().name() == "CompositionPlaylist" {
                        return true;
                    }
                }
            }
        }
    }
    false
}

/// Если в корне нет CPL, но пакет лежит в единственной подпапке — спускаемся туда.
fn pick_dcp_dir(root: &Path) -> PathBuf {
    if dir_has_cpl(root) {
        return root.to_path_buf();
    }
    if let Ok(rd) = std::fs::read_dir(root) {
        let subdirs: Vec<PathBuf> = rd
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.is_dir())
            .collect();
        if subdirs.len() == 1 && dir_has_cpl(&subdirs[0]) {
            return subdirs[0].clone();
        }
    }
    root.to_path_buf()
}

fn mi_tracks(v: &serde_json::Value) -> Vec<serde_json::Value> {
    match v.get("media").and_then(|m| m.get("track")) {
        Some(serde_json::Value::Array(a)) => a.clone(),
        Some(other) => vec![other.clone()],
        None => vec![],
    }
}
fn sget(t: &serde_json::Value, k: &str) -> Option<String> {
    t.get(k).and_then(|v| v.as_str()).map(|s| s.to_string())
}

/// Быстрая проба: путь — это директория?
#[tauri::command]
fn is_directory(path: String) -> bool {
    Path::new(&path).is_dir()
}

/// Welcome intentionally opens compact. Each real workspace has its own minimum
/// working size; entering it only grows the window and never shrinks a user's view.
#[tauri::command]
fn ensure_workspace_window(workspace: Option<String>, app: tauri::AppHandle) -> Result<(), String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "Main window is unavailable".to_string())?;

    if window.is_maximized().map_err(|e| e.to_string())? {
        return Ok(());
    }

    let scale = window.scale_factor().map_err(|e| e.to_string())?;
    let current = window.inner_size().map_err(|e| e.to_string())?;
    let (target_width, target_height) = match workspace.as_deref() {
        // ArchiveMax has a full source/destination setup and evidence view.
        // Give it enough vertical room to avoid a distracting page scrollbar.
        Some("offload") => (1320.0, 1090.0),
        _ => (1160.0, 760.0),
    };
    let width = (current.width as f64 / scale).max(target_width);
    let height = (current.height as f64 / scale).max(target_height);

    if width > current.width as f64 / scale || height > current.height as f64 / scale {
        window
            .set_size(tauri::LogicalSize::new(width, height))
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Проверка DCP-пакета: структура + разбор CPL + реальные параметры MXF (mediainfo).
#[tauri::command]
fn analyze_dcp(app: tauri::AppHandle, path: String) -> Result<DcpReport, String> {
    let root = Path::new(&path);
    if !root.exists() {
        return Err(format!("Папка не найдена: {path}"));
    }
    if !root.is_dir() {
        return Err("Это не папка. Для проверки DCP укажи папку пакета.".into());
    }
    let scan = pick_dcp_dir(root);
    log::info!("analyze_dcp scan={scan:?}");

    let mut rep = DcpReport {
        path: path.clone(),
        folder_name: root
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default(),
        ..Default::default()
    };
    let mut mxf_paths: Vec<PathBuf> = Vec::new();

    let rd = std::fs::read_dir(&scan).map_err(|e| e.to_string())?;
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            continue;
        }
        let name = p
            .file_name()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        let size = std::fs::metadata(&p).map(|m| m.len()).unwrap_or(0);
        rep.total_size += size;
        let lower = name.to_lowercase();
        let mut kind = "other";
        if lower == "assetmap" || lower == "assetmap.xml" {
            kind = "ASSETMAP";
            rep.has_assetmap = true;
        } else if lower == "volindex" || lower == "volindex.xml" {
            kind = "VOLINDEX";
            rep.has_volindex = true;
        } else if lower.ends_with(".mxf") {
            kind = "MXF";
            mxf_paths.push(p.clone());
        } else if lower.ends_with(".xml") {
            if let Ok(xml) = std::fs::read_to_string(&p) {
                if let Ok(doc) = roxmltree::Document::parse(&xml) {
                    match doc.root_element().tag_name().name() {
                        "CompositionPlaylist" => {
                            kind = "CPL";
                            rep.cpls.push(cpl_from_doc(&doc, &p));
                        }
                        "PackingList" => {
                            kind = "PKL";
                            rep.has_pkl = true;
                        }
                        "AssetMap" => {
                            kind = "ASSETMAP";
                            rep.has_assetmap = true;
                        }
                        _ => {}
                    }
                }
            }
        }
        rep.files.push(DcpFile {
            name,
            kind: kind.to_string(),
            size,
        });
    }

    rep.mxf_count = mxf_paths.len();
    rep.cpl_count = rep.cpls.len();
    rep.is_dcp = !rep.cpls.is_empty();

    // Реальные параметры эссенций из MXF (заголовок читается быстро, не весь файл).
    if rep.is_dcp {
        let tools_dir = app.path().resource_dir().ok().map(|d| d.join("tools"));
        let mi = resolve_tool(&tools_dir, "mediainfo");
        for mp in mxf_paths.iter().take(24) {
            if rep.picture.is_some() && rep.audio.is_some() {
                break;
            }
            let mut args = mi.base.clone();
            args.push("--Output=JSON".to_string());
            args.push(mp.to_string_lossy().into_owned());
            let out = run_tool(&mi.program, &args).output;
            let v: serde_json::Value =
                serde_json::from_str(&out).unwrap_or(serde_json::Value::Null);
            for tr in mi_tracks(&v) {
                match tr.get("@type").and_then(|x| x.as_str()) {
                    Some("Video") if rep.picture.is_none() => {
                        rep.picture = Some(DcpPicture {
                            width: sget(&tr, "Width").and_then(|s| s.parse().ok()),
                            height: sget(&tr, "Height").and_then(|s| s.parse().ok()),
                            frame_rate: sget(&tr, "FrameRate"),
                            format: sget(&tr, "Format"),
                            color_space: sget(&tr, "ColorSpace"),
                            bit_depth: sget(&tr, "BitDepth"),
                            bit_rate: sget(&tr, "BitRate"),
                        });
                    }
                    Some("Audio") if rep.audio.is_none() => {
                        rep.audio = Some(DcpAudio {
                            channels: sget(&tr, "Channels").and_then(|s| s.parse().ok()),
                            sample_rate: sget(&tr, "SamplingRate"),
                            bit_depth: sget(&tr, "BitDepth"),
                            format: sget(&tr, "Format"),
                        });
                    }
                    _ => {}
                }
            }
        }
    }

    log::info!(
        "analyze_dcp done is_dcp={} cpl={} mxf={} pic={} aud={}",
        rep.is_dcp,
        rep.cpl_count,
        rep.mxf_count,
        rep.picture.is_some(),
        rep.audio.is_some()
    );
    Ok(rep)
}

// ── Offload mode: слив карты на N дисков с verify + ASC MHL ────────────────

/// Runtime-флаги текущего оффлоада (cancel/pause из UI, один джоб за раз).
struct OffloadCtl {
    cancel: std::sync::Arc<std::sync::atomic::AtomicBool>,
    pause: std::sync::Arc<std::sync::atomic::AtomicBool>,
    running: std::sync::Arc<std::sync::atomic::AtomicBool>,
    active_job: std::sync::Arc<std::sync::Mutex<Option<(PathBuf, String)>>>,
}

impl Default for OffloadCtl {
    fn default() -> Self {
        Self {
            cancel: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            pause: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            running: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            active_job: std::sync::Arc::new(std::sync::Mutex::new(None)),
        }
    }
}

/// Держатель sentry-guard. Живёт в managed-state весь рантайм; Drop флашит очередь
/// на выходе. `None`, если DSN не вшит или opt-in выключен (штатно).
struct CrashGuard(#[allow(dead_code)] Option<sentry::ClientInitGuard>);

#[tauri::command]
async fn offload_pick_folder(app: tauri::AppHandle) -> Option<String> {
    // Неблокирующая колбэк-форма (рекомендованный async-паттерн Tauri): панель
    // открывается из event-loop главного потока, результат приходит через канал.
    // Так уходит вторичная паника RecvError, которую даёт blocking_* при сбое диалога.
    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog().file().pick_folder(move |folder| {
        let _ = tx.send(folder);
    });
    rx.await
        .ok()
        .flatten()
        .and_then(|f| f.into_path().ok())
        .map(|p| p.to_string_lossy().into_owned())
}

#[tauri::command]
fn offload_cancel(state: tauri::State<'_, OffloadCtl>) {
    state
        .cancel
        .store(true, std::sync::atomic::Ordering::SeqCst);
    if let Ok(active) = state.active_job.lock() {
        if let Some((db_path, job_id)) = active.as_ref() {
            if let Ok(conn) = offload::checkpoint::open_db(db_path) {
                let _ = offload::checkpoint::update_job_status(&conn, job_id, "terminated");
                let _ =
                    offload::checkpoint::append_job_event(&conn, job_id, "cancelRequested", "{}");
            }
        }
    }
}

#[tauri::command]
fn offload_set_pause(state: tauri::State<'_, OffloadCtl>, paused: bool) {
    state
        .pause
        .store(paused, std::sync::atomic::Ordering::SeqCst);
    if let Ok(active) = state.active_job.lock() {
        if let Some((db_path, job_id)) = active.as_ref() {
            if let Ok(conn) = offload::checkpoint::open_db(db_path) {
                let _ = offload::checkpoint::update_job_status(
                    &conn,
                    job_id,
                    if paused { "paused" } else { "running" },
                );
                let _ = offload::checkpoint::append_job_event(
                    &conn,
                    job_id,
                    if paused { "jobPaused" } else { "jobContinued" },
                    "{}",
                );
            }
        }
    }
}

#[tauri::command]
#[allow(clippy::too_many_arguments)]
fn offload_start(
    app: tauri::AppHandle,
    state: tauri::State<'_, OffloadCtl>,
    source: String,
    destinations: Vec<String>,
    algorithms: Vec<String>,
    write_mhl: bool,
    profile: Option<String>,
    small_file_concurrency: Option<usize>,
    report_contacts: Option<Vec<offload::orchestrator::DitContact>>,
    auto_eject: Option<bool>,
) -> Result<String, String> {
    use std::sync::atomic::Ordering;
    use tauri::Emitter;

    let db_path = offload_db_path(&app)?;
    if state.running.swap(true, Ordering::SeqCst) {
        return Err("Offload already running".into());
    }
    state.cancel.store(false, Ordering::SeqCst);
    state.pause.store(false, Ordering::SeqCst);

    let job_id = format!("job-{}", chrono::Utc::now().format("%Y%m%d-%H%M%S%.3f"));
    let parsed_algorithms = algorithms
        .iter()
        .map(|value| match value.to_ascii_lowercase().as_str() {
            "sha256" => offload::hash_engine::HashAlgorithm::SHA256,
            "xxh128" => offload::hash_engine::HashAlgorithm::XXH128,
            "xxh3" => offload::hash_engine::HashAlgorithm::XXH3,
            "blake3" => offload::hash_engine::HashAlgorithm::BLAKE3,
            "md5" => offload::hash_engine::HashAlgorithm::MD5,
            _ => offload::hash_engine::HashAlgorithm::XXH64,
        })
        .collect();
    let request = offload::orchestrator::OffloadRequest {
        source: PathBuf::from(source),
        destinations: destinations.into_iter().map(PathBuf::from).collect(),
        algorithms: parsed_algorithms,
        write_mhl,
        checkpoint_db: Some(db_path.clone()),
        profile: match profile.as_deref() {
            Some("fast") => offload::orchestrator::VerificationProfile::Fast,
            _ => offload::orchestrator::VerificationProfile::ArchiveMax,
        },
        job_id: Some(job_id.clone()),
        small_file_concurrency: small_file_concurrency.unwrap_or(1),
        report_contacts: report_contacts.unwrap_or_default(),
        auto_eject: auto_eject.unwrap_or(false),
    };
    if let Ok(mut active) = state.active_job.lock() {
        *active = Some((db_path.clone(), job_id.clone()));
    }

    let cancel = state.cancel.clone();
    let pause = state.pause.clone();
    let running = state.running.clone();
    let active_job = state.active_job.clone();
    let emitter = app.clone();
    let failed_job_id = job_id.clone();
    tauri::async_runtime::spawn(async move {
        let result = offload::orchestrator::run_offload(
            &request,
            Some(&cancel),
            Some(&pause),
            &|progress| {
                let _ = emitter.emit("offload-progress", &progress);
            },
        )
        .await;
        running.store(false, Ordering::SeqCst);
        if let Ok(mut active) = active_job.lock() {
            *active = None;
        }
        match result {
            Ok(summary) => {
                // Canonical JSON evidence travels with every successful copy.
                // Manual export remains for operator-facing HTML, CSV, TXT or a
                // second local JSON file.
                for error in write_canonical_evidence_to_destinations(&summary).await {
                    log::warn!("Could not save destination evidence: {error}");
                }
                let _ = emitter.emit("offload-complete", &summary);
            }
            Err(error) => {
                if let Ok(conn) = offload::checkpoint::open_db(&db_path) {
                    let status = if cancel.load(Ordering::SeqCst) {
                        "terminated"
                    } else {
                        "failed"
                    };
                    let _ = offload::checkpoint::update_job_status(&conn, &failed_job_id, status);
                }
                let _ = emitter.emit(
                    "offload-error",
                    serde_json::json!({
                        "jobId": failed_job_id,
                        "message": format!("{error:#}"),
                    }),
                );
            }
        }
    });
    Ok(job_id)
}

fn offload_db_path(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|error| error.to_string())?;
    std::fs::create_dir_all(&dir).map_err(|error| error.to_string())?;
    Ok(dir.join("offload.sqlite"))
}

#[tauri::command]
fn offload_get_job(
    app: tauri::AppHandle,
    job_id: String,
) -> Result<offload::orchestrator::JobEvidence, String> {
    let db_path = offload_db_path(&app)?;
    let conn = offload::checkpoint::open_db(&db_path).map_err(|error| format!("{error:#}"))?;
    let record = offload::checkpoint::get_job(&conn, &job_id)
        .map_err(|error| format!("{error:#}"))?
        .ok_or_else(|| format!("Checkpoint job not found: {job_id}"))?;
    let summary = record
        .summary_json
        .map(|json| serde_json::from_str(&json).map_err(|error| error.to_string()))
        .transpose()?;
    let progress = offload::checkpoint::get_job_progress(&conn, &job_id)
        .map_err(|error| format!("{error:#}"))?;
    Ok(offload::orchestrator::JobEvidence {
        job_id,
        state: offload::orchestrator::JobState::from_checkpoint(&record.status),
        progress,
        summary,
    })
}

#[tauri::command]
async fn offload_resume(
    app: tauri::AppHandle,
    state: tauri::State<'_, OffloadCtl>,
    job_id: String,
) -> Result<offload::orchestrator::OffloadSummary, String> {
    use std::sync::atomic::Ordering;
    use tauri::Emitter;

    let db_path = offload_db_path(&app)?;
    let config_json = {
        let conn = offload::checkpoint::open_db(&db_path).map_err(|error| format!("{error:#}"))?;
        offload::checkpoint::get_job_config(&conn, &job_id)
            .map_err(|error| format!("{error:#}"))?
            .ok_or_else(|| format!("Checkpoint job not found: {job_id}"))?
    };
    let mut request: offload::orchestrator::OffloadRequest =
        serde_json::from_str(&config_json).map_err(|error| error.to_string())?;
    request.job_id = Some(job_id.clone());
    request.checkpoint_db = Some(db_path.clone());

    if state.running.swap(true, Ordering::SeqCst) {
        return Err("Offload already running".into());
    }
    state.cancel.store(false, Ordering::SeqCst);
    state.pause.store(false, Ordering::SeqCst);
    if let Ok(mut active) = state.active_job.lock() {
        *active = Some((db_path.clone(), job_id.clone()));
    }

    let cancel = state.cancel.clone();
    let pause = state.pause.clone();
    let emitter = app.clone();
    let result =
        offload::orchestrator::run_offload(&request, Some(&cancel), Some(&pause), &|progress| {
            let _ = emitter.emit("offload-progress", &progress);
        })
        .await;
    state.running.store(false, Ordering::SeqCst);
    if let Ok(mut active) = state.active_job.lock() {
        *active = None;
    }
    let summary = result.map_err(|error| format!("{error:#}"))?;
    for error in write_canonical_evidence_to_destinations(&summary).await {
        log::warn!("Could not save destination evidence after resume: {error}");
    }
    Ok(summary)
}

#[tauri::command]
async fn offload_verify(
    app: tauri::AppHandle,
    state: tauri::State<'_, OffloadCtl>,
    path: Option<String>,
    job_id: Option<String>,
    options: Option<offload::mhl::verifier::MhlVerifyOptions>,
) -> Result<offload::mhl::verifier::MhlVerifyReport, String> {
    use std::sync::atomic::Ordering;
    use tauri::Emitter;

    let (targets, report_path, report_mode) = if let Some(job_id) = job_id {
        let evidence = offload_get_job(app.clone(), job_id.clone())?;
        let summary = evidence
            .summary
            .ok_or_else(|| format!("Job has no completed evidence snapshot: {job_id}"))?;
        (
            summary
                .destination_volumes
                .into_iter()
                .map(|volume| volume.path)
                .collect::<Vec<_>>(),
            job_id,
            "job".to_string(),
        )
    } else {
        let path = path.ok_or_else(|| "path or jobId is required".to_string())?;
        (vec![path.clone()], path, "target".to_string())
    };
    if state.running.swap(true, Ordering::SeqCst) {
        return Err("Offload or verification already running".into());
    }
    state.cancel.store(false, Ordering::SeqCst);
    state.pause.store(false, Ordering::SeqCst);
    let cancel = state.cancel.clone();
    let pause = state.pause.clone();
    let running = state.running.clone();
    let emitter = app.clone();
    let task = tokio::task::spawn_blocking(move || {
        let progress = |value: offload::mhl::verifier::MhlVerifyProgress| {
            let _ = emitter.emit(
                "offload-progress",
                serde_json::json!({
                    "phase": value.phase,
                    "currentFile": value.current_file,
                    "fileIndex": value.file_index,
                    "totalFiles": value.total_files,
                    "bytesDone": value.file_index,
                    "bytesTotal": value.total_files,
                }),
            );
        };
        let control = offload::mhl::verifier::MhlVerifyControl {
            cancel_flag: Some(&cancel),
            pause_flag: Some(&pause),
            on_progress: Some(&progress),
        };
        let options = options.unwrap_or_default();
        let reports = targets
            .iter()
            .map(|target| {
                offload::mhl::verifier::verify_mhl_path_with_control(
                    Path::new(target),
                    options.clone(),
                    &control,
                )
                .map_err(|error| format!("{error:#}"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut reports = reports.into_iter();
        let mut report = reports
            .next()
            .ok_or_else(|| "Job has no destination volumes".to_string())?;
        for additional in reports {
            report = merge_mhl_verify_reports(report, additional);
        }
        report.summary.path = report_path;
        report.summary.mode = report_mode;
        report.summary.verified_generations.sort_unstable();
        report.summary.verified_generations.dedup();
        Ok(report)
    })
    .await;
    running.store(false, Ordering::SeqCst);
    task.map_err(|error| error.to_string())?
}

fn merge_mhl_verify_reports(
    mut report: offload::mhl::verifier::MhlVerifyReport,
    additional: offload::mhl::verifier::MhlVerifyReport,
) -> offload::mhl::verifier::MhlVerifyReport {
    report.summary.success &= additional.summary.success;
    report.summary.chain_entries += additional.summary.chain_entries;
    report.summary.chain_valid += additional.summary.chain_valid;
    report.summary.chain_invalid += additional.summary.chain_invalid;
    report.summary.total_files += additional.summary.total_files;
    report.summary.passed += additional.summary.passed;
    report.summary.failed += additional.summary.failed;
    report.summary.missing += additional.summary.missing;
    report.summary.errors += additional.summary.errors;
    report.summary.duration_secs += additional.summary.duration_secs;
    report
        .summary
        .verified_generations
        .extend(additional.summary.verified_generations);
    report.chain_results.extend(additional.chain_results);
    report.issues.extend(additional.issues);
    report
}

#[tauri::command]
async fn offload_export(
    app: tauri::AppHandle,
    job_id: String,
    format: String,
) -> Result<Option<String>, String> {
    let evidence = offload_get_job(app.clone(), job_id.clone())?;
    let summary = evidence
        .summary
        .ok_or_else(|| format!("Job has no completed evidence snapshot: {job_id}"))?;
    let report_format =
        offload::report::ReportFormat::parse(&format).map_err(|error| error.to_string())?;
    let contents =
        offload::report::render(&summary, report_format).map_err(|error| error.to_string())?;
    let extension = report_format.extension();
    let default_name = offload_evidence_file_name(&job_id, extension);

    let (tx, rx) = tokio::sync::oneshot::channel();
    app.dialog()
        .file()
        .add_filter("Offload evidence", &[extension])
        .set_file_name(&default_name)
        .save_file(move |picked| {
            let _ = tx.send(picked);
        });
    match rx.await.ok().flatten() {
        Some(file_path) => {
            let path = file_path.into_path().map_err(|error| error.to_string())?;
            write_evidence_file(&path, &contents).await?;
            Ok(Some(path.to_string_lossy().into_owned()))
        }
        None => Ok(None),
    }
}

fn mhl_directory(mhl_path: String) -> Result<PathBuf, String> {
    let manifest_path = PathBuf::from(mhl_path);
    if !manifest_path
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case("mhl"))
    {
        return Err("Expected an MHL manifest path".to_string());
    }

    let mhl_dir = manifest_path
        .parent()
        .filter(|path| path.file_name().is_some_and(|name| name == "ascmhl"))
        .ok_or_else(|| "MHL manifest is not inside an ascmhl folder".to_string())?
        .to_path_buf();
    if !mhl_dir.is_dir() {
        return Err(format!(
            "MHL folder no longer exists: {}",
            mhl_dir.display()
        ));
    }
    Ok(mhl_dir)
}

/// Opens the ASC MHL folder(s) for a completed offload. The frontend receives
/// these paths only from the completed job summary.
#[tauri::command]
fn offload_open_mhl_folders(app: tauri::AppHandle, mhl_paths: Vec<String>) -> Result<(), String> {
    if mhl_paths.is_empty() {
        return Err("No MHL manifests are available for this job".to_string());
    }

    let mut directories = Vec::new();
    for mhl_path in mhl_paths {
        let directory = mhl_directory(mhl_path)?;
        if !directories.contains(&directory) {
            directories.push(directory);
        }
    }

    for directory in directories {
        app.opener()
            .open_path(directory.to_string_lossy(), None::<&str>)
            .map_err(|error| error.to_string())?;
    }
    Ok(())
}

// ── Crash reporting commands (opt-in тумблер + JS-мост) ────────────────────

/// Прочитать состояние opt-in отправки крашей (default OFF).
#[tauri::command]
fn crash_get_opt_in(app: tauri::AppHandle) -> bool {
    app.path()
        .app_config_dir()
        .map(|d| crash::opt_in_enabled(&d))
        .unwrap_or(false)
}

/// Переключить opt-in. Применяется при следующем запуске (sentry инитится в setup).
#[tauri::command]
fn crash_set_opt_in(app: tauri::AppHandle, enabled: bool) -> Result<(), String> {
    let dir = app.path().app_config_dir().map_err(|e| e.to_string())?;
    crash::set_opt_in(&dir, enabled).map_err(|e| e.to_string())?;
    log::info!("crash opt-in set to {enabled} (applies on next launch)");
    Ok(())
}

/// Мост ошибок фронтенда: локальный лог всегда + пересылка в sentry (no-op без клиента).
#[tauri::command]
fn crash_report_js(
    message: String,
    source: Option<String>,
    line: Option<u32>,
    stack: Option<String>,
) {
    log::error!("[js-error] {message} @ {source:?}:{line:?}");
    crash::capture_js_error(&message, source.as_deref(), line, stack.as_deref());
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(
            tauri_plugin_log::Builder::new()
                .level(log::LevelFilter::Info)
                .targets([
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::Stdout),
                    tauri_plugin_log::Target::new(tauri_plugin_log::TargetKind::LogDir {
                        file_name: Some(PROOFCAT_LOG_STEM.into()),
                    }),
                ])
                .build(),
        )
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_updater::Builder::new().build())
        .setup(|app| {
            log::info!("ProofCat started");
            // macOS: явный режим обычного foreground-приложения. Без этого нативные
            // диалоги (NSOpenPanel) из небандленного dev-бинарника (`cargo run`, не .app)
            // могут вернуть nil и уронить процесс. В .app-бандле политика и так Regular.
            #[cfg(target_os = "macos")]
            app.set_activation_policy(tauri::ActivationPolicy::Regular);
            // Crash reporting: инициализируется только если DSN вшит И opt-in включён.
            // Иначе полностью инертно (None). Локальный лог пишется независимо.
            let config_dir = app
                .path()
                .app_config_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."));
            let guard = crash::init(&config_dir);
            app.manage(CrashGuard(guard));
            Ok(())
        })
        .manage(OffloadCtl::default())
        .invoke_handler(tauri::generate_handler![
            analyze_file,
            pick_files,
            save_report,
            log_event,
            open_url,
            collect_feedback,
            measure_loudness,
            hash_file,
            scan_frames,
            check_update,
            install_update,
            is_directory,
            ensure_workspace_window,
            analyze_dcp,
            offload_pick_folder,
            offload_start,
            offload_resume,
            offload_get_job,
            offload_verify,
            offload_export,
            offload_open_mhl_folders,
            offload_cancel,
            offload_set_pause,
            crash_get_opt_in,
            crash_set_opt_in,
            crash_report_js
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod dcp_tests {
    use super::*;

    // Минимальный SMPTE CPL с namespace по умолчанию — проверяем, что tag_name().name()
    // возвращает локальное имя без учёта namespace, и что рил разбирается.
    const CPL: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<CompositionPlaylist xmlns="http://www.smpte-ra.org/schemas/429-7/2006/CPL">
  <Id>urn:uuid:11111111-1111-1111-1111-111111111111</Id>
  <ContentTitleText>HobbitBattle_FTR_F-185_RU-XX_51_4K_NET_20260711_SMPTE_OV</ContentTitleText>
  <AnnotationText>HobbitBattle</AnnotationText>
  <IssueDate>2026-07-11T00:00:00+00:00</IssueDate>
  <Creator>DCP-o-matic</Creator>
  <ReelList>
    <Reel>
      <AssetList>
        <MainPicture>
          <Id>urn:uuid:22222222-2222-2222-2222-222222222222</Id>
          <EditRate>24 1</EditRate>
          <IntrinsicDuration>34560</IntrinsicDuration>
          <Duration>34560</Duration>
          <ScreenAspectRatio>1998 1080</ScreenAspectRatio>
        </MainPicture>
        <MainSound>
          <Id>urn:uuid:33333333-3333-3333-3333-333333333333</Id>
          <KeyId>urn:uuid:44444444-4444-4444-4444-444444444444</KeyId>
        </MainSound>
      </AssetList>
    </Reel>
  </ReelList>
</CompositionPlaylist>"#;

    #[test]
    fn parses_smpte_cpl() {
        let doc = roxmltree::Document::parse(CPL).unwrap();
        assert_eq!(doc.root_element().tag_name().name(), "CompositionPlaylist");
        let cpl = cpl_from_doc(&doc, Path::new("cpl_abc.xml"));
        assert_eq!(
            cpl.content_title,
            "HobbitBattle_FTR_F-185_RU-XX_51_4K_NET_20260711_SMPTE_OV"
        );
        assert_eq!(cpl.standard, "SMPTE");
        assert!(cpl.encrypted, "KeyId present -> encrypted");
        assert_eq!(cpl.reels.len(), 1);
        let r = &cpl.reels[0];
        assert_eq!(r.duration_frames, Some(34560));
        assert_eq!(r.fps, Some(24.0));
        // 34560 / 24 = 1440 сек = 24 мин (должно потом ловиться как >22 мин)
        assert_eq!(r.duration_sec, Some(1440.0));
        assert_eq!(r.aspect.as_deref(), Some("1998 1080"));
    }

    #[test]
    fn job_verification_aggregates_each_destination_chain_once() {
        use crate::offload::mhl::verifier::{MhlVerifyReport, MhlVerifySummary};

        let report = |chain_entries, generation| MhlVerifyReport {
            summary: MhlVerifySummary {
                path: String::new(),
                mode: "job".into(),
                success: true,
                chain_only: false,
                chain_entries,
                chain_valid: chain_entries,
                chain_invalid: 0,
                total_files: chain_entries,
                passed: chain_entries,
                failed: 0,
                missing: 0,
                errors: 0,
                verified_generations: vec![generation],
                duration_secs: 1.0,
            },
            chain_results: Vec::new(),
            issues: Vec::new(),
        };

        let merged = merge_mhl_verify_reports(report(2, 1), report(3, 2));
        assert_eq!(merged.summary.chain_entries, 5);
        assert_eq!(merged.summary.chain_valid, 5);
        assert_eq!(merged.summary.total_files, 5);
        assert_eq!(merged.summary.verified_generations, vec![1, 2]);
    }
}

#[cfg(test)]
mod runtime_identity_tests {
    use super::*;

    #[test]
    fn feedback_prefers_proofcat_log_and_keeps_legacy_fallback() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(
            dir.path().join(LEGACY_META_REPORT_LOG_FILE),
            "legacy diagnostics",
        )
        .unwrap();
        assert_eq!(feedback_log_tail(dir.path()), "legacy diagnostics");

        std::fs::write(dir.path().join(PROOFCAT_LOG_FILE), "current diagnostics").unwrap();
        assert_eq!(feedback_log_tail(dir.path()), "current diagnostics");
    }

    #[test]
    fn evidence_file_name_uses_proofcat_and_sanitizes_job_colons() {
        assert_eq!(
            offload_evidence_file_name("job-2026-07-14T12:34:56Z", "json"),
            "proofcat-job-2026-07-14T12-34-56Z.json"
        );
    }

    #[test]
    fn replica_association_matches_path_components_not_string_prefix() {
        assert!(replica_belongs_to_destination(
            "/Volumes/CARD/clip.mov",
            "/Volumes/CARD"
        ));
        assert!(!replica_belongs_to_destination(
            "/Volumes/CARD2/clip.mov",
            "/Volumes/CARD"
        ));
        assert!(!replica_belongs_to_destination(
            "/Volumes/CARD-backup/clip.mov",
            "/Volumes/CARD"
        ));
    }
}
