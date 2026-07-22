#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
tools="$root/src-tauri/tools"

required=(
  "$tools/exiftool/LICENSE.txt"
  "$tools/MEDIAINFO_LICENSE.txt"
  "$tools/ZENLIB_LICENSE.txt"
  "$tools/TINYXML2_LICENSE.txt"
  "$tools/CURL_LICENSE.txt"
  "$tools/ZLIB_LICENSE.txt"
  "$tools/ff/COPYING.LGPLv2.1"
  "$tools/ff/COPYING.LGPLv3"
  "$tools/ff/configure.txt"
)
for file in "${required[@]}"; do
  [[ -s "$file" ]] || { echo "Missing bundled license: $file" >&2; exit 1; }
done

exif_version=$(/usr/bin/perl -I"$tools/exiftool/lib" \
  "$tools/exiftool/exiftool" -ver)
[[ "$exif_version" == "13.55" ]] || {
  echo "Unexpected ExifTool version: $exif_version" >&2
  exit 1
}

mediainfo_version=$("$tools/mediainfo" --Version 2>&1)
grep -Fq 'MediaInfoLib - v26.05' <<<"$mediainfo_version" || {
  echo "Unexpected MediaInfo version" >&2
  exit 1
}
otool -L "$tools/mediainfo" | grep -Fq '@loader_path/libzen.0.dylib' || {
  echo "Bundled MediaInfo does not resolve bundled ZenLib" >&2
  exit 1
}
otool -L "$tools/libzen.0.dylib" | grep -Fq 'current version 0.4.41' || {
  echo "Unexpected bundled ZenLib version" >&2
  exit 1
}

grep -Fq 'This product uses MediaInfo library' \
  "$tools/MEDIAINFO_LICENSE.txt" || {
  echo "MediaInfo binary attribution is missing" >&2
  exit 1
}

bash "$root/scripts/check-ffmpeg-license.sh" "$tools/ff/ffmpeg"
DYLD_LIBRARY_PATH="$tools/ff:$tools" "$tools/ff/ffprobe" -version >/dev/null

python3 - "$root/sbom/native-tools.cdx.json" "$tools/ff/ffmpeg" <<'PY'
import hashlib
import json
import pathlib
import sys

sbom_path, ffmpeg_path = map(pathlib.Path, sys.argv[1:])
sbom = json.loads(sbom_path.read_text())
component = next(item for item in sbom["components"] if item["name"] == "FFmpeg")
expected = next(item["content"] for item in component["hashes"] if item["alg"] == "SHA-256")
actual = hashlib.sha256(ffmpeg_path.read_bytes()).hexdigest()
if actual != expected:
    raise SystemExit(f"Native SBOM FFmpeg hash mismatch: {actual} != {expected}")
PY

echo "Bundled tool versions and license notices passed"
