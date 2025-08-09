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
