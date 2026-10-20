# Harmless official ClamAV fixture; never executed, never an AV exclusion.
[CmdletBinding()]
param()
$ErrorActionPreference = 'Stop'
$testRoot = Join-Path $env:LOCALAPPDATA 'C2L-Antivirus\engine\validation'
New-Item -ItemType Directory -Force $testRoot | Out-Null
$encoded = Join-Path $testRoot 'clam.exe.xor'
Invoke-WebRequest 'https://raw.githubusercontent.com/Cisco-Talos/clamav/main/unit_tests/input/clamav_hdb_scanfiles/clam.exe.xor' -OutFile $encoded
# Upstream xor_testfile.py key, used solely to prepare this benign vendor test fixture.
$key = [Text.Encoding]::ASCII.GetBytes('bhcftqarohcdiayfohalohkgmoefxrrgfnczssgybajvkzjaahpfrlqsratkhhfvpxytculmwgmtyzujlbjlgrhtwxhzpjazlibbwepffyjyfkjwzyofgpopoueurinpdujkphxwhnaxfkaiwrpzdqsnwughtejr')
$bytes = [IO.File]::ReadAllBytes($encoded)
for ($i = 0; $i -lt $bytes.Length; $i++) { $bytes[$i] = $bytes[$i] -bxor $key[$i % $key.Length] }
$sha = [Security.Cryptography.SHA256]::Create()
$actual = [BitConverter]::ToString($sha.ComputeHash($bytes)).Replace('-', '').ToLowerInvariant()
$sha.Dispose()
if ($actual -ne '71e7b604d18aefd839e51a39c88df8383bb4c071dc31f87f00a2b5df580d4495') { throw 'Vendor fixture changed: review required; no decoded file written.' }
[IO.File]::WriteAllBytes((Join-Path $testRoot 'clam-test.bin'), $bytes)
Write-Output 'Official harmless fixture prepared. Expected detection: Clamav.Test.File. Never execute this file.'
