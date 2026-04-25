//! # Sonic Core
//!
//! Core audio processing library for the Sonic Flow music player.
//!
//! Provides audio playback, metadata extraction, and FFT spectrum analysis.

pub mod audio;
pub mod error;

pub use audio::{AudioFormat, AudioFormatType, PlayerManager, PlayerStatus};
pub use error::{AudioError, Error, Result};
