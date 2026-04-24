//! Audio engine and processing
//!
//! This module provides the core audio functionality for the Sonic Flow,
//! including playback control, audio decoding, and format support.

pub mod analysis;
pub mod decoder;
pub mod effects;
pub mod engine;
pub mod metadata;
pub mod player_manager;
pub mod renderer;
pub mod simple_player;
pub mod traits;

// Re-export main types for convenience
pub use decoder::{create_decoder, is_supported_extension, supported_extensions, UniversalDecoder};
pub use engine::{AudioEngine, AudioEngineStatus, TrackInfo};
pub use simple_player::SimplePlayer;
pub use traits::{
    AudioDecoder, AudioFormat, AudioFormatType, PlaybackControl, PlaybackState, PlaybackStatus,
    TrackLoader, VolumeControl,
};



// Re-export error types
pub use crate::error::AudioError;

/// Audio engine configuration
#[derive(Debug, Clone)]
pub struct AudioConfig {
    /// Default volume (0.0 - 1.0)
    pub default_volume: f32,
    /// Buffer size for audio processing
    pub buffer_size: usize,
    /// Sample rate for audio output
    pub sample_rate: u32,
    /// Number of output channels
    pub channels: u16,
    /// Enable hardware acceleration if available
    pub hardware_acceleration: bool,
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            default_volume: 0.8,
            buffer_size: 1024,
            sample_rate: 44100,
            channels: 2,
            hardware_acceleration: true,
        }
    }
}

/// Audio engine builder for configuration
pub struct AudioEngineBuilder {
    config: AudioConfig,
}

impl AudioEngineBuilder {
    /// Create a new audio engine builder with default configuration
    pub fn new() -> Self {
        Self {
            config: AudioConfig::default(),
        }
    }

    /// Set the default volume
    pub fn with_volume(mut self, volume: f32) -> Self {
        self.config.default_volume = volume.clamp(0.0, 1.0);
        self
    }

    /// Set the buffer size
    pub fn with_buffer_size(mut self, buffer_size: usize) -> Self {
        self.config.buffer_size = buffer_size;
        self
    }

    /// Set the sample rate
    pub fn with_sample_rate(mut self, sample_rate: u32) -> Self {
        self.config.sample_rate = sample_rate;
        self
    }

    /// Set the number of channels
    pub fn with_channels(mut self, channels: u16) -> Self {
        self.config.channels = channels;
        self
    }

    /// Enable or disable hardware acceleration
    pub fn with_hardware_acceleration(mut self, enabled: bool) -> Self {
        self.config.hardware_acceleration = enabled;
        self
    }

    /// Build the audio engine with the configured options
    pub fn build(self) -> Result<AudioEngine, crate::error::AudioError> {
        // For now, we ignore the configuration and create a default engine
        // TODO: Use the configuration to customize the engine
        AudioEngine::new()
    }
}

impl Default for AudioEngineBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Utility functions for audio processing
pub mod utils {
    use super::AudioFormatType;
    use std::path::Path;

    /// Check if a file is an audio file based on its extension
    pub fn is_audio_file(path: &Path) -> bool {
        if let Some(extension) = path.extension().and_then(|s| s.to_str()) {
            AudioFormatType::from_extension(extension).is_supported()
        } else {
            false
        }
    }

    /// Get the audio format type from a file path
    pub fn get_format_from_path(path: &Path) -> Option<AudioFormatType> {
        path.extension()
            .and_then(|s| s.to_str())
            .map(AudioFormatType::from_extension)
            .filter(AudioFormatType::is_supported)
    }

    /// Convert duration to a human-readable string (MM:SS)
    pub fn format_duration(duration: std::time::Duration) -> String {
        let total_seconds = duration.as_secs();
        let minutes = total_seconds / 60;
        let seconds = total_seconds % 60;
        format!("{:02}:{:02}", minutes, seconds)
    }

    /// Convert duration to a human-readable string with hours (HH:MM:SS)
    pub fn format_duration_with_hours(duration: std::time::Duration) -> String {
        let total_seconds = duration.as_secs();
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;

        if hours > 0 {
            format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
        } else {
            format!("{:02}:{:02}", minutes, seconds)
        }
    }

    /// Convert file size to human-readable string
    pub fn format_file_size(size: u64) -> String {
        const UNITS: &[&str] = &["B", "KB", "MB", "GB"];
        let mut size = size as f64;
        let mut unit_index = 0;

        while size >= 1024.0 && unit_index < UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }

        if unit_index == 0 {
            format!("{} {}", size as u64, UNITS[unit_index])
        } else {
            format!("{:.1} {}", size, UNITS[unit_index])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::Duration;

    #[test]
    fn test_audio_config_default() {
        let config = AudioConfig::default();
        assert_eq!(config.default_volume, 0.8);
        assert_eq!(config.buffer_size, 1024);
        assert_eq!(config.sample_rate, 44100);
        assert_eq!(config.channels, 2);
        assert!(config.hardware_acceleration);
    }

    #[test]
    fn test_audio_engine_builder() {
        let builder = AudioEngineBuilder::new()
            .with_volume(0.5)
            .with_buffer_size(2048)
            .with_sample_rate(48000)
            .with_channels(1)
            .with_hardware_acceleration(false);

        assert_eq!(builder.config.default_volume, 0.5);
        assert_eq!(builder.config.buffer_size, 2048);
        assert_eq!(builder.config.sample_rate, 48000);
        assert_eq!(builder.config.channels, 1);
        assert!(!builder.config.hardware_acceleration);
    }

    #[test]
    fn test_volume_clamping() {
        let builder = AudioEngineBuilder::new()
            .with_volume(-0.5) // Should be clamped to 0.0
            .with_volume(1.5); // Should be clamped to 1.0

        assert_eq!(builder.config.default_volume, 1.0);

        let builder = AudioEngineBuilder::new().with_volume(0.3);
        assert_eq!(builder.config.default_volume, 0.3);
    }

    #[test]
    fn test_audio_file_detection() {
        assert!(utils::is_audio_file(&PathBuf::from("test.mp3")));
        assert!(utils::is_audio_file(&PathBuf::from("song.flac")));
        assert!(utils::is_audio_file(&PathBuf::from("audio.wav")));
        assert!(!utils::is_audio_file(&PathBuf::from("document.txt")));
        assert!(!utils::is_audio_file(&PathBuf::from("no_extension")));
    }

    #[test]
    fn test_format_from_path() {
        let path = PathBuf::from("test.mp3");
        let format = utils::get_format_from_path(&path);
        assert_eq!(format, Some(AudioFormatType::Mp3));

        let path = PathBuf::from("test.unknown");
        let format = utils::get_format_from_path(&path);
        assert_eq!(format, None);
    }

    #[test]
    fn test_duration_formatting() {
        let duration = Duration::from_secs(125); // 2:05
        assert_eq!(utils::format_duration(duration), "02:05");

        let duration = Duration::from_secs(3665); // 1:01:05
        assert_eq!(utils::format_duration_with_hours(duration), "01:01:05");

        let duration = Duration::from_secs(65); // 1:05 (no hours)
        assert_eq!(utils::format_duration_with_hours(duration), "01:05");
    }

    #[test]
    fn test_file_size_formatting() {
        assert_eq!(utils::format_file_size(512), "512 B");
        assert_eq!(utils::format_file_size(1536), "1.5 KB");
        assert_eq!(utils::format_file_size(1024 * 1024), "1.0 MB");
        assert_eq!(utils::format_file_size(1024 * 1024 * 1024), "1.0 GB");
    }
}
