use offload_core::offload::hash_engine::{hash_bytes, HashAlgorithm};
use std::time::Instant;

fn bench(label: &str, data: &[u8], algorithms: &[HashAlgorithm]) {
    let started = Instant::now();
    let results = hash_bytes(data, algorithms);
    let elapsed = started.elapsed().as_secs_f64();
    let gib_per_second = data.len() as f64 / elapsed / 1024.0 / 1024.0 / 1024.0;
    println!(
        "{label:20} {gib_per_second:7.2} GiB/s  {:7.2} ms  {} digest(s)",
        elapsed * 1000.0,
        results.len()
    );
}

fn main() {
    let size_mib = std::env::args()
        .nth(1)
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(512);
    let mut data = vec![0u8; size_mib * 1024 * 1024];
    for (index, byte) in data.iter_mut().enumerate() {
        *byte = (index as u64).wrapping_mul(0x9e3779b97f4a7c15) as u8;
    }
    println!("In-memory hash benchmark, {size_mib} MiB");
    bench("XXH64", &data, &[HashAlgorithm::XXH64]);
    bench("BLAKE3", &data, &[HashAlgorithm::BLAKE3]);
    bench(
        "XXH64 + BLAKE3",
        &data,
        &[HashAlgorithm::XXH64, HashAlgorithm::BLAKE3],
    );
    bench(
        "all report hashes",
        &data,
        &[
            HashAlgorithm::XXH64,
            HashAlgorithm::XXH3,
            HashAlgorithm::XXH128,
            HashAlgorithm::BLAKE3,
            HashAlgorithm::SHA256,
            HashAlgorithm::MD5,
        ],
    );
}
