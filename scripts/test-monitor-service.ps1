# Elevated acceptance for a disposable Windows machine. Never runs malware.
[CmdletBinding()]
param([Parameter(Mandatory=$true)][string]$Executable,[Parameter(Mandatory=$true)][string]$Database,[Parameter(Mandatory=$true)][string]$EngineArchive,[Parameter(Mandatory=$true)][string]$Fixture,[string]$PythonExecutable="python",[switch]$KeepInstalled)
$ErrorActionPreference='Stop'
if (Get-Service C2LMonitor -ErrorAction SilentlyContinue) {throw 'Existing service; acceptance refused.'}
Get-Command $PythonExecutable -ErrorAction Stop | Out-Null
$watch=Join-Path $env:PUBLIC ('C2L-Service-Test-'+[guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $watch | Out-Null
Set-Content -LiteralPath (Join-Path $watch 'clean.txt') -Value 'Benign C2L service acceptance'
if ((Get-FileHash -LiteralPath $Fixture -Algorithm SHA256).Hash.ToLowerInvariant() -ne '71e7b604d18aefd839e51a39c88df8383bb4c071dc31f87f00a2b5df580d4495') {throw 'Only the pinned harmless official fixture is accepted.'}
function Read-State {
    $db=Join-Path $env:ProgramData 'C2L-Monitor\status.sqlite'
    $json=& $PythonExecutable -c 'import sqlite3,sys; c=sqlite3.connect(sys.argv[1]); print(c.execute(sys.argv[2]).fetchone()[0])' $db 'SELECT json FROM status WHERE id=1'
    if ($LASTEXITCODE -ne 0) {throw 'Status read failed'}
    return ($json | ConvertFrom-Json)
}
function Wait-State([scriptblock]$Predicate) {
    $deadline=[DateTime]::UtcNow.AddSeconds(120)
    do {
        Start-Sleep -Seconds 2
        try {$state=Read-State; if (& $Predicate $state) {return $state}} catch {}
    } while ([DateTime]::UtcNow -lt $deadline)
    throw 'Expected service state not observed within 120 seconds.'
}
& (Join-Path $PSScriptRoot 'install-monitor-service.ps1') -Executable $Executable -Database $Database -EngineArchive $EngineArchive -WatchPath $watch -GrantReadAccess
try {
    $initial=Wait-State {param($s) $s.scans -ge 1 -and $s.phase -eq 'a observar' -and $s.detections -eq 0}
    $service=Get-CimInstance Win32_Service -Filter "Name='C2LMonitor'"
    if ($service.StartName -notmatch 'LocalService') {throw 'Unexpected service account.'}
    Copy-Item -LiteralPath $Fixture -Destination (Join-Path $watch 'vendor-test.bin')
    $detected=Wait-State {param($s) $s.detections -gt 0 -and $s.scans -gt $initial.scans}
    Stop-Service C2LMonitor
    (Get-Service C2LMonitor).WaitForStatus('Stopped',[TimeSpan]::FromSeconds(45))
    if ((Read-State).phase -ne 'parado') {throw 'Stop not persisted.'}
    Start-Service C2LMonitor
    $restarted=Wait-State {param($s) $s.scans -gt $detected.scans -and $s.detections -ge $detected.detections}
    Write-Output "PASS: SCM startup, LocalService, change detection without GUI, stop, restart, persistent history. Watch=$watch"
} finally {
    if (-not $KeepInstalled) { & (Join-Path $PSScriptRoot 'uninstall-monitor-service.ps1') -RemoveReadGrant }
}
Write-Output 'Test reports and files retained. Reboot acceptance must be performed separately.'
