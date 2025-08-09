//! MP3 decoder implementation

use super::super::traits::AudioDecoder;
use crate::error::AudioError;
use std::time::Duration;

/// MP3-specific decoder
///
/// This is a placeholder for a dedicated MP3 decoder.
/// For now, we use the universal decoder for all formats.
pub struct Mp3Decoder {
    _placeholder: (),
}

impl Mp3Decoder {
    /// Create a new MP3 decoder
    pub fn new() -> Self {
        Self { _placeholder: () }
    }
}

impl AudioDecoder for Mp3Decoder {
    fn decode(&mut self, _input: &[u8], _output: &mut [f32]) -> Result<usize, AudioError> {
        // TODO: Implement MP3-specific decoding if needed
        // For now, use UniversalDecoder
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
