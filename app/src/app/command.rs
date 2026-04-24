use std::path::PathBuf;

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
    /// Skip forward by seconds
    SkipForward(f64),
    /// Skip backward by seconds
    SkipBackward(f64),
}
