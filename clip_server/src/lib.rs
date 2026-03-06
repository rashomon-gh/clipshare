//! ClipShare Server Library
//! Modular clipboard server with authentication and multi-format support

pub mod auth;
pub mod config;
pub mod handlers;
pub mod models;

// Re-export commonly used types
pub use auth::{AuthState, TOKEN_ENV_VAR};
pub use handlers::ClipboardState;
pub use models::{AppError, ClipboardContent, ClipboardRequest, ErrorResponse, SuccessResponse};
