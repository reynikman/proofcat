# Physical-media qualification

Virtual filesystems prove path, MHL and filesystem semantics but never count as
independent physical copies. The normal qualification record contains all of
these separate runs:

1. macOS: exFAT source -> APFS + exFAT destinations using three physical
   devices (`scripts/qualify-physical-macos.sh`);
2. Windows: exFAT source -> NTFS + exFAT destinations using three physical
   devices (`scripts/qualify-physical-windows.ps1`);
3. cable disconnect during copy and destination readback, followed by reconnect
   and resume of the same job;
4. replacement by a different disk at the same mount/drive letter, which must
   be rejected before copying;
5. an actual disk-full event and process kill in every reported phase;
6. `scripts/soak-offload.sh` with its default 86,400-second duration. For
   release evidence set `SOAK_SOURCE_DIR` to the physical exFAT source,
   `SOAK_DESTINATIONS` to the colon-separated APFS/exFAT destination roots and
   `EXPECT_SAFE_TO_FORMAT=1`; the script writes and removes only its uniquely
   named per-iteration subdirectories;
7. `scripts/benchmark-release-gates.sh` against at least 100 GiB and the
   previous beta CLI.

The record directory contains canonical evidence JSON, verification JSON,
device topology, exact commit, timings and logs. Results from disk images,
sparse files, cached reads or shortened soak runs must be labelled diagnostic
and cannot satisfy a release gate.

Artifact provenance and physical qualification are separate records. A clean
rebuild or an updater-key rotation requires fresh installer checksums and Tauri
signatures for the new commit, but it does not retroactively convert an earlier
physical run into evidence for that commit. Conversely, a successful physical
run does not prove that a later installer was built, signed or packaged with the
same source. Record both exact SHAs and the artifact checksums. The release
owner may publish with an explicitly documented provenance boundary; in that
case the record must never relabel the passed physical run as pending or claim
it ran on the later commit.

No physical qualification script formats a device. Preparing or erasing test
media is deliberately outside the application and requires an explicit operator
action.
