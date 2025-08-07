
//! # Sonic Flow
//! 
//! A high-quality music player with advanced audio spectrum visualizers.
//! 
//! ## Architecture Overview
//! 
//! This application follows a layered architecture pattern:
//! 
//! - **UI Layer**: Slint-based user interface
//! - **Application Layer**: State management and control logic
//! - **Business Logic Layer**: Core domain logic (audio, visualizer, playlist)
//! - **Infrastructure Layer**: External system integrations
//! 
//! ## Example Usage
//! 
//! ```no_run
//! use sonic_flow::{SonicFlow, Result};
//! 
//! #[tokio::main]
//! async fn main() -> Result<()> {
//!     let player = SonicFlow::new().await?;
//!     player.run().await
//! }
//! ```

#![deny(unsafe_op_in_unsafe_fn)]
#![warn(
    missing_docs,
    rust_2018_idioms,
    unreachable_pub,
    clippy::all,
    clippy::pedantic,
    clippy::nursery,
)]
#![allow(
    clippy::module_name_repetitions,
    clippy::similar_names,
    clippy::too_many_lines,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_possible_wrap,
)]
