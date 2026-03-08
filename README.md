# ClipShare

[![Tests](https://github.com/rashomon-gh/clipshare/workflows/Tests/badge.svg)](https://github.com/rashomon-gh/clipshare/actions/workflows/test.yml)


A local network clipboard sharing service built in Rust. Seamlessly share clipboard content between your iOS devices and PC/Mac via REST API.

## 🎯 Overview

ClipShare consists of three components:
- **clip_server**: A background REST server that stores clipboard content in memory
- **clip_client**: A daemon that continuously monitors the server and automatically updates your system clipboard
- **clip_token_gen**: Secure token generation tool for authentication


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


### Persistent Configuration

For convenience, add the environment variable to your shell profile:

```bash
# Linux/macOS - add to ~/.bashrc, ~/.zshrc, or ~/.profile
export CLIPSHARE_TOKEN="your_generated_token_here"

# Windows PowerShell - add to $PROFILE
$env:CLIPSHARE_TOKEN="your_generated_token_here"
```

## 🚀 Getting Started

### Installation

#### Option 1: Docker (Recommended for Server)

Build and run the server using Docker:

```bash
# Generate a token first
TOKEN=$(openssl rand -base64 32)

# Build the Docker image
docker build -t clipshare-server .

# Run the server
docker run -d \
  --name clipshare \
  -p 3000:3000 \
  -e CLIPSHARE_TOKEN=$TOKEN \
  -e RUST_LOG=info \
  clipshare-server

# View logs
docker logs -f clipshare

# Get your token
docker inspect clipshare | grep CLIPSHARE_TOKEN
```

#### Option 2: Build from Source

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

> [!NOTE]
> 📖 **[Complete Service Setup Guide →](services/README.md)**

For automatic startup and background operation, install the client as a system service:

- **Linux (systemd)**: See [services/README.md](services/README.md#linux-systemd)
- **macOS (LaunchDaemon)**: See [services/README.md](services/README.md#macos-launchdaemon)
- **Windows (Service)**: See [services/README.md](services/README.md#windows)



## 📱 iOS Shortcuts Integration

Create an iOS Shortcut to send clipboard content to your PC:

### Shortcut Configuration

> [!IMPORTANT]
> Your iOS Shortcut needs to include the same authentication token that you set on your server. Make sure to replace `your_generated_token` with the actual token you generated using `clip_token_gen`.

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

For images:
```json
{
  "contentType": "image/png",
  "data": "[Base64 encoded image data]"
}
```




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



## 🛠️ Development


### Testing

```bash
# Run all tests (unit + integration)
cargo test --all

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
```

### Docker Development

**Build the Docker image:**

```bash
docker build -t clipshare-server .
```

**Run the container:**

```bash
# Generate a secure token
TOKEN=$(openssl rand -base64 32)

# Run the server
docker run -d \
  --name clipshare \
  -p 3000:3000 \
  -e CLIPSHARE_TOKEN=$TOKEN \
  -e RUST_LOG=info \
  clipshare-server

# View logs
docker logs -f clipshare
```

**Docker Compose:**

```bash
# Start the service
docker-compose up -d

# View logs
docker-compose logs -f

# Stop the service
docker-compose down
```


## 📄 License

AGPL-3.0
