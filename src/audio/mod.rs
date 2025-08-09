//! Audio engine and processing

pub mod traits;
pub mod engine;
pub mod decoder;
pub mod renderer;
pub mod effects;
pub mod analysis;

// Re-export main types for convenience
pub use engine::{AudioEngine, TrackInfo, AudioEngineStatus};
pub use decoder::{UniversalDecoder, create_decoder, supported_extensions, is_supported_extension};
pub use traits::{
    PlaybackControl, VolumeControl, TrackLoader, PlaybackStatus, AudioDecoder,
    PlaybackState, AudioFormat, AudioFormatType
};

// Re-export error types
pub use crate::error::{AudioError, DecoderError};

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
    ///
    /// # Errors
    ///
    /// Returns `AudioError` if the audio engine cannot be initialized.
    pub fn build(self) -> Result<AudioEngine, AudioError> {
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
