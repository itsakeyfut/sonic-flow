//! Sonic Flow - Main entry point
//!
//! A high-quality music player with advanced audio spectrum visualizers.

// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;

use sonic_flow::{Result, SonicFlow};
use tracing::{error, info};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

// slint::include_modules!();

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging system
    init_logging()?;

    info!("Starting Sonic Flow v{}", env!("CARGO_PKG_VERSION"));

    // Handle application lifecycle
    let result = run_application().await;

    match &result {
        Ok(()) => info!("Sonic Flow shut down successfully"),
        Err(e) => {
            error!("Application error: {}", e);

            // Log error chain for debugging
            let mut source = e.source();
            while let Some(err) = source {
                error!("  Caused by: {}", err);
                source = err.source();
            }
        }
    }

    result
}

async fn run_application() -> Result<()> {
    // Create and initialize the main application
    let player = SonicFlow::new().await?;

    // Run the application
    player.run().await?;

    Ok(())
}

fn init_logging() -> Result<()> {
    use tracing_subscriber::fmt;

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        // Default logging configuration
        #[cfg(debug_assertions)]
        return EnvFilter::new("sonic_flow=debug,warn");

        #[cfg(not(debug_assertions))]
        return EnvFilter::new("sonic_flow=info,warn");
    });

    let subscriber = tracing_subscriber::registry().with(env_filter).with(
        fmt::layer()
            .with_target(true)
            .with_thread_ids(true)
            .with_line_number(true)
            .with_file(cfg!(debug_assertions)),
    );

    // Add file logging in release mode
    #[cfg(not(debug_assertions))]
    let subscriber = {
        use std::path::Path;
        use tracing_appender::rolling::{RollingFileAppender, Rotation};

        if let Some(config_dir) = dirs::config_dir() {
            let log_dir = config_dir.join("sonic-flow").join("logs");

            if std::fs::create_dir_all(&log_dir).is_ok() {
                let file_appender =
                    RollingFileAppender::new(Rotation::daily(), log_dir, "sonic-flow.log");

                let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

                subscriber.with(
                    fmt::layer()
                        .with_writer(non_blocking)
                        .with_ansi(false)
                        .json(),
                )
            } else {
                warn!("Failed to create log directory, using console only");
                subscriber
            }
        } else {
            warn!("Could not determine config directory, using console only");
            subscriber
        }
    };

    subscriber.try_init().map_err(|e| {
        eprintln!("Failed to initialize logging: {}", e);
        sonic_flow::Error::Application("Logging initialization failed".to_string())
    })?;

    info!("Logging system initialized");
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_main_module_compiles() {
        // This test ensures the main module compiles correctly
        // More comprehensive tests will be in integration tests
        assert!(true);
    }
}
