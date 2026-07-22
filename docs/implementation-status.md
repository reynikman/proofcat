# Offload implementation status

This document separates implemented guarantees from release qualification.
Passing automated tests is necessary, but it does not turn virtual disks into
evidence about real cards, cables or Windows storage drivers.

## Implemented

- **MR-0:** the UI and reports distinguish `COPY_COMPLETE`,
  `ARCHIVE_VERIFIED`, `SAFE_TO_FORMAT` and `FAILED`; the public guarantee and
  fault model are documented.
- **MR-1:** `offload-core`, the `proofcat-cli` binary and the Tauri adapter
  share one Rust engine. SQLite schema v4 stores a stable job, media
  fingerprints, snapshots, tasks, observations, replicas and repairs. Resume
  continues the same job and fails closed on changed or ambiguous media.
- **MR-2:** ArchiveMax records XXH64 and BLAKE3 evidence for every independent
  read. ASC MHL contains compatible hashes only, includes the complete verified
  job after resume and is tested against the pinned official ASC implementation
  and XSDs. Unknown imported algorithms are reported as unsupported.
- **MR-3:** ArchiveMax runs source pre-read, copy-read, durable commit,
  destination readback and targeted repair. Repairs are limited to two per
  replica and use only the stable source or an independently verified replica.
  Pause, cancel and process death preserve resumable state.
- **MR-4:** manual MHL/job verification runs outside the UI thread with progress,
  pause and cancel. Canonical JSON and derived HTML, RFC 4180 CSV and TXT reports
  contain application identity, media fingerprints, per-file byte counts,
  file-by-destination hashes, destination free-space preflight, optional DIT
  contacts, MHL paths, failures and repair history. The copy path preserves the
  source modification time after durable rename and sync.
- **MR-5:** the scheduler groups paths by physical device, serializes SD/HDD and
  unknown devices, bounds SSD work and memory, and parallelizes readback only
  across independent destinations. Benchmarks enforce the published regression
  and dual-hash thresholds rather than promising unmeasured speed.
- **MR-6:** project code is MIT licensed. DIT-Pro attribution, native-tool
  licenses, notices, threat model, security policy, SBOM, LGPL-only FFmpeg build
  recipes, corresponding-source fetches, secret scanning, macOS/Windows CI,
  signed checksums and provenance workflows are checked in.
- **v0.1 operator tail:** individual-file selection is not accepted; a source
  mixing loose root files and folders is flagged for operator review. Local
  completion notification uses the operating system's permission prompt.
  Auto-eject is default-off and, when
  explicitly requested, currently runs only through macOS `diskutil` after all
  evidence has been written.

## Automated qualification

The following gates are reproducible from the repository:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --all-targets
npm run check && npm run lint
cargo audit
cargo deny check licenses sources
scripts/check-bundled-tools.sh
scripts/process-kill-resume.sh
scripts/macos-disk-full.sh
scripts/macos-filesystem-matrix.sh
```

CI additionally cross-checks generated MHL through the pinned official ASC MHL
reference. The same check must remain green after any MHL/XML change.

## Release qualification (recorded for v0.3.0)

The DIT-max offload implementation is complete. The physical evidence below was
recorded on real hardware for the v0.3.0 release; see
[`TEST_REPORT.md`](TEST_REPORT.md) and the provenance register
[`release-evidence-v0.3.0.md`](release-evidence-v0.3.0.md).

1. ✅ physical macOS exFAT source to independent APFS and exFAT destinations;
2. ✅ physical Windows exFAT source to independent NTFS and exFAT destinations;
3. ✅ cable disconnect/reconnect during copy and readback;
4. ✅ wrong physical disk at the same mount point or drive letter during resume;
5. ✅ real-device disk-full and a full 86,400-second physical-media soak;
6. ⬜ 100 GiB physical benchmarks (known follow-up);
7. ◑ macOS and Windows installers are built and Tauri updater-signed; Apple
   notarization and Windows Authenticode are not done and are optional for an
   OSS release.

Disk images and shortened soak runs are diagnostic only; they were never
recorded as satisfying these gates. Commit-alignment detail for each run is in
the provenance register.
