//! Application event system for inter-component communication

use crate::{PlaylistId, TrackId};
use std::time::Duration;
use tokio::sync::broadcast;

/// Application events
#[derive(Debug, Clone)]
pub enum AppEvent {
    // Playback control events
    /// Toggle between play and pause
    TogglePlayback,

    /// Stop playback
    PlaybackStopped,

    /// Playback was started
    PlaybackStarted(TrackId),

    /// Playback was paused
    PlaybackPaused,

    /// Move to next track
    NextTrack,

    /// Move to previous track
    PreviousTrack,

    /// Seek to position (0.0 - 1.0 range)
    PlaybackPositionChanged(f64),

    /// Volume changed (0.0 - 1.0)
    VolumeChanged(f32),

    /// Mute state changed
    MuteToggled,

    // Track events
    /// Track was changed
    TrackChanged(TrackInfo),

    /// Track was loaded
    TrackLoaded { track_id: TrackId },

    /// Track loading failed
    TrackLoadFailed { path: String, error: String },

    // Visualizer events
    /// Visualizer type changed
    VisualizerChanged(String),

    /// Spectrum data updated (for visualizers)
    SpectrumDataUpdated { data: Vec<f32> },

    /// Visualizer configuration changed
    VisualizerConfigChanged {
        sensitivity: f32,
        color_scheme: String,
    },

    // UI events
    /// Window was resized
    WindowResized { width: u32, height: u32 },

    /// View changed (Library, Playlist, etc.)
    ViewChanged { view: String },

    /// Theme changed
    ThemeChanged { theme: String },

    // Library events
    /// Library scan started
    LibraryScanStarted,

    /// Library scan completed
    LibraryScanCompleted { tracks_found: usize },

    /// Library scan failed
    LibraryScanFailed { error: String },

    /// Track added to library
    TrackAddedToLibrary { track_id: TrackId },

    // Playlist events
    /// Playlist was created
    PlaylistCreated {
        playlist_id: PlaylistId,
        name: String,
    },

    /// Playlist was updated
    PlaylistUpdated { playlist_id: PlaylistId },

    /// Track added to playlist
    TrackAddedToPlaylist {
        playlist_id: PlaylistId,
        track_id: TrackId,
    },

    /// Track removed from playlist
    TrackRemovedFromPlaylist {
        playlist_id: PlaylistId,
        track_id: TrackId,
    },

    // System events
    /// Application error occurred
    Error { error: String },

    /// Application exit requested
    ApplicationExit,

    /// Configuration reloaded
    ConfigurationReloaded,
}

/// Track information for events
#[derive(Debug, Clone)]
pub struct TrackInfo {
    pub id: TrackId,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub duration: Duration,
    pub file_path: std::path::PathBuf,
}

/// Event bus for application-wide communication
#[derive(Clone)]
pub struct EventBus {
    sender: broadcast::Sender<AppEvent>,
}

impl EventBus {
    /// Create a new event bus
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1000);
        Self { sender }
    }

    /// Publish an event
    pub fn publish(&self, event: AppEvent) -> Result<usize, broadcast::error::SendError<AppEvent>> {
        self.sender.send(event)
    }

    /// Subscribe to events
    pub fn subscribe(&self) -> broadcast::Receiver<AppEvent> {
        self.sender.subscribe()
    }

    /// Get the number of active receivers
    pub fn receiver_count(&self) -> usize {
        self.sender.receiver_count()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_bus_publish_subscribe() {
        let event_bus = EventBus::new();
        let mut receiver = event_bus.subscribe();

        // Publish an event
        let result = event_bus.publish(AppEvent::PlaybackStopped);
        assert!(result.is_ok());

        // Receive the event
        let received_event = receiver.recv().await;
        assert!(received_event.is_ok());

        match received_event.unwrap() {
            AppEvent::PlaybackStopped => {} // Expected
            _ => panic!("Unexpected event received"),
        }
    }

    #[test]
    fn test_track_info_creation() {
        let track_info = TrackInfo {
            id: uuid::Uuid::new_v4(),
            title: Some("Test Track".to_string()),
            artist: Some("Test Artist".to_string()),
            album: Some("Test Album".to_string()),
            duration: Duration::from_secs(180),
            file_path: std::path::PathBuf::from("/path/to/track.mp3"),
        };

        assert_eq!(track_info.title, Some("Test Track".to_string()));
        assert_eq!(track_info.duration, Duration::from_secs(180));
    }
}
