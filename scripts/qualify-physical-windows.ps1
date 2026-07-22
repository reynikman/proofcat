param(
    [Parameter(Mandatory=$true)][string]$Source,
    [Parameter(Mandatory=$true)][string]$NtfsDestination,
    [Parameter(Mandatory=$true)][string]$ExfatDestination,
    [string]$RecordDir = ".build/physical-windows",
    [string]$Cli = "target/release/proofcat-cli.exe"
)
$ErrorActionPreference = "Stop"

function Get-PhysicalVolume([string]$Path) {
    $Item = Get-Item $Path
    $Volume = Get-Volume -FilePath $Item.FullName
    $Partition = Get-Partition -Volume $Volume
    $Disk = Get-Disk -Number $Partition.DiskNumber
    if ($Disk.BusType -in @("Virtual", "File Backed Virtual", "Unknown")) {
        throw "Virtual or ambiguous disk is not physical evidence: $($Disk.BusType)"
    }
    [pscustomobject]@{ Volume=$Volume; Partition=$Partition; Disk=$Disk }
}

$SourceInfo = Get-PhysicalVolume $Source
$NtfsInfo = Get-PhysicalVolume $NtfsDestination
$ExfatInfo = Get-PhysicalVolume $ExfatDestination
if ($SourceInfo.Volume.FileSystem -ne "exFAT") { throw "Source must be exFAT" }
if ($NtfsInfo.Volume.FileSystem -ne "NTFS") { throw "Destination 1 must be NTFS" }
if ($ExfatInfo.Volume.FileSystem -ne "exFAT") { throw "Destination 2 must be exFAT" }
$DiskNumbers = @($SourceInfo.Disk.Number, $NtfsInfo.Disk.Number, $ExfatInfo.Disk.Number)
if (($DiskNumbers | Select-Object -Unique).Count -ne 3) {
    throw "Source and destinations must use three physical disks"
}
if (-not (Test-Path $Cli)) { cargo build --release -p proofcat --bin proofcat-cli }
Remove-Item -Recurse -Force $RecordDir -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force $RecordDir | Out-Null
$Job = "job-physical-windows-$((Get-Date).ToUniversalTime().ToString('yyyyMMddTHHmmssZ'))"
$Db = Join-Path $RecordDir "offload.sqlite"
$OffloadStdout = Join-Path $RecordDir "offload.stdout.log"
$OffloadStderr = Join-Path $RecordDir "offload.stderr.log"
$OffloadArgs = "offload --source $Source --dest $NtfsDestination --dest $ExfatDestination " +
    "--profile archive-max --db $Db --job $Job"
$OffloadCommand = "`"$Cli`" $OffloadArgs 1>`"$OffloadStdout`" 2>`"$OffloadStderr`""
cmd.exe /d /s /c $OffloadCommand
$OffloadExitCode = $LASTEXITCODE
Get-Content $OffloadStdout, $OffloadStderr | Set-Content -Encoding utf8 (Join-Path $RecordDir "offload.log")
if ($OffloadExitCode -ne 0) { throw "Offload failed with exit code $OffloadExitCode" }
& $Cli verify $NtfsDestination --all > (Join-Path $RecordDir "ntfs-verify.json")
if ($LASTEXITCODE -ne 0) { throw "NTFS verification failed" }
& $Cli verify $ExfatDestination --all > (Join-Path $RecordDir "exfat-verify.json")
if ($LASTEXITCODE -ne 0) { throw "exFAT verification failed" }
& $Cli report --job $Job --db $Db --format json --output (Join-Path $RecordDir "evidence.json")
$Evidence = Get-Content -Raw (Join-Path $RecordDir "evidence.json") | ConvertFrom-Json
if ($Evidence.verdict -ne "SAFE_TO_FORMAT" -or -not $Evidence.safeToFormat) {
    throw "Physical qualification did not produce SAFE_TO_FORMAT"
}
$Git = Get-Command git -ErrorAction SilentlyContinue
if (-not $Git) {
    $BundledGit = Join-Path $env:USERPROFILE ".cache\codex-runtimes\codex-primary-runtime\dependencies\native\git\cmd\git.exe"
    if (Test-Path $BundledGit) { $Git = $BundledGit }
}
$Commit = if ($Git) { & $Git rev-parse HEAD } else { "unknown" }
@($SourceInfo, $NtfsInfo, $ExfatInfo) | ConvertTo-Json -Depth 5 |
    Set-Content -Encoding utf8 (Join-Path $RecordDir "storage-topology.json")
[pscustomobject]@{
    schemaVersion = 1
    kind = "physical-media"
    completedAt = (Get-Date).ToUniversalTime().ToString("o")
    jobId = $Job
    verdict = $Evidence.verdict
    commit = $Commit
} | ConvertTo-Json | Set-Content -Encoding utf8 (Join-Path $RecordDir "qualification.json")
Write-Host "Physical Windows qualification passed: $RecordDir"
