//! Symphonia-backed audio decoder.
//!
//! Supports MP3, FLAC, WAV, OGG Vorbis, AAC, OPUS and any other format
//! enabled by the `symphonia` crate's feature flags.

use std::collections::VecDeque;
use std::fs::File;
use std::path::Path;
use std::time::Duration;

use rodio::Source;
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::{CODEC_TYPE_NULL, Decoder as SymphDecoder, DecoderOptions};
use symphonia::core::errors::Error as SymphError;
use symphonia::core::formats::{FormatOptions, FormatReader, SeekMode, SeekTo};
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;
use symphonia::core::units::{Time, TimeBase};

use crate::audio::traits::AudioFormatType;
use crate::error::AudioError;

use super::{AudioBuffer, AudioDecoder, AudioFormatInfo};

/// Symphonia-based decoder that implements both [`AudioDecoder`] and
/// [`rodio::Source`], allowing it to be pushed directly into a `rodio::Sink`.
pub struct SymphoniaDecoder {
    format_reader: Box<dyn FormatReader>,
    decoder: Box<dyn SymphDecoder>,
    /// ID of the selected audio track within the container.
    track_id: u32,
    sample_rate: u32,
    channels: u16,
    duration: Option<Duration>,
    /// Stored to convert timestamp units back to wall-clock time after seeking.
    time_base: Option<TimeBase>,
    format_info: AudioFormatInfo,
    /// Decoded samples waiting to be consumed by the [`Iterator`] impl.
    sample_buf: VecDeque<f32>,
    exhausted: bool,
}

impl SymphoniaDecoder {
    /// Open an audio file and prepare the decoder.
    ///
    /// The container format is probed automatically. The first non-null audio
    /// track is selected for playback.
    pub fn open(path: &Path) -> Result<Self, AudioError> {
        let file = File::open(path).map_err(|e| {
            AudioError::Streaming(format!("Failed to open '{}': {e}", path.display()))
        })?;

        let mut hint = Hint::new();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            hint.with_extension(ext);
        }

        let mss = MediaSourceStream::new(Box::new(file), Default::default());

        let probed = symphonia::default::get_probe()
            .format(
                &hint,
                mss,
                &FormatOptions::default(),
                &MetadataOptions::default(),
            )
            .map_err(|e| {
                AudioError::Decode(format!("Format probe failed for '{}': {e}", path.display()))
            })?;

        let format_reader = probed.format;

        // Select the first non-null audio track.
        let track = format_reader
            .tracks()
            .iter()
            .find(|t| t.codec_params.codec != CODEC_TYPE_NULL)
            .ok_or_else(|| AudioError::Decode("No supported audio track found".into()))?;

        let track_id = track.id;
        let sample_rate = track.codec_params.sample_rate.unwrap_or(44100);
        let channels = track
            .codec_params
            .channels
            .map(|c| c.count() as u16)
            .unwrap_or(2);
        let bit_depth = track.codec_params.bits_per_sample;
        let time_base = track.codec_params.time_base;

        let duration = time_base.zip(track.codec_params.n_frames).map(|(tb, n)| {
            let t = tb.calc_time(n);
            Duration::from_secs_f64(t.seconds as f64 + t.frac)
        });

        let format_type = path
            .extension()
            .and_then(|e| e.to_str())
            .map(AudioFormatType::from_extension)
            .unwrap_or(AudioFormatType::Unknown("unknown".into()));

        let codec_name = format_type.codec_name().to_string();

        let format_info = AudioFormatInfo {
            sample_rate,
            channels,
            bit_depth,
            format_type,
            duration,
            codec_name,
        };

        let decoder = symphonia::default::get_codecs()
            .make(&track.codec_params, &DecoderOptions::default())
            .map_err(|e| AudioError::Decode(format!("Failed to create codec decoder: {e}")))?;

        Ok(Self {
            format_reader,
            decoder,
            track_id,
            sample_rate,
            channels,
            duration,
            time_base,
            format_info,
            sample_buf: VecDeque::new(),
            exhausted: false,
        })
    }

    /// Decode the next packet from the format reader into `self.sample_buf`.
    ///
    /// Returns `true` if new samples were pushed, `false` when the stream is
    /// exhausted or an unrecoverable error occurs.
    fn fill_buffer(&mut self) -> bool {
        loop {
            let packet = match self.format_reader.next_packet() {
                Ok(p) => p,
                // Any IO error (including EOF) means the stream is done.
                Err(SymphError::IoError(_)) => {
                    self.exhausted = true;
                    return false;
                }
                // Decoder state must be reset after a discontinuity.
                Err(SymphError::ResetRequired) => {
                    self.decoder.reset();
                    continue;
                }
                Err(_) => {
                    self.exhausted = true;
                    return false;
                }
            };

            // Skip packets belonging to other tracks (e.g. subtitle or video).
            if packet.track_id() != self.track_id {
                continue;
            }

            match self.decoder.decode(&packet) {
                Ok(decoded) => {
                    let spec = *decoded.spec();
                    let capacity = decoded.capacity() as u64;
                    let mut buf = SampleBuffer::<f32>::new(capacity, spec);
                    buf.copy_interleaved_ref(decoded);
                    self.sample_buf.extend(buf.samples().iter().copied());
                    return true;
                }
                // Damaged packets can be skipped without stopping playback.
                Err(SymphError::DecodeError(_)) => continue,
                Err(_) => {
                    self.exhausted = true;
                    return false;
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// AudioDecoder trait
// ---------------------------------------------------------------------------

impl AudioDecoder for SymphoniaDecoder {
    fn decode_next(&mut self) -> Result<Option<AudioBuffer>, AudioError> {
        if self.exhausted && self.sample_buf.is_empty() {
            return Ok(None);
        }

        if self.sample_buf.is_empty() && !self.fill_buffer() {
            return Ok(None);
        }

        // Drain the entire internal buffer as one logical packet.
        let samples: Vec<f32> = self.sample_buf.drain(..).collect();
        Ok(Some(AudioBuffer {
            samples,
            sample_rate: self.sample_rate,
            channels: self.channels,
        }))
    }

    fn seek(&mut self, position: Duration) -> Result<Duration, AudioError> {
        let secs = position.as_secs_f64();
        let time = Time {
            seconds: secs as u64,
            frac: secs.fract(),
        };

        let seeked = self
            .format_reader
            .seek(
                SeekMode::Accurate,
                SeekTo::Time {
                    time,
                    track_id: Some(self.track_id),
                },
            )
            .map_err(|e| AudioError::Streaming(format!("Seek failed: {e}")))?;

        // Reset decoder state so the next packet is treated as a fresh start.
        self.decoder.reset();
        self.sample_buf.clear();
        self.exhausted = false;

        // Convert the actual timestamp back to wall-clock time.
        let actual = self.time_base.map_or(position, |tb| {
            let t = tb.calc_time(seeked.actual_ts);
            Duration::from_secs_f64(t.seconds as f64 + t.frac)
        });

        Ok(actual)
    }

    fn duration(&self) -> Option<Duration> {
        self.duration
    }

    fn format_info(&self) -> AudioFormatInfo {
        self.format_info.clone()
    }
}

// ---------------------------------------------------------------------------
// rodio::Source — allows the decoder to be pushed directly into a rodio Sink
// ---------------------------------------------------------------------------

impl Iterator for SymphoniaDecoder {
    type Item = f32;

    fn next(&mut self) -> Option<f32> {
        // Fast path: buffered samples from the last decoded packet.
        if let Some(s) = self.sample_buf.pop_front() {
            return Some(s);
        }
        if self.exhausted || !self.fill_buffer() {
            return None;
        }
        self.sample_buf.pop_front()
    }
}

impl Source for SymphoniaDecoder {
    fn current_frame_len(&self) -> Option<usize> {
        // Returning None lets rodio pull samples one at a time from next().
        None
    }

    fn channels(&self) -> u16 {
        self.channels
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        self.duration
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn open_nonexistent_file_returns_error() {
        let result = SymphoniaDecoder::open(&PathBuf::from("nonexistent.mp3"));
        assert!(result.is_err());
    }

    #[test]
    fn open_unsupported_path_returns_streaming_error() {
        // Cannot test with a real file in unit tests, but we can verify that
        // the constructor correctly rejects missing files.
        let result = SymphoniaDecoder::open(&PathBuf::from("missing.flac"));
        assert!(
            matches!(result, Err(AudioError::Streaming(_))),
            "expected Streaming error for missing file"
        );
    }
}
