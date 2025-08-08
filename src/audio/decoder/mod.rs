//! Audio decoder implementation
//! 
//! This module provides audio decoders for various formats using the
//! symphonia crate for maximum format support and performance.

use std::path::Path;
use std::time::Duration;

use symphonia::core::audio::{AudioBufferRef, Signal};
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::{FormatOptions, FormatReader, Track};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

use crate::error::{AudioError, DecoderError};
use crate::Result;

use super::traits::{AudioDecoder, AudioFormat, AudioFormatType};

pub mod mp3;
pub mod flac;
pub mod wav;

pub use mp3::Mp3Decoder;
pub use flac::FlacDecoder;
pub use wav::WavDecoder;

/// Universal audio decoder that can handle multiple formats
pub struct UniversalDecoder {
    /// Symphonia format reader
    format_reader: Box<dyn FormatReader>,
    /// Symphonia decoder
    decoder: Box<dyn symphonia::core::codecs::Decoder>,
    /// Selected audio track
    track: Track,
    /// Audio format information
    format: AudioFormat,
    /// Current sample buffer
    sample_buffer: Option<symphonia::core::audio::SampleBuffer<f32>>,
}
