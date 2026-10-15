@echo off
setlocal enableextensions
cd /d "%~dp0.."
if errorlevel 1 exit /b 1

echo [C2L] Running Rust workspace tests...
cargo test --workspace --locked
if errorlevel 1 exit /b 1

echo [C2L] Checking demo indicator matching...
set "MATCH_FOUND="
for /f "delims=" %%L in ('cargo run --quiet --locked -p c2l-cli -- scan "tests\fixtures\c2l-demo.txt" --indicators "tests\fixtures\demo-indicators.csv"') do (
    echo %%L
    echo %%L | findstr /C:"Indicator: MATCH (C2L_DEMO_ONLY)" >nul && set "MATCH_FOUND=1"
)
if not defined MATCH_FOUND (
    echo [C2L] ERROR: demo indicator did not match.
    exit /b 1
)

echo [C2L] Checking real Windows file...
if not exist "%WINDIR%\notepad.exe" (
    echo [C2L] ERROR: Windows Notepad executable not found.
    exit /b 1
)
cargo run --quiet --locked -p c2l-cli -- scan "%WINDIR%\notepad.exe"
if errorlevel 1 exit /b 1

echo [C2L] Checking recursive scanning...
cargo run --quiet --locked -p c2l-cli -- scan-dir "tests\fixtures" --indicators "tests\fixtures\demo-indicators.csv"
if errorlevel 1 exit /b 1

echo [C2L] Windows smoke tests passed.
exit /b 0
