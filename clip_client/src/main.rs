use anyhow::{Context, Result};
use arboard::Clipboard;
use reqwest::Client;
use serde::Deserialize;
use std::env;
use std::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::time::interval;

// Base64 encoding support
use base64::prelude::*;

/// Configuration for the clipboard client
const SERVER_URL: &str = "http://127.0.0.1:3000/clipboard";
const REQUEST_TIMEOUT: u64 = 5; // seconds
const TOKEN_ENV_VAR: &str = "CLIPSHARE_TOKEN";
const DEFAULT_POLL_INTERVAL: u64 = 2; // seconds

/// Response structure for clipboard content from the server
#[derive(Debug, Deserialize, Clone, PartialEq)]
#[serde(tag = "type", content = "data")]
enum ClipboardContent {
    #[serde(rename = "text")]
    Text(String),
    #[serde(rename = "image")]
    Image { data: String, mime_type: String },
    #[serde(rename = "file")]
    File { name: String, data: String, mime_type: String },
}

impl ClipboardContent {
    /// Get a unique hash of this content for change detection
    fn content_hash(&self) -> String {
        match self {
            ClipboardContent::Text(text) => format!("text:{}", text),
            ClipboardContent::Image { data, .. } => format!("image:{}", data),
            ClipboardContent::File { name, data, .. } => format!("file:{}:{}", name, data),
        }
    }
}

/// Command line arguments
struct Args {
    poll_interval: Duration,
    one_shot: bool,
    verbose: bool,
}

impl Args {
    fn from_env() -> Self {
        let poll_interval = env::var("CLIPSHARE_POLL_INTERVAL")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_POLL_INTERVAL);

        let one_shot = env::var("CLIPSHARE_ONE_SHOT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(false);

        let verbose = env::var("CLIPSHARE_VERBOSE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(false);

        Args {
            poll_interval: Duration::from_secs(poll_interval),
            one_shot,
            verbose,
        }
    }
}

/// Main function that runs the clipboard client
#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::from_env();

    if args.one_shot {
        run_one_shot(args).await
    } else {
        run_daemon(args).await
    }
}

/// One-shot mode: fetch clipboard content once and exit
async fn run_one_shot(args: Args) -> Result<()> {
    println!("📋 Clipboard Client (One-Shot Mode)");
    println!("🔗 Connecting to server at: {}", SERVER_URL);

    let auth_token = load_auth_token()?;
    let client = create_client()?;

    match fetch_and_process_clipboard(&client, &auth_token, args.verbose).await {
        Ok(_) => {
            println!("🎉 Clipboard updated successfully!");
            Ok(())
        }
        Err(e) => {
            eprintln!("❌ Failed to update clipboard: {}", e);
            Err(e)
        }
    }
}

/// Daemon mode: continuously monitor server and update clipboard
async fn run_daemon(args: Args) -> Result<()> {
    println!("🚀 ClipShare Daemon Starting");
    println!("📡 Monitoring server at: {}", SERVER_URL);
    println!("⏱️  Poll interval: {} seconds", args.poll_interval.as_secs());
    println!("Press Ctrl+C to stop\n");

    let auth_token = load_auth_token()?;
    let client = create_client()?;

    // Set up graceful shutdown
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c().await.unwrap();
        println!("\n🛑 Received shutdown signal");
        r.store(false, Ordering::SeqCst);
    });

    let mut last_hash = String::new();
    let mut poller = interval(args.poll_interval);
    poller.tick().await; // First tick completes immediately

    while running.load(Ordering::SeqCst) {
        poller.tick().await;

        if !running.load(Ordering::SeqCst) {
            break;
        }

        match fetch_clipboard_content(&client, &auth_token).await {
            Ok(Some(content)) => {
                let current_hash = content.content_hash();

                if current_hash != last_hash {
                    if args.verbose {
                        println!("🔄 New content detected!");
                    }

                    match process_clipboard_content(content.clone(), args.verbose) {
                        Ok(_) => {
                            last_hash = current_hash;
                            if args.verbose {
                                println!("✅ Clipboard updated at {}", timestamp());
                            } else {
                                println!("✅ {} - Clipboard updated", timestamp());
                            }
                        }
                        Err(e) => {
                            eprintln!("❌ Failed to process content: {}", e);
                        }
                    }
                } else if args.verbose {
                    println!("⏸️  No new content");
                }
            }
            Ok(None) => {
                // No content on server yet
                if args.verbose {
                    println!("⏸️  No content available on server");
                }
            }
            Err(e) => {
                eprintln!("❌ Failed to fetch content: {} (will retry)", e);
            }
        }
    }

    println!("👋 ClipShare Daemon stopped");
    Ok(())
}

/// Load authentication token from environment
fn load_auth_token() -> Result<String> {
    Ok(env::var(TOKEN_ENV_VAR).unwrap_or_else(|_| {
        eprintln!("⚠️  WARNING: {} environment variable not set!", TOKEN_ENV_VAR);
        eprintln!("📝 To set it up:");
        eprintln!("   1. Generate a token: cargo run --bin clip_token_gen");
        eprintln!("   2. Set the environment variable:");
        eprintln!("      export {}=\"your_generated_token\"", TOKEN_ENV_VAR);
        eprintln!();
        eprintln!("❌ Client cannot authenticate without the token.");
        std::process::exit(1);
    }))
}

/// Create HTTP client with timeout
fn create_client() -> Result<Client> {
    Client::builder()
        .timeout(Duration::from_secs(REQUEST_TIMEOUT))
        .build()
        .context("Failed to create HTTP client")
}

/// Fetch clipboard content and process it
async fn fetch_and_process_clipboard(
    client: &Client,
    auth_token: &str,
    verbose: bool,
) -> Result<()> {
    match fetch_clipboard_content(client, auth_token).await? {
        Some(content) => {
            println!("✅ Successfully retrieved clipboard content from server");
            process_clipboard_content(content, verbose)?;
            Ok(())
        }
        None => {
            anyhow::bail!("No clipboard content available on the server");
        }
    }
}

/// Fetches clipboard content from the server with authentication
async fn fetch_clipboard_content(
    client: &Client,
    auth_token: &str,
) -> Result<Option<ClipboardContent>> {
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

    match status.as_u16() {
        200 => {
            let content: ClipboardContent = serde_json::from_str(&response_text)
                .context("Failed to parse clipboard content")?;
            Ok(Some(content))
        }
        404 => Ok(None), // No content available
        401 => anyhow::bail!("Authentication failed - invalid or missing token"),
        500 => anyhow::bail!("Internal server error - check server logs"),
        _ => anyhow::bail!("Unexpected server response: {}", status),
    }
}

/// Process clipboard content based on its type
fn process_clipboard_content(content: ClipboardContent, verbose: bool) -> Result<()> {
    match content {
        ClipboardContent::Text(text) => {
            if verbose {
                println!("📄 Content type: Text (length: {} bytes)", text.len());
            }
            write_text_to_clipboard(&text)?;
        }
        ClipboardContent::Image { data, mime_type } => {
            if verbose {
                println!("🖼️  Content type: Image ({})", mime_type);
                println!("📊 Data size: {} bytes (base64 encoded)", data.len());
            }
            write_image_to_clipboard(&data)?;
        }
        ClipboardContent::File { name, data, mime_type } => {
            if verbose {
                println!("📁 Content type: File ({})", mime_type);
                println!("📝 Filename: {}", name);
                println!("📊 Data size: {} bytes (base64 encoded)", data.len());
            }
            write_file_to_clipboard(&name, &data, &mime_type)?;
        }
    }
    Ok(())
}

/// Writes text content to the system clipboard
fn write_text_to_clipboard(content: &str) -> Result<()> {
    let mut clipboard = Clipboard::new().context("Failed to access system clipboard")?;

    clipboard
        .set_text(content.to_string())
        .context("Failed to write text to clipboard")?;

    if !env::var("CLIPSHARE_VERBOSE").ok().and_then(|s| s.parse().ok()).unwrap_or(false) {
        println!("💡 Text content ready to paste");
    }
    Ok(())
}

/// Writes image content to a file (clipboard image support varies by platform)
fn write_image_to_clipboard(base64_data: &str) -> Result<()> {
    let image_data = BASE64_STANDARD
        .decode(base64_data)
        .context("Failed to decode base64 image data")?;

    let temp_path = format!("clipboard_image_{}.png", timestamp());
    std::fs::write(&temp_path, &image_data)
        .context("Failed to write image to temp file")?;

    println!("💡 Image saved to: {}", temp_path);
    println!("💡 Tip: Open the file to view the image");
    Ok(())
}

/// Writes file content to disk
fn write_file_to_clipboard(filename: &str, base64_data: &str, _mime_type: &str) -> Result<()> {
    let file_data = BASE64_STANDARD
        .decode(base64_data)
        .context("Failed to decode base64 file data")?;

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

/// Generate a simple timestamp
fn timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    let secs = duration.as_secs();
    let hours = (secs % 86400) / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}
