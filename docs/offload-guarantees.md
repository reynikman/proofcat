# Offload guarantees and fault model

## Verdicts

### `COPY_COMPLETE`

Every destination accepted the complete byte count through an atomic temporary
file, the file data was synced, and the temporary name was atomically replaced.
The destination was not independently read back. This verdict never authorizes
formatting the source.

### `ARCHIVE_VERIFIED`

For every file, the XXH64 and BLAKE3 results of these independent passes match:

1. source pre-read;
2. source copy-read;
3. destination readback for every selected destination.

All selected replicas passed, but there are fewer than two independent physical
destinations, MHL was disabled/unavailable, or an operator warning remains.

### `SAFE_TO_FORMAT`

`ARCHIVE_VERIFIED` plus at least two distinct, platform-confirmed physical
destination devices, a persisted
ASC MHL on every destination, a local evidence snapshot and no unresolved
warning. This is the only verdict that may tell the operator the card can be
formatted.

### `FAILED`

At least one required file/replica, durable write, independent readback, repair
or MHL write failed. Source media must be retained.

## Automatic repair

When destination readback differs from source evidence, only that replica is
rewritten. At most two repair attempts are made. A repair is accepted only when
the repair copy-read and the subsequent destination readback both equal the
original source pre-read. Healthy replicas are never overwritten.

## Crash recovery

Jobs have a stable ID before copying begins. SQLite WAL stores the request and
per-replica state before the scan starts. Resume reuses the same job and
deterministic task IDs, validates media fingerprints rather than mount names,
removes orphaned temporary files and independently verifies reused files.

Hash observations, replica states and every repair attempt are journaled in
normalized tables. The two-repair limit applies to the whole job and is not
reset by restarting the process.

## IO scheduling

Source, copy and destination readback operations acquire a shared queue for
their physical device identity. Unknown, SD, HDD and network sources stay
serial. Readbacks on independent destinations may run concurrently. Fast-mode
small-file copy is configurable from 1 to 8 workers only for an SSD source;
per-device defaults cap SSD/NVMe at four and the global memory budget is 512 MiB.

## Residual risks

- No desktop application can guarantee that a removable drive controller has
  honest non-volatile cache semantics.
- Two folders on one physical volume are not independent backups.
- BLAKE3 is evidence-only because it is not an ASC MHL interoperability hash.
- Some removable readers expose no hardware serial. Resume additionally binds
  the volume UUID/GUID, filesystem serial and recorded source tree; cloned media
  identifiers remain a residual platform limitation.
- A 0-byte source file is preserved but leaves an operator warning, preventing
  `SAFE_TO_FORMAT` until reviewed.
