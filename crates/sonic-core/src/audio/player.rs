//! Rodio-based audio player.

use std::path::Path;
use std::time::Duration;
use tokio::time::sleep;

use rodio::{Decoder, OutputStream, Sink, Source};
use tracing::info;

use crate::audio::traits::AudioFormatType;
use crate::error::AudioError;

/// Audio player backed by rodio.
pub struct Player {
    /// Audio output stream
    _stream: OutputStream,
    /// Stream handle for creating sinks
    stream_handle: rodio::OutputStreamHandle,
    /// Current audio sink
    sink: Option<Sink>,
}

unsafe impl Send for Player {}
unsafe impl Sync for Player {}

impl Player {
    /// Create a new simple player
    pub fn new() -> Result<Self, AudioError> {
        let (stream, stream_handle) = OutputStream::try_default()
            .map_err(|e| AudioError::Device(format!("Failed to initialize audio output: {}", e)))?;

        Ok(Self {
            _stream: stream,
            stream_handle,
            sink: None,
        })
    }

    /// Load and play an audio file.
    pub async fn play_file(&mut self, path: &Path) -> Result<(), AudioError> {
        info!("Loading audio file: {}", path.display());

        if !path.exists() {
            return Err(AudioError::Streaming(format!(
                "File not found: {}",
                path.display()
            )));
        }

        let extension = path
            .extension()
            .and_then(|s| s.to_str())
            .ok_or_else(|| AudioError::UnsupportedFormat {
                format: "unknown".to_string(),
            })?;

        if !AudioFormatType::from_extension(extension).is_supported() {
            return Err(AudioError::UnsupportedFormat {
                format: extension.to_string(),
            });
        }

        let file = std::fs::File::open(path).map_err(|e| {
            AudioError::Streaming(format!("Failed to open file {}: {}", path.display(), e))
        })?;

        let source = Decoder::new(std::io::BufReader::new(file))
            .map_err(|e| AudioError::Streaming(format!("Failed to decode file: {}", e)))?;

        let sample_rate = source.sample_rate();
        let channels = source.channels();
        info!(
            "Audio file info - Sample rate: {} Hz, Channels: {}",
            sample_rate, channels
        );

        // Create sink
        let sink = Sink::try_new(&self.stream_handle).map_err(|e| {
            AudioError::Device(format!("Failed to create audio sink: {}", e))
        })?;

        // Set volume
        sink.set_volume(0.8);

        // Add source to sink and play
        sink.append(source);
        sink.play();

        // Store sink
        self.sink = Some(sink);

        info!("MP3 playback started");
        Ok(())
    }

    /// Stop playback
    pub fn stop(&mut self) {
        if let Some(sink) = &self.sink {
            sink.stop();
            info!("Playback stopped");
        }
        self.sink = None;
    }

    /// Pause playback
    pub fn pause(&mut self) {
        if let Some(sink) = &self.sink {
            sink.pause();
            info!("Playback paused");
        }
    }

    /// Resume playback
    pub fn resume(&mut self) {
        if let Some(sink) = &self.sink {
            sink.play();
            info!("Playback resumed");
        }
    }

    /// Check if currently playing
    pub fn is_playing(&self) -> bool {
        self.sink
            .as_ref()
            .map(|sink| !sink.is_paused() && !sink.empty())
            .unwrap_or(false)
    }

    /// Set volume (0.0 - 1.0)
    pub fn set_volume(&mut self, volume: f32) {
        if let Some(sink) = &self.sink {
            sink.set_volume(volume.clamp(0.0, 1.0));
        }
    }

    /// Wait for playback to finish
    pub async fn wait_for_finish(&self) {
        if let Some(sink) = &self.sink {
            while !sink.empty() {
                sleep(Duration::from_millis(100)).await;
            }
        }
    }
}

impl Drop for Player {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_player_creation() {
        // Test creating a simple player
        let result = Player::new();
        match result {
            Ok(_player) => {
                // Player creation successful
                assert!(true);
            }
            Err(e) => {
                // Audio system might not be available in test environment
                eprintln!("Audio system not available in test: {}", e);
            }
        }
    }

    #[tokio::test]
    async fn test_mp3_file_validation() {
        if let Ok(mut player) = Player::new() {
            // Test with non-existent file
            let result = player.play_file(&PathBuf::from("nonexistent.mp3")).await;
            assert!(result.is_err());

            // Test with wrong extension
            let result = player.play_file(&PathBuf::from("test.txt")).await;
            assert!(result.is_err());
        }
    }
}
