# Elevated acceptance followed by activation of a user-selected folder.
[CmdletBinding()]
param(
 [Parameter(Mandatory=$true)][string]$Executable,
 [Parameter(Mandatory=$true)][string]$Database,
 [Parameter(Mandatory=$true)][string]$EngineArchive,
 [Parameter(Mandatory=$true)][string]$Fixture,
 [Parameter(Mandatory=$true)][string]$WatchPath,
 [string]$PythonExecutable='python',
 [string[]]$TraversalPaths=@()
)
$ErrorActionPreference='Stop'
if (Get-Service C2LMonitor -ErrorAction SilentlyContinue) {throw 'Existing service; activation refused without changing its state.'}
$watch=[IO.Path]::GetFullPath($WatchPath)
$item=Get-Item -LiteralPath $watch -Force
if (-not $item.PSIsContainer -or $watch.StartsWith('\\') -or $watch.TrimEnd('\') -eq [IO.Path]::GetPathRoot($watch).TrimEnd('\')) {throw 'A dedicated local folder is required.'}
while ($null -ne $item) {
 if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {throw 'Reparse points are not accepted.'}
 $item=$item.Parent
}
$install=Join-Path $env:ProgramFiles 'C2L-Monitor'
$data=Join-Path $env:ProgramData 'C2L-Monitor'
foreach ($reserved in @($install,$data,$env:ProgramFiles,$env:windir)) {
 if ($watch.Equals($reserved,[StringComparison]::OrdinalIgnoreCase) -or $watch.StartsWith($reserved.TrimEnd('\')+'\',[StringComparison]::OrdinalIgnoreCase) -or $reserved.StartsWith($watch.TrimEnd('\')+'\',[StringComparison]::OrdinalIgnoreCase)) {throw 'System or service folder rejected.'}
}
try {
 & (Join-Path $PSScriptRoot 'test-monitor-service.ps1') -Executable $Executable -Database $Database -EngineArchive $EngineArchive -Fixture $Fixture -PythonExecutable $PythonExecutable -KeepInstalled
 Stop-Service C2LMonitor
 (Get-Service C2LMonitor).WaitForStatus('Stopped',[TimeSpan]::FromSeconds(45))
 $record=Get-Content -LiteralPath (Join-Path $install 'installation.json') -Raw | ConvertFrom-Json
 $sid=[Security.Principal.SecurityIdentifier]::new($record.service_sid)
 # Remove only the read rule introduced for the acceptance folder.
 if ($record.read_grant) {
  $acl=Get-Acl -LiteralPath $record.watch_root
  $acl.RemoveAccessRuleSpecific([Security.AccessControl.FileSystemAccessRule]::new($sid,'ReadAndExecute','ContainerInherit,ObjectInherit','None','Allow'))
  Set-Acl -LiteralPath $record.watch_root -AclObject $acl
 }
 $acl=Get-Acl -LiteralPath $watch
 $acl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new($sid,'ReadAndExecute','ContainerInherit,ObjectInherit','None','Allow'))
 Set-Acl -LiteralPath $watch -AclObject $acl
 # Preserve test evidence separately, so test detections never appear as Downloads detections.
 $dataAcl=Get-Acl -LiteralPath $data
 $archive=$data+'-acceptance-'+[DateTime]::UtcNow.ToString('yyyyMMddHHmmss')
 $parent=[IO.Path]::GetFullPath($env:ProgramData).TrimEnd('\')+'\'
 if ([IO.Path]::GetFullPath($data) -ne ($parent+'C2L-Monitor') -or -not [IO.Path]::GetFullPath($archive).StartsWith($parent+'C2L-Monitor-acceptance-',[StringComparison]::OrdinalIgnoreCase) -or (Test-Path -LiteralPath $archive)) {throw 'Archive destination verification failed.'}
 Move-Item -LiteralPath $data -Destination $archive
 New-Item -ItemType Directory -Path $data | Out-Null
 Set-Acl -LiteralPath $data -AclObject $dataAcl
 @{watch_root=$watch} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $install 'service.json') -Encoding utf8
 $record.watch_root=$watch; $record.read_grant=$true
 $record | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $install 'installation.json') -Encoding utf8
 & icacls.exe $install /setowner '*S-1-5-32-544' /T | Out-Null
 if ($LASTEXITCODE -ne 0) {throw 'Ownership verification failed.'}
 if ($TraversalPaths.Count) { & (Join-Path $PSScriptRoot 'grant-monitor-traversal.ps1') -Path $TraversalPaths }
 Start-Service C2LMonitor
 Write-Output "ACTIVATED: $watch; acceptance evidence: $archive. Confirm initial scan status in the GUI."
} catch {
 if (Get-Service C2LMonitor -ErrorAction SilentlyContinue) {
  Stop-Service C2LMonitor -ErrorAction SilentlyContinue
  Set-Service C2LMonitor -StartupType Manual
 }
 throw
}
