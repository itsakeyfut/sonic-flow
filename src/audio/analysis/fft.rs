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
