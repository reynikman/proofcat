// Ported from DIT-Pro (https://github.com/WillZ5/DIT-Pro), MIT License,
// Copyright (c) 2026 WillZ. See repository NOTICE. Adapted for Meta Report offload mode.

//! Checkpoint / Recovery System — Crash-safe task state persistence.
//!
//! Uses SQLite WAL mode to maintain task state across crashes.
//! State machine: pending → copying → verifying → completed | failed
//!
//! Recovery flow:
//! 1. Scan copy_tasks where status != 'completed'
//! 2. Clean up orphaned .tmp files for interrupted tasks
//! 3. Reset 'copying'/'verifying' tasks back to 'pending'
//! 4. Resume from the last completed file

use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::path::Path;

use crate::offload::copy_engine::atomic_writer;

/// Schema for the checkpoint database (в DIT-Pro жила в db-модуле, который не портируем —
/// оффлоаду нужны только jobs + copy_tasks).
const SCHEMA: &str = "CREATE TABLE IF NOT EXISTS jobs (
        id TEXT PRIMARY KEY, name TEXT NOT NULL,
        status TEXT NOT NULL DEFAULT 'pending',
        source_path TEXT NOT NULL,
        config_json TEXT,
        summary_json TEXT,
        source_volume_key TEXT,
        destination_volume_keys_json TEXT,
        source_snapshot_json TEXT,
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        updated_at TEXT NOT NULL DEFAULT (datetime('now'))
    );
    CREATE TABLE IF NOT EXISTS copy_tasks (
        id TEXT PRIMARY KEY,
        job_id TEXT NOT NULL REFERENCES jobs(id),
        source_path TEXT NOT NULL, dest_path TEXT NOT NULL,
        file_size INTEGER NOT NULL DEFAULT 0,
        status TEXT NOT NULL DEFAULT 'pending',
        hash_xxh64 TEXT, hash_sha256 TEXT,
        hash_md5 TEXT, hash_xxh128 TEXT, hash_xxh3 TEXT, hash_blake3 TEXT,
        error_msg TEXT, retry_count INTEGER NOT NULL DEFAULT 0,
        retry_note TEXT,
        created_at TEXT NOT NULL DEFAULT (datetime('now')),
        updated_at TEXT NOT NULL DEFAULT (datetime('now'))
    );
    CREATE TABLE IF NOT EXISTS job_events (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        job_id TEXT NOT NULL REFERENCES jobs(id),
        event_type TEXT NOT NULL,
        payload_json TEXT NOT NULL,
        created_at TEXT NOT NULL DEFAULT (datetime('now'))
    );
    CREATE TABLE IF NOT EXISTS hash_observations (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        job_id TEXT NOT NULL REFERENCES jobs(id),
        pass_type TEXT NOT NULL,
        file_path TEXT NOT NULL,
        destination TEXT,
        payload_json TEXT NOT NULL,
        created_at TEXT NOT NULL DEFAULT (datetime('now'))
    );
    CREATE TABLE IF NOT EXISTS repair_attempts (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        job_id TEXT NOT NULL REFERENCES jobs(id),
        file_path TEXT NOT NULL,
        destination TEXT NOT NULL,
        attempt INTEGER NOT NULL,
        payload_json TEXT NOT NULL,
        created_at TEXT NOT NULL DEFAULT (datetime('now'))
    );
    CREATE TABLE IF NOT EXISTS replica_states (
        job_id TEXT NOT NULL REFERENCES jobs(id),
        file_path TEXT NOT NULL,
        destination TEXT NOT NULL,
        payload_json TEXT NOT NULL,
        updated_at TEXT NOT NULL DEFAULT (datetime('now')),
        PRIMARY KEY (job_id, file_path, destination)
    );";

/// Ensure the checkpoint schema exists on a connection.
pub fn init_schema(conn: &Connection) -> Result<()> {
    conn.execute_batch("BEGIN IMMEDIATE")?;
    let migration = (|| -> Result<()> {
        conn.execute_batch(SCHEMA)
            .context("Failed to initialize checkpoint schema")?;
        // CREATE TABLE IF NOT EXISTS does not update databases created by older
        // releases. Every migration stays idempotent and runs in one transaction.
        ensure_column(conn, "jobs", "summary_json", "TEXT")?;
        ensure_column(conn, "jobs", "source_volume_key", "TEXT")?;
        ensure_column(conn, "jobs", "destination_volume_keys_json", "TEXT")?;
        ensure_column(conn, "jobs", "source_snapshot_json", "TEXT")?;
        ensure_column(conn, "copy_tasks", "hash_blake3", "TEXT")?;
        conn.pragma_update(None, "user_version", 4)?;
        Ok(())
    })();
    match migration {
        Ok(()) => conn
            .execute_batch("COMMIT")
            .context("Failed to commit checkpoint migration"),
        Err(error) => {
            let _ = conn.execute_batch("ROLLBACK");
            Err(error)
        }
    }
}

fn ensure_column(conn: &Connection, table: &str, column: &str, sql_type: &str) -> Result<()> {
    let mut stmt = conn.prepare(&format!("PRAGMA table_info({table})"))?;
    let names = stmt
        .query_map([], |row| row.get::<_, String>(1))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    if !names.iter().any(|name| name == column) {
        conn.execute_batch(&format!(
            "ALTER TABLE {table} ADD COLUMN {column} {sql_type}"
        ))?;
    }
    Ok(())
}

/// Open (or create) the checkpoint database with WAL mode and schema.
pub fn open_db(path: &Path) -> Result<Connection> {
    let conn = Connection::open(path)
        .with_context(|| format!("Failed to open checkpoint DB at {:?}", path))?;
    conn.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA synchronous=FULL;
         PRAGMA foreign_keys=ON;
         PRAGMA busy_timeout=5000;",
    )?;
    init_schema(&conn)?;
    Ok(conn)
}

/// Task status values stored in the database
pub const STATUS_PENDING: &str = "pending";
pub const STATUS_COPYING: &str = "copying";
pub const STATUS_VERIFYING: &str = "verifying";
pub const STATUS_COMPLETED: &str = "completed";
pub const STATUS_FAILED: &str = "failed";
pub const STATUS_SKIPPED: &str = "skipped";
pub const STATUS_PAUSED: &str = "paused";
pub const STATUS_TERMINATED: &str = "terminated";
pub const STATUS_CONFLICT: &str = "conflict";

/// All hash values for a completed copy task
#[derive(Debug, Clone, Default)]
pub struct TaskHashes {
    pub xxh64: Option<String>,
    pub sha256: Option<String>,
    pub md5: Option<String>,
    pub xxh128: Option<String>,
    pub xxh3: Option<String>,
    pub blake3: Option<String>,
}

/// A checkpoint-managed copy task record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckpointRecord {
    pub task_id: String,
    pub job_id: String,
    pub source_path: String,
    pub dest_path: String,
    pub file_size: i64,
    pub status: String,
    pub hash_xxh64: Option<String>,
    pub hash_sha256: Option<String>,
    pub hash_blake3: Option<String>,
    pub error_msg: Option<String>,
    pub retry_count: i32,
}

fn has_persisted_hash(record: &CheckpointRecord) -> bool {
    record.hash_xxh64.is_some() || record.hash_sha256.is_some() || record.hash_blake3.is_some()
}

/// Create a new job in the database
pub fn create_job(
    conn: &Connection,
    job_id: &str,
    name: &str,
    source_path: &str,
    config_json: Option<&str>,
) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO jobs (id, name, status, source_path, config_json) VALUES (?1, ?2, 'pending', ?3, ?4)",
        params![job_id, name, source_path, config_json],
    )?;
    Ok(())
}

/// Get stored config JSON for a job (used for re-run)
pub fn get_job_config(conn: &Connection, job_id: &str) -> Result<Option<String>> {
    let config: Option<String> = conn.query_row(
        "SELECT config_json FROM jobs WHERE id = ?1",
        params![job_id],
        |row| row.get(0),
    )?;
    Ok(config)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobRecord {
    pub id: String,
    pub status: String,
    pub config_json: Option<String>,
    pub summary_json: Option<String>,
}

pub fn get_job(conn: &Connection, job_id: &str) -> Result<Option<JobRecord>> {
    let mut statement =
        conn.prepare("SELECT id, status, config_json, summary_json FROM jobs WHERE id = ?1")?;
    let mut rows = statement.query(params![job_id])?;
    let Some(row) = rows.next()? else {
        return Ok(None);
    };
    Ok(Some(JobRecord {
        id: row.get(0)?,
        status: row.get(1)?,
        config_json: row.get(2)?,
        summary_json: row.get(3)?,
    }))
}

/// Persist the media identity used by a job, or validate it on resume.
/// This prevents a path with a different card/drive from silently inheriting
/// an earlier job's evidence.
pub fn bind_or_validate_job_context(
    conn: &Connection,
    job_id: &str,
    source_volume_key: &str,
    destination_volume_keys_json: &str,
    source_snapshot_json: &str,
) -> Result<()> {
    let existing = conn.query_row(
        "SELECT source_volume_key, destination_volume_keys_json, source_snapshot_json
         FROM jobs WHERE id = ?1",
        params![job_id],
        |row| {
            Ok((
                row.get::<_, Option<String>>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, Option<String>>(2)?,
            ))
        },
    )?;

    if existing.0.is_none() && existing.1.is_none() && existing.2.is_none() {
        conn.execute(
            "UPDATE jobs SET source_volume_key = ?1, destination_volume_keys_json = ?2,
             source_snapshot_json = ?3, updated_at = datetime('now') WHERE id = ?4",
            params![
                source_volume_key,
                destination_volume_keys_json,
                source_snapshot_json,
                job_id
            ],
        )?;
        return Ok(());
    }

    if existing.0.as_deref() != Some(source_volume_key) {
        anyhow::bail!("Resume blocked: source volume identity does not match job {job_id}");
    }
    if existing.1.as_deref() != Some(destination_volume_keys_json) {
        anyhow::bail!("Resume blocked: destination volume identities do not match job {job_id}");
    }
    if existing.2.as_deref() != Some(source_snapshot_json) {
        anyhow::bail!("Resume blocked: source file snapshot changed for job {job_id}");
    }
    Ok(())
}

pub fn append_job_event(
    conn: &Connection,
    job_id: &str,
    event_type: &str,
    payload_json: &str,
) -> Result<()> {
    conn.execute(
        "INSERT INTO job_events (job_id, event_type, payload_json) VALUES (?1, ?2, ?3)",
        params![job_id, event_type, payload_json],
    )?;
    Ok(())
}

pub fn append_hash_observation(
    conn: &Connection,
    job_id: &str,
    pass_type: &str,
    file_path: &str,
    destination: Option<&str>,
    payload_json: &str,
) -> Result<()> {
    conn.execute(
        "INSERT INTO hash_observations
         (job_id, pass_type, file_path, destination, payload_json)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![job_id, pass_type, file_path, destination, payload_json],
    )?;
    Ok(())
}

pub fn append_repair_attempt(
    conn: &Connection,
    job_id: &str,
    file_path: &str,
    destination: &str,
    attempt: u32,
    payload_json: &str,
) -> Result<()> {
    conn.execute(
        "INSERT INTO repair_attempts
         (job_id, file_path, destination, attempt, payload_json)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![job_id, file_path, destination, attempt, payload_json],
    )?;
    Ok(())
}

pub fn repair_attempt_count(
    conn: &Connection,
    job_id: &str,
    file_path: &str,
    destination: &str,
) -> Result<u32> {
    let count: i64 = conn.query_row(
        "SELECT COUNT(*) FROM repair_attempts
         WHERE job_id = ?1 AND file_path = ?2 AND destination = ?3",
        params![job_id, file_path, destination],
        |row| row.get(0),
    )?;
    Ok(count.try_into().unwrap_or(u32::MAX))
}

pub fn upsert_replica_state(
    conn: &Connection,
    job_id: &str,
    file_path: &str,
    destination: &str,
    payload_json: &str,
) -> Result<()> {
    conn.execute(
        "INSERT INTO replica_states (job_id, file_path, destination, payload_json)
         VALUES (?1, ?2, ?3, ?4)
         ON CONFLICT(job_id, file_path, destination) DO UPDATE SET
           payload_json = excluded.payload_json,
           updated_at = datetime('now')",
        params![job_id, file_path, destination, payload_json],
    )?;
    Ok(())
}

fn load_payloads(conn: &Connection, table: &str, job_id: &str) -> Result<Vec<String>> {
    let sql = format!("SELECT payload_json FROM {table} WHERE job_id = ?1 ORDER BY rowid ASC");
    let mut statement = conn.prepare(&sql)?;
    let payloads = statement
        .query_map(params![job_id], |row| row.get(0))?
        .collect::<std::result::Result<Vec<_>, _>>()?;
    Ok(payloads)
}

pub fn get_hash_observation_payloads(conn: &Connection, job_id: &str) -> Result<Vec<String>> {
    load_payloads(conn, "hash_observations", job_id)
}

pub fn get_repair_attempt_payloads(conn: &Connection, job_id: &str) -> Result<Vec<String>> {
    load_payloads(conn, "repair_attempts", job_id)
}

pub fn get_replica_state_payloads(conn: &Connection, job_id: &str) -> Result<Vec<String>> {
    load_payloads(conn, "replica_states", job_id)
}

/// Insert a new copy task
pub fn insert_task(
    conn: &Connection,
    task_id: &str,
    job_id: &str,
    source_path: &str,
    dest_path: &str,
    file_size: u64,
) -> Result<()> {
    conn.execute(
        "INSERT OR IGNORE INTO copy_tasks (id, job_id, source_path, dest_path, file_size, status)
         VALUES (?1, ?2, ?3, ?4, ?5, 'pending')",
        params![task_id, job_id, source_path, dest_path, file_size as i64],
    )?;
    Ok(())
}

/// Update task status
pub fn update_task_status(conn: &Connection, task_id: &str, status: &str) -> Result<()> {
    if status == STATUS_FAILED {
        conn.execute(
            "UPDATE copy_tasks SET status = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![status, task_id],
        )?;
    } else {
        conn.execute(
            "UPDATE copy_tasks SET status = ?1, error_msg = NULL, updated_at = datetime('now') WHERE id = ?2",
            params![status, task_id],
        )?;
    }
    Ok(())
}

/// Update task status with hash results (all algorithms)
pub fn update_task_completed(conn: &Connection, task_id: &str, hashes: &TaskHashes) -> Result<()> {
    conn.execute(
        "UPDATE copy_tasks SET status = 'completed',
         hash_xxh64 = ?1, hash_sha256 = ?2, hash_md5 = ?3, hash_xxh128 = ?4, hash_xxh3 = ?5, hash_blake3 = ?6,
         error_msg = NULL,
         updated_at = datetime('now') WHERE id = ?7",
        params![
            hashes.xxh64,
            hashes.sha256,
            hashes.md5,
            hashes.xxh128,
            hashes.xxh3,
            hashes.blake3,
            task_id
        ],
    )?;
    Ok(())
}

/// Update task as failed with error message.
/// Also appends the error to `retry_note` so the report can show the failure history
/// even if the file is later retried and succeeds.
pub fn update_task_failed(conn: &Connection, task_id: &str, error_msg: &str) -> Result<()> {
    // Append to retry_note: "Round 1 verify failed: <reason>"
    let retry_count: i32 = conn
        .query_row(
            "SELECT retry_count FROM copy_tasks WHERE id = ?1",
            params![task_id],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let round = retry_count + 1;
    let note_entry = format!("Round {} failed: {}", round, error_msg);

    conn.execute(
        "UPDATE copy_tasks SET status = 'failed', error_msg = ?1,
         retry_count = retry_count + 1,
         retry_note = CASE
             WHEN retry_note IS NULL THEN ?3
             ELSE retry_note || '; ' || ?3
         END,
         updated_at = datetime('now') WHERE id = ?2",
        params![error_msg, task_id, note_entry],
    )?;
    Ok(())
}

/// Append a success note to retry_note after a successful retry.
/// Called when a previously-failed task is re-copied and re-verified successfully.
pub fn append_retry_success(conn: &Connection, task_id: &str) -> Result<()> {
    let retry_count: i32 = conn
        .query_row(
            "SELECT retry_count FROM copy_tasks WHERE id = ?1",
            params![task_id],
            |row| row.get(0),
        )
        .unwrap_or(1);
    let note_entry = format!("Round {} retry succeeded", retry_count + 1);

    conn.execute(
        "UPDATE copy_tasks SET
         retry_note = CASE
             WHEN retry_note IS NULL THEN ?2
             ELSE retry_note || '; ' || ?2
         END,
         updated_at = datetime('now') WHERE id = ?1",
        params![task_id, note_entry],
    )?;
    Ok(())
}

/// Update job-level status directly
pub fn update_job_status(conn: &Connection, job_id: &str, status: &str) -> Result<()> {
    conn.execute(
        "UPDATE jobs SET status = ?1, updated_at = datetime('now') WHERE id = ?2",
        params![status, job_id],
    )?;
    Ok(())
}

pub fn save_job_summary(conn: &Connection, job_id: &str, summary_json: &str) -> Result<()> {
    conn.execute(
        "UPDATE jobs SET summary_json = ?1, updated_at = datetime('now') WHERE id = ?2",
        params![summary_json, job_id],
    )?;
    Ok(())
}

pub fn get_job_summary(conn: &Connection, job_id: &str) -> Result<Option<String>> {
    let summary = conn.query_row(
        "SELECT summary_json FROM jobs WHERE id = ?1",
        params![job_id],
        |row| row.get(0),
    )?;
    Ok(summary)
}

/// Mark task as skipped (existing file verified identical)
pub fn update_task_skipped(conn: &Connection, task_id: &str, hashes: &TaskHashes) -> Result<()> {
    conn.execute(
        "UPDATE copy_tasks SET status = 'skipped',
         hash_xxh64 = ?1, hash_sha256 = ?2, hash_md5 = ?3, hash_xxh128 = ?4, hash_xxh3 = ?5, hash_blake3 = ?6,
         error_msg = NULL,
         updated_at = datetime('now') WHERE id = ?7",
        params![
            hashes.xxh64,
            hashes.sha256,
            hashes.md5,
            hashes.xxh128,
            hashes.xxh3,
            hashes.blake3,
            task_id
        ],
    )?;
    Ok(())
}

// NB: update_task_media_metadata из DIT-Pro не портирована — метадату медиа
// Meta Report снимает своим analyze_file (mediainfo/ffprobe), camera-модуль не нужен.

/// Get all pending tasks for a job
pub fn get_pending_tasks(conn: &Connection, job_id: &str) -> Result<Vec<CheckpointRecord>> {
    let mut stmt = conn.prepare(
        "SELECT id, job_id, source_path, dest_path, file_size, status,
                hash_xxh64, hash_sha256, hash_blake3, error_msg, retry_count
         FROM copy_tasks WHERE job_id = ?1 AND status = 'pending'
         ORDER BY rowid ASC",
    )?;

    let records = stmt
        .query_map(params![job_id], |row| {
            Ok(CheckpointRecord {
                task_id: row.get(0)?,
                job_id: row.get(1)?,
                source_path: row.get(2)?,
                dest_path: row.get(3)?,
                file_size: row.get(4)?,
                status: row.get(5)?,
                hash_xxh64: row.get(6)?,
                hash_sha256: row.get(7)?,
                hash_blake3: row.get(8)?,
                error_msg: row.get(9)?,
                retry_count: row.get(10)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("Failed to read checkpoint records")?;

    Ok(records)
}

/// Get all copy tasks for a job.
pub fn get_all_tasks(conn: &Connection, job_id: &str) -> Result<Vec<CheckpointRecord>> {
    let mut stmt = conn.prepare(
        "SELECT id, job_id, source_path, dest_path, file_size, status,
                hash_xxh64, hash_sha256, hash_blake3, error_msg, retry_count
         FROM copy_tasks WHERE job_id = ?1
         ORDER BY rowid ASC",
    )?;

    let records = stmt
        .query_map(params![job_id], |row| {
            Ok(CheckpointRecord {
                task_id: row.get(0)?,
                job_id: row.get(1)?,
                source_path: row.get(2)?,
                dest_path: row.get(3)?,
                file_size: row.get(4)?,
                status: row.get(5)?,
                hash_xxh64: row.get(6)?,
                hash_sha256: row.get(7)?,
                hash_blake3: row.get(8)?,
                error_msg: row.get(9)?,
                retry_count: row.get(10)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("Failed to read job task records")?;

    Ok(records)
}

/// Get tasks that were interrupted or failed (recoverable tasks)
pub fn get_interrupted_tasks(conn: &Connection, job_id: &str) -> Result<Vec<CheckpointRecord>> {
    let mut stmt = conn.prepare(
        "SELECT id, job_id, source_path, dest_path, file_size, status,
                hash_xxh64, hash_sha256, hash_blake3, error_msg, retry_count
         FROM copy_tasks WHERE job_id = ?1 AND status IN ('copying', 'verifying', 'failed')
         ORDER BY rowid ASC",
    )?;

    let records = stmt
        .query_map(params![job_id], |row| {
            Ok(CheckpointRecord {
                task_id: row.get(0)?,
                job_id: row.get(1)?,
                source_path: row.get(2)?,
                dest_path: row.get(3)?,
                file_size: row.get(4)?,
                status: row.get(5)?,
                hash_xxh64: row.get(6)?,
                hash_sha256: row.get(7)?,
                hash_blake3: row.get(8)?,
                error_msg: row.get(9)?,
                retry_count: row.get(10)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()
        .context("Failed to read interrupted tasks")?;

    Ok(records)
}

/// Recovery procedure: clean up .tmp files and reset interrupted tasks.
///
/// Returns the list of tasks that were reset and are now ready to retry.
pub fn recover_job(conn: &Connection, job_id: &str) -> Result<Vec<CheckpointRecord>> {
    // 1. Find interrupted tasks
    let interrupted = get_interrupted_tasks(conn, job_id)?;

    for task in &interrupted {
        // 2. Clean up .tmp files
        let dest = Path::new(&task.dest_path);
        let tmp_path = atomic_writer::AtomicWriter::temp_path_for(dest);
        if tmp_path.exists() {
            std::fs::remove_file(&tmp_path).ok();
            log::info!("Cleaned up orphaned tmp file: {:?}", tmp_path);
        }

        // 3. Reset interrupted copy/failure tasks to pending. A task interrupted
        // during destination verification already has copy hashes, so it can be
        // verified again without requiring the source card.
        if task.status == STATUS_VERIFYING && has_persisted_hash(task) {
            continue;
        }

        update_task_status(conn, &task.task_id, STATUS_PENDING)?;
        log::info!(
            "Reset interrupted task {} ({} -> {})",
            task.task_id,
            task.source_path,
            task.dest_path
        );
    }

    // 4. Return all pending tasks (original pending + reset ones)
    get_pending_tasks(conn, job_id)
}

/// Get job progress summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobProgress {
    pub job_id: String,
    pub total_tasks: usize,
    pub completed: usize,
    pub pending: usize,
    pub copying: usize,
    pub failed: usize,
    pub skipped: usize,
    pub total_bytes: u64,
    pub completed_bytes: u64,
}

pub fn get_job_progress(conn: &Connection, job_id: &str) -> Result<JobProgress> {
    let total_tasks: usize = conn.query_row(
        "SELECT COUNT(*) FROM copy_tasks WHERE job_id = ?1",
        params![job_id],
        |row| row.get(0),
    )?;

    let completed: usize = conn.query_row(
        "SELECT COUNT(*) FROM copy_tasks WHERE job_id = ?1 AND status IN ('completed', 'skipped')",
        params![job_id],
        |row| row.get(0),
    )?;

    let pending: usize = conn.query_row(
        "SELECT COUNT(*) FROM copy_tasks WHERE job_id = ?1 AND status = 'pending'",
        params![job_id],
        |row| row.get(0),
    )?;

    let copying: usize = conn.query_row(
        "SELECT COUNT(*) FROM copy_tasks WHERE job_id = ?1 AND status IN ('copying', 'verifying')",
        params![job_id],
        |row| row.get(0),
    )?;

    let failed: usize = conn.query_row(
        "SELECT COUNT(*) FROM copy_tasks WHERE job_id = ?1 AND status = 'failed'",
        params![job_id],
        |row| row.get(0),
    )?;

    let skipped: usize = conn.query_row(
        "SELECT COUNT(*) FROM copy_tasks WHERE job_id = ?1 AND status = 'skipped'",
        params![job_id],
        |row| row.get(0),
    )?;

    let total_bytes: i64 = conn.query_row(
        "SELECT COALESCE(SUM(file_size), 0) FROM copy_tasks WHERE job_id = ?1",
        params![job_id],
        |row| row.get(0),
    )?;

    let completed_bytes: i64 = conn.query_row(
        "SELECT COALESCE(SUM(file_size), 0) FROM copy_tasks WHERE job_id = ?1 AND status = 'completed'",
        params![job_id],
        |row| row.get(0),
    )?;

    let skipped_bytes: i64 = conn.query_row(
        "SELECT COALESCE(SUM(file_size), 0) FROM copy_tasks WHERE job_id = ?1 AND status = 'skipped'",
        params![job_id],
        |row| row.get(0),
    )?;

    Ok(JobProgress {
        job_id: job_id.to_string(),
        total_tasks,
        completed,
        pending,
        copying,
        failed,
        skipped,
        total_bytes: total_bytes as u64,
        completed_bytes: (completed_bytes + skipped_bytes) as u64,
    })
}

/// Delete a single job and all its tasks by job ID
pub fn delete_job_by_id(conn: &Connection, job_id: &str) -> Result<()> {
    conn.execute(
        "DELETE FROM hash_observations WHERE job_id = ?1",
        params![job_id],
    )?;
    conn.execute(
        "DELETE FROM repair_attempts WHERE job_id = ?1",
        params![job_id],
    )?;
    conn.execute(
        "DELETE FROM replica_states WHERE job_id = ?1",
        params![job_id],
    )?;
    conn.execute("DELETE FROM job_events WHERE job_id = ?1", params![job_id])?;
    conn.execute("DELETE FROM copy_tasks WHERE job_id = ?1", params![job_id])?;
    let deleted = conn.execute("DELETE FROM jobs WHERE id = ?1", params![job_id])?;
    if deleted == 0 {
        anyhow::bail!("Job not found: {}", job_id);
    }
    Ok(())
}

/// Clear job records older than the given number of days
pub fn clear_old_jobs(conn: &Connection, days: u32) -> Result<usize> {
    for table in ["hash_observations", "repair_attempts", "replica_states"] {
        conn.execute(
            &format!(
                "DELETE FROM {table} WHERE job_id IN (
                    SELECT id FROM jobs WHERE created_at < datetime('now', ?1)
                )"
            ),
            params![format!("-{} days", days)],
        )?;
    }
    let _deleted_events: usize = conn.execute(
        "DELETE FROM job_events WHERE job_id IN (
            SELECT id FROM jobs WHERE created_at < datetime('now', ?1)
        )",
        params![format!("-{} days", days)],
    )?;
    let _deleted_tasks: usize = conn.execute(
        "DELETE FROM copy_tasks WHERE job_id IN (
            SELECT id FROM jobs WHERE created_at < datetime('now', ?1)
        )",
        params![format!("-{} days", days)],
    )?;
    let deleted_jobs: usize = conn.execute(
        "DELETE FROM jobs WHERE created_at < datetime('now', ?1)",
        params![format!("-{} days", days)],
    )?;
    Ok(deleted_jobs)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup_test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("PRAGMA journal_mode=WAL;").unwrap();
        conn.execute_batch("PRAGMA foreign_keys=ON;").unwrap();
        init_schema(&conn).unwrap();
        conn
    }

    #[test]
    fn test_create_job_and_tasks() {
        let conn = setup_test_db();
        create_job(&conn, "job-1", "Day 1 Offload", "/Volumes/CARD_A", None).unwrap();
        insert_task(
            &conn,
            "t-1",
            "job-1",
            "/src/clip1.mov",
            "/dst/clip1.mov",
            1000,
        )
        .unwrap();
        insert_task(
            &conn,
            "t-2",
            "job-1",
            "/src/clip2.mov",
            "/dst/clip2.mov",
            2000,
        )
        .unwrap();

        let pending = get_pending_tasks(&conn, "job-1").unwrap();
        assert_eq!(pending.len(), 2);
    }

    #[test]
    fn resume_context_rejects_replaced_media() {
        let conn = setup_test_db();
        create_job(&conn, "job-1", "Test", "/src", None).unwrap();
        bind_or_validate_job_context(&conn, "job-1", "source-a", "[\"dest-a\"]", "[]").unwrap();
        bind_or_validate_job_context(&conn, "job-1", "source-a", "[\"dest-a\"]", "[]").unwrap();

        let wrong_source =
            bind_or_validate_job_context(&conn, "job-1", "source-b", "[\"dest-a\"]", "[]")
                .unwrap_err();
        assert!(wrong_source.to_string().contains("source volume identity"));

        let changed_snapshot = bind_or_validate_job_context(
            &conn,
            "job-1",
            "source-a",
            "[\"dest-a\"]",
            "[{\"path\":\"new.mov\"}]",
        )
        .unwrap_err();
        assert!(changed_snapshot
            .to_string()
            .contains("source file snapshot"));
    }

    #[test]
    fn corrupted_checkpoint_never_becomes_a_fresh_job_silently() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("offload.sqlite");
        std::fs::write(&path, b"not a sqlite database").unwrap();
        let error = open_db(&path).unwrap_err();
        assert!(!error.to_string().is_empty());
        assert_eq!(std::fs::read(path).unwrap(), b"not a sqlite database");
    }

    #[test]
    fn test_task_status_transitions() {
        let conn = setup_test_db();
        create_job(&conn, "job-1", "Test", "/src", None).unwrap();
        insert_task(&conn, "t-1", "job-1", "/src/a.mov", "/dst/a.mov", 500).unwrap();

        update_task_status(&conn, "t-1", STATUS_COPYING).unwrap();
        let pending = get_pending_tasks(&conn, "job-1").unwrap();
        assert_eq!(pending.len(), 0); // no longer pending

        update_task_completed(
            &conn,
            "t-1",
            &TaskHashes {
                xxh64: Some("abc123".into()),
                sha256: Some("def456".into()),
                ..Default::default()
            },
        )
        .unwrap();

        let progress = get_job_progress(&conn, "job-1").unwrap();
        assert_eq!(progress.completed, 1);
        assert_eq!(progress.completed_bytes, 500);
    }

    #[test]
    fn test_failure_and_retry_count() {
        let conn = setup_test_db();
        create_job(&conn, "job-1", "Test", "/src", None).unwrap();
        insert_task(&conn, "t-1", "job-1", "/src/a.mov", "/dst/a.mov", 500).unwrap();

        update_task_failed(&conn, "t-1", "IO error: disk full").unwrap();
        update_task_failed(&conn, "t-1", "IO error: disk full").unwrap();

        let progress = get_job_progress(&conn, "job-1").unwrap();
        assert_eq!(progress.failed, 1);
    }

    #[test]
    fn evidence_passes_repairs_and_replica_state_are_transactionally_persisted() {
        let conn = setup_test_db();
        create_job(&conn, "job-evidence", "Evidence", "/src", None).unwrap();
        append_hash_observation(
            &conn,
            "job-evidence",
            "sourcePreRead",
            "clip.mov",
            None,
            r#"{"side":"sourcePreRead"}"#,
        )
        .unwrap();
        append_repair_attempt(
            &conn,
            "job-evidence",
            "clip.mov",
            "/dest/clip.mov",
            1,
            r#"{"attempt":1}"#,
        )
        .unwrap();
        upsert_replica_state(
            &conn,
            "job-evidence",
            "clip.mov",
            "/dest/clip.mov",
            r#"{"status":"verifyFailed"}"#,
        )
        .unwrap();
        upsert_replica_state(
            &conn,
            "job-evidence",
            "clip.mov",
            "/dest/clip.mov",
            r#"{"status":"verified"}"#,
        )
        .unwrap();

        assert_eq!(
            get_hash_observation_payloads(&conn, "job-evidence").unwrap(),
            vec![r#"{"side":"sourcePreRead"}"#]
        );
        assert_eq!(
            repair_attempt_count(&conn, "job-evidence", "clip.mov", "/dest/clip.mov").unwrap(),
            1
        );
        assert_eq!(
            get_replica_state_payloads(&conn, "job-evidence").unwrap(),
            vec![r#"{"status":"verified"}"#]
        );
        let version: i64 = conn
            .pragma_query_value(None, "user_version", |row| row.get(0))
            .unwrap();
        assert_eq!(version, 4);
    }

    #[test]
    fn test_success_states_clear_stale_error_message() {
        let conn = setup_test_db();
        create_job(&conn, "job-1", "Test", "/src", None).unwrap();
        insert_task(&conn, "t-1", "job-1", "/src/a.mov", "/dst/a.mov", 500).unwrap();
        insert_task(&conn, "t-2", "job-1", "/src/b.mov", "/dst/b.mov", 500).unwrap();
        insert_task(&conn, "t-3", "job-1", "/src/c.mov", "/dst/c.mov", 500).unwrap();

        update_task_failed(&conn, "t-1", "Verify failed: old mismatch").unwrap();
        update_task_completed(
            &conn,
            "t-1",
            &TaskHashes {
                xxh64: Some("abc123".into()),
                ..Default::default()
            },
        )
        .unwrap();

        update_task_failed(&conn, "t-2", "Copy failed: old temp rename").unwrap();
        update_task_status(&conn, "t-2", STATUS_COMPLETED).unwrap();

        update_task_failed(&conn, "t-3", "Skipped after old failure").unwrap();
        update_task_skipped(&conn, "t-3", &TaskHashes::default()).unwrap();

        for task_id in ["t-1", "t-2", "t-3"] {
            let error_msg: Option<String> = conn
                .query_row(
                    "SELECT error_msg FROM copy_tasks WHERE id = ?1",
                    params![task_id],
                    |row| row.get(0),
                )
                .unwrap();
            assert_eq!(error_msg, None);
        }

        let retry_note: Option<String> = conn
            .query_row(
                "SELECT retry_note FROM copy_tasks WHERE id = 't-1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(retry_note
            .as_deref()
            .unwrap_or_default()
            .contains("old mismatch"));
    }

    #[test]
    fn test_delete_job_by_id() {
        let conn = setup_test_db();
        create_job(&conn, "job-1", "To Delete", "/src", None).unwrap();
        insert_task(&conn, "t-1", "job-1", "/src/a.mov", "/dst/a.mov", 100).unwrap();
        insert_task(&conn, "t-2", "job-1", "/src/b.mov", "/dst/b.mov", 200).unwrap();

        // Delete should remove both tasks and the job
        delete_job_by_id(&conn, "job-1").unwrap();

        let progress = get_job_progress(&conn, "job-1");
        assert!(progress.is_ok());
        assert_eq!(progress.unwrap().total_tasks, 0);

        // Deleting again should fail
        assert!(delete_job_by_id(&conn, "job-1").is_err());
    }

    #[tokio::test]
    async fn test_recover_interrupted_tasks() {
        let conn = setup_test_db();
        create_job(&conn, "job-1", "Test", "/src", None).unwrap();
        insert_task(&conn, "t-1", "job-1", "/src/a.mov", "/dst/a.mov", 100).unwrap();
        insert_task(&conn, "t-2", "job-1", "/src/b.mov", "/dst/b.mov", 200).unwrap();
        insert_task(&conn, "t-3", "job-1", "/src/c.mov", "/dst/c.mov", 300).unwrap();

        // Simulate: t-1 completed, t-2 was copying when crash happened
        update_task_completed(
            &conn,
            "t-1",
            &TaskHashes {
                xxh64: Some("hash1".into()),
                ..Default::default()
            },
        )
        .unwrap();
        update_task_status(&conn, "t-2", STATUS_COPYING).unwrap();

        let recovered = recover_job(&conn, "job-1").unwrap();
        // t-2 should be reset to pending, t-3 was already pending
        assert_eq!(recovered.len(), 2);

        let progress = get_job_progress(&conn, "job-1").unwrap();
        assert_eq!(progress.completed, 1);
        assert_eq!(progress.pending, 2);
    }
}
