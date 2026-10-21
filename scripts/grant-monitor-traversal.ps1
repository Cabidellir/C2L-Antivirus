# Grant only directory attributes/traversal on explicit ancestors; no listing, file read or inheritance.
[CmdletBinding()]
param([Parameter(Mandatory=$true)][string[]]$Path)
$ErrorActionPreference='Stop'
$identity=[Security.Principal.WindowsIdentity]::GetCurrent()
if (-not ([Security.Principal.WindowsPrincipal]$identity).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {throw 'Administrator/UAC required.'}
. (Join-Path $PSScriptRoot 'directory-dacl.ps1')
$recordPath=Join-Path $env:ProgramFiles 'C2L-Monitor\installation.json'
$record=Get-Content -LiteralPath $recordPath -Raw | ConvertFrom-Json
$sid=([Security.Principal.NTAccount]::new('NT SERVICE\C2LMonitor')).Translate([Security.Principal.SecurityIdentifier])
$watch=[IO.Path]::GetFullPath($record.watch_root).TrimEnd('\')
$granted=@($record.traversal_grants | Where-Object {$_})
foreach ($candidate in $Path) {
 $full=[IO.Path]::GetFullPath($candidate).TrimEnd('\')
 if ($full.StartsWith('\\') -or $full -eq [IO.Path]::GetPathRoot($full).TrimEnd('\') -or -not $watch.StartsWith($full+'\',[StringComparison]::OrdinalIgnoreCase)) {throw 'Only a non-volume ancestor of the configured target is accepted.'}
 $item=Get-Item -LiteralPath $full -Force
 if (-not $item.PSIsContainer -or ($item.Attributes -band [IO.FileAttributes]::ReparsePoint)) {throw 'Plain directory required.'}
 $acl=Get-Acl -LiteralPath $full
 $rule=[Security.AccessControl.FileSystemAccessRule]::new($sid,'ReadAttributes, Traverse','None','None','Allow')
 $acl.AddAccessRule($rule)
 Write-Output "Applying directory-only traversal rule: $full"
 Set-C2LDirectoryDacl $full $acl
 $granted+= $full
 # Persist each successful grant so a later path failure remains reversible.
 $record | Add-Member -NotePropertyName traversal_grants -NotePropertyValue @($granted | Select-Object -Unique) -Force
 $record | ConvertTo-Json | Set-Content -LiteralPath $recordPath -Encoding utf8
}
$record | Add-Member -NotePropertyName traversal_grants -NotePropertyValue @($granted | Select-Object -Unique) -Force
$record | ConvertTo-Json | Set-Content -LiteralPath $recordPath -Encoding utf8
Write-Output 'Granted non-inherited directory attributes/traversal only; no listing or file-content access on ancestors.'
