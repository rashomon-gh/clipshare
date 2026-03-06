# ClipShare Daemon Setup Guide

This guide explains how to set up ClipShare as a background service that automatically updates your clipboard when new content is pushed to the server.

## 🚀 Quick Start

### One-Time Execution (Default)
```bash
# Set your token
export CLIPSHARE_TOKEN="your_token_here"

# Run once to fetch current clipboard content
cargo run --bin clip_client
```

### Daemon Mode (Continuous Monitoring)
```bash
# Set your token and polling interval (optional, defaults to 2 seconds)
export CLIPSHARE_TOKEN="your_token_here"
export CLIPSHARE_POLL_INTERVAL=2

# Run as daemon - will continuously monitor server
cargo run --bin clip_client
```

Press `Ctrl+C` to stop the daemon.

## 🔧 Platform-Specific Setup

### Linux (systemd)

#### 1. Build the client
```bash
cargo build --release
```

#### 2. Install the service
```bash
# Copy the service file
sudo cp services/clipshare-daemon.service /etc/systemd/system/

# Edit the service file to set your configuration
sudo nano /etc/systemd/system/clipshare-daemon.service
```

Update these lines in the service file:
- `Environment="CLIPSHARE_TOKEN=your_actual_token_here"`
- `ExecStart=/actual/path/to/target/release/clip_client`

#### 3. Enable and start the service
```bash
# Reload systemd
sudo systemctl daemon-reload

# Enable auto-start on boot
sudo systemctl enable clipshare-daemon

# Start the service now
sudo systemctl start clipshare-daemon

# Check service status
sudo systemctl status clipshare-daemon

# View logs
sudo journalctl -u clipshare-daemon -f
```

#### Service Management
```bash
# Stop the service
sudo systemctl stop clipshare-daemon

# Restart the service
sudo systemctl restart clipshare-daemon

# Disable auto-start
sudo systemctl disable clipshare-daemon
```

### macOS (LaunchDaemon)

#### 1. Build the client
```bash
cargo build --release
```

#### 2. Install the LaunchDaemon
```bash
# Copy the plist file
sudo cp services/com.clipshare.daemon.plist /Library/LaunchDaemons/

# Edit the plist file to set your configuration
sudo nano /Library/LaunchDaemons/com.clipshare.daemon.plist
```

Update these values in the plist file:
- `<string>your_token_here</string>` (under CLIPSHARE_TOKEN)
- `<string>/path/to/target/release/clip_client</string>` (under ProgramArguments)

#### 3. Load the service
```bash
# Load the LaunchDaemon
sudo launchctl load /Library/LaunchDaemons/com.clipshare.daemon.plist

# Check if it's running
sudo launchctl list | grep clipshare

# View logs
log show --predicate 'process == "clip_client"' --last 1h
```

#### Service Management
```bash
# Unload the service
sudo launchctl unload /Library/LaunchDaemons/com.clipshare.daemon.plist

# Reload the service
sudo launchctl unload /Library/LaunchDaemons/com.clipshare.daemon.plist
sudo launchctl load /Library/LaunchDaemons/com.clipshare.daemon.plist
```

### Windows

#### Option 1: Windows Service (Recommended)

1. **Build the client**
```powershell
cargo build --release
```

2. **Install as service** (Run PowerShell as Administrator)
```powershell
cd services
.\install-clipshare-service.ps1 -Token "your_token_here" -BinaryPath "C:\path\to\target\release\clip_client.exe"
```

3. **Manage the service**
```powershell
# Start the service
Start-Service -Name ClipShareDaemon

# Check status
Get-Service -Name ClipShareDaemon

# Stop the service
Stop-Service -Name ClipShareDaemon

# Remove the service
Remove-Service -Name ClipShareDaemon
```

#### Option 2: Startup Application (Simpler)

1. **Build the client**
```powershell
cargo build --release
```

2. **Create startup script**
- Edit `services/clipshare-startup.bat`
- Update `CLIPSHARE_TOKEN` and `CLIENT_PATH` variables
- Copy the file to your Startup folder:
  `C:\Users\YOUR_USERNAME\AppData\Roaming\Microsoft\Windows\Start Menu\Programs\Startup\`

3. **Test the startup script**
- Double-click the `.bat` file to test it
- If it works correctly, copy it to your Startup folder
- Restart your computer to verify auto-startup

## ⚙️ Configuration Options

### Environment Variables

| Variable | Description | Default | Example |
|----------|-------------|---------|---------|
| `CLIPSHARE_TOKEN` | Authentication token | Required | `"your_token_here"` |
| `CLIPSHARE_POLL_INTERVAL` | Polling interval in seconds | `2` | `"5"` |
| `CLIPSHARE_VERBOSE` | Enable verbose logging | `false` | `"true"` |
| `CLIPSHARE_ONE_SHOT` | Run once and exit | `false` | `"true"` |

### Advanced Configuration

#### Change Polling Interval
```bash
# Poll every 5 seconds instead of 2
export CLIPSHARE_POLL_INTERVAL=5
cargo run --bin clip_client
```

#### Enable Verbose Logging
```bash
export CLIPSHARE_VERBOSE=true
cargo run --bin clip_client
```

#### Run Once and Exit
```bash
export CLIPSHARE_ONE_SHOT=true
cargo run --bin clip_client
```

## 📊 Monitoring and Logging

### Check if Daemon is Running

**Linux:**
```bash
ps aux | grep clip_client
sudo systemctl status clipshare-daemon
```

**macOS:**
```bash
ps aux | grep clip_client
sudo launchctl list | grep clipshare
```

**Windows:**
```powershell
Get-Process | Where-Object {$_.ProcessName -like "clip_client"}
Get-Service -Name ClipShareDaemon
```

### View Logs

**Linux (systemd):**
```bash
# Real-time logs
sudo journalctl -u clipshare-daemon -f

# Last 100 lines
sudo journalctl -u clipshare-daemon -n 100
```

**macOS (LaunchDaemon):**
```bash
# View recent logs
log show --predicate 'process == "clip_client"' --last 1h

# Real-time logs
log stream --predicate 'process == "clip_client"'
```

**Windows:**
```powershell
# If using Windows Service
Get-EventLog -LogName Application -Source ClipShareDaemon -Newest 50

# If using startup script, logs are in the console window
```

## 🔒 Security Considerations

1. **Token Security**: Store your token securely in the service configuration files
2. **File Permissions**: Ensure service files have appropriate permissions
3. **Network Access**: Daemon works with localhost server IP by default
4. **Resource Usage**: Daemon uses minimal CPU and memory (~10-20MB RAM)

## 🐛 Troubleshooting

### Daemon not starting
- Check if the token is set correctly
- Verify the binary path in service configuration
- Check service logs for error messages

### Clipboard not updating
- Verify server is running: `curl http://localhost:3000/clipboard`
- Check if token matches between server and client
- Ensure network connectivity to server

### High CPU usage
- Increase `CLIPSHARE_POLL_INTERVAL` to reduce polling frequency
- Check for errors in logs causing rapid restarts

### Service won't stop
- Use `kill` command on Linux/macOS: `pkill clip_client`
- Use Task Manager on Windows to end the process
- As last resort, restart your computer

## 🔄 Uninstallation

### Linux
```bash
sudo systemctl stop clipshare-daemon
sudo systemctl disable clipshare-daemon
sudo rm /etc/systemd/system/clipshare-daemon.service
sudo systemctl daemon-reload
```

### macOS
```bash
sudo launchctl unload /Library/LaunchDaemons/com.clipshare.daemon.plist
sudo rm /Library/LaunchDaemons/com.clipshare.daemon.plist
```

### Windows
```powershell
Stop-Service -Name ClipShareDaemon
Remove-Service -Name ClipShareDaemon
# Remove startup shortcut from Startup folder if using that method
```

## 📝 Performance Notes

- **Memory Usage**: ~10-20 MB RAM
- **CPU Usage**: < 1% when idle
- **Network Usage**: Minimal (small HTTP requests every 2 seconds)
- **Battery Impact**: Negligible on laptops

The daemon is designed to be lightweight and have minimal impact on system performance.
