# Apply a DACL only to the named directory. Do not propagate existing profile ACEs.
if (-not ('C2LDirectoryDacl' -as [type])) {
 Add-Type -TypeDefinition @"
using System.Runtime.InteropServices;
public static class C2LDirectoryDacl {
 [DllImport("advapi32.dll", CharSet=CharSet.Unicode, SetLastError=true)]
 public static extern bool SetFileSecurity(string path, uint information, byte[] descriptor);
}
"@
}
function Set-C2LDirectoryDacl([string]$Directory,[Security.AccessControl.DirectorySecurity]$Acl) {
 if (-not [C2LDirectoryDacl]::SetFileSecurity($Directory,4,$Acl.GetSecurityDescriptorBinaryForm())) {
  throw [ComponentModel.Win32Exception]::new([Runtime.InteropServices.Marshal]::GetLastWin32Error())
 }
}
