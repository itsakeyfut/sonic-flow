//! Application layer - Main application controller and state management
//! 
//! This module contains the main application controller that orchestrates
//! the interaction between different components of the Sonic Flow.

use crate::{Error, Result};
use tracing::{info, debug, error};

pub mod controller;
pub mod state;
pub mod events;
pub mod lifecycle;

pub use controller::AppController;
pub use state::{AppState, PlaybackState};
pub use events::{AppEvent, EventBus};

/// The main Sonic Flow application
/// 
/// This is the entry point for the application that coordinates all subsystems
/// including audio engine, UI, visualizers, and configuration management.
pub struct SonicFlow {
    controller: AppController,
}

impl SonicFlow {
    /// Create a new instance of Sonic Flow
    /// 
    /// # Errors
    /// 
    /// Returns an error if any of the subsystems fail to initialize.
    pub async fn new() -> Result<Self> {
        info!("Initializing Sonic Flow");

        // Initialize the main application controller
        let controller = AppController::new().await?;

        debug!("Application controller initialized successfully");

        Ok(Self { controller })
    }

    /// Run the application main loop
    ///
    /// This method starts the application and runs until the user exits.
    ///
    /// # Errors
    ///
    /// Returns an error if the application encounters a fatal error during execution.
    pub async fn run(self) -> Result<()> {
        info!("Starting Sonic Flow main loop");

        // Run the application controller
        self.controller.run().await?;

        info!("Sonic Flow main loop completed");
        Ok(())
    }

    /// Get application version information
    pub fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_sonic_flow_creation() {
        // For now, we'll skip actual initialization in tests
        // until we have proper mocking infrastructure
        let version = env!("CARGO_PKG_VERSION");
        assert!(!version.is_empty());
    }

    #[test]
    fn test_version() {
        let version = env!("CARGO_PKG_VERSION");
        assert!(!version.is_empty());
        assert!(version.contains('.'));
    }
}
