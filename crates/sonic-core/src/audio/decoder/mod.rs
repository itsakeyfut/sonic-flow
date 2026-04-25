//! Audio decoder trait and supporting types.
//!
//! Provides a unified interface for decoding audio from various container
//! formats (MP3, FLAC, WAV, OGG, AAC, OPUS) backed by Symphonia.

pub mod registry;
mod symphonia;

pub use symphonia::SymphoniaDecoder;

use std::time::Duration;

use crate::audio::traits::AudioFormatType;
use crate::error::AudioError;

/// A single packet of decoded, interleaved f32 audio samples.
#[derive(Debug, Clone)]
pub struct AudioBuffer {
    /// Interleaved samples in channel order (e.g. [L, R, L, R, ...] for stereo).
    pub samples: Vec<f32>,
    /// Sample rate in Hz.
    pub sample_rate: u32,
    /// Channel count.
    pub channels: u16,
}

/// Detailed information about an audio stream returned by a decoder.
#[derive(Debug, Clone)]
pub struct AudioFormatInfo {
    /// Sample rate in Hz (e.g. 44100, 48000, 192000).
    pub sample_rate: u32,
    /// Number of channels.
    pub channels: u16,
    /// Bit depth per sample, if reported by the container (e.g. 16, 24, 32).
    pub bit_depth: Option<u32>,
    /// Container/format type inferred from the file extension.
    pub format_type: AudioFormatType,
    /// Total duration, if known before full decode.
    pub duration: Option<Duration>,
    /// Human-readable codec name (e.g. "FLAC", "MP3").
    pub codec_name: String,
}

/// Unified interface for audio decoders.
///
/// Implementations are expected to be `Send` so they can be driven from
/// a dedicated audio or blocking thread.
pub trait AudioDecoder: Send {
    /// Decode the next packet of audio samples.
    ///
    /// Returns `Ok(None)` when the stream is fully consumed.
    fn decode_next(&mut self) -> Result<Option<AudioBuffer>, AudioError>;

    /// Seek to the requested playback position.
    ///
    /// Returns the actual position seeked to, which may differ from the
    /// requested position due to keyframe alignment.
    fn seek(&mut self, position: Duration) -> Result<Duration, AudioError>;

    /// Total duration of the stream, if known from container metadata.
    fn duration(&self) -> Option<Duration>;

    /// Format and codec information for the stream.
    fn format_info(&self) -> AudioFormatInfo;
}
