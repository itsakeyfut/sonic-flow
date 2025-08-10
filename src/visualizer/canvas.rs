//! Canvas implementations for visualizer rendering
//! 
//! Provides a software-based rendering canvas that can be used
//! by visualizer plugins to draw graphics.

use std::vec::Vec;

use super::traits::{BlendMode, Canvas, Color, Point, Rect};

/// Software-based canvas implementation
pub struct SoftwareCanvas {
    /// Canvas width in pixels
    width: u32,
    /// Canvas height in pixels
    height: u32,
    /// Pixel buffer (RGBA format)
    pixels: Vec<u8>,
    /// Current blend mode
    blend_mode: BlendMode,
}
