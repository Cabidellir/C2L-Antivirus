# Install the experimental read-only service from an elevated PowerShell.
# Only the explicit WatchPath can optionally receive a read grant. No write grant/remediation.
[CmdletBinding()]
param(
    [Parameter(Mandatory=$true)][string]$Executable,
    [Parameter(Mandatory=$true)][string]$WatchPath,
    [Parameter(Mandatory=$true)][string]$Database,
    [string]$EngineArchive = (Join-Path $env:LOCALAPPDATA 'C2L-Antivirus\engine\clamav-1.5.4.win.x64.zip'),
    [switch]$GrantReadAccess,
    [switch]$DisableAutomaticUpdates,
    [switch]$RequireSigned
)
$ErrorActionPreference='Stop'
$identity=[Security.Principal.WindowsIdentity]::GetCurrent()
if (-not ([Security.Principal.WindowsPrincipal]$identity).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)) {throw 'Open PowerShell as administrator (UAC) to install the Windows service.'}
$install=Join-Path $env:ProgramFiles 'C2L-Monitor'
$data=Join-Path $env:ProgramData 'C2L-Monitor'
function Assert-LocalPlainPath([string]$Path) {
    $full=[IO.Path]::GetFullPath($Path)
    if ($full.StartsWith('\\') -or $full.Contains('"')) {throw 'Only plain local paths are accepted.'}
    $item=Get-Item -LiteralPath $full -Force
    while ($null -ne $item) {
        if (($item.Attributes -band [IO.FileAttributes]::ReparsePoint) -ne 0) {throw "Reparse point rejected: $($item.FullName)"}
        if ($item -is [IO.DirectoryInfo]) {$item=$item.Parent} else {$item=$item.Directory}
    }
    return $full
}
$source=Assert-LocalPlainPath $Executable
$watch=Assert-LocalPlainPath $WatchPath
$databaseSource=Assert-LocalPlainPath $Database
$archive=Assert-LocalPlainPath $EngineArchive
if (-not (Test-Path -LiteralPath $watch -PathType Container) -or $watch.TrimEnd('\') -eq [IO.Path]::GetPathRoot($watch).TrimEnd('\')) {throw 'Choose a dedicated folder, not an entire drive.'}
foreach ($reserved in @($install,$data,$env:ProgramFiles,$env:windir)) {
    if ($watch.Equals($reserved,[StringComparison]::OrdinalIgnoreCase) -or $reserved.StartsWith($watch.TrimEnd('\')+'\',[StringComparison]::OrdinalIgnoreCase) -or $watch.StartsWith($reserved.TrimEnd('\')+'\',[StringComparison]::OrdinalIgnoreCase)) {throw 'Protected installation/system folders cannot be watched.'}
}
if (Get-Service -Name C2LMonitor -ErrorAction SilentlyContinue) {throw 'Service already exists. Refusing to change an existing installation.'}
if ((Test-Path -LiteralPath $install) -or (Test-Path -LiteralPath $data)) {throw 'Installation/data directory already exists. Refusing to overwrite.'}
if ($RequireSigned -and (Get-AuthenticodeSignature -LiteralPath $source).Status -ne 'Valid') {throw 'A trusted code signature is required.'}
if ((Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant() -ne '0d9e0228b2674137ea1a2853566c98a0278ad52ab2582c3d6dbd75373848c395') {throw 'Official ClamAV archive checksum mismatch.'}
$databaseFiles=@('main.cvd','main.cld','daily.cvd','daily.cld','bytecode.cvd','bytecode.cld') | Where-Object {Test-Path -LiteralPath (Join-Path $databaseSource $_)}
if (-not ($databaseFiles -contains 'daily.cvd' -or $databaseFiles -contains 'daily.cld')) {throw 'Daily database missing.'}
function Set-PrivateDirectory([string]$Path, [string]$ServiceSid, [bool]$Writable) {
    $acl=New-Object Security.AccessControl.DirectorySecurity
    $acl.SetAccessRuleProtection($true,$false)
    $acl.SetOwner([Security.Principal.SecurityIdentifier]::new('S-1-5-32-544'))
    foreach ($sid in @('S-1-5-18','S-1-5-32-544')) {
        $acl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new([Security.Principal.SecurityIdentifier]::new($sid),'FullControl','ContainerInherit,ObjectInherit','None','Allow'))
    }
    $rights=if ($Writable) {'Modify'} else {'ReadAndExecute'}
    $acl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new([Security.Principal.SecurityIdentifier]::new($ServiceSid),$rights,'ContainerInherit,ObjectInherit','None','Allow'))
    # The installing user can view status and reports, but cannot change service configuration.
    $acl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new($identity.User,'ReadAndExecute','ContainerInherit,ObjectInherit','None','Allow'))
    Set-Acl -LiteralPath $Path -AclObject $acl
}
New-Item -ItemType Directory -Path $install | Out-Null
New-Item -ItemType Directory -Path $data | Out-Null
# First secure both roots to admins/SYSTEM. Service grants are added after registration.
foreach ($path in @($install,$data)) {
    $acl=New-Object Security.AccessControl.DirectorySecurity
    $acl.SetAccessRuleProtection($true,$false)
    $acl.SetOwner([Security.Principal.SecurityIdentifier]::new('S-1-5-32-544'))
    foreach ($sid in @('S-1-5-18','S-1-5-32-544')) {$acl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new([Security.Principal.SecurityIdentifier]::new($sid),'FullControl','ContainerInherit,ObjectInherit','None','Allow'))}
    Set-Acl -LiteralPath $path -AclObject $acl
}
$sourceHash=(Get-FileHash -LiteralPath $source -Algorithm SHA256).Hash
Copy-Item -LiteralPath $source -Destination (Join-Path $install 'c2l-desktop.exe')
if ((Get-FileHash -LiteralPath (Join-Path $install 'c2l-desktop.exe') -Algorithm SHA256).Hash -ne $sourceHash) {throw 'Executable changed while copying.'}
if ($RequireSigned -and (Get-AuthenticodeSignature -LiteralPath (Join-Path $install 'c2l-desktop.exe')).Status -ne 'Valid') {throw 'Installed code signature invalid.'}
$protectedArchive=Join-Path $install 'engine.zip'
Copy-Item -LiteralPath $archive -Destination $protectedArchive
if ((Get-FileHash -LiteralPath $protectedArchive -Algorithm SHA256).Hash.ToLowerInvariant() -ne '0d9e0228b2674137ea1a2853566c98a0278ad52ab2582c3d6dbd75373848c395') {throw 'Copied engine archive checksum mismatch.'}
$unpack=Join-Path $install 'vendor'
Expand-Archive -LiteralPath $protectedArchive -DestinationPath $unpack
$vendor=Join-Path $unpack 'clamav-1.5.4.win.x64'
Copy-Item -LiteralPath $vendor -Destination (Join-Path $install 'engine') -Recurse
New-Item -ItemType Directory -Path (Join-Path $install 'database') | Out-Null
foreach ($name in $databaseFiles) {
    $file=Assert-LocalPlainPath (Join-Path $databaseSource $name)
    Copy-Item -LiteralPath $file -Destination (Join-Path $install "database\$name")
}
@{watch_root=$watch; automatic_updates=(-not $DisableAutomaticUpdates)} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $install 'service.json') -Encoding utf8
$credential=[pscredential]::new('NT AUTHORITY\LocalService',[Security.SecureString]::new())
$binary='"'+(Join-Path $install 'c2l-desktop.exe')+'" --service'
New-Service -Name C2LMonitor -DisplayName 'C2L — análise contínua experimental' -BinaryPathName $binary -StartupType Automatic -Credential $credential -Description 'Read-only post-change scanning of one configured folder. No pre-execution blocking.' | Out-Null
& sc.exe sidtype C2LMonitor unrestricted | Out-Null
if ($LASTEXITCODE -ne 0) {throw 'Unable to enable service SID. Service was not started.'}
$serviceSid=([Security.Principal.NTAccount]::new('NT SERVICE\C2LMonitor')).Translate([Security.Principal.SecurityIdentifier]).Value
Set-PrivateDirectory $install $serviceSid $false
Set-PrivateDirectory $data $serviceSid $true
if ($GrantReadAccess) {
    $acl=Get-Acl -LiteralPath $watch
    $acl.AddAccessRule([Security.AccessControl.FileSystemAccessRule]::new([Security.Principal.SecurityIdentifier]::new($serviceSid),'ReadAndExecute','ContainerInherit,ObjectInherit','None','Allow'))
    Set-Acl -LiteralPath $watch -AclObject $acl
}
@{watch_root=$watch; read_grant=[bool]$GrantReadAccess; service_sid=$serviceSid; installed_utc=[DateTime]::UtcNow.ToString('o')} | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $install 'installation.json') -Encoding utf8
& sc.exe failure C2LMonitor reset= 86400 actions= restart/60000/restart/120000/restart/300000 | Out-Null
if ($LASTEXITCODE -ne 0) {throw 'Could not configure service recovery.'}
& sc.exe failureflag C2LMonitor 1 | Out-Null
if ($LASTEXITCODE -ne 0) {throw 'Could not enable recovery after reported service failure.'}
foreach ($path in @($install,$data)) {
    & icacls.exe $path /setowner '*S-1-5-32-544' /T | Out-Null
    if ($LASTEXITCODE -ne 0) {throw 'Unable to protect ownership of service files.'}
}
Start-Service -Name C2LMonitor
Write-Output "Installed. Check Monitorização in the GUI: SCM running alone does not prove healthy scanning. Target: $watch. No file write grants; no automatic quarantine. Official signatures update automatically unless explicitly disabled."
