//! Audio engine traits and interfaces
//!
//! This module defines the core traits for audio playback, decoding,
//! and rendering that form the foundation of the audio engine.

use crate::error::AudioError;
use crate::TrackId;
use async_trait::async_trait;
use std::path::Path;
use std::time::Duration;

/// Core playback control interface
///
/// This trait defines the essential playback operations that any audio engine
/// must implement. All methods are async to support non-blocking operations.
#[async_trait]
pub trait PlaybackControl: Send + Sync {
    /// Start or resume playback
    async fn play(&mut self) -> Result<(), AudioError>;

    /// Pause playback while maintaining current position
    async fn pause(&mut self) -> Result<(), AudioError>;

    /// Stop playback and reset position to beginning
    async fn stop(&mut self) -> Result<(), AudioError>;

    /// Seek to a specific position in the current track
    async fn seek(&mut self, position: Duration) -> Result<(), AudioError>;

    /// Move to the next track in the queue
    async fn next_track(&mut self) -> Result<(), AudioError>;

    /// Move to the previous track in the queue  
    async fn previous_track(&mut self) -> Result<(), AudioError>;
}

/// Volume control interface
pub trait VolumeControl: Send + Sync {
    /// Set the playback volume
    fn set_volume(&mut self, volume: f32);

    /// Get the current volume level
    fn volume(&self) -> f32;

    /// Mute or unmute audio output
    fn set_muted(&mut self, muted: bool);

    /// Check if audio is currently muted
    fn is_muted(&self) -> bool;
}

/// Track loading and management interface
#[async_trait]
pub trait TrackLoader: Send + Sync {
    /// Load a track from a file path
    async fn load_track(&mut self, path: &Path) -> Result<TrackId, AudioError>;

    /// Set the current track for playback
    async fn set_current_track(&mut self, track_id: TrackId) -> Result<(), AudioError>;

    /// Get the currently loaded track ID
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
    fn position(&self) -> Duration;

    /// Get total duration of the current track
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
    fn decode(&mut self, input: &[u8], output: &mut [f32]) -> Result<usize, AudioError>;

    /// Get the sample rate of the decoded audio
    fn sample_rate(&self) -> u32;

    /// Get the number of audio channels
    fn channels(&self) -> u16;

    /// Seek to a specific position in the audio stream
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

    /// Check if the format is supported
    pub fn is_supported(&self) -> bool {
        !matches!(self, Self::Unknown(_))
    }

    /// Get the format name as a string
    pub fn as_str(&self) -> &str {
        match self {
            Self::Mp3 => "mp3",
            Self::Flac => "flac",
            Self::Wav => "wav",
            Self::Ogg => "ogg",
            Self::Aac => "aac",
            Self::Unknown(s) => s,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audio_format_type_from_extension() {
        assert_eq!(AudioFormatType::from_extension("mp3"), AudioFormatType::Mp3);
        assert_eq!(AudioFormatType::from_extension("MP3"), AudioFormatType::Mp3);
        assert_eq!(
            AudioFormatType::from_extension("flac"),
            AudioFormatType::Flac
        );
        assert_eq!(AudioFormatType::from_extension("wav"), AudioFormatType::Wav);
        assert_eq!(
            AudioFormatType::from_extension("unknown"),
            AudioFormatType::Unknown("unknown".to_string())
        );
    }

    #[test]
    fn test_audio_format_type_is_supported() {
        assert!(AudioFormatType::Mp3.is_supported());
        assert!(AudioFormatType::Flac.is_supported());
        assert!(!AudioFormatType::Unknown("xyz".to_string()).is_supported());
    }

    #[test]
    fn test_playback_state_helpers() {
        let state = PlaybackState::Playing;
        assert!(matches!(state, PlaybackState::Playing));

        let state = PlaybackState::Stopped;
        assert!(matches!(state, PlaybackState::Stopped));
    }
}
