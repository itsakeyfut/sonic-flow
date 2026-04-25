//! Source tap for real-time spectrum analysis.
//!
//! Wraps any `rodio::Source<Item = f32>` and intercepts the audio samples to
//! feed a [`SpectrumAnalyzer`] without altering the audio output.

use std::time::Duration;

use rodio::Source;
use tokio::sync::watch;

use super::analysis::{SpectrumAnalyzer, SpectrumData};

/// Default number of frequency bands for the spectrum pipeline.
pub const DEFAULT_BAND_COUNT: usize = 64;

/// FFT window size used by the tap's internal analyzer.
const FFT_SIZE: usize = 1024;

/// Number of mono samples accumulated before running the analyzer.
///
/// At 44100 Hz stereo this is ~5.8 ms per batch, yielding ~172 FFT updates
/// per second — well above the 60 fps UI target.
const HOP_MONO_SAMPLES: usize = FFT_SIZE / 4;

/// A `rodio::Source` wrapper that intercepts audio samples and publishes
/// real-time [`SpectrumData`] via a `tokio::sync::watch` channel.
///
/// Multi-channel audio is mixed to mono before analysis. The original
/// interleaved stream passes through unmodified to the downstream `Sink`.
pub struct SpectrumTap<S> {
    inner: S,
    channels: u16,
    sample_rate: u32,
    duration: Option<Duration>,
    analyzer: SpectrumAnalyzer,
    /// Accumulated mono samples for the current batch.
    mono_buf: Vec<f32>,
    /// Running sum of the current multi-channel frame (reset each frame).
    frame_accum: f32,
    /// Channel index within the current frame.
    frame_ch: usize,
    tx: watch::Sender<SpectrumData>,
}

impl<S: Source<Item = f32>> SpectrumTap<S> {
    /// Create a new `SpectrumTap` wrapping `inner`.
    ///
    /// `band_count` — number of output frequency bands (32, 64, or 128).
    /// `tx` — watch sender that receives a new [`SpectrumData`] after each
    ///         FFT computation.
    pub fn new(inner: S, band_count: usize, tx: watch::Sender<SpectrumData>) -> Self {
        let channels = inner.channels().max(1);
        let sample_rate = inner.sample_rate();
        let duration = inner.total_duration();
        let analyzer = SpectrumAnalyzer::new(FFT_SIZE, sample_rate, band_count);

        Self {
            inner,
            channels,
            sample_rate,
            duration,
            analyzer,
            mono_buf: Vec::with_capacity(HOP_MONO_SAMPLES),
            frame_accum: 0.0,
            frame_ch: 0,
            tx,
        }
    }
}

impl<S: Source<Item = f32>> Iterator for SpectrumTap<S> {
    type Item = f32;

    #[inline]
    fn next(&mut self) -> Option<f32> {
        let sample = self.inner.next()?;

        // Accumulate samples for the current frame and mix to mono.
        self.frame_accum += sample;
        self.frame_ch += 1;

        if self.frame_ch >= self.channels as usize {
            let mono = self.frame_accum / self.channels as f32;
            self.mono_buf.push(mono);
            self.frame_accum = 0.0;
            self.frame_ch = 0;

            if self.mono_buf.len() >= HOP_MONO_SAMPLES {
                let data = self.analyzer.analyze(&self.mono_buf);
                // Best-effort: drop the result if no receiver is listening.
                let _ = self.tx.send(data);
                self.mono_buf.clear();
            }
        }

        Some(sample)
    }
}

impl<S: Source<Item = f32>> Source for SpectrumTap<S> {
    fn current_frame_len(&self) -> Option<usize> {
        self.inner.current_frame_len()
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

#[cfg(test)]
mod tests {
    use super::*;
    use rodio::Source;

    struct SilenceSource {
        remaining: usize,
    }

    impl SilenceSource {
        fn new(samples: usize) -> Self {
            Self { remaining: samples }
        }
    }

    impl Iterator for SilenceSource {
        type Item = f32;
        fn next(&mut self) -> Option<f32> {
            if self.remaining == 0 {
                return None;
            }
            self.remaining -= 1;
            Some(0.0)
        }
    }

    impl Source for SilenceSource {
        fn current_frame_len(&self) -> Option<usize> {
            None
        }
        fn channels(&self) -> u16 {
            2
        }
        fn sample_rate(&self) -> u32 {
            44100
        }
        fn total_duration(&self) -> Option<Duration> {
            None
        }
    }

    #[test]
    fn tap_passes_samples_through() {
        let silence = SilenceSource::new(100);
        let initial = SpectrumData::new(vec![0.0; DEFAULT_BAND_COUNT], 0.0, 0.0);
        let (tx, _rx) = watch::channel(initial);
        let mut tap = SpectrumTap::new(silence, DEFAULT_BAND_COUNT, tx);

        let samples: Vec<f32> = (0..100).filter_map(|_| tap.next()).collect();
        assert_eq!(samples.len(), 100);
        assert!(samples.iter().all(|&s| s == 0.0));
    }

    #[test]
    fn tap_publishes_spectrum_after_hop() {
        // Feed enough samples to trigger at least one FFT batch.
        // HOP_MONO_SAMPLES = 256; with 2 channels → 512 interleaved samples.
        let sample_count = HOP_MONO_SAMPLES * 2 * 4; // 4 full hops
        let silence = SilenceSource::new(sample_count);
        let initial = SpectrumData::new(vec![0.0; DEFAULT_BAND_COUNT], 0.0, 0.0);
        let (tx, rx) = watch::channel(initial);
        let mut tap = SpectrumTap::new(silence, DEFAULT_BAND_COUNT, tx);

        // Drain all samples.
        let _: Vec<_> = tap.by_ref().collect();

        // The watch receiver should have been updated.
        let data = rx.borrow();
        assert_eq!(data.bands.len(), DEFAULT_BAND_COUNT);
    }
}
