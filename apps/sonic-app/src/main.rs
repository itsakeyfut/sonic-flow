//! Sonic Flow - Main entry point
//!
//! A high-quality music player with advanced audio spectrum visualizers.

// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::error::Error;
use std::path::PathBuf;

use sonic_ui::MainWindowBinding;
use sonic_core::Result;
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

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

/// Load default visualization shaders
fn load_default_visualization_shaders(player: &mut sonic_ui::MainWindowBinding) -> Result<()> {
    info!("Loading default visualization shaders");
    
    // Load spectrum bars shader
    let spectrum_bars_shader = r#"
        // Simple spectrum bars vertex shader
        struct VertexInput {
            @location(0) position: vec3<f32>,
            @location(1) uv: vec2<f32>,
        };
        
        struct VertexOutput {
            @builtin(position) position: vec4<f32>,
            @location(0) uv: vec2<f32>,
        };
        
        @vertex
        fn vertexMain(input: VertexInput) -> VertexOutput {
            var output: VertexOutput;
            output.position = vec4<f32>(input.position, 1.0);
            output.uv = input.uv;
            return output;
        }
        
        @fragment
        fn fragmentMain(input: VertexOutput) -> @location(0) vec4<f32> {
            let intensity = input.uv.y;
            return vec4<f32>(intensity, 0.5, 1.0 - intensity, 1.0);
        }
    "#;
    
    if let Err(e) = player.load_visualization_shader(
        "spectrum_bars",
        spectrum_bars_shader,
        "vertexMain",
        "fragmentMain"
    ) {
        warn!("Failed to load spectrum bars shader: {}", e);
    }
    
    // Load waveform shader
    let waveform_shader = r#"
        // Simple waveform vertex shader
        struct VertexInput {
            @location(0) position: vec3<f32>,
            @location(1) uv: vec2<f32>,
        };
        
        struct VertexOutput {
            @builtin(position) position: vec4<f32>,
            @location(0) uv: vec2<f32>,
        };
        
        @vertex
        fn vertexMain(input: VertexInput) -> VertexOutput {
            var output: VertexOutput;
            output.position = vec4<f32>(input.position, 1.0);
            output.uv = input.uv;
            return output;
        }
        
        @fragment
        fn fragmentMain(input: VertexOutput) -> @location(0) vec4<f32> {
            let wave = sin(input.uv.x * 10.0) * 0.5 + 0.5;
            let alpha = step(input.uv.y, wave);
            return vec4<f32>(0.2, 0.8, 1.0, alpha);
        }
    "#;
    
    if let Err(e) = player.load_visualization_shader(
        "waveform",
        waveform_shader,
        "vertexMain",
        "fragmentMain"
    ) {
        warn!("Failed to load waveform shader: {}", e);
    }
    
    info!("Default visualization shaders loaded");
    Ok(())
}

async fn run_application() -> Result<()> {
    // Create main window binding with integrated audio support
    let mut player = MainWindowBinding::new()?;

    // Initialize GPU visualization bridge
    info!("Initializing GPU visualization bridge...");
    if let Err(e) = player.initialize_gpu_visualization().await {
        warn!("Failed to initialize GPU visualization: {}", e);
        info!("Continuing without GPU acceleration");
    } else {
        info!("GPU visualization bridge initialized successfully");
        
        // Load default visualization shaders
        if let Err(e) = load_default_visualization_shaders(&mut player) {
            warn!("Failed to load default shaders: {}", e);
        }
    }

    // Check for demo audio files and load one if available
    if let Some(demo_path) = find_demo_audio_file() {
        info!("Found demo audio file: {}", demo_path.display());
        info!("Demo track found - you can load it through the UI file dialog");
    } else {
        info!("No demo audio files found - user will need to load tracks manually");
        print_usage_instructions();
    }

    // Run the application
    player.run()?;

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
    println!("\n=== Sonic Flow Music Player ===");
    println!("🎵 Welcome to Sonic Flow - High-Quality Audio Player with Visualizer");
    println!();
    println!("📁 To get started:");
    println!("   1. Click the 'Load Track' button in the UI");
    println!("   2. Select an audio file (MP3, FLAC, WAV, OGG supported)");
    println!("   3. Press the Play button to start playback");
    println!();
    println!("🎨 Features:");
    println!("   • High-quality audio playback");
    println!("   • Real-time spectrum visualizer");
    println!("   • Volume control and 10-second skip buttons");
    println!("   • Track information display");
    println!("   • Modern, responsive UI design");
    println!();
    println!("🔧 UI Controls:");
    println!("   • Load Track: Select audio files");
    println!("   • Play/Pause: Control playback");
    println!("   • Stop: Stop current playback");
    println!("   • Volume Slider: Adjust audio level");
    println!("   • Skip buttons: Jump 10 seconds forward/backward");
    println!();
    println!("💡 Supported Formats:");
    println!("   • MP3 (MPEG Audio Layer 3)");
    println!("   • FLAC (Free Lossless Audio Codec)");
    println!("   • WAV (Waveform Audio)");
    println!("   • OGG (Ogg Vorbis)");
    println!("   • M4A/AAC (Advanced Audio Coding)");
    println!();
    println!("For demo purposes, you can place audio files in:");
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
        if let Some(config_dir) = dirs::config_dir() {
            let log_dir = config_dir.join("sonic-flow").join("logs");

            if std::fs::create_dir_all(&log_dir).is_ok() {
                subscriber // For now, just use console logging
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
        sonic_core::Error::Application("Logging initialization failed".to_string())
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
        .map_err(|e| sonic_core::Error::Application(format!("Failed to create demo WAV: {}", e)))?;

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
            sonic_core::Error::Application(format!("Failed to write sample: {}", e))
        })?;
        writer.write_sample(amplitude).map_err(|e| {
            sonic_core::Error::Application(format!("Failed to write sample: {}", e))
        })?;
    }

    writer
        .finalize()
        .map_err(|e| sonic_core::Error::Application(format!("Failed to finalize WAV: {}", e)))?;

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
