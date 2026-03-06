//! Authentication middleware and token management

use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use std::sync::Arc;
use tracing::{info, warn};

/// Authentication state containing the expected token
pub type AuthState = Arc<String>;

/// Environment variable name for the authentication token
pub const TOKEN_ENV_VAR: &str = "CLIPSHARE_TOKEN";

/// Authentication middleware that validates the Bearer token
pub async fn auth_middleware(
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

/// Load authentication token from environment variable
pub fn load_auth_token() -> Result<String, String> {
    std::env::var(TOKEN_ENV_VAR).map_err(|_| {
        format!(
            "CLIPSHARE_TOKEN environment variable not set. \
             Please set it using: export {}=\"your_token\"",
            TOKEN_ENV_VAR
        )
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_token_env_var() {
        assert_eq!(TOKEN_ENV_VAR, "CLIPSHARE_TOKEN");
    }

    #[test]
    fn test_token_validation_logic() {
        // Test the Bearer token prefix validation logic
        let valid_token = "Bearer my_token";
        assert!(valid_token.starts_with("Bearer "));
        assert_eq!(&valid_token[7..], "my_token");

        let invalid_token = "InvalidFormat my_token";
        assert!(!invalid_token.starts_with("Bearer "));
    }

    #[test]
    fn test_load_auth_token_error() {
        // When env var is not set, should return error
        std::env::set_var(TOKEN_ENV_VAR, "");
        let result = load_auth_token();
        // Empty string is still valid, so this will succeed
        assert!(result.is_ok());

        std::env::remove_var(TOKEN_ENV_VAR);
        let result = load_auth_token();
        assert!(result.is_err());
    }
}
