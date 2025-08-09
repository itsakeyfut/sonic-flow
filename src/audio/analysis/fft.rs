//! FFT (Fast Fourier Transform) implementation for audio analysis
//!
//! This module provides real-time FFT processing for spectrum analysis
//! used by the visualizer system.

use rustfft::{FftPlanner, num_complex::Complex};
use std::collections::VecDeque;

use super::SpectrumData;

/// Window function types for FFT processing
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WindowFunction {
    /// Rectangular window (no windowing)
    Rectangular,
    /// Hann window (good general purpose)
    Hann,
    /// Hamming window
    Hamming,
    /// Blackman window (good frequency resolution)
    Blackman,
    /// Blackman-Harris window
    BlackmanHarris,
}

/// FFT-based spectrum analyzer
pub struct SpectrumAnalyzer {
    /// FFT size (must be power of 2)
    fft_size: usize,
    /// FFT planner for creating FFT instances
    planner: FftPlanner<f32>,
    /// FFT instance
    fft: std::sync::Arc<dyn rustfft::Fft<f32>>,
    /// Input buffer for FFT
    input_buffer: Vec<Complex<f32>>,
    /// Output buffer for FFT
    output_buffer: Vec<Complex<f32>>,
    /// Window function coefficients
    window: Vec<f32>,
    /// Window function type
    window_function: WindowFunction,
    /// Sample rate for frequency mapping
    sample_rate: u32,
    /// Overlap buffer for windowed processing
    overlap_buffer: VecDeque<f32>,
    /// Overlap ratio (0.0 to 0.75)
    overlap_ratio: f32,
    /// Number of frequency bands to output
    output_bands: usize,
    /// Frequency range for analysis (Hz)
    frequency_range: (f32, f32),
    /// Logarithmic frequency spacing
    logarithmic_spacing: bool,
}

impl SpectrumAnalyzer {
    /// Create a new spectrum analyzer
    ///
    /// # Arguments
    ///
    /// * `fft_size` - Size of FFT (must be power of 2, typically 1024, 2048, or 4096)
    /// * `sample_rate` - Audio sample rate in Hz
    /// * `output_bands` - Number of frequency bands to output (typically 32, 64, or 128)
    pub fn new(fft_size: usize, sample_rate: u32, output_bands: usize) -> Self {
        assert!(fft_size.is_power_of_two(), "FFT size must be power of 2");
        assert!(output_bands > 0, "Output bands must be > 0");

        let mut planner = FftPlanner::new();
        let fft = planner.plan_fft_forward(fft_size);

        let window_function = WindowFunction::Hann;
        let window = Self::generate_window(fft_size, window_function);

        Self {
            fft_size,
            planner,
            fft,
            input_buffer: vec![Complex::new(0.0, 0.0); fft_size],
            output_buffer: vec![Complex::new(0.0, 0.0); fft_size],
            window,
            window_function,
            sample_rate,
            overlap_buffer: VecDeque::new(),
            overlap_ratio: 0.5, // 50% overlap
            output_bands,
            frequency_range: (20.0, (sample_rate / 2) as f32), // 20 Hz to Nyquist
            logarithmic_spacing: true,
        }
    }

    /// Set the window function
    pub fn set_window_function(&mut self, window_function: WindowFunction) {
        if window_function != self.window_function {
            self.window_function = window_function;
            self.window = Self::generate_window(self.fft_size, window_function);
        }
    }

    /// Set the overlap ratio
    pub fn set_overlap_ratio(&mut self, ratio: f32) {
        self.overlap_ratio = ratio.clamp(0.0, 0.75);
    }

    /// Set the frequency range for analysis
    pub fn set_frequency_range(&mut self, min_freq: f32, max_freq: f32) {
        let max_freq = max_freq.min((self.sample_rate / 2) as f32);
        let min_freq = min_freq.max(1.0);
        self.frequency_range = (min_freq, max_freq);
    }

    /// Set logarithmic frequency spacing
    pub fn set_logarithmic_spacing(&mut self, logarithmic: bool) {
        self.logarithmic_spacing = logarithmic;
    }

    /// Analyze audio samples and return spectrum data
    ///
    /// # Arguments
    ///
    /// * `samples` - Audio samples to analyze (mono)
    ///
    /// # Returns
    ///
    /// Spectrum data with frequency bands
    pub fn analyze(&mut self, samples: &[f32]) -> SpectrumData {
        // Add samples to overlap buffer
        for &sample in samples {
            self.overlap_buffer.push_back(sample);
        }

        let mut all_bands = Vec::new();
        let hop_size = (self.fft_size as f32 * (1.0 - self.overlap_ratio)) as usize;

        // Process overlapping windows
        while self.overlap_buffer.len() >= self.fft_size {
            // Fill input buffer with windowed samples
            for (i, sample) in self.overlap_buffer.iter().take(self.fft_size).enumerate() {
                let windowed_sample = sample * self.window[i];
                self.input_buffer[i] = Complex::new(windowed_sample, 0.0);
            }

            // Remove processed samples (hop size)
            for _ in 0..hop_size.min(self.overlap_buffer.len()) {
                self.overlap_buffer.pop_front();
            }

            // Perform FFT
            self.output_buffer.copy_from_slice(&self.input_buffer);
            self.fft.process(&mut self.output_buffer);

            // Convert to magnitude spectrum
            let magnitude_spectrum = self.compute_magnitude_spectrum();

            // Convert to frequency bands
            let bands = self.spectrum_to_bands(&magnitude_spectrum);
            all_bands.extend(bands);
        }

        // Average overlapping results
        let final_bands = if all_bands.is_empty() {
            vec![0.0; self.output_bands]
        } else {
            let num_windows = all_bands.len() / self.output_bands;
            let mut averaged_bands = vec![0.0; self.output_bands];
            
            for (i, &value) in all_bands.iter().enumerate() {
                averaged_bands[i % self.output_bands] += value;
            }
            
            if num_windows > 0 {
                for band in &mut averaged_bands {
                    *band /= num_windows as f32;
                }
            }
            
            averaged_bands
        };

        // Calculate peak and RMS levels from original samples
        let (peak_level, rms_level) = self.calculate_levels(samples);

        SpectrumData::new(final_bands, peak_level, rms_level)
    }

    /// Generate window function coefficients
    fn generate_window(size: usize, window_type: WindowFunction) -> Vec<f32> {
        let mut window = vec![0.0; size];
        let n = size as f32;

        for (i, coeff) in window.iter_mut().enumerate() {
            let i = i as f32;
            *coeff = match window_type {
                WindowFunction::Rectangular => 1.0,
                WindowFunction::Hann => 0.5 * (1.0 - (2.0 * std::f32::consts::PI * i / (n - 1.0)).cos()),
                WindowFunction::Hamming => 0.54 - 0.46 * (2.0 * std::f32::consts::PI * i / (n - 1.0)).cos(),
                WindowFunction::Blackman => {
                    0.42 - 0.5 * (2.0 * std::f32::consts::PI * i / (n - 1.0)).cos()
                        + 0.08 * (4.0 * std::f32::consts::PI * i / (n - 1.0)).cos()
                }
                WindowFunction::BlackmanHarris => {
                    0.35875 - 0.48829 * (2.0 * std::f32::consts::PI * i / (n - 1.0)).cos()
                        + 0.14128 * (4.0 * std::f32::consts::PI * i / (n - 1.0)).cos()
                        - 0.01168 * (6.0 * std::f32::consts::PI * i / (n - 1.0)).cos()
                }
            };
        }

        window
    }

    /// Compute magnitude spectrum from FFT output
    fn compute_magnitude_spectrum(&self) -> Vec<f32> {
        let mut magnitude = Vec::with_capacity(self.fft_size / 2 + 1);

        for complex in self.output_buffer.iter().take(self.fft_size / 2 + 1) {
            let mag = complex.norm();
            magnitude.push(mag);
        }

        magnitude
    }

    /// Convert magnitude spectrum to frequency bands
    fn spectrum_to_bands(&self, magnitude_spectrum: &[f32]) -> Vec<f32> {
        let mut bands = vec![0.0; self.output_bands];
        let nyquist = (self.sample_rate / 2) as f32;
        let bin_frequency_step = nyquist / (magnitude_spectrum.len() - 1) as f32;

        // Create frequency band boundaries
        let band_boundaries = if self.logarithmic_spacing {
            self.create_log_bands()
        } else {
            self.create_linear_bands()
        };

        // Map spectrum bins to bands
        for (band_idx, &(start_freq, end_freq)) in band_boundaries.iter().enumerate() {
            let start_bin = (start_freq / bin_frequency_step) as usize;
            let end_bin = ((end_freq / bin_frequency_step) as usize + 1).min(magnitude_spectrum.len());

            if start_bin < end_bin && band_idx < bands.len() {
                let mut sum = 0.0;
                let mut count = 0;

                for bin_idx in start_bin..end_bin {
                    sum += magnitude_spectrum[bin_idx];
                    count += 1;
                }

                if count > 0 {
                    bands[band_idx] = sum / count as f32;
                }
            }
        }

        // Normalize bands
        let max_value = bands.iter().fold(0.0f32, |acc, &x| acc.max(x));
        if max_value > 0.0 {
            for band in &mut bands {
                *band /= max_value;
            }
        }

        bands
    }

    /// Create logarithmically spaced frequency bands
    fn create_log_bands(&self) -> Vec<(f32, f32)> {
        let (min_freq, max_freq) = self.frequency_range;
        let log_min = min_freq.ln();
        let log_max = max_freq.ln();
        let log_step = (log_max - log_min) / self.output_bands as f32;

        let mut bands = Vec::with_capacity(self.output_bands);
        for i in 0..self.output_bands {
            let start_freq = (log_min + i as f32 * log_step).exp();
            let end_freq = (log_min + (i + 1) as f32 * log_step).exp();
            bands.push((start_freq, end_freq));
        }

        bands
    }

    /// Create linearly spaced frequency bands
    fn create_linear_bands(&self) -> Vec<(f32, f32)> {
        let (min_freq, max_freq) = self.frequency_range;
        let freq_step = (max_freq - min_freq) / self.output_bands as f32;

        let mut bands = Vec::with_capacity(self.output_bands);
        for i in 0..self.output_bands {
            let start_freq = min_freq + i as f32 * freq_step;
            let end_freq = min_freq + (i + 1) as f32 * freq_step;
            bands.push((start_freq, end_freq));
        }

        bands
    }

    /// Calculate peak and RMS levels from audio samples
    fn calculate_levels(&self, samples: &[f32]) -> (f32, f32) {
        if samples.is_empty() {
            return (0.0, 0.0);
        }

        let mut peak = 0.0f32;
        let mut sum_squares = 0.0f32;

        for &sample in samples {
            let abs_sample = sample.abs();
            peak = peak.max(abs_sample);
            sum_squares += sample * sample;
        }

        let rms = (sum_squares / samples.len() as f32).sqrt();

        (peak, rms)
    }

    /// Get FFT size
    pub fn fft_size(&self) -> usize {
        self.fft_size
    }

    /// Get sample rate
    pub fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    /// Get number of output bands
    pub fn output_bands(&self) -> usize {
        self.output_bands
    }

    /// Get frequency range
    pub fn frequency_range(&self) -> (f32, f32) {
        self.frequency_range
    }

    /// Reset the analyzer state
    pub fn reset(&mut self) {
        self.overlap_buffer.clear();
        for sample in &mut self.input_buffer {
            *sample = Complex::new(0.0, 0.0);
        }
        for sample in &mut self.output_buffer {
            *sample = Complex::new(0.0, 0.0);
        }
    }
}
