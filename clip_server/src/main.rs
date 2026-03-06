use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use std::env;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};
use tokio::net::TcpListener;
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Application state that holds the clipboard content
/// Arc<RwLock<>> allows multiple concurrent reads/writes across async tasks
type ClipboardState = Arc<RwLock<Option<ClipboardContent>>>;

/// Authentication state containing the expected token
type AuthState = Arc<String>;

/// Environment variable name for the authentication token
const TOKEN_ENV_VAR: &str = "CLIPSHARE_TOKEN";

/// Supported content types for clipboard data
#[derive(Debug, Clone, Serialize, Deserialize)]
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
    /// Get the MIME type for this content
    fn mime_type(&self) -> &str {
        match self {
            ClipboardContent::Text(_) => "text/plain",
            ClipboardContent::Image { mime_type, .. } => mime_type,
            ClipboardContent::File { mime_type, .. } => mime_type,
        }
    }

    /// Get the size in bytes (estimated for base64 encoded data)
    fn size_bytes(&self) -> usize {
        match self {
            ClipboardContent::Text(text) => text.len(),
            ClipboardContent::Image { data, .. } => data.len(),
            ClipboardContent::File { data, .. } => data.len(),
        }
    }
}

/// Request payload for POST /clipboard (supports all content types)
#[derive(Debug, Deserialize, Serialize)]
struct ClipboardRequest {
    #[serde(rename = "contentType")]
    content_type: String,
    #[serde(rename = "data")]
    data: String,
    #[serde(rename = "filename")]
    filename: Option<String>,
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
    InternalServerError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            AppError::NoContent => (
                StatusCode::NOT_FOUND,
                "No clipboard content available".to_string(),
            ),
            AppError::InternalServerError(msg) => (StatusCode::INTERNAL_SERVER_ERROR, msg),
        };

        let body = Json(ErrorResponse {
            status: "error".to_string(),
            message,
        });

        (status, body).into_response()
    }
}

/// Authentication middleware that validates the Bearer token
async fn auth_middleware(
    State(expected_token): State<AuthState>,
    headers: HeaderMap,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    // Extract the Authorization header
    let auth_header = headers
        .get("Authorization")
        .and_then(|header| header.to_str().ok())
        .ok_or_else(|| {
            warn!("Unauthorized request: Missing Authorization header");
            StatusCode::UNAUTHORIZED
        })?;

    // Validate the Bearer token format
    if !auth_header.starts_with("Bearer ") {
        warn!("Unauthorized request: Invalid Authorization header format");
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Extract and validate the token
    let provided_token = &auth_header[7..]; // Skip "Bearer "
    if provided_token != expected_token.as_str() {
        warn!("Unauthorized request: Invalid token");
        return Err(StatusCode::UNAUTHORIZED);
    }

    // Token is valid, proceed with the request
    info!("Request authenticated successfully");
    Ok(next.run(request).await)
}

/// POST /clipboard endpoint
/// Receives clipboard content and stores it in the application state
/// Supports text, images (base64 encoded), and files (base64 encoded)
async fn set_clipboard(
    State(state): State<ClipboardState>,
    Json(payload): Json<ClipboardRequest>,
) -> Result<Json<SuccessResponse>, AppError> {
    info!(
        "Received clipboard content (type: {}, data length: {} bytes)",
        payload.content_type,
        payload.data.len()
    );

    let content = if payload.content_type.starts_with("text/") {
        ClipboardContent::Text(payload.data.clone())
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
            Ok(Json(SuccessResponse {
                status: "success".to_string(),
                message: "Clipboard content updated successfully".to_string(),
            }))
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
async fn get_clipboard(State(state): State<ClipboardState>) -> Result<Json<ClipboardContent>, AppError> {
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

    // Load the authentication token from environment variable
    let auth_token = env::var(TOKEN_ENV_VAR).unwrap_or_else(|_| {
        eprintln!("⚠️  WARNING: {} environment variable not set!", TOKEN_ENV_VAR);
        eprintln!("📝 To set it up:");
        eprintln!("   1. Generate a token: cargo run --bin clip_token_gen");
        eprintln!("   2. Set the environment variable:");
        eprintln!("      export {}=\"your_generated_token\"", TOKEN_ENV_VAR);
        eprintln!();
        eprintln!("❌ Server cannot start without authentication token.");
        eprintln!("💡 For testing purposes, you can use: export {}=\"test-token-123\"", TOKEN_ENV_VAR);
        std::process::exit(1);
    });

    info!("🔐 Authentication token loaded successfully");

    // Initialize the application state with empty content
    let clipboard_state: ClipboardState = Arc::new(RwLock::new(None));
    let auth_state: AuthState = Arc::new(auth_token);

    // Build the application router with authentication middleware
    let app = Router::new()
        .route("/clipboard", post(set_clipboard))
        .route("/clipboard", get(get_clipboard))
        .layer(axum::middleware::from_fn_with_state(
            auth_state.clone(),
            auth_middleware,
        ))
        .with_state(clipboard_state);

    // Bind to 0.0.0.0:3000 to accept connections from the local network
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    let listener = TcpListener::bind(addr).await?;

    info!("🚀 Clipboard Server starting on http://0.0.0.0:3000");
    info!("📡 Server is accessible from your local Wi-Fi network");
    info!("🔒 Authentication is enabled - all requests require a valid Bearer token");
    info!("📝 Supporting content types: text, images, files");

    // Start the server
    axum::serve(listener, app).await?;

    Ok(())
}
