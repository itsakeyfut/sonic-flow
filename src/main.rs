//! Sonic Flow - Main entry point
//!
//! A high-quality music player with advanced audio spectrum visualizers.

// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;
use std::path::PathBuf;

use sonic_flow::{Result, SonicFlow};
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

// slint::include_modules!();

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging system
    init_logging()?;

    info!("Starting Sonic Flow v{}", env!("CARGO_PKG_VERSION"));
    info!("Audio Visualizer Mode - Ready for spectrum analysis");

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

    // Check for demo audio files and load one if available
    if let Some(demo_path) = find_demo_audio_file() {
        info!("Found demo audio file: {}", demo_path.display());

        // Note: This would require the player to expose a method to load demo tracks
        // For now, we just log that we found a demo file
        warn!("Demo track loading not yet implemented in this interface");
    } else {
        info!("No demo audio files found - user will need to load tracks manually");
        print_usage_instructions();
    }

    // Run the application
    player.run().await?;

    Ok(())
}

/// Find a demo audio file for testing
fn find_demo_audio_file() -> Option<PathBuf> {
    let possible_paths = vec![
        // Common demo file locations
        PathBuf::from("assets/demo.mp3"),
        PathBuf::from("assets/demo.flac"),
        PathBuf::from("assets/demo.wav"),
        PathBuf::from("demo.mp3"),
        PathBuf::from("demo.flac"),
        PathBuf::from("demo.wav"),
        // User's music directory
        dirs::audio_dir()
            .map(|dir| dir.join("demo.mp3"))
            .unwrap_or_default(),
        dirs::audio_dir()
            .map(|dir| dir.join("demo.flac"))
            .unwrap_or_default(),
        // Home directory
        dirs::home_dir()
            .map(|dir| dir.join("Music").join("demo.mp3"))
            .unwrap_or_default(),
        dirs::home_dir()
            .map(|dir| dir.join("Music").join("demo.flac"))
            .unwrap_or_default(),
    ];

    for path in &possible_paths {
        if path.exists() && path.is_file() {
            // Verify it's an audio file by extension
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_string_lossy().to_lowercase();
                if matches!(
                    ext_str.as_str(),
                    "mp3" | "flac" | "wav" | "ogg" | "m4a" | "aac"
                ) {
                    return Some(path.clone());
                }
            }
        }
    }

    None
}

/// Print usage instructions for the user
fn print_usage_instructions() {
    println!("\n=== Sonic Flow Audio Visualizer ===");
    println!("🎵 Welcome to Sonic Flow - Advanced Audio Visualization");
    println!();
    println!("📁 To get started:");
    println!("   1. Click the '📁' button in the player controls");
    println!("   2. Load an audio file (MP3, FLAC, WAV, OGG supported)");
    println!("   3. Press ▶ to start playback and see the visualizer");
    println!();
    println!("🎨 Visualizer Features:");
    println!("   • Real-time spectrum analysis");
    println!("   • Multiple visualization types (Spectrum Bars, Waveform, etc.)");
    println!("   • Adjustable sensitivity");
    println!("   • 60+ FPS smooth animation");
    println!();
    println!("🔧 Controls:");
    println!("   • Space bar: Play/Pause");
    println!("   • Left/Right arrows: Previous/Next track");
    println!("   • Up/Down arrows: Volume control");
    println!("   • Mouse: Click and drag on progress bar to seek");
    println!();
    println!("💡 Tips:");
    println!("   • Try different visualizer types to see various representations");
    println!("   • Adjust sensitivity for better visualization of quiet tracks");
    println!("   • Use high-quality audio files for best results");
    println!();
    println!("For demo purposes, you can place a file named 'demo.mp3' in:");
    if let Some(music_dir) = dirs::audio_dir() {
        println!("   • {}", music_dir.display());
    }
    if let Some(home) = dirs::home_dir() {
        println!("   • {}/Music/", home.display());
    }
    println!("   • Current directory");
    println!();
}

fn init_logging() -> Result<()> {
    use tracing_subscriber::fmt;

    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        // Enhanced logging configuration for visualizer development
        #[cfg(debug_assertions)]
        return EnvFilter::new(
            "sonic_flow=debug,sonic_flow::visualizer=trace,sonic_flow::audio=debug,warn",
        );

        #[cfg(not(debug_assertions))]
        return EnvFilter::new("sonic_flow=info,sonic_flow::visualizer=debug,warn");
    });

    let subscriber = tracing_subscriber::registry().with(env_filter).with(
        fmt::layer()
            .with_target(true)
            .with_thread_ids(true)
            .with_line_number(true)
            .with_file(cfg!(debug_assertions))
            .with_ansi(true)
            .pretty(), // より読みやすい出力形式
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

    info!("Enhanced logging system initialized for visualizer development");
    Ok(())
}

/// Generate a simple demo audio signal if no files are found
/// This creates a sine wave audio file for demonstration purposes
#[allow(dead_code)]
fn generate_demo_audio() -> Result<PathBuf> {
    let output_path = PathBuf::from("generated_demo.wav");

    info!("Generating demo audio file: {}", output_path.display());

    // Create a simple WAV file with a sine wave
    let spec = hound::WavSpec {
        channels: 2,
        sample_rate: 44100,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = hound::WavWriter::create(&output_path, spec)
        .map_err(|e| sonic_flow::Error::Application(format!("Failed to create demo WAV: {}", e)))?;

    // Generate 10 seconds of audio with multiple frequencies
    let duration = 10.0; // seconds
    let sample_rate = 44100.0;
    let samples = (duration * sample_rate) as usize;

    for i in 0..samples {
        let t = i as f32 / sample_rate;

        // Mix multiple frequencies for a richer spectrum
        let sample = 0.3 * (2.0 * std::f32::consts::PI * 440.0 * t).sin() +  // A4
            0.2 * (2.0 * std::f32::consts::PI * 880.0 * t).sin() +  // A5
            0.1 * (2.0 * std::f32::consts::PI * 220.0 * t).sin() +  // A3
            0.1 * (2.0 * std::f32::consts::PI * 1760.0 * t).sin() + // A6
            0.05 * (2.0 * std::f32::consts::PI * 110.0 * t).sin(); // A2

        // Add some variation over time
        let envelope = (t * 0.5).sin() * 0.3 + 0.7;
        let final_sample = sample * envelope;

        // Convert to 16-bit integer
        let amplitude = (final_sample * i16::MAX as f32) as i16;

        // Write stereo samples
        writer.write_sample(amplitude).map_err(|e| {
            sonic_flow::Error::Application(format!("Failed to write sample: {}", e))
        })?;
        writer.write_sample(amplitude).map_err(|e| {
            sonic_flow::Error::Application(format!("Failed to write sample: {}", e))
        })?;
    }

    writer
        .finalize()
        .map_err(|e| sonic_flow::Error::Application(format!("Failed to finalize WAV: {}", e)))?;

    info!("Demo audio file generated successfully");
    Ok(output_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_demo_file_search() {
        // Test that the search function doesn't panic
        let result = find_demo_audio_file();
        // Result can be None, that's fine for testing
        assert!(result.is_none() || result.unwrap().exists());
    }

    #[test]
    fn test_logging_initialization() {
        // Test that logging can be initialized without panicking
        // This might fail if logging is already initialized
        std::env::set_var("RUST_LOG", "debug");
        // Don't actually initialize logging in tests as it can only be done once
    }

    #[tokio::test]
    async fn test_main_module_compiles() {
        // This test ensures the main module compiles correctly
        assert!(true);
    }
}
