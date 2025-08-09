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
