//! # Sonic Core
//!
//! Core audio processing library for the Sonic Flow music player.
//!
//! Provides audio playback, metadata extraction, and FFT spectrum analysis.

pub mod audio;
pub mod error;

pub use audio::{
    AudioBuffer, AudioDecoder, AudioFormat, AudioFormatInfo, AudioFormatType, MetadataExtractor,
    PlayerManager, PlayerStatus, SymphoniaDecoder, TrackMetadata,
};
pub use error::{AudioError, Error, Result};
