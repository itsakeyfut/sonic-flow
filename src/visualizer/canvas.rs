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

impl SoftwareCanvas {
    /// Create a new software canvas
    pub fn new(width: u32, height: u32) -> Self {
        let pixel_count = (width * height * 4) as usize;
        Self {
            width,
            height,
            pixels: vec![0; pixel_count],
            blend_mode: BlendMode::Normal,
        }
    }

    /// Resize the canvas
    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        let pixel_count = (width * height * 4) as usize;
        self.pixels.resize(pixel_count, 0);
    }
}