param(
    [string]$Version = "latest"
)

$repo = "kongdeju/ccvm"

if ($Version -eq "latest") {
    $url = "https://github.com/$repo/releases/latest/download/ccvm-setup.exe"
} else {
    $url = "https://github.com/$repo/releases/download/v$Version/ccvm-setup-$Version.exe"
}

$temp = Join-Path $env:TEMP "ccvm-setup.exe"

Write-Host "Downloading ccvm $Version..." -ForegroundColor Cyan
try {
    Invoke-WebRequest -Uri $url -OutFile $temp -UseBasicParsing -ErrorAction Stop
} catch {
    Write-Host "Download failed: $_" -ForegroundColor Red
    Write-Host "Check that the release exists at $url" -ForegroundColor Yellow
    exit 1
}

Write-Host "Running installer..." -ForegroundColor Cyan
Start-Process -FilePath $temp -Wait -NoNewWindow
Remove-Item $temp -ErrorAction SilentlyContinue

Write-Host "Done. Restart your terminal and run 'ccvm --help'." -ForegroundColor Green
