#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
binary=${META_REPORT_CLI:-"$root/target/debug/proofcat-cli"}
duration=${SOAK_DURATION_SECONDS:-86400}
interval=${SOAK_INTERVAL_SECONDS:-300}
payload_mb=${SOAK_PAYLOAD_MB:-64}
work_root=${SOAK_WORK_DIR:-"$root/.build/soak"}
results="$work_root/results.jsonl"
source_dir=${SOAK_SOURCE_DIR:-"$work_root/source"}
destination_list=${SOAK_DESTINATIONS:-"$work_root/destination"}
expect_safe=${EXPECT_SAFE_TO_FORMAT:-0}
IFS=':' read -r -a destination_roots <<< "$destination_list"

if [[ ! -x "$binary" ]]; then
  cargo build -p proofcat --bin proofcat-cli --manifest-path "$root/Cargo.toml"
fi
if ! [[ "$duration" =~ ^[0-9]+$ && "$interval" =~ ^[0-9]+$ && "$payload_mb" =~ ^[0-9]+$ && "$expect_safe" =~ ^[01]$ ]]; then
  echo "Soak duration, interval and payload must be non-negative integers" >&2
  exit 1
fi
if (( ${#destination_roots[@]} == 0 )); then
  echo "SOAK_DESTINATIONS must contain at least one destination root" >&2
  exit 1
fi

rm -rf "$work_root"
mkdir -p "$work_root"
if [[ -z "${SOAK_SOURCE_DIR:-}" ]]; then
  mkdir -p "$source_dir"
  dd if=/dev/zero of="$source_dir/master.mov" bs=1048576 count="$payload_mb" status=none
  printf 'Meta Report 24-hour soak\n' > "$source_dir/metadata.txt"
elif [[ ! -d "$source_dir" ]]; then
  echo "SOAK_SOURCE_DIR does not exist: $source_dir" >&2
  exit 2
fi
for destination_root in "${destination_roots[@]}"; do
  [[ -d "$destination_root" ]] || mkdir -p "$destination_root"
done
: > "$results"

started_epoch=$(date +%s)
deadline=$((started_epoch + duration))
iteration=0
while :; do
  now=$(date +%s)
  if (( iteration > 0 && now >= deadline )); then
    break
  fi
  iteration=$((iteration + 1))
  job="job-soak-$started_epoch-$iteration"
  db="$work_root/offload-$iteration.sqlite"
  destination_args=()
  iteration_destinations=()
  for destination_root in "${destination_roots[@]}"; do
    iteration_destination="$destination_root/meta-report-soak-$started_epoch-$iteration"
    rm -rf "$iteration_destination"
    mkdir -p "$iteration_destination"
    iteration_destinations+=("$iteration_destination")
    destination_args+=(--dest "$iteration_destination")
  done

  "$binary" offload --source "$source_dir" \
    "${destination_args[@]}" --profile archive-max \
    --db "$db" --job "$job" > "$work_root/offload-$iteration.json" 2>&1
  verify_index=0
  for iteration_destination in "${iteration_destinations[@]}"; do
    verify_index=$((verify_index + 1))
    "$binary" verify "$iteration_destination" --all \
      > "$work_root/verify-$iteration-$verify_index.json"
  done
  "$binary" report --job "$job" --db "$db" --format json \
    --output "$work_root/evidence-$iteration.json" >/dev/null
  python3 - "$work_root/evidence-$iteration.json" "$results" "$iteration" "$expect_safe" <<'PY'
import datetime
import json
import pathlib
import sys

evidence_path, results_path, iteration, expect_safe = sys.argv[1:]
evidence = json.loads(pathlib.Path(evidence_path).read_text())
expected_verdict = "SAFE_TO_FORMAT" if expect_safe == "1" else "ARCHIVE_VERIFIED"
assert evidence["verdict"] == expected_verdict, evidence["verdict"]
assert evidence["safeToFormat"] is (expect_safe == "1")
assert evidence["verificationFailed"] == 0
assert all(row["status"] == "verified" for row in evidence["replicas"])
record = {
    "iteration": int(iteration),
    "completedAt": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "jobId": evidence["jobId"],
    "verdict": evidence["verdict"],
    "verifiedReplicas": evidence["verifiedReplicas"],
    "safeToFormat": evidence["safeToFormat"],
}
with open(results_path, "a", encoding="utf-8") as stream:
    stream.write(json.dumps(record, sort_keys=True) + "\n")
PY
  for iteration_destination in "${iteration_destinations[@]}"; do
    rm -rf "$iteration_destination"
  done

  now=$(date +%s)
  if (( now >= deadline )); then
    break
  fi
  remaining=$((deadline - now))
  delay=$interval
  if (( delay > remaining )); then
    delay=$remaining
  fi
  if (( delay > 0 )); then
    sleep "$delay"
  fi
done

finished_epoch=$(date +%s)
python3 - "$results" "$work_root/summary.json" "$started_epoch" "$finished_epoch" "$duration" "$expect_safe" <<'PY'
import json
import pathlib
import sys

results, output, started, finished, requested, expect_safe = sys.argv[1:]
records = [json.loads(line) for line in pathlib.Path(results).read_text().splitlines() if line]
expected_verdict = "SAFE_TO_FORMAT" if expect_safe == "1" else "ARCHIVE_VERIFIED"
summary = {
    "schemaVersion": 1,
    "requestedDurationSeconds": int(requested),
    "actualDurationSeconds": int(finished) - int(started),
    "iterations": len(records),
    "expectedVerdict": expected_verdict,
    "allExpectedVerdict": bool(records) and all(r["verdict"] == expected_verdict for r in records),
    "safeToFormatCount": sum(bool(r["safeToFormat"]) for r in records),
}
pathlib.Path(output).write_text(json.dumps(summary, indent=2, sort_keys=True) + "\n")
assert summary["allExpectedVerdict"]
assert summary["safeToFormatCount"] == (len(records) if expect_safe == "1" else 0)
assert summary["actualDurationSeconds"] >= int(requested)
PY

echo "Soak passed: $work_root/summary.json"
