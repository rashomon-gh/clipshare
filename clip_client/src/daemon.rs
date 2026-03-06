//! Daemon mode for continuous clipboard monitoring

use crate::{api::fetch_clipboard_content, clipboard_ops::process_clipboard_content, config::Args};
use anyhow::Result;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::time::interval;

/// Run the daemon in continuous monitoring mode
pub async fn run_daemon(args: Args) -> Result<()> {
    println!("🚀 ClipShare Daemon Starting");
    println!("📡 Monitoring server at: http://127.0.0.1:3000/clipboard");
    println!("⏱️  Poll interval: {} seconds", args.poll_interval.as_secs());
    println!("Press Ctrl+C to stop\n");

    let auth_token = crate::api::load_auth_token()?;
    let client = crate::api::create_client()?;

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

/// Run one-shot mode (fetch once and exit)
pub async fn run_one_shot(args: Args) -> Result<()> {
    println!("📋 Clipboard Client (One-Shot Mode)");
    println!("🔗 Connecting to server at: http://127.0.0.1:3000/clipboard");

    let auth_token = crate::api::load_auth_token()?;
    let client = crate::api::create_client()?;

    match fetch_clipboard_content(&client, &auth_token).await? {
        Some(content) => {
            println!("✅ Successfully retrieved clipboard content from server");
            process_clipboard_content(content, args.verbose)?;
            println!("🎉 Clipboard updated successfully!");
            Ok(())
        }
        None => {
            anyhow::bail!("No clipboard content available on the server");
        }
    }
}

/// Generate a simple timestamp for display
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp_format() {
        let ts = timestamp();
        assert!(ts.len() == 8); // HH:MM:SS format
        assert!(ts.contains(':'));
    }

    #[test]
    fn test_daemon_flow_control() {
        let running = Arc::new(AtomicBool::new(true));
        assert!(running.load(Ordering::SeqCst));

        running.store(false, Ordering::SeqCst);
        assert!(!running.load(Ordering::SeqCst));
    }
}
