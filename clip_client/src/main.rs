mod api;
mod clipboard_ops;
mod config;
mod daemon;
mod models;

use anyhow::Result;
use config::Args;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::from_env();

    if args.one_shot {
        daemon::run_one_shot(args).await
    } else {
        daemon::run_daemon(args).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_args_parsing() {
        let args = Args::from_env();
        // Just ensure it doesn't panic
        assert!(args.poll_interval.as_secs() > 0);
    }
}
