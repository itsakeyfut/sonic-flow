//! Equalizer effect implementation

use super::AudioEffect;
use crate::error::AudioError;
use crate::Result;

/// Equalizer band configuration
#[derive(Debug, Clone)]
pub struct EqualizerBand {
    /// Center frequency in Hz
    pub frequency: f32,
    /// Gain in dB (-12.0 to +12.0)
    pub gain: f32,
    /// Q factor (bandwidth)
    pub q_factor: f32,
}

impl EqualizerBand {
    /// Create a new equalizer band
    pub fn new(frequency: f32, gain: f32, q_factor: f32) -> Self {
        Self {
            frequency,
            gain: gain.clamp(-12.0, 12.0),
            q_factor,
        }
    }
}
