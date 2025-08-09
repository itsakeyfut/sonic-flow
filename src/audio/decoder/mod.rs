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

impl UniversalDecoder {
    /// Create a new universal decoder from a file path
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the audio file
    ///
    /// # Errors
    ///
    /// Returns `AudioError` if the file cannot be opened or decoded.
    pub fn from_file(path: &Path) -> Result<Self, AudioError> {
        // Open the file
        let file = std::fs::File::open(path)
            .map_err(|e| AudioError::Decoder(DecoderError::InitializationFailed {
                format: format!("Failed to open file {}: {}", path.display(), e),
            }))?;

        // Create media source stream
        let source = Box::new(file);
        let mss = MediaSourceStream::new(source, Default::default());

        // Create format hint from file extension
        let mut hint = Hint::new();
        if let Some(extension) = path.extension().and_then(|s| s.to_str()) {
            hint.with_extension(extension);
        }

        // Probe the format
        let probed = symphonia::default::get_probe()
            .format(&hint, mss, &FormatOptions::default(), &MetadataOptions::default())
            .map_err(|e| AudioError::Decoder(DecoderError::InitializationFailed {
                format: format!("Failed to probe format: {}", e),
            }))?;

        let format_reader = probed.format;

        // Find the best audio track
        let track = format_reader
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .cloned()
            .ok_or_else(|| AudioError::Decoder(DecoderError::InitializationFailed {
                format: "No suitable audio track found".to_string(),
            }))?;

        // Create decoder for the track
        let decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &DecoderOptions { verify: false })
            .map_err(|e| AudioError::Decoder(DecoderError::InitializationFailed {
                format: format!("Failed to create decoder: {}", e),
            }))?;

        // Extract format information
        let codec_params = &track.codec_params;
        let sample_rate = codec_params.sample_rate.unwrap_or(44100);
        let channels = codec_params.channels
            .map(|ch| ch.count() as u16)
            .unwrap_or(2);
        let bit_depth = codec_params.bits_per_sample.unwrap_or(16) as u16;

        // Determine format type from codec
        let format_type = match codec_params.codec {
            symphonia::core::codecs::CODEC_TYPE_MP3 => AudioFormatType::Mp3,
            symphonia::core::codecs::CODEC_TYPE_FLAC => AudioFormatType::Flac,
            symphonia::core::codecs::CODEC_TYPE_PCM_S16LE |
            symphonia::core::codecs::CODEC_TYPE_PCM_S24LE |
            symphonia::core::codecs::CODEC_TYPE_PCM_F32LE => AudioFormatType::Wav,
            symphonia::core::codecs::CODEC_TYPE_VORBIS => AudioFormatType::Ogg,
            symphonia::core::codecs::CODEC_TYPE_AAC => AudioFormatType::Aac,
            _ => {
                // Try to determine from file extension as fallback
                if let Some(extension) = path.extension().and_then(|s| s.to_str()) {
                    AudioFormatType::from_extension(extension)
                } else {
                    AudioFormatType::Unknown("unknown".to_string())
                }
            }
        };

        let format = AudioFormat {
            sample_rate,
            channels,
            bit_depth,
            format_type,
        };

        Ok(Self {
            format_reader,
            decoder,
            track,
            format,
            sample_buffer: None,
        })
    }

    /// Get the duration of the audio file
    /// 
    /// # Returns
    /// 
    /// Duration of the audio file, or None if not available
    pub fn duration(&self) -> Option<Duration> {
        self.track.codec_params.n_frames
            .map(|frames| {
                let sample_rate = self.format.sample_rate as u64;
                Duration::from_secs(frames / sample_rate)
            })
    }

    /// Read and decode the next packet of audio data
    ///
    /// # Arguments
    ///
    /// * `output` - Buffer to write decoded samples
    ///
    /// # Returns
    ///
    /// Number of samples written, or 0 if end of stream
    ///
    /// # Errors
    ///
    /// Returns `AudioError` if decoding fails
    pub fn read_samples(&mut self, output: &mut [f32]) -> Result<usize, AudioError> {
        // Read the next packet
        let packet = match self.format_reader.next_packet() {
            Ok(packet) => packet,
            Err(SymphoniaError::IoError(ref err)) if err.kind() == std::io::ErrorKind::UnexpectedEof => {
                // End of stream
                return Ok(0);
            }
            Err(e) => {
                return Err(AudioError::Decoder(DecoderError::CorruptedData(
                    format!("Failed to read packet: {}", e),
                )));
            }
        };

        // Skip packets that don't belong to our track
        if packet.track_id() != self.track.id {
            return self.read_samples(output);
        }

        // Decode the packet
        let decoded = self.decoder.decode(&packet)
            .map_err(|e| AudioError::Decoder(DecoderError::CorruptedData(
                format!("Failed to decode packet: {}", e),
            )))?;

        // Convert to f32 samples
        self.convert_samples(&decoded, output)
    }
}