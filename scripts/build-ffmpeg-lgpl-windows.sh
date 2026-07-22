#!/usr/bin/env bash
set -euo pipefail

version="8.0.3"
archive="ffmpeg-${version}.tar.xz"
expected_sha256="6136812ea6d4e68bdba27e33c2a94382711cdf4f8602ffef056ff792bd6f9818"
root=$(cd "$(dirname "$0")/.." && pwd)
work_dir=${FFMPEG_WORK_DIR:-"$root/.build/ffmpeg-lgpl-windows"}
output_dir=${FFMPEG_OUTPUT_DIR:-"$root/src-tauri/tools/ff-lgpl-windows"}
install_prefix="/meta-report/ffmpeg"
stage_dir="$work_dir/stage"
install_dir="$stage_dir$install_prefix"

mkdir -p "$work_dir"
if [[ ! -f "$work_dir/$archive" ]]; then
  curl --fail --location --retry 3 --output "$work_dir/$archive" \
    "https://ffmpeg.org/releases/$archive"
fi
actual_sha256=$(sha256sum "$work_dir/$archive" | awk '{print $1}')
[[ "$actual_sha256" == "$expected_sha256" ]] || {
  echo "FFmpeg source checksum mismatch" >&2
  exit 1
}

rm -rf "$work_dir/source" "$stage_dir" "$output_dir"
mkdir -p "$work_dir/source" "$stage_dir" "$output_dir"
tar -xf "$work_dir/$archive" -C "$work_dir/source" --strip-components=1

configure=(
  --prefix="$install_prefix"
  --disable-gpl
  --disable-nonfree
  --disable-doc
  --disable-debug
  --disable-static
  --enable-shared
  --disable-ffplay
  --enable-ffmpeg
  --enable-ffprobe
  --disable-sdl2
)
(
  cd "$work_dir/source"
  ./configure "${configure[@]}"
  make -j"${NUMBER_OF_PROCESSORS:-4}"
  make install DESTDIR="$stage_dir"
)

cp "$install_dir/bin/ffmpeg.exe" "$install_dir/bin/ffprobe.exe" "$output_dir/"
find "$install_dir/bin" -maxdepth 1 -type f -name '*.dll' -exec cp {} "$output_dir/" \;
cp "$work_dir/source/COPYING.LGPLv2.1" "$work_dir/source/COPYING.LGPLv3" "$output_dir/"
cp "$work_dir/$archive" "$output_dir/"
printf '%s\n' "${configure[@]}" > "$output_dir/configure.txt"

bash "$root/scripts/check-ffmpeg-license.sh" "$output_dir/ffmpeg.exe"
"$output_dir/ffprobe.exe" -version >/dev/null
"$output_dir/ffmpeg.exe" -hide_banner -f lavfi -i color=size=16x16:rate=1 \
  -t 0.1 -f null - >/dev/null 2>&1
echo "LGPL-only FFmpeg Windows bundle: $output_dir"
