# LGPL-only FFmpeg release build

The macOS bundle is built from pinned FFmpeg 8.0.3 source with GPL and nonfree
features disabled. The same policy is enforced for the Windows release build.

Public artifacts build the pinned FFmpeg 8.0.3 source with
`scripts/build-ffmpeg-lgpl-macos.sh` and at least:

```text
--disable-gpl
--disable-nonfree
--disable-doc
--disable-debug
--enable-shared
--disable-static
--enable-ffmpeg
--enable-ffprobe
```

Do not enable x264, x265 or other GPL/nonfree libraries. The release job must:

1. record the upstream tag and exact configure line;
2. archive the exact source alongside binaries;
3. retain dynamic library names and license texts;
4. run `ffmpeg -version` and fail if it contains `--enable-gpl` or
   `--enable-nonfree`;
5. exercise MediaInfo/FFprobe, loudness and black/freeze detection before
   publishing the installer.

Any bundled binary that fails the license gate is a release blocker.

The build output contains `configure.txt`, the LGPL texts and the exact verified
source archive. `scripts/check-ffmpeg-license.sh` rejects GPL/nonfree flags and
known GPL/nonfree external codec libraries. The manual Release readiness
workflow uploads the binary bundle and corresponding source together.

On Windows, `scripts/assemble-windows-tools.ps1` combines that FFmpeg build with
checksum-pinned MediaInfo 26.05 and ExifTool 13.55 distributions. Their runtime
files and transitive license texts are staged under `tools-windows/`; the
platform Tauri config maps only that directory into the installer `tools/`
resource path.

## Windows release gate and VM caveat

`src-tauri/tools-windows/` is gitignored and can be created empty merely to let
Tauri resolve its resource mapping during a normal compile. That placeholder is
**not** a native-tools bundle and does not prove MediaInfo, ExifTool or FFmpeg
availability in the installed application.

The assembler recreates `tools-windows/` on every run. It retains downloaded
source archives in `.build/windows-tools/` only as a cache, and verifies each
cached archive against its pinned SHA-256 before extracting it; a cache hit is
never accepted without that verification.

For a release that claims Windows native media tooling, the required sequence is:

1. build the pinned LGPL-only Windows FFmpeg input;
2. run `scripts/assemble-windows-tools.ps1` against that input;
3. run the bundled-tools and license gates; and
4. retain the generated checksums, source archives and licence texts with the
   release evidence.

Do not reuse an unverified folder left on a VM or a binary downloaded outside
this workflow. If the VM cannot obtain the pinned build dependencies, stop this
gate and ship only after the package description states that Windows native
runtime tools are absent; do not silently label the placeholder directory as a
complete bundle.
