//! Audio effects and processing

pub mod equalizer;
pub mod reverb;
pub mod crossfade;

use crate::error::AudioError;
use crate::Result;

/// Audio effect trait
pub trait AudioEffect: Send + Sync {
    /// Apply the effect to an audio buffer
    ///
    /// # Arguments
    ///
    /// * `buffer` - Audio samples to process (in-place)
    /// * `sample_rate` - Sample rate of the audio
    /// * `channels` - Number of audio channels
    fn process(&mut self, buffer: &mut [f32], sample_rate: u32, channels: u16) -> Result<(), AudioError>;

    /// Reset the effect state
    fn reset(&mut self);

    /// Check if the effect is enabled
    fn is_enabled(&self) -> bool;

    /// Enable or disable the effect
    fn set_enabled(&mut self, enabled: bool);
}

/// Effects chain processor
pub struct EffectsChain {
    effects: Vec<Box<dyn AudioEffect>>,
    enabled: bool,
}
