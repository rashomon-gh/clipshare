use anyhow::{Context, Result};
use arboard::Clipboard;
use reqwest::Client;
use serde::Deserialize;
use std::time::Duration;

/// Configuration for the clipboard client
const SERVER_URL: &str = "http://127.0.0.1:3000/clipboard";
const REQUEST_TIMEOUT: u64 = 5; // seconds

/// Response structure for parsing the clipboard content from the server
#[derive(Debug, Deserialize)]
struct ClipboardResponse {
    content: String,
}

/// Main function that fetches clipboard content from the server
/// and writes it to the local OS clipboard
#[tokio::main]
async fn main() -> Result<()> {
    println!("📋 Clipboard Client");
    println!("🔗 Connecting to server at: {}", SERVER_URL);

    // Create an HTTP client with timeout
    let client = Client::builder()
        .timeout(Duration::from_secs(REQUEST_TIMEOUT))
        .build()
        .context("Failed to create HTTP client")?;

    // Fetch clipboard content from the server
    match fetch_clipboard_content(&client).await {
        Ok(content) => {
            println!("✅ Successfully retrieved clipboard content from server");
            println!("📄 Content length: {} bytes", content.len());

            // Write the content to the local clipboard
            match write_to_clipboard(&content) {
                Ok(_) => {
                    println!("🎉 Clipboard updated successfully!");
                    println!("💡 Your clipboard now contains the content from the server");
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ Failed to write to clipboard: {}", e);
                    eprintln!("💡 Tip: Make sure no other application is locking the clipboard");
                    Err(e)
                }
            }
        }
        Err(e) => {
            eprintln!("❌ Failed to retrieve clipboard content: {}", e);
            eprintln!("💡 Troubleshooting tips:");
            eprintln!("   1. Make sure the server is running at: {}", SERVER_URL);
            eprintln!("   2. Check if the server has received any content yet");
            eprintln!("   3. Verify your network connection");
            Err(e)
        }
    }
}

/// Fetches clipboard content from the server
async fn fetch_clipboard_content(client: &Client) -> Result<String> {
    let response = client
        .get(SERVER_URL)
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
            // Try to parse as JSON first, then fall back to plain text
            if let Ok(resp) = serde_json::from_str::<ClipboardResponse>(&response_text) {
                Ok(resp.content)
            } else {
                // If not JSON, use the response text directly
                Ok(response_text)
            }
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

/// Writes the given content to the system clipboard
fn write_to_clipboard(content: &str) -> Result<()> {
    let mut clipboard = Clipboard::new().context("Failed to access system clipboard")?;

    clipboard
        .set_text(content.to_string())
        .context("Failed to write to clipboard")?;

    Ok(())
}
