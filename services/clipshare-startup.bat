@echo off
REM ClipShare Windows Startup Script
REM Place this file in your Startup folder:
REM C:\Users\YOUR_USERNAME\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Startup

REM Configuration
SET CLIPSHARE_TOKEN=your_token_here
SET CLIPSHARE_POLL_INTERVAL=2
SET CLIPSHARE_VERBOSE=false

REM Path to the compiled binary
SET CLIENT_PATH=C:\path\to\clip_client.exe

echo Starting ClipShare Daemon...
echo Server: http://127.0.0.1:3000
echo Poll interval: %CLIPSHARE_POLL_INTERVAL% seconds
echo.

REM Check if binary exists
if not exist "%CLIENT_PATH%" (
    echo ERROR: Client binary not found at: %CLIENT_PATH%
    echo Please update the CLIENT_PATH variable in this script
    pause
    exit /b 1
)

REM Run the client - it will automatically restart if it crashes
:loop
"%CLIENT_PATH%"
echo Client exited, restarting in 5 seconds...
timeout /t 5 /nobreak > nul
goto loop
