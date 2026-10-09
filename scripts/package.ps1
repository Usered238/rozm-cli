param(
    [string]$Binary = "$PSScriptRoot/../target/i686-pc-windows-msvc/release/rozm-cli.exe",
    [string]$Destination = "$PSScriptRoot/../dist/win32",
    [string]$DataRoot = "$PSScriptRoot/../orig"
)
$ErrorActionPreference = 'Stop'
$files = @('BSNbn', 'skfs/skf', 'skfs/wif', 'skfs/wlf', 'snf1/ip2f', 'snf1/lp2f', 'snf1/Sd2f', 'snf2/ip2f', 'snf2/lp2f', 'snf2/Sd2f', 'snf3/ip2f', 'snf3/lp2f', 'snf3/Sd2f')
if (!(Test-Path -LiteralPath $Binary -PathType Leaf)) { throw "Build the release binary first: $Binary" }
foreach ($name in $files) {
    if (!(Test-Path -LiteralPath (Join-Path $DataRoot $name) -PathType Leaf)) { throw "Missing resource: $name" }
}
[void](New-Item -ItemType Directory -Force -Path $Destination)
Copy-Item -LiteralPath $Binary -Destination (Join-Path $Destination 'rozm-cli.exe') -Force
foreach ($name in $files) {
    $targetFile = Join-Path (Join-Path $Destination 'data') $name
    [void](New-Item -ItemType Directory -Force -Path (Split-Path -Parent $targetFile))
    Copy-Item -LiteralPath (Join-Path $DataRoot $name) -Destination $targetFile -Force
}
Copy-Item -LiteralPath "$PSScriptRoot/../docs/package-readme.md" -Destination (Join-Path $Destination 'README.md') -Force
Write-Output "Packaged: $Destination"
