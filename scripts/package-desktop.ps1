[CmdletBinding()]
param([Parameter(Mandatory=$true)][string]$Executable, [switch]$RequireSigned)
$ErrorActionPreference = 'Stop'
$repoRoot = Split-Path -Parent $PSScriptRoot
$version = '0.5.2'
$source = (Resolve-Path -LiteralPath $Executable).Path
$signature = (Get-AuthenticodeSignature -LiteralPath $source).Status.ToString()
if ($RequireSigned -and $signature -ne 'Valid') { throw 'Production package rejected: trusted Authenticode signature required.' }
$output = Join-Path $repoRoot "dist\c2l-windows-$version"
if (Test-Path -LiteralPath $output) { throw 'Package directory exists; refusing to overwrite a previous artifact.' }
New-Item -ItemType Directory -Path (Join-Path $output 'scripts') -Force | Out-Null
Copy-Item -LiteralPath $source -Destination (Join-Path $output 'c2l-desktop.exe')
Copy-Item -LiteralPath (Join-Path $repoRoot 'LICENSE') -Destination $output
Copy-Item -LiteralPath (Join-Path $repoRoot 'docs\FIVE-MILESTONES.md') -Destination (Join-Path $output 'README.md')
foreach ($name in @('setup-clamav.ps1','install-desktop.ps1')) { Copy-Item -LiteralPath (Join-Path $PSScriptRoot $name) -Destination (Join-Path $output 'scripts') }
$files = @(Get-ChildItem -LiteralPath $output -File -Recurse | ForEach-Object {
    @{ path = $_.FullName.Substring($output.Length + 1).Replace('\','/'); sha256 = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash }
})
@{ version=$version; signature=$signature; productionReady=$false; files=$files } | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $output 'manifest.json') -Encoding utf8
Compress-Archive -LiteralPath $output -DestinationPath "$output.zip"
Write-Output "Created $output.zip; Authenticode=$signature; experimental package, engine and signatures installed separately."
