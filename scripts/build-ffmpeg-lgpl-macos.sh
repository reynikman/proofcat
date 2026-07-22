#!/usr/bin/env bash
set -euo pipefail

version="8.0.3"
archive="ffmpeg-${version}.tar.xz"
expected_sha256="6136812ea6d4e68bdba27e33c2a94382711cdf4f8602ffef056ff792bd6f9818"
root=$(cd "$(dirname "$0")/.." && pwd)
work_dir=${FFMPEG_WORK_DIR:-"$root/.build/ffmpeg-lgpl"}
output_dir=${FFMPEG_OUTPUT_DIR:-"$root/src-tauri/tools/ff-lgpl"}
source_url="https://ffmpeg.org/releases/${archive}"
install_prefix="/meta-report/ffmpeg"
stage_dir="$work_dir/stage"
install_dir="$stage_dir$install_prefix"

mkdir -p "$work_dir"
if [[ ! -f "$work_dir/$archive" ]]; then
  curl --fail --location --retry 3 --output "$work_dir/$archive" "$source_url"
fi
actual_sha256=$(shasum -a 256 "$work_dir/$archive" | awk '{print $1}')
if [[ "$actual_sha256" != "$expected_sha256" ]]; then
  echo "FFmpeg source checksum mismatch" >&2
  exit 1
fi

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
  --enable-videotoolbox
  --enable-audiotoolbox
  --disable-sdl2
  --disable-xlib
  --disable-libxcb
  --disable-indev=xcbgrab
)

(
  cd "$work_dir/source"
  ./configure "${configure[@]}"
  make -j"$(sysctl -n hw.logicalcpu)"
  make install DESTDIR="$stage_dir"
)

cp "$install_dir/bin/ffmpeg" "$install_dir/bin/ffprobe" "$output_dir/"
find "$install_dir/lib" -maxdepth 1 -type f -name '*.dylib' -exec cp {} "$output_dir/" \;
cp "$work_dir/source/COPYING.LGPLv2.1" "$output_dir/"
cp "$work_dir/source/COPYING.LGPLv3" "$output_dir/"
cp "$work_dir/$archive" "$output_dir/"
printf '%s\n' "${configure[@]}" > "$output_dir/configure.txt"

for library in "$output_dir"/*.dylib; do
  install_name_tool -id "@loader_path/$(basename "$library")" "$library"
done
for binary in "$output_dir/ffmpeg" "$output_dir/ffprobe" "$output_dir"/*.dylib; do
  while IFS= read -r dependency; do
    [[ "$dependency" == "$install_prefix/lib/"* ]] || continue
    dependency_name=$(basename "$dependency")
    if [[ -L "$install_dir/lib/$dependency_name" ]]; then
      dependency_name=$(readlink "$install_dir/lib/$dependency_name")
    fi
    install_name_tool -change "$dependency" "@loader_path/$dependency_name" "$binary"
  done < <(otool -L "$binary" | tail -n +2 | awk '{print $1}')
done

bash "$root/scripts/check-ffmpeg-license.sh" "$output_dir/ffmpeg"
DYLD_LIBRARY_PATH="$output_dir" "$output_dir/ffprobe" -version >/dev/null
DYLD_LIBRARY_PATH="$output_dir" "$output_dir/ffmpeg" -hide_banner -f lavfi \
  -i color=size=16x16:rate=1 -t 0.1 -f null - >/dev/null 2>&1

echo "LGPL-only FFmpeg bundle: $output_dir"
