//! Deterministic evidence report rendering.
//!
//! JSON is the canonical representation persisted in the checkpoint database.
//! Human-readable formats are pure views over that snapshot, so re-exporting a
//! job cannot silently change its verdict or file results.

use anyhow::{bail, Result};
use serde::{Deserialize, Serialize};

use super::hash_engine::HashAlgorithm;
use super::orchestrator::{DitContact, OffloadSummary, ReplicaEvidence};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ReportFormat {
    Json,
    Html,
    Csv,
    Txt,
}

impl ReportFormat {
    pub fn parse(value: &str) -> Result<Self> {
        match value.to_ascii_lowercase().as_str() {
            "json" => Ok(Self::Json),
            "html" | "htm" => Ok(Self::Html),
            "csv" => Ok(Self::Csv),
            "txt" | "text" => Ok(Self::Txt),
            _ => bail!("Unsupported report format: {value}"),
        }
    }

    pub fn extension(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Html => "html",
            Self::Csv => "csv",
            Self::Txt => "txt",
        }
    }
}

pub fn render(summary: &OffloadSummary, format: ReportFormat) -> Result<String> {
    match format {
        ReportFormat::Json => Ok(serde_json::to_string_pretty(summary)?),
        ReportFormat::Html => Ok(render_html(summary)),
        ReportFormat::Csv => Ok(render_csv(summary)),
        ReportFormat::Txt => Ok(render_text(summary)),
    }
}

fn hash(replica: &ReplicaEvidence, algorithm: HashAlgorithm, observed: bool) -> &str {
    let values = if observed {
        &replica.observed_hashes
    } else {
        &replica.expected_hashes
    };
    values
        .iter()
        .find(|item| item.algorithm == algorithm)
        .map(|item| item.hex_digest.as_str())
        .unwrap_or("")
}

fn html_escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}

fn csv_field(value: &str) -> String {
    // Evidence paths are untrusted media metadata. Neutralize spreadsheet
    // formulas while retaining valid RFC 4180 quoting.
    let value = if value.starts_with(['=', '+', '-', '@', '\t', '\r']) {
        format!("'{value}")
    } else {
        value.to_string()
    };
    if value.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value
    }
}

fn contacts_text(summary: &OffloadSummary) -> String {
    summary
        .report_contacts
        .iter()
        .map(contact_text)
        .collect::<Vec<_>>()
        .join("; ")
}

fn contact_text(contact: &DitContact) -> String {
    [
        contact.name.as_str(),
        contact.role.as_str(),
        contact.contact.as_str(),
    ]
    .into_iter()
    .filter(|value| !value.is_empty())
    .collect::<Vec<_>>()
    .join(" | ")
}

fn render_csv(summary: &OffloadSummary) -> String {
    let mut output = String::from(
        "evidence_schema_version,app,app_version,commit,started_at,finished_at,job_id,profile,verdict,source_volume,source_fingerprint,destination_volume,destination_fingerprint,file,file_bytes,destination,status,expected_xxh64,expected_blake3,observed_xxh64,observed_blake3,repair_attempts,contacts,mhl_paths,error\r\n",
    );
    for replica in &summary.replicas {
        let destination_volume = summary
            .destination_volumes
            .iter()
            .find(|volume| replica.destination.starts_with(&volume.path));
        let fields = vec![
            summary.evidence_schema_version.to_string(),
            summary.app_name.clone(),
            summary.app_version.clone(),
            summary.commit.clone(),
            summary.started_at.clone(),
            summary.finished_at.clone(),
            summary.job_id.clone(),
            match summary.profile {
                super::orchestrator::VerificationProfile::Fast => "fast",
                super::orchestrator::VerificationProfile::ArchiveMax => "archiveMax",
            }
            .into(),
            match summary.verdict {
                super::orchestrator::OffloadVerdict::CopyComplete => "COPY_COMPLETE",
                super::orchestrator::OffloadVerdict::ArchiveVerified => "ARCHIVE_VERIFIED",
                super::orchestrator::OffloadVerdict::SafeToFormat => "SAFE_TO_FORMAT",
                super::orchestrator::OffloadVerdict::Failed => "FAILED",
            }
            .into(),
            summary.source_volume.key.clone(),
            summary.source_volume.fingerprint.clone(),
            destination_volume
                .map(|volume| volume.key.clone())
                .unwrap_or_default(),
            destination_volume
                .map(|volume| volume.fingerprint.clone())
                .unwrap_or_default(),
            replica.file.clone(),
            replica.bytes.to_string(),
            replica.destination.clone(),
            replica.status.as_str().into(),
            hash(replica, HashAlgorithm::XXH64, false).into(),
            hash(replica, HashAlgorithm::BLAKE3, false).into(),
            hash(replica, HashAlgorithm::XXH64, true).into(),
            hash(replica, HashAlgorithm::BLAKE3, true).into(),
            replica.repair_attempts.to_string(),
            contacts_text(summary),
            summary.mhl_paths.join(";"),
            replica.error.clone().unwrap_or_default(),
        ];
        let rendered: Vec<String> = fields.iter().map(|value| csv_field(value)).collect();
        output.push_str(&rendered.join(","));
        output.push_str("\r\n");
    }
    output
}

fn render_text(summary: &OffloadSummary) -> String {
    let mut output = format!(
        "ProofCat Offload Evidence v{}\nApp: {} {} ({})\nStarted: {}\nFinished: {}\nJob: {}\nProfile: {:?}\nVerdict: {:?}\nSafe to format: {}\nAuto eject requested: {}\nSource volume: {}\nSource fingerprint: {}\nFiles: {}\nVerified replicas: {}\nVerification failures: {}\n\n",
        summary.evidence_schema_version,
        summary.app_name,
        summary.app_version,
        summary.commit,
        summary.started_at,
        summary.finished_at,
        summary.job_id,
        summary.profile,
        summary.verdict,
        summary.safe_to_format,
        summary.auto_eject_requested,
        summary.source_volume.key,
        summary.source_volume.fingerprint,
        summary.total_files,
        summary.verified_replicas,
        summary.verification_failed,
    );
    for warning in &summary.warnings {
        output.push_str(&format!("WARNING: {warning}\n"));
    }
    if !summary.warnings.is_empty() {
        output.push('\n');
    }
    for volume in &summary.destination_volumes {
        output.push_str(&format!(
            "Destination volume: {} | fingerprint={} | path={}\n",
            volume.key, volume.fingerprint, volume.path
        ));
    }
    for mhl_path in &summary.mhl_paths {
        output.push_str(&format!("MHL: {mhl_path}\n"));
    }
    if !summary.destination_volumes.is_empty() || !summary.mhl_paths.is_empty() {
        output.push('\n');
    }
    if !summary.destination_preflight.is_empty() {
        output.push_str("Destination preflight:\n");
        for check in &summary.destination_preflight {
            output.push_str(&format!(
                "{} | available={} | required={}\n",
                check.destination, check.available_bytes, check.required_bytes
            ));
        }
        output.push('\n');
    }
    if !summary.report_contacts.is_empty() {
        output.push_str("DIT contacts:\n");
        for contact in &summary.report_contacts {
            output.push_str(&format!("{}\n", contact_text(contact)));
        }
        output.push('\n');
    }
    for replica in &summary.replicas {
        output.push_str(&format!(
            "[{}] {} ({} bytes) -> {} | expected XXH64={} BLAKE3={} | observed XXH64={} BLAKE3={} | repairs={}{}\n",
            replica.status,
            replica.file,
            replica.bytes,
            replica.destination,
            hash(replica, HashAlgorithm::XXH64, false),
            hash(replica, HashAlgorithm::BLAKE3, false),
            hash(replica, HashAlgorithm::XXH64, true),
            hash(replica, HashAlgorithm::BLAKE3, true),
            replica.repair_attempts,
            replica
                .error
                .as_ref()
                .map(|error| format!(" | error={error}"))
                .unwrap_or_default(),
        ));
    }
    if !summary.repairs.is_empty() {
        output.push_str("\nRepair history:\n");
        for repair in &summary.repairs {
            output.push_str(&format!(
                "attempt={} success={} file={} destination={} source={}{}\n",
                repair.attempt,
                repair.success,
                repair.file,
                repair.destination,
                repair.source,
                repair
                    .error
                    .as_ref()
                    .map(|error| format!(" error={error}"))
                    .unwrap_or_default()
            ));
        }
    }
    output
}

fn render_html(summary: &OffloadSummary) -> String {
    let rows = summary
        .replicas
        .iter()
        .map(|replica| {
            format!(
                "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td><code>{}</code></td><td><code>{}</code></td><td><code>{}</code></td><td><code>{}</code></td><td>{}</td><td>{}</td></tr>",
                html_escape(replica.status.as_str()),
                html_escape(&replica.file),
                replica.bytes,
                html_escape(&replica.destination),
                html_escape(hash(replica, HashAlgorithm::XXH64, false)),
                html_escape(hash(replica, HashAlgorithm::BLAKE3, false)),
                html_escape(hash(replica, HashAlgorithm::XXH64, true)),
                html_escape(hash(replica, HashAlgorithm::BLAKE3, true)),
                replica.repair_attempts,
                html_escape(replica.error.as_deref().unwrap_or("")),
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let warnings = summary
        .warnings
        .iter()
        .map(|warning| format!("<li>{}</li>", html_escape(warning)))
        .collect::<Vec<_>>()
        .join("");
    let destinations = summary
        .destination_volumes
        .iter()
        .map(|volume| {
            format!(
                "<li><code>{}</code> · fingerprint <code>{}</code> · {}</li>",
                html_escape(&volume.key),
                html_escape(&volume.fingerprint),
                html_escape(&volume.path)
            )
        })
        .collect::<Vec<_>>()
        .join("");
    let mhl_paths = summary
        .mhl_paths
        .iter()
        .map(|path| format!("<li><code>{}</code></li>", html_escape(path)))
        .collect::<Vec<_>>()
        .join("");
    let contacts = summary
        .report_contacts
        .iter()
        .map(|contact| format!("<li>{}</li>", html_escape(&contact_text(contact))))
        .collect::<Vec<_>>()
        .join("");
    let preflight = summary
        .destination_preflight
        .iter()
        .map(|check| {
            format!(
                "<li>{}: {} bytes available; {} bytes required</li>",
                html_escape(&check.destination),
                check.available_bytes,
                check.required_bytes
            )
        })
        .collect::<Vec<_>>()
        .join("");
    let repairs = summary
        .repairs
        .iter()
        .map(|repair| {
            format!(
                "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                repair.attempt,
                repair.success,
                html_escape(&repair.file),
                html_escape(&repair.destination),
                html_escape(&repair.source),
                html_escape(repair.error.as_deref().unwrap_or(""))
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        "<!doctype html><html lang=\"en\"><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width\"><title>Offload {job}</title><style>body{{font:14px system-ui;margin:32px;color:#202124}}h1{{margin-bottom:4px}}.pass{{color:#087830}}.fail{{color:#b3261e}}table{{border-collapse:collapse;width:100%;margin-top:20px}}th,td{{border:1px solid #ddd;padding:7px;text-align:left;vertical-align:top}}th{{background:#f5f5f5}}code{{font-size:11px;word-break:break-all}}</style><h1>ProofCat Offload Evidence</h1><p>{app} {version} · commit <code>{commit}</code><br>Started: {started}<br>Finished: {finished}<br>Job <code>{job}</code><br>Profile: {profile:?}<br>Verdict: <strong class=\"{class}\">{verdict:?}</strong><br>Safe to format: <strong>{safe}</strong><br>Auto eject requested: {auto_eject}<br>Source volume: <code>{source_volume}</code><br>Source fingerprint: <code>{source_fingerprint}</code></p>{warning_block}<h2>DIT contacts</h2><ul>{contacts}</ul><h2>Destination preflight</h2><ul>{preflight}</ul><h2>Destination volumes</h2><ul>{destinations}</ul><h2>MHL</h2><ul>{mhl_paths}</ul><table><thead><tr><th>Status</th><th>File</th><th>Bytes</th><th>Destination</th><th>Expected XXH64</th><th>Expected BLAKE3</th><th>Observed XXH64</th><th>Observed BLAKE3</th><th>Repairs</th><th>Error</th></tr></thead><tbody>{rows}</tbody></table>{repair_block}</html>",
        app = html_escape(&summary.app_name),
        version = html_escape(&summary.app_version),
        commit = html_escape(&summary.commit),
        started = html_escape(&summary.started_at),
        finished = html_escape(&summary.finished_at),
        job = html_escape(&summary.job_id),
        profile = summary.profile,
        verdict = summary.verdict,
        class = if summary.failed == 0 { "pass" } else { "fail" },
        safe = summary.safe_to_format,
        auto_eject = summary.auto_eject_requested,
        source_volume = html_escape(&summary.source_volume.key),
        source_fingerprint = html_escape(&summary.source_volume.fingerprint),
        destinations = destinations,
        mhl_paths = mhl_paths,
        contacts = contacts,
        preflight = preflight,
        warning_block = if warnings.is_empty() {
            String::new()
        } else {
            format!("<h2>Warnings</h2><ul>{warnings}</ul>")
        },
        repair_block = if repairs.is_empty() {
            String::new()
        } else {
            format!("<h2>Repair history</h2><table><thead><tr><th>Attempt</th><th>Success</th><th>File</th><th>Destination</th><th>Source</th><th>Error</th></tr></thead><tbody>{repairs}</tbody></table>")
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::offload::orchestrator::{OffloadVerdict, VerificationProfile};

    fn sample() -> OffloadSummary {
        OffloadSummary {
            evidence_schema_version: 2,
            app_name: "ProofCat".into(),
            app_version: "0.2.4".into(),
            commit: "test".into(),
            started_at: "2026-01-01T00:00:00Z".into(),
            finished_at: "2026-01-01T00:00:01Z".into(),
            job_id: "job-1".into(),
            total_files: 1,
            copied: 1,
            skipped: 0,
            failed: 0,
            bytes_copied: 3,
            failures: Vec::new(),
            mhl_paths: vec!["/d/ascmhl/a.mhl".into()],
            profile: VerificationProfile::ArchiveMax,
            hash_policy: crate::offload::orchestrator::HashPolicy::default(),
            verdict: OffloadVerdict::ArchiveVerified,
            safe_to_format: false,
            verified_replicas: 1,
            verification_failed: 0,
            effective_small_file_workers: 1,
            warnings: vec!["one destination".into()],
            observations: Vec::new(),
            replicas: vec![ReplicaEvidence {
                file: "A,<clip>.mov".into(),
                bytes: 3,
                destination: "/d".into(),
                status: crate::offload::orchestrator::ReplicaState::Verified,
                expected_hashes: Vec::new(),
                observed_hashes: Vec::new(),
                repair_attempts: 0,
                error: None,
            }],
            repairs: Vec::new(),
            source_volume: crate::offload::volume::VolumeIdentity {
                key: "dev:1".into(),
                fingerprint: "volume:source".into(),
                path: "/source".into(),
                device_type: crate::offload::volume::DeviceType::Unknown,
                total_bytes: 10,
                available_bytes: 5,
                is_physical: true,
            },
            destination_volumes: vec![crate::offload::volume::VolumeIdentity {
                key: "dev:2".into(),
                fingerprint: "volume:destination".into(),
                path: "/d".into(),
                device_type: crate::offload::volume::DeviceType::Unknown,
                total_bytes: 10,
                available_bytes: 5,
                is_physical: true,
            }],
            source_snapshot: Vec::new(),
            destination_preflight: vec![crate::offload::orchestrator::DestinationPreflight {
                destination: "/d".into(),
                required_bytes: 3,
                available_bytes: 5,
            }],
            report_contacts: vec![crate::offload::orchestrator::DitContact {
                name: "Nikolay".into(),
                role: "DIT".into(),
                contact: "dit@example.test".into(),
            }],
            auto_eject_requested: false,
        }
    }

    #[test]
    fn csv_quotes_special_file_names() {
        let csv = render(&sample(), ReportFormat::Csv).unwrap();
        assert!(csv.contains("\"A,<clip>.mov\""));
        assert!(csv.contains(",3,"));
        assert!(csv.contains("Nikolay | DIT | dit@example.test"));
    }

    #[test]
    fn csv_neutralizes_spreadsheet_formulas_from_media_paths() {
        assert_eq!(csv_field("=cmd|' /C calc'!A0"), "'=cmd|' /C calc'!A0");
        assert_eq!(csv_field("+SUM(1,2)"), "\"'+SUM(1,2)\"");
    }

    #[test]
    fn html_escapes_paths() {
        let html = render(&sample(), ReportFormat::Html).unwrap();
        assert!(html.contains("A,&lt;clip&gt;.mov"));
        assert!(!html.contains("A,<clip>.mov"));
        assert!(html.contains("Destination preflight"));
    }
}
