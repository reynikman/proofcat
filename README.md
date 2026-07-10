# Meta Report

Local, offline media **metadata inspector** for macOS and Windows. Drop a file — get the full picture from **MediaInfo · ExifTool · FFprobe** in one window, plus **EBU R128 loudness** (LUFS / True Peak / LRA) measured on demand. Export a compact Markdown report to hand to a colorist, sound engineer, or expert.

Everything runs on your machine. Nothing is uploaded — the only network use is the optional "Check for updates" button.

## Features

- **One window, three engines** — MediaInfo, ExifTool, FFprobe, native readable output.
- **Pro mode** (toggle next to Compare) — grouped cards for cinematographers / colorists / sound:
  - Camera & lens: model, lens, focal length, aperture, ISO, shutter, white balance, color temp, 35mm eq., serial.
  - Color / codec: picture profile / log, color primaries, transfer (gamma), matrix, range, chroma, bit depth.
  - Slate / sound: timecode, Scene, Take, Reel, track names (iXML / bext).
- **Loudness (EBU R128)** — Integrated LUFS, True Peak (dBTP), LRA. Targets: YouTube −14 · Broadcast −23 LUFS · Peak ≤ −1 dBTP.
- **Compare** two files side by side · in-output **search** (⌘/Ctrl+F) · **Copy** / **Save report (.md)**.
- **English / Russian**, dark UI.
- **Auto-update** — built-in, one click.

## Download

Grab the latest installer from [**Releases**](https://github.com/reynikman/meta-report/releases/latest):

- **macOS (Apple Silicon):** `.dmg`
- **Windows (x64):** `-setup.exe`

The apps are not code-signed yet, so on first launch:
- **macOS:** right-click → Open (or System Settings → Privacy & Security → Open Anyway).
- **Windows:** SmartScreen → More info → Run anyway.

## Feedback

Use the in-app **Send feedback** button, or email nikolaytolstykh@yandex.ru.
