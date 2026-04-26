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
    /// Real-time spectrum analysis update (~60 fps)
    SpectrumUpdated { bands: Vec<f32>, peak: f32 },
    /// Playlist contents changed (tracks added/removed/reordered) or current
    /// index changed (next/prev/select).
    PlaylistUpdated {
        tracks: Vec<TrackSummary>,
        current_index: Option<usize>,
        total_duration: Duration,
    },
    /// General error
    Error(String),
}

/// Lightweight track description for playlist display.
#[derive(Debug, Clone)]
pub struct TrackSummary {
    pub title: String,
    pub artist: String,
    pub duration: Option<Duration>,
}

/// Audio format information for display
#[derive(Debug, Clone)]
pub struct FormatInfo {
    pub sample_rate: u32,
    pub channels: u16,
    pub bit_depth: u16,
}
