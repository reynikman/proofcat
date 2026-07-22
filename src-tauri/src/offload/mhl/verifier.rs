// Ported from DIT-Pro (https://github.com/WillZ5/DIT-Pro), MIT License,
// Copyright (c) 2026 WillZ. See repository NOTICE. Adapted for Meta Report offload mode.

//! Application-level ASC MHL verification.
//!
//! Verifies both the `ascmhl_chain.xml` chain references and, unless requested
//! otherwise, recomputes media file hashes from MHL manifests.

use anyhow::{bail, Context, Result};
use quick_xml::events::Event;
use quick_xml::Reader;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use crate::offload::hash_engine::{self, HashAlgorithm, HashControl, HashEngineConfig};

use super::{ASCMHL_DIR_NAME, CHAIN_FILE_NAME};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MhlVerifyOptions {
    #[serde(default)]
    pub chain_only: bool,
    #[serde(default)]
    pub verify_all_generations: bool,
    #[serde(default)]
    pub generation: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MhlVerifyReport {
    pub summary: MhlVerifySummary,
    pub chain_results: Vec<MhlChainCheckResult>,
    pub issues: Vec<MhlVerifyIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MhlVerifySummary {
    pub path: String,
    pub mode: String,
    pub success: bool,
    pub chain_only: bool,
    pub chain_entries: usize,
    pub chain_valid: usize,
    pub chain_invalid: usize,
    pub total_files: usize,
    pub passed: usize,
    pub failed: usize,
    pub missing: usize,
    pub errors: usize,
    pub verified_generations: Vec<u32>,
    pub duration_secs: f64,
}

impl MhlVerifySummary {
    fn new(path: &Path, mode: &str, chain_only: bool) -> Self {
        Self {
            path: path.to_string_lossy().to_string(),
            mode: mode.to_string(),
            success: false,
            chain_only,
            chain_entries: 0,
            chain_valid: 0,
            chain_invalid: 0,
            total_files: 0,
            passed: 0,
            failed: 0,
            missing: 0,
            errors: 0,
            verified_generations: Vec::new(),
            duration_secs: 0.0,
        }
    }

    fn finalize(&mut self, elapsed: f64) {
        self.duration_secs = elapsed;
        self.success =
            self.chain_invalid == 0 && self.failed == 0 && self.missing == 0 && self.errors == 0;
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MhlChainCheckResult {
    pub generation: u32,
    pub manifest_path: String,
    pub expected_hash: String,
    pub actual_hash: Option<String>,
    pub valid: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MhlVerifyIssue {
    pub kind: String,
    pub message: String,
    pub generation: Option<u32>,
    pub rel_path: Option<String>,
    pub manifest_path: Option<String>,
    pub algorithm: Option<HashAlgorithm>,
    pub expected: Option<String>,
    pub actual: Option<String>,
}

#[derive(Debug, Clone)]
struct ManifestFileEntry {
    path: String,
    hashes: HashMap<HashAlgorithm, String>,
}

#[derive(Debug, Clone)]
struct ChainEntry {
    sequence_nr: u32,
    path: String,
    reference_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MhlVerifyProgress {
    pub phase: String,
    pub current_file: String,
    pub file_index: usize,
    pub total_files: usize,
}

pub struct MhlVerifyControl<'a> {
    pub cancel_flag: Option<&'a AtomicBool>,
    pub pause_flag: Option<&'a AtomicBool>,
    pub on_progress: Option<&'a (dyn Fn(MhlVerifyProgress) + Send + Sync)>,
}

impl MhlVerifyControl<'_> {
    pub fn none() -> Self {
        Self {
            cancel_flag: None,
            pause_flag: None,
            on_progress: None,
        }
    }
}

fn check_control(control: &MhlVerifyControl<'_>) -> Result<()> {
    if control
        .cancel_flag
        .is_some_and(|flag| flag.load(Ordering::SeqCst))
    {
        bail!("Verification cancelled by user");
    }
    if let Some(pause) = control.pause_flag {
        while pause.load(Ordering::SeqCst) {
            if control
                .cancel_flag
                .is_some_and(|flag| flag.load(Ordering::SeqCst))
            {
                bail!("Verification cancelled by user");
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
    }
    Ok(())
}

pub fn verify_mhl_path(path: &Path, options: MhlVerifyOptions) -> Result<MhlVerifyReport> {
    verify_mhl_path_with_control(path, options, &MhlVerifyControl::none())
}

pub fn verify_mhl_path_with_control(
    path: &Path,
    options: MhlVerifyOptions,
    control: &MhlVerifyControl<'_>,
) -> Result<MhlVerifyReport> {
    if !path.exists() {
        bail!("Path does not exist: {}", path.display());
    }

    let started = Instant::now();
    let mut report = if is_mhl_file(path) {
        verify_single_manifest(path, &options, control)?
    } else if path.is_dir() {
        verify_directory(path, &options, control)?
    } else {
        bail!(
            "Path must be a directory containing ascmhl/ or a .mhl file: {}",
            path.display()
        );
    };

    report.summary.finalize(started.elapsed().as_secs_f64());
    Ok(report)
}

fn verify_directory(
    root: &Path,
    options: &MhlVerifyOptions,
    control: &MhlVerifyControl<'_>,
) -> Result<MhlVerifyReport> {
    let ascmhl_dir = root.join(ASCMHL_DIR_NAME);
    if !ascmhl_dir.exists() {
        bail!("No ascmhl/ directory found in: {}", root.display());
    }

    let chain_path = ascmhl_dir.join(CHAIN_FILE_NAME);
    if !chain_path.exists() {
        bail!("No ascmhl_chain.xml found in: {}", ascmhl_dir.display());
    }

    let chain = parse_chain_file(&chain_path)?;
    if chain.is_empty() {
        bail!("Chain file is empty: {}", chain_path.display());
    }

    let mut summary = MhlVerifySummary::new(root, "directory", options.chain_only);
    summary.chain_entries = chain.len();
    let mut issues = Vec::new();
    let mut chain_results = Vec::new();

    for entry in &chain {
        check_control(control)?;
        let manifest_path = ascmhl_dir.join(&entry.path);
        let result = verify_chain_entry(&manifest_path, &entry.reference_hash);
        match result {
            Ok((valid, actual_hash)) => {
                if valid {
                    summary.chain_valid += 1;
                } else {
                    summary.chain_invalid += 1;
                    issues.push(MhlVerifyIssue {
                        kind: "chainMismatch".to_string(),
                        message: format!(
                            "Generation {:04} manifest hash does not match chain reference",
                            entry.sequence_nr
                        ),
                        generation: Some(entry.sequence_nr),
                        rel_path: None,
                        manifest_path: Some(manifest_path.to_string_lossy().to_string()),
                        algorithm: None,
                        expected: Some(entry.reference_hash.clone()),
                        actual: Some(actual_hash.clone()),
                    });
                }
                chain_results.push(MhlChainCheckResult {
                    generation: entry.sequence_nr,
                    manifest_path: manifest_path.to_string_lossy().to_string(),
                    expected_hash: entry.reference_hash.clone(),
                    actual_hash: Some(actual_hash),
                    valid,
                    error: None,
                });
            }
            Err(e) => {
                summary.chain_invalid += 1;
                let message = e.to_string();
                issues.push(MhlVerifyIssue {
                    kind: "chainError".to_string(),
                    message: message.clone(),
                    generation: Some(entry.sequence_nr),
                    rel_path: None,
                    manifest_path: Some(manifest_path.to_string_lossy().to_string()),
                    algorithm: None,
                    expected: Some(entry.reference_hash.clone()),
                    actual: None,
                });
                chain_results.push(MhlChainCheckResult {
                    generation: entry.sequence_nr,
                    manifest_path: manifest_path.to_string_lossy().to_string(),
                    expected_hash: entry.reference_hash.clone(),
                    actual_hash: None,
                    valid: false,
                    error: Some(message),
                });
            }
        }
    }

    if !options.chain_only {
        let generations = select_generations(&chain, options)?;
        for entry in generations {
            let manifest_path = ascmhl_dir.join(&entry.path);
            let manifest_entries = parse_manifest_file(&manifest_path)?;
            summary.verified_generations.push(entry.sequence_nr);
            verify_manifest_entries(
                root,
                &manifest_path,
                entry.sequence_nr,
                &manifest_entries,
                &mut summary,
                &mut issues,
                control,
            )?;
        }
    }

    Ok(MhlVerifyReport {
        summary,
        chain_results,
        issues,
    })
}

fn verify_single_manifest(
    manifest_path: &Path,
    options: &MhlVerifyOptions,
    control: &MhlVerifyControl<'_>,
) -> Result<MhlVerifyReport> {
    let root = manifest_path
        .parent()
        .and_then(|p| {
            if p.file_name().is_some_and(|n| n == ASCMHL_DIR_NAME) {
                p.parent()
            } else {
                Some(p)
            }
        })
        .unwrap_or_else(|| Path::new("."));

    let mut summary = MhlVerifySummary::new(manifest_path, "manifest", options.chain_only);
    let mut issues = Vec::new();

    let entries = parse_manifest_file(manifest_path)?;

    if options.chain_only {
        summary.total_files = entries.len();
    } else {
        verify_manifest_entries(
            root,
            manifest_path,
            0,
            &entries,
            &mut summary,
            &mut issues,
            control,
        )?;
    }

    Ok(MhlVerifyReport {
        summary,
        chain_results: Vec::new(),
        issues,
    })
}

fn select_generations<'a>(
    chain: &'a [ChainEntry],
    options: &MhlVerifyOptions,
) -> Result<Vec<&'a ChainEntry>> {
    if let Some(generation) = options.generation {
        let selected: Vec<&ChainEntry> = chain
            .iter()
            .filter(|entry| entry.sequence_nr == generation)
            .collect();
        if selected.is_empty() {
            bail!("Generation {} not found in MHL chain", generation);
        }
        Ok(selected)
    } else if options.verify_all_generations {
        Ok(chain.iter().collect())
    } else {
        Ok(chain.last().into_iter().collect())
    }
}

fn verify_manifest_entries(
    root: &Path,
    manifest_path: &Path,
    generation: u32,
    entries: &[ManifestFileEntry],
    summary: &mut MhlVerifySummary,
    issues: &mut Vec<MhlVerifyIssue>,
    control: &MhlVerifyControl<'_>,
) -> Result<()> {
    summary.total_files += entries.len();

    for (index, entry) in entries.iter().enumerate() {
        check_control(control)?;
        if let Some(callback) = control.on_progress {
            callback(MhlVerifyProgress {
                phase: "manualVerify".into(),
                current_file: entry.path.clone(),
                file_index: index + 1,
                total_files: entries.len(),
            });
        }
        let file_path = match resolve_manifest_file_path(root, &entry.path) {
            Ok(path) => path,
            Err(e) => {
                summary.errors += 1;
                issues.push(MhlVerifyIssue {
                    kind: "unsafePath".to_string(),
                    message: e.to_string(),
                    generation: generation_option(generation),
                    rel_path: Some(entry.path.clone()),
                    manifest_path: Some(manifest_path.to_string_lossy().to_string()),
                    algorithm: None,
                    expected: None,
                    actual: None,
                });
                continue;
            }
        };
        if !file_path.exists() {
            summary.missing += 1;
            issues.push(MhlVerifyIssue {
                kind: "missing".to_string(),
                message: format!("File not found: {}", entry.path),
                generation: generation_option(generation),
                rel_path: Some(entry.path.clone()),
                manifest_path: Some(manifest_path.to_string_lossy().to_string()),
                algorithm: None,
                expected: None,
                actual: None,
            });
            continue;
        }

        let algorithms: Vec<HashAlgorithm> = entry.hashes.keys().copied().collect();
        let config = HashEngineConfig {
            algorithms,
            buffer_size: 4 * 1024 * 1024,
        };

        let hash_control = HashControl {
            cancel_flag: control.cancel_flag,
            pause_flag: control.pause_flag,
            on_progress: None,
        };
        let hash_results =
            match hash_engine::hash_file_sync_with_control(&file_path, &config, &hash_control) {
                Ok(results) => results,
                Err(e) => {
                    summary.errors += 1;
                    issues.push(MhlVerifyIssue {
                        kind: "readError".to_string(),
                        message: format!("Could not read {}: {}", entry.path, e),
                        generation: generation_option(generation),
                        rel_path: Some(entry.path.clone()),
                        manifest_path: Some(manifest_path.to_string_lossy().to_string()),
                        algorithm: None,
                        expected: None,
                        actual: None,
                    });
                    continue;
                }
            };

        let actual: HashMap<HashAlgorithm, String> = hash_results
            .into_iter()
            .map(|result| (result.algorithm, result.hex_digest))
            .collect();

        let mut file_failed = false;
        for (algorithm, expected_hash) in &entry.hashes {
            let actual_hash = actual.get(algorithm).cloned().unwrap_or_default();
            if actual_hash.to_lowercase() != expected_hash.to_lowercase() {
                file_failed = true;
                issues.push(MhlVerifyIssue {
                    kind: "hashMismatch".to_string(),
                    message: format!("Hash mismatch for {}", entry.path),
                    generation: generation_option(generation),
                    rel_path: Some(entry.path.clone()),
                    manifest_path: Some(manifest_path.to_string_lossy().to_string()),
                    algorithm: Some(*algorithm),
                    expected: Some(expected_hash.clone()),
                    actual: Some(actual_hash),
                });
            }
        }

        if file_failed {
            summary.failed += 1;
        } else {
            summary.passed += 1;
        }
    }
    Ok(())
}

fn resolve_manifest_file_path(root: &Path, rel_path: &str) -> Result<PathBuf> {
    let path = Path::new(rel_path);
    if path.is_absolute() {
        bail!("Manifest path must be relative: {}", rel_path);
    }

    let mut safe_path = PathBuf::new();
    for component in path.components() {
        match component {
            Component::Normal(part) => safe_path.push(part),
            Component::CurDir => {}
            _ => bail!("Manifest path escapes media root: {}", rel_path),
        }
    }

    if safe_path.as_os_str().is_empty() {
        bail!("Manifest path is empty");
    }

    Ok(root.join(safe_path))
}

fn generation_option(generation: u32) -> Option<u32> {
    if generation == 0 {
        None
    } else {
        Some(generation)
    }
}

fn is_mhl_file(path: &Path) -> bool {
    path.is_file()
        && path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| ext.eq_ignore_ascii_case("mhl"))
}

fn verify_chain_entry(manifest_path: &Path, expected_hash: &str) -> Result<(bool, String)> {
    let bytes = fs::read(manifest_path)
        .with_context(|| format!("Failed to read manifest: {}", manifest_path.display()))?;
    let computed = if expected_hash.starts_with("c4") {
        super::c4::hash_bytes(&bytes)
    } else {
        // Backwards-compatible verification for legacy Meta Report chains.
        // Writers only emit standard C4 references.
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        format!("{:x}", hasher.finalize())
    };
    Ok((computed.eq_ignore_ascii_case(expected_hash), computed))
}

fn parse_chain_file(path: &Path) -> Result<Vec<ChainEntry>> {
    let content =
        fs::read(path).with_context(|| format!("Failed to read chain file: {}", path.display()))?;
    parse_chain_xml(&content)
}

fn parse_chain_xml(xml_bytes: &[u8]) -> Result<Vec<ChainEntry>> {
    let mut reader = Reader::from_reader(xml_bytes);
    reader.config_mut().trim_text(true);

    let mut entries = Vec::new();
    let mut current_entry: Option<ChainEntry> = None;
    let mut current_element = String::new();
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                match name.as_str() {
                    "hashlist" => {
                        let mut sequence_nr = 0;
                        for attr in e.attributes().flatten() {
                            if attr.key.as_ref() == b"sequencenr" {
                                sequence_nr =
                                    String::from_utf8_lossy(&attr.value).parse().unwrap_or(0);
                            }
                        }
                        current_entry = Some(ChainEntry {
                            sequence_nr,
                            path: String::new(),
                            reference_hash: String::new(),
                        });
                    }
                    "path" | "sha256" | "c4" => {
                        current_element = name;
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(ref e)) => {
                let decoded = e.decode().unwrap_or_default();
                let text = quick_xml::escape::unescape(&decoded)
                    .map(|value| value.into_owned())
                    .unwrap_or_else(|_| decoded.into_owned());
                if let Some(ref mut entry) = current_entry {
                    match current_element.as_str() {
                        "path" => entry.path.push_str(&text),
                        "sha256" | "c4" => entry.reference_hash.push_str(&text),
                        _ => {}
                    }
                }
            }
            Ok(Event::GeneralRef(ref e)) => {
                let encoded = format!("&{};", String::from_utf8_lossy(e.as_ref()));
                let text = quick_xml::escape::unescape(&encoded)
                    .map(|value| value.into_owned())
                    .unwrap_or(encoded);
                if let Some(ref mut entry) = current_entry {
                    match current_element.as_str() {
                        "path" => entry.path.push_str(&text),
                        "sha256" | "c4" => entry.reference_hash.push_str(&text),
                        _ => {}
                    }
                }
            }
            Ok(Event::End(ref e)) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                if name == "hashlist" {
                    if let Some(entry) = current_entry.take() {
                        if entry.reference_hash.is_empty() {
                            bail!("Unsupported or missing ASC MHL chain reference hash algorithm");
                        }
                        entries.push(entry);
                    }
                }
                current_element.clear();
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                bail!(
                    "Error parsing chain XML at position {}: {:?}",
                    reader.buffer_position(),
                    e
                );
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(entries)
}

fn parse_manifest_file(path: &Path) -> Result<Vec<ManifestFileEntry>> {
    let content =
        fs::read(path).with_context(|| format!("Failed to read manifest: {}", path.display()))?;
    parse_manifest_xml(&content)
}

fn parse_manifest_xml(xml_bytes: &[u8]) -> Result<Vec<ManifestFileEntry>> {
    let mut reader = Reader::from_reader(xml_bytes);
    reader.config_mut().trim_text(true);

    let mut entries = Vec::new();
    let mut buf = Vec::new();
    let mut in_hash_block = false;
    let mut in_hashes_section = false;
    let mut current_file_path = String::new();
    let mut current_hashes: HashMap<HashAlgorithm, String> = HashMap::new();
    let mut current_element = String::new();
    let mut current_algo: Option<HashAlgorithm> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                match name.as_str() {
                    "hashes" => in_hashes_section = true,
                    "hash" if in_hashes_section => {
                        in_hash_block = true;
                        current_file_path.clear();
                        current_hashes.clear();
                    }
                    "path" if in_hash_block => current_element = "path".to_string(),
                    algo_name if in_hash_block => {
                        if let Some(algo) = hash_algorithm_from_xml_name(algo_name) {
                            current_algo = Some(algo);
                            current_element = algo_name.to_string();
                        } else {
                            bail!("Unsupported MHL hash algorithm: {algo_name}");
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::Text(ref e)) => {
                let decoded = e.decode().unwrap_or_default();
                let text = quick_xml::escape::unescape(&decoded)
                    .map(|value| value.into_owned())
                    .unwrap_or_else(|_| decoded.into_owned());
                if current_element == "path" && in_hash_block {
                    current_file_path.push_str(&text);
                } else if let Some(algo) = current_algo {
                    current_hashes.entry(algo).or_default().push_str(&text);
                }
            }
            Ok(Event::GeneralRef(ref e)) => {
                let encoded = format!("&{};", String::from_utf8_lossy(e.as_ref()));
                let text = quick_xml::escape::unescape(&encoded)
                    .map(|value| value.into_owned())
                    .unwrap_or(encoded);
                if current_element == "path" && in_hash_block {
                    current_file_path.push_str(&text);
                } else if let Some(algo) = current_algo {
                    current_hashes.entry(algo).or_default().push_str(&text);
                }
            }
            Ok(Event::End(ref e)) => {
                let name = String::from_utf8_lossy(e.local_name().as_ref()).to_string();
                match name.as_str() {
                    "hash" if in_hash_block => {
                        if !current_file_path.is_empty() && !current_hashes.is_empty() {
                            entries.push(ManifestFileEntry {
                                path: current_file_path.clone(),
                                hashes: current_hashes.clone(),
                            });
                        }
                        in_hash_block = false;
                    }
                    "hashes" => in_hashes_section = false,
                    _ => {}
                }
                current_element.clear();
                current_algo = None;
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                bail!(
                    "Error parsing manifest XML at position {}: {:?}",
                    reader.buffer_position(),
                    e
                );
            }
            _ => {}
        }
        buf.clear();
    }

    Ok(entries)
}

fn hash_algorithm_from_xml_name(name: &str) -> Option<HashAlgorithm> {
    match name.to_lowercase().as_str() {
        "xxh64" => Some(HashAlgorithm::XXH64),
        "xxh3" => Some(HashAlgorithm::XXH3),
        "xxh128" => Some(HashAlgorithm::XXH128),
        "blake3" => Some(HashAlgorithm::BLAKE3),
        "sha256" | "sha-256" => Some(HashAlgorithm::SHA256),
        "md5" => Some(HashAlgorithm::MD5),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_manifest_xml(hash: &str) -> String {
        format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<hashlist version="2.0" xmlns="urn:ASC:MHL:v2.0">
  <hashes>
    <hash>
      <path size="18">Clips/test.mov</path>
      <xxh64 action="original">{}</xxh64>
    </hash>
  </hashes>
</hashlist>"#,
            hash
        )
    }

    #[test]
    fn parses_manifest_hashes() {
        let entries = parse_manifest_xml(sample_manifest_xml("abc").as_bytes()).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path, "Clips/test.mov");
        assert_eq!(entries[0].hashes.get(&HashAlgorithm::XXH64).unwrap(), "abc");
    }

    #[test]
    fn parses_manifest_paths_with_apostrophes() {
        let xml = sample_manifest_xml("abc")
            .replace("Clips/test.mov", "Transformery.Mest&apos;.Padshih.avi");
        let entries = parse_manifest_xml(xml.as_bytes()).unwrap();
        assert_eq!(entries[0].path, "Transformery.Mest'.Padshih.avi");
    }

    #[test]
    fn rejects_unknown_manifest_hash_algorithm() {
        let xml = sample_manifest_xml("abc").replace("xxh64", "futurehash");
        let error = parse_manifest_xml(xml.as_bytes()).unwrap_err();
        assert!(error
            .to_string()
            .contains("Unsupported MHL hash algorithm: futurehash"));
    }

    #[test]
    fn rejects_unknown_chain_reference_algorithm() {
        let xml = br#"<ascmhldirectory xmlns="urn:ASC:MHL:DIRECTORY:v2.0">
  <hashlist sequencenr="1"><path>0001_test.mhl</path><futurehash>abc</futurehash></hashlist>
</ascmhldirectory>"#;
        let error = parse_chain_xml(xml).unwrap_err();
        assert!(error.to_string().contains("Unsupported or missing"));
    }

    #[test]
    fn parses_chain_paths_with_apostrophes() {
        let xml = br#"<ascmhldirectory xmlns="urn:ASC:MHL:DIRECTORY:v2.0">
  <hashlist sequencenr="1"><path>0001_Day&apos;s_take.mhl</path><c4>c4deadbeef</c4></hashlist>
</ascmhldirectory>"#;
        let entries = parse_chain_xml(xml).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].path, "0001_Day's_take.mhl");
        assert_eq!(entries[0].reference_hash, "c4deadbeef");
    }

    #[test]
    fn verifies_directory_end_to_end() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let clips = root.join("Clips");
        fs::create_dir_all(&clips).unwrap();
        fs::write(clips.join("test.mov"), b"test video content").unwrap();

        let file_hash = hash_engine::hash_file_sync(
            &clips.join("test.mov"),
            &HashEngineConfig {
                algorithms: vec![HashAlgorithm::XXH64],
                buffer_size: 4 * 1024 * 1024,
            },
        )
        .unwrap()
        .remove(0)
        .hex_digest;

        let manifest_xml = sample_manifest_xml(&file_hash);
        let ascmhl_dir = root.join(ASCMHL_DIR_NAME);
        fs::create_dir_all(&ascmhl_dir).unwrap();
        let manifest_name = "0001_test_2024-01-01_000000Z.mhl";
        fs::write(ascmhl_dir.join(manifest_name), manifest_xml.as_bytes()).unwrap();

        let manifest_hash = super::super::c4::hash_bytes(manifest_xml.as_bytes());
        let chain_xml = format!(
            r#"<?xml version="1.0" encoding="UTF-8"?>
<ascmhldirectory xmlns="urn:ASC:MHL:DIRECTORY:v2.0">
  <hashlist sequencenr="1">
    <path>{}</path>
    <c4>{}</c4>
  </hashlist>
</ascmhldirectory>"#,
            manifest_name, manifest_hash
        );
        fs::write(ascmhl_dir.join(CHAIN_FILE_NAME), chain_xml).unwrap();

        let report = verify_mhl_path(root, MhlVerifyOptions::default()).unwrap();
        assert!(report.summary.success);
        assert_eq!(report.summary.chain_valid, 1);
        assert_eq!(report.summary.passed, 1);
        assert!(report.issues.is_empty());
    }

    #[test]
    fn reports_hash_mismatch() {
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        fs::write(root.join("clip.mov"), b"changed").unwrap();

        let manifest = root.join("test.mhl");
        fs::write(
            &manifest,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<hashlist version="2.0" xmlns="urn:ASC:MHL:v2.0">
  <hashes>
    <hash>
      <path>clip.mov</path>
      <xxh64>0000000000000000</xxh64>
    </hash>
  </hashes>
</hashlist>"#,
        )
        .unwrap();

        let report = verify_mhl_path(&manifest, MhlVerifyOptions::default()).unwrap();
        assert!(!report.summary.success);
        assert_eq!(report.summary.failed, 1);
        assert_eq!(report.issues[0].kind, "hashMismatch");
    }

    #[test]
    fn reports_missing_file_without_a_false_pass() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = dir.path().join("test.mhl");
        fs::write(&manifest, sample_manifest_xml("0000000000000000")).unwrap();

        let report = verify_mhl_path(&manifest, MhlVerifyOptions::default()).unwrap();

        assert!(!report.summary.success);
        assert_eq!(report.summary.passed, 0);
        assert_eq!(report.summary.missing, 1);
        assert_eq!(report.issues[0].kind, "missing");
    }

    #[test]
    fn reports_truncated_file_as_hash_mismatch() {
        let dir = tempfile::tempdir().unwrap();
        let clips = dir.path().join("Clips");
        fs::create_dir_all(&clips).unwrap();
        let complete = b"test video content";
        let expected = hash_engine::hash_bytes(complete, &[HashAlgorithm::XXH64])
            .remove(0)
            .hex_digest;
        fs::write(clips.join("test.mov"), &complete[..5]).unwrap();
        let manifest = dir.path().join("test.mhl");
        fs::write(&manifest, sample_manifest_xml(&expected)).unwrap();

        let report = verify_mhl_path(&manifest, MhlVerifyOptions::default()).unwrap();

        assert!(!report.summary.success);
        assert_eq!(report.summary.passed, 0);
        assert_eq!(report.summary.failed, 1);
        assert_eq!(report.issues[0].kind, "hashMismatch");
    }

    #[test]
    fn chain_only_single_manifest_reports_parseable_files() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = dir.path().join("test.mhl");
        fs::write(&manifest, sample_manifest_xml("abc")).unwrap();

        let report = verify_mhl_path(
            &manifest,
            MhlVerifyOptions {
                chain_only: true,
                ..Default::default()
            },
        )
        .unwrap();

        assert!(report.summary.success);
        assert_eq!(report.summary.total_files, 1);
        assert_eq!(report.summary.passed, 0);
    }

    #[test]
    fn manual_verification_honors_cancel() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = dir.path().join("test.mhl");
        fs::write(
            &manifest,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<hashlist version="2.0" xmlns="urn:ASC:MHL:v2.0">
  <hashes><hash><path>clip.mov</path><xxh64>0000000000000000</xxh64></hash></hashes>
</hashlist>"#,
        )
        .unwrap();
        fs::write(dir.path().join("clip.mov"), b"content").unwrap();
        let cancel = AtomicBool::new(true);
        let control = MhlVerifyControl {
            cancel_flag: Some(&cancel),
            pause_flag: None,
            on_progress: None,
        };

        let error = verify_mhl_path_with_control(&manifest, MhlVerifyOptions::default(), &control)
            .unwrap_err();
        assert!(error.to_string().contains("cancelled"));
    }

    #[test]
    fn rejects_manifest_paths_outside_root() {
        let dir = tempfile::tempdir().unwrap();
        let manifest = dir.path().join("test.mhl");
        fs::write(
            &manifest,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<hashlist version="2.0" xmlns="urn:ASC:MHL:v2.0">
  <hashes>
    <hash>
      <path>../outside.mov</path>
      <xxh64>0000000000000000</xxh64>
    </hash>
  </hashes>
</hashlist>"#,
        )
        .unwrap();

        let report = verify_mhl_path(&manifest, MhlVerifyOptions::default()).unwrap();

        assert!(!report.summary.success);
        assert_eq!(report.summary.errors, 1);
        assert_eq!(report.issues[0].kind, "unsafePath");
    }
}
