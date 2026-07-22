#!/usr/bin/env bash
set -euo pipefail

[[ "$(uname -s)" == "Darwin" ]] || { echo "macOS is required" >&2; exit 1; }
root=$(cd "$(dirname "$0")/.." && pwd)
binary=${META_REPORT_CLI:-"$root/target/debug/proofcat-cli"}
work_root=${META_REPORT_FS_WORK_DIR:-"$root/.build/macos-filesystem-matrix"}
suffix=$$
source_name="MRS${suffix: -5}"
apfs_name="MRA${suffix: -5}"
exfat_name="MRE${suffix: -5}"
source_mount="/Volumes/$source_name"
apfs_mount="/Volumes/$apfs_name"
exfat_mount="/Volumes/$exfat_name"

cleanup() {
  for mount in "$source_mount" "$apfs_mount" "$exfat_mount"; do
    if mount | grep -Fq " on $mount "; then
      hdiutil detach "$mount" -force >/dev/null 2>&1 || true
    fi
  done
}
trap cleanup EXIT INT TERM

if [[ ! -x "$binary" ]]; then
  cargo build -p proofcat --bin proofcat-cli --manifest-path "$root/Cargo.toml"
fi
rm -rf "$work_root"
mkdir -p "$work_root"

hdiutil create -quiet -size 256m -fs ExFAT -volname "$source_name" \
  "$work_root/source.dmg"
hdiutil create -quiet -size 256m -fs APFS -volname "$apfs_name" \
  "$work_root/apfs.dmg"
hdiutil create -quiet -size 256m -fs ExFAT -volname "$exfat_name" \
  "$work_root/exfat.dmg"
hdiutil attach -quiet -nobrowse -mountpoint "$source_mount" "$work_root/source.dmg"
hdiutil attach -quiet -nobrowse -mountpoint "$apfs_mount" "$work_root/apfs.dmg"
hdiutil attach -quiet -nobrowse -mountpoint "$exfat_mount" "$work_root/exfat.dmg"

mkdir -p "$source_mount/A001"
dd if=/dev/zero of="$source_mount/A001/clip.mov" bs=1048576 count=32 status=none
printf 'filesystem matrix\n' > "$source_mount/A001/clip.txt"

job="job-macos-filesystem-matrix-$suffix"
db="$work_root/offload.sqlite"
"$binary" offload --source "$source_mount" --dest "$apfs_mount" \
  --dest "$exfat_mount" --profile archive-max --db "$db" --job "$job" \
  > "$work_root/offload.json" 2>&1
"$binary" verify "$apfs_mount" --all > "$work_root/apfs-verify.json"
"$binary" verify "$exfat_mount" --all > "$work_root/exfat-verify.json"
"$binary" report --job "$job" --db "$db" --format json \
  --output "$work_root/evidence.json" >/dev/null

python3 - "$work_root/evidence.json" <<'PY'
import json
import pathlib
import sys

evidence = json.loads(pathlib.Path(sys.argv[1]).read_text())
assert evidence["verdict"] == "ARCHIVE_VERIFIED", evidence["verdict"]
assert evidence["safeToFormat"] is False
assert evidence["verifiedReplicas"] == evidence["totalFiles"] * 2
assert all(not volume["isPhysical"] for volume in evidence["destinationVolumes"])
assert all(row["status"] == "verified" for row in evidence["replicas"])
PY

diskutil info "$source_mount" > "$work_root/source-diskutil.txt"
diskutil info "$apfs_mount" > "$work_root/apfs-diskutil.txt"
diskutil info "$exfat_mount" > "$work_root/exfat-diskutil.txt"
echo "Virtual macOS exFAT -> APFS + exFAT matrix passed: $work_root"
