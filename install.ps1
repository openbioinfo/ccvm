param(
    [string]$Version = ""
)

$ErrorActionPreference = "Stop"
$repo = "openbioinfo/ccvm"

# ----- Detect architecture ------------------------------------------------
$arch = switch ([System.Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture) {
    "X64"   { "x64" }
    "Arm64" { "arm64" }
    default { throw "Unsupported architecture: $_" }
}
$artifact = "ccvm-windows-$arch"

# ----- Resolve version ----------------------------------------------------
if (-not $Version) {
    Write-Host "Fetching latest release..." -ForegroundColor Cyan
    try {
        $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$repo/releases/latest" `
            -Proxy "http://127.0.0.1:7890" `
            -ErrorAction SilentlyContinue
    } catch { }
    if (-not $release) {
        $release = Invoke-RestMethod -Uri "https://api.github.com/repos/$repo/releases/latest"
    }
    $Version = $release.tag_name -replace '^v', ''
} else {
    $Version = $Version -replace '^v', ''
}

Write-Host "Installing ccvm v$Version ($arch)..." -ForegroundColor Cyan

# ----- Download & extract -------------------------------------------------
$url = "https://github.com/$repo/releases/download/v$Version/$artifact.zip"
$tmpDir = Join-Path $env:TEMP "ccvm-install-$(Get-Random)"
New-Item -ItemType Directory -Force $tmpDir | Out-Null

$zipFile = Join-Path $tmpDir "$artifact.zip"

Write-Host "Downloading $artifact.zip..." -ForegroundColor Cyan
try {
    Invoke-WebRequest -Uri $url -OutFile $zipFile -UseBasicParsing `
        -Proxy "http://127.0.0.1:7890" `
        -ErrorAction Stop
} catch {
    Invoke-WebRequest -Uri $url -OutFile $zipFile -UseBasicParsing -ErrorAction Stop
}

Write-Host "Extracting..." -ForegroundColor Cyan
Expand-Archive -Path $zipFile -DestinationPath $tmpDir -Force

# ----- Install binaries ---------------------------------------------------
$installDir = Join-Path $env:LOCALAPPDATA "ccvm"
New-Item -ItemType Directory -Force $installDir | Out-Null

$extractedDir = $tmpDir   # zip contains files at root level
Get-ChildItem -Path $extractedDir -Filter "*.exe" | ForEach-Object {
    Copy-Item -Path $_.FullName -Destination $installDir -Force
}

# ----- Cleanup ------------------------------------------------------------
Remove-Item -Recurse -Force $tmpDir -ErrorAction SilentlyContinue

# ----- Add to PATH --------------------------------------------------------
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$installDir*") {
    Write-Host "Adding to user PATH..." -ForegroundColor Cyan
    [Environment]::SetEnvironmentVariable("Path", "$userPath;$installDir", "User")
    # Also update the current session
    $env:Path = "$env:Path;$installDir"
}

# ----- Run setup ----------------------------------------------------------
Write-Host "Running ccvm setup..." -ForegroundColor Cyan
$ccvmBin = Join-Path $installDir "ccvm.exe"
& $ccvmBin setup

Write-Host ""
Write-Host "Done! Restart your terminal and run 'ccvm --help'." -ForegroundColor Green
