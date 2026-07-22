#!/usr/bin/env bash
set -euo pipefail

[[ "$(uname -s)" == "Darwin" ]] || { echo "macOS is required" >&2; exit 1; }
root=$(cd "$(dirname "$0")/.." && pwd)
binary=${META_REPORT_CLI:-"$root/target/debug/proofcat-cli"}
work=${META_REPORT_DISK_FULL_WORK_DIR:-"$root/.build/macos-disk-full"}
suffix=$$
name="MRF${suffix: -5}"
mount_point="/Volumes/$name"
cleanup() {
  if mount | grep -Fq " on $mount_point "; then
    hdiutil detach "$mount_point" -force >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT INT TERM

[[ -x "$binary" ]] || cargo build -p proofcat --bin proofcat-cli \
  --manifest-path "$root/Cargo.toml"
rm -rf "$work"
mkdir -p "$work/source"
dd if=/dev/zero of="$work/source/too-large.mov" bs=1048576 count=48 status=none
hdiutil create -quiet -size 32m -fs ExFAT -volname "$name" "$work/full.dmg"
hdiutil attach -quiet -nobrowse -mountpoint "$mount_point" "$work/full.dmg"
job="job-disk-full-${$}"
db="$work/offload.sqlite"
if "$binary" offload --source "$work/source" --dest "$mount_point" \
  --profile archive-max --db "$db" --job "$job" > "$work/offload.log" 2>&1; then
  echo "Disk-full preflight unexpectedly succeeded" >&2
  exit 1
fi
"$binary" job --job "$job" --db "$db" > "$work/job.json"
python3 - "$work/job.json" "$mount_point" <<'PY'
import json
import pathlib
import sys

job = json.loads(pathlib.Path(sys.argv[1]).read_text())
assert job["state"] == "failed", job["state"]
assert job["summary"] is None
mount = pathlib.Path(sys.argv[2])
assert not (mount / "too-large.mov").exists()
PY
echo "Real filesystem ENOSPC preflight passed: $work"
