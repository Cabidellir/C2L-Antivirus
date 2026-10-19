# Install the pinned official x64 engine under the current user; no elevation or AV changes.
[CmdletBinding()]
param([switch]$SkipUpdate)
$ErrorActionPreference = 'Stop'
$version = '1.5.4'
$expected = '0d9e0228b2674137ea1a2853566c98a0278ad52ab2582c3d6dbd75373848c395'
$engineRoot = Join-Path $env:LOCALAPPDATA 'C2L-Antivirus\engine'
New-Item -ItemType Directory -Path $engineRoot -Force | Out-Null
$archive = Join-Path $engineRoot "clamav-$version.win.x64.zip"
if (-not (Test-Path -LiteralPath $archive)) {
    Invoke-WebRequest "https://github.com/Cisco-Talos/clamav/releases/download/clamav-$version/clamav-$version.win.x64.zip" -OutFile $archive
}
if ((Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant() -ne $expected) {
    throw 'Official package checksum mismatch. Installation stopped.'
}
$destination = Join-Path $engineRoot "clamav-$version"
$binaryDir = Join-Path $destination "clamav-$version.win.x64"
if (-not (Test-Path -LiteralPath (Join-Path $binaryDir 'clamscan.exe'))) {
    Expand-Archive -LiteralPath $archive -DestinationPath $destination
}
$database = Join-Path $engineRoot 'database'
New-Item -ItemType Directory -Path $database -Force | Out-Null
$config = Join-Path $engineRoot 'freshclam.conf'
$lines = @("DatabaseDirectory `"$database`"", 'DatabaseMirror database.clamav.net', 'Checks 12', 'TestDatabases yes', 'ConnectTimeout 15', 'ReceiveTimeout 60')
[IO.File]::WriteAllLines($config, $lines, [Text.UTF8Encoding]::new($false))
& (Join-Path $binaryDir 'clamscan.exe') --version
if ($LASTEXITCODE -ne 0) { throw 'Engine could not start.' }
if (-not $SkipUpdate) {
    & (Join-Path $binaryDir 'freshclam.exe') "--config-file=$config" --quiet
    if ($LASTEXITCODE -ne 0) { throw 'Signature update failed. Retry from the application.' }
}
Write-Output "Engine ready at $binaryDir. GPL notices and license remain in the extracted package."
