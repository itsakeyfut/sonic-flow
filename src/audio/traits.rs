
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
