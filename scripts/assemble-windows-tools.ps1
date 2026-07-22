param(
    [string]$FfmpegDir = "",
    [string]$OutputDir = ""
)

$ErrorActionPreference = "Stop"
$Root = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
if (-not $OutputDir) { $OutputDir = Join-Path $Root "src-tauri/tools-windows" }
if (-not $FfmpegDir) { $FfmpegDir = Join-Path $Root "src-tauri/tools/ff-lgpl-windows" }
$Work = Join-Path $Root ".build/windows-tools"
$MediaInfoArchive = Join-Path $Work "MediaInfo_CLI_26.05_Windows_x64.zip"
$ExifToolArchive = Join-Path $Work "exiftool-13.55_64.zip"

function Get-VerifiedArchive {
    param([string]$Uri, [string]$Path, [string]$Expected)
    if (-not (Test-Path $Path)) {
        Invoke-WebRequest -UseBasicParsing -Uri $Uri -OutFile $Path
    }
    $Actual = (Get-FileHash -Algorithm SHA256 $Path).Hash.ToLowerInvariant()
    if ($Actual -ne $Expected) {
        throw "Checksum mismatch for $Path`: $Actual"
    }
}

Remove-Item -Recurse -Force $OutputDir -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force $Work, $OutputDir | Out-Null
Get-VerifiedArchive `
    "https://mediaarea.net/download/binary/mediainfo/26.05/MediaInfo_CLI_26.05_Windows_x64.zip" `
    $MediaInfoArchive `
    "f7f80620ce6d14f4995f0de6f98e3ef18ad29496db01899571152ee3311229f9"
Get-VerifiedArchive `
    "https://sourceforge.net/projects/exiftool/files/exiftool-13.55_64.zip/download" `
    $ExifToolArchive `
    "9fadaf5221dcb07a5d018e21c8529e257917d2fad1fdfc8e64855c4fb73293df"

$MediaInfoExtract = Join-Path $Work "mediainfo"
$ExifToolExtract = Join-Path $Work "exiftool"
Expand-Archive $MediaInfoArchive $MediaInfoExtract
Expand-Archive $ExifToolArchive $ExifToolExtract
Copy-Item (Join-Path $MediaInfoExtract "MediaInfo.exe") (Join-Path $OutputDir "mediainfo.exe")
Copy-Item (Join-Path $MediaInfoExtract "LIBCURL.DLL") $OutputDir
$ExifRoot = Join-Path $ExifToolExtract "exiftool-13.55_64"
Copy-Item (Join-Path $ExifRoot "exiftool(-k).exe") (Join-Path $OutputDir "exiftool.exe")
Copy-Item -Recurse (Join-Path $ExifRoot "exiftool_files") $OutputDir

if (-not (Test-Path (Join-Path $FfmpegDir "ffmpeg.exe"))) {
    throw "Pinned LGPL FFmpeg output is missing: $FfmpegDir"
}
Copy-Item (Join-Path $FfmpegDir "*.exe") $OutputDir
Copy-Item (Join-Path $FfmpegDir "*.dll") $OutputDir
Copy-Item (Join-Path $FfmpegDir "COPYING.LGPL*") $OutputDir
Copy-Item (Join-Path $FfmpegDir "configure.txt") $OutputDir

foreach ($Notice in @(
    "MEDIAINFO_LICENSE.txt", "ZENLIB_LICENSE.txt", "TINYXML2_LICENSE.txt",
    "CURL_LICENSE.txt", "ZLIB_LICENSE.txt"
)) {
    Copy-Item (Join-Path $Root "src-tauri/tools/$Notice") $OutputDir
}
Copy-Item (Join-Path $Root "src-tauri/tools/exiftool/LICENSE.txt") `
    (Join-Path $OutputDir "EXIFTOOL_LICENSE.txt")

$MediaInfoVersion = & (Join-Path $OutputDir "mediainfo.exe") --Version
if (($MediaInfoVersion -join "`n") -notmatch "MediaInfoLib - v26.05") {
    throw "Unexpected MediaInfo version"
}
$ExifToolVersion = & (Join-Path $OutputDir "exiftool.exe") -ver
if ($ExifToolVersion.Trim() -ne "13.55") { throw "Unexpected ExifTool version" }
$FfmpegVersion = & (Join-Path $OutputDir "ffmpeg.exe") -version
if (($FfmpegVersion -join "`n") -match "--enable-(gpl|nonfree)") {
    throw "GPL/nonfree FFmpeg configuration detected"
}

$Hashes = Get-ChildItem -File -Recurse $OutputDir | Sort-Object FullName | ForEach-Object {
    $Relative = $_.FullName.Substring($OutputDir.Length + 1).Replace("\", "/")
    "{0}  {1}" -f (Get-FileHash -Algorithm SHA256 $_.FullName).Hash.ToLowerInvariant(), $Relative
}
$Hashes | Set-Content -Encoding ascii (Join-Path $OutputDir "SHA256SUMS")
Write-Host "Verified Windows tools: $OutputDir"
