# ProofCat 0.3.0 — release evidence

This page records the evidence published with ProofCat 0.3.0. It is a statement
of what was checked, not a claim that software can remove every storage risk.

## Published artifacts

The GitHub Release includes:

- a macOS Apple Silicon DMG and updater archive;
- Windows x64 MSI and NSIS installers;
- Tauri updater signatures for the updater-capable artifacts; and
- a `SHA256SUMS` manifest for each platform.

Verify a release checksum before overriding an operating-system first-launch
warning. Tauri updater signing is not Apple notarization or Windows
Authenticode.

## Real-media qualification

The ArchiveMax hardware matrix was completed on physical media:

- macOS: 13 scenarios, including physical filesystem coverage, resume, repair
  and an uninterrupted 24-hour soak;
- Windows: exFAT source to independent NTFS and exFAT destinations, ending in
  `SAFE_TO_FORMAT`;
- cable disconnect during copy and read-back;
- replacement-disk rejection during resume;
- real disk-full preflight refusal; and
- process kill and resume in every documented phase.

The readable results are in the [hardware test report](TEST_REPORT.md). The
release source also contains the [fault matrix](fault-matrix.md) and
[threat model](threat-model.md), which define what the product does not claim.

## Source provenance

The application artifacts were built from the v0.3.0 release code snapshot.
This public repository deliberately uses a clean publication history rather than
exposing private development history. Its product source is the same snapshot;
the public-history import changes documentation and repository metadata, not
the application logic or packaged resources.

## Known distribution boundaries

Apple notarization and Windows Authenticode are not configured for v0.3.0.
Users may see Gatekeeper or SmartScreen on first launch. That distribution
boundary is independent of the offload verdict: only the application’s
`SAFE_TO_FORMAT` result addresses whether source media may be reused.
