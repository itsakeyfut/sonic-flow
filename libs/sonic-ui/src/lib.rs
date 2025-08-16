//! # Sonic UI
//!
//! User interface library for Sonic Flow music player.
//!
//! This library provides the Slint-based UI components and integration
//! with the audio engine for the Sonic Flow music player.
//!
//! ## Features
//!
//! - **Modern UI**: Beautiful Slint-based interface
//! - **Audio integration**: Real-time audio player controls
//! - **Visualizations**: Integrated audio spectrum visualizer
//! - **File management**: Drag-and-drop and file dialog support
//! - **Responsive design**: Adaptive layouts for different screen sizes
//! - **Theme support**: Customizable color schemes and styles
//! - **Accessibility**: Screen reader and keyboard navigation support
//!
//! ## Quick Start
//!
//! ```no_run
//! use sonic_ui::MainWindowBinding;
//! use sonic_core::Result;
//!
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let ui = MainWindowBinding::new()?;
//!     ui.run()?;
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

// UI modules
pub mod bindings;
pub mod audio_bridge;
pub mod main_binding;

// Re-exports for convenience
pub use main_binding::MainWindowBinding;
pub use bindings::MainWindowBinding as LegacyMainWindowBinding;
pub use audio_bridge::{AudioIntegration, AudioCommand, UiUpdateEvent};

// Re-exports from dependencies
pub use sonic_core::{Error, Result};
pub use sonic_visualizer;
