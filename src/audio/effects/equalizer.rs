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
