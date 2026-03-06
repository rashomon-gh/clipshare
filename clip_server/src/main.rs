use axum::{
    extract::{Json, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use tokio::net::TcpListener;
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Application state that holds the clipboard content
/// Arc<RwLock<>> allows multiple concurrent reads/writes across async tasks
type ClipboardState = Arc<RwLock<Option<String>>>;

/// Request payload for POST /clipboard
#[derive(Debug, Deserialize, Serialize)]
struct ClipboardRequest {
    content: String,
}

/// Response payload for successful operations
#[derive(Debug, Serialize)]
struct SuccessResponse {
    status: String,
    message: String,
}

/// Error response for API errors
#[derive(Debug, Serialize)]
struct ErrorResponse {
    status: String,
    message: String,
}

/// Custom error type for application-specific errors
enum AppError {
    NoContent,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NoContent => (
                StatusCode::NOT_FOUND,
                "No clipboard content available".to_string(),
            ),
        };

        let body = Json(ErrorResponse {
            status: "error".to_string(),
            message,
        });

        (status, body).into_response()
    }
}

/// POST /clipboard endpoint
/// Receives clipboard content and stores it in the application state
async fn set_clipboard(
    State(state): State<ClipboardState>,
    Json(payload): Json<ClipboardRequest>,
) -> Result<Json<SuccessResponse>, StatusCode> {
    info!("Received clipboard content (length: {} bytes)", payload.content.len());

    // Acquire write lock and update the state
    match state.write() {
        Ok(mut guard) => {
            *guard = Some(payload.content.clone());
            Ok(Json(SuccessResponse {
                status: "success".to_string(),
                message: "Clipboard content updated successfully".to_string(),
            }))
        }
        Err(e) => {
            warn!("Failed to acquire write lock: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// GET /clipboard endpoint
/// Returns the currently stored clipboard content
async fn get_clipboard(State(state): State<ClipboardState>) -> Result<Json<String>, AppError> {
    // Acquire read lock and retrieve the content
    match state.read() {
        Ok(guard) => {
            if let Some(content) = guard.as_ref() {
                info!("Serving clipboard content (length: {} bytes)", content.len());
                Ok(Json(content.clone()))
            } else {
                warn!("Attempted to retrieve clipboard content, but none is available");
                Err(AppError::NoContent)
            }
        }
        Err(e) => {
            warn!("Failed to acquire read lock: {}", e);
            Err(AppError::NoContent)
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber for logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "clip_server=debug,tower_http=debug,axum=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize the application state with empty content
    let clipboard_state: ClipboardState = Arc::new(RwLock::new(None));

    // Build the application router
    let app = Router::new()
        .route("/clipboard", post(set_clipboard))
        .route("/clipboard", get(get_clipboard))
        .with_state(clipboard_state);

    // Bind to 0.0.0.0:3000 to accept connections from the local network
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = TcpListener::bind(addr).await?;

    info!("🚀 Clipboard Server starting on http://0.0.0.0:3000");
    info!("📡 Server is accessible from your local Wi-Fi network");

    // Start the server
    axum::serve(listener, app).await?;

    Ok(())
}
