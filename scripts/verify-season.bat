@echo off
echo ================================================
echo   Gorbage Hands Season Verification Tool
echo ================================================
echo.

if "%1"=="" (
    set /p SEASON="Enter season number to verify: "
) else (
    set SEASON=%1
)

echo.
echo Verifying Season %SEASON%...
echo.

powershell -ExecutionPolicy Bypass -File "%~dp0verify-season-results.ps1" -SeasonNumber %SEASON% -OutputPath "%~dp0"

echo.
echo Done! Check the CSV files in this folder.
pause
