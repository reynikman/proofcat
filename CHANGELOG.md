# Changelog

All notable user-facing changes are documented here. Release notes are written
in English so one technical source stays authoritative.

## [0.3.0] — 2026-07-22

- First full macOS Apple Silicon and Windows x64 release of ProofCat.
- Added ArchiveMax: independent source pre-read, destination read-back,
  fail-closed `SAFE_TO_FORMAT`, resumable checkpoints, MHL and evidence export.
- Added physical-device topology checks so two folders on one disk never count
  as independent backups.
- Added real-hardware Windows qualification and documented the 24-hour macOS
  soak, disconnect, disk-swap, disk-full and kill/resume results.
- Added signed release artifacts and SHA-256 checksum files for both platforms.

[0.3.0]: https://github.com/reynikman/proofcat/releases/tag/v0.3.0
