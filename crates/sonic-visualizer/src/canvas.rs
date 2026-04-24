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

        self.pixels[index] = (src_color.r * 255.0) as u8; // R
        self.pixels[index + 1] = (src_color.g * 255.0) as u8; // G
        self.pixels[index + 2] = (src_color.b * 255.0) as u8; // B
        self.pixels[index + 3] = (src_color.a * 255.0) as u8; // A
    }

    /// Get pixel color at coordinates
    fn get_pixel(&self, x: u32, y: u32) -> Color {
        if x >= self.width || y >= self.height {
            return Color::rgba(0.0, 0.0, 0.0, 0.0);
        }

        let index = ((y * self.width + x) * 4) as usize;
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

    /// Fill a rectangle region
    fn fill_rect_region(&mut self, rect: Rect, color: Color) {
        let start_x = rect.x.max(0.0) as u32;
        let start_y = rect.y.max(0.0) as u32;
        let end_x = (rect.x + rect.width).min(self.width as f32) as u32;
        let end_y = (rect.y + rect.height).min(self.height as f32) as u32;

        for y in start_y..end_y {
            for x in start_x..end_x {
                self.set_pixel(x, y, color);
            }
        }
    }
}

impl Canvas for SoftwareCanvas {
    fn size(&self) -> (u32, u32) {
        (self.width, self.height)
    }

    fn clear(&mut self, color: Color) {
        let r = (color.r * 255.0) as u8;
        let g = (color.g * 255.0) as u8;
        let b = (color.b * 255.0) as u8;
        let a = (color.a * 255.0) as u8;

        for chunk in self.pixels.chunks_exact_mut(4) {
            chunk[0] = r;
            chunk[1] = g;
            chunk[2] = b;
            chunk[3] = a;
        }
    }

    fn draw_rect(&mut self, rect: Rect, color: Color) {
        self.fill_rect_region(rect, color);
    }

    fn draw_line(&mut self, start: Point, end: Point, color: Color, width: f32) {
        // Bresenham's line algorithm with width support
        let x0 = start.x as i32;
        let y0 = start.y as i32;
        let x1 = end.x as i32;
        let y1 = end.y as i32;

        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx - dy;

        let mut x = x0;
        let mut y = y0;

        let half_width = (width / 2.0) as i32;

        loop {
            // Draw thick line by drawing multiple pixels around the center point
            for dy_offset in -half_width..=half_width {
                for dx_offset in -half_width..=half_width {
                    let px = x + dx_offset;
                    let py = y + dy_offset;

                    if px >= 0 && px < self.width as i32 && py >= 0 && py < self.height as i32 {
                        self.set_pixel(px as u32, py as u32, color);
                    }
                }
            }

            if x == x1 && y == y1 {
                break;
            }

            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x += sx;
            }
            if e2 < dx {
                err += dx;
                y += sy;
            }
        }
    }

    fn draw_circle(&mut self, center: Point, radius: f32, color: Color) {
        let cx = center.x as i32;
        let cy = center.y as i32;
        let r = radius as i32;

        // Midpoint circle algorithm
        let mut x = 0;
        let mut y = r;
        let mut d = 1 - r;

        while x <= y {
            // Draw 8 symmetric points
            let points = [
                (cx + x, cy + y),
                (cx - x, cy + y),
                (cx + x, cy - y),
                (cx - x, cy - y),
                (cx + y, cy + x),
                (cx - y, cy + x),
                (cx + y, cy - x),
                (cx - y, cy - x),
            ];

            for (px, py) in points {
                if px >= 0 && px < self.width as i32 && py >= 0 && py < self.height as i32 {
                    self.set_pixel(px as u32, py as u32, color);
                }
            }

            if d < 0 {
                d += 2 * x + 3;
            } else {
                d += 2 * (x - y) + 5;
                y -= 1;
            }
            x += 1;
        }
    }

    fn draw_text(&mut self, text: &str, position: Point, color: Color, size: f32) {
        // Simple text rendering - just draw placeholder rectangles for now
        // In a real implementation, this would use a font rendering library
        let char_width = size * 0.6;
        let char_height = size;

        for (i, _) in text.chars().enumerate() {
            let x = position.x + i as f32 * char_width;
            let y = position.y;

            let char_rect = Rect::new(x, y, char_width * 0.8, char_height);
            self.draw_rect(char_rect, color);
        }
    }

    fn draw_polygon(&mut self, points: &[Point], color: Color) {
        if points.len() < 3 {
            return;
        }

        // Simple polygon filling using scanline algorithm
        let min_y = points.iter().map(|p| p.y as i32).min().unwrap_or(0);
        let max_y = points.iter().map(|p| p.y as i32).max().unwrap_or(0);

        for y in min_y..=max_y {
            let mut intersections = Vec::new();

            // Find intersections with polygon edges
            for i in 0..points.len() {
                let j = (i + 1) % points.len();
                let p1 = points[i];
                let p2 = points[j];

                let y1 = p1.y as i32;
                let y2 = p2.y as i32;

                if (y1 <= y && y < y2) || (y2 <= y && y < y1) {
                    let x = p1.x + (y as f32 - p1.y) * (p2.x - p1.x) / (p2.y - p1.y);
                    intersections.push(x as i32);
                }
            }

            // Sort intersections and fill between pairs
            intersections.sort_unstable();
            for chunk in intersections.chunks(2) {
                if chunk.len() == 2 {
                    self.draw_horizontal_line(chunk[0], chunk[1], y, color);
                }
            }
        }
    }

    fn set_blend_mode(&mut self, mode: BlendMode) {
        self.blend_mode = mode;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canvas_creation() {
        let canvas = SoftwareCanvas::new(800, 600);
        assert_eq!(canvas.size(), (800, 600));
        assert_eq!(canvas.pixels.len(), 800 * 600 * 4);
    }

    #[test]
    fn test_canvas_clear() {
        let mut canvas = SoftwareCanvas::new(100, 100);
        let red = Color::rgb(1.0, 0.0, 0.0);

        canvas.clear(red);

        // Check a few pixels
        assert_eq!(canvas.get_pixel(0, 0), red);
        assert_eq!(canvas.get_pixel(50, 50), red);
        assert_eq!(canvas.get_pixel(99, 99), red);
    }

    #[test]
    fn test_draw_rect() {
        let mut canvas = SoftwareCanvas::new(100, 100);
        let blue = Color::rgb(0.0, 0.0, 1.0);

        canvas.clear(Color::rgb(0.0, 0.0, 0.0));
        canvas.draw_rect(Rect::new(10.0, 10.0, 20.0, 20.0), blue);

        // Check that pixels inside the rect are blue
        assert_eq!(canvas.get_pixel(15, 15), blue);
        assert_eq!(canvas.get_pixel(25, 25), blue);

        // Check that pixels outside the rect are black
        assert_eq!(canvas.get_pixel(5, 5), Color::rgb(0.0, 0.0, 0.0));
        assert_eq!(canvas.get_pixel(35, 35), Color::rgb(0.0, 0.0, 0.0));
    }

    #[test]
    fn test_blend_modes() {
        let mut canvas = SoftwareCanvas::new(100, 100);
        let red = Color::rgb(1.0, 0.0, 0.0);
        let green = Color::rgb(0.0, 1.0, 0.0);

        // Set base color
        canvas.set_pixel(50, 50, red);

        // Test additive blending
        canvas.set_blend_mode(BlendMode::Add);
        canvas.set_pixel(50, 50, green);

        let result = canvas.get_pixel(50, 50);
        assert!(result.r > 0.0 && result.g > 0.0); // Should have both red and green
    }

    #[test]
    fn test_resize() {
        let mut canvas = SoftwareCanvas::new(100, 100);
        assert_eq!(canvas.size(), (100, 100));

        canvas.resize(200, 150);
        assert_eq!(canvas.size(), (200, 150));
        assert_eq!(canvas.pixels.len(), 200 * 150 * 4);
    }
}
