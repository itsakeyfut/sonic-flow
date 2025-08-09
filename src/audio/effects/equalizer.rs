//! Equalizer effect implementation

use super::AudioEffect;
use crate::error::AudioError;

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

/// 10-band equalizer effect
pub struct Equalizer {
    bands: [EqualizerBand; 10],
    enabled: bool,
}

impl Equalizer {
    /// Create a new equalizer with default bands
    pub fn new() -> Self {
        Self {
            bands: [
                EqualizerBand::new(31.0, 0.0, 0.707),    // 31 Hz
                EqualizerBand::new(62.0, 0.0, 0.707),    // 62 Hz
                EqualizerBand::new(125.0, 0.0, 0.707),   // 125 Hz
                EqualizerBand::new(250.0, 0.0, 0.707),   // 250 Hz
                EqualizerBand::new(500.0, 0.0, 0.707),   // 500 Hz
                EqualizerBand::new(1000.0, 0.0, 0.707),  // 1 kHz
                EqualizerBand::new(2000.0, 0.0, 0.707),  // 2 kHz
                EqualizerBand::new(4000.0, 0.0, 0.707),  // 4 kHz
                EqualizerBand::new(8000.0, 0.0, 0.707),  // 8 kHz
                EqualizerBand::new(16000.0, 0.0, 0.707), // 16 kHz
            ],
            enabled: false,
        }
    }

    /// Set gain for a specific band
    pub fn set_band_gain(&mut self, band_index: usize, gain: f32) -> Result<(), AudioError> {
        if band_index >= self.bands.len() {
            return Err(AudioError::Effects("Invalid band index".to_string()));
        }

        self.bands[band_index].gain = gain.clamp(-12.0, 12.0);
        Ok(())
    }

    /// Get gain for a specific band
    pub fn get_band_gain(&self, band_index: usize) -> Result<f32, AudioError> {
        if band_index >= self.bands.len() {
            return Err(AudioError::Effects("Invalid band index".to_string()));
        }

        Ok(self.bands[band_index].gain)
    }

    /// Get all band configurations
    pub fn bands(&self) -> &[EqualizerBand; 10] {
        &self.bands
    }

    /// Reset all bands to flat (0 dB)
    pub fn reset_to_flat(&mut self) {
        for band in &mut self.bands {
            band.gain = 0.0;
        }
    }
}

impl AudioEffect for Equalizer {
    fn process(&mut self, _buffer: &mut [f32], _sample_rate: u32, _channels: u16) -> Result<(), AudioError> {
        // TODO: Implement actual EQ processing
        // This would involve implementing biquad filters for each band
        Ok(())
    }

    fn reset(&mut self) {
        // TODO: Reset filter states
    }

    fn is_enabled(&self) -> bool {
        self.enabled
    }

    fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }
}

impl Default for Equalizer {
    fn default() -> Self {
        Self::new()
    }
}

/// Equalizer presets
pub enum EqualizerPreset {
    Flat,
    Rock,
    Pop,
    Jazz,
    Classical,
    Electronic,
    Custom([f32; 10]),
}

impl EqualizerPreset {
    /// Get the gain values for the preset
    pub fn gains(&self) -> [f32; 10] {
        match self {
            Self::Flat => [0.0; 10],
            Self::Rock => [5.0, 3.0, -1.0, -2.0, -1.0, 2.0, 4.0, 6.0, 6.0, 6.0],
            Self::Pop => [-1.0, 2.0, 4.0, 4.0, 0.0, -1.0, -1.0, 0.0, 2.0, 3.0],
            Self::Jazz => [4.0, 2.0, 0.0, 1.0, -1.0, -1.0, 0.0, 1.0, 2.0, 4.0],
            Self::Classical => [5.0, 3.0, -1.0, -1.0, -1.0, 0.0, 1.0, 2.0, 3.0, 4.0],
            Self::Electronic => [5.0, 3.0, 1.0, 0.0, -2.0, 2.0, 1.0, 1.0, 4.0, 6.0],
            Self::Custom(gains) => *gains,
        }
    }

    /// Apply preset to an equalizer
    pub fn apply_to(&self, equalizer: &mut Equalizer) -> Result<(), AudioError> {
        let gains = self.gains();
        for (i, &gain) in gains.iter().enumerate() {
            equalizer.set_band_gain(i, gain)?;
        }
        Ok(())
    }
}
