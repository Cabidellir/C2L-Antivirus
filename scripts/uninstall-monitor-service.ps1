# Elevated removal; keep reports, binaries and signature snapshot for recovery/audit.
[CmdletBinding()]
param([switch]$RemoveReadGrant)
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'directory-dacl.ps1')
$identity=[Security.Principal.WindowsIdentity]::GetCurrent()
if (-not ([Security.Principal.WindowsPrincipal]$identity).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {throw 'Run as administrator (UAC).'}
$install=Join-Path $env:ProgramFiles 'C2L-Monitor'
$service=Get-Service -Name C2LMonitor -ErrorAction Stop
Stop-Service -InputObject $service
$service.WaitForStatus('Stopped',[TimeSpan]::FromSeconds(45))
if ($RemoveReadGrant) {
    $record=Get-Content -LiteralPath (Join-Path $install 'installation.json') -Raw | ConvertFrom-Json
    foreach ($parent in @($record.traversal_grants | Where-Object {$_})) {
        $acl=Get-Acl -LiteralPath $parent
        $rule=[Security.AccessControl.FileSystemAccessRule]::new([Security.Principal.SecurityIdentifier]::new($record.service_sid),'ReadAttributes, Traverse','None','None','Allow')
        $acl.RemoveAccessRuleSpecific($rule)
        Set-C2LDirectoryDacl $parent $acl
    }
    if ($record.read_grant) {
        $acl=Get-Acl -LiteralPath $record.watch_root
        $rule=[Security.AccessControl.FileSystemAccessRule]::new([Security.Principal.SecurityIdentifier]::new($record.service_sid),'ReadAndExecute','ContainerInherit,ObjectInherit','None','Allow')
        $acl.RemoveAccessRuleSpecific($rule)
        Set-Acl -LiteralPath $record.watch_root -AclObject $acl
    }
}
& sc.exe delete C2LMonitor
if ($LASTEXITCODE -ne 0) {throw 'Service deletion failed.'}
Write-Output 'Service removed. Binaries and reports preserved. Use -RemoveReadGrant to remove the optional target-folder read rule during removal.'
