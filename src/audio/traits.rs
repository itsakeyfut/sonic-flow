
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
