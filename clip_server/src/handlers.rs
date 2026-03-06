//! HTTP request handlers for the clipboard API

use super::models::{AppError, ClipboardContent, ClipboardRequest, SuccessResponse};
use axum::{extract::State, routing, Json, Router};
use std::sync::{Arc, RwLock};
use tracing::{info, warn};

/// Application state that holds the clipboard content
/// Arc<RwLock<>> allows multiple concurrent reads/writes across async tasks
pub type ClipboardState = Arc<RwLock<Option<ClipboardContent>>>;

/// POST /clipboard endpoint
/// Receives clipboard content and stores it in the application state
/// Supports text, images (base64 encoded), and files (base64 encoded)
pub async fn set_clipboard(
    State(state): State<ClipboardState>,
    Json(payload): Json<ClipboardRequest>,
) -> Result<Json<SuccessResponse>, AppError> {
    info!(
        "Received clipboard content (type: {}, data length: {} bytes)",
        payload.content_type,
        payload.data.len()
    );

    let content = if payload.content_type.starts_with("text/") {
        ClipboardContent::Text {
            data: payload.data.clone(),
        }
    } else if payload.content_type.starts_with("image/") {
        ClipboardContent::Image {
            data: payload.data.clone(),
            mime_type: payload.content_type.clone(),
        }
    } else {
        // Handle as file
        ClipboardContent::File {
            name: payload.filename.clone().unwrap_or_else(|| {
                // Simple extension mapping for common types
                let ext = if payload.content_type.contains("pdf") {
                    "pdf"
                } else if payload.content_type.contains("zip") {
                    "zip"
                } else if payload.content_type.contains("json") {
                    "json"
                } else {
                    "bin"
                };
                format!("unknown.{}", ext)
            }),
            data: payload.data.clone(),
            mime_type: payload.content_type.clone(),
        }
    };

    // Acquire write lock and update the state
    match state.write() {
        Ok(mut guard) => {
            *guard = Some(content);
            info!("Clipboard content updated successfully");
            Ok(Json(SuccessResponse::new(
                "Clipboard content updated successfully",
            )))
        }
        Err(e) => {
            warn!("Failed to acquire write lock: {}", e);
            Err(AppError::InternalServerError(
                "Failed to update clipboard content".to_string(),
            ))
        }
    }
}

/// GET /clipboard endpoint
/// Returns the currently stored clipboard content with appropriate content type
pub async fn get_clipboard(
    State(state): State<ClipboardState>,
) -> Result<Json<ClipboardContent>, AppError> {
    // Acquire read lock and retrieve the content
    match state.read() {
        Ok(guard) => {
            if let Some(content) = guard.as_ref() {
                info!(
                    "Serving clipboard content (type: {}, size: {} bytes)",
                    content.mime_type(),
                    content.size_bytes()
                );
                Ok(Json(content.clone()))
            } else {
                warn!("Attempted to retrieve clipboard content, but none is available");
                Err(AppError::NoContent)
            }
        }
        Err(e) => {
            warn!("Failed to acquire read lock: {}", e);
            Err(AppError::InternalServerError(
                "Failed to retrieve clipboard content".to_string(),
            ))
        }
    }
}

/// Create the router with all clipboard routes
pub fn create_router() -> Router<ClipboardState> {
    Router::new()
        .route("/clipboard", routing::post(set_clipboard))
        .route("/clipboard", routing::get(get_clipboard))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_clipboard_state_operations() {
        let state: ClipboardState = Arc::new(RwLock::new(None));

        // Test initial state is empty
        {
            let guard = state.read().unwrap();
            assert!(guard.is_none());
        }

        // Test writing text content
        {
            let mut guard = state.write().unwrap();
            *guard = Some(ClipboardContent::Text {
                data: "test content".to_string(),
            });
        }

        // Test reading content
        {
            let guard = state.read().unwrap();
            assert!(guard.is_some());
            if let Some(content) = guard.as_ref() {
                assert_eq!(content.mime_type(), "text/plain");
                assert_eq!(content.size_bytes(), 12);
            }
        }
    }

    #[test]
    fn test_clipboard_content_from_request() {
        let request = ClipboardRequest {
            content_type: "text/plain".to_string(),
            data: "test data".to_string(),
            filename: None,
        };

        assert_eq!(request.content_type, "text/plain");
        assert_eq!(request.data, "test data");
        assert!(request.filename.is_none());
    }
}
