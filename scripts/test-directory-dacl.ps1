# Reversible ACL regression in an isolated temporary folder; no administrator required.
$ErrorActionPreference='Stop'
. (Join-Path $PSScriptRoot 'directory-dacl.ps1')
function Get-EffectiveDescriptor($Acl) {
 # SetFileSecurity normalizes SE_DACL_AUTO_INHERITED. Compare the complete
 # ordered ACE bytes, owner/group and inheritance protection, not that hint.
 $raw=[Security.AccessControl.RawSecurityDescriptor]::new($Acl.Sddl)
 $bytes=New-Object byte[] $raw.DiscretionaryAcl.BinaryLength
 $raw.DiscretionaryAcl.GetBinaryForm($bytes,0)
 return ($raw.Owner.Value+'|'+$raw.Group.Value+'|'+$Acl.AreAccessRulesProtected+'|'+[Convert]::ToBase64String($bytes))
}
$root=Join-Path ([IO.Path]::GetTempPath()) ('c2l-dacl-test-'+[guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path (Join-Path $root 'child') | Out-Null
try {
 foreach ($protected in @($false,$true)) {
 $setup=Get-Acl -LiteralPath $root
 $setup.SetAccessRuleProtection($protected,$true)
 Set-Acl -LiteralPath $root -AclObject $setup
 $original=Get-Acl -LiteralPath $root
 $childBefore=(Get-Acl -LiteralPath (Join-Path $root 'child')).Sddl
 $acl=Get-Acl -LiteralPath $root
 $sid=[Security.Principal.SecurityIdentifier]::new('S-1-5-19')
 $rule=[Security.AccessControl.FileSystemAccessRule]::new($sid,'ReadAttributes, Traverse','None','None','Allow')
 $acl.AddAccessRule($rule)
 Set-C2LDirectoryDacl $root $acl
 $rootAfter=Get-Acl -LiteralPath $root
 $found=@($rootAfter.GetAccessRules($true,$false,[Security.Principal.SecurityIdentifier]) | Where-Object {$_.IdentityReference.Value -eq 'S-1-5-19'})
 if ($found.Count -ne 1 -or $found[0].InheritanceFlags -ne 'None' -or ($found[0].FileSystemRights -band [Security.AccessControl.FileSystemRights]::ReadData)) {throw 'Unexpected grant scope.'}
 if ((Get-Acl -LiteralPath (Join-Path $root 'child')).Sddl -ne $childBefore) {throw 'Child ACL unexpectedly changed.'}
 Set-C2LDirectoryDacl $root $original
 if ((Get-EffectiveDescriptor (Get-Acl -LiteralPath $root)) -ne (Get-EffectiveDescriptor $original)) {throw 'DACL restoration failed.'}
 if ((Get-Acl -LiteralPath (Join-Path $root 'child')).Sddl -ne $childBefore) {throw 'Restoration changed child ACL.'}
 }
 Write-Output 'PASS: directory-only attributes/traversal grant, child ACL unchanged, original restored.'
} finally {
 $full=[IO.Path]::GetFullPath($root)
 $temp=[IO.Path]::GetFullPath([IO.Path]::GetTempPath()).TrimEnd('\')+'\'
 if (-not $full.StartsWith($temp+'c2l-dacl-test-',[StringComparison]::OrdinalIgnoreCase)) {throw 'Cleanup containment check failed.'}
 Remove-Item -LiteralPath $full -Recurse -Force
}
