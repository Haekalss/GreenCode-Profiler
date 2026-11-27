# Green Profiler Windows Installer (PowerShell)
# Usage: irm https://greenprofiler.github.io/install/install.ps1 | iex

$ErrorActionPreference = "Stop"

Write-Host "==========================================================" -ForegroundColor Green
Write-Host "   Installing Green Profiler CLI (Rust Native Engine)     " -ForegroundColor Green
Write-Host "==========================================================" -ForegroundColor Green

$installDir = "$HOME\.greenprofiler\bin"
if (-not (Test-Path $installDir)) {
    New-Item -ItemType Directory -Path $installDir -Force | Out-Null
}

$zipUrl = "https://github.com/Haekalss/GreenCode-Profiler/releases/latest/download/GreenProfiler-v1.9.0-windows-x64.zip"
$tempZip = "$env:TEMP\greenprofiler.zip"
$tempExtract = "$env:TEMP\greenprofiler_extract"

Write-Host "-> Downloading latest Green Profiler release..." -ForegroundColor Cyan
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
Invoke-WebRequest -Uri $zipUrl -OutFile $tempZip -UseBasicParsing

Write-Host "-> Extracting executable..." -ForegroundColor Cyan
if (Test-Path $tempExtract) { Remove-Item -Path $tempExtract -Recurse -Force }
Expand-Archive -Path $tempZip -DestinationPath $tempExtract -Force

$exeSource = Get-ChildItem -Path $tempExtract -Filter "greenprofiler.exe" -Recurse | Select-Object -First 1
if (-not $exeSource) {
    Write-Host "[ERROR] Binary executable not found in archive." -ForegroundColor Red
    exit 1
}

Copy-Item -Path $exeSource.FullName -Destination "$installDir\greenprofiler.exe" -Force
Copy-Item -Path $exeSource.FullName -Destination "$installDir\gp.exe" -Force

# Clean temporary files
Remove-Item -Path $tempZip -Force -ErrorAction SilentlyContinue
Remove-Item -Path $tempExtract -Recurse -Force -ErrorAction SilentlyContinue

# Check PATH
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -split ';' -notcontains $installDir) {
    [Environment]::SetEnvironmentVariable("Path", "$userPath;$installDir", "User")
    $env:Path = "$env:Path;$installDir"
    Write-Host "-> Added $installDir to User PATH." -ForegroundColor Yellow
}

Write-Host ""
Write-Host "Green Profiler v1.9.0 successfully installed to: $installDir" -ForegroundColor Green
Write-Host "Commands available: 'greenprofiler' or 'gp' (alias)" -ForegroundColor Cyan
Write-Host "Website: https://greenprofiler.github.io" -ForegroundColor Gray
