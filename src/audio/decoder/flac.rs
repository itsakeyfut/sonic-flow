//! FLAC decoder implementation

use std::time::Duration;
use crate::error::AudioError;
use super::super::traits::AudioDecoder;

/// FLAC-specific decoder
///
/// This is a placeholder for a dedicated FLAC decoder.
/// For now, we use the universal decoder for all formats.
pub struct FlacDecoder {
    _placeholder: (),
}

impl FlacDecoder {
    /// Create a new FLAC decoder
    pub fn new() -> Self {
        Self {
            _placeholder: (),
        }
    }
}

impl AudioDecoder for FlacDecoder {
    fn decode(&mut self, _input: &[u8], _output: &mut [f32]) -> Result<usize, AudioError> {
        // TODO: Implement FLAC-specific decoding if needed
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
