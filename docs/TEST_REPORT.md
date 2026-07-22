# Offload engine — hardware test report

Real-hardware verification of the ArchiveMax offload engine, beyond the in-repo
unit/fault suite. Every result below was produced on live external drives and
**independently checked** — where it matters, source and destination were hashed
by hand (not by the app) and compared, so a passing verdict is corroborated, not
trusted.

- **Date:** 2026-07-13
- **App:** ProofCat `0.3.0` CLI (`proofcat-cli`), commit on `feat/archive-max-offload`
- **Host:** MacBook, Apple Silicon (arm64)
- **Drives:**
  - Kingston SD card, `disk4`, reformatted between **exFAT** and **FAT32**
  - Netac USB SSD 2TB, `disk5`, reformatted between **APFS** and **HFS+**
- **Method:** each case run through the CLI; JSON evidence inspected; integrity
  spot-checked with an external `shasum -a 256` of the whole file set.

## Summary

**13 / 13 scenarios behaved exactly as specified.** No correctness defect found.
Physical-device detection is filesystem-agnostic (APFS, exFAT, FAT32, HFS+ all
resolve to real physical devices). Fail-closed holds: every ambiguous or damaged
condition withheld `SAFE_TO_FORMAT`.

This closes the **DIT-max offload implementation**: ArchiveMax performs the full
source pre-read → copy-read → durable commit → independent destination read-back
pipeline with XXH64 + BLAKE3 evidence, repair, resume, MHL and report export.
The parallelism policy is also complete: Fast may parallelize bounded small-file
work from an SSD, while ArchiveMax parallelizes read-back across independent
physical destinations. Source reads on SD/HDD/network/unknown devices remain
sequential deliberately; this is a safety and throughput decision, not an
unfinished feature.

## Results

| # | Scenario | Setup | Result |
|---|---|---|---|
| 1 | Cherry-pick guard | Source mixes loose files + a subfolder at root, two physical devices | `ARCHIVE_VERIFIED`, `safeToFormat:false`, warning: *"Source folder mixes loose files with subfolders … SAFE_TO_FORMAT is withheld until reviewed."* Both devices `isPhysical:true`. ✅ |
| 2 | SAFE_TO_FORMAT (APFS + exFAT) | Card-like source (`DCIM/…`), Netac **APFS** + Kingston **exFAT** | `SAFE_TO_FORMAT`, `safeToFormat:true`, 8 verified replicas, zero warnings. ✅ Confirms APFS as a real physical destination. |
| 3 | Silent corruption | Flip bytes in one destination file, then `verify` | `passed:3 failed:1`, `kind:hashMismatch`, exact `relPath` reported. ✅ |
| 4 | Missing file | Delete one destination file, then `verify` | `passed:3 failed:0 missing:1`, `kind:missing`, exact `relPath`. ✅ |
| 5 | Zero-byte source | Card contains an empty file | `ARCHIVE_VERIFIED`, `safeToFormat:false`, warning: *"Zero-byte source file requires operator review."* ✅ |
| 6 | Cross-platform unsafe name | File named `CON.mov` (Windows-reserved) | Aborted before any copy: `Error: Source path uses a Windows-reserved name: DCIM/CON.mov`. ✅ |
| 7 | Preflight free space | 16 GB source → 15.9 GB card | Refused before the first read: `Insufficient space … 15437725696 available, 16777216000 required before copy`. ✅ |
| 8 | Process kill + resume | `kill -9` mid-copy (during file 2 of 4), then `resume` same job | `copied:3 skipped:1 failed:0`, `ARCHIVE_VERIFIED`. **External set-hash of source == destination (byte-for-byte).** ✅ |
| 9 | Preserve original date | Source `mtime` set to 2020-01-01, then offload | Destination `mtime` == source `mtime` (`1577871000`). ✅ |
| 10 | Repair a damaged replica | Two destinations, corrupt one file, re-run same job | `SAFE_TO_FORMAT`, `failed:0`; the corrupted file's hash restored to match source. ✅ |
| 11 | Report export | `report` in html/csv/txt/json | All four produced (HTML 5.1 KB with verdict + clip rows, CSV 5.6 KB with full header, TXT 3.6 KB, JSON 15.6 KB). ✅ |
| 12 | Many tiny files | 1000 × 4 KB under `DCIM/` | `totalFiles:1000 copied:1000 failed:0`, `ARCHIVE_VERIFIED`; 1000 files on destination, in ~6 s. ✅ |
| 13 | Throughput | 10 GB, ArchiveMax vs Fast, to Netac USB SSD | **Fast ≈ 386 MB/s** (copy only), **ArchiveMax ≈ 310 MB/s** (copy + independent read-back + XXH64 + BLAKE3). Full verification costs only ~20%. ✅ |

## Filesystem matrix (physical-device detection)

| Filesystem | Confirmed as real physical destination |
|---|---|
| APFS | ✅ (test 2) |
| exFAT | ✅ (test 2; camera-card default) |
| FAT32 | ✅ (earlier field run, `docs/FIELD_FINDINGS.md`) |
| HFS+ | ✅ (earlier field run) |
| NTFS | ✅ real Windows destination (see Windows physical qualification below) |

The macOS table above is complemented by the Windows physical qualification
below. The only item still open is the **100 GB benchmark** (the throughput
figure above is from a 10 GB run).

## Windows physical qualification (real hardware)

Run on a real Windows x64 machine with four separate physical disks — no VHD,
virtual disk, RAM disk or single-disk partition substitution. Source is real
video (4 files, 9 369 249 792 bytes); each two-destination test uses distinct
physical disks. Physical media: Netac USB SSD (exFAT source), WDC WD10JPVX SATA
(NTFS dest), Toshiba external USB (exFAT dest), Kingston on a physical card
reader (exFAT, disconnect test).

| # | Windows physical test | Result |
|---|---|---|
| W1 | Main qualification: exFAT → NTFS + exFAT on three physical disks | Both copies confirmed, `SAFE_TO_FORMAT`. ✅ |
| W2 | Cable pulled during **copy** | Job went `failed`; after reconnecting the same disk it resumed and both copies passed `verify --all`. ✅ |
| W3 | Cable pulled during **destinationVerify** (read-back) | Corrupted file was not accepted; after reconnect `ARCHIVE_VERIFIED`, 4/4 verified, independent check `passed:4 failed:0`. ✅ |
| W4 | Disk swap before resume | A different physical disk on the same drive letter was rejected before copy: *"Resume blocked: destination volume identities do not match."* ✅ |
| W5 | Real disk almost full | 8 450 048 bytes left on NTFS dest; offload refused before any media byte, job `failed`, `total_tasks=0`; filler removed after. ✅ |
| W6 | Process kill + resume on physical G: → I: | Killed in `sourcePreRead`, `copyingData`, `destinationVerify` and `mhl`; each resume reached `ARCHIVE_VERIFIED`, `verify --all` clean, no `*.tmp-*` left. ✅ |

Windows-specific code paths (physical device number, Volume GUID, write-through)
run only on this platform and are exercised by W1–W6. The public release record
is [`release-evidence-v0.3.0.md`](release-evidence-v0.3.0.md); qualification
scripts and the complete scenario descriptions remain in this source tree.

## 24-hour physical-media soak

Separate release-qualification gate: run the full ArchiveMax pipeline in a tight
loop against a real USB SSD for 24 hours, watching for any wrong verdict, data
failure, temp-file leak, checkpoint-DB bloat or disk exhaustion.

- **Window:** 2026-07-13 19:52 → 2026-07-14 19:52 (24 h, uninterrupted).
- **Host:** MacBook, Apple Silicon (arm64); machine kept awake with `caffeinate`.
- **Source:** 500 MB fixed set (5 × 100 MB `/dev/urandom` `.mov` under `DCIM/`).
- **Destination:** Netac USB SSD 2 TB, **APFS**, `archive-max` profile
  (source pre-read → copy-read → durable commit → independent read-back verify,
  XXH64 + BLAKE3, MHL + checkpoint DB).
- **Loop:** each iteration wipes the destination, offloads the set under a fresh
  job id, asserts `"failed": 0`; every 25th iteration also samples checkpoint-DB
  size, orphan `*.tmp` count and free space. 20 s sleep between iterations.

**Result — 2228 / 2228 iterations passed, 0 failures.**

| Signal | Outcome |
|---|---|
| Verdict | `ARCHIVE_VERIFIED` on every iteration; `failed:0` throughout |
| Data integrity | No hash mismatch, no wrong verdict across 2228 runs (~1.1 TB copied+verified) |
| Temp-file leak | `orphanTmp = 0` at every checkpoint — AtomicWriter left nothing behind |
| Disk exhaustion | Free space 1 906 737 → 1 906 714 MB (23 MB drift over 24 h); no leak |
| Checkpoint DB | Linear ~31 KB/iteration (790 KB → 67 MB); expected per-job growth, no runaway bloat |
| Stability | Process ran to completion and exited cleanly; no panic, hang or restart |

The engine held integrity and resource discipline over a full day of continuous
adversarial-free load. The only monotonic growth is the append-only checkpoint DB
(one job row set per run) — bounded in real use where jobs are pruned, not
retained forever. **Soak gate: passed.**

Raw checkpoint log (25-iteration samples + clean exit):
[`assets/soak-2026-07-13-log.md`](assets/soak-2026-07-13-log.md).

## Release-provenance boundary (2026-07-20)

The behavioural evidence above (macOS matrix, soak, Windows W1–W6) was recorded
on real hardware. The exact source commits, installer SHAs and the only
remaining provenance boundary are tracked in the single release register
[`release-evidence-v0.3.0.md`](release-evidence-v0.3.0.md). Later MHL
apostrophe/entity and resume-space fixes have targeted regression coverage and
do not change device identity, disconnect, disk-swap or disk-full logic.

## Observations / hardening ideas

1. **Unsafe name aborts the whole offload.** A single Windows-reserved/illegal
   name (`CON`, trailing dot, …) stops the entire job (test 6). Safe and correct,
   but a real card with one bad clip name would fail wholesale. Consider an
   operator-confirmed *skip-and-report* (or sanitized-copy) mode so one bad name
   doesn't block an otherwise-good card. **Decision, not a bug.**
2. **CLI progress is verbose.** The stream prints one `copyingData …` line per
   chunk. A single rewriting progress line with percentage/ETA would read better
   (GUI already shows structured progress).
3. **Repair vs. re-copy.** On a re-run, a corrupted replica is fixed through the
   skip-if-verified re-hash path, so `repairAttempts` stays `0` (the in-flight
   repair counter only moves on read-back-time failures). Both paths yield correct
   data; worth a one-line doc note so the counter isn't misread.
4. **No correctness weaknesses surfaced.** Integrity, verdict gating, resume,
   preservation and reporting all held under adversarial input.

## Reproducing

All cases are CLI one-liners against two external drives; see the scenario column
for the exact trigger. Verdicts are in each run's JSON; integrity was
cross-checked with `find … -exec shasum -a 256` over the file set.
