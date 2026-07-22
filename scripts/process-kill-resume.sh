#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
binary=${META_REPORT_CLI:-"$root/target/debug/proofcat-cli"}
work_root=${META_REPORT_KILL_WORK_DIR:-"$root/.build/process-kill"}

if [[ ! -x "$binary" ]]; then
  cargo build -p proofcat --bin proofcat-cli --manifest-path "$root/Cargo.toml"
fi

rm -rf "$work_root"
mkdir -p "$work_root"

run_phase() {
  local phase=$1
  local case_dir="$work_root/$phase"
  local source="$case_dir/source"
  local destination="$case_dir/destination"
  local db="$case_dir/offload.sqlite"
  local job="job-process-kill-$phase"
  local fifo="$case_dir/progress.fifo"
  local stdout="$case_dir/first-run.stdout"
  mkdir -p "$source" "$destination"
  dd if=/dev/zero of="$source/clip.mov" bs=1048576 count=16 status=none
  printf 'sidecar\n' > "$source/clip.txt"
  mkfifo "$fifo"

  META_REPORT_HARNESS_PHASE_DELAY_MS=500 "$binary" offload \
    --source "$source" --dest "$destination" --profile archive-max \
    --db "$db" --job "$job" >"$stdout" 2>"$fifo" &
  local pid=$!
  local killed=0
  while IFS= read -r line; do
    printf '%s\n' "$line" >> "$case_dir/first-run.progress"
    if [[ "$phase" == repairing* || "$phase" == "repairReadback" ]] && \
       [[ "$line" == destinationVerify* ]]; then
      printf '\377' | dd of="$destination/clip.mov" bs=1 count=1 conv=notrunc status=none
    fi
    if [[ "$line" == "$phase "* ]]; then
      kill -9 "$pid" 2>/dev/null || true
      killed=1
      break
    fi
  done < "$fifo"
  if [[ "$killed" != 1 ]]; then
    echo "Phase was not reached: $phase" >&2
    kill -9 "$pid" 2>/dev/null || true
    wait "$pid" 2>/dev/null || true
    return 1
  fi
  wait "$pid" 2>/dev/null || true
  rm -f "$fifo"

  "$binary" resume --job "$job" --db "$db" > "$case_dir/resume.json"
  "$binary" verify "$destination" --all > "$case_dir/verify.json"
  "$binary" report --job "$job" --db "$db" --format json \
    --output "$case_dir/evidence.json" >/dev/null
  python3 - "$case_dir/evidence.json" "$phase" <<'PY'
import json
import pathlib
import sys

evidence = json.loads(pathlib.Path(sys.argv[1]).read_text())
phase = sys.argv[2]
assert evidence["verdict"] == "ARCHIVE_VERIFIED", (phase, evidence["verdict"])
assert evidence["safeToFormat"] is False
assert evidence["verificationFailed"] == 0
assert all(row["status"] == "verified" for row in evidence["replicas"])
PY
  if find "$destination" -type f -name '*.tmp-*' -print -quit | grep -q .; then
    echo "Orphan temporary file after resume: $phase" >&2
    return 1
  fi
  echo "process-kill phase passed: $phase"
}

for phase in scanning sourcePreRead copying copyingData destinationVerify \
  repairing repairingData repairReadback mhl; do
  run_phase "$phase"
done

echo "All process-kill/resume phases passed: $work_root"
