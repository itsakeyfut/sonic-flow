
//! Audio engine traits and interfaces
//!
//! This module defines the core traits for audio playback, decoding,
//! and rendering that form the foundation of the audio engine.

use std::time::Duration;
use std::path::Path;
use async_trait::async_trait;
use crate::{Result, TrackId};
use crate::error::AudioError;

/// Core playback control interface
///
/// This trait defines the essential playback operations that any audio engine
/// must implement. All methods are async to support non-blocking operations.
#[async_trait]
pub trait PlaybackControl: Send + Sync {
    /// Start or resume playback
    ///
    /// # Errors
    /// 
    /// Returns `AudioError` if playback cannot be started due to device issues,
    /// missing track, or invalid state.
    async fn play(&mut self) -> Result<(), AudioError>;

    /// Pause playback while maintaining current position
    /// 
    /// # Errors
    /// 
    /// Returns `AudioError` if the pause operation fials.
    async fn pause(&mut self) -> Result<(), AudioError>;

    /// Stop playback and reset position to beginning
    /// 
    /// # Errors
    /// 
    /// Returns `AudioError` if the stop operation fails.
    async fn stop(&mut self) -> Result<(), AudioError>;

    /// Seek to a specific position in the current track
    /// 
    /// # Arguments
    /// 
    /// * `position` - Target position as duration from the beginning
    /// 
    /// # Errors
    /// 
    /// Returns `AudioError` if seeking is not supported or the position is invalid.
    async fn seek(&mut self, position: Duration) -> Result<(), AudioError>;

    /// Move to the next track in the queue
    /// 
    /// # Errors
    /// 
    /// Returns `AudioError` if there is no next track or the operation fails.
    async fn next_track(&mut self) -> Result<(), AudioError>;

    /// Move to the previous track in the queue
    /// 
    /// # Errors
    /// 
    /// Returns `AudioError` if there is no previous track or the operation fails.
    async fn previous_track(&mut self) -> Result<(), AudioError>;
}

/// Volume control interface
pub trait VolumeControl: Send + Sync {
    /// Set the playback volume
    /// 
    /// # Arguments
    /// 
    /// * `volume` - Volume level between 0.0 (silent) and 1.0 (maximum)
    fn set_volume(&mut self, volume: f32);

    /// Get the current volume level
    /// 
    /// # Returns
    /// 
    /// Current volume as a value between 0.0 and 1.0
    fn volume(&self) -> f32;

    /// Mute or unmute audio output
    /// 
    /// # Arguments
    /// 
    /// * `muted` - True to mute, false to unmute
    fn set_muted(&mut self, muted: bool);

    /// Check if audio is currently muted
    /// 
    /// # Returns
    /// 
    /// True if audio is muted, false otherwise
    fn is_muted(&self) -> bool;
}

/// Track loading and management interface
#[async_trait]
pub trait TrackLoader: Send + Sync {
    /// Load a track from a file path
    /// 
    /// # Arguments
    /// 
    /// * `path` - Path to the audio file
    /// 
    /// # Returns
    /// 
    /// Track ID that can be used for playback operations
    /// 
    /// # Errors
    /// 
    /// Returns `AudioError` if the file cannot be loaded or is in an unsupported format.
    async fn load_track(&mut self, path: &Path) -> Result<TrackId, AudioError>;

    /// Set the current track for playback
    /// 
    /// # Arguments
    /// 
    /// * `track_id` - ID of the track to set as current
    /// 
    /// # Errors
    /// 
    /// Returns `AudioError` if the track ID is invalid
    async fn set_current_track(&mut self, track_id: TrackId) -> Result<(), AudioError>;

    /// Get the currently loaded track ID
    /// 
    /// # Returns
    /// 
    /// Current track ID, or None if no track is loaded
    fn current_track(&self) -> Option<TrackId>;
}

/// Playback state information
#[derive(Debug, Clone, PartialEq)]
pub enum PlaybackState {
    /// No track loaded or playback stopped
    Stopped,
    /// Playback is active
    Playing,
    /// Playback is paused
    Paused,
    /// Buffering or loading
    Buffering,
    /// Error state
    Error(String),
}

/// Playback status interface
pub trait PlaybackStatus: Send + Sync {
    /// Get current playback state
    fn state(&self) -> PlaybackState;

    /// Get current playback position
    /// 
    /// # Returns
    /// 
    /// Current position as duration from the beginning
    fn position(&self) -> Duration;

    /// Get total duration of the current track
    /// 
    /// # Returns
    /// 
    /// Total track duration, or None if not available
    fn duration(&self) -> Option<Duration>;

    /// Check if playback is currently active
    fn is_playing(&self) -> bool {
        matches!(self.state(), PlaybackState::Playing)
    }

    /// Check if playback is paused
    fn is_paused(&self) -> bool {
        matches!(self.state(), PlaybackState::Paused)
    }

    /// Check if playback is stopped
    fn is_stopped(&self) -> bool {
        matches!(self.state(), PlaybackState::Stopped)
    }
}

/// Audio decoder interface
pub trait AudioDecoder: Send + Sync {
    /// Decode audio data from the input buffer
    /// 
    /// # Arguments
    /// 
    /// * `input` - Encoded audio data
    /// * `output` - Buffer to write decoded samples
    /// 
    /// # Returns
    /// 
    /// Number of samples written to the output buffer
    /// 
    /// # Errors
    /// 
    /// Returns `AudioError` if decoding fails
    fn decode(&mut self, input: &[u8], output: &mut [f32]) -> Result<usize, AudioError>;

    /// Get the sample rate of the decoded audio
    fn sample_rate(&self) -> u32;

    /// Get the number of audio channels
    fn channels(&self) -> u16;

    /// Seek to a specific position in the audio stream
    /// 
    /// # Arguments
    /// 
    /// * `position` - Target position as duration from the beginning
    /// 
    /// # Errors
    /// 
    /// Returns `AudioError` if seeking fails or is not supported
    fn seek(&mut self, position: Duration) -> Result<(), AudioError>;

    /// Check if the decoder supports seeking
    fn supports_seek(&self) -> bool;
}

/// Audio format information
#[derive(Debug, Clone, PartialEq)]
pub struct AudioFormat {
    /// Sample rate in Hz
    pub sample_rate: u32,
    /// Number of channels (1 for mono, 2 for stereo)
    pub channels: u16,
    /// Bit depth (16, 24, etc.)
    pub bit_depth: u16,
    /// Audio format type
    pub format_type: AudioFormatType,
}

/// Supported audio format types
#[derive(Debug, Clone, PartialEq)]
pub enum AudioFormatType {
    /// MP3 format
    Mp3,
    /// FLAC lossless format
    Flac,
    /// WAV uncompressed format
    Wav,
    /// OGG Vorbis format
    Ogg,
    /// AAC format
    Aac,
    /// Unknown or unsupported format
    Unknown(String),
}

impl AudioFormatType {
    /// Create format type from file extension
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "mp3" => Self::Mp3,
            "flac" => Self::Flac,
            "wav" => Self::Wav,
            "ogg" => Self::Ogg,
            "aac" | "m4a" => Self::Aac,
            other => Self::Unknown(other.to_string()),
        }
    }
}