# ClipShare

[![Tests](https://github.com/rashomon-gh/clipshare/workflows/Tests/badge.svg)](https://github.com/rashomon-gh/clipshare/actions/workflows/test.yml)


A local network clipboard sharing service built in Rust. Seamlessly share clipboard content between your iOS devices and PC/Mac via REST API.

## 🎯 Overview

ClipShare consists of three components:
- **clip_server**: A background REST server that stores clipboard content in memory
- **clip_client**: A daemon that continuously monitors the server and automatically updates your system clipboard
- **clip_token_gen**: Secure token generation tool for authentication

Perfect for integration with iOS Shortcuts to share text, images, and files between your iPhone/iPad and PC/Mac!

## ✨ Features

- 🔄 **Real-time Sharing**: Instant clipboard synchronization across devices
- 🤖 **Background Daemon**: Automatically monitors server and updates clipboard
- 🌐 **Network Accessible**: Server accessible from local Wi-Fi network
- 🔐 **Token Authentication**: Secure Bearer token authentication for all requests
- 📝 **Multi-Format Support**: Text, images, and files with proper MIME type handling
- 💾 **In-Memory Storage**: Fast performance with `Arc<RwLock<T>>` state management
- 🛡️ **Thread-Safe**: Concurrent access handling for multiple requests
- 📊 **Comprehensive Logging**: Detailed request/response logging with tracing
- 🎯 **Simple API**: Clean REST endpoints for easy integration
- 🔧 **Error Handling**: Graceful error handling with helpful messages
- 🔑 **Token Generator**: Built-in secure token generation tool
- 🚀 **Cross-Platform**: Runs on Windows, macOS, and Linux as native services
- 🧪 **Comprehensive Testing**: 44 tests covering unit and integration scenarios
- ✅ **CI/CD Pipeline**: GitHub Actions workflow for automated testing
- 🏗️ **Modular Architecture**: Clean separation of concerns with dedicated modules
- 📦 **Service Integration**: Run as system service on all major platforms

## 🏗️ Architecture

```
iOS Shortcuts ──> clip_server (0.0.0.0:3000) ──> clip_client ──> System Clipboard
```

### Components

- **clip_server**: Axum-based HTTP server with async tokio runtime
- **clip_client**: CLI tool using reqwest for HTTP and arboard for cross-platform clipboard operations
- **clip_token_gen**: Secure token generation tool for authentication

### Platform Support

**Client (clip_client):**
- ✅ **Windows 10+**: Full clipboard support (text, images, files)
- ✅ **macOS 10.13+**: Full clipboard support (text, images, files)
- ✅ **Linux**: Full clipboard support on Wayland and X11 (text, images, files)

**Server (clip_server):**
- ✅ **Windows, macOS, Linux**: Runs on any platform with Rust support

## 🔐 Authentication

ClipShare uses Bearer token authentication to secure all API requests. Both the server and client require the same authentication token.

### Setting Up Authentication

1. **Generate a secure token:**

```bash
cargo run --bin clip_token_gen
```

This will generate a cryptographically secure random token and display usage instructions.

2. **Set the environment variable on your server:**

```bash
# Linux/macOS
export CLIPSHARE_TOKEN="your_generated_token_here"

# Windows PowerShell
$env:CLIPSHARE_TOKEN="your_generated_token_here"

# Windows Command Prompt
set CLIPSHARE_TOKEN=your_generated_token_here
```

3. **Set the same environment variable on your client:**

Use the same token value on your client machine.

### Persistent Configuration

For convenience, add the environment variable to your shell profile:

```bash
# Linux/macOS - add to ~/.bashrc, ~/.zshrc, or ~/.profile
export CLIPSHARE_TOKEN="your_generated_token_here"

# Windows PowerShell - add to $PROFILE
$env:CLIPSHARE_TOKEN="your_generated_token_here"
```

## 🚀 Getting Started

### Prerequisites

**Platform-specific notes:**

- **Linux/Wayland**: Ensure your Wayland compositor supports clipboard operations (most modern compositors like GNOME, KDE Plasma, Sway do)
- **Linux/X11**: No additional requirements - works out of the box
- **macOS**: Grant clipboard permissions when prompted
- **Windows**: No additional requirements - works out of the box

### Installation

1. Clone the repository:
```bash
git clone https://codeberg.org/rashomon/clipshare.git
cd clipshare
```

2. Build the project:
```bash
cargo build --release
```

This creates two binaries:
- `target/release/clip_server`
- `target/release/clip_client`

## 📖 Usage

### 1. Start the Server

Run the clipboard server (make sure to set the `CLIPSHARE_TOKEN` environment variable first):
```bash
# Set the token first
export CLIPSHARE_TOKEN="your_generated_token"

# Start the server
cargo run --bin clip_server
```

Or use the compiled binary:
```bash
./target/release/clip_server.exe
```

The server will start on `http://0.0.0.0:3000` and log:
```
🚀 Clipboard Server starting on http://0.0.0.0:3000
📡 Server is accessible from your local Wi-Fi network
🔒 Authentication is enabled - all requests require a valid Bearer token
```

### 2. Test the Server

Send clipboard content via curl with authentication:
```bash
# Store text content
curl -X POST http://localhost:3000/clipboard \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your_generated_token" \
  -d '{"contentType": "text/plain", "data": "Hello from iOS!"}'

# Store image content (base64 encoded)
curl -X POST http://localhost:3000/clipboard \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your_generated_token" \
  -d '{"contentType": "image/png", "data": "iVBORw0KG..."}'

# Retrieve clipboard content
curl http://localhost:3000/clipboard \
  -H "Authorization: Bearer your_generated_token"
```

### 3. Run the Client

The client supports two modes:

#### One-Shot Mode (Default)
Retrieve content once and update your clipboard:
```bash
# Set the token first (same as server)
export CLIPSHARE_TOKEN="your_generated_token"

# Run once to fetch current clipboard content
cargo run --bin clip_client
```

Expected output:
```
📋 Clipboard Client (One-Shot Mode)
🔗 Connecting to server at: http://127.0.0.1:3000/clipboard
✅ Successfully retrieved clipboard content from server
🎉 Clipboard updated successfully!
```

#### Daemon Mode (Continuous Monitoring)
Automatically monitor the server and update your clipboard when new content is pushed:
```bash
# Set polling interval (optional, defaults to 2 seconds)
export CLIPSHARE_POLL_INTERVAL=2

# Run as daemon - will continuously monitor server
cargo run --bin clip_client
```

Expected output:
```
🚀 ClipShare Daemon Starting
📡 Monitoring server at: http://127.0.0.1:3000/clipboard
⏱️  Poll interval: 2 seconds
Press Ctrl+C to stop

✅ 12:34:56 - Clipboard updated
```

Press `Ctrl+C` to stop the daemon.

#### Install as System Service

For automatic startup and background operation, install the client as a system service:

- **Linux (systemd)**: See [services/README.md](services/README.md#linux-systemd)
- **macOS (LaunchDaemon)**: See [services/README.md](services/README.md#macos-launchdaemon)
- **Windows (Service)**: See [services/README.md](services/README.md#windows)

📖 **[Complete Service Setup Guide →](services/README.md)**

## 📱 iOS Shortcuts Integration

Create an iOS Shortcut to send clipboard content to your PC:

### Shortcut Configuration

1. **Action**: Get Clipboard from iOS
2. **Action**: Detect Content Type (text, image, or file)
3. **Action**: Convert to Base64 (for images/files)
4. **Action**: HTTP Request
   - **URL**: `http://YOUR_PC_IP:3000/clipboard`
   - **Method**: POST
   - **Headers**:
     - `Content-Type: application/json`
     - `Authorization: Bearer your_generated_token`
   - **Body**:
   ```json
   {
     "contentType": "text/plain",
     "data": "[Your Clipboard Content or Base64 Data]"
   }
   ```

For images and files:
```json
{
  "contentType": "image/png",
  "data": "[Base64 encoded image data]"
}
```

### Important Security Note

Your iOS Shortcut needs to include the same authentication token that you set on your server. Make sure to replace `your_generated_token` with the actual token you generated using `clip_token_gen`.

### Finding Your PC IP Address

On Windows, run in PowerShell:
```powershell
ipconfig
```

Look for "IPv4 Address" under your network adapter (usually `192.168.x.x` or `10.0.x.x`)

## 🔌 API Documentation

### POST /clipboard

Store clipboard content on the server. Supports text, images, and files.

**Request (Text):**
```http
POST /clipboard HTTP/1.1
Content-Type: application/json
Authorization: Bearer your_generated_token

{
  "contentType": "text/plain",
  "data": "Your clipboard text here"
}
```

**Request (Image - Base64 Encoded):**
```http
POST /clipboard HTTP/1.1
Content-Type: application/json
Authorization: Bearer your_generated_token

{
  "contentType": "image/png",
  "data": "iVBORw0KGgoAAAANSUhEUgAAAAUA..."
}
```

**Request (File - Base64 Encoded):**
```http
POST /clipboard HTTP/1.1
Content-Type: application/json
Authorization: Bearer your_generated_token

{
  "contentType": "application/pdf",
  "filename": "document.pdf",
  "data": "JVBERi0xLjQKJe..."
}
```

**Response (Success):**
```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "status": "success",
  "message": "Clipboard content updated successfully"
}
```

**Response (Error):**
```http
HTTP/1.1 500 Internal Server Error
Content-Type: application/json

{
  "status": "error",
  "message": "Failed to update clipboard content"
}
```

**Response (Unauthorized):**
```http
HTTP/1.1 401 Unauthorized
Content-Type: application/json

{
  "status": "error",
  "message": "Unauthorized request"
}
```

### GET /clipboard

Retrieve stored clipboard content.

**Request:**
```http
GET /clipboard HTTP/1.1
Authorization: Bearer your_generated_token
```

**Response (Success - Text):**
```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "type": "text",
  "data": "Your clipboard text here"
}
```

**Response (Success - Image):**
```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "type": "image",
  "data": "base64_encoded_image_data",
  "mimeType": "image/png"
}
```

**Response (Success - File):**
```http
HTTP/1.1 200 OK
Content-Type: application/json

{
  "type": "file",
  "name": "document.pdf",
  "data": "base64_encoded_file_data",
  "mimeType": "application/pdf"
}
```

**Response (No Content):**
```http
HTTP/1.1 404 Not Found
Content-Type: application/json

{
  "status": "error",
  "message": "No clipboard content available"
}
```

**Response (Unauthorized):**
```http
HTTP/1.1 401 Unauthorized
Content-Type: application/json

{
  "status": "error",
  "message": "Unauthorized request"
}
```

## ⚙️ Configuration

### Server Configuration

Default configuration in [clip_server/src/main.rs](clip_server/src/main.rs):

```rust
const SERVER_PORT: u16 = 3000;
const SERVER_ADDRESS: &str = "0.0.0.0";  // Accepts connections from any IP
const TOKEN_ENV_VAR: &str = "CLIPSHARE_TOKEN";  // Environment variable for auth token
```

### Client Configuration

Default configuration in [clip_client/src/main.rs](clip_client/src/main.rs):

```rust
const SERVER_URL: &str = "http://127.0.0.1:3000/clipboard";
const REQUEST_TIMEOUT: u64 = 5;  // seconds
const TOKEN_ENV_VAR: &str = "CLIPSHARE_TOKEN";  // Environment variable for auth token
```


## 🛠️ Development

### Project Structure

```
clipshare/
├── .github/
│   └── workflows/
│       └── test.yml         # CI/CD pipeline for automated testing
├── Cargo.toml               # Workspace configuration
├── clip_server/
│   ├── Cargo.toml           # Server dependencies
│   └── src/
│       ├── main.rs          # Server entry point
│       ├── lib.rs           # Library exports
│       ├── auth.rs          # Authentication middleware
│       ├── config.rs        # Server configuration
│       ├── handlers.rs      # HTTP request handlers
│       └── models.rs        # Data models and types
├── clip_client/
│   ├── Cargo.toml           # Client dependencies
│   └── src/
│       ├── main.rs          # Client entry point
│       ├── api.rs           # HTTP client operations
│       ├── clipboard_ops.rs # Clipboard operations
│       ├── config.rs        # Client configuration
│       ├── daemon.rs        # Background daemon mode
│       └── models.rs        # Data models for responses
├── clip_token_gen/
│   ├── Cargo.toml           # Token generator dependencies
│   └── src/
│       └── main.rs          # Token generation utility
├── tests/
│   ├── Cargo.toml           # Integration test dependencies
│   └── src/
│       └── e2e_tests.rs     # End-to-end integration tests
└── services/
    ├── README.md            # Service installation guide
    ├── clipshare-daemon.service  # Linux systemd service
    ├── com.clipshare.daemon.plist # macOS LaunchDaemon
    ├── install-clipshare-service.ps1 # Windows Service installer
    └── clipshare-startup.bat      # Windows startup script
```

### Architecture

The project follows a modular architecture with clear separation of concerns:

**Server Components:**
- **auth.rs**: Bearer token authentication middleware
- **handlers.rs**: REST API endpoint handlers (GET/POST /clipboard)
- **models.rs**: Request/response models with comprehensive validation
- **config.rs**: Server configuration constants

**Client Components:**
- **api.rs**: HTTP client with authentication and error handling
- **clipboard_ops.rs**: Multi-format clipboard operations (text, images, files)
- **daemon.rs**: Continuous monitoring with change detection and graceful shutdown
- **config.rs**: Environment-based configuration management

### Testing

The project has comprehensive test coverage with **44 tests** across multiple levels:

```bash
# Run all tests (unit + integration)
cargo test --all

# Run specific test suites
cargo test -p clip_server     # Server unit tests (12 tests)
cargo test -p clip_client     # Client unit tests (12 tests)
cargo test -p clip_token_gen  # Token generator tests (2 tests)
cargo test -p tests           # End-to-end tests (5 tests)

# Run tests with output
cargo test --all -- --nocapture

# Run tests in parallel
cargo test --all --jobs 4
```


### Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test --all

# Check code without building
cargo check

# Run linter
cargo clippy --all-targets --all-features -- -D warnings
```


## 📄 License

AGPL-3.0
