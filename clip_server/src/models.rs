//! Data models and types for the clipboard server

use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Supported content types for clipboard data
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", content = "data")]
pub enum ClipboardContent {
    #[serde(rename = "text")]
    Text(String),
    #[serde(rename = "image")]
    Image { data: String, mime_type: String },
    #[serde(rename = "file")]
    File { name: String, data: String, mime_type: String },
}

impl ClipboardContent {
    /// Get the MIME type for this content
    pub fn mime_type(&self) -> &str {
        match self {
            ClipboardContent::Text(_) => "text/plain",
            ClipboardContent::Image { mime_type, .. } => mime_type,
            ClipboardContent::File { mime_type, .. } => mime_type,
        }
    }

    /// Get the size in bytes (estimated for base64 encoded data)
    pub fn size_bytes(&self) -> usize {
        match self {
            ClipboardContent::Text(text) => text.len(),
            ClipboardContent::Image { data, .. } => data.len(),
            ClipboardContent::File { data, .. } => data.len(),
        }
    }

    /// Get a unique hash of this content for change detection
    pub fn content_hash(&self) -> String {
        match self {
            ClipboardContent::Text(text) => format!("text:{}", text),
            ClipboardContent::Image { data, .. } => format!("image:{}", data),
            ClipboardContent::File { name, data, .. } => format!("file:{}:{}", name, data),
        }
    }
}

impl fmt::Display for ClipboardContent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ClipboardContent::Text(text) => write!(f, "Text ({} bytes)", text.len()),
            ClipboardContent::Image { mime_type, .. } => write!(f, "Image ({})", mime_type),
            ClipboardContent::File { name, mime_type, .. } => write!(f, "File {} ({})", name, mime_type),
        }
    }
}

/// Request payload for POST /clipboard (supports all content types)
#[derive(Debug, Deserialize, Serialize)]
pub struct ClipboardRequest {
    #[serde(rename = "contentType")]
    pub content_type: String,
    #[serde(rename = "data")]
    pub data: String,
    #[serde(rename = "filename")]
    pub filename: Option<String>,
}

/// Response payload for successful operations
#[derive(Debug, Serialize)]
pub struct SuccessResponse {
    pub status: String,
    pub message: String,
}

impl SuccessResponse {
    pub fn new(message: &str) -> Self {
        Self {
            status: "success".to_string(),
            message: message.to_string(),
        }
    }
}

/// Error response for API errors
#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub status: String,
    pub message: String,
}

impl ErrorResponse {
    pub fn new(message: &str) -> Self {
        Self {
            status: "error".to_string(),
            message: message.to_string(),
        }
    }
}

/// Custom error type for application-specific errors
#[derive(Debug)]
pub enum AppError {
    NoContent,
    InternalServerError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::NoContent => write!(f, "No clipboard content available"),
            AppError::InternalServerError(msg) => write!(f, "Internal server error: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NoContent => (StatusCode::NOT_FOUND, "No clipboard content available".to_string()),
            AppError::InternalServerError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(ErrorResponse::new(&message));
        (status, body).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_content_mime_type() {
        let text = ClipboardContent::Text("hello".to_string());
        assert_eq!(text.mime_type(), "text/plain");

        let image = ClipboardContent::Image {
            data: "abc".to_string(),
            mime_type: "image/png".to_string(),
        };
        assert_eq!(image.mime_type(), "image/png");
    }

    #[test]
    fn test_clipboard_content_size() {
        let text = ClipboardContent::Text("hello".to_string());
        assert_eq!(text.size_bytes(), 5);

        let image = ClipboardContent::Image {
            data: "abcdef".to_string(),
            mime_type: "image/png".to_string(),
        };
        assert_eq!(image.size_bytes(), 6);
    }

    #[test]
    fn test_clipboard_content_hash() {
        let text1 = ClipboardContent::Text("hello".to_string());
        let text2 = ClipboardContent::Text("hello".to_string());
        let text3 = ClipboardContent::Text("world".to_string());

        assert_eq!(text1.content_hash(), text2.content_hash());
        assert_ne!(text1.content_hash(), text3.content_hash());
    }

    #[test]
    fn test_success_response_creation() {
        let response = SuccessResponse::new("Test message");
        assert_eq!(response.status, "success");
        assert_eq!(response.message, "Test message");
    }

    #[test]
    fn test_error_response_creation() {
        let response = ErrorResponse::new("Error message");
        assert_eq!(response.status, "error");
        assert_eq!(response.message, "Error message");
    }

    #[test]
    fn test_app_error_display() {
        let err = AppError::NoContent;
        assert_eq!(format!("{}", err), "No clipboard content available");

        let err = AppError::InternalServerError("test error".to_string());
        assert_eq!(format!("{}", err), "Internal server error: test error");
    }
}
