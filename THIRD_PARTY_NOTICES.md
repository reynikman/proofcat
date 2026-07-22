# Third-party notices

ProofCat is MIT-licensed. Components distributed with or used by the app
retain their own licenses.

## DIT-Pro

Parts of `src-tauri/src/offload/` were ported and adapted from
[DIT-Pro](https://github.com/WillZ5/DIT-Pro), Copyright (c) 2026 WillZ, MIT
License. The original notice and license are retained in `NOTICE`.

## IBM Plex

The user interface bundles the IBM Plex Sans and IBM Plex Mono webfonts
(`src/fonts/*.woff2`), Copyright (c) IBM Corp., licensed under the
[SIL Open Font License 1.1](https://github.com/IBM/plex/blob/master/LICENSE.txt).
The fonts are shipped with the app so the interface renders identically with no
network access; nothing is fetched from a font CDN at runtime.

## ASC Media Hash List

The ASC MHL format and reference implementation are maintained by the American
Society of Cinematographers. The reference implementation is MIT-licensed.
ProofCat's Rust implementation is independent and must pass interoperability
fixtures before a release is labelled ASC MHL compatible.

## FFmpeg

FFmpeg is LGPL-2.1-or-later by default. ProofCat bundles a pinned FFmpeg
8.0.3 build with GPL/nonfree features disabled. The bundle includes the exact
configuration and LGPL texts; every release also publishes the verified source
archive used by the macOS and Windows build workflows.

See `docs/ffmpeg-build.md` and <https://ffmpeg.org/legal.html>.

## ExifTool

ProofCat bundles the unmodified ExifTool 13.55 distribution, Copyright
2003-2026 Phil Harvey, under the Perl Artistic License option. The complete
license is bundled at `src-tauri/tools/exiftool/LICENSE.txt`; the exact upstream
source archive is published with binary releases. <https://exiftool.org/>

## MediaInfo / ZenLib

ProofCat bundles MediaInfo/MediaInfoLib 26.05 under the BSD 2-Clause License
and ZenLib 0.4.41 under the zlib license. MediaInfoLib embeds TinyXML-2 under
its zlib-style license and dynamically uses the macOS-provided libcurl and
zlib. The official Windows MediaInfo binary bundles libcurl 8.11.0-DEV and
zlib 1.3.1. Their complete notices are at
`src-tauri/tools/MEDIAINFO_LICENSE.txt` and
`src-tauri/tools/ZENLIB_LICENSE.txt` and
`src-tauri/tools/TINYXML2_LICENSE.txt`.
The libcurl and zlib texts are at `src-tauri/tools/CURL_LICENSE.txt` and
`src-tauri/tools/ZLIB_LICENSE.txt`.

This product uses [MediaInfo](https://mediaarea.net/MediaInfo) library,
Copyright (c) 2002-2025 MediaArea.net SARL.

Rust and JavaScript dependency licenses are captured by the release SBOM and
dependency audit workflow.
