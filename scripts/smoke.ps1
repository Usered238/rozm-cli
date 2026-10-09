param([string]$Binary = "$PSScriptRoot/../dist/win32/rozm-cli.exe")
$ErrorActionPreference = 'Stop'
$Binary = (Resolve-Path -LiteralPath $Binary).Path
$resultDir = "$PSScriptRoot/../target/shell-smoke-win32"
[void](New-Item -ItemType Directory -Force -Path $resultDir)
$outputFile = Join-Path $resultDir 'quoted-newline.wav'
$a = [string][char]0x430
$text = '"' + $a + '"' + "`n" + $a
& $Binary --text $text --output $outputFile
if ($LASTEXITCODE -ne 0) { throw 'PowerShell CLI smoke failed' }
$actual = [System.IO.File]::ReadAllBytes($outputFile)
$golden = [System.IO.File]::ReadAllBytes("$PSScriptRoot/../tests/fixtures/original-a-v1-s5.wav")
$length = 2 * ($golden.Length - 44)
if ($actual.Length -ne $length + 44) { throw 'PowerShell newline speech length differs' }
for ($i = 0; $i -lt $length; $i++) {
    if ($actual[$i + 44] -ne $golden[44 + ($i % ($golden.Length - 44))]) { throw "PCM differs at $i" }
}
Write-Output 'PowerShell quoted text and real newline: PASS'
