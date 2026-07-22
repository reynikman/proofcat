#!/usr/bin/env bash
set -euo pipefail

[[ "$(uname -s)" == "Darwin" ]] || { echo "macOS is required" >&2; exit 1; }
root=$(cd "$(dirname "$0")/.." && pwd)
source_path=${1:-}
apfs_destination=${2:-}
exfat_destination=${3:-}
record_dir=${4:-"$root/.build/physical-macos"}
binary=${META_REPORT_CLI:-"$root/target/release/proofcat-cli"}

for path in "$source_path" "$apfs_destination" "$exfat_destination"; do
  [[ -d "$path" ]] || { echo "Physical qualification path is missing: $path" >&2; exit 2; }
done
python3 - "$source_path" "$apfs_destination" "$exfat_destination" <<'PY'
import plistlib
import subprocess
import sys

paths = sys.argv[1:]
infos = [plistlib.loads(subprocess.check_output(["diskutil", "info", "-plist", p])) for p in paths]
filesystems = [str(info.get("FilesystemType", "")).lower() for info in infos]
assert filesystems[0] == "exfat", f"source must be exFAT, got {filesystems[0]}"
assert filesystems[1] == "apfs", f"destination 1 must be APFS, got {filesystems[1]}"
assert filesystems[2] == "exfat", f"destination 2 must be exFAT, got {filesystems[2]}"
whole = [info.get("ParentWholeDisk") or info.get("DeviceIdentifier") for info in infos]
assert len(set(whole)) == 3, f"source and destinations must use three physical devices: {whole}"
assert all(info.get("SystemImage", 0) == 0 for info in infos), "disk images are not physical evidence"
PY

[[ -x "$binary" ]] || cargo build --release -p proofcat --bin proofcat-cli \
  --manifest-path "$root/Cargo.toml"
rm -rf "$record_dir"
mkdir -p "$record_dir"
job="job-physical-macos-$(date -u +%Y%m%dT%H%M%SZ)"
db="$record_dir/offload.sqlite"
"$binary" offload --source "$source_path" --dest "$apfs_destination" \
  --dest "$exfat_destination" --profile archive-max --db "$db" --job "$job" \
  > "$record_dir/offload.json" 2> "$record_dir/progress.log"
"$binary" verify "$apfs_destination" --all > "$record_dir/apfs-verify.json"
"$binary" verify "$exfat_destination" --all > "$record_dir/exfat-verify.json"
"$binary" report --job "$job" --db "$db" --format json \
  --output "$record_dir/evidence.json" >/dev/null
for pair in "source:$source_path" "apfs:$apfs_destination" "exfat:$exfat_destination"; do
  name=${pair%%:*}
  path=${pair#*:}
  diskutil info -plist "$path" > "$record_dir/$name-diskutil.plist"
done
python3 - "$record_dir/evidence.json" "$record_dir/qualification.json" <<'PY'
import datetime
import json
import pathlib
import platform
import subprocess
import sys

evidence = json.loads(pathlib.Path(sys.argv[1]).read_text())
assert evidence["verdict"] == "SAFE_TO_FORMAT", evidence["verdict"]
assert evidence["safeToFormat"] is True
assert all(volume["isPhysical"] for volume in evidence["destinationVolumes"])
record = {
    "schemaVersion": 1,
    "kind": "physical-media",
    "platform": platform.platform(),
    "commit": subprocess.check_output(["git", "rev-parse", "HEAD"], text=True).strip(),
    "completedAt": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "jobId": evidence["jobId"],
    "verdict": evidence["verdict"],
}
pathlib.Path(sys.argv[2]).write_text(json.dumps(record, indent=2, sort_keys=True) + "\n")
PY
echo "Physical macOS qualification passed: $record_dir"
