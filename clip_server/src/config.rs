//! Configuration constants for the clipboard server

/// Default server port
pub const DEFAULT_SERVER_PORT: u16 = 3000;

/// Default server address (binds to all interfaces)
pub const DEFAULT_SERVER_ADDRESS: &str = "0.0.0.0";

/// Default log filter for tracing
pub const DEFAULT_LOG_FILTER: &str = "clip_server=debug,tower_http=debug,axum=debug";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_constants() {
        assert_eq!(DEFAULT_SERVER_PORT, 3000);
        assert_eq!(DEFAULT_SERVER_ADDRESS, "0.0.0.0");
        assert!(DEFAULT_LOG_FILTER.contains("clip_server"));
    }
}
