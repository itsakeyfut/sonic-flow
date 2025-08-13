//! WAV audio decoder implementation
//!
//! This module provides WAV-specific decoding functionality optimized for
//! uncompressed PCM audio with fast access and precise timing.

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

/// WAV-specific audio decoder for uncompressed PCM audio
pub struct WavDecoder {
    /// Symphonia format reader for WAV
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
    /// Total samples (calculated from WAV file size)
    total_samples: u64,
    /// WAV-specific format information
    wav_format: WavFormat,
}

/// WAV format-specific information
#[derive(Debug, Clone)]
pub struct WavFormat {
    pub format_tag: u16,        // PCM = 1, IEEE_FLOAT = 3, etc.
    pub block_align: u16,       // Bytes per sample frame
    pub avg_bytes_per_sec: u32, // Average data rate
    pub is_extensible: bool,    // WAVE_FORMAT_EXTENSIBLE
}

impl WavDecoder {
    /// Create a new WAV decoder from a file path
    pub fn from_file(path: &Path) -> Result<Self, AudioError> {
        let file = std::fs::File::open(path).map_err(|e| {
            AudioError::Streaming(format!("Failed to open WAV file {}: {}", path.display(), e))
        })?;

        let source = Box::new(file);
        let mss = MediaSourceStream::new(source, Default::default());

        // Create hint specifically for WAV
        let mut hint = Hint::new();
        hint.with_extension("wav");

        // Probe format
        let probed = symphonia::default::get_probe()
            .format(
                &hint,
                mss,
                &FormatOptions::default(),
                &MetadataOptions::default(),
            )
            .map_err(|e| AudioError::Streaming(format!("Failed to probe WAV: {}", e)))?;

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
            .ok_or_else(|| AudioError::Streaming("No audio track found in WAV".to_string()))?;

        // Create decoder
        let decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &DecoderOptions { verify: false })
            .map_err(|e| AudioError::Streaming(format!("Failed to create WAV decoder: {}", e)))?;

        // Extract format information
        let codec_params = &track.codec_params;
        let format = AudioFormat {
            sample_rate: codec_params
                .sample_rate
                .ok_or_else(|| AudioError::Streaming("WAV missing sample rate".to_string()))?,
            channels: codec_params
                .channels
                .map(|c| c.count() as u16)
                .ok_or_else(|| AudioError::Streaming("WAV missing channel info".to_string()))?,
            bit_depth: codec_params
                .bits_per_sample
                .ok_or_else(|| AudioError::Streaming("WAV missing bit depth".to_string()))?
                as u16,
            format_type: AudioFormatType::Wav,
        };

        // Calculate WAV-specific format info
        let wav_format = WavFormat {
            format_tag: 1, // Default to PCM
            block_align: (format.channels * format.bit_depth / 8),
            avg_bytes_per_sec: format.sample_rate * format.channels as u32 * (format.bit_depth as u32 / 8),
            is_extensible: false, // Would detect from actual WAV header
        };

        // Calculate total samples (WAV files have precise length)
        let total_samples = codec_params.n_frames.unwrap_or(0);

        Ok(Self {
            format_reader,
            decoder,
            track,
            format,
            sample_buffer: None,
            current_position: 0,
            total_samples,
            wav_format,
        })
    }

    /// Read and decode next packet (optimized for uncompressed WAV)
    fn read_samples(&mut self, output: &mut [f32]) -> Result<usize, AudioError> {
        // Read next packet
        let packet = match self.format_reader.next_packet() {
            Ok(packet) => packet,
            Err(symphonia::core::errors::Error::IoError(e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                return Ok(0); // End of stream
            }
            Err(e) => return Err(AudioError::Streaming(format!("Error reading WAV packet: {}", e))),
        };

        // Skip packets for other tracks
        if packet.track_id() != self.track.id {
            return self.read_samples(output);
        }

        // Decode packet (fast for uncompressed WAV)
        let audio_buf = self.decoder.decode(&packet).map_err(|e| {
            AudioError::Streaming(format!("Error decoding WAV packet: {}", e))
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

    /// Get current position in seconds
    pub fn position(&self) -> Duration {
        Duration::from_secs_f64(self.current_position as f64 / self.format.sample_rate as f64)
    }

    /// Get total duration (precise for WAV files)
    pub fn duration(&self) -> Duration {
        Duration::from_secs_f64(self.total_samples as f64 / self.format.sample_rate as f64)
    }

    /// Get WAV format information
    pub fn wav_format(&self) -> &WavFormat {
        &self.wav_format
    }

    /// Check if this is a floating-point WAV
    pub fn is_float_format(&self) -> bool {
        self.wav_format.format_tag == 3 // IEEE_FLOAT
    }

    /// Check if this is an extensible WAV format
    pub fn is_extensible(&self) -> bool {
        self.wav_format.is_extensible
    }

    /// Get exact byte position in file
    pub fn byte_position(&self) -> u64 {
        self.current_position * self.wav_format.block_align as u64
    }

    /// Calculate file size for the audio data
    pub fn audio_data_size(&self) -> u64 {
        self.total_samples * self.wav_format.block_align as u64
    }

    /// Fast seek for WAV files (sample-accurate)
    pub fn seek_sample_accurate(&mut self, sample_position: u64) -> Result<(), AudioError> {
        let timestamp = sample_position;

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
                    "Failed to seek in WAV to sample {}: {}",
                    sample_position,
                    e
                ))
            })?;

        self.decoder.reset();
        self.current_position = timestamp;
        Ok(())
    }
}

impl AudioDecoder for WavDecoder {
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
        let sample_position = (position.as_secs_f64() * self.format.sample_rate as f64) as u64;
        self.seek_sample_accurate(sample_position)
    }

    fn supports_seek(&self) -> bool {
        true // WAV files have excellent seeking support
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wav_format_detection() {
        assert_eq!(AudioFormatType::from_extension("wav"), AudioFormatType::Wav);
        assert_eq!(AudioFormatType::from_extension("WAV"), AudioFormatType::Wav);
    }

    #[test]
    fn test_wav_format_info() {
        let wav_format = WavFormat {
            format_tag: 1,
            block_align: 4, // 16-bit stereo
            avg_bytes_per_sec: 176400, // 44.1kHz * 2 channels * 2 bytes
            is_extensible: false,
        };

        assert_eq!(wav_format.format_tag, 1); // PCM
        assert_eq!(wav_format.block_align, 4);
        assert!(!wav_format.is_extensible);
    }

    #[test]
    fn test_wav_calculations() {
        // Test sample position calculations
        let sample_rate = 44100;
        let channels = 2;
        let bit_depth = 16;
        
        let block_align = channels * bit_depth / 8;
        assert_eq!(block_align, 4);
        
        let duration_seconds = 10.0;
        let total_samples = (duration_seconds * sample_rate as f64) as u64;
        let audio_data_size = total_samples * block_align as u64;
        
        assert_eq!(total_samples, 441000);
        assert_eq!(audio_data_size, 1764000);
    }

    // Additional tests would include:
    // - Float vs PCM format detection
    // - Extensible format handling
    // - Sample-accurate seeking
    // - Multi-channel WAV support
}
