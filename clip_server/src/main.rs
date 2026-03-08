mod auth;
mod config;
mod handlers;
mod models;

use auth::{load_auth_token, AuthState, TOKEN_ENV_VAR};
use axum::Router;
use config::{DEFAULT_LOG_FILTER, DEFAULT_SERVER_ADDRESS, DEFAULT_SERVER_PORT};
use handlers::{create_router, ApiDoc, ClipboardState};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing subscriber for logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| DEFAULT_LOG_FILTER.into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load the authentication token from environment variable
    let auth_token = load_auth_token().unwrap_or_else(|err| {
        eprintln!("⚠️  WARNING: {}", err);
        eprintln!("📝 To set it up:");
        eprintln!("   1. Generate a token: cargo run --bin clip_token_gen");
        eprintln!("   2. Set the environment variable:");
        eprintln!("      export {}=\"your_generated_token\"", TOKEN_ENV_VAR);
        eprintln!();
        eprintln!("❌ Server cannot start without authentication token.");
        eprintln!(
            "💡 For testing purposes, you can use: export {}=\"test-token-123\"",
            TOKEN_ENV_VAR
        );
        std::process::exit(1);
    });

    info!("🔐 Authentication token loaded successfully");

    // Initialize the application state with empty content
    let clipboard_state: ClipboardState = Arc::new(std::sync::RwLock::new(None));
    let auth_state: AuthState = Arc::new(auth_token);

    // Build the clipboard API router with authentication middleware
    let clipboard_router = create_router()
        .layer(axum::middleware::from_fn_with_state(
            auth_state.clone(),
            auth::auth_middleware,
        ))
        .with_state(clipboard_state);

    // Combine clipboard router with Swagger UI
    let app = Router::new()
        .merge(clipboard_router)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()));

    // Bind to 0.0.0.0:3000 to accept connections from the local network
    let addr = SocketAddr::from(([0, 0, 0, 0], config::DEFAULT_SERVER_PORT));
    let listener = TcpListener::bind(addr).await?;

    info!(
        "🚀 Clipboard Server starting on http://{}:{}",
        DEFAULT_SERVER_ADDRESS, DEFAULT_SERVER_PORT
    );
    info!("📡 Server is accessible from your local Wi-Fi network");
    info!("🔒 Authentication is enabled - all requests require a valid Bearer token");
    info!("📝 Supporting content types: text, images, files");
    info!(
        "📚 API documentation available at http://{}:{}/swagger-ui",
        DEFAULT_SERVER_ADDRESS, DEFAULT_SERVER_PORT
    );

    // Start the server
    axum::serve(listener, app).await?;

    Ok(())
}

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_main_components() {
        // Test that all modules are accessible
        assert_eq!(TOKEN_ENV_VAR, "CLIPSHARE_TOKEN");
        assert_eq!(config::DEFAULT_SERVER_PORT, 3000);
        assert_eq!(DEFAULT_SERVER_ADDRESS, "0.0.0.0");
    }
}
