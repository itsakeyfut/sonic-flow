use std::path::PathBuf;
use std::time::Duration;

/// Commands sent from UI to the application controller.
#[derive(Debug)]
pub enum Command {
    /// Load and play an audio file
    LoadFile(PathBuf),
    /// Toggle play/pause
    TogglePlayback,
    /// Stop playback
    Stop,
    /// Set volume (0.0 - 1.0)
    SetVolume(f32),
    /// Seek to an absolute position within the current track
    #[allow(dead_code)]
    Seek(Duration),
    /// Skip forward by the given number of seconds (relative to current position)
    SkipForward(f64),
    /// Skip backward by the given number of seconds (relative to current position)
    SkipBackward(f64),
}
