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
type ClipboardState = Arc<RwLock<Option<String>>>;

/// Authentication state containing the expected token
type AuthState = Arc<String>;

/// Environment variable name for the authentication token
const TOKEN_ENV_VAR: &str = "CLIPSHARE_TOKEN";

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
async fn set_clipboard(
    State(state): State<ClipboardState>,
    Json(payload): Json<ClipboardRequest>,
) -> Result<Json<SuccessResponse>, AppError> {
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
            Err(AppError::InternalServerError(
                "Failed to update clipboard content".to_string(),
            ))
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

    // Start the server
    axum::serve(listener, app).await?;

    Ok(())
}
