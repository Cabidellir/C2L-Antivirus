# Non-elevated isolated acceptance: real official network update, no SCM changes.
[CmdletBinding()]
param([Parameter(Mandatory=$true)][string]$Executable,
 [string]$EngineDirectory=(Join-Path $env:ProgramFiles 'C2L-Monitor\engine'),
 [string]$Database=(Join-Path $env:ProgramFiles 'C2L-Monitor\database'),
 [string]$PythonExecutable='python')
$ErrorActionPreference='Stop'
$root=Join-Path ([IO.Path]::GetTempPath()) ('c2l-auto-update-'+[guid]::NewGuid().ToString('N'))
$install=Join-Path $root 'install'; $data=Join-Path $root 'data'; $watch=Join-Path $root 'watch'
foreach ($path in @($install,$data,$watch)) {New-Item -ItemType Directory -Path $path -Force | Out-Null}
Copy-Item -LiteralPath $EngineDirectory -Destination (Join-Path $install 'engine') -Recurse
Copy-Item -LiteralPath $Database -Destination (Join-Path $install 'database') -Recurse
@{watch_root=$watch;automatic_updates=$true} | ConvertTo-Json | Set-Content (Join-Path $install 'service.json') -Encoding utf8
Set-Content -LiteralPath (Join-Path $watch 'clean.txt') -Value 'Benign automatic update acceptance'
$exe=(Resolve-Path -LiteralPath $Executable).Path
$job=Start-Process -FilePath $exe -ArgumentList @('--service-worker',('"'+$install+'"'),('"'+$data+'"'),'120') -WindowStyle Hidden -PassThru
Write-Output "Running isolated automatic-update acceptance: $root"
if (-not $job.WaitForExit(150000)) { $job.Kill(); throw 'Owned test worker exceeded its deadline' }
if ($job.ExitCode -ne 0) {throw "Worker failed; retained evidence: $root"}
$state=& $PythonExecutable -c 'import sqlite3,sys; c=sqlite3.connect(sys.argv[1]); print(c.execute(sys.argv[2]).fetchone()[0])' (Join-Path $data 'status.sqlite') 'SELECT json FROM status WHERE id=1'
if ($LASTEXITCODE -ne 0) {throw 'Unable to inspect test state'}
$s=$state | ConvertFrom-Json
if ($s.update_success -le 0 -or $s.update_next -le $s.update_success -or $s.scans -lt 2 -or $s.errors -ne 0) {throw "Automatic update/reconciliation failed: $state"}
$generations=@(Get-ChildItem -LiteralPath (Join-Path $data 'definitions') -Directory -Filter 'generation-*')
if ($generations.Count -ne 1) {throw 'Unexpected duplicate update within six-hour interval'}
Write-Output "PASS: automatic publication, post-update reconciliation, single scheduled update; evidence retained: $root"
