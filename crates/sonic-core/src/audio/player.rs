//! Rodio-backed audio player.
//!
//! Wraps a `rodio::Sink` and drives it with a [`SymphoniaDecoder`] source.
//! Seeking is implemented by re-opening the file, seeking the decoder, and
//! replacing the sink's source.

use std::path::{Path, PathBuf};
use std::time::Duration;

use rodio::{OutputStream, Sink};
use tracing::info;

use crate::audio::decoder::{AudioDecoder, AudioFormatInfo, SymphoniaDecoder};
use crate::error::AudioError;

/// Audio player backed by a `rodio::Sink`.
///
/// Not `Clone` — intended to be owned by the audio thread inside
/// [`super::player_manager::PlayerManager`].
pub struct Player {
    /// Kept alive to maintain the audio output stream.
    _stream: OutputStream,
    stream_handle: rodio::OutputStreamHandle,
    sink: Option<Sink>,
    /// Path of the currently loaded file, required for seek re-opening.
    current_path: Option<PathBuf>,
    /// Volume applied to every new sink (0.0 – 1.0).
    volume: f32,
}

// OutputStreamHandle is Send but OutputStream is not — we own both exclusively.
unsafe impl Send for Player {}
unsafe impl Sync for Player {}

impl Player {
    /// Create a new player connected to the default audio output device.
    pub fn new() -> Result<Self, AudioError> {
        let (stream, stream_handle) = OutputStream::try_default()
            .map_err(|e| AudioError::Device(format!("Failed to initialise audio output: {e}")))?;

        Ok(Self {
            _stream: stream,
            stream_handle,
            sink: None,
            current_path: None,
            volume: 0.8,
        })
    }

    /// Load and immediately begin playing an audio file.
    ///
    /// Stops any current playback before opening the new file.
    /// Returns [`AudioFormatInfo`] with the stream's sample rate, channel
    /// count, bit depth, and duration.
    pub fn play_file(&mut self, path: &Path) -> Result<AudioFormatInfo, AudioError> {
        info!("Loading audio file: {}", path.display());

        // Stop existing sink before creating a new one.
        if let Some(old) = self.sink.take() {
            old.stop();
        }

        let decoder = SymphoniaDecoder::open(path)?;
        let info = decoder.format_info();

        info!(
            "Opened: {} Hz, {} ch, {:?} bit, codec={}, duration={:?}",
            info.sample_rate, info.channels, info.bit_depth, info.codec_name, info.duration,
        );

        let sink = Sink::try_new(&self.stream_handle)
            .map_err(|e| AudioError::Device(format!("Failed to create audio sink: {e}")))?;
        sink.set_volume(self.volume);
        sink.append(decoder);
        sink.play();

        self.sink = Some(sink);
        self.current_path = Some(path.to_owned());

        Ok(info)
    }

    /// Seek to `position` within the currently loaded file.
    ///
    /// Implemented by re-opening the decoder, seeking it, then swapping the
    /// sink's source. Preserves the current pause/play state.
    ///
    /// Returns the actual position seeked to (may differ from requested due to
    /// keyframe alignment).
    pub fn seek(&mut self, position: Duration) -> Result<Duration, AudioError> {
        let path = self
            .current_path
            .clone()
            .ok_or_else(|| AudioError::InvalidState {
                from: "no_file_loaded".into(),
                to: "seek".into(),
            })?;

        let was_paused = self.sink.as_ref().map(|s| s.is_paused()).unwrap_or(false);

        // Open a fresh decoder and seek it to the requested position.
        let mut decoder = SymphoniaDecoder::open(&path)?;
        let actual = decoder.seek(position)?;

        // Replace the sink with a new one using the seeked decoder.
        if let Some(old) = self.sink.take() {
            old.stop();
        }

        let sink = Sink::try_new(&self.stream_handle).map_err(|e| {
            AudioError::Device(format!("Failed to create audio sink after seek: {e}"))
        })?;
        sink.set_volume(self.volume);
        sink.append(decoder);
        // Restore pause state — Sink starts playing by default after append.
        if was_paused {
            sink.pause();
        }

        self.sink = Some(sink);

        info!("Seeked to {:?} (requested {:?})", actual, position);
        Ok(actual)
    }

    /// Stop playback and release the current source.
    pub fn stop(&mut self) {
        if let Some(sink) = self.sink.take() {
            sink.stop();
            info!("Playback stopped");
        }
        self.current_path = None;
    }

    /// Pause playback. No-op when already paused or no track is loaded.
    pub fn pause(&mut self) {
        if let Some(sink) = &self.sink {
            sink.pause();
            info!("Playback paused");
        }
    }

    /// Resume playback. No-op when already playing or no track is loaded.
    pub fn resume(&mut self) {
        if let Some(sink) = &self.sink {
            sink.play();
            info!("Playback resumed");
        }
    }

    /// Returns `true` when audio is actively playing (not paused, not empty).
    pub fn is_playing(&self) -> bool {
        self.sink
            .as_ref()
            .map(|s| !s.is_paused() && !s.empty())
            .unwrap_or(false)
    }

    /// Returns `true` when the sink is paused (but may still have a source).
    pub fn is_paused(&self) -> bool {
        self.sink.as_ref().map(|s| s.is_paused()).unwrap_or(false)
    }

    /// Set output volume. Clamped to [0.0, 1.0]. Persists across seeks.
    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
        if let Some(sink) = &self.sink {
            sink.set_volume(self.volume);
        }
    }

    /// Returns the path of the currently loaded file, if any.
    pub fn current_path(&self) -> Option<&Path> {
        self.current_path.as_deref()
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

    #[test]
    fn player_creation() {
        // Audio device may not be available in CI — treat as a soft failure.
        match Player::new() {
            Ok(_) => {}
            Err(e) => eprintln!("Audio device unavailable in test environment: {e}"),
        }
    }

    #[test]
    fn play_nonexistent_file_returns_error() {
        if let Ok(mut player) = Player::new() {
            let result = player.play_file(&PathBuf::from("nonexistent.mp3"));
            assert!(result.is_err());
        }
    }

    #[test]
    fn seek_without_loaded_file_returns_error() {
        if let Ok(mut player) = Player::new() {
            let result = player.seek(Duration::from_secs(10));
            assert!(matches!(result, Err(AudioError::InvalidState { .. })));
        }
    }
}
