//! Audio playback, decoding, metadata, and spectrum analysis.

pub mod analysis;
pub mod decoder;
pub mod metadata;
pub mod player;
pub mod player_manager;
pub mod spectrum_tap;
pub mod traits;

pub use analysis::{SpectrumAnalyzer, SpectrumData};
pub use decoder::{AudioBuffer, AudioDecoder, AudioFormatInfo, SymphoniaDecoder};
pub use metadata::{MetadataExtractor, TrackMetadata};
pub use player_manager::{PlayerManager, PlayerStatus};
pub use spectrum_tap::DEFAULT_BAND_COUNT;
pub use traits::{AudioFormat, AudioFormatType};
