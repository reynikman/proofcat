# Threat model

This document describes what the offload pipeline protects, which failures it
detects, and which claims it deliberately does not make. It applies to the
`ArchiveMax` profile. `Fast` provides durable copying but no independent
destination verification.

## Protected assets

- the source media, which the offload workflow never modifies;
- every required destination replica;
- the SQLite job journal and immutable evidence snapshot;
- ASC MHL manifests and their generation chain;
- the truthfulness of `COPY_COMPLETE`, `ARCHIVE_VERIFIED`, `SAFE_TO_FORMAT` and
  `FAILED` verdicts.

The primary safety property is fail-closed: an interrupted, ambiguous or
unproven operation must not produce `SAFE_TO_FORMAT`.

## Trust boundaries

The application trusts the operating system to return bytes from storage and
to implement the documented flush primitives. It does not trust a path, volume
label, file name, existing destination file, temporary file or previous job
state by itself. They are rebound to recorded file and volume fingerprints and
then independently hashed.

The local operator and OS account are trusted not to intentionally replace the
running executable or rewrite the checkpoint database while the job is active.
Release signatures and provenance protect distribution, not a compromised host.

## Faults and adversaries in scope

- accidental bit flips, truncation, missing and zero-byte files;
- source mutation or replacement between scan, pre-read and copy-read;
- a destination returning different bytes after a successful write;
- disk-full, read/write/flush/rename errors and removable-media loss;
- process termination, power-loss-like interruption and resume;
- stale or misleading existing files at a destination;
- two destination paths resolving to the same physical device;
- case collisions, Unicode normalization collisions, unsafe Windows names,
  traversal and symlink escape;
- MHL content, generation-chain and algorithm tampering;
- a replaced source or destination mounted under a familiar path during resume;
- corrupted or incompatible SQLite checkpoint data.

These faults may be accidental or caused by an attacker who can alter media
between passes. Hash equality proves byte identity between observed passes; it
does not prove that the original camera bytes were semantically correct.

## Mitigations

- `ArchiveMax` reads the source before copying, reads it again while copying,
  and independently reads every destination.
- XXH64 and BLAKE3 must both match across all required observations. BLAKE3 is
  internal evidence; ASC MHL contains only standard-compatible hashes.
- files are written to unique temporary names, flushed, renamed atomically and
  followed by a parent-directory flush. A flush error blocks strong verdicts.
- repair affects only a failed replica, is limited to two attempts and uses
  only the stable source or another independently verified replica.
- SQLite migrations are transactional. Job, event, file, volume, hash, retry
  and repair records retain the same job identity across resume.
- `SAFE_TO_FORMAT` additionally requires at least two distinct physical
  destination identities, persisted evidence and MHL, and no unresolved fault
  or warning.

## Out of scope and residual risk

- malicious firmware, a controller that lies about durable flush, faulty RAM,
  kernel compromise and hash implementation compromise;
- authenticity, ownership or creative correctness of the source content;
- protection against a privileged attacker who can alter the executable,
  database and media together after all observations;
- long-term archival health after the recorded verification time;
- cloud/object storage and remote transport in the beta release;
- automatic source formatting. ProofCat only emits a verdict; destructive
  formatting remains a separate operator action.

Device topology is platform-derived. Unknown, virtual, network or ambiguous
devices are never sufficient evidence of two independent physical copies for
`SAFE_TO_FORMAT`. Real release qualification therefore includes cable removal,
reconnect, process-kill and filesystem tests on declared physical hardware.

## Evidence and disclosure

Canonical job evidence is JSON rendered from an immutable SQLite snapshot.
HTML, CSV and TXT are views of that snapshot. Reports may reveal file names,
paths, device identities and timing, so operators must treat them as production
metadata and redact them before public disclosure.

Security reports follow [`SECURITY.md`](../SECURITY.md). The executable does
not claim a stronger guarantee than the tests recorded in
[`fault-matrix.md`](fault-matrix.md) and
[`benchmark-protocol.md`](benchmark-protocol.md).
