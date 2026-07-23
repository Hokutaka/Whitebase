@echo off
setlocal

powershell.exe ^
    -NoProfile ^
    -ExecutionPolicy Bypass ^
    -File "%~dp0export-tree.ps1"

if errorlevel 1 (
    echo Failed to export project tree.
    exit /b 1
)

endlocal