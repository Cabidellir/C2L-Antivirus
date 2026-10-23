# Per-user, non-elevated companion. Windows Run starts it on this user's next logon.
[CmdletBinding()]
param([switch]$Disable, [string]$Executable=(Join-Path $env:ProgramFiles 'C2L-Monitor\c2l-desktop.exe'))
$ErrorActionPreference='Stop'
$root=Join-Path $env:LOCALAPPDATA 'C2L-Antivirus'
New-Item -ItemType Directory -Path $root -Force | Out-Null
$key='HKCU:\Software\Microsoft\Windows\CurrentVersion\Run'
New-Item -Path $key -Force | Out-Null
if ($Disable) {
 Set-Content -LiteralPath (Join-Path $root 'notifications.disabled') -Value 'disabled'
 Remove-ItemProperty -Path $key -Name C2LNotificationAgent -ErrorAction SilentlyContinue
 Write-Output 'Notifications disabled for this user; scanning service remains active.'
} else {
 $exe=(Resolve-Path -LiteralPath $Executable).Path
 if ($exe.Contains('"')) {throw 'Invalid executable path'}
 $disabled=Join-Path $root 'notifications.disabled'
 if (Test-Path -LiteralPath $disabled) {Remove-Item -LiteralPath $disabled}
 Set-ItemProperty -Path $key -Name C2LNotificationAgent -Value ('"'+$exe+'" --notification-agent')
 Start-Process -FilePath $exe -ArgumentList '--notification-agent' -WindowStyle Hidden
 Write-Output 'Notification companion started; enabled for this user at logon. Windows notification settings may suppress display.'
}
