//! Headless interface for auditable offload, verification, resume and reports.

use anyhow::{bail, Context, Result};
use offload_core::offload::checkpoint;
use offload_core::offload::copy_engine::atomic_writer::AtomicWriter;
use offload_core::offload::hash_engine::HashAlgorithm;
use offload_core::offload::mhl::verifier::{verify_mhl_path, MhlVerifyOptions};
use offload_core::offload::orchestrator::{
    run_offload, DitContact, OffloadRequest, OffloadSummary, VerificationProfile,
};
use offload_core::offload::report::{self, ReportFormat};
use std::env;
use std::path::{Path, PathBuf};

fn usage() {
    eprintln!(
        "ProofCat CLI\n\n\
         offload --source PATH --dest PATH [--dest PATH] [--profile archive-max|fast] [--db PATH] [--job ID] [--small-file-workers 1..8] [--contact 'Name|Role|Phone or email'] [--auto-eject] [--no-mhl] [--hash sha256|md5]\n\
         resume --job ID --db PATH\n\
         job --job ID --db PATH\n\
         verify PATH [--all]\n\
         report --job ID --db PATH --format json|html|csv|txt [--output PATH]"
    );
}

fn values(args: &[String], flag: &str) -> Vec<String> {
    args.windows(2)
        .filter(|pair| pair[0] == flag)
        .map(|pair| pair[1].clone())
        .collect()
}

fn value(args: &[String], flag: &str) -> Option<String> {
    values(args, flag).into_iter().last()
}

fn parse_hash(value: &str) -> Result<HashAlgorithm> {
    match value.to_ascii_lowercase().as_str() {
        "xxh64" => Ok(HashAlgorithm::XXH64),
        "xxh3" => Ok(HashAlgorithm::XXH3),
        "xxh128" => Ok(HashAlgorithm::XXH128),
        "blake3" => Ok(HashAlgorithm::BLAKE3),
        "sha256" | "sha-256" => Ok(HashAlgorithm::SHA256),
        "md5" => Ok(HashAlgorithm::MD5),
        _ => bail!("Unsupported hash algorithm: {value}"),
    }
}

fn parse_contact(value: &str) -> DitContact {
    let mut pieces = value.splitn(3, '|').map(str::trim);
    DitContact {
        name: pieces.next().unwrap_or_default().to_string(),
        role: pieces.next().unwrap_or_default().to_string(),
        contact: pieces.next().unwrap_or_default().to_string(),
    }
}

fn load_summary(db: &Path, job_id: &str) -> Result<OffloadSummary> {
    let conn = checkpoint::open_db(db)?;
    let json = checkpoint::get_job_summary(&conn, job_id)?
        .with_context(|| format!("Job has no completed evidence snapshot: {job_id}"))?;
    Ok(serde_json::from_str(&json)?)
}

fn emit_progress(progress: offload_core::offload::orchestrator::OffloadProgress) {
    eprintln!(
        "{} {}/{} {}",
        progress.phase, progress.file_index, progress.total_files, progress.current_file
    );
    // Debug-only deterministic window used by the real SIGKILL harness. The
    // release binary neither reads nor honors this variable.
    #[cfg(debug_assertions)]
    if let Ok(delay) = env::var("META_REPORT_HARNESS_PHASE_DELAY_MS") {
        if let Ok(delay) = delay.parse::<u64>() {
            std::thread::sleep(std::time::Duration::from_millis(delay.min(5_000)));
        }
    }
}

async fn command_offload(args: &[String]) -> Result<()> {
    let source = value(args, "--source").context("--source is required")?;
    let destinations = values(args, "--dest");
    if destinations.is_empty() {
        bail!("At least one --dest is required");
    }
    let db = PathBuf::from(value(args, "--db").unwrap_or_else(|| "offload.sqlite".into()));
    let profile = match value(args, "--profile").as_deref() {
        Some("fast") => VerificationProfile::Fast,
        Some("archive-max") | Some("archiveMax") | None => VerificationProfile::ArchiveMax,
        Some(other) => bail!("Unsupported profile: {other}"),
    };
    let mut algorithms = vec![HashAlgorithm::XXH64];
    for hash in values(args, "--hash") {
        let algorithm = parse_hash(&hash)?;
        if !algorithms.contains(&algorithm) {
            algorithms.push(algorithm);
        }
    }
    let job_id = value(args, "--job")
        .unwrap_or_else(|| format!("job-{}", chrono::Utc::now().format("%Y%m%d-%H%M%S%.3f")));
    println!("{job_id}");
    let request = OffloadRequest {
        source: PathBuf::from(source),
        destinations: destinations.into_iter().map(PathBuf::from).collect(),
        algorithms,
        write_mhl: !args.iter().any(|arg| arg == "--no-mhl"),
        checkpoint_db: Some(db),
        profile,
        job_id: Some(job_id),
        small_file_concurrency: value(args, "--small-file-workers")
            .map(|value| value.parse())
            .transpose()
            .context("--small-file-workers must be an integer from 1 to 8")?
            .unwrap_or(1),
        report_contacts: values(args, "--contact")
            .into_iter()
            .filter_map(|value| parse_contact(&value).normalized())
            .collect(),
        auto_eject: args.iter().any(|arg| arg == "--auto-eject"),
    };
    let summary = run_offload(&request, None, None, &emit_progress).await?;
    println!("{}", serde_json::to_string_pretty(&summary)?);
    Ok(())
}

async fn command_resume(args: &[String]) -> Result<()> {
    let job_id = value(args, "--job").context("--job is required")?;
    let db = PathBuf::from(value(args, "--db").context("--db is required")?);
    let config_json = {
        let conn = checkpoint::open_db(&db)?;
        checkpoint::get_job_config(&conn, &job_id)?
            .with_context(|| format!("Checkpoint job not found: {job_id}"))?
    };
    let mut request: OffloadRequest = serde_json::from_str(&config_json)?;
    request.job_id = Some(job_id);
    request.checkpoint_db = Some(db);
    let summary = run_offload(&request, None, None, &emit_progress).await?;
    println!("{}", serde_json::to_string_pretty(&summary)?);
    Ok(())
}

fn command_verify(args: &[String]) -> Result<()> {
    let path = args
        .first()
        .filter(|value| !value.starts_with('-'))
        .context("verify requires a path")?;
    let report = verify_mhl_path(
        Path::new(path),
        MhlVerifyOptions {
            verify_all_generations: args.iter().any(|arg| arg == "--all"),
            ..Default::default()
        },
    )?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    if !report.summary.success {
        std::process::exit(2);
    }
    Ok(())
}

async fn command_report(args: &[String]) -> Result<()> {
    let job_id = value(args, "--job").context("--job is required")?;
    let db = PathBuf::from(value(args, "--db").context("--db is required")?);
    let format = ReportFormat::parse(&value(args, "--format").unwrap_or_else(|| "json".into()))?;
    let rendered = report::render(&load_summary(&db, &job_id)?, format)?;
    if let Some(output) = value(args, "--output") {
        let mut writer = AtomicWriter::new(Path::new(&output))
            .await
            .with_context(|| format!("Failed to create report: {output}"))?;
        writer.write(rendered.as_bytes()).await?;
        writer.finalize().await?;
        println!("{output}");
    } else {
        println!("{rendered}");
    }
    Ok(())
}

fn command_job(args: &[String]) -> Result<()> {
    let job_id = value(args, "--job").context("--job is required")?;
    let db = PathBuf::from(value(args, "--db").context("--db is required")?);
    let conn = checkpoint::open_db(&db)?;
    let record = checkpoint::get_job(&conn, &job_id)?
        .with_context(|| format!("Checkpoint job not found: {job_id}"))?;
    let progress = checkpoint::get_job_progress(&conn, &job_id)?;
    let summary = record
        .summary_json
        .as_deref()
        .map(serde_json::from_str::<OffloadSummary>)
        .transpose()?;
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "jobId": job_id,
            "state": record.status,
            "progress": progress,
            "summary": summary,
        }))?
    );
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let mut args: Vec<String> = env::args().skip(1).collect();
    let Some(command) = args.first().cloned() else {
        usage();
        return Ok(());
    };
    args.remove(0);
    match command.as_str() {
        "offload" => command_offload(&args).await,
        "resume" => command_resume(&args).await,
        "job" => command_job(&args),
        "verify" => command_verify(&args),
        "report" => command_report(&args).await,
        "help" | "--help" | "-h" => {
            usage();
            Ok(())
        }
        _ => {
            usage();
            bail!("Unknown command: {command}")
        }
    }
}
