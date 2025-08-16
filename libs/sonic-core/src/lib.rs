//! # Sonic Core
//!
//! Core audio processing library for Sonic Flow music player.
//!
//! This library provides the foundational components for audio playback,
//! analysis, effects processing, and metadata handling.
//!
//! ## Features
//!
//! - **Multi-format audio support**: MP3, FLAC, WAV, OGG, AAC
//! - **Real-time audio processing**: Low-latency playback engine
//! - **Audio analysis**: FFT, spectrum analysis, and visualization data
//! - **Effects processing**: EQ, reverb, crossfade, and more
//! - **Metadata handling**: ID3 tags, album art, and track information
//! - **Plugin system**: Extensible architecture for custom effects
//! - **Database integration**: Track library and playlist management
//!
//! ## Quick Start
//!
//! ```no_run
//! use sonic_core::{AudioEngine, Result, PlaybackControl, TrackLoader};
//! use std::path::Path;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let mut engine = AudioEngine::new()?;
//!     engine.load_track(Path::new("song.mp3")).await?;
//!     engine.play().await?;
//!     Ok(())
//! }
//! ```

#![deny(unsafe_op_in_unsafe_fn)]
#![warn(
    missing_docs,
    rust_2018_idioms,
    unreachable_pub,
    clippy::all,
    clippy::pedantic,
    clippy::nursery
)]
#![allow(
    clippy::module_name_repetitions,
    clippy::similar_names,
    clippy::too_many_lines,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap
)]

// Core modules
pub mod audio;
pub mod config;
pub mod error;
pub mod plugin;
pub mod utils;

// Re-exports for convenience
pub use audio::{
    AudioEngine, AudioFormat, AudioFormatType, PlaybackControl, PlaybackState, PlaybackStatus,
    TrackLoader, VolumeControl,
};
pub use config::ConfigManager;
pub use error::{Error, Result};

// Type aliases
pub use chrono::{DateTime, Utc};
pub use uuid::Uuid;

/// Track identifier
pub type TrackId = Uuid;
/// Playlist identifier  
pub type PlaylistId = Uuid;
/// Plugin identifier
pub type PluginId = String;
