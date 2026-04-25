//! Audio playback, decoding, metadata, and spectrum analysis.

pub mod analysis;
pub mod metadata;
pub mod player_manager;
pub mod player;
pub mod traits;

pub use player_manager::{PlayerManager, PlayerStatus};
pub use traits::{AudioFormat, AudioFormatType};
