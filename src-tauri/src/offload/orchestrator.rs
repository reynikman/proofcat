//! Offload orchestrator — слим-замена workflow DIT-Pro (5157 строк там; здесь только то,
//! что нужно Meta Report): скан источника → checkpoint-джоб → tee-copy на N дисков с
//! inline-хешем → ASC MHL на каждом получателе → сводка.
//!
//! Источник никогда не модифицируется. Копия идёт через AtomicWriter (temp → rename после
//! verify), обрыв на середине оставляет карту целой и checkpoint-БД с recoverable-state.

use anyhow::{bail, Context, Result};
use futures::stream::{self, StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
#[cfg(target_os = "macos")]
use std::path::Component;
use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;
use unicode_normalization::UnicodeNormalization;

use crate::offload::checkpoint;
use crate::offload::copy_engine::{
    copy_file_multi, copy_file_single, CopyControl, CopyEngineConfig, CopyFileResult,
    FileConflictPolicy,
};
use crate::offload::hash_engine::{
    hash_file_with_control, HashAlgorithm, HashControl, HashEngineConfig, HashResult,
};
use crate::offload::io_scheduler::{
    DevicePermit, DeviceQueue, DeviceSchedulerConfig, IoScheduler, SchedulerPolicy,
};
use crate::offload::mhl;
use crate::offload::volume::{identify_volume, VolumeIdentity};

/// Что оффлоадим и куда.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub enum VerificationProfile {
    Fast,
    #[default]
    ArchiveMax,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OffloadRequest {
    /// Корень источника (карта / папка съёмки). Только чтение.
    pub source: PathBuf,
    /// Корни получателей (1..N дисков). Файлы лягут в `dest/<rel_path>`.
    pub destinations: Vec<PathBuf>,
    /// Алгоритмы inline-хеша (по умолчанию XXH64 — стандарт MHL-мира).
    pub algorithms: Vec<HashAlgorithm>,
    /// Писать ASC MHL generation на каждом получателе.
    pub write_mhl: bool,
    /// Путь к checkpoint-БД (None — без checkpoint, только для тестов).
    pub checkpoint_db: Option<PathBuf>,
    /// Fast performs a durable copy only. ArchiveMax independently reads the
    /// source twice and each destination after the write.
    #[serde(default)]
    pub profile: VerificationProfile,
    /// Existing checkpoint job to continue. None starts a new job.
    #[serde(default)]
    pub job_id: Option<String>,
    /// Requested small-file workers. Honored only for SSD sources and still
    /// capped by per-device queues plus the 512 MiB memory policy.
    #[serde(default = "default_small_file_concurrency")]
    pub small_file_concurrency: usize,
    /// DIT/operator contacts included in immutable job evidence and exports.
    #[serde(default)]
    pub report_contacts: Vec<DitContact>,
    /// Intentionally opt-in. v0.1 never ejects a device unless a future
    /// platform adapter explicitly honors this persisted request.
    #[serde(default)]
    pub auto_eject: bool,
}

fn default_small_file_concurrency() -> usize {
    1
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DitContact {
    pub name: String,
    #[serde(default)]
    pub role: String,
    #[serde(default)]
    pub contact: String,
}

impl DitContact {
    pub fn normalized(mut self) -> Option<Self> {
        self.name = self.name.trim().to_string();
        self.role = self.role.trim().to_string();
        self.contact = self.contact.trim().to_string();
        (!self.name.is_empty()).then_some(self)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct DestinationPreflight {
    pub destination: String,
    pub required_bytes: u64,
    pub available_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HashPolicy {
    pub evidence_algorithms: Vec<HashAlgorithm>,
    pub mhl_algorithm: HashAlgorithm,
}

impl Default for HashPolicy {
    fn default() -> Self {
        Self {
            evidence_algorithms: vec![HashAlgorithm::XXH64],
            mhl_algorithm: HashAlgorithm::XXH64,
        }
    }
}

impl HashPolicy {
    fn for_request(req: &OffloadRequest) -> Self {
        let mut evidence = if req.algorithms.is_empty() {
            vec![HashAlgorithm::XXH64]
        } else {
            req.algorithms.clone()
        };
        if !evidence.contains(&HashAlgorithm::XXH64) {
            evidence.insert(0, HashAlgorithm::XXH64);
        }
        if req.profile == VerificationProfile::ArchiveMax
            && !evidence.contains(&HashAlgorithm::BLAKE3)
        {
            evidence.push(HashAlgorithm::BLAKE3);
        }
        let mut seen = HashSet::new();
        evidence.retain(|algo| seen.insert(*algo));
        Self {
            evidence_algorithms: evidence,
            mhl_algorithm: HashAlgorithm::XXH64,
        }
    }
}

/// Прогресс для UI (эмитится колбэком, Tauri-слой превращает в event).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OffloadProgress {
    pub phase: String, // scanning | copying | mhl | done
    pub current_file: String,
    pub file_index: usize,
    pub total_files: usize,
    pub bytes_done: u64,
    pub bytes_total: u64,
}

/// Одна ошибка по файлу (джоб не падает целиком из-за одного битого файла).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OffloadFailure {
    pub file: String,
    pub destination: Option<String>,
    pub error: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OffloadVerdict {
    CopyComplete,
    ArchiveVerified,
    SafeToFormat,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum JobState {
    Pending,
    Running,
    Paused,
    Completed,
    Failed,
    Terminated,
    Unknown,
}

impl JobState {
    pub fn from_checkpoint(value: &str) -> Self {
        match value {
            "pending" => Self::Pending,
            "running" => Self::Running,
            "paused" => Self::Paused,
            "completed" => Self::Completed,
            "failed" => Self::Failed,
            "terminated" => Self::Terminated,
            _ => Self::Unknown,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HashObservation {
    pub side: String,
    pub file: String,
    pub destination: Option<String>,
    pub hashes: Vec<HashResult>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RepairAttempt {
    pub file: String,
    pub destination: String,
    pub source: String,
    pub attempt: u32,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ReplicaState {
    CopyFailed,
    AlreadyMatched,
    CopyComplete,
    SourceChanged,
    Verified,
    VerifyFailed,
}

impl ReplicaState {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::CopyFailed => "copyFailed",
            Self::AlreadyMatched => "alreadyMatched",
            Self::CopyComplete => "copyComplete",
            Self::SourceChanged => "sourceChanged",
            Self::Verified => "verified",
            Self::VerifyFailed => "verifyFailed",
        }
    }
}

impl std::fmt::Display for ReplicaState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReplicaEvidence {
    pub file: String,
    /// Source byte count, repeated per destination so CSV remains a complete
    /// file × destination ledger without joining another table.
    #[serde(default)]
    pub bytes: u64,
    pub destination: String,
    pub status: ReplicaState,
    pub expected_hashes: Vec<HashResult>,
    pub observed_hashes: Vec<HashResult>,
    pub repair_attempts: u32,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceFileEvidence {
    pub path: String,
    pub size: u64,
    pub modified_ns: u128,
    pub file_identity: String,
}

/// Итог оффлоада.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OffloadSummary {
    #[serde(default)]
    pub evidence_schema_version: u32,
    #[serde(default)]
    pub app_name: String,
    #[serde(default)]
    pub app_version: String,
    #[serde(default)]
    pub commit: String,
    #[serde(default)]
    pub started_at: String,
    #[serde(default)]
    pub finished_at: String,
    pub job_id: String,
    pub total_files: usize,
    pub copied: usize,
    pub skipped: usize,
    pub failed: usize,
    pub bytes_copied: u64,
    pub failures: Vec<OffloadFailure>,
    pub mhl_paths: Vec<String>,
    pub profile: VerificationProfile,
    #[serde(default)]
    pub hash_policy: HashPolicy,
    pub verdict: OffloadVerdict,
    pub safe_to_format: bool,
    pub verified_replicas: usize,
    pub verification_failed: usize,
    #[serde(default = "default_small_file_concurrency")]
    pub effective_small_file_workers: usize,
    pub warnings: Vec<String>,
    pub observations: Vec<HashObservation>,
    pub replicas: Vec<ReplicaEvidence>,
    pub repairs: Vec<RepairAttempt>,
    pub source_volume: VolumeIdentity,
    pub destination_volumes: Vec<VolumeIdentity>,
    #[serde(default)]
    pub source_snapshot: Vec<SourceFileEvidence>,
    /// Space observed before the first source read. A copy still repeats a
    /// per-file check, because free space may change while the job runs.
    #[serde(default)]
    pub destination_preflight: Vec<DestinationPreflight>,
    #[serde(default)]
    pub report_contacts: Vec<DitContact>,
    /// Stored for auditability. Defaults to false and is not actioned by the
    /// core engine in v0.1.
    #[serde(default)]
    pub auto_eject_requested: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct JobEvidence {
    pub job_id: String,
    pub state: JobState,
    pub progress: checkpoint::JobProgress,
    pub summary: Option<OffloadSummary>,
}

/// Файл, найденный сканом источника.
#[derive(Debug, Clone)]
struct ScannedFile {
    abs: PathBuf,
    rel: String,
    size: u64,
    modified_ns: u128,
    file_identity: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct SourceSnapshotEntry<'a> {
    path: &'a str,
    size: u64,
    modified_ns: u128,
    file_identity: &'a str,
}

fn metadata_identity(path: &Path, _metadata: &std::fs::Metadata) -> String {
    #[cfg(unix)]
    {
        let _ = path;
        use std::os::unix::fs::MetadataExt;
        format!("dev:{}:ino:{}", _metadata.dev(), _metadata.ino())
    }
    #[cfg(not(unix))]
    {
        #[cfg(windows)]
        {
            use std::os::windows::io::AsRawHandle;
            use windows::Win32::Foundation::HANDLE;
            use windows::Win32::Storage::FileSystem::{
                GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION,
            };
            let Ok(file) = std::fs::File::open(path) else {
                return String::new();
            };
            let mut info = BY_HANDLE_FILE_INFORMATION::default();
            let handle = HANDLE(file.as_raw_handle());
            if unsafe { GetFileInformationByHandle(handle, &mut info) }.is_err() {
                return String::new();
            }
            format!(
                "volume:{:08x}:file:{:08x}{:08x}",
                info.dwVolumeSerialNumber, info.nFileIndexHigh, info.nFileIndexLow
            )
        }
        #[cfg(not(windows))]
        {
            String::new()
        }
    }
}

fn modified_ns(metadata: &std::fs::Metadata) -> u128 {
    metadata
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|duration| duration.as_nanos())
        .unwrap_or_default()
}

fn matches_scanned_identity(file: &ScannedFile) -> bool {
    std::fs::metadata(&file.abs).is_ok_and(|metadata| {
        metadata.len() == file.size
            && modified_ns(&metadata) == file.modified_ns
            && metadata_identity(&file.abs, &metadata) == file.file_identity
    })
}

fn source_looks_like_mixed_selection(root: &Path) -> bool {
    let Ok(entries) = std::fs::read_dir(root) else {
        return false;
    };
    let mut has_file = false;
    let mut has_directory = false;
    for entry in entries.flatten() {
        let Ok(kind) = entry.file_type() else {
            continue;
        };
        has_file |= kind.is_file();
        has_directory |= kind.is_dir();
        if has_file && has_directory {
            return true;
        }
    }
    false
}

fn preflight_destinations(
    destinations: &[PathBuf],
    required_bytes: u64,
) -> Result<Vec<DestinationPreflight>> {
    destinations
        .iter()
        .map(|destination| {
            let space =
                crate::offload::volume::get_volume_space(destination).with_context(|| {
                    format!(
                        "Cannot determine free space for destination {:?}",
                        destination
                    )
                })?;
            if space.available_bytes < required_bytes {
                bail!(
                    "Insufficient space on {:?}: {} available, {} required before copy",
                    destination,
                    space.available_bytes,
                    required_bytes
                );
            }
            Ok(DestinationPreflight {
                destination: destination.display().to_string(),
                required_bytes,
                available_bytes: space.available_bytes,
            })
        })
        .collect()
}

/// A new job reserves enough space for the complete source before reading it.
/// A resume cannot use that total: already verified files legitimately occupy
/// destination space. The copy engine still checks every remaining write before
/// opening its writer.
fn initial_destination_preflight(
    is_resume: bool,
    destinations: &[PathBuf],
    source_bytes: u64,
) -> Result<Vec<DestinationPreflight>> {
    if is_resume {
        Ok(Vec::new())
    } else {
        preflight_destinations(destinations, source_bytes)
    }
}

#[cfg(target_os = "macos")]
fn auto_eject_destinations(volumes: &[VolumeIdentity]) -> Vec<String> {
    let mut warnings = Vec::new();
    for volume in volumes {
        let Some(device) = volume.key.strip_prefix("mac-physical:") else {
            warnings.push(format!(
                "Auto-eject skipped for {}: physical disk identity is unavailable",
                volume.path
            ));
            continue;
        };
        match std::process::Command::new("diskutil")
            .args(["eject", device])
            .output()
        {
            Ok(output) if output.status.success() => {}
            Ok(output) => warnings.push(format!(
                "Auto-eject failed for {}: {}",
                volume.path,
                String::from_utf8_lossy(&output.stderr).trim()
            )),
            Err(error) => warnings.push(format!("Auto-eject failed for {}: {error}", volume.path)),
        }
    }
    warnings
}

#[cfg(not(target_os = "macos"))]
fn auto_eject_destinations(_volumes: &[VolumeIdentity]) -> Vec<String> {
    vec!["Auto-eject is currently supported only on macOS; destinations remain mounted.".into()]
}

/// Рекурсивный скан источника с ignore-паттернами MHL (.DS_Store, ascmhl/ и т.п.).
fn scan_source(root: &Path) -> Result<Vec<ScannedFile>> {
    let patterns: Vec<String> = mhl::DEFAULT_IGNORE_PATTERNS
        .iter()
        .map(|s| s.to_string())
        .collect();
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];

    while let Some(dir) = stack.pop() {
        let entries = std::fs::read_dir(&dir)
            .with_context(|| format!("Cannot read source directory {:?}", dir))?;
        for entry in entries {
            let entry = entry?;
            let path = entry.path();
            if entry.file_type()?.is_symlink() {
                continue;
            }
            let rel_path = path
                .strip_prefix(root)
                .expect("entry is under root by construction");
            let rel = rel_path
                .to_str()
                .with_context(|| format!("Source path is not valid UTF-8: {:?}", rel_path))?
                .replace('\\', "/");
            validate_portable_relative_path(&rel)?;
            if mhl::should_ignore(&rel, &patterns) {
                continue;
            }
            let meta = entry.metadata()?;
            if meta.is_dir() {
                stack.push(path);
            } else if meta.is_file() {
                let file_identity = metadata_identity(&path, &meta);
                out.push(ScannedFile {
                    abs: path,
                    rel,
                    size: meta.len(),
                    modified_ns: modified_ns(&meta),
                    file_identity,
                });
            }
            // Симлинки пропускаем: на картах их не бывает, а follow — риск выйти за root.
        }
    }

    // Стабильный порядок: линейное чтение карты + детерминированный MHL.
    out.sort_by(|a, b| a.rel.cmp(&b.rel));
    Ok(out)
}

fn validate_portable_relative_path(rel_path: &str) -> Result<()> {
    const RESERVED: &[&str] = &["CON", "PRN", "AUX", "NUL"];
    for component in rel_path.split('/') {
        if component.is_empty() || component == "." || component == ".." {
            bail!("Unsafe source path component in {rel_path}");
        }
        if component.ends_with(['.', ' '])
            || component.chars().any(|character| {
                matches!(character, '<' | '>' | ':' | '"' | '|' | '?' | '*')
                    || character.is_control()
            })
        {
            bail!("Source path is not portable to Windows destinations: {rel_path}");
        }
        let stem = component
            .split('.')
            .next()
            .unwrap_or_default()
            .to_ascii_uppercase();
        let numbered_reserved = stem.len() == 4
            && (stem.starts_with("COM") || stem.starts_with("LPT"))
            && stem.as_bytes()[3].is_ascii_digit()
            && stem.as_bytes()[3] != b'0';
        if RESERVED.contains(&stem.as_str()) || numbered_reserved {
            bail!("Source path uses a Windows-reserved name: {rel_path}");
        }
    }
    Ok(())
}

fn collision_key(path: &str) -> String {
    path.nfc().flat_map(char::to_lowercase).collect()
}

fn ensure_destination_directory(destination: &Path) -> Result<()> {
    if destination.exists() {
        if destination.is_dir() {
            return Ok(());
        }
        bail!("Destination exists but is not a directory: {destination:?}");
    }

    // Do not create a look-alike mount point under /Volumes if a removable
    // device was unplugged. Once the mounted volume exists, nested project
    // folders are safe and expected to be created automatically.
    #[cfg(target_os = "macos")]
    if let Ok(relative) = destination.strip_prefix("/Volumes") {
        let Some(Component::Normal(volume_name)) = relative.components().next() else {
            bail!("Destination must be inside a mounted volume: {destination:?}");
        };
        let mount_point = Path::new("/Volumes").join(volume_name);
        if !mount_point.is_dir() {
            bail!(
                "Destination volume is not mounted: {}. Reconnect the device before offload.",
                mount_point.display()
            );
        }
    }

    std::fs::create_dir_all(destination)
        .with_context(|| format!("Failed to create destination directory {destination:?}"))?;
    Ok(())
}

fn user_facing_storage_message(message: &str) -> String {
    let lower = message.to_ascii_lowercase();
    if lower.contains("device not configured") || lower.contains("os error 6") {
        "A storage device was disconnected. Reconnect the source or destination, then resume the same job."
            .into()
    } else {
        message.into()
    }
}

fn humanize_offload_error(error: anyhow::Error) -> anyhow::Error {
    let detailed = format!("{error:#}");
    let user_facing = user_facing_storage_message(&detailed);
    if user_facing == detailed {
        error
    } else {
        anyhow::anyhow!(user_facing)
    }
}

fn readback_concurrency(distinct_volume_count: usize, destination_count: usize) -> usize {
    if distinct_volume_count == destination_count {
        destination_count.clamp(1, 4)
    } else {
        1
    }
}

async fn acquire_unique_device_permits(
    scheduler: &IoScheduler,
    paths: &[&Path],
) -> Result<Vec<DevicePermit>> {
    let mut queues: Vec<&DeviceQueue> = paths
        .iter()
        .filter_map(|path| scheduler.get_device_queue(path))
        .collect();
    queues.sort_by_key(|queue| *queue as *const DeviceQueue as usize);
    queues.dedup_by_key(|queue| *queue as *const DeviceQueue as usize);
    let mut permits = Vec::with_capacity(queues.len());
    for queue in queues {
        permits.push(queue.acquire().await?);
    }
    Ok(permits)
}

fn register_destination_volume(
    keys: &mut HashSet<String>,
    key: &str,
    enforce_independence: bool,
) -> Result<bool> {
    if keys.insert(key.to_owned()) {
        return Ok(true);
    }
    if enforce_independence {
        bail!(
            "ArchiveMax destinations are on the same physical volume ({key}); they do not count as independent backups"
        );
    }
    Ok(false)
}

#[derive(Clone, Copy)]
struct SafetyGate {
    archive_verified: bool,
    destination_count: usize,
    distinct_destination_count: usize,
    physical_destination_count: usize,
    write_mhl: bool,
    mhl_count: usize,
    evidence_persisted: bool,
    warning_count: usize,
}

impl SafetyGate {
    fn allows_format(&self) -> bool {
        self.archive_verified
            && self.destination_count >= 2
            && self.distinct_destination_count == self.destination_count
            && self.physical_destination_count >= 2
            && self.write_mhl
            && self.mhl_count == self.destination_count
            && self.evidence_persisted
            && self.warning_count == 0
    }
}

fn hashes_to_task(results: &[HashResult]) -> checkpoint::TaskHashes {
    let mut th = checkpoint::TaskHashes::default();
    for r in results {
        let hex = Some(r.hex_digest.clone());
        match r.algorithm {
            HashAlgorithm::XXH64 => th.xxh64 = hex,
            HashAlgorithm::XXH3 => th.xxh3 = hex,
            HashAlgorithm::XXH128 => th.xxh128 = hex,
            HashAlgorithm::BLAKE3 => th.blake3 = hex,
            HashAlgorithm::SHA256 => th.sha256 = hex,
            HashAlgorithm::MD5 => th.md5 = hex,
        }
    }
    th
}

fn persist_observation(
    conn: Option<&rusqlite::Connection>,
    job_id: &str,
    observation: &HashObservation,
) -> Result<()> {
    if let Some(conn) = conn {
        checkpoint::append_hash_observation(
            conn,
            job_id,
            &observation.side,
            &observation.file,
            observation.destination.as_deref(),
            &serde_json::to_string(observation)?,
        )?;
    }
    Ok(())
}

fn persist_replica(
    conn: Option<&rusqlite::Connection>,
    job_id: &str,
    replica: &ReplicaEvidence,
) -> Result<()> {
    if let Some(conn) = conn {
        checkpoint::upsert_replica_state(
            conn,
            job_id,
            &replica.file,
            &replica.destination,
            &serde_json::to_string(replica)?,
        )?;
    }
    Ok(())
}

fn persist_repair(
    conn: Option<&rusqlite::Connection>,
    job_id: &str,
    repair: &RepairAttempt,
) -> Result<()> {
    if let Some(conn) = conn {
        checkpoint::append_repair_attempt(
            conn,
            job_id,
            &repair.file,
            &repair.destination,
            repair.attempt,
            &serde_json::to_string(repair)?,
        )?;
    }
    Ok(())
}

fn deserialize_payloads<T: serde::de::DeserializeOwned>(payloads: Vec<String>) -> Result<Vec<T>> {
    payloads
        .into_iter()
        .map(|payload| serde_json::from_str(&payload).map_err(Into::into))
        .collect()
}

fn hashes_match(expected: &[HashResult], actual: &[HashResult]) -> bool {
    expected.len() == actual.len()
        && expected.iter().all(|left| {
            actual.iter().any(|right| {
                left.algorithm == right.algorithm
                    && left.hex_digest.eq_ignore_ascii_case(&right.hex_digest)
            })
        })
}

async fn hash_with_runtime_control(
    path: &Path,
    algorithms: &[HashAlgorithm],
    cancel: Option<&AtomicBool>,
    pause: Option<&AtomicBool>,
) -> Result<Vec<HashResult>> {
    let config = HashEngineConfig {
        algorithms: algorithms.to_vec(),
        buffer_size: 4 * 1024 * 1024,
    };
    let control = HashControl {
        cancel_flag: cancel,
        pause_flag: pause,
        on_progress: None,
    };
    hash_file_with_control(path, &config, &control).await
}

/// Прогнать полный оффлоад. `on_progress` дёргается на каждом файле и батчами байт.
pub async fn run_offload(
    req: &OffloadRequest,
    cancel: Option<&AtomicBool>,
    pause: Option<&AtomicBool>,
    on_progress: &(dyn Fn(OffloadProgress) + Send + Sync),
) -> Result<OffloadSummary> {
    let result = run_offload_inner(req, cancel, pause, on_progress).await;
    if let Err(error) = &result {
        if let (Some(db_path), Some(job_id)) = (&req.checkpoint_db, &req.job_id) {
            if let Ok(conn) = checkpoint::open_db(db_path) {
                if checkpoint::get_job(&conn, job_id).ok().flatten().is_some() {
                    let status = if cancel
                        .is_some_and(|flag| flag.load(std::sync::atomic::Ordering::SeqCst))
                    {
                        "terminated"
                    } else {
                        "failed"
                    };
                    let _ = checkpoint::update_job_status(&conn, job_id, status);
                    let _ = checkpoint::append_job_event(
                        &conn,
                        job_id,
                        if status == "terminated" {
                            "jobTerminated"
                        } else {
                            "jobFailed"
                        },
                        &serde_json::json!({ "error": format!("{error:#}") }).to_string(),
                    );
                }
            }
        }
    }
    result.map_err(humanize_offload_error)
}

async fn run_offload_inner(
    req: &OffloadRequest,
    cancel: Option<&AtomicBool>,
    pause: Option<&AtomicBool>,
    on_progress: &(dyn Fn(OffloadProgress) + Send + Sync),
) -> Result<OffloadSummary> {
    let started_at = chrono::Utc::now();
    // ── Валидация ──────────────────────────────────────────────────────────
    if !req.source.is_dir() {
        bail!("Source is not a directory: {:?}", req.source);
    }
    if req.destinations.is_empty() {
        bail!("No destinations specified");
    }
    if !(1..=8).contains(&req.small_file_concurrency) {
        bail!("small_file_concurrency must be in the range 1..=8");
    }
    let mut warnings = Vec::new();
    if req.profile == VerificationProfile::ArchiveMax && req.checkpoint_db.is_none() {
        warnings.push(
            "Local checkpoint/evidence database is disabled; SAFE_TO_FORMAT is unavailable".into(),
        );
    }
    let source_volume = identify_volume(&req.source)?;
    let mut destination_volume_keys = HashSet::new();
    let mut physical_destination_keys = HashSet::new();
    let mut destination_volumes = Vec::new();
    for dest in &req.destinations {
        // Получатель внутри источника = бесконечная рекурсия; источник внутри
        // получателя = риск самоперезаписи. Оба запрещены.
        if dest.starts_with(&req.source) || req.source.starts_with(dest) {
            bail!(
                "Destination {:?} overlaps with source {:?}",
                dest,
                req.source
            );
        }
        ensure_destination_directory(dest)?;
        if req.profile == VerificationProfile::ArchiveMax {
            let identity = identify_volume(dest)?;
            if identity.key == source_volume.key {
                warnings.push(format!(
                    "Destination {} is on the same physical volume as the source",
                    dest.display()
                ));
            }
            if !register_destination_volume(
                &mut destination_volume_keys,
                &identity.key,
                cfg!(not(test)),
            )? {
                let warning = format!(
                    "ArchiveMax destinations are on the same physical volume ({}); they do not count as independent backups",
                    identity.key
                );
                warnings.push(warning);
            }
            if identity.is_physical {
                physical_destination_keys.insert(identity.key.clone());
            } else {
                warnings.push(format!(
                    "Destination {} could not be proven to be a physical device; it does not authorize source formatting",
                    dest.display()
                ));
            }
            destination_volumes.push(identity);
        } else {
            destination_volumes.push(identify_volume(dest)?);
        }
    }

    // Every source/destination path is routed through one queue per physical
    // device key. Aliases on the same disk share a semaphore.
    let mut io_scheduler = IoScheduler::new();
    let mut queues_by_device: HashMap<String, std::sync::Arc<DeviceQueue>> = HashMap::new();
    let source_queue = io_scheduler.register_device(
        req.source.clone(),
        DeviceSchedulerConfig::default_for(source_volume.device_type),
    );
    queues_by_device.insert(source_volume.key.clone(), source_queue);
    for (destination, identity) in req.destinations.iter().zip(&destination_volumes) {
        if let Some(queue) = queues_by_device.get(&identity.key) {
            io_scheduler.register_alias(destination.clone(), std::sync::Arc::clone(queue));
        } else {
            let queue = io_scheduler.register_device(
                destination.clone(),
                DeviceSchedulerConfig::default_for(identity.device_type),
            );
            queues_by_device.insert(identity.key.clone(), queue);
        }
    }

    // Persist the stable job identity before scanning. A process killed in the
    // scan phase must still be resumable under the same job ID.
    let job_id = req
        .job_id
        .clone()
        .unwrap_or_else(|| format!("job-{}", chrono::Utc::now().format("%Y%m%d-%H%M%S%.3f")));
    let (conn, is_resume) = match &req.checkpoint_db {
        Some(db_path) => {
            let conn = checkpoint::open_db(db_path)?;
            let existed = checkpoint::get_job(&conn, &job_id)?.is_some();
            let config_json = serde_json::to_string(req)?;
            checkpoint::create_job(
                &conn,
                &job_id,
                &format!("Offload {}", req.source.display()),
                &req.source.to_string_lossy(),
                Some(&config_json),
            )?;
            checkpoint::update_job_status(&conn, &job_id, "running")?;
            checkpoint::append_job_event(
                &conn,
                &job_id,
                if existed { "jobResumed" } else { "jobStarted" },
                &serde_json::json!({
                    "profile": req.profile,
                    "source": req.source,
                    "destinations": req.destinations,
                })
                .to_string(),
            )?;
            (Some(conn), existed)
        }
        None => (None, false),
    };

    // ── Скан ───────────────────────────────────────────────────────────────
    on_progress(OffloadProgress {
        phase: "scanning".into(),
        current_file: String::new(),
        file_index: 0,
        total_files: 0,
        bytes_done: 0,
        bytes_total: 0,
    });
    let files = scan_source(&req.source)?;
    if files.is_empty() {
        bail!("Source contains no files: {:?}", req.source);
    }
    let mut normalized_paths = HashSet::new();
    for file in &files {
        let key = collision_key(&file.rel);
        if !normalized_paths.insert(key) {
            bail!(
                "Source contains paths that collide on a case-insensitive destination: {}",
                file.rel
            );
        }
        if file.size == 0 {
            warnings.push(format!(
                "Zero-byte source file requires operator review: {}",
                file.rel
            ));
        }
    }
    if source_looks_like_mixed_selection(&req.source) {
        warnings.push(
            "Source folder mixes loose files with subfolders. Confirm that it is the complete card or shoot folder, not a cherry-picked selection; SAFE_TO_FORMAT is withheld until reviewed."
                .into(),
        );
    }
    let bytes_total: u64 = files.iter().map(|f| f.size).sum();
    let source_snapshot = files
        .iter()
        .map(|file| SourceSnapshotEntry {
            path: &file.rel,
            size: file.size,
            modified_ns: file.modified_ns,
            file_identity: &file.file_identity,
        })
        .collect::<Vec<_>>();
    let source_snapshot_json = serde_json::to_string(&source_snapshot)?;

    // New jobs reserve the complete source before their first read. A resume
    // validates each remaining write in copy_engine instead: already verified
    // files occupy destination space and must not make recovery impossible.
    let destination_preflight =
        initial_destination_preflight(is_resume, &req.destinations, bytes_total)?;
    let report_contacts = req
        .report_contacts
        .clone()
        .into_iter()
        .filter_map(DitContact::normalized)
        .collect::<Vec<_>>();

    // ── Checkpoint job ─────────────────────────────────────────────────────
    if let Some(conn) = &conn {
        let destination_keys = destination_volumes
            .iter()
            .map(|identity| identity.fingerprint.as_str())
            .collect::<Vec<_>>();
        checkpoint::bind_or_validate_job_context(
            conn,
            &job_id,
            &source_volume.fingerprint,
            &serde_json::to_string(&destination_keys)?,
            &source_snapshot_json,
        )?;
        if req.job_id.is_some() {
            checkpoint::recover_job(conn, &job_id)?;
        }
        checkpoint::update_job_status(conn, &job_id, "running")?;
    }

    // ── Копирование ────────────────────────────────────────────────────────
    let hash_policy = HashPolicy::for_request(req);
    let algorithms = hash_policy.evidence_algorithms.clone();
    let copy_config = CopyEngineConfig {
        hash_algorithms: algorithms.clone(),
        ..Default::default()
    };

    let mut copied = 0usize;
    let mut skipped = 0usize;
    let mut failures: Vec<OffloadFailure> = Vec::new();
    let mut bytes_copied = 0u64;
    let mut bytes_done_before_file = 0u64;
    // rel_path -> hashes источника (для MHL); per-dest список реально скопированных rel.
    let mut source_hashes: HashMap<String, Vec<HashResult>> = HashMap::new();
    let mut copied_per_dest: Vec<Vec<String>> = vec![Vec::new(); req.destinations.len()];
    let mut observations = Vec::new();
    let mut replicas = Vec::new();
    let mut repairs = Vec::new();
    let mut verified_replicas = 0usize;
    let mut verification_failed = 0usize;

    // Fast may parallelize only small files from an SSD source. Per-device
    // semaphores still serialize aliases and cap SSD/NVMe at four active tasks;
    // the policy also enforces the global 512 MiB budget.
    const SMALL_FILE_LIMIT: u64 = 16 * 1024 * 1024;
    let destination_configs = destination_volumes
        .iter()
        .map(|identity| DeviceSchedulerConfig::default_for(identity.device_type))
        .collect::<Vec<_>>();
    let small_file_workers = if req.profile == VerificationProfile::Fast
        && source_volume.device_type == crate::offload::volume::DeviceType::SSD
    {
        SchedulerPolicy {
            memory_budget_bytes: 512 * 1024 * 1024,
            requested_workers: req.small_file_concurrency,
        }
        .effective_workers(source_volume.device_type, &destination_configs)
    } else {
        1
    };
    let parallel_file_indices = if req.profile == VerificationProfile::Fast
        && source_volume.device_type == crate::offload::volume::DeviceType::SSD
        && small_file_workers > 1
    {
        files
            .iter()
            .enumerate()
            .filter_map(|(index, file)| (file.size <= SMALL_FILE_LIMIT).then_some(index))
            .collect::<HashSet<_>>()
    } else {
        HashSet::new()
    };

    if !parallel_file_indices.is_empty() {
        let mut work = Vec::with_capacity(parallel_file_indices.len());
        for &index in &parallel_file_indices {
            let file = &files[index];
            let dest_paths = req
                .destinations
                .iter()
                .map(|destination| destination.join(&file.rel))
                .collect::<Vec<_>>();
            let task_ids = (0..dest_paths.len())
                .map(|destination_index| format!("{}-f{}-d{}", job_id, index, destination_index))
                .collect::<Vec<_>>();
            if let Some(conn) = &conn {
                for (destination_index, dest_path) in dest_paths.iter().enumerate() {
                    checkpoint::insert_task(
                        conn,
                        &task_ids[destination_index],
                        &job_id,
                        &file.abs.to_string_lossy(),
                        &dest_path.to_string_lossy(),
                        file.size,
                    )?;
                    checkpoint::update_task_status(
                        conn,
                        &task_ids[destination_index],
                        checkpoint::STATUS_COPYING,
                    )?;
                }
            }
            work.push((index, dest_paths, task_ids));
        }
        work.sort_by_key(|(index, _, _)| *index);
        let files_ref = &files;
        let total_files = files.len();

        type ParallelCopyOutcome = (
            usize,
            Vec<PathBuf>,
            Vec<String>,
            Result<Vec<CopyFileResult>>,
        );
        let outcomes: Vec<ParallelCopyOutcome> =
            stream::iter(work.into_iter().map(|(index, dest_paths, task_ids)| {
                let file = &files_ref[index];
                let scheduler_ref = &io_scheduler;
                let copy_config_ref = &copy_config;
                async move {
                    on_progress(OffloadProgress {
                        phase: "copying".into(),
                        current_file: file.rel.clone(),
                        file_index: index + 1,
                        total_files,
                        bytes_done: 0,
                        bytes_total,
                    });
                    let mut paths = Vec::with_capacity(dest_paths.len() + 1);
                    paths.push(file.abs.as_path());
                    paths.extend(dest_paths.iter().map(PathBuf::as_path));
                    let result = match acquire_unique_device_permits(scheduler_ref, &paths).await {
                        Ok(permits) => {
                            let control = CopyControl {
                                cancel_flag: cancel,
                                pause_flag: pause,
                                on_progress: Some(Box::new(|written, total| {
                                    on_progress(OffloadProgress {
                                        phase: "copyingData".into(),
                                        current_file: file.rel.clone(),
                                        file_index: index + 1,
                                        total_files,
                                        bytes_done: written,
                                        bytes_total: total,
                                    });
                                })),
                            };
                            let result =
                                copy_file_multi(&file.abs, &dest_paths, copy_config_ref, &control)
                                    .await;
                            drop(permits);
                            result
                        }
                        Err(error) => Err(error),
                    };
                    (index, dest_paths, task_ids, result)
                }
            }))
            .buffered(small_file_workers)
            .collect()
            .await;

        for (index, dest_paths, task_ids, outcome) in outcomes {
            let file = &files[index];
            match outcome {
                Ok(results) => {
                    let mut copy_hashes = results
                        .iter()
                        .find(|result| !result.hash_results.is_empty())
                        .map(|result| result.hash_results.clone())
                        .unwrap_or_default();
                    if copy_hashes.is_empty() {
                        let _permits =
                            acquire_unique_device_permits(&io_scheduler, &[&file.abs]).await?;
                        copy_hashes =
                            hash_with_runtime_control(&file.abs, &algorithms, cancel, pause)
                                .await?;
                    }
                    let observation = HashObservation {
                        side: "sourceCopyRead".into(),
                        file: file.rel.clone(),
                        destination: None,
                        hashes: copy_hashes.clone(),
                    };
                    persist_observation(conn.as_ref(), &job_id, &observation)?;
                    observations.push(observation);
                    source_hashes.insert(file.rel.clone(), copy_hashes.clone());

                    let mut file_all_ok = true;
                    let all_skipped = results
                        .iter()
                        .all(|result| result.success && result.skipped);
                    for (destination_index, result) in results.iter().enumerate() {
                        let destination = &dest_paths[destination_index];
                        if !result.success {
                            file_all_ok = false;
                            let message = result
                                .error
                                .clone()
                                .unwrap_or_else(|| "Destination copy failed".into());
                            let message = user_facing_storage_message(&message);
                            if let Some(conn) = &conn {
                                checkpoint::update_task_failed(
                                    conn,
                                    &task_ids[destination_index],
                                    &message,
                                )?;
                            }
                            failures.push(OffloadFailure {
                                file: file.rel.clone(),
                                destination: Some(destination.display().to_string()),
                                error: message.clone(),
                            });
                            let replica = ReplicaEvidence {
                                file: file.rel.clone(),
                                bytes: file.size,
                                destination: destination.display().to_string(),
                                status: ReplicaState::CopyFailed,
                                expected_hashes: copy_hashes.clone(),
                                observed_hashes: Vec::new(),
                                repair_attempts: 0,
                                error: Some(message),
                            };
                            persist_replica(conn.as_ref(), &job_id, &replica)?;
                            replicas.push(replica);
                            continue;
                        }

                        if let Some(conn) = &conn {
                            let task_hashes = hashes_to_task(&copy_hashes);
                            if result.skipped {
                                checkpoint::update_task_skipped(
                                    conn,
                                    &task_ids[destination_index],
                                    &task_hashes,
                                )?;
                            } else {
                                checkpoint::update_task_completed(
                                    conn,
                                    &task_ids[destination_index],
                                    &task_hashes,
                                )?;
                            }
                        }
                        copied_per_dest[destination_index].push(file.rel.clone());
                        let replica = ReplicaEvidence {
                            file: file.rel.clone(),
                            bytes: file.size,
                            destination: destination.display().to_string(),
                            status: if result.skipped {
                                ReplicaState::AlreadyMatched
                            } else {
                                ReplicaState::CopyComplete
                            },
                            expected_hashes: copy_hashes.clone(),
                            observed_hashes: Vec::new(),
                            repair_attempts: 0,
                            error: None,
                        };
                        persist_replica(conn.as_ref(), &job_id, &replica)?;
                        replicas.push(replica);
                    }
                    if file_all_ok {
                        if all_skipped {
                            skipped += 1;
                        } else {
                            copied += 1;
                            bytes_copied += file.size;
                        }
                    }
                }
                Err(error) => {
                    let cancelled =
                        cancel.is_some_and(|flag| flag.load(std::sync::atomic::Ordering::SeqCst));
                    if let Some(conn) = &conn {
                        for task_id in &task_ids {
                            checkpoint::update_task_status(
                                conn,
                                task_id,
                                if cancelled {
                                    checkpoint::STATUS_TERMINATED
                                } else {
                                    checkpoint::STATUS_FAILED
                                },
                            )?;
                        }
                    }
                    if cancelled {
                        return Err(error.context("Offload cancelled"));
                    }
                    failures.push(OffloadFailure {
                        file: file.rel.clone(),
                        destination: None,
                        error: user_facing_storage_message(&format!("{error:#}")),
                    });
                }
            }
        }
        bytes_done_before_file = parallel_file_indices
            .iter()
            .map(|index| files[*index].size)
            .sum();
    }

    for (idx, file) in files.iter().enumerate() {
        if parallel_file_indices.contains(&idx) {
            continue;
        }
        let source_pre_hashes = if req.profile == VerificationProfile::ArchiveMax {
            on_progress(OffloadProgress {
                phase: "sourcePreRead".into(),
                current_file: file.rel.clone(),
                file_index: idx + 1,
                total_files: files.len(),
                bytes_done: 0,
                bytes_total: file.size,
            });
            let _permits = acquire_unique_device_permits(&io_scheduler, &[&file.abs]).await?;
            let hashes = hash_with_runtime_control(&file.abs, &algorithms, cancel, pause).await?;
            let observation = HashObservation {
                side: "sourcePreRead".into(),
                file: file.rel.clone(),
                destination: None,
                hashes: hashes.clone(),
            };
            persist_observation(conn.as_ref(), &job_id, &observation)?;
            observations.push(observation);
            hashes
        } else {
            Vec::new()
        };

        on_progress(OffloadProgress {
            phase: "copying".into(),
            current_file: file.rel.clone(),
            file_index: idx + 1,
            total_files: files.len(),
            bytes_done: bytes_done_before_file,
            bytes_total,
        });

        let dest_paths: Vec<PathBuf> = req.destinations.iter().map(|d| d.join(&file.rel)).collect();
        let task_ids: Vec<String> = (0..dest_paths.len())
            .map(|di| format!("{}-f{}-d{}", job_id, idx, di))
            .collect();
        if let Some(conn) = &conn {
            for (di, dest_path) in dest_paths.iter().enumerate() {
                checkpoint::insert_task(
                    conn,
                    &task_ids[di],
                    &job_id,
                    &file.abs.to_string_lossy(),
                    &dest_path.to_string_lossy(),
                    file.size,
                )?;
                checkpoint::update_task_status(conn, &task_ids[di], checkpoint::STATUS_COPYING)?;
            }
        }

        let control = CopyControl {
            cancel_flag: cancel,
            pause_flag: pause,
            on_progress: Some(Box::new(|written, total| {
                on_progress(OffloadProgress {
                    phase: "copyingData".into(),
                    current_file: file.rel.clone(),
                    file_index: idx + 1,
                    total_files: files.len(),
                    bytes_done: written,
                    bytes_total: total,
                });
            })),
        };

        let mut copy_paths = Vec::with_capacity(dest_paths.len() + 1);
        copy_paths.push(file.abs.as_path());
        copy_paths.extend(dest_paths.iter().map(PathBuf::as_path));
        let copy_permits = acquire_unique_device_permits(&io_scheduler, &copy_paths).await?;
        let copy_result = copy_file_multi(&file.abs, &dest_paths, &copy_config, &control).await;
        drop(copy_permits);
        match copy_result {
            Ok(results) => {
                let mut copy_hashes = results
                    .iter()
                    .find(|result| !result.hash_results.is_empty())
                    .map(|result| result.hash_results.clone())
                    .unwrap_or_default();
                // If every destination was already present, the copy engine did
                // not need to read the source. ArchiveMax still requires a second,
                // independent source read before trusting those replicas.
                if copy_hashes.is_empty()
                    && (req.profile == VerificationProfile::ArchiveMax || req.write_mhl)
                {
                    copy_hashes =
                        hash_with_runtime_control(&file.abs, &algorithms, cancel, pause).await?;
                }
                if !copy_hashes.is_empty() {
                    let observation = HashObservation {
                        side: "sourceCopyRead".into(),
                        file: file.rel.clone(),
                        destination: None,
                        hashes: copy_hashes.clone(),
                    };
                    persist_observation(conn.as_ref(), &job_id, &observation)?;
                    observations.push(observation);
                }

                let source_stable = req.profile != VerificationProfile::ArchiveMax
                    || (hashes_match(&source_pre_hashes, &copy_hashes)
                        && matches_scanned_identity(file));
                let expected_hashes = if req.profile == VerificationProfile::ArchiveMax {
                    source_pre_hashes.clone()
                } else {
                    copy_hashes.clone()
                };
                if !expected_hashes.is_empty() {
                    source_hashes.insert(file.rel.clone(), expected_hashes.clone());
                }

                let mut file_all_ok = source_stable;
                let all_skipped = results
                    .iter()
                    .all(|result| result.success && result.skipped);

                if !source_stable {
                    let message =
                        "Source changed or produced different bytes between independent reads";
                    for (di, dest_path) in dest_paths.iter().enumerate() {
                        if let Some(conn) = &conn {
                            checkpoint::update_task_failed(conn, &task_ids[di], message)?;
                        }
                        failures.push(OffloadFailure {
                            file: file.rel.clone(),
                            destination: Some(dest_path.display().to_string()),
                            error: message.into(),
                        });
                        verification_failed += 1;
                        let replica = ReplicaEvidence {
                            file: file.rel.clone(),
                            bytes: file.size,
                            destination: dest_path.display().to_string(),
                            status: ReplicaState::SourceChanged,
                            expected_hashes: source_pre_hashes.clone(),
                            observed_hashes: copy_hashes.clone(),
                            repair_attempts: 0,
                            error: Some(message.into()),
                        };
                        persist_replica(conn.as_ref(), &job_id, &replica)?;
                        replicas.push(replica);
                    }
                } else {
                    // Readback is the useful concurrency boundary: each future
                    // touches a different physical destination and never adds
                    // source-card seeks. Same-volume test setups stay serial.
                    // Four-megabyte buffers are capped by the 512 MiB policy.
                    let max_readbacks =
                        readback_concurrency(destination_volume_keys.len(), req.destinations.len());
                    let mut initial_readbacks = if req.profile == VerificationProfile::ArchiveMax {
                        let algorithms_ref = algorithms.as_slice();
                        let file_rel = file.rel.as_str();
                        let file_size = file.size;
                        let total_files = files.len();
                        let dest_paths_ref = dest_paths.as_slice();
                        let readback_successes = results
                            .iter()
                            .map(|result| result.success)
                            .collect::<Vec<_>>();
                        let readback_successes_ref = readback_successes.as_slice();
                        let scheduler_ref = &io_scheduler;
                        stream::iter((0..results.len()).map(move |di| {
                            let dest_path = &dest_paths_ref[di];
                            let result_success = readback_successes_ref[di];
                            async move {
                                if !result_success {
                                    return None;
                                }
                                let _permits = match acquire_unique_device_permits(
                                    scheduler_ref,
                                    &[dest_path],
                                )
                                .await
                                {
                                    Ok(permits) => permits,
                                    Err(error) => return Some(Err(error)),
                                };
                                on_progress(OffloadProgress {
                                    phase: "destinationVerify".into(),
                                    current_file: format!("{} → {}", file_rel, dest_path.display()),
                                    file_index: idx + 1,
                                    total_files,
                                    bytes_done: 0,
                                    bytes_total: file_size,
                                });
                                Some(
                                    hash_with_runtime_control(
                                        dest_path,
                                        algorithms_ref,
                                        cancel,
                                        pause,
                                    )
                                    .await,
                                )
                            }
                        }))
                        .buffered(max_readbacks)
                        .collect::<Vec<_>>()
                        .await
                    } else {
                        (0..results.len()).map(|_| None).collect()
                    };

                    for (di, result) in results.iter().enumerate() {
                        let dest_path = &dest_paths[di];
                        if !result.success {
                            let message = result
                                .error
                                .clone()
                                .unwrap_or_else(|| "Destination copy failed".into());
                            let message = user_facing_storage_message(&message);
                            if let Some(conn) = &conn {
                                checkpoint::update_task_failed(conn, &task_ids[di], &message)?;
                            }
                            failures.push(OffloadFailure {
                                file: file.rel.clone(),
                                destination: Some(dest_path.display().to_string()),
                                error: message.clone(),
                            });
                            let replica = ReplicaEvidence {
                                file: file.rel.clone(),
                                bytes: file.size,
                                destination: dest_path.display().to_string(),
                                status: ReplicaState::CopyFailed,
                                expected_hashes: expected_hashes.clone(),
                                observed_hashes: Vec::new(),
                                repair_attempts: 0,
                                error: Some(message),
                            };
                            persist_replica(conn.as_ref(), &job_id, &replica)?;
                            replicas.push(replica);
                            file_all_ok = false;
                            continue;
                        }

                        if req.profile == VerificationProfile::Fast {
                            let task_hashes = hashes_to_task(&expected_hashes);
                            if let Some(conn) = &conn {
                                if result.skipped {
                                    checkpoint::update_task_skipped(
                                        conn,
                                        &task_ids[di],
                                        &task_hashes,
                                    )?;
                                } else {
                                    checkpoint::update_task_completed(
                                        conn,
                                        &task_ids[di],
                                        &task_hashes,
                                    )?;
                                }
                            }
                            copied_per_dest[di].push(file.rel.clone());
                            let replica = ReplicaEvidence {
                                file: file.rel.clone(),
                                bytes: file.size,
                                destination: dest_path.display().to_string(),
                                status: if result.skipped {
                                    ReplicaState::AlreadyMatched
                                } else {
                                    ReplicaState::CopyComplete
                                },
                                expected_hashes: expected_hashes.clone(),
                                observed_hashes: Vec::new(),
                                repair_attempts: 0,
                                error: None,
                            };
                            persist_replica(conn.as_ref(), &job_id, &replica)?;
                            replicas.push(replica);
                            continue;
                        }

                        if let Some(conn) = &conn {
                            checkpoint::update_task_status(
                                conn,
                                &task_ids[di],
                                checkpoint::STATUS_VERIFYING,
                            )?;
                        }
                        let mut observed = match initial_readbacks[di]
                            .take()
                            .expect("ArchiveMax readback exists for every successful destination")
                        {
                            Ok(hashes) => hashes,
                            Err(error) => {
                                if cancel.is_some_and(|flag| {
                                    flag.load(std::sync::atomic::Ordering::SeqCst)
                                }) {
                                    return Err(error.context("Offload cancelled"));
                                }
                                Vec::new()
                            }
                        };
                        let mut repair_count = match &conn {
                            Some(conn) => checkpoint::repair_attempt_count(
                                conn,
                                &job_id,
                                &file.rel,
                                &dest_path.display().to_string(),
                            )?,
                            None => 0,
                        };

                        while !hashes_match(&expected_hashes, &observed) && repair_count < 2 {
                            // A repair source must itself match the immutable
                            // source evidence. Prefer the card, but if it is no
                            // longer readable use another independently checked
                            // replica. Never copy from an unchecked destination.
                            let mut repair_source = None;
                            for candidate in std::iter::once(&file.abs).chain(
                                dest_paths
                                    .iter()
                                    .enumerate()
                                    .filter(|(candidate_index, _)| *candidate_index != di)
                                    .map(|(_, path)| path),
                            ) {
                                let _permits = acquire_unique_device_permits(
                                    &io_scheduler,
                                    &[candidate.as_path()],
                                )
                                .await?;
                                match hash_with_runtime_control(
                                    candidate,
                                    &algorithms,
                                    cancel,
                                    pause,
                                )
                                .await
                                {
                                    Ok(candidate_hashes)
                                        if hashes_match(&expected_hashes, &candidate_hashes) =>
                                    {
                                        repair_source = Some(candidate.clone());
                                        break;
                                    }
                                    Err(error)
                                        if cancel.is_some_and(|flag| {
                                            flag.load(std::sync::atomic::Ordering::SeqCst)
                                        }) =>
                                    {
                                        return Err(error.context("Offload cancelled"));
                                    }
                                    _ => {}
                                }
                            }
                            let Some(repair_source) = repair_source else {
                                break;
                            };
                            repair_count += 1;
                            on_progress(OffloadProgress {
                                phase: "repairing".into(),
                                current_file: format!("{} → {}", file.rel, dest_path.display()),
                                file_index: idx + 1,
                                total_files: files.len(),
                                bytes_done: 0,
                                bytes_total: file.size,
                            });
                            let repair_config = CopyEngineConfig {
                                conflict_policy: FileConflictPolicy::Overwrite,
                                hash_algorithms: algorithms.clone(),
                                ..copy_config.clone()
                            };
                            let _permits = acquire_unique_device_permits(
                                &io_scheduler,
                                &[repair_source.as_path(), dest_path.as_path()],
                            )
                            .await?;
                            let repair_control = CopyControl {
                                cancel_flag: cancel,
                                pause_flag: pause,
                                on_progress: Some(Box::new(|written, total| {
                                    on_progress(OffloadProgress {
                                        phase: "repairingData".into(),
                                        current_file: format!(
                                            "{} → {}",
                                            file.rel,
                                            dest_path.display()
                                        ),
                                        file_index: idx + 1,
                                        total_files: files.len(),
                                        bytes_done: written,
                                        bytes_total: total,
                                    });
                                })),
                            };
                            let repair_result = copy_file_single(
                                &repair_source,
                                dest_path,
                                &repair_config,
                                &repair_control,
                            )
                            .await;
                            let repair_ok = match repair_result {
                                Ok(ref copied_result)
                                    if hashes_match(
                                        &expected_hashes,
                                        &copied_result.hash_results,
                                    ) =>
                                {
                                    on_progress(OffloadProgress {
                                        phase: "repairReadback".into(),
                                        current_file: format!(
                                            "{} → {}",
                                            file.rel,
                                            dest_path.display()
                                        ),
                                        file_index: idx + 1,
                                        total_files: files.len(),
                                        bytes_done: 0,
                                        bytes_total: file.size,
                                    });
                                    observed = hash_with_runtime_control(
                                        dest_path,
                                        &algorithms,
                                        cancel,
                                        pause,
                                    )
                                    .await
                                    .unwrap_or_default();
                                    hashes_match(&expected_hashes, &observed)
                                }
                                _ => false,
                            };
                            let repair = RepairAttempt {
                                file: file.rel.clone(),
                                destination: dest_path.display().to_string(),
                                source: repair_source.display().to_string(),
                                attempt: repair_count,
                                success: repair_ok,
                                error: if repair_ok {
                                    None
                                } else {
                                    Some(
                                        "Repair copy or readback did not match source evidence"
                                            .into(),
                                    )
                                },
                            };
                            persist_repair(conn.as_ref(), &job_id, &repair)?;
                            repairs.push(repair);
                            if repair_ok {
                                break;
                            }
                        }

                        let observation = HashObservation {
                            side: "destinationReadback".into(),
                            file: file.rel.clone(),
                            destination: Some(dest_path.display().to_string()),
                            hashes: observed.clone(),
                        };
                        persist_observation(conn.as_ref(), &job_id, &observation)?;
                        observations.push(observation);

                        if hashes_match(&expected_hashes, &observed) {
                            verified_replicas += 1;
                            copied_per_dest[di].push(file.rel.clone());
                            if let Some(conn) = &conn {
                                checkpoint::update_task_completed(
                                    conn,
                                    &task_ids[di],
                                    &hashes_to_task(&expected_hashes),
                                )?;
                                if repair_count > 0 {
                                    checkpoint::append_retry_success(conn, &task_ids[di])?;
                                }
                            }
                            let replica = ReplicaEvidence {
                                file: file.rel.clone(),
                                bytes: file.size,
                                destination: dest_path.display().to_string(),
                                status: ReplicaState::Verified,
                                expected_hashes: expected_hashes.clone(),
                                observed_hashes: observed,
                                repair_attempts: repair_count,
                                error: None,
                            };
                            persist_replica(conn.as_ref(), &job_id, &replica)?;
                            replicas.push(replica);
                        } else {
                            let message =
                                "Destination readback hash mismatch after repair attempts";
                            verification_failed += 1;
                            file_all_ok = false;
                            if let Some(conn) = &conn {
                                checkpoint::update_task_failed(conn, &task_ids[di], message)?;
                            }
                            failures.push(OffloadFailure {
                                file: file.rel.clone(),
                                destination: Some(dest_path.display().to_string()),
                                error: message.into(),
                            });
                            let replica = ReplicaEvidence {
                                file: file.rel.clone(),
                                bytes: file.size,
                                destination: dest_path.display().to_string(),
                                status: ReplicaState::VerifyFailed,
                                expected_hashes: expected_hashes.clone(),
                                observed_hashes: observed,
                                repair_attempts: repair_count,
                                error: Some(message.into()),
                            };
                            persist_replica(conn.as_ref(), &job_id, &replica)?;
                            replicas.push(replica);
                        }
                    }
                }

                if file_all_ok {
                    if all_skipped {
                        skipped += 1;
                    } else {
                        copied += 1;
                        bytes_copied += file.size;
                    }
                }
            }
            Err(e) => {
                let is_cancel = cancel
                    .map(|c| c.load(std::sync::atomic::Ordering::SeqCst))
                    .unwrap_or(false);
                if let Some(conn) = &conn {
                    for tid in &task_ids {
                        let status = if is_cancel {
                            checkpoint::STATUS_TERMINATED
                        } else {
                            checkpoint::STATUS_FAILED
                        };
                        checkpoint::update_task_status(conn, tid, status)?;
                    }
                }
                if is_cancel {
                    if let Some(conn) = &conn {
                        checkpoint::update_job_status(conn, &job_id, "terminated")?;
                    }
                    return Err(e.context("Offload cancelled"));
                }
                failures.push(OffloadFailure {
                    file: file.rel.clone(),
                    destination: None,
                    error: e.to_string(),
                });
            }
        }

        bytes_done_before_file += file.size;
    }

    // ── ASC MHL на каждом получателе ───────────────────────────────────────
    let mut mhl_paths = Vec::new();
    if req.write_mhl {
        for (di, dest_root) in req.destinations.iter().enumerate() {
            // ArchiveMax records every replica independently verified in this
            // job, including files that were safely reused during resume.
            let mut file_hashes: HashMap<String, Vec<HashResult>> = HashMap::new();
            let mut file_metadata: HashMap<String, (u64, chrono::DateTime<chrono::Utc>)> =
                HashMap::new();
            for rel in &copied_per_dest[di] {
                let Some(hashes) = source_hashes.get(rel) else {
                    continue;
                };
                let interoperable: Vec<HashResult> = hashes
                    .iter()
                    .filter(|hash| hash.algorithm == hash_policy.mhl_algorithm)
                    .cloned()
                    .collect();
                if interoperable.is_empty() {
                    continue;
                }
                file_hashes.insert(rel.clone(), interoperable);
                let dest_file = dest_root.join(rel);
                let size = files
                    .iter()
                    .find(|f| &f.rel == rel)
                    .map(|f| f.size)
                    .unwrap_or(0);
                let modified = tokio::fs::metadata(&dest_file)
                    .await
                    .ok()
                    .and_then(|m| m.modified().ok())
                    .map(chrono::DateTime::<chrono::Utc>::from)
                    .unwrap_or_else(chrono::Utc::now);
                file_metadata.insert(rel.clone(), (size, modified));
            }
            if file_hashes.is_empty() {
                continue;
            }

            on_progress(OffloadProgress {
                phase: "mhl".into(),
                current_file: dest_root.display().to_string(),
                file_index: di + 1,
                total_files: req.destinations.len(),
                bytes_done: bytes_total,
                bytes_total,
            });

            let mhl_result = async {
                let mut history = mhl::load_or_create_history(dest_root).await?;
                let mhl_config = mhl::MhlConfig {
                    hash_format: hash_policy.mhl_algorithm,
                    ..Default::default()
                };
                mhl::create_generation(
                    &mut history,
                    &file_hashes,
                    &file_metadata,
                    mhl::MhlProcessType::Transfer,
                    &mhl_config,
                )
                .await
            }
            .await;
            match mhl_result {
                Ok(path) => mhl_paths.push(path.display().to_string()),
                Err(error) => failures.push(OffloadFailure {
                    file: "ascmhl".into(),
                    destination: Some(dest_root.display().to_string()),
                    error: format!("MHL generation failed: {error:#}"),
                }),
            }
        }
    }

    // ── Финал ──────────────────────────────────────────────────────────────
    let failed = failures.len();
    let all_archive_verified = req.profile == VerificationProfile::ArchiveMax
        && failed == 0
        && verified_replicas == files.len() * req.destinations.len();
    let safe_to_format = SafetyGate {
        archive_verified: all_archive_verified,
        destination_count: req.destinations.len(),
        distinct_destination_count: destination_volume_keys.len(),
        physical_destination_count: physical_destination_keys.len(),
        write_mhl: req.write_mhl,
        mhl_count: mhl_paths.len(),
        evidence_persisted: conn.is_some(),
        warning_count: warnings.len(),
    }
    .allows_format();
    if all_archive_verified && req.destinations.len() < 2 {
        warnings.push(
            "Only one independently verified destination exists; source media must not be formatted"
                .into(),
        );
    }
    let verdict = if failed > 0 {
        OffloadVerdict::Failed
    } else if safe_to_format {
        OffloadVerdict::SafeToFormat
    } else if all_archive_verified {
        OffloadVerdict::ArchiveVerified
    } else {
        OffloadVerdict::CopyComplete
    };
    on_progress(OffloadProgress {
        phase: "done".into(),
        current_file: String::new(),
        file_index: files.len(),
        total_files: files.len(),
        bytes_done: bytes_total,
        bytes_total,
    });

    // A resumed job exports the complete persisted evidence history, not just
    // the observations produced after this process started.
    if let Some(conn) = &conn {
        observations =
            deserialize_payloads(checkpoint::get_hash_observation_payloads(conn, &job_id)?)?;
        repairs = deserialize_payloads(checkpoint::get_repair_attempt_payloads(conn, &job_id)?)?;
        replicas = deserialize_payloads(checkpoint::get_replica_state_payloads(conn, &job_id)?)?;
    }

    let mut summary = OffloadSummary {
        evidence_schema_version: 2,
        app_name: "ProofCat".into(),
        app_version: env!("CARGO_PKG_VERSION").into(),
        commit: option_env!("GIT_COMMIT").unwrap_or("unknown").into(),
        started_at: started_at.to_rfc3339(),
        finished_at: chrono::Utc::now().to_rfc3339(),
        job_id,
        total_files: files.len(),
        copied,
        skipped,
        failed,
        bytes_copied,
        failures,
        mhl_paths,
        profile: req.profile,
        hash_policy,
        verdict,
        safe_to_format,
        verified_replicas,
        verification_failed,
        effective_small_file_workers: small_file_workers,
        warnings,
        observations,
        replicas,
        repairs,
        source_volume,
        destination_volumes,
        source_snapshot: files
            .iter()
            .map(|file| SourceFileEvidence {
                path: file.rel.clone(),
                size: file.size,
                modified_ns: file.modified_ns,
                file_identity: file.file_identity.clone(),
            })
            .collect(),
        destination_preflight,
        report_contacts,
        auto_eject_requested: req.auto_eject,
    };
    if req.auto_eject && summary.failed == 0 {
        // This runs only after all writes, readbacks and MHL generations have
        // produced an immutable local evidence snapshot in memory.
        summary
            .warnings
            .extend(auto_eject_destinations(&summary.destination_volumes));
    }
    if let Some(conn) = &conn {
        let status = if failed == 0 { "completed" } else { "failed" };
        checkpoint::update_job_status(conn, &summary.job_id, status)?;
        checkpoint::save_job_summary(conn, &summary.job_id, &serde_json::to_string(&summary)?)?;
        checkpoint::append_job_event(
            conn,
            &summary.job_id,
            "jobEvidence",
            &serde_json::to_string(&summary)?,
        )?;
    }
    Ok(summary)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;

    fn write_tree(root: &Path) {
        std::fs::create_dir_all(root.join("A001/CLIP")).unwrap();
        std::fs::write(root.join("A001/CLIP/clip1.mov"), vec![0xAAu8; 64 * 1024]).unwrap();
        std::fs::write(root.join("A001/CLIP/clip2.mov"), vec![0xBBu8; 32 * 1024]).unwrap();
        std::fs::write(root.join("notes.txt"), b"day 1").unwrap();
        std::fs::write(root.join(".DS_Store"), b"junk").unwrap(); // должен игнориться
    }

    #[test]
    fn rejects_cross_platform_unsafe_names_before_copy() {
        for path in ["CON.mov", "folder/clip?.mov", "folder/trailing. "] {
            assert!(validate_portable_relative_path(path).is_err(), "{path}");
        }
        assert!(validate_portable_relative_path("A001/Клип 🎬.mov").is_ok());
        assert_eq!(collision_key("É.mov"), collision_key("e\u{301}.MOV"));
        assert_eq!(readback_concurrency(2, 2), 2);
        assert_eq!(readback_concurrency(1, 2), 1);
        assert_eq!(readback_concurrency(12, 12), 4);

        let mut volumes = HashSet::new();
        assert!(register_destination_volume(&mut volumes, "physical-1", true).unwrap());
        let duplicate = register_destination_volume(&mut volumes, "physical-1", true)
            .unwrap_err()
            .to_string();
        assert!(duplicate.contains("same physical volume"));
    }

    #[test]
    fn preflight_records_available_space_before_copy() {
        let dir = tempfile::tempdir().unwrap();
        let result = preflight_destinations(&[dir.path().to_path_buf()], 1).unwrap();
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].required_bytes, 1);
        assert!(result[0].available_bytes >= 1);
    }

    #[test]
    fn resume_does_not_require_space_for_the_already_verified_source() {
        let dir = tempfile::tempdir().unwrap();
        let result =
            initial_destination_preflight(true, &[dir.path().to_path_buf()], u64::MAX).unwrap();
        assert!(result.is_empty());
    }

    #[test]
    fn mixed_root_files_and_folders_warn_as_possible_cherry_pick() {
        let dir = tempfile::tempdir().unwrap();
        std::fs::write(dir.path().join("loose.mov"), b"media").unwrap();
        std::fs::create_dir(dir.path().join("PRIVATE")).unwrap();
        assert!(source_looks_like_mixed_selection(dir.path()));
    }

    #[test]
    fn safe_to_format_requires_every_gate() {
        let gate = SafetyGate {
            archive_verified: true,
            destination_count: 2,
            distinct_destination_count: 2,
            physical_destination_count: 2,
            write_mhl: true,
            mhl_count: 2,
            evidence_persisted: true,
            warning_count: 0,
        };
        assert!(gate.allows_format());

        for unsafe_gate in [
            SafetyGate {
                archive_verified: false,
                ..gate
            },
            SafetyGate {
                destination_count: 1,
                distinct_destination_count: 1,
                physical_destination_count: 1,
                mhl_count: 1,
                ..gate
            },
            SafetyGate {
                distinct_destination_count: 1,
                ..gate
            },
            SafetyGate {
                physical_destination_count: 1,
                ..gate
            },
            SafetyGate {
                write_mhl: false,
                ..gate
            },
            SafetyGate {
                mhl_count: 1,
                ..gate
            },
            SafetyGate {
                evidence_persisted: false,
                ..gate
            },
            SafetyGate {
                warning_count: 1,
                ..gate
            },
        ] {
            assert!(!unsafe_gate.allows_format());
        }
    }

    #[test]
    fn disconnected_device_error_is_humanized() {
        assert_eq!(
            user_facing_storage_message("Device not configured (os error 6)"),
            "A storage device was disconnected. Reconnect the source or destination, then resume the same job."
        );
        assert_eq!(
            user_facing_storage_message("permission denied"),
            "permission denied"
        );
    }

    #[tokio::test]
    async fn test_offload_end_to_end_two_destinations() {
        let src = tempfile::tempdir().unwrap();
        let d1 = tempfile::tempdir().unwrap();
        let d2 = tempfile::tempdir().unwrap();
        let db = tempfile::tempdir().unwrap();
        write_tree(src.path());

        let req = OffloadRequest {
            source: src.path().to_path_buf(),
            destinations: vec![d1.path().to_path_buf(), d2.path().to_path_buf()],
            algorithms: vec![HashAlgorithm::XXH64],
            write_mhl: true,
            checkpoint_db: Some(db.path().join("offload.sqlite")),
            profile: VerificationProfile::Fast,
            job_id: None,
            small_file_concurrency: 1,
            report_contacts: Vec::new(),
            auto_eject: false,
        };

        let summary = run_offload(&req, None, None, &|_| {}).await.unwrap();

        assert_eq!(summary.total_files, 3, ".DS_Store must be ignored");
        assert_eq!(summary.copied, 3);
        assert_eq!(summary.failed, 0);

        // Байт-в-байт на обоих получателях
        for d in [d1.path(), d2.path()] {
            for rel in ["A001/CLIP/clip1.mov", "A001/CLIP/clip2.mov", "notes.txt"] {
                let src_bytes = std::fs::read(src.path().join(rel)).unwrap();
                let dst_bytes = std::fs::read(d.join(rel)).unwrap();
                assert_eq!(src_bytes, dst_bytes, "byte mismatch for {rel}");
            }
            // ascmhl generation написан
            assert!(d.join("ascmhl").is_dir(), "ascmhl dir missing");
        }
        assert_eq!(summary.mhl_paths.len(), 2);

        // Checkpoint: все таски completed
        let conn = checkpoint::open_db(&db.path().join("offload.sqlite")).unwrap();
        let progress = checkpoint::get_job_progress(&conn, &summary.job_id).unwrap();
        assert_eq!(progress.completed, 6); // 3 файла × 2 dest
        assert_eq!(progress.failed, 0);
    }

    #[tokio::test]
    async fn test_offload_rerun_skips_verified() {
        let src = tempfile::tempdir().unwrap();
        let d1 = tempfile::tempdir().unwrap();
        write_tree(src.path());

        let req = OffloadRequest {
            source: src.path().to_path_buf(),
            destinations: vec![d1.path().to_path_buf()],
            algorithms: vec![HashAlgorithm::XXH64],
            write_mhl: false,
            checkpoint_db: None,
            profile: VerificationProfile::Fast,
            job_id: None,
            small_file_concurrency: 1,
            report_contacts: Vec::new(),
            auto_eject: false,
        };

        let first = run_offload(&req, None, None, &|_| {}).await.unwrap();
        assert_eq!(first.copied, 3);

        // Повторный прогон: всё уже лежит и верифицировано — skip, не перекопирование.
        let second = run_offload(&req, None, None, &|_| {}).await.unwrap();
        assert_eq!(second.copied, 0);
        assert_eq!(second.skipped, 3);
    }

    #[tokio::test]
    async fn offload_creates_missing_destination_directory() {
        let src = tempfile::tempdir().unwrap();
        let destination_root = tempfile::tempdir().unwrap();
        write_tree(src.path());
        let destination = destination_root.path().join("new-project/day-01");
        assert!(!destination.exists());

        let summary = run_offload(
            &OffloadRequest {
                source: src.path().to_path_buf(),
                destinations: vec![destination.clone()],
                algorithms: vec![HashAlgorithm::XXH64],
                write_mhl: false,
                checkpoint_db: None,
                profile: VerificationProfile::Fast,
                job_id: None,
                small_file_concurrency: 1,
                report_contacts: Vec::new(),
                auto_eject: false,
            },
            None,
            None,
            &|_| {},
        )
        .await
        .unwrap();

        assert_eq!(summary.copied, 3);
        assert!(destination.join("A001/CLIP/clip1.mov").is_file());
    }

    #[tokio::test]
    async fn fast_parallel_small_file_copy_is_ssd_only_bounded_and_complete() {
        let src = tempfile::tempdir().unwrap();
        let dst = tempfile::tempdir().unwrap();
        for index in 0..32 {
            std::fs::write(
                src.path().join(format!("small-{index:02}.mov")),
                vec![index as u8; 64 * 1024],
            )
            .unwrap();
        }
        let request = OffloadRequest {
            source: src.path().to_path_buf(),
            destinations: vec![dst.path().to_path_buf()],
            algorithms: vec![HashAlgorithm::XXH64],
            write_mhl: true,
            checkpoint_db: None,
            profile: VerificationProfile::Fast,
            job_id: None,
            small_file_concurrency: 8,
            report_contacts: Vec::new(),
            auto_eject: false,
        };

        let summary = run_offload(&request, None, None, &|_| {}).await.unwrap();
        assert_eq!(summary.copied, 32);
        assert_eq!(summary.failed, 0);
        assert!((1..=4).contains(&summary.effective_small_file_workers));
        if summary.source_volume.device_type == crate::offload::volume::DeviceType::SSD {
            assert_eq!(summary.effective_small_file_workers, 4);
        } else {
            assert_eq!(summary.effective_small_file_workers, 1);
        }
        for index in 0..32 {
            assert_eq!(
                std::fs::read(src.path().join(format!("small-{index:02}.mov"))).unwrap(),
                std::fs::read(dst.path().join(format!("small-{index:02}.mov"))).unwrap()
            );
        }
        let report = crate::offload::mhl::verifier::verify_mhl_path(
            dst.path(),
            crate::offload::mhl::verifier::MhlVerifyOptions::default(),
        )
        .unwrap();
        assert!(report.summary.success, "{:?}", report.issues);
    }

    #[tokio::test]
    async fn test_offload_rejects_dest_inside_source() {
        let src = tempfile::tempdir().unwrap();
        write_tree(src.path());
        let inner = src.path().join("backup");
        std::fs::create_dir(&inner).unwrap();

        let req = OffloadRequest {
            source: src.path().to_path_buf(),
            destinations: vec![inner],
            algorithms: vec![],
            write_mhl: false,
            checkpoint_db: None,
            profile: VerificationProfile::Fast,
            job_id: None,
            small_file_concurrency: 1,
            report_contacts: Vec::new(),
            auto_eject: false,
        };
        let err = run_offload(&req, None, None, &|_| {}).await.unwrap_err();
        assert!(err.to_string().contains("overlaps"));
    }

    // Windows rejects creating a directory entry named CON.mov before the
    // offload scanner can inspect it. The portable-name validator itself is
    // covered by rejects_cross_platform_unsafe_names_before_copy on all
    // platforms; this end-to-end fixture only makes sense where the fixture
    // can actually exist.
    #[cfg(not(windows))]
    #[tokio::test]
    async fn unsafe_source_name_blocks_before_any_destination_write() {
        let src = tempfile::tempdir().unwrap();
        let dst = tempfile::tempdir().unwrap();
        std::fs::write(src.path().join("CON.mov"), b"unsafe").unwrap();
        let request = OffloadRequest {
            source: src.path().to_path_buf(),
            destinations: vec![dst.path().to_path_buf()],
            algorithms: vec![HashAlgorithm::XXH64],
            write_mhl: true,
            checkpoint_db: None,
            profile: VerificationProfile::ArchiveMax,
            job_id: None,
            small_file_concurrency: 1,
            report_contacts: Vec::new(),
            auto_eject: false,
        };

        let error = run_offload(&request, None, None, &|_| {})
            .await
            .unwrap_err()
            .to_string();
        assert!(error.contains("Windows-reserved"), "{error}");
        assert_eq!(std::fs::read_dir(dst.path()).unwrap().count(), 0);
    }

    #[tokio::test]
    async fn archive_max_zero_byte_media_requires_operator_review() {
        let src = tempfile::tempdir().unwrap();
        let dst = tempfile::tempdir().unwrap();
        std::fs::write(src.path().join("empty.mov"), b"").unwrap();
        let request = OffloadRequest {
            source: src.path().to_path_buf(),
            destinations: vec![dst.path().to_path_buf()],
            algorithms: vec![HashAlgorithm::XXH64],
            write_mhl: true,
            checkpoint_db: None,
            profile: VerificationProfile::ArchiveMax,
            job_id: None,
            small_file_concurrency: 1,
            report_contacts: Vec::new(),
            auto_eject: false,
        };

        let summary = run_offload(&request, None, None, &|_| {}).await.unwrap();
        assert_eq!(summary.verdict, OffloadVerdict::ArchiveVerified);
        assert!(!summary.safe_to_format);
        assert!(summary
            .warnings
            .iter()
            .any(|warning| warning.contains("Zero-byte")));
        assert_eq!(summary.replicas[0].status, ReplicaState::Verified);
    }

    #[tokio::test]
    async fn test_offload_cancel_terminates_job() {
        let src = tempfile::tempdir().unwrap();
        let d1 = tempfile::tempdir().unwrap();
        write_tree(src.path());

        let cancel = AtomicBool::new(true); // отменяем сразу
        let req = OffloadRequest {
            source: src.path().to_path_buf(),
            destinations: vec![d1.path().to_path_buf()],
            algorithms: vec![],
            write_mhl: false,
            checkpoint_db: None,
            profile: VerificationProfile::Fast,
            job_id: None,
            small_file_concurrency: 1,
            report_contacts: Vec::new(),
            auto_eject: false,
        };
        let err = run_offload(&req, Some(&cancel), None, &|_| {})
            .await
            .unwrap_err();
        assert!(err.to_string().to_lowercase().contains("cancel"));
        assert!(cancel.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_progress_phases_emitted() {
        use std::sync::Mutex;
        let src = tempfile::tempdir().unwrap();
        let d1 = tempfile::tempdir().unwrap();
        write_tree(src.path());

        let phases: Mutex<Vec<String>> = Mutex::new(Vec::new());
        let req = OffloadRequest {
            source: src.path().to_path_buf(),
            destinations: vec![d1.path().to_path_buf()],
            algorithms: vec![],
            write_mhl: true,
            checkpoint_db: None,
            profile: VerificationProfile::Fast,
            job_id: None,
            small_file_concurrency: 1,
            report_contacts: Vec::new(),
            auto_eject: false,
        };
        run_offload(&req, None, None, &|p| {
            phases.lock().unwrap().push(p.phase);
        })
        .await
        .unwrap();

        let seen = phases.lock().unwrap();
        assert!(seen.contains(&"scanning".to_string()));
        assert!(seen.contains(&"copying".to_string()));
        assert!(seen.contains(&"mhl".to_string()));
        assert_eq!(seen.last().map(|s| s.as_str()), Some("done"));
    }

    #[tokio::test]
    async fn archive_max_independently_verifies_and_keeps_blake3_out_of_mhl() {
        let src = tempfile::tempdir().unwrap();
        let dst = tempfile::tempdir().unwrap();
        std::fs::write(src.path().join("clip.mov"), vec![0x42; 128 * 1024]).unwrap();
        let request = OffloadRequest {
            source: src.path().to_path_buf(),
            destinations: vec![dst.path().to_path_buf()],
            algorithms: vec![HashAlgorithm::XXH64],
            write_mhl: true,
            checkpoint_db: None,
            profile: VerificationProfile::ArchiveMax,
            job_id: None,
            small_file_concurrency: 1,
            report_contacts: Vec::new(),
            auto_eject: false,
        };

        let summary = run_offload(&request, None, None, &|_| {}).await.unwrap();
        assert_eq!(summary.verdict, OffloadVerdict::ArchiveVerified);
        assert!(!summary.safe_to_format, "one destination is not enough");
        assert_eq!(summary.verified_replicas, 1);
        assert!(summary.replicas[0]
            .observed_hashes
            .iter()
            .any(|hash| hash.algorithm == HashAlgorithm::BLAKE3));

        let manifest = std::fs::read_to_string(&summary.mhl_paths[0]).unwrap();
        assert!(manifest.contains("<xxh64"));
        assert!(!manifest.contains("blake3"));
        assert!(!manifest.contains("sha256"));
    }

    #[tokio::test]
    async fn archive_max_rejects_source_changed_between_reads() {
        let src = tempfile::tempdir().unwrap();
        let dst = tempfile::tempdir().unwrap();
        let source_file = src.path().join("clip.mov");
        std::fs::write(&source_file, vec![0x11; 64 * 1024]).unwrap();
        let changed = AtomicBool::new(false);
        let request = OffloadRequest {
            source: src.path().to_path_buf(),
            destinations: vec![dst.path().to_path_buf()],
            algorithms: vec![HashAlgorithm::XXH64],
            write_mhl: false,
            checkpoint_db: None,
            profile: VerificationProfile::ArchiveMax,
            job_id: None,
            small_file_concurrency: 1,
            report_contacts: Vec::new(),
            auto_eject: false,
        };

        let summary = run_offload(&request, None, None, &|progress| {
            if progress.phase == "copying" && !changed.swap(true, Ordering::SeqCst) {
                std::fs::write(&source_file, vec![0x22; 64 * 1024]).unwrap();
            }
        })
        .await
        .unwrap();

        assert_eq!(summary.verdict, OffloadVerdict::Failed);
        assert!(summary
            .failures
            .iter()
            .any(|failure| failure.error.contains("Source changed")));
    }

    #[tokio::test]
    async fn archive_max_repairs_destination_corrupted_before_readback() {
        let src = tempfile::tempdir().unwrap();
        let dst = tempfile::tempdir().unwrap();
        let destination_file = dst.path().join("clip.mov");
        std::fs::write(src.path().join("clip.mov"), vec![0x33; 64 * 1024]).unwrap();
        let corrupted = AtomicBool::new(false);
        let request = OffloadRequest {
            source: src.path().to_path_buf(),
            destinations: vec![dst.path().to_path_buf()],
            algorithms: vec![HashAlgorithm::XXH64],
            write_mhl: false,
            checkpoint_db: None,
            profile: VerificationProfile::ArchiveMax,
            job_id: None,
            small_file_concurrency: 1,
            report_contacts: Vec::new(),
            auto_eject: false,
        };

        let summary = run_offload(&request, None, None, &|progress| {
            if progress.phase == "destinationVerify" && !corrupted.swap(true, Ordering::SeqCst) {
                let mut bytes = std::fs::read(&destination_file).unwrap();
                bytes[0] ^= 0xff;
                std::fs::write(&destination_file, bytes).unwrap();
            }
        })
        .await
        .unwrap();

        assert_eq!(summary.verdict, OffloadVerdict::ArchiveVerified);
        assert_eq!(summary.repairs.len(), 1);
        assert!(summary.repairs[0].success);
        assert_eq!(
            std::fs::read(src.path().join("clip.mov")).unwrap(),
            std::fs::read(destination_file).unwrap()
        );
    }

    #[tokio::test]
    async fn archive_max_repairs_from_verified_replica_when_source_disappears() {
        let src = tempfile::tempdir().unwrap();
        let d1 = tempfile::tempdir().unwrap();
        let d2 = tempfile::tempdir().unwrap();
        let source_file = src.path().join("clip.mov");
        let damaged_file = d1.path().join("clip.mov");
        std::fs::write(&source_file, vec![0x55; 64 * 1024]).unwrap();
        let injected = AtomicBool::new(false);
        let request = OffloadRequest {
            source: src.path().to_path_buf(),
            destinations: vec![d1.path().to_path_buf(), d2.path().to_path_buf()],
            algorithms: vec![HashAlgorithm::XXH64],
            write_mhl: false,
            checkpoint_db: None,
            profile: VerificationProfile::ArchiveMax,
            job_id: None,
            small_file_concurrency: 1,
            report_contacts: Vec::new(),
            auto_eject: false,
        };

        let summary = run_offload(&request, None, None, &|progress| {
            if progress.phase == "destinationVerify" && !injected.swap(true, Ordering::SeqCst) {
                let mut bytes = std::fs::read(&damaged_file).unwrap();
                bytes[0] ^= 0xff;
                std::fs::write(&damaged_file, bytes).unwrap();
                std::fs::remove_file(&source_file).unwrap();
            }
        })
        .await
        .unwrap();

        assert_eq!(summary.verdict, OffloadVerdict::ArchiveVerified);
        assert_eq!(summary.repairs.len(), 1);
        assert_eq!(
            summary.repairs[0].destination,
            damaged_file.display().to_string()
        );
        assert!(summary
            .repairs
            .iter()
            .all(|repair| repair.destination != d2.path().join("clip.mov").display().to_string()));
        assert_eq!(
            summary
                .replicas
                .iter()
                .find(|replica| replica.destination
                    == d2.path().join("clip.mov").display().to_string())
                .unwrap()
                .repair_attempts,
            0
        );
        assert_eq!(
            summary.repairs[0].source,
            d2.path().join("clip.mov").display().to_string()
        );
        assert_eq!(
            std::fs::read(d1.path().join("clip.mov")).unwrap(),
            std::fs::read(d2.path().join("clip.mov")).unwrap()
        );
    }

    #[tokio::test]
    async fn archive_max_resume_continues_same_checkpoint_job() {
        let src = tempfile::tempdir().unwrap();
        let dst = tempfile::tempdir().unwrap();
        let db_dir = tempfile::tempdir().unwrap();
        let db_path = db_dir.path().join("offload.sqlite");
        for index in 0..3 {
            std::fs::write(
                src.path().join(format!("0{index}.mov")),
                vec![index as u8; 32 * 1024],
            )
            .unwrap();
        }
        let mut request = OffloadRequest {
            source: src.path().to_path_buf(),
            destinations: vec![dst.path().to_path_buf()],
            algorithms: vec![HashAlgorithm::XXH64],
            write_mhl: true,
            checkpoint_db: Some(db_path.clone()),
            profile: VerificationProfile::ArchiveMax,
            job_id: Some("job-resume-test".into()),
            small_file_concurrency: 1,
            report_contacts: Vec::new(),
            auto_eject: false,
        };
        let cancel = AtomicBool::new(false);
        let first = run_offload(&request, Some(&cancel), None, &|progress| {
            if progress.phase == "copying" && progress.file_index >= 2 {
                cancel.store(true, Ordering::SeqCst);
            }
        })
        .await;
        assert!(first.is_err());

        cancel.store(false, Ordering::SeqCst);
        request.job_id = Some("job-resume-test".into());
        let resumed = run_offload(&request, Some(&cancel), None, &|_| {})
            .await
            .unwrap();
        assert_eq!(resumed.job_id, "job-resume-test");
        assert_eq!(resumed.verdict, OffloadVerdict::ArchiveVerified);
        assert_eq!(resumed.verified_replicas, 3);
        let conn = checkpoint::open_db(&db_path).unwrap();
        let tasks = checkpoint::get_all_tasks(&conn, "job-resume-test").unwrap();
        assert_eq!(tasks.len(), 3, "resume must not duplicate checkpoint rows");
        let manifest = std::fs::read_to_string(&resumed.mhl_paths[0]).unwrap();
        for index in 0..3 {
            assert!(manifest.contains(&format!("0{index}.mov")));
        }
    }

    #[tokio::test]
    async fn archive_max_source_loss_during_copy_never_verifies_replica() {
        let src = tempfile::tempdir().unwrap();
        let dst = tempfile::tempdir().unwrap();
        let source_file = src.path().join("clip.mov");
        std::fs::write(&source_file, vec![0x61; 64 * 1024]).unwrap();
        let removed = AtomicBool::new(false);
        let request = OffloadRequest {
            source: src.path().to_path_buf(),
            destinations: vec![dst.path().to_path_buf()],
            algorithms: vec![HashAlgorithm::XXH64],
            write_mhl: false,
            checkpoint_db: None,
            profile: VerificationProfile::ArchiveMax,
            job_id: None,
            small_file_concurrency: 1,
            report_contacts: Vec::new(),
            auto_eject: false,
        };

        let summary = run_offload(&request, None, None, &|progress| {
            if progress.phase == "copying" && !removed.swap(true, Ordering::SeqCst) {
                std::fs::remove_file(&source_file).unwrap();
            }
        })
        .await
        .unwrap();

        assert_eq!(summary.verdict, OffloadVerdict::Failed);
        assert_eq!(summary.verified_replicas, 0);
        assert!(summary
            .replicas
            .iter()
            .all(|replica| replica.status != ReplicaState::Verified));
    }

    #[tokio::test]
    async fn archive_max_destination_loss_exhausts_repairs_without_false_verified_state() {
        let src = tempfile::tempdir().unwrap();
        let dst = tempfile::tempdir().unwrap();
        std::fs::write(src.path().join("clip.mov"), vec![0x62; 64 * 1024]).unwrap();
        let disconnected = AtomicBool::new(false);
        let request = OffloadRequest {
            source: src.path().to_path_buf(),
            destinations: vec![dst.path().to_path_buf()],
            algorithms: vec![HashAlgorithm::XXH64],
            write_mhl: false,
            checkpoint_db: None,
            profile: VerificationProfile::ArchiveMax,
            job_id: None,
            small_file_concurrency: 1,
            report_contacts: Vec::new(),
            auto_eject: false,
        };

        let summary = run_offload(&request, None, None, &|progress| {
            if progress.phase == "destinationVerify" && !disconnected.swap(true, Ordering::SeqCst) {
                std::fs::remove_dir_all(dst.path()).unwrap();
                std::fs::write(dst.path(), b"disconnected destination placeholder").unwrap();
            }
        })
        .await
        .unwrap();

        std::fs::remove_file(dst.path()).unwrap();
        std::fs::create_dir(dst.path()).unwrap();
        assert_eq!(summary.verdict, OffloadVerdict::Failed);
        assert_eq!(summary.verified_replicas, 0);
        assert_eq!(summary.repairs.len(), 2);
        assert!(summary.repairs.iter().all(|attempt| !attempt.success));
        assert_eq!(summary.replicas[0].status, ReplicaState::VerifyFailed);
    }

    #[tokio::test]
    async fn archive_max_resume_rejects_mutated_source_snapshot() {
        let src = tempfile::tempdir().unwrap();
        let dst = tempfile::tempdir().unwrap();
        let db_dir = tempfile::tempdir().unwrap();
        let db_path = db_dir.path().join("offload.sqlite");
        let source_file = src.path().join("clip.mov");
        std::fs::write(&source_file, vec![0x71; 64 * 1024]).unwrap();
        let mut request = OffloadRequest {
            source: src.path().to_path_buf(),
            destinations: vec![dst.path().to_path_buf()],
            algorithms: vec![HashAlgorithm::XXH64],
            write_mhl: true,
            checkpoint_db: Some(db_path),
            profile: VerificationProfile::ArchiveMax,
            job_id: Some("job-replaced-source".into()),
            small_file_concurrency: 1,
            report_contacts: Vec::new(),
            auto_eject: false,
        };
        let first = run_offload(&request, None, None, &|_| {}).await.unwrap();
        assert_eq!(first.verdict, OffloadVerdict::ArchiveVerified);

        std::thread::sleep(std::time::Duration::from_millis(2));
        std::fs::write(&source_file, vec![0x72; 64 * 1024]).unwrap();
        request.job_id = Some("job-replaced-source".into());
        let error = run_offload(&request, None, None, &|_| {})
            .await
            .unwrap_err()
            .to_string();
        assert!(error.contains("source file snapshot changed"), "{error}");
    }

    #[tokio::test]
    async fn resume_converges_to_same_final_replica_evidence_from_each_interruptible_phase() {
        fn projection(
            summary: &OffloadSummary,
        ) -> Vec<(String, String, Vec<HashResult>, Vec<HashResult>)> {
            summary
                .replicas
                .iter()
                .map(|replica| {
                    (
                        replica.file.clone(),
                        replica.status.as_str().to_string(),
                        replica.expected_hashes.clone(),
                        replica.observed_hashes.clone(),
                    )
                })
                .collect()
        }

        let src = tempfile::tempdir().unwrap();
        for index in 0..2 {
            std::fs::write(
                src.path().join(format!("clip-{index}.mov")),
                vec![0x80 + index as u8; 128 * 1024],
            )
            .unwrap();
        }

        let baseline_dest = tempfile::tempdir().unwrap();
        let baseline_db = tempfile::tempdir().unwrap();
        let baseline = run_offload(
            &OffloadRequest {
                source: src.path().to_path_buf(),
                destinations: vec![baseline_dest.path().to_path_buf()],
                algorithms: vec![HashAlgorithm::XXH64],
                write_mhl: true,
                checkpoint_db: Some(baseline_db.path().join("baseline.sqlite")),
                profile: VerificationProfile::ArchiveMax,
                job_id: Some("job-baseline".into()),
                small_file_concurrency: 1,
                report_contacts: Vec::new(),
                auto_eject: false,
            },
            None,
            None,
            &|_| {},
        )
        .await
        .unwrap();
        let expected = projection(&baseline);

        for phase in ["sourcePreRead", "copying", "destinationVerify", "repairing"] {
            let dest = tempfile::tempdir().unwrap();
            let db_dir = tempfile::tempdir().unwrap();
            let job_id = format!("job-interrupt-{phase}");
            let request = OffloadRequest {
                source: src.path().to_path_buf(),
                destinations: vec![dest.path().to_path_buf()],
                algorithms: vec![HashAlgorithm::XXH64],
                write_mhl: true,
                checkpoint_db: Some(db_dir.path().join("offload.sqlite")),
                profile: VerificationProfile::ArchiveMax,
                job_id: Some(job_id.clone()),
                small_file_concurrency: 1,
                report_contacts: Vec::new(),
                auto_eject: false,
            };
            let cancel = AtomicBool::new(false);
            let injected = AtomicBool::new(false);
            let corrupted = AtomicBool::new(false);
            let interrupted = run_offload(&request, Some(&cancel), None, &|progress| {
                if phase == "repairing"
                    && progress.phase == "destinationVerify"
                    && !corrupted.swap(true, Ordering::SeqCst)
                {
                    let damaged = dest.path().join("clip-0.mov");
                    let mut bytes = std::fs::read(&damaged).unwrap();
                    bytes[0] ^= 0xff;
                    std::fs::write(damaged, bytes).unwrap();
                }
                if progress.phase == phase && !injected.swap(true, Ordering::SeqCst) {
                    cancel.store(true, Ordering::SeqCst);
                }
            })
            .await;
            assert!(interrupted.is_err(), "phase {phase} did not interrupt");
            let interrupted_conn =
                checkpoint::open_db(&db_dir.path().join("offload.sqlite")).unwrap();
            assert_eq!(
                checkpoint::get_job(&interrupted_conn, &job_id)
                    .unwrap()
                    .unwrap()
                    .status,
                "terminated",
                "phase {phase}"
            );
            drop(interrupted_conn);

            cancel.store(false, Ordering::SeqCst);
            let resumed = run_offload(&request, Some(&cancel), None, &|_| {})
                .await
                .unwrap();
            assert_eq!(resumed.job_id, job_id);
            assert_eq!(resumed.verdict, OffloadVerdict::ArchiveVerified);
            assert_eq!(projection(&resumed), expected, "phase {phase}");
            let event_conn = checkpoint::open_db(&db_dir.path().join("offload.sqlite")).unwrap();
            for event_type in ["jobStarted", "jobTerminated", "jobResumed", "jobEvidence"] {
                let count: i64 = event_conn
                    .query_row(
                        "SELECT COUNT(*) FROM job_events WHERE job_id = ?1 AND event_type = ?2",
                        rusqlite::params![job_id, event_type],
                        |row| row.get(0),
                    )
                    .unwrap();
                assert!(count >= 1, "missing {event_type} event for phase {phase}");
            }
            let report = crate::offload::mhl::verifier::verify_mhl_path(
                dest.path(),
                crate::offload::mhl::verifier::MhlVerifyOptions::default(),
            )
            .unwrap();
            assert!(report.summary.success, "phase {phase}: {:?}", report.issues);
        }
    }
}
