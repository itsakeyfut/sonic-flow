//! Utility functions and helpers

pub mod file;
pub mod math;
pub mod color;
pub mod animation;

/// File utility functions
pub mod file {
    /// Check if a path is an audio file based on extension
    pub fn is_audio_file(path: &std::path::Path) -> bool {
        if let Some(ext) = path.extension().and_then(|s| s.to_str()) {
            matches!(ext.to_lowercase().as_str(), "mp3" | "flac" | "wav" | "ogg" | "m4a" | "aac")
        } else {
            false
        }
    }
}
