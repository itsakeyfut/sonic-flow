//! Event system for application communication

use crate::{PlaylistId, Result, TrackId};
use tokio::sync::broadcast;

/// Application event types
#[derive(Debug, Clone)]
pub enum AppEvent {
    /// Playback events
    PlaybackStarted(TrackId),
    PlaybackPaused,
    PlaybackStopped,
    PlaybackPositionChanged(f64),
    VolumeChanged(f32),

    /// Playlist events
    PlaylistChanged(PlaylistId),
    TrackAdded(PlaylistId, TrackId),
    TrackRemoved(PlaylistId, TrackId),

    /// UI events
    ThemeChanged(String),
    VisualizerChanged(String),
    WindowResized(u32, u32),

    /// System events
    ApplicationExit,
    ConfigReloaded,
}

/// Event bus for application-wide communication
pub struct EventBus {
    sender: broadcast::Sender<AppEvent>,
    _receiver: broadcast::Receiver<AppEvent>,
}

impl EventBus {
    /// Create a new event bus
    pub fn new() -> Self {
        let (sender, receiver) = broadcast::channel(1000);
        Self {
            sender,
            _receiver: receiver,
        }
    }

    /// Publish an event
    pub fn publish(&self, event: AppEvent) -> Result<()> {
        self.sender
            .send(event)
            .map_err(|e| crate::Error::Application(format!("Failed to publish event: {}", e)))?;
        Ok(())
    }

    /// Subscribe to events
    pub fn subscribe(&self) -> broadcast::Receiver<AppEvent> {
        self.sender.subscribe()
    }
}

impl Default for EventBus {
    fn default() -> Self {
        Self::new()
    }
}
