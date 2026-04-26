//! # Sonic Core
//!
//! Core audio processing library for the Sonic Flow music player.
//!
//! Provides audio playback, metadata extraction, and FFT spectrum analysis.

pub mod audio;
pub mod error;

pub use audio::{
    AUDIO_EXTENSIONS, AudioBuffer, AudioDecoder, AudioFormat, AudioFormatInfo, AudioFormatType,
    DEFAULT_BAND_COUNT, MetadataExtractor, PlayerManager, PlayerStatus, Playlist, RepeatMode,
    SpectrumAnalyzer, SpectrumData, SymphoniaDecoder, TrackInfo, TrackMetadata, scan_folder,
};
pub use error::{AudioError, Error, Result};
