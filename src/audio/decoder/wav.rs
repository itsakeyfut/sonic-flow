//! WAV decoder implementation

use super::super::traits::AudioDecoder;
use crate::error::AudioError;
use std::time::Duration;

/// WAV-specific decoder
///
/// This is a placeholder for a dedicated WAV decoder.
/// For now, we use the universal decoder for all formats.
pub struct WavDecoder {
    _placeholder: (),
}

impl WavDecoder {
    /// Create a new WAV decoder
    pub fn new() -> Self {
        Self { _placeholder: () }
    }
}

impl AudioDecoder for WavDecoder {
    fn decode(&mut self, _input: &[u8], _output: &mut [f32]) -> Result<usize, AudioError> {
        // TODO: Implement WAV-specific decoding if needed
        unimplemented!("Use UniversalDecoder instead")
    }

    fn sample_rate(&self) -> u32 {
        44100 // Default
    }

    fn channels(&self) -> u16 {
        2 // Default stereo
    }

    fn seek(&mut self, _position: Duration) -> Result<(), AudioError> {
        unimplemented!("Use UniversalDecoder instead")
    }

    fn supports_seek(&self) -> bool {
        true
    }
}
