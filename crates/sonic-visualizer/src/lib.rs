//! # Sonic Visualizer
//!
//! Audio visualization engine for Sonic Flow music player.
//!
//! This library provides real-time audio visualization capabilities including
//! spectrum analysis, waveform rendering, and various visualization plugins.
//!
//! ## Features
//!
//! - **Real-time visualization**: Low-latency audio-to-visual rendering
//! - **Multiple visualization types**: Spectrum bars, waveforms, circular displays
//! - **Plugin architecture**: Extensible visualization system
//! - **GPU acceleration**: Hardware-accelerated rendering where available
//! - **Customizable themes**: Color schemes and visual styles
//! - **Performance optimized**: 60+ FPS smooth animations
//!
//! ## Quick Start
//!
//! ```no_run
//! use sonic_visualizer::{VisualizerEngine, SpectrumVisualizer};
//! use sonic_core::AudioEngine;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let audio_engine = AudioEngine::new().await?;
//!     let mut visualizer = VisualizerEngine::new();
//!     
//!     // Add spectrum visualization
//!     visualizer.add_plugin(Box::new(SpectrumVisualizer::new()));
//!     
//!     // Connect to audio stream
//!     visualizer.connect_audio_stream(audio_engine.get_stream()).await?;
//!     
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

// Re-export from the old module structure
pub use canvas::*;
pub use engine::*;
pub use traits::*;
pub use plugins::*;

// Modules
pub mod canvas;
pub mod engine;
pub mod traits;
pub mod plugins;

// Re-exports from sonic-core
pub use sonic_core::{Error, Result};
