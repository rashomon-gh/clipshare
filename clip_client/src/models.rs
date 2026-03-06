//! Data models for clipboard content

use serde::Deserialize;

/// Response structure for clipboard content from the server
#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
#[serde(tag = "type")]
pub enum ClipboardContent {
    #[serde(rename = "text")]
    Text { data: String },
    #[serde(rename = "image")]
    Image {
        data: String,
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
    #[serde(rename = "file")]
    File {
        name: String,
        data: String,
        #[serde(rename = "mimeType")]
        mime_type: String,
    },
}

impl ClipboardContent {
    /// Get a unique hash of this content for change detection
    pub fn content_hash(&self) -> String {
        match self {
            ClipboardContent::Text { data } => format!("text:{}", data),
            ClipboardContent::Image { data, .. } => format!("image:{}", data),
            ClipboardContent::File { name, data, .. } => format!("file:{}:{}", name, data),
        }
    }

    /// Get the data length for display
    #[allow(dead_code)]
    pub fn data_length(&self) -> usize {
        match self {
            ClipboardContent::Text { data } => data.len(),
            ClipboardContent::Image { data, .. } => data.len(),
            ClipboardContent::File { data, .. } => data.len(),
        }
    }

    /// Get the MIME type
    #[allow(dead_code)]
    pub fn mime_type(&self) -> &str {
        match self {
            ClipboardContent::Text { .. } => "text/plain",
            ClipboardContent::Image { mime_type, .. } => mime_type,
            ClipboardContent::File { mime_type, .. } => mime_type,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_content_hash() {
        let text1 = ClipboardContent::Text {
            data: "hello".to_string(),
        };
        let text2 = ClipboardContent::Text {
            data: "hello".to_string(),
        };
        let text3 = ClipboardContent::Text {
            data: "world".to_string(),
        };

        assert_eq!(text1.content_hash(), text2.content_hash());
        assert_ne!(text1.content_hash(), text3.content_hash());
    }

    #[test]
    fn test_clipboard_content_data_length() {
        let text = ClipboardContent::Text {
            data: "hello world".to_string(),
        };
        assert_eq!(text.data_length(), 11);
    }

    #[test]
    fn test_clipboard_content_mime_type() {
        let text = ClipboardContent::Text {
            data: "test".to_string(),
        };
        assert_eq!(text.mime_type(), "text/plain");

        let image = ClipboardContent::Image {
            data: "abc".to_string(),
            mime_type: "image/png".to_string(),
        };
        assert_eq!(image.mime_type(), "image/png");
    }
}
