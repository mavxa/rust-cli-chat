@echo off
setlocal

REM --- Parameters ---
set TOR_EXE=C:\tor\tor.exe
set TORRC=C:\vsc\vsc-projects\rust-cli-chat\torrc.example
set SERVER_BIN=target\release\server.exe
set LOCAL_PORT=9000
set HSDIR=C:\vsc\vsc-projects\rust-cli-chat\tor_hidden_service
set TOR_LOG=C:\vsc\vsc-projects\rust-cli-chat\tor.log

REM --- Check if tor.exe exists ---
if not exist "%TOR_EXE%" (
    echo Tor not found at %TOR_EXE%
    echo Download Tor Expert Bundle: https://www.torproject.org/download/tor/
    exit /b 1
)

REM --- Start Tor in background with logging ---
echo Starting Tor...
start /B "" "%TOR_EXE%" -f "%TORRC%" --Log "notice file %TOR_LOG%" 2>&1

REM --- Wait for hostname file with timeout ---
echo Waiting for .onion address (max 60s)...
set /a COUNT=0
:WAIT_HOSTNAME
if exist "%HSDIR%\hostname" (
    set /p ONION=<"%HSDIR%\hostname"
    echo Your onion address: %ONION%
) else (
    set /a COUNT+=2
    if %COUNT% GEQ 60 (
        echo Timeout waiting for .onion address. Check Tor logs at %TOR_LOG%
        pause
        exit /b 1
    )
    timeout /t 2 >nul
    goto WAIT_HOSTNAME
)

REM --- Build the server ---
echo Building Rust server...
cargo build --release
if errorlevel 1 (
    echo Rust build failed. Fix errors and try again.
    pause
    exit /b 1
)

REM --- Start the server ---
if exist "%SERVER_BIN%" (
    echo Starting server on 127.0.0.1:%LOCAL_PORT%...
    start "" "%SERVER_BIN%"
) else (
    echo Server binary not found: %SERVER_BIN%
    pause
    exit /b 1
)

echo All running. Press Ctrl+C or close the window to stop.
pause
endlocal
