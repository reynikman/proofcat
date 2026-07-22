#!/usr/bin/env bash
set -euo pipefail

root=$(cd "$(dirname "$0")/.." && pwd)
output_dir=${THIRD_PARTY_SOURCE_DIR:-"$root/.build/third-party-sources"}
mkdir -p "$output_dir"

fetch_verified() {
  local name=$1
  local url=$2
  local expected=$3
  local target="$output_dir/$name"
  curl --fail --location --retry 3 --output "$target" "$url"
  local actual
  if command -v shasum >/dev/null 2>&1; then
    actual=$(shasum -a 256 "$target" | awk '{print $1}')
  else
    actual=$(sha256sum "$target" | awk '{print $1}')
  fi
  [[ "$actual" == "$expected" ]] || {
    echo "Source checksum mismatch for $name" >&2
    exit 1
  }
}

fetch_verified \
  Image-ExifTool-13.55.tar.gz \
  'https://sourceforge.net/projects/exiftool/files/Image-ExifTool-13.55.tar.gz/download' \
  '5f4c81d34ad406538c2871ad72dbfceb5d9b412b2f16cbbeb4d712d270846667'
fetch_verified \
  MediaInfo_CLI_26.05.tar.bz2 \
  'https://mediaarea.net/download/source/mediainfo/26.05/mediainfo_26.05.tar.bz2' \
  '4f0cb5959498f0c4fe74399e68532ffe96a20e96527542ac8bcbc96937d1e3cf'

(
  cd "$output_dir"
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 Image-ExifTool-13.55.tar.gz MediaInfo_CLI_26.05.tar.bz2 \
      > SHA256SUMS
  else
    sha256sum Image-ExifTool-13.55.tar.gz MediaInfo_CLI_26.05.tar.bz2 \
      > SHA256SUMS
  fi
)

echo "Verified third-party sources: $output_dir"
