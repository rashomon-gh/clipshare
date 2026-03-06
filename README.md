# ClipShare

A local network clipboard sharing service built in Rust. Seamlessly share clipboard content between your iOS devices and Windows PC via REST API.

## 🎯 Overview

ClipShare consists of two components:
- **clip_server**: A background REST server that stores clipboard content in memory
- **clip_client**: A CLI tool that retrieves content from the server and places it on your Windows clipboard

Perfect for integration with iOS Shortcuts to share text between your iPhone/iPad and PC!

## ✨ Features

- 🔄 **Real-time Sharing**: Instant clipboard synchronization across devices
- 🌐 **Network Accessible**: Server accessible from local Wi-Fi network
- 🔐 **Token Authentication**: Secure Bearer token authentication for all requests
- 💾 **In-Memory Storage**: Fast performance with `Arc<RwLock<T>>` state management
- 🛡️ **Thread-Safe**: Concurrent access handling for multiple requests
- 📊 **Comprehensive Logging**: Detailed request/response logging with tracing
- 🎯 **Simple API**: Clean REST endpoints for easy integration
- 🔧 **Error Handling**: Graceful error handling with helpful messages
- 🔑 **Token Generator**: Built-in secure token generation tool

## 🏗️ Architecture

```
iOS Shortcuts ──> clip_server (0.0.0.0:3000) ──> clip_client ──> Windows Clipboard
```

### Components

- **clip_server**: Axum-based HTTP server with async tokio runtime
- **clip_client**: CLI tool using reqwest for HTTP and arboard for clipboard operations
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

Use the same token value on your client machine.

### Persistent Configuration

For convenience, add the environment variable to your shell profile:

```bash
# Linux/macOS - add to ~/.bashrc, ~/.zshrc, or ~/.profile
export CLIPSHARE_TOKEN="your_generated_token_here"

# Windows PowerShell - add to $PROFILE
$env:CLIPSHARE_TOKEN="your_generated_token_here"
```

### Security Best Practices

- 🔒 Keep your authentication token secret and secure
- 🚫 Never share tokens publicly or commit them to version control
- 🔄 Generate a new token if you suspect it has been compromised
- 🌐 Use different tokens for different environments (dev/prod)
- 📱 Ensure your iOS Shortcuts include the token in requests

## 🚀 Getting Started

### Prerequisites

- Rust 1.70+ installed
- Windows OS (for clipboard operations)
- Local Wi-Fi network for device communication

### Installation

1. Clone the repository:
```bash
git clone <your-repo-url>
cd clipshare
```

2. Build the project:
```bash
cargo build --release
```

This creates two binaries:
- `target/release/clip_server.exe`
- `target/release/clip_client.exe`

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
# Store clipboard content
curl -X POST http://localhost:3000/clipboard \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer your_generated_token" \
  -d '{"content": "Hello from iOS!"}'

# Retrieve clipboard content
curl http://localhost:3000/clipboard \
  -H "Authorization: Bearer your_generated_token"
```

### 3. Run the Client

Retrieve content and update your Windows clipboard (make sure to set the `CLIPSHARE_TOKEN` environment variable):
```bash
# Set the token first (same as server)
export CLIPSHARE_TOKEN="your_generated_token"

# Run the client
cargo run --bin clip_client
```

Or use the compiled binary:
```bash
./target/release/clip_client.exe
```

Expected output:
```
📋 Clipboard Client
🔗 Connecting to server at: http://127.0.0.1:3000/clipboard
🔐 Authentication token loaded successfully
✅ Successfully retrieved clipboard content from server
📄 Content length: 16 bytes
🎉 Clipboard updated successfully!
💡 Your clipboard now contains the content from the server
```

## 📱 iOS Shortcuts Integration

Create an iOS Shortcut to send clipboard content to your PC:

### Shortcut Configuration

1. **Action**: Get Clipboard from iOS
2. **Action**: HTTP Request
   - **URL**: `http://YOUR_PC_IP:3000/clipboard`
   - **Method**: POST
   - **Headers**:
     - `Content-Type: application/json`
     - `Authorization: Bearer your_generated_token`
   - **Body**:
   ```json
   {
     "content": "[Your Clipboard Content]"
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

Store clipboard content on the server.

**Request:**
```http
POST /clipboard HTTP/1.1
Content-Type: application/json
Authorization: Bearer your_generated_token

{
  "content": "Your clipboard text here"
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

**Response (Success):**
```http
HTTP/1.1 200 OK
Content-Type: application/json

"Your clipboard text here"
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

## 🔧 Troubleshooting

### Server Issues

**Server won't start:**
- Check if port 3000 is already in use
- Ensure firewall allows connections on port 3000
- Verify no other instances are running

**Can't access server from other devices:**
- Ensure server is running on 0.0.0.0 (not 127.0.0.1)
- Check Windows Firewall settings
- Verify devices are on the same Wi-Fi network
- Confirm correct IP address in iOS Shortcut
- Verify authentication token matches between server and client

**Authentication errors (401 Unauthorized):**
- Ensure `CLIPSHARE_TOKEN` environment variable is set on both server and client
- Verify the token matches exactly between server and client
- Check that Authorization header includes "Bearer " prefix in requests
- Make sure iOS Shortcut includes the token in the Authorization header

### Client Issues

**"Failed to connect to server":**
- Ensure server is running
- Verify server URL is correct
- Check network connectivity

**"Failed to write to clipboard":**
- Close other applications using the clipboard
- Run client with elevated permissions if needed

**"Authentication failed - invalid or missing token":**
- Ensure `CLIPSHARE_TOKEN` environment variable is set
- Verify the token matches the server's token exactly
- Check for typos in the environment variable name or value

**"No clipboard content available":**
- Server has received no content yet
- Send content via iOS Shortcut or curl first

## 🛠️ Development

### Project Structure

```
clipshare/
├── Cargo.toml              # Workspace configuration
├── clip_server/
│   ├── Cargo.toml          # Server dependencies
│   └── src/
│       └── main.rs         # REST API implementation
├── clip_client/
│   ├── Cargo.toml          # Client dependencies
│   └── src/
│       └── main.rs         # CLI implementation
└── clip_token_gen/
    ├── Cargo.toml          # Token generator dependencies
    └── src/
        └── main.rs         # Token generation utility
```

### Dependencies

**Server:**
- `axum` - Web framework
- `tokio` - Async runtime
- `serde/serde_json` - Serialization
- `tracing` - Logging
- `dotenvy` - Environment variable loading

**Client:**
- `reqwest` - HTTP client
- `arboard` - Clipboard operations
- `anyhow` - Error handling

**Token Generator:**
- `rand` - Cryptographically secure random number generation
- `base64` - Token encoding

### Building

```bash
# Debug build
cargo build

# Release build (optimized)
cargo build --release

# Run tests
cargo test

# Check code without building
cargo check
```

## 📝 Notes

- **Data Persistence**: Clipboard content is stored in memory only - lost on server restart
- **Security**: Token-based authentication required for all API requests
- **Performance**: Single clipboard item stored - new content overwrites existing
- **Platform**: Client tested on Windows; server should work on any platform
- **Token Management**: Generate new tokens using `clip_token_gen` binary
- **Environment**: Requires `CLIPSHARE_TOKEN` environment variable on both server and client

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

MIT License - feel free to use this project for personal or commercial purposes.

## 🙏 Acknowledgments

Built with:
- [Axum](https://github.com/tokio-rs/axum) - Web framework
- [Tokio](https://tokio.rs/) - Async runtime
- [Arboard](https://github.com/1Password/arboard) - Clipboard operations
