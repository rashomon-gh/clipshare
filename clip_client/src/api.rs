//! API communication with the clipboard server

use super::config::{REQUEST_TIMEOUT, SERVER_URL, TOKEN_ENV_VAR};
use super::models::ClipboardContent;
use anyhow::{Context, Result};
use reqwest::Client;
use std::env;
use std::time::Duration;

/// Create HTTP client with timeout
pub fn create_client() -> Result<Client> {
    Client::builder()
        .timeout(Duration::from_secs(REQUEST_TIMEOUT))
        .build()
        .context("Failed to create HTTP client")
}

/// Load authentication token from environment
pub fn load_auth_token() -> Result<String> {
    Ok(env::var(TOKEN_ENV_VAR).unwrap_or_else(|_| {
        eprintln!(
            "⚠️  WARNING: {} environment variable not set!",
            TOKEN_ENV_VAR
        );
        eprintln!("📝 To set it up:");
        eprintln!("   1. Generate a token: cargo run --bin clip_token_gen");
        eprintln!("   2. Set the environment variable:");
        eprintln!("      export {}=\"your_generated_token\"", TOKEN_ENV_VAR);
        eprintln!();
        eprintln!("❌ Client cannot authenticate without the token.");
        std::process::exit(1);
    }))
}

/// Fetches clipboard content from the server with authentication
pub async fn fetch_clipboard_content(
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_access() {
        assert_eq!(SERVER_URL, "http://127.0.0.1:3000/clipboard");
        assert_eq!(REQUEST_TIMEOUT, 5);
        assert_eq!(TOKEN_ENV_VAR, "CLIPSHARE_TOKEN");
    }

    #[test]
    fn test_client_creation() {
        let client = create_client();
        assert!(client.is_ok());
    }
}
