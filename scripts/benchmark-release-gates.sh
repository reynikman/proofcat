#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
source_dir=${1:-}
destination_root=${2:-}
device_class=${3:-}
output=${4:-"$root/.build/benchmark/release-gates.json"}
baseline=${BASELINE_META_REPORT_CLI:-}

if [[ ! -d "$source_dir" || ! -d "$destination_root" ]]; then
  echo "usage: $0 SOURCE_DIR EMPTY_DESTINATION_ROOT sd-hdd|ssd [OUTPUT_JSON]" >&2
  exit 2
fi
if [[ -n "$(find "$destination_root" -mindepth 1 -maxdepth 1 -print -quit)" ]]; then
  echo "Destination root must be empty; benchmark creates and removes test paths" >&2
  exit 2
fi
if [[ -z "$baseline" || ! -x "$baseline" ]]; then
  echo "BASELINE_META_REPORT_CLI must point to the previous release binary" >&2
  exit 3
fi
case "$device_class" in
  sd-hdd) dual_threshold=5 ;;
  ssd) dual_threshold=15 ;;
  *) echo "device class must be sd-hdd or ssd" >&2; exit 2 ;;
esac

candidate=${CANDIDATE_META_REPORT_CLI:-"$root/target/release/proofcat-cli"}
if [[ ! -x "$candidate" ]]; then
  cargo build --release -p proofcat --bin proofcat-cli \
    --manifest-path "$root/Cargo.toml"
fi
hash_file=${HASH_BENCH_FILE:-$(find "$source_dir" -type f -print0 | \
  xargs -0 stat -f '%z %N' | sort -nr | head -1 | cut -d' ' -f2-)}
if [[ -z "$hash_file" || ! -f "$hash_file" ]]; then
  echo "SOURCE_DIR contains no benchmark file" >&2
  exit 2
fi

mkdir -p "$(dirname "$output")"
hash_output="${output%.json}-hash.json"
cargo run --release -q -p offload-core --example storage-hash-bench \
  --manifest-path "$root/Cargo.toml" -- "$hash_file" 3 > "$hash_output"

python3 - "$baseline" "$candidate" "$source_dir" "$destination_root" \
  "$hash_output" "$output" "$dual_threshold" <<'PY'
import json
import pathlib
import shutil
import statistics
import subprocess
import sys
import time

baseline, candidate, source, dest_root, hash_path, output, dual_limit = sys.argv[1:]
dual_limit = float(dual_limit)

def runs(label, binary):
    timings = []
    for index in range(3):
        destination = pathlib.Path(dest_root) / f"{label}-{index}"
        database = pathlib.Path(dest_root) / f"{label}-{index}.sqlite"
        shutil.rmtree(destination, ignore_errors=True)
        destination.mkdir()
        if database.exists():
            database.unlink()
        started = time.monotonic()
        subprocess.run(
            [binary, "offload", "--source", source, "--dest", str(destination),
             "--profile", "fast", "--db", str(database), "--no-mhl"],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            check=True,
        )
        timings.append(time.monotonic() - started)
    return timings

baseline_times = runs("baseline", baseline)
candidate_times = runs("candidate", candidate)
baseline_median = statistics.median(baseline_times)
candidate_median = statistics.median(candidate_times)
fast_regression = (candidate_median / baseline_median - 1.0) * 100.0
hash_report = json.loads(pathlib.Path(hash_path).read_text())
dual_overhead = hash_report["dualHashOverheadPercent"]
report = {
    "schemaVersion": 1,
    "source": str(pathlib.Path(source).resolve()),
    "destinationRoot": str(pathlib.Path(dest_root).resolve()),
    "baselineBinary": str(pathlib.Path(baseline).resolve()),
    "candidateBinary": str(pathlib.Path(candidate).resolve()),
    "baselineFastSeconds": baseline_times,
    "candidateFastSeconds": candidate_times,
    "baselineFastMedianSeconds": baseline_median,
    "candidateFastMedianSeconds": candidate_median,
    "fastRegressionPercent": fast_regression,
    "fastRegressionLimitPercent": 10.0,
    "storageHash": hash_report,
    "dualHashLimitPercent": dual_limit,
}
pathlib.Path(output).write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
if fast_regression > 10.0:
    raise SystemExit(f"Fast gate failed: {fast_regression:.2f}% > 10.00%")
if dual_overhead > dual_limit:
    raise SystemExit(f"dual-hash gate failed: {dual_overhead:.2f}% > {dual_limit:.2f}%")
print(f"Fast gate passed: {fast_regression:.2f}% <= 10.00%")
print(f"dual-hash gate passed: {dual_overhead:.2f}% <= {dual_limit:.2f}%")
PY
