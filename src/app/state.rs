//! Application state management

use std::sync::Arc;

use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

use crate::{PlaylistId, TrackId};

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

/// Thread-safe state manager
pub struct StateManager {
    state: Arc<RwLock<AppState>>,
}

impl StateManager {
    /// Create a new state manager
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(AppState::default())),
        }
    }

    /// Get a read lock on the state
    pub fn read(&self) -> parking_lot::RwLockReadGuard<'_, AppState> {
        self.state.read()
    }

    /// Get a write lock on the state
    pub fn write(&self) -> parking_lot::RwLockWriteGuard<'_, AppState> {
        self.state.write()
    }

    /// Get a clone of the current state
    pub fn get_state(&self) -> AppState {
        self.state.read().clone()
    }

    /// Update playback state with a closure
    pub fn update_player_state<F>(&self, f: F)
    where
        F: FnOnce(&mut PlaybackState),
    {
        let mut state = self.state.write();
        f(&mut state.playback);
    }

    /// Update UI state with a closure
    pub fn update_ui_state<F>(&self, f: F)
    where
        F: FnOnce(&mut UiState),
    {
        let mut state = self.state.write();
        f(&mut state.ui);
    }
}

impl Default for StateManager {
    fn default() -> Self {
        Self::new()
    }
}
