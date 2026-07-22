use offload_core::offload::hash_engine::{hash_file_sync, HashAlgorithm, HashEngineConfig};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::time::Instant;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Measurement {
    label: &'static str,
    algorithms: Vec<HashAlgorithm>,
    seconds: f64,
    gib_per_second: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Report {
    schema_version: u32,
    path: String,
    bytes: u64,
    iterations: usize,
    measurements: Vec<Measurement>,
    dual_hash_overhead_percent: f64,
}

fn measure(
    path: &Path,
    bytes: u64,
    iterations: usize,
    label: &'static str,
    algorithms: Vec<HashAlgorithm>,
) -> anyhow::Result<Measurement> {
    let started = Instant::now();
    for _ in 0..iterations {
        hash_file_sync(
            path,
            &HashEngineConfig {
                algorithms: algorithms.clone(),
                buffer_size: 4 * 1024 * 1024,
            },
        )?;
    }
    let seconds = started.elapsed().as_secs_f64() / iterations as f64;
    Ok(Measurement {
        label,
        algorithms,
        seconds,
        gib_per_second: bytes as f64 / seconds / 1024.0 / 1024.0 / 1024.0,
    })
}

fn main() -> anyhow::Result<()> {
    let path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .ok_or_else(|| anyhow::anyhow!("usage: storage-hash-bench FILE [ITERATIONS]"))?;
    let iterations = std::env::args()
        .nth(2)
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(3)
        .max(1);
    let bytes = std::fs::metadata(&path)?.len();
    if bytes == 0 {
        anyhow::bail!("benchmark file must not be empty");
    }
    let xxh64 = measure(
        &path,
        bytes,
        iterations,
        "xxh64",
        vec![HashAlgorithm::XXH64],
    )?;
    let dual = measure(
        &path,
        bytes,
        iterations,
        "xxh64+blake3",
        vec![HashAlgorithm::XXH64, HashAlgorithm::BLAKE3],
    )?;
    let overhead = (dual.seconds / xxh64.seconds - 1.0) * 100.0;
    println!(
        "{}",
        serde_json::to_string_pretty(&Report {
            schema_version: 1,
            path: path.display().to_string(),
            bytes,
            iterations,
            measurements: vec![xxh64, dual],
            dual_hash_overhead_percent: overhead,
        })?
    );
    Ok(())
}
