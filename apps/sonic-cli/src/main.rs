//! # Sonic CLI
//!
//! Command-line interface for Sonic Flow music player.
//!
//! This binary provides a command-line interface for playing music,
//! managing playlists, and controlling audio playback.

use clap::{Parser, Subcommand};
use sonic_core::{AudioEngine, Result, PlaybackControl, VolumeControl, TrackLoader};
use std::path::PathBuf;
use tracing::{info, warn};

#[derive(Parser)]
#[command(name = "sonic-cli")]
#[command(about = "Sonic Flow Music Player CLI")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Play a single audio file
    Play {
        /// Path to the audio file
        file: PathBuf,
        
        /// Volume level (0.0 to 1.0)
        #[arg(short, long, default_value = "0.5")]
        volume: f32,
    },
    
    /// Play all audio files in a directory
    PlayDir {
        /// Path to the directory containing audio files
        dir: PathBuf,
        
        /// Volume level (0.0 to 1.0)
        #[arg(short, long, default_value = "0.5")]
        volume: f32,
    },
    
    /// List supported audio formats
    Formats,
    
    /// Show audio file metadata
    Info {
        /// Path to the audio file
        file: PathBuf,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Play { file, volume } => {
            play_file(&file, volume).await?;
        }
        Commands::PlayDir { dir, volume } => {
            play_directory(&dir, volume).await?;
        }
        Commands::Formats => {
            list_formats();
        }
        Commands::Info { file } => {
            show_file_info(&file).await?;
        }
    }
    
    Ok(())
}

async fn play_file(file: &PathBuf, volume: f32) -> Result<()> {
    info!("Playing file: {}", file.display());
    
    let mut engine = AudioEngine::new()?;
    engine.load_track(file).await?;
    engine.set_volume(volume);
    engine.play().await?;
    
    info!("Press Ctrl+C to stop playback");
    
    // Keep the application running
    tokio::signal::ctrl_c().await.map_err(|e| {
        sonic_core::Error::Application(format!("Failed to wait for Ctrl+C: {}", e))
    })?;
    
    engine.stop().await?;
    info!("Playback stopped");
    
    Ok(())
}

async fn play_directory(dir: &PathBuf, _volume: f32) -> Result<()> {
    info!("Playing directory: {}", dir.display());
    
    // TODO: Implement directory playback
    warn!("Directory playback not yet implemented");
    
    Ok(())
}

fn list_formats() {
    println!("Supported audio formats:");
    println!("  - MP3 (.mp3)");
    println!("  - FLAC (.flac)");
    println!("  - WAV (.wav)");
    println!("  - OGG (.ogg)");
    println!("  - AAC (.aac, .m4a)");
}

async fn show_file_info(file: &PathBuf) -> Result<()> {
    info!("Getting file info: {}", file.display());
    
    // TODO: Implement metadata extraction
    warn!("File info not yet implemented");
    
    Ok(())
}
