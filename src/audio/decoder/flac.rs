//! FLAC audio decoder implementation
//!
//! This module provides FLAC-specific decoding functionality optimized for
//! lossless audio playback with high precision and metadata extraction.

use std::path::Path;
use std::time::Duration;

use symphonia::core::audio::AudioBufferRef;
use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::formats::{FormatOptions, FormatReader};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

use crate::error::AudioError;
use super::super::traits::{AudioDecoder, AudioFormat, AudioFormatType};

/// FLAC-specific audio decoder for high-quality lossless audio
pub struct FlacDecoder {
    /// Symphonia format reader for FLAC
    format_reader: Box<dyn FormatReader>,
    /// Symphonia decoder
    decoder: Box<dyn symphonia::core::codecs::Decoder>,
    /// Track information
    track: symphonia::core::formats::Track,
    /// Audio format info
    format: AudioFormat,
    /// Sample buffer for format conversion
    sample_buffer: Option<symphonia::core::audio::SampleBuffer<f32>>,
    /// Current position in samples
    current_position: u64,
    /// Total samples (from FLAC metadata)
    total_samples: Option<u64>,
}

impl FlacDecoder {
    /// Create a new FLAC decoder from a file path
    pub fn from_file(path: &Path) -> Result<Self, AudioError> {
        let file = std::fs::File::open(path).map_err(|e| {
            AudioError::Streaming(format!("Failed to open FLAC file {}: {}", path.display(), e))
        })?;

        let source = Box::new(file);
        let mss = MediaSourceStream::new(source, Default::default());

        // Create hint specifically for FLAC
        let mut hint = Hint::new();
        hint.with_extension("flac");

        // Probe format
        let probed = symphonia::default::get_probe()
            .format(
                &hint,
                mss,
                &FormatOptions::default(),
                &MetadataOptions::default(),
            )
            .map_err(|e| AudioError::Streaming(format!("Failed to probe FLAC: {}", e)))?;

        let format_reader = probed.format;

        // Find the default audio track
        let track = format_reader
            .default_track()
            .or_else(|| {
                format_reader
                    .tracks()
                    .iter()
                    .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            })
            .cloned()
            .ok_or_else(|| AudioError::Streaming("No audio track found in FLAC".to_string()))?;

        // Create decoder
        let decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &DecoderOptions { verify: true }) // Verify for lossless
            .map_err(|e| AudioError::Streaming(format!("Failed to create FLAC decoder: {}", e)))?;

        // Extract format information (FLAC has precise metadata)
        let codec_params = &track.codec_params;
        let format = AudioFormat {
            sample_rate: codec_params
                .sample_rate
                .ok_or_else(|| AudioError::Streaming("FLAC missing sample rate".to_string()))?,
            channels: codec_params
                .channels
                .map(|c| c.count() as u16)
                .ok_or_else(|| AudioError::Streaming("FLAC missing channel info".to_string()))?,
            bit_depth: codec_params
                .bits_per_sample
                .ok_or_else(|| AudioError::Streaming("FLAC missing bit depth".to_string()))?
                as u16,
            format_type: AudioFormatType::Flac,
        };

        // FLAC files typically have precise frame count
        let total_samples = codec_params.n_frames.map(|frames| {
            // For FLAC, n_frames represents total samples
            frames
        });

        Ok(Self {
            format_reader,
            decoder,
            track,
            format,
            sample_buffer: None,
            current_position: 0,
            total_samples,
        })
    }

    /// Read and decode next packet with high precision
    fn read_samples(&mut self, output: &mut [f32]) -> Result<usize, AudioError> {
        // Read next packet
        let packet = match self.format_reader.next_packet() {
            Ok(packet) => packet,
            Err(symphonia::core::errors::Error::IoError(e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                return Ok(0); // End of stream
            }
            Err(e) => return Err(AudioError::Streaming(format!("Error reading FLAC packet: {}", e))),
        };

        // Skip packets for other tracks
        if packet.track_id() != self.track.id {
            return self.read_samples(output);
        }

        // Decode packet and convert to f32 samples with high precision in one go
        let audio_buf = self.decoder.decode(&packet).map_err(|e| {
            AudioError::Streaming(format!("Error decoding FLAC packet: {}", e))
        })?;

        // Initialize sample buffer if needed
        if self.sample_buffer.is_none() {
            self.sample_buffer = Some(symphonia::core::audio::SampleBuffer::new(
                audio_buf.capacity() as u64,
                *audio_buf.spec(),
            ));
        }

        let sample_buffer = self.sample_buffer.as_mut().unwrap();
        sample_buffer.copy_interleaved_ref(audio_buf);

        let samples = sample_buffer.samples();
        let copy_len = output.len().min(samples.len());

        output[..copy_len].copy_from_slice(&samples[..copy_len]);
        self.current_position += copy_len as u64 / self.format.channels as u64;

        Ok(copy_len)
    }


    /// Get current position in seconds with high precision
    pub fn position(&self) -> Duration {
        Duration::from_secs_f64(self.current_position as f64 / self.format.sample_rate as f64)
    }

    /// Get total duration with high precision (FLAC has exact duration)
    pub fn duration(&self) -> Option<Duration> {
        self.total_samples.map(|samples| {
            Duration::from_secs_f64(samples as f64 / self.format.sample_rate as f64)
        })
    }

    /// Get compression ratio (comparing to uncompressed size)
    pub fn compression_ratio(&self) -> Option<f32> {
        if let Some(total_samples) = self.total_samples {
            let _uncompressed_size = total_samples 
                * self.format.channels as u64 
                * (self.format.bit_depth as u64 / 8);
            
            // This would require file size information
            // For now, return None - in a full implementation,
            // we'd track the compressed file size
            None
        } else {
            None
        }
    }

    /// Check if FLAC file has embedded cue sheet
    pub fn has_cue_sheet(&self) -> bool {
        // Check metadata for cue sheet
        // This would be implemented by examining the format reader's metadata
        false // Placeholder
    }

    /// Get FLAC-specific encoding info
    pub fn encoding_info(&self) -> FlacEncodingInfo {
        let codec_params = &self.track.codec_params;
        
        FlacEncodingInfo {
            bits_per_sample: codec_params.bits_per_sample.unwrap_or(16) as u8,
            block_size: None, // Would extract from FLAC metadata blocks
            md5_signature: None, // Would extract from STREAMINFO block
            encoder_version: None, // Would extract from VORBIS_COMMENT
        }
    }
}

/// FLAC-specific encoding information
#[derive(Debug, Clone)]
pub struct FlacEncodingInfo {
    pub bits_per_sample: u8,
    pub block_size: Option<u16>,
    pub md5_signature: Option<[u8; 16]>,
    pub encoder_version: Option<String>,
}

impl AudioDecoder for FlacDecoder {
    fn decode(&mut self, _input: &[u8], output: &mut [f32]) -> Result<usize, AudioError> {
        self.read_samples(output)
    }

    fn sample_rate(&self) -> u32 {
        self.format.sample_rate
    }

    fn channels(&self) -> u16 {
        self.format.channels
    }

    fn seek(&mut self, position: Duration) -> Result<(), AudioError> {
        let timestamp = (position.as_secs_f64() * self.format.sample_rate as f64) as u64;

        self.format_reader
            .seek(
                symphonia::core::formats::SeekMode::Accurate,
                symphonia::core::formats::SeekTo::TimeStamp {
                    ts: timestamp,
                    track_id: self.track.id,
                },
            )
            .map_err(|e| {
                AudioError::Streaming(format!(
                    "Failed to seek in FLAC to {:.2}s: {}",
                    position.as_secs_f64(),
                    e
                ))
            })?;

        self.decoder.reset();
        self.current_position = timestamp;
        Ok(())
    }

    fn supports_seek(&self) -> bool {
        true // FLAC has excellent seeking support
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flac_format_detection() {
        assert_eq!(AudioFormatType::from_extension("flac"), AudioFormatType::Flac);
        assert_eq!(AudioFormatType::from_extension("FLAC"), AudioFormatType::Flac);
    }

    #[test]
    fn test_flac_encoding_info() {
        let info = FlacEncodingInfo {
            bits_per_sample: 24,
            block_size: Some(4096),
            md5_signature: None,
            encoder_version: Some("FLAC 1.4.0".to_string()),
        };
        
        assert_eq!(info.bits_per_sample, 24);
        assert_eq!(info.block_size, Some(4096));
    }

    // Additional tests would include:
    // - High-precision seeking
    // - Multi-channel FLAC support
    // - Metadata extraction
    // - Compression ratio calculation
}
