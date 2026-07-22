# Technical documentation

This is the engineering entry point for ProofCat. The main README deliberately
keeps the operator journey simple; this page explains the safety contract,
verification design, release evidence and contribution path behind it.

## The safety contract

ProofCat has four verdicts. Only `SAFE_TO_FORMAT` permits reuse of source media.

| Verdict | Meaning |
|---|---|
| `COPY_COMPLETE` | Data reached every requested destination, but it was not independently read back. Never permission to format. |
| `ARCHIVE_VERIFIED` | The verification passes match, but an additional safety gate is missing. |
| `SAFE_TO_FORMAT` | Verification passed, destinations are distinct physical devices, ASC MHL and local evidence exist, and no warning remains. |
| `FAILED` | A required copy, verification, repair, evidence or safety gate failed. Keep the source media. |

Read the complete [offload guarantees and fault model](offload-guarantees.md)
before relying on a verdict in production.

## How ArchiveMax works

1. ProofCat fingerprints the source and destination media and records the job.
2. It reads the source before copying, then reads it again while copying.
3. It independently reads every destination back from disk.
4. It persists ASC MHL plus JSON, HTML, CSV and TXT evidence.
5. It grants `SAFE_TO_FORMAT` only after every required gate is satisfied.

The implementation is fail-closed: an interrupted, ambiguous or unproven job
must not produce a stronger verdict. The detailed adversary model and residual
risks are in the [threat model](threat-model.md).

## Evidence, tests and limits

| Subject | Reference |
|---|---|
| Real macOS and Windows hardware matrix, disconnect, disk-swap, disk-full, kill/resume and 24-hour soak | [Hardware test report](TEST_REPORT.md) |
| Fault cases covered in the automated suite | [Fault matrix](fault-matrix.md) |
| Performance measurement method | [Benchmark protocol](benchmark-protocol.md) |
| v0.3.0 artifact and physical-evidence provenance | [Release evidence](release-evidence-v0.3.0.md) |
| Third-party source and licence boundary | [Third-party notices](../THIRD_PARTY_NOTICES.md) |

ProofCat does not promise protection from malicious firmware, a compromised host,
or long-term archive health after verification. It never formats source media.

## Installation and release integrity

The GitHub release includes macOS and Windows installers, the
`SHA256SUMS-macos.txt` and `SHA256SUMS-windows.txt` manifests, and Tauri updater
signatures. Its `latest.json` feed contains the matching signatures and tells
installed applications which macOS and Windows package to download. Verify the
checksum before overriding an operating-system warning.

Tauri updater signing is not Apple Developer ID signing/notarization or Windows
Authenticode. Therefore Gatekeeper or SmartScreen may warn on first launch.
This is a distribution-trust boundary, not an offload verdict.

## Architecture and source

- [`crates/offload-core/`](../crates/offload-core/) is the Tauri-independent
  offload engine.
- [`src-tauri/`](../src-tauri/) hosts the desktop app and native tool resources.
- [`src/`](../src/) contains the offline desktop UI.
- [`scripts/`](../scripts/) contains reproducibility, qualification and native
  tool assembly scripts.

For local development and safety-critical changes, see
[CONTRIBUTING.md](../CONTRIBUTING.md). Changes to copy, checkpoint, verification,
MHL or verdict logic must add a fault test.
