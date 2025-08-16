//! Integrated music player with event loop based UI-Audio integration
//!
//! This example demonstrates the complete integration of Slint UI with the
//! audio system using proper event loop based architecture.

use tracing::{Level, info, error};
use tracing_subscriber::FmtSubscriber;

use sonic_flow::ui::EnhancedMainWindowBinding;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    
    tracing::subscriber::set_global_default(subscriber)
        .expect("Setting default subscriber failed");

    info!("Starting Sonic Flow Integrated Music Player");

    // Create enhanced UI binding with event loop based architecture
    let ui_binding = match EnhancedMainWindowBinding::new() {
        Ok(binding) => {
            info!("Integrated UI binding created successfully");
            binding
        }
        Err(e) => {
            error!("Failed to create enhanced UI binding: {}", e);
            return Err(e.into());
        }
    };

    // Show some information about the architecture
    info!("Architecture: Event loop based UI-Audio integration");
    info!("Threading: invoke_from_event_loop for UI updates");
    info!("Communication: Channel based message passing");
    info!("Audio Engine: Thread-safe PlayerManager");

    // Run the UI - this blocks until the window is closed
    info!("Starting enhanced UI main loop");
    match ui_binding.run() {
        Ok(()) => {
            info!("UI closed successfully");
        }
        Err(e) => {
            error!("UI error: {}", e);
            return Err(e.into());
        }
    }

    info!("Sonic Flow Integrated Music Player shutdown complete");
    Ok(())
}
