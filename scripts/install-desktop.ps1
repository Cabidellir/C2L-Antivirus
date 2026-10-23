# Per-user experimental installation. No service, UAC, disk ACL changes, or automatic launch.
[CmdletBinding()]
param([string]$DestinationRoot = (Join-Path $env:LOCALAPPDATA 'C2L-Antivirus\app'), [switch]$RequireSigned)
$ErrorActionPreference = 'Stop'
$packageRoot = Split-Path -Parent $PSScriptRoot
$manifest = Get-Content -LiteralPath (Join-Path $packageRoot 'manifest.json') -Raw | ConvertFrom-Json
if ($manifest.version -notmatch '^\d+\.\d+\.\d+$') { throw 'Invalid package version.' }
foreach ($entry in $manifest.files) {
    if ($entry.path -match '(^[/\\]|\.\.|:)' ) { throw 'Invalid manifest path.' }
    $file = Join-Path $packageRoot $entry.path
    if ((Get-FileHash -LiteralPath $file -Algorithm SHA256).Hash -ne $entry.sha256) { throw "Checksum failure: $($entry.path)" }
}
if ($RequireSigned -and (Get-AuthenticodeSignature (Join-Path $packageRoot 'c2l-desktop.exe')).Status -ne 'Valid') { throw 'Production installation requires a valid code signature.' }
$destination = Join-Path $DestinationRoot $manifest.version
if (Test-Path -LiteralPath $destination) { throw 'Version already installed; no files overwritten.' }
New-Item -ItemType Directory -Path $destination -Force | Out-Null
foreach ($entry in $manifest.files) {
    $target = Join-Path $destination $entry.path
    New-Item -ItemType Directory -Path (Split-Path -Parent $target) -Force | Out-Null
    Copy-Item -LiteralPath (Join-Path $packageRoot $entry.path) -Destination $target
    if ((Get-FileHash -LiteralPath $target -Algorithm SHA256).Hash -ne $entry.sha256) { throw 'Installed file verification failed; incomplete version must not be used.' }
}
Copy-Item -LiteralPath (Join-Path $packageRoot 'manifest.json') -Destination $destination
Write-Output "Installed experimental version at $destination. Existing versions and reports preserved. Engine setup is separate. Checksums are not publisher authentication."
