# Product roadmap

ProofCat is an offline, cross-platform DIT workstation. Its public claims
are limited to guarantees demonstrated by the fault suite, ASC MHL conformance
tests and published benchmark protocol.

## Product direction

1. Preserve a complete camera-card tree and its original timestamps.
2. Produce durable, independently verified replicas and evidence that can be
   inspected without the desktop UI.
3. Make every offload job easy to revisit and support: searchable job history,
   readable event details and an evidence-oriented support bundle.
4. Add useful set metadata: thumbnails, camera/reel/timecode, operator notes,
   iXML/BWF sound reports and camera-to-sound sync checks.
5. Export open post-production handoff formats such as ALE, FCPXML, CSV and CDL.
6. Keep media, metadata and crash reporting local/offline by default.

## Implemented baseline (audited 2026-07-15)

These are already part of the transport core and are not new v0.4 scope:

- source volume identity, physical-volume topology and the `SAFE_TO_FORMAT`
  physical-volume gate;
- source snapshot/fingerprint binding, recursive source scan and destination
  free-space preflight;
- ArchiveMax source pre-read, per-destination independent readback, repair,
  checkpointed resume and fail-closed verdicts;
- ASC MHL generation/chain writing, verify-all-generations and the existing UI
  action to re-verify an archive by MHL;
- checkpoint jobs/tasks/events, JSON/HTML/CSV/TXT evidence and canonical
  destination evidence files.

The remaining gap is the operator-facing history/catalog and comparison layer:
the backend can load one job, but the UI does not yet provide a searchable list
of all jobs, cross-job archive comparison or restore planning.

## Release order

- `v0.3.0`: ArchiveMax, real resume, repair, evidence reports, ASC MHL, and
  the completed macOS and Windows fault/physical-media matrix.
- Next (`v0.4`): job history with a job-centric detail view, incremental
  archive, MHL history/diff, operator/project notes, thumbnail contact sheets
  and original timestamp preservation. Add a portable support bundle containing
  canonical evidence/report data, job events, tool versions and redacted
  diagnostics; it must never include source media by default. Add a
  non-destructive **Source Health & Readiness** card (see below).
- Later: BWF/iXML validation, camera presets, metadata handoff and measured
  tuning of the existing bounded SSD-only small-file concurrency.

## Competitive guardrails

The public Video Commander roadmap was reviewed for reusable product patterns.
We adopt the job-centric history and supportability pattern, but implement it
around ProofCat's checkpoint, MHL and evidence contract rather than copying
their UI or private code.

- Canary releases may be used later for non-safety UI and updater changes only.
  A canary must never weaken or ambiguously expose the `SAFE_TO_FORMAT` gate.
- Remote URLs, S3/browser sources, cloud workflows, transcoding, VMAF and
  HLS/DASH packaging stay outside the near-term roadmap. They solve a different
  streaming-engineering problem and conflict with ProofCat's offline-first
  evidence boundary.
- Automatic FFmpeg installation is not a roadmap item. Native tools remain
  pinned, license-checked, checksum-pinned and provenance-attested.
- Deep MP4 box/sample inspection is deferred until a concrete DIT workflow
  proves the need; it is not allowed to displace delivery proof or archive
  safety work.

## Source Health & Readiness (v0.4, non-destructive)

This is a readiness report, not a promise that a removable flash device will
last. It should show, when the OS exposes the data:

- device identity/topology, filesystem, mount/read-only state and total/used/
  free capacity;
- filesystem/mount sanity checks that are safe on the current platform;
- best-effort SMART/health telemetry for HDD/SSD/NVMe (model, temperature,
  power-on and critical attributes), with `Unavailable` treated as neutral for
  USB flash cards and readers that do not pass SMART through;
- the already implemented source scan and ArchiveMax pre-read result as the
  actual evidence that this particular source was readable for this offload.

The health result is evidence and a warning surface. It must never turn a green
SMART result into `SAFE_TO_FORMAT`, and it must never block a copy solely
because SMART is unavailable. A later version may expose an explicit advanced
flash-capacity/authenticity test, but it must be clearly destructive and never
run as part of normal offload.

Performance numbers are not product promises until reproduced by the checked-in
benchmark harness on the published hardware/media matrix.
