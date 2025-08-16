//! Simple MP3 player example
//!
//! This example demonstrates basic MP3 playback functionality
//! without the full application interface.

use std::env;
use std::path::Path;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{error, info, Level};
use tracing_subscriber::FmtSubscriber;

use sonic_flow::simple_player::SimplePlayer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    
    tracing::subscriber::set_global_default(subscriber)
        .expect("Setting default subscriber failed");

    info!("Starting simple MP3 player example");

    // Get MP3 file path from command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <mp3_file_path>", args[0]);
        eprintln!("Example: {} /path/to/your/music.mp3", args[0]);
        std::process::exit(1);
    }

    let mp3_path = Path::new(&args[1]);

    // Create player
    let mut player = match SimplePlayer::new() {
        Ok(player) => {
            info!("Audio system initialized successfully");
            player
        }
        Err(e) => {
            error!("Failed to initialize audio system: {}", e);
            return Err(e.into());
        }
    };

    // Play the MP3 file
    match player.play_file(mp3_path).await {
        Ok(()) => {
            info!("MP3 playback started successfully");
            
            // Let it play for a few seconds
            info!("Playing for 10 seconds...");
            sleep(Duration::from_secs(10)).await;
            
            // Test pause/resume
            info!("Pausing playback...");
            player.pause();
            sleep(Duration::from_secs(2)).await;
            
            info!("Resuming playback...");
            player.resume();
            sleep(Duration::from_secs(5)).await;
            
            // Test volume control
            info!("Lowering volume to 30%...");
            player.set_volume(0.3);
            sleep(Duration::from_secs(3)).await;
            
            info!("Restoring volume to 80%...");
            player.set_volume(0.8);
            sleep(Duration::from_secs(2)).await;
            
            // Stop playback
            info!("Stopping playback...");
            player.stop();
            
            info!("MP3 playback test completed successfully");
        }
        Err(e) => {
            error!("Failed to play MP3 file: {}", e);
            return Err(e.into());
        }
    }

    Ok(())
}
