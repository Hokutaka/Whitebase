@echo off
chcp 65001 >nul
setlocal

rem tools\control-panel.bat からリポジトリルートへ移動
pushd "%~dp0.." || exit /b 1

set "PROJECT=tools\whitebase-control-panel\Whitebase.ControlPanel\Whitebase.ControlPanel\Whitebase.ControlPanel.csproj"

if not exist "%PROJECT%" (
    echo [ERROR] Control Panel project was not found.
    echo         %CD%\%PROJECT%
    pause
    popd
    exit /b 1
)

echo Starting Whitebase Control Panel...
dotnet run --project "%PROJECT%"

set "EXIT_CODE=%ERRORLEVEL%"

if not "%EXIT_CODE%"=="0" (
    echo.
    echo [ERROR] Control Panel exited with code %EXIT_CODE%.
    pause
)

popd
exit /b %EXIT_CODE%