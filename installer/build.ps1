# Build ccvm Windows installer
# Prerequisites: Rust toolchain, Inno Setup (https://jrsoftware.org/isinfo.php)

$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot\..

# 1. Read version from Cargo.toml
$toml = Get-Content Cargo.toml -Raw
$version = if ($toml -match 'version\s*=\s*"([^"]+)"') { $matches[1] } else { throw "could not parse version from Cargo.toml" }
Write-Host "version: $version"

# 2. Build release binaries
Write-Host "building release binaries..."
cargo build --release
if ($LASTEXITCODE -ne 0) { throw "cargo build --release failed" }

# 3. Locate Inno Setup compiler
$iscc = $null
$candidates = @(
    "${env:ProgramFiles(x86)}\Inno Setup 6\ISCC.exe",
    "$env:ProgramFiles\Inno Setup 6\ISCC.exe"
)
foreach ($c in $candidates) {
    if (Test-Path $c) { $iscc = $c; break }
}
if (-not $iscc) {
    throw "ISCC.exe not found. Install Inno Setup from https://jrsoftware.org/isinfo.php"
}
Write-Host "iscc: $iscc"

# 4. Generate installer
$outputDir = Join-Path (Get-Location) "target\release"
Write-Host "generating installer to $outputDir ..."
& $iscc /Qp /DVersion="$version" /O"$outputDir" installer\ccvm.iss
if ($LASTEXITCODE -ne 0) { throw "ISCC failed" }

$installer = Join-Path $outputDir "ccvm-setup-$version.exe"
Write-Host "`ninstaller: $installer"
Write-Host "done."
