[CmdletBinding()]
param([Parameter(Mandatory=$true)][string]$Package)
$ErrorActionPreference = 'Stop'
$packageRoot = (Resolve-Path -LiteralPath $Package).Path
$testRoot = Join-Path ([IO.Path]::GetTempPath()) ('c2l-package-test-' + [Guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $testRoot | Out-Null
$installer = Join-Path $packageRoot 'scripts\install-desktop.ps1'
$destination = Join-Path $testRoot 'install'
& $installer -DestinationRoot $destination
$manifest = Get-Content -LiteralPath (Join-Path $packageRoot 'manifest.json') -Raw | ConvertFrom-Json
$installed = Join-Path (Join-Path $destination $manifest.version) 'c2l-desktop.exe'
if (-not (Test-Path -LiteralPath $installed)) { throw 'Executable missing after installation.' }
$before = (Get-FileHash -LiteralPath $installed).Hash
$rejected = $false
try { & $installer -DestinationRoot $destination } catch {
    if ($_.Exception.Message -notlike '*already installed*') { throw }
    $rejected = $true
}
if (-not $rejected -or (Get-FileHash -LiteralPath $installed).Hash -ne $before) { throw 'Existing installation was not protected.' }
$rejected = $false
try { & $installer -DestinationRoot (Join-Path $testRoot 'signed') -RequireSigned } catch {
    if ($_.Exception.Message -notlike '*valid code signature*') { throw }
    $rejected = $true
}
if (-not $rejected) { throw 'Unsigned package unexpectedly passed signed-only gate.' }
$corrupt = Join-Path $testRoot 'corrupt'
Copy-Item -LiteralPath $packageRoot -Destination $corrupt -Recurse
Add-Content -LiteralPath (Join-Path $corrupt 'README.md') -Value 'Intentional test corruption'
$rejected = $false
try { & (Join-Path $corrupt 'scripts\install-desktop.ps1') -DestinationRoot (Join-Path $testRoot 'rejected') } catch {
    if ($_.Exception.Message -notlike '*Checksum failure*') { throw }
    $rejected = $true
}
if (-not $rejected -or (Test-Path -LiteralPath (Join-Path $testRoot 'rejected'))) { throw 'Corrupt package was not rejected before installation.' }
Write-Output "PASS: install, content verification, overwrite refusal, unsigned refusal, corruption refusal. Isolated test files retained at $testRoot"
