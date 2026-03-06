# ClipShare Windows Service Installation Script
# Run this script as Administrator

param(
    [Parameter(Mandatory=$true)]
    [string]$Token,

    [Parameter(Mandatory=$true)]
    [string]$BinaryPath
)

# Configuration
$ServiceName = "ClipShareDaemon"
$DisplayName = "ClipShare Clipboard Daemon"
$Description = "Continuously monitors ClipShare server and updates system clipboard"

# Check if running as Administrator
if (-NOT ([Security.Principal.WindowsPrincipal][Security.Principal.WindowsIdentity]::GetCurrent()).IsInRole([Security.Principal.WindowsBuiltInRole] "Administrator")) {
    Write-Warning "You need Administrator privileges to run this script"
    Write-Warning "Please run PowerShell as Administrator"
    exit 1
}

# Check if binary exists
if (-NOT (Test-Path $BinaryPath)) {
    Write-Error "Binary not found at: $BinaryPath"
    exit 1
}

# Create the service wrapper script
$WrapperScript = "$env:ProgramFiles\ClipShare\service-wrapper.ps1"
$ServiceDir = "$env:ProgramFiles\ClipShare"

# Create service directory if it doesn't exist
if (-NOT (Test-Path $ServiceDir)) {
    New-Item -ItemType Directory -Path $ServiceDir -Force
    Write-Host "Created service directory: $ServiceDir" -ForegroundColor Green
}

# Create the wrapper script
$WrapperContent = @'
# ClipShare Service Wrapper
$env:CLIPSHARE_TOKEN = "TOKEN_PLACEHOLDER"
$env:CLIPSHARE_POLL_INTERVAL = "2"
$env:CLIPSHARE_VERBOSE = "false"

$ClientPath = "BINARY_PATH_PLACEHOLDER"

while ($true) {
    try {
        & $ClientPath
        # If we get here, the client exited normally
        Write-Host "ClipShare client exited, restarting in 5 seconds..."
        Start-Sleep -Seconds 5
    }
    catch {
        Write-Error "Error running ClipShare client: $_"
        Start-Sleep -Seconds 10
    }
}
'@

$WrapperContent = $WrapperContent -replace "TOKEN_PLACEHOLDER", $Token
$WrapperContent = $WrapperContent -replace "BINARY_PATH_PLACEHOLDER", $BinaryPath

$WrapperContent | Out-File -FilePath $WrapperScript -Encoding ASCII
Write-Host "Created service wrapper: $WrapperScript" -ForegroundColor Green

# Create the Windows Service using PowerShell's New-Service
try {
    # Check if service already exists
    $existingService = Get-Service -Name $ServiceName -ErrorAction SilentlyContinue

    if ($existingService) {
        Write-Warning "Service already exists, removing old service..."
        Stop-Service -Name $ServiceName -Force -ErrorAction SilentlyContinue
        Remove-Service -Name $ServiceName -Force
        Start-Sleep -Seconds 2
    }

    # Create the new service
    New-Service -Name $ServiceName `
                 -BinaryPathName "powershell.exe -ExecutionPolicy Bypass -File `"$WrapperScript`"" `
                 -DisplayName $DisplayName `
                 -Description $Description `
                 -StartupType Automatic

    Write-Host "Service created successfully!" -ForegroundColor Green
    Write-Host "Service Name: $ServiceName" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "To start the service:" -ForegroundColor Yellow
    Write-Host "  Start-Service -Name $ServiceName" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "To check service status:" -ForegroundColor Yellow
    Write-Host "  Get-Service -Name $ServiceName" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "To stop the service:" -ForegroundColor Yellow
    Write-Host "  Stop-Service -Name $ServiceName" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "To remove the service:" -ForegroundColor Yellow
    Write-Host "  Remove-Service -Name $ServiceName" -ForegroundColor Cyan
    Write-Host ""

    # Optionally start the service immediately
    $startNow = Read-Host "Start the service now? (Y/N)"
    if ($startNow -eq 'Y' -or $startNow -eq 'y') {
        Start-Service -Name $ServiceName
        Write-Host "Service started!" -ForegroundColor Green
        Write-Host "Check status with: Get-Service -Name $ServiceName" -ForegroundColor Cyan
    }
}
catch {
    Write-Error "Failed to create service: $_"
    exit 1
}
