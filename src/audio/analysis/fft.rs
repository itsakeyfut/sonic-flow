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

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::PI;

    #[test]
    fn test_spectrum_analyzer_creation() {
        let analyzer = SpectrumAnalyzer::new(1024, 44100, 64);
        assert_eq!(analyzer.fft_size(), 1024);
        assert_eq!(analyzer.sample_rate(), 44100);
        assert_eq!(analyzer.output_bands(), 64);
    }

    #[test]
    #[should_panic(expected = "FFT size must be power of 2")]
    fn test_invalid_fft_size() {
        SpectrumAnalyzer::new(1000, 44100, 64); // Not a power of 2
    }

    #[test]
    fn test_window_function_generation() {
        let hann_window = SpectrumAnalyzer::generate_window(8, WindowFunction::Hann);
        assert_eq!(hann_window.len(), 8);
        assert!((hann_window[0] - 0.0).abs() < 1e-6); // Should be close to 0 at edges
        assert!((hann_window[7] - 0.0).abs() < 1e-6);
        assert!(hann_window[4] > 0.9); // Should be close to 1 at center

        let rect_window = SpectrumAnalyzer::generate_window(8, WindowFunction::Rectangular);
        assert!(rect_window.iter().all(|&x| (x - 1.0).abs() < 1e-6)); // All should be 1.0
    }

    #[test]
    fn test_frequency_range_setting() {
        let mut analyzer = SpectrumAnalyzer::new(1024, 44100, 64);
        analyzer.set_frequency_range(50.0, 15000.0);
        assert_eq!(analyzer.frequency_range(), (50.0, 15000.0));

        // Test clamping
        analyzer.set_frequency_range(-10.0, 50000.0); // Should clamp min to 1.0, max to 22050
        let (min_freq, max_freq) = analyzer.frequency_range();
        assert_eq!(min_freq, 1.0);
        assert_eq!(max_freq, 22050.0);
    }

    #[test]
    fn test_analyze_sine_wave() {
        let mut analyzer = SpectrumAnalyzer::new(1024, 44100, 32);
        
        // Generate a 1kHz sine wave
        let sample_rate = 44100.0;
        let frequency = 1000.0;
        let duration_samples = 2048;
        
        let samples: Vec<f32> = (0..duration_samples)
            .map(|i| (2.0 * PI * frequency * i as f32 / sample_rate).sin())
            .collect();

        let spectrum_data = analyzer.analyze(&samples);
        
        assert_eq!(spectrum_data.bands.len(), 32);
        assert!(spectrum_data.peak_level > 0.8); // Should detect significant signal
        assert!(spectrum_data.rms_level > 0.5);
        
        // The 1kHz signal should create a peak in one of the frequency bands
        let max_band = spectrum_data.bands.iter().fold(0.0f32, |acc, &x| acc.max(x));
        assert!(max_band > 0.1); // Should have significant energy in some band
    }

    #[test]
    fn test_analyze_silence() {
        let mut analyzer = SpectrumAnalyzer::new(1024, 44100, 32);
        let samples = vec![0.0; 2048]; // Silence
        
        let spectrum_data = analyzer.analyze(&samples);
        
        assert_eq!(spectrum_data.bands.len(), 32);
        assert_eq!(spectrum_data.peak_level, 0.0);
        assert_eq!(spectrum_data.rms_level, 0.0);
        
        // All bands should be zero for silence
        for &band in &spectrum_data.bands {
            assert_eq!(band, 0.0);
        }
    }

    #[test]
    fn test_window_function_setting() {
        let mut analyzer = SpectrumAnalyzer::new(1024, 44100, 32);
        
        analyzer.set_window_function(WindowFunction::Blackman);
        // We can't directly test the internal state, but we can verify it doesn't panic
        
        analyzer.set_window_function(WindowFunction::Hamming);
        analyzer.set_window_function(WindowFunction::Rectangular);
    }

    #[test]
    fn test_overlap_ratio_setting() {
        let mut analyzer = SpectrumAnalyzer::new(1024, 44100, 32);
        
        analyzer.set_overlap_ratio(0.75);
        analyzer.set_overlap_ratio(-0.1); // Should clamp to 0.0
        analyzer.set_overlap_ratio(0.9);  // Should clamp to 0.75
    }

    #[test]
    fn test_logarithmic_spacing() {
        let mut analyzer = SpectrumAnalyzer::new(1024, 44100, 10);
        
        analyzer.set_logarithmic_spacing(true);
        let log_bands = analyzer.create_log_bands();
        assert_eq!(log_bands.len(), 10);
        
        // Check that bands get progressively wider in linear frequency
        let linear_widths: Vec<f32> = log_bands.iter()
            .map(|(start, end)| end - start)
            .collect();
        
        // Each band should be wider than the previous (approximately)
        for i in 1..linear_widths.len() {
            assert!(linear_widths[i] > linear_widths[i-1] * 0.9); // Allow some tolerance
        }
        
        analyzer.set_logarithmic_spacing(false);
        let linear_bands = analyzer.create_linear_bands();
        assert_eq!(linear_bands.len(), 10);
        
        // Linear bands should have roughly equal widths
        let linear_widths: Vec<f32> = linear_bands.iter()
            .map(|(start, end)| end - start)
            .collect();
        
        let expected_width = linear_widths[0];
        for width in &linear_widths {
            assert!((width - expected_width).abs() < 1.0); // Small tolerance for floating point
        }
    }

    #[test]
    fn test_reset() {
        let mut analyzer = SpectrumAnalyzer::new(1024, 44100, 32);
        
        // Add some samples
        let samples = vec![0.5; 2000];
        analyzer.analyze(&samples);
        
        // Reset should clear internal state
        analyzer.reset();
        
        // Analyzing silence after reset should give clean results
        let silence = vec![0.0; 1024];
        let spectrum_data = analyzer.analyze(&silence);
        
        assert_eq!(spectrum_data.peak_level, 0.0);
        assert_eq!(spectrum_data.rms_level, 0.0);
    }
}
