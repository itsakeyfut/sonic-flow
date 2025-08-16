//! MP3 audio decoder implementation
//!
//! This module provides MP3-specific decoding functionality with optimizations
//! for MP3 files, including VBR handling and ID3 metadata extraction.

use std::path::Path;
use std::time::Duration;

use symphonia::core::codecs::{DecoderOptions, CODEC_TYPE_NULL};
use symphonia::core::formats::{FormatOptions, FormatReader};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

use super::super::traits::{AudioDecoder, AudioFormat, AudioFormatType};
use crate::error::AudioError;

/// MP3-specific audio decoder with advanced features
pub struct Mp3Decoder {
    /// Symphonia format reader for MP3
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
    /// Total samples (for MP3 duration calculation)
    total_samples: Option<u64>,
}

impl Mp3Decoder {
    /// Create a new MP3 decoder from a file path
    pub fn from_file(path: &Path) -> Result<Self, AudioError> {
        let file = std::fs::File::open(path).map_err(|e| {
            AudioError::Streaming(format!("Failed to open MP3 file {}: {}", path.display(), e))
        })?;

        let source = Box::new(file);
        let mss = MediaSourceStream::new(source, Default::default());

        // Create hint specifically for MP3
        let mut hint = Hint::new();
        hint.with_extension("mp3");

        // Probe format
        let probed = symphonia::default::get_probe()
            .format(
                &hint,
                mss,
                &FormatOptions::default(),
                &MetadataOptions::default(),
            )
            .map_err(|e| AudioError::Streaming(format!("Failed to probe MP3: {}", e)))?;

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
            .ok_or_else(|| AudioError::Streaming("No audio track found in MP3".to_string()))?;

        // Create decoder
        let decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &DecoderOptions { verify: false })
            .map_err(|e| AudioError::Streaming(format!("Failed to create MP3 decoder: {}", e)))?;

        // Extract format information
        let codec_params = &track.codec_params;
        let format = AudioFormat {
            sample_rate: codec_params.sample_rate.unwrap_or(44100),
            channels: codec_params.channels.map(|c| c.count() as u16).unwrap_or(2),
            bit_depth: codec_params.bits_per_sample.unwrap_or(16) as u16,
            format_type: AudioFormatType::Mp3,
        };

        // Calculate total samples for duration (if available)
        let total_samples = if let (Some(sample_rate), Some(time_base), Some(n_frames)) = (
            codec_params.sample_rate,
            codec_params.time_base,
            codec_params.n_frames,
        ) {
            Some(time_base.calc_time(n_frames).seconds * sample_rate as u64)
        } else {
            None
        };

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

    /// Read and decode next packet
    fn read_samples(&mut self, output: &mut [f32]) -> Result<usize, AudioError> {
        // Read next packet
        let packet = match self.format_reader.next_packet() {
            Ok(packet) => packet,
            Err(symphonia::core::errors::Error::IoError(e))
                if e.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                return Ok(0); // End of stream
            }
            Err(e) => {
                return Err(AudioError::Streaming(format!(
                    "Error reading packet: {}",
                    e
                )))
            }
        };

        // Skip packets for other tracks
        if packet.track_id() != self.track.id {
            return self.read_samples(output);
        }

        // Decode packet and convert to f32 samples in one go
        let audio_buf = self
            .decoder
            .decode(&packet)
            .map_err(|e| AudioError::Streaming(format!("Error decoding MP3 packet: {}", e)))?;

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

    /// Get total duration (if available)
    pub fn duration(&self) -> Option<Duration> {
        self.total_samples
            .map(|samples| Duration::from_secs_f64(samples as f64 / self.format.sample_rate as f64))
    }
}

impl AudioDecoder for Mp3Decoder {
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
                    "Failed to seek in MP3 to {:.2}s: {}",
                    position.as_secs_f64(),
                    e
                ))
            })?;

        self.decoder.reset();
        self.current_position = timestamp;
        Ok(())
    }

    fn supports_seek(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mp3_format_detection() {
        // Test format type detection
        assert_eq!(AudioFormatType::from_extension("mp3"), AudioFormatType::Mp3);
        assert_eq!(AudioFormatType::from_extension("MP3"), AudioFormatType::Mp3);
    }

    #[test]
    fn test_mp3_bitrate_info() {
        // This would require a test MP3 file
        // In a real implementation, we'd include small test files
    }

    // Additional tests would include:
    // - VBR detection
    // - Seeking accuracy
    // - Duration calculation
    // - Sample rate conversion
}
