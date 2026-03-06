//! Configuration and command-line argument parsing

use std::env;
use std::time::Duration;

/// Configuration constants
pub const SERVER_URL: &str = "http://127.0.0.1:3000/clipboard";
pub const REQUEST_TIMEOUT: u64 = 5; // seconds
pub const TOKEN_ENV_VAR: &str = "CLIPSHARE_TOKEN";
pub const DEFAULT_POLL_INTERVAL: u64 = 2; // seconds

/// Command line arguments and configuration
#[derive(Debug, Clone)]
pub struct Args {
    pub poll_interval: Duration,
    pub one_shot: bool,
    pub verbose: bool,
}

impl Args {
    /// Parse arguments from environment variables
    pub fn from_env() -> Self {
        let poll_interval = env::var("CLIPSHARE_POLL_INTERVAL")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(DEFAULT_POLL_INTERVAL);

        let one_shot = env::var("CLIPSHARE_ONE_SHOT")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(false);

        let verbose = env::var("CLIPSHARE_VERBOSE")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or(false);

        Args {
            poll_interval: Duration::from_secs(poll_interval),
            one_shot,
            verbose,
        }
    }

    /// Create default arguments for testing
    #[cfg(test)]
    pub fn test_default() -> Self {
        Args {
            poll_interval: Duration::from_secs(1),
            one_shot: true,
            verbose: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_constants() {
        assert_eq!(SERVER_URL, "http://127.0.0.1:3000/clipboard");
        assert_eq!(REQUEST_TIMEOUT, 5);
        assert_eq!(TOKEN_ENV_VAR, "CLIPSHARE_TOKEN");
        assert_eq!(DEFAULT_POLL_INTERVAL, 2);
    }

    #[test]
    fn test_args_test_default() {
        let args = Args::test_default();
        assert_eq!(args.poll_interval, Duration::from_secs(1));
        assert!(args.one_shot);
        assert!(!args.verbose);
    }
}
