//! Application state management

use crate::{PlaylistId, TrackId};
use serde::{Deserialize, Serialize};

/// Global application state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    /// Current playback state
    pub playback: PlaybackState,
    /// Currently active playlist
    pub current_playlist: Option<PlaylistId>,
    /// UI state
    pub ui: UiState,
}

/// Playback state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaybackState {
    /// Currently playing track
    pub current_track: Option<TrackId>,
    /// Playback position in seconds
    pub position: f64,
    /// Track duration in seconds
    pub duration: f64,
    /// Current volume (0.0 - 1.0)
    pub volume: f32,
    /// Is currently playing
    pub is_playing: bool,
    /// Is muted
    pub is_muted: bool,
}

/// UI state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiState {
    /// Main window dimensions
    pub window_size: (u32, u32),
    /// Current theme
    pub theme: String,
    /// Active visualizer
    pub active_visualizer: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            playback: PlaybackState::default(),
            current_playlist: None,
            ui: UiState::default(),
        }
    }
}

impl Default for PlaybackState {
    fn default() -> Self {
        Self {
            current_track: None,
            position: 0.0,
            duration: 0.0,
            volume: 0.8,
            is_playing: false,
            is_muted: false,
        }
    }
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            window_size: (1200, 800),
            theme: "dark".to_string(),
            active_visualizer: "spectrum_bars".to_string(),
        }
    }
}
