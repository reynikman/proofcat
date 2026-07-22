# Release engineering

This document describes the public release process for ProofCat.

## Release requirements

A release must have:

1. a clean, committed source snapshot;
2. passing Rust tests, clippy, frontend checks and lint;
3. checksum-verified native media tools;
4. platform installer artifacts, Tauri updater signatures and SHA-256 manifests;
5. a release record that links the artifact and hardware-test evidence.

The hardware qualification and artifact provenance are related but separate:
a passed physical-media test proves behaviour on the recorded devices; an installer
checksum proves the file that was published. Record both facts without claiming
that one automatically proves the other.

## v0.3.0 publication (2026-07-22)

ProofCat 0.3.0 is published at
[`reynikman/proofcat`](https://github.com/reynikman/proofcat/releases/tag/v0.3.0).
The release contains nine assets: macOS Apple Silicon DMG and updater archive,
Windows x64 MSI and NSIS installers, their updater signatures, and separate
`SHA256SUMS-macos.txt` and `SHA256SUMS-windows.txt` manifests. GitHub's
SHA-256 digests were checked against the local release artifacts after upload.

Tauri updater signatures are present. They are not Apple notarization or
Windows Authenticode; state that distinction plainly to users.

## Publish a GitHub release

1. Build the macOS Apple Silicon and Windows x64 installer sets from the intended
   source snapshot.
2. Create SHA-256 manifests and Tauri updater signatures for the exact files.
3. Create the annotated version tag and attach the installer files, signatures
   and manifest to the GitHub Release.
4. Publish English release notes and link to
   [technical documentation](docs/TECHNICAL.md),
   [hardware test report](docs/TEST_REPORT.md) and
   [release evidence](docs/release-evidence-v0.3.0.md).
5. Never put signing keys, client media, client paths, private infrastructure
   details or unreleased artifacts in Git.

Tauri updater signatures are distinct from Apple notarization and Windows
Authenticode. A release must state that distinction plainly.
