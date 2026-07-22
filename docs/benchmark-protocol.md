# Benchmark protocol

Performance claims are release evidence, not constants in UI copy.

## CPU hash envelope

Run an optimized in-memory pass to prove that hashing itself is not slower than
the target storage:

```bash
cargo run --release -p offload-core --example hash-bench -- 256
```

Reference development result (2026-07-12, MacBookPro18,3, Apple M1 Pro, 32 GiB,
macOS 15.7.5): XXH64 10.75 GiB/s, BLAKE3 13.49 GiB/s and combined XXH64 +
BLAKE3 6.38 GiB/s. This is not a storage-speed claim.

The storage pass is machine-readable:

```bash
BASELINE_META_REPORT_CLI=/Applications/MetaReport-previous/meta-report-cli \
  scripts/benchmark-release-gates.sh \
  /Volumes/CARD/BENCH_TREE /Volumes/EMPTY_BENCH_DEST sd-hdd
```

It exits non-zero when the dual-hash threshold fails or when a previous-release
`BASELINE_META_REPORT_CLI` was not supplied for the Fast regression gate. A run
against a sparse file, disk image or warm OS cache is diagnostic only and may
not be attached as physical release evidence.

## Physical-media release run

For every release candidate, create a fixed random tree with at least 100 GiB,
including one 50+ GiB file and 10,000 small files. Run each case three times and
publish median wall time, bytes, source/destination model, filesystem, connection,
OS, app commit, profile and maximum memory:

1. current `Fast` versus the previous release `Fast`;
2. ArchiveMax XXH64-only passes versus ArchiveMax XXH64+BLAKE3 passes;
3. one destination versus two independent destinations;
4. destination readback serial versus cross-device parallel mode, when enabled.

Required physical combinations are exFAT source to APFS+exFAT on macOS and
exFAT source to NTFS+exFAT on Windows. Do not reuse OS-cache-only numbers as
media throughput.

Release gates: Fast regression no more than 10%; dual-hash overhead no more than
5% on SD/HDD and 15% on SSD for otherwise identical passes. A failed gate blocks
the performance claim and defaults concurrency back to one.
