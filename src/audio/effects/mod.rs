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

impl EffectsChain {
    /// Create a new effects chain
    pub fn new() -> Self {
        Self {
            effects: Vec::new(),
            enabled: true,
        }
    }

    /// Add an effect to the chain
    pub fn add_effect(&mut self, effect: Box<dyn AudioEffect>) {
        self.effects.push(effect);
    }

    /// Process audio through the entire effects chain
    pub fn process(&mut self, buffer: &mut [f32], sample_rate: u32, channels: u16) -> Result<(), AudioError> {
        if !self.enabled {
            return Ok(());
        }

        for effect in &mut self.effects {
            if effect.is_enabled() {
                effect.process(buffer, sample_rate, channels)?;
            }
        }

        Ok(())
    }

    /// Enable or disable the entire effects chain
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Check if the effects chain is enabled
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Clear all effects
    pub fn clear(&mut self) {
        self.effects.clear();
    }

    /// Reset all effects
    pub fn reset(&mut self) {
        for effect in &mut self.effects {
            effect.reset();
        }
    }
}

impl Default for EffectsChain {
    fn default() -> Self {
        Self::new()
    }
}
