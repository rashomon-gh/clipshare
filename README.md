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
- 💾 **In-Memory Storage**: Fast performance with `Arc<RwLock<T>>` state management
- 🛡️ **Thread-Safe**: Concurrent access handling for multiple requests
- 📊 **Comprehensive Logging**: Detailed request/response logging with tracing
- 🎯 **Simple API**: Clean REST endpoints for easy integration
- 🔧 **Error Handling**: Graceful error handling with helpful messages

## 🏗️ Architecture

```
iOS Shortcuts ──> clip_server (0.0.0.0:3000) ──> clip_client ──> Windows Clipboard
```

### Components

- **clip_server**: Axum-based HTTP server with async tokio runtime
- **clip_client**: CLI tool using reqwest for HTTP and arboard for clipboard operations

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

Run the clipboard server:
```bash
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
```

### 2. Test the Server

Send clipboard content via curl:
```bash
# Store clipboard content
curl -X POST http://localhost:3000/clipboard \
  -H "Content-Type: application/json" \
  -d '{"content": "Hello from iOS!"}'

# Retrieve clipboard content
curl http://localhost:3000/clipboard
```

### 3. Run the Client

Retrieve content and update your Windows clipboard:
```bash
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
   - **Headers**: `Content-Type: application/json`
   - **Body**:
   ```json
   {
     "content": "[Your Clipboard Content]"
   }
   ```

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
  "message": "Failed to acquire write lock"
}
```

### GET /clipboard

Retrieve stored clipboard content.

**Request:**
```http
GET /clipboard HTTP/1.1
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

## ⚙️ Configuration

### Server Configuration

Default configuration in [clip_server/src/main.rs](clip_server/src/main.rs):

```rust
const SERVER_PORT: u16 = 3000;
const SERVER_ADDRESS: &str = "0.0.0.0";  // Accepts connections from any IP
```

### Client Configuration

Default configuration in [clip_client/src/main.rs](clip_client/src/main.rs):

```rust
const SERVER_URL: &str = "http://127.0.0.1:3000/clipboard";
const REQUEST_TIMEOUT: u64 = 5;  // seconds
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

### Client Issues

**"Failed to connect to server":**
- Ensure server is running
- Verify server URL is correct
- Check network connectivity

**"Failed to write to clipboard":**
- Close other applications using the clipboard
- Run client with elevated permissions if needed

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
└── clip_client/
    ├── Cargo.toml          # Client dependencies
    └── src/
        └── main.rs         # CLI implementation
```

### Dependencies

**Server:**
- `axum` - Web framework
- `tokio` - Async runtime
- `serde/serde_json` - Serialization
- `tracing` - Logging

**Client:**
- `reqwest` - HTTP client
- `arboard` - Clipboard operations
- `anyhow` - Error handling

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
- **Security**: No authentication - use only on trusted local networks
- **Performance**: Single clipboard item stored - new content overwrites existing
- **Platform**: Client tested on Windows; server should work on any platform

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

MIT License - feel free to use this project for personal or commercial purposes.

## 🙏 Acknowledgments

Built with:
- [Axum](https://github.com/tokio-rs/axum) - Web framework
- [Tokio](https://tokio.rs/) - Async runtime
- [Arboard](https://github.com/1Password/arboard) - Clipboard operations
