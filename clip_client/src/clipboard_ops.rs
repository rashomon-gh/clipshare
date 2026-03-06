//! Clipboard operations for different content types

use super::models::ClipboardContent;
use anyhow::{Context, Result};
use arboard::{Clipboard, ImageData};
use base64::prelude::*;

/// Writes text content to the system clipboard
pub fn write_text_to_clipboard(content: &str) -> Result<()> {
    let mut clipboard = Clipboard::new().context("Failed to access system clipboard")?;

    clipboard
        .set_text(content.to_string())
        .context("Failed to write text to clipboard")?;

    Ok(())
}

/// Writes image content to the system clipboard
/// Supports Windows, macOS, and Linux (X11/Wayland)
pub fn write_image_to_clipboard(base64_data: &str) -> Result<String> {
    let image_bytes = BASE64_STANDARD
        .decode(base64_data)
        .context("Failed to decode base64 image data")?;

    // Try to copy image to clipboard first
    let mut clipboard = Clipboard::new().context("Failed to access system clipboard")?;

    // Parse PNG to get dimensions and RGBA data
    let png_decoder = png::Decoder::new(image_bytes.as_slice());
    let mut reader = png_decoder.read_info().context("Failed to read PNG info")?;

    let width = reader.info().width;
    let height = reader.info().height;
    let mut buffer = vec![0; reader.output_buffer_size()];
    let _info = reader
        .next_frame(&mut buffer)
        .context("Failed to decode PNG frame")?;

    // Use the buffer directly for clipboard (assumes RGBA format)
    let image_data = ImageData {
        width: width as usize,
        height: height as usize,
        bytes: buffer.into(),
    };

    match clipboard.set_image(image_data) {
        Ok(_) => Ok("clipboard".to_string()),
        Err(e) => {
            // Fallback: save to file if clipboard copy fails
            eprintln!(
                "⚠️  Could not copy image to clipboard: {}. Saving to file instead.",
                e
            );
            let temp_path = format!("clipboard_image_{}.png", timestamp());
            std::fs::write(&temp_path, &image_bytes)
                .context("Failed to write image to temp file")?;
            Ok(temp_path)
        }
    }
}

/// Writes file content to disk
pub fn write_file_to_clipboard(filename: &str, base64_data: &str) -> Result<String> {
    let file_data = BASE64_STANDARD
        .decode(base64_data)
        .context("Failed to decode base64 file data")?;

    let safe_filename = filename
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect::<String>();

    let temp_path = format!("clipboard_file_{}", safe_filename);
    std::fs::write(&temp_path, &file_data).context("Failed to write file data")?;

    Ok(temp_path)
}

/// Process clipboard content based on its type
pub fn process_clipboard_content(content: ClipboardContent, verbose: bool) -> Result<()> {
    match content {
        ClipboardContent::Text { data } => {
            if verbose {
                println!("📄 Content type: Text (length: {} bytes)", data.len());
            }
            write_text_to_clipboard(&data)?;
            if !verbose {
                println!("💡 Text content ready to paste");
            }
        }
        ClipboardContent::Image { data, mime_type } => {
            if verbose {
                println!("🖼️  Content type: Image ({})", mime_type);
                println!("📊 Data size: {} bytes (base64 encoded)", data.len());
            }
            let result = write_image_to_clipboard(&data)?;
            if result == "clipboard" {
                println!("💡 Image copied to clipboard");
                if verbose {
                    println!("💡 Tip: Paste directly into any application");
                }
            } else {
                println!("💡 Image saved to: {}", result);
                if verbose {
                    println!("💡 Tip: Open the file to view the image");
                }
            }
        }
        ClipboardContent::File {
            name,
            data,
            mime_type,
        } => {
            if verbose {
                println!("📁 Content type: File ({})", mime_type);
                println!("📝 Filename: {}", name);
                println!("📊 Data size: {} bytes (base64 encoded)", data.len());
            }
            let path = write_file_to_clipboard(&name, &data)?;
            println!("💡 File saved to: {}", path);
            if verbose {
                println!("💡 Tip: The file has been saved to your current directory");
            }
        }
    }
    Ok(())
}

/// Generate a simple timestamp
fn timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("{}", duration.as_secs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timestamp_format() {
        let ts = timestamp();
        assert!(ts.parse::<u64>().is_ok());
    }

    #[test]
    fn test_filename_sanitization() {
        let filename = "test/file@name#.pdf";
        let safe: String = filename
            .chars()
            .map(|c| {
                if c.is_alphanumeric() || c == '.' || c == '-' || c == '_' {
                    c
                } else {
                    '_'
                }
            })
            .collect();
        assert_eq!(safe, "test_file_name_.pdf");
    }
}
