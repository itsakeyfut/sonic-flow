use std::path::PathBuf;
use std::time::Duration;

/// Commands sent from UI to the application controller.
#[derive(Debug)]
pub enum Command {
    /// Load and play an audio file (single-file, bypasses playlist)
    #[allow(dead_code)]
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

    // -- Playlist management ----------------------------------------------
    /// Add one or more audio files to the playlist
    AddTracks(Vec<PathBuf>),
    /// Recursively scan a folder and add all audio files found
    AddFolder(PathBuf),
    /// Advance to the next track in the playlist
    NextTrack,
    /// Return to the previous track in the playlist
    PreviousTrack,
    /// Jump to a specific track by playlist index
    SelectTrack(usize),
    /// Remove the track at the given playlist index
    RemoveTrack(usize),
    /// Clear the entire playlist
    ClearPlaylist,
    /// Toggle shuffle mode
    ToggleShuffle,
    /// Cycle through repeat modes (None → One → All → None)
    CycleRepeat,
}
