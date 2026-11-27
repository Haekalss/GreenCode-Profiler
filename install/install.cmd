@echo off
rem Green Profiler Windows Installer (CMD)
rem Usage: curl -fsSL https://greenprofiler.github.io/install/install.cmd -o install.cmd && install.cmd && del install.cmd

echo ==========================================================
echo    Installing Green Profiler CLI (Rust Native Engine)
echo ==========================================================

set "INSTALL_DIR=%USERPROFILE%\.greenprofiler\bin"
if not exist "%INSTALL_DIR%" mkdir "%INSTALL_DIR%"

set "ZIP_URL=https://github.com/Haekalss/GreenCode-Profiler/releases/latest/download/GreenProfiler-v1.9.0-windows-x64.zip"
set "TEMP_ZIP=%TEMP%\greenprofiler.zip"
set "TEMP_DIR=%TEMP%\greenprofiler_extract"

echo [1/3] Downloading Green Profiler...
powershell -NoProfile -ExecutionPolicy Bypass -Command "[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12; Invoke-WebRequest -Uri '%ZIP_URL%' -OutFile '%TEMP_ZIP%' -UseBasicParsing"

echo [2/3] Extracting archive...
powershell -NoProfile -ExecutionPolicy Bypass -Command "if (Test-Path '%TEMP_DIR%') { Remove-Item -Path '%TEMP_DIR%' -Recurse -Force }; Expand-Archive -Path '%TEMP_ZIP%' -DestinationPath '%TEMP_DIR%' -Force; $src = (Get-ChildItem -Path '%TEMP_DIR%' -Filter 'greenprofiler.exe' -Recurse | Select-Object -First 1).FullName; Copy-Item $src '%INSTALL_DIR%\greenprofiler.exe' -Force; Copy-Item $src '%INSTALL_DIR%\gp.exe' -Force; Remove-Item '%TEMP_ZIP%' -Force; Remove-Item '%TEMP_DIR%' -Recurse -Force"

echo [3/3] Configuring Environment PATH...
powershell -NoProfile -ExecutionPolicy Bypass -Command "$p = [Environment]::GetEnvironmentVariable('Path', 'User'); if ($p -split ';' -notcontains '%INSTALL_DIR%') { [Environment]::SetEnvironmentVariable('Path', $p + ';%INSTALL_DIR%', 'User') }"

echo.
echo Green Profiler installed successfully to %INSTALL_DIR%!
echo Restart your terminal and run 'greenprofiler' or 'gp' (alias) to begin.
echo Website: https://greenprofiler.github.io
