use std::path::PathBuf;
use std::time::Duration;

use sonic_core::TrackMetadata;

/// Events sent from the application controller to the UI.
#[derive(Debug, Clone)]
pub enum Event {
    /// Periodic playback status update
    PlaybackStatus {
        is_playing: bool,
        is_paused: bool,
        volume: f32,
        position: Duration,
        duration: Option<Duration>,
        track_path: Option<PathBuf>,
        format: Option<FormatInfo>,
    },
    /// Track loaded successfully
    TrackLoaded {
        path: PathBuf,
        metadata: Box<TrackMetadata>,
    },
    /// Track failed to load
    TrackLoadFailed { path: PathBuf, error: String },
    /// General error
    Error(String),
}

/// Audio format information for display
#[derive(Debug, Clone)]
pub struct FormatInfo {
    pub sample_rate: u32,
    pub channels: u16,
    pub bit_depth: u16,
}
