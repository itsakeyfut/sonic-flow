//! Audio playback, decoding, metadata, and spectrum analysis.

pub mod analysis;
pub mod decoder;
pub mod metadata;
pub mod player;
pub mod player_manager;
pub mod traits;

pub use decoder::{AudioBuffer, AudioDecoder, AudioFormatInfo, SymphoniaDecoder};
pub use player_manager::{PlayerManager, PlayerStatus};
pub use traits::{AudioFormat, AudioFormatType};
