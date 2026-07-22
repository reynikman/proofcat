#!/usr/bin/env bash
set -euo pipefail

binary=${1:-src-tauri/tools/ff/ffmpeg}
version_output=$("$binary" -version 2>&1)
configuration=$(printf '%s\n' "$version_output" | sed -n 's/^configuration: //p')

if [[ -z "$configuration" ]]; then
  echo "Cannot read FFmpeg configure flags from $binary" >&2
  exit 1
fi
if grep -Eq -- '--enable-(gpl|nonfree)|--enable-lib(x264|x265|fdk-aac)' <<<"$configuration"; then
  echo "Release blocked: GPL/nonfree FFmpeg configuration detected" >&2
  exit 1
fi
for required in --disable-gpl --disable-nonfree --enable-ffmpeg --enable-ffprobe; do
  if [[ "$configuration" != *"$required"* ]]; then
    echo "Release blocked: missing FFmpeg flag $required" >&2
    exit 1
  fi
done
filters=$("$binary" -hide_banner -filters 2>/dev/null)
for required_filter in ebur128 blackdetect freezedetect; do
  if ! grep -Eq "[[:space:]]${required_filter}[[:space:]]" <<<"$filters"; then
    echo "Release blocked: missing FFmpeg filter $required_filter" >&2
    exit 1
  fi
done

echo "FFmpeg license gate passed"
