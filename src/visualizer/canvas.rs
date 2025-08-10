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

    /// Get the pixel buffer for display
    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    /// Set a pixel at the given coordinates
    fn set_pixel(&mut self, x: u32, y: u32, color: Color) {
        if x >= self.width || y >= self.height {
            return;
        }

        let index = ((y * self.width + x) * 4) as usize;
        if index + 3 >= self.pixels.len() {
            return;
        }

        let src_color = self.apply_blend_mode(color, x, y);

        self.pixels[index] = (src_color.r * 255.0) as u8;     // R
        self.pixels[index + 1] = (src_color.g * 255.0) as u8; // G
        self.pixels[index + 2] = (src_color.b * 255.0) as u8; // B
        self.pixels[index + 3] = (src_color.a * 255.0) as u8; // A
    }

    /// Get pixel color at coordinates
    fn get_pixel(&self, x: u32, y: u32) -> Color {
        if x >= self.width || y >= self.height {
            return Color::rgba(0.0, 0.0, 0.0, 0.0,);
        }

        let index = ((y *self.width + x) * 4) as usize;
        if index + 3 >= self.pixels.len() {
            return Color::rgba(0.0, 0.0, 0.0, 0.0);
        }

        Color::rgba(
            self.pixels[index] as f32 / 255.0,
            self.pixels[index + 1] as f32 / 255.0,
            self.pixels[index + 2] as f32 / 255.0,
            self.pixels[index + 3] as f32 / 255.0,
        )
    }

    /// Apply blend mode to color
    fn apply_blend_mode(&self, src_color: Color, x: u32, y: u32) -> Color {
        match self.blend_mode {
            BlendMode::Normal => src_color,
            BlendMode::Add => {
                let dst_color = self.get_pixel(x, y);
                Color::rgba(
                    (src_color.r + dst_color.r).min(1.0),
                    (src_color.g + dst_color.g).min(1.0),
                    (src_color.b + dst_color.b).min(1.0),
                    src_color.a,
                )
            }
            BlendMode::Multiply => {
                let dst_color = self.get_pixel(x, y);
                Color::rgba(
                    src_color.r * dst_color.r,
                    src_color.g * dst_color.g,
                    src_color.b * dst_color.b,
                    src_color.a,
                )
            }
            BlendMode::Screen => {
                let dst_color = self.get_pixel(x, y);
                Color::rgba(
                    1.0 - (1.0 - src_color.r) * (1.0 - dst_color.r),
                    1.0 - (1.0 - src_color.g) * (1.0 - dst_color.g),
                    1.0 - (1.0 - src_color.b) * (1.0 - dst_color.b),
                    src_color.a,
                )
            }
        }
    }

    /// Draw a horizontal line (optimized)
    fn draw_horizontal_line(&mut self, x1: i32, x2: i32, y: i32, color: Color) {
        if y < 0 || y >= self.height as i32 {
            return;
        }

        let start_x = x1.min(x2).max(0) as u32;
        let end_x = x1.max(x2).min(self.width as i32 - 1) as u32;

        for x in start_x..=end_x {
            self.set_pixel(x, y as u32, color);
        }
    }
}