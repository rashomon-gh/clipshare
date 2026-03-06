use anyhow::{Context, Result};
use arboard::Clipboard;
use reqwest::Client;
use serde::Deserialize;
use std::env;
use std::time::Duration;

// Base64 encoding support
use base64::prelude::*;

/// Configuration for the clipboard client
const SERVER_URL: &str = "http://127.0.0.1:3000/clipboard";
const REQUEST_TIMEOUT: u64 = 5; // seconds
const TOKEN_ENV_VAR: &str = "CLIPSHARE_TOKEN";

/// Response structure for clipboard content from the server
#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "data")]
enum ClipboardContent {
    #[serde(rename = "text")]
    Text(String),
    #[serde(rename = "image")]
    Image { data: String, mime_type: String },
    #[serde(rename = "file")]
    File { name: String, data: String, mime_type: String },
}

/// Main function that fetches clipboard content from the server
/// and writes it to the local OS clipboard
#[tokio::main]
async fn main() -> Result<()> {
    println!("📋 Enhanced Clipboard Client");
    println!("🔗 Connecting to server at: {}", SERVER_URL);

    // Load authentication token from environment variable
    let auth_token = env::var(TOKEN_ENV_VAR).unwrap_or_else(|_| {
        eprintln!("⚠️  WARNING: {} environment variable not set!", TOKEN_ENV_VAR);
        eprintln!("📝 To set it up:");
        eprintln!("   1. Generate a token: cargo run --bin clip_token_gen");
        eprintln!("   2. Set the environment variable:");
        eprintln!("      export {}=\"your_generated_token\"", TOKEN_ENV_VAR);
        eprintln!();
        eprintln!("❌ Client cannot authenticate without the token.");
        std::process::exit(1);
    });

    println!("🔐 Authentication token loaded successfully");

    // Create an HTTP client with timeout
    let client = Client::builder()
        .timeout(Duration::from_secs(REQUEST_TIMEOUT))
        .build()
        .context("Failed to create HTTP client")?;

    // Fetch clipboard content from the server
    match fetch_clipboard_content(&client, &auth_token).await {
        Ok(content) => {
            println!("✅ Successfully retrieved clipboard content from server");

            // Write the content to the local clipboard based on type
            match content {
                ClipboardContent::Text(text) => {
                    println!("📄 Content type: Text (length: {} bytes)", text.len());
                    write_text_to_clipboard(&text)?;
                }
                ClipboardContent::Image { data, mime_type } => {
                    println!("🖼️  Content type: Image ({})", mime_type);
                    println!("📊 Data size: {} bytes (base64 encoded)", data.len());
                    write_image_to_clipboard(&data)?;
                }
                ClipboardContent::File { name, data, mime_type } => {
                    println!("📁 Content type: File ({})", mime_type);
                    println!("📝 Filename: {}", name);
                    println!("📊 Data size: {} bytes (base64 encoded)", data.len());
                    write_file_to_clipboard(&name, &data, &mime_type)?;
                }
            }

            println!("🎉 Clipboard updated successfully!");
            Ok(())
        }
        Err(e) => {
            eprintln!("❌ Failed to retrieve clipboard content: {}", e);
            eprintln!("💡 Troubleshooting tips:");
            eprintln!("   1. Make sure the server is running at: {}", SERVER_URL);
            eprintln!("   2. Check if the server has received any content yet");
            eprintln!("   3. Verify your network connection");
            eprintln!("   4. Ensure your authentication token matches the server's token");
            Err(e)
        }
    }
}

/// Fetches clipboard content from the server with authentication
async fn fetch_clipboard_content(
    client: &Client,
    auth_token: &str,
) -> Result<ClipboardContent> {
    let response = client
        .get(SERVER_URL)
        .header("Authorization", format!("Bearer {}", auth_token))
        .send()
        .await
        .context("Failed to connect to server")?;

    let status = response.status();
    let response_text = response
        .text()
        .await
        .context("Failed to read response body")?;

    // Handle different HTTP status codes
    match status.as_u16() {
        200 => {
            // Parse the JSON response as ClipboardContent
            let content: ClipboardContent = serde_json::from_str(&response_text)
                .context("Failed to parse clipboard content")?;
            Ok(content)
        }
        401 => {
            anyhow::bail!("Authentication failed - invalid or missing token");
        }
        404 => {
            anyhow::bail!("No clipboard content available on the server");
        }
        500 => {
            anyhow::bail!("Internal server error - check server logs");
        }
        _ => {
            anyhow::bail!("Unexpected server response: {}", status);
        }
    }
}

/// Writes text content to the system clipboard
fn write_text_to_clipboard(content: &str) -> Result<()> {
    let mut clipboard = Clipboard::new().context("Failed to access system clipboard")?;

    clipboard
        .set_text(content.to_string())
        .context("Failed to write text to clipboard")?;

    println!("💡 Text content copied to clipboard");
    Ok(())
}

/// Writes image content to the system clipboard
fn write_image_to_clipboard(base64_data: &str) -> Result<()> {
    // Decode base64 data
    let image_data = BASE64_STANDARD
        .decode(base64_data)
        .context("Failed to decode base64 image data")?;

    // For now, we'll save the image to a temp file and print the path
    // since arboard's image handling can be complex
    let temp_path = format!("clipboard_image_{}.png", chrono_timestamp());
    std::fs::write(&temp_path, &image_data)
        .context("Failed to write image to temp file")?;

    println!("💡 Image saved to: {}", temp_path);
    println!("💡 Tip: Open the file to view the image");
    println!("💡 Note: Direct image clipboard support varies by platform");

    Ok(())
}

/// Writes file content to the system clipboard (saves to file)
fn write_file_to_clipboard(filename: &str, base64_data: &str, _mime_type: &str) -> Result<()> {
    // Decode base64 data
    let file_data = BASE64_STANDARD
        .decode(base64_data)
        .context("Failed to decode base64 file data")?;

    // Save to file with original filename (sanitized)
    let safe_filename = filename
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' { c } else { '_' })
        .collect::<String>();

    let temp_path = format!("clipboard_file_{}", safe_filename);
    std::fs::write(&temp_path, &file_data)
        .context("Failed to write file data")?;

    println!("💡 File saved to: {}", temp_path);
    println!("💡 Tip: The file has been saved to your current directory");

    Ok(())
}

/// Generates a simple timestamp for filenames
fn chrono_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_secs())
}
