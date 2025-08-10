//! Spectrum bars visualizer implementations
//! 
//! This visualizer displays frequency spectrum data as vertical bars,
//! providing a classic and intuitive visualization of audio content.

use std::collections::HashMap;
use std::time::Instant;

use crate::audio::analysis::SpectrumData;
use crate::error::VisualizerError;
use crate::visualizer::traits::{
    BlendMode, Canvas, Color, ColorScheme, ConfigParameter, ParameterType, PluginValue, Point,
    Rect, VisualizationConfig, Visualizer, VisualizerMetadata,
};

/// Spectrum bars visualizer configuration
#[derive(Debug, Clone)]
pub struct SpectrumBarsConfig {
    /// Number of frequency bars to display
    pub bar_count: usize,
    /// Width ratio of bars (0.0 - 1.0, where 1.0 means no gaps)
    pub bar_width_ratio: f32,
    /// Minimum bar height (as fraction of canvas height)
    pub min_bar_height: f32,
    /// Maximum bar height (as fraction of canvas height)
    pub max_bar_height: f32,
    /// Peak hold time in seconds
    pub peak_hold_time: f32,
    /// Peak fall speed (fraction per second)
    pub peak_fall_speed: f32,
    /// Enable peak indicators
    pub show_peaks: bool,
    /// Logarithmic frequency scaling
    pub logarithmic_scale: bool,
    /// Bar style
    pub bar_style: BarStyle,
    /// Gradient direction
    pub gradient_direction: GradientDirection,
}

/// Bar rendering styles
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BarStyle {
    /// Solid filled bars
    Solid,
    /// Outlined bars
    Outlined,
    /// Rounded bars
    Rounded,
    /// Line bars (thin lines)
    Line,
}

/// Gradient directions for bars
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum GradientDirection {
    /// Vertical gradient (bottom to top)
    Vertical,
    /// Horizontal gradient (left to right)
    Horizontal,
    /// Radial gradient from center
    Radial,
    /// No gradient (solid color)
    None,
}

/// Individual bar state
#[derive(Debug, Clone)]
struct BarState {
    /// Current height (0.0 - 1.0)
    height: f32,
    /// Peak height (0.0 - 1.0)
    peak_height: f32,
    /// Peak hold start time
    peak_hold_start: Instant,
    /// Smoothed height for animation
    smoothed_height: f32,
}

/// Spectrum bars visualizer
pub struct SpectrumBarsVisualizer {
    /// Visualizer configuration
    config: SpectrumBarsConfig,
    /// Visualization settings
    vis_config: VisualizationConfig,
    /// Bar states
    bars: Vec<BarState>,
    /// Last update time
    last_update: Instant,
    /// Frequency bin mapping for logarithmic scaling
    frequency_bins: Vec<usize>,
    /// Maximum amplitude for normalization
    max_amplitude: f32,
    /// Smoothing factor for animations
    smoothing_factor: f32,
}

impl SpectrumBarsVisualizer {
    /// Create a new spectrum bars visualizer
    pub fn new() -> Self {
        let config = SpectrumBarsConfig::default();
        let bar_count = config.bar_count;

        Self {
            config,
            vis_config: VisualizationConfig::default(),
            bars: vec![BarState::default(); bar_count],
            last_update: Instant::now(),
            frequency_bins: Vec::new(),
            max_amplitude: 1.0,
            smoothing_factor: 0.8,
        }
    }

    /// Update frequency bin mapping based on configuration
    fn update_frequency_mapping(&mut self, spectrum_size: usize) {
        self.frequency_bins.clear();

        if self.config.logarithmic_scale {
            // Logarithmic frequency mapping
            self.create_log_frequency_mapping(spectrum_size);
        } else {
            // Linear frequency mapping
            self.create_linear_frequency_mapping(spectrum_size);
        }
    }

    /// Create logarithmic frequency mapping
    fn create_log_frequency_mapping(&mut self, spectrum_size: usize) {
        let (min_freq, max_freq) = self.vis_config.frequency_range;
        let log_min = min_freq.ln();
        let log_max = max_freq.ln();
        let log_step = (log_max - log_min) / self.config.bar_count as f32;

        let nyquist = 22050.0; // Assuming 44.1kHz sample rate
        let freq_per_bin = nyquist / spectrum_size as f32;

        for i in 0..self.config.bar_count {
            let freq = (log_min + i as f32 * log_step).exp();
            let bin = (freq / freq_per_bin) as usize;
            self.frequency_bins.push(bin.min(spectrum_size - 1));
        }
    }

    /// Create linear frequency mapping
    fn create_linear_frequency_mapping(&mut self, spectrum_size: usize) {
        let (min_freq, max_freq) = self.vis_config.frequency_range;
        let freq_step = (max_freq - min_freq) / self.config.bar_count as f32;

        let nyquist = 22050.0; // Assuming 44.1kHz sample rate
        let freq_per_bin = nyquist / spectrum_size as f32;

        for i in 0..self.config.bar_count {
            let freq = min_freq + i as f32 * freq_step;
            let bin = (freq / freq_per_bin) as usize;
            self.frequency_bins.push(bin.min(spectrum_size - 1));
        }
    }

    /// Update bar states with new spectrum data
    fn update_bars(&mut self, spectrum_data: &SpectrumData) {
        let now = Instant::now();
        let dt = now.duration_since(self.last_update).as_secs_f32();
        self.last_update = now;

        // Update frequency mapping if needed
        if self.frequency_bins.len() != self.config.bar_count
            || self.frequency_bins.is_empty()
        {
            self.update_frequency_mapping(spectrum_data.bands.len());
        }

        // Apply sensitivity multiplier
        let sensitivity = self.vis_config.sensitivity;

        for (i, bar) in self.bars.iter_mut().enumerate() {
            // Get amplitude for this bar
            let bin_index = self.frequency_bins.get(i).copied().unwrap_or(0);
            let amplitude = spectrum_data
                .bands
                .get(bin_index)
                .copied()
                .unwrap_or(0.0)
                * sensitivity;

            // Update maximum amplitude for auto-gain
            if self.vis_config.auto_gain {
                self.max_amplitude = self.max_amplitude * 0.99 + amplitude * 0.01;
                self.max_amplitude = self.max_amplitude.max(0.1); // Prevent division by zero
            }

            // Normalize amplitude
            let normalized_amplitude = (amplitude / self.max_amplitude).clamp(0.0, 1.0);

            // Update bar height
            if normalized_amplitude > bar.height {
                bar.height = normalized_amplitude;
            } else if self.vis_config.smoothing {
                // Smooth fall
                let fall_rate = 2.0; // Bars per second
                bar.height = (bar.height - fall_rate * dt).max(normalized_amplitude);
            } else {
                bar.height = normalized_amplitude;
            }

            // Update smoothed height for animation
            if self.vis_config.smoothing {
                let smoothing = self.smoothing_factor * self.vis_config.animation_speed;
                bar.smoothed_height = bar.smoothed_height * smoothing + bar.height * (1.0 - smoothing);
            } else {
                bar.smoothed_height = bar.height;
            }

            // Update peak hold
            if self.config.show_peaks {
                if normalized_amplitude > bar.peak_height {
                    bar.peak_height = normalized_amplitude;
                    bar.peak_hold_start = now;
                } else {
                    let hold_elapsed = now.duration_since(bar.peak_hold_start).as_secs_f32();
                    if hold_elapsed > self.config.peak_hold_time {
                        bar.peak_height = (bar.peak_height - self.config.peak_fall_speed * dt)
                            .max(bar.height);
                    }
                }
            }
        }
    }

    /// Render the spectrum bars
    fn render_bars(&self, canvas: &mut dyn Canvas) {
        let (canvas_width, canvas_height) = canvas.size();
        let canvas_width = canvas_width as f32;
        let canvas_height = canvas_height as f32;

        // Calculate bar dimensions
        let total_bar_width = canvas_width / self.config.bar_count as f32;
        let bar_width = total_bar_width * self.config.bar_width_ratio;
        let bar_spacing = total_bar_width - bar_width;

        let min_height = canvas_height * self.config.min_bar_height;
        let max_height = canvas_height * self.config.max_bar_height;
        let height_range = max_height - min_height;

        // Set blend mode
        canvas.set_blend_mode(BlendMode::Normal);

        for (i, bar) in self.bars.iter().enumerate() {
            let x = i as f32 * total_bar_width + bar_spacing / 2.0;
            let height = min_height + bar.smoothed_height * height_range;
            let y = canvas_height - height;

            // Calculate bar color
            let color = self.calculate_bar_color(bar.smoothed_height, i);

            // Render bar based on style
            match self.config.bar_style {
                BarStyle::Solid => self.render_solid_bar(canvas, x, y, bar_width, height, color),
                BarStyle::Outlined => self.render_outlined_bar(canvas, x, y, bar_width, height, color),
                BarStyle::Rounded => self.render_rounded_bar(canvas, x, y, bar_width, height, color),
                BarStyle::Line => self.render_line_bar(canvas, x, y, bar_width, height, color),
            }

            // Render peak indicator
            if self.config.show_peaks && bar.peak_height > bar.smoothed_height {
                let peak_y = canvas_height - (min_height + bar.peak_height * height_range);
                let peak_color = Color::rgba(color.r, color.g, color.b, 0.8);
                
                canvas.draw_line(
                    Point::new(x, peak_y),
                    Point::new(x + bar_width, peak_y),
                    peak_color,
                    2.0,
                );
            }
        }
    }

    /// Calculate color for a bar based on amplitude and position
    fn calculate_bar_color(&self, amplitude: f32, bar_index: usize) -> Color {
        match self.config.gradient_direction {
            GradientDirection::None => self.vis_config.color_scheme.primary,
            GradientDirection::Vertical => {
                // Color based on amplitude
                self.interpolate_gradient_color(amplitude)
            }
            GradientDirection::Horizontal => {
                // Color based on position
                let position = bar_index as f32 / self.config.bar_count as f32;
                self.interpolate_gradient_color(position)
            }
            GradientDirection::Radial => {
                // Color based on distance from center
                let center = self.config.bar_count as f32 / 2.0;
                let distance = (bar_index as f32 - center).abs() / center;
                self.interpolate_gradient_color(distance)
            }
        }
    }
    /// Interpolate color from gradient
    fn interpolate_gradient_color(&self, t: f32) -> Color {
        let t = t.clamp(0.0, 1.0);
        let gradient = &self.vis_config.color_scheme.gradient;

        if gradient.is_empty() {
            return self.vis_config.color_scheme.primary;
        }

        if gradient.len() == 1 {
            return gradient[0];
        }

        let scaled = t * (gradient.len() - 1) as f32;
        let index = scaled.floor() as usize;
        let frac = scaled - index as f32;

        let color1 = gradient[index];
        let color2 = gradient.get(index + 1).copied().unwrap_or(color1);

        Color::rgba(
            color1.r + (color2.r - color1.r) * frac,
            color1.g + (color2.g - color1.g) * frac,
            color1.b + (color2.b - color1.b) * frac,
            color1.a + (color2.a - color1.a) * frac,
        )
    }

    /// Render a solid bar
    fn render_solid_bar(&self, canvas: &mut dyn Canvas, x: f32, y: f32, width: f32, height: f32, color: Color) {
        canvas.draw_rect(Rect::new(x, y, width, height), color);
    }

    /// Render an outlined bar
    fn render_outlined_bar(&self, canvas: &mut dyn Canvas, x: f32, y: f32, width: f32, height: f32, color: Color) {
        // Fill with transparent version
        let fill_color = Color::rgba(color.r, color.g, color.b, 0.3);
        canvas.draw_rect(Rect::new(x, y, width, height), fill_color);

        // Draw outline
        let outline_width = 1.0;
        let bottom_y = y + height;
        
        // Top
        canvas.draw_line(Point::new(x, y), Point::new(x + width, y), color, outline_width);
        // Bottom
        canvas.draw_line(Point::new(x, bottom_y), Point::new(x + width, bottom_y), color, outline_width);
        // Left
        canvas.draw_line(Point::new(x, y), Point::new(x, bottom_y), color, outline_width);
        // Right
        canvas.draw_line(Point::new(x + width, y), Point::new(x + width, bottom_y), color, outline_width);
    }

    /// Render a rounded bar (simplified as regular rect for now)
    fn render_rounded_bar(&self, canvas: &mut dyn Canvas, x: f32, y: f32, width: f32, height: f32, color: Color) {
        // TODO: Implement actual rounded corners
        // For now, just draw a solid bar with slightly smaller dimensions for visual effect
        let corner_radius = width.min(height) * 0.1;
        let adjusted_rect = Rect::new(
            x + corner_radius * 0.5,
            y + corner_radius * 0.5,
            width - corner_radius,
            height - corner_radius,
        );
        canvas.draw_rect(adjusted_rect, color);
    }

    /// Render a line bar
    fn render_line_bar(&self, canvas: &mut dyn Canvas, x: f32, y: f32, width: f32, height: f32, color: Color) {
        let center_x = x + width / 2.0;
        let bottom_y = y + height;
        
        canvas.draw_line(
            Point::new(center_x, bottom_y),
            Point::new(center_x, y),
            color,
            width.min(4.0),
        );
    }
}

impl Default for SpectrumBarsConfig {
    fn default() -> Self {
        Self {
            bar_count: 64,
            bar_width_ratio: 0.8,
            min_bar_height: 0.02,
            max_bar_height: 0.95,
            peak_hold_time: 1.0,
            peak_fall_speed: 0.5,
            show_peaks: true,
            logarithmic_scale: true,
            bar_style: BarStyle::Solid,
            gradient_direction: GradientDirection::Vertical,
        }
    }
}

impl Default for BarState {
    fn default() -> Self {
        Self {
            height: 0.0,
            peak_height: 0.0,
            peak_hold_start: Instant::now(),
            smoothed_height: 0.0,
        }
    }
}

impl Visualizer for SpectrumBarsVisualizer {
    fn metadata(&self) -> VisualizerMetadata {
        VisualizerMetadata {
            id: "spectrum_bars".to_string(),
            name: "Spectrum Bars".to_string(),
            version: "1.0.0".to_string(),
            author: "Sonic Flow Team".to_string(),
            description: "Classic frequency spectrum visualization with vertical bars".to_string(),
            config_schema: vec![
                ConfigParameter {
                    name: "bar_count".to_string(),
                    label: "Number of Bars".to_string(),
                    param_type: ParameterType::Integer,
                    default_value: PluginValue::Integer(64),
                    min_value: Some(PluginValue::Integer(8)),
                    max_value: Some(PluginValue::Integer(256)),
                    description: "Number of frequency bars to display".to_string(),
                },
                ConfigParameter {
                    name: "bar_width_ratio".to_string(),
                    label: "Bar Width Ratio".to_string(),
                    param_type: ParameterType::Float,
                    default_value: PluginValue::Float(0.8),
                    min_value: Some(PluginValue::Float(0.1)),
                    max_value: Some(PluginValue::Float(1.0)),
                    description: "Width ratio of bars (1.0 = no gaps)".to_string(),
                },
                ConfigParameter {
                    name: "show_peaks".to_string(),
                    label: "Show Peak Indicators".to_string(),
                    param_type: ParameterType::Boolean,
                    default_value: PluginValue::Boolean(true),
                    min_value: None,
                    max_value: None,
                    description: "Display peak hold indicators".to_string(),
                },
                ConfigParameter {
                    name: "logarithmic_scale".to_string(),
                    label: "Logarithmic Frequency Scale".to_string(),
                    param_type: ParameterType::Boolean,
                    default_value: PluginValue::Boolean(true),
                    min_value: None,
                    max_value: None,
                    description: "Use logarithmic frequency distribution".to_string(),
                },
                ConfigParameter {
                    name: "bar_style".to_string(),
                    label: "Bar Style".to_string(),
                    param_type: ParameterType::Enum(vec![
                        "Solid".to_string(),
                        "Outlined".to_string(),
                        "Rounded".to_string(),
                        "Line".to_string(),
                    ]),
                    default_value: PluginValue::String("Solid".to_string()),
                    min_value: None,
                    max_value: None,
                    description: "Visual style of the bars".to_string(),
                },
                ConfigParameter {
                    name: "peak_hold_time".to_string(),
                    label: "Peak Hold Time (seconds)".to_string(),
                    param_type: ParameterType::Float,
                    default_value: PluginValue::Float(1.0),
                    min_value: Some(PluginValue::Float(0.0)),
                    max_value: Some(PluginValue::Float(5.0)),
                    description: "How long peaks are held before falling".to_string(),
                },
                ConfigParameter {
                    name: "peak_fall_speed".to_string(),
                    label: "Peak Fall Speed".to_string(),
                    param_type: ParameterType::Float,
                    default_value: PluginValue::Float(0.5),
                    min_value: Some(PluginValue::Float(0.1)),
                    max_value: Some(PluginValue::Float(2.0)),
                    description: "Speed at which peaks fall".to_string(),
                },
                ConfigParameter {
                    name: "min_bar_height".to_string(),
                    label: "Minimum Bar Height".to_string(),
                    param_type: ParameterType::Float,
                    default_value: PluginValue::Float(0.02),
                    min_value: Some(PluginValue::Float(0.0)),
                    max_value: Some(PluginValue::Float(0.5)),
                    description: "Minimum height of bars as fraction of canvas".to_string(),
                },
                ConfigParameter {
                    name: "max_bar_height".to_string(),
                    label: "Maximum Bar Height".to_string(),
                    param_type: ParameterType::Float,
                    default_value: PluginValue::Float(0.95),
                    min_value: Some(PluginValue::Float(0.5)),
                    max_value: Some(PluginValue::Float(1.0)),
                    description: "Maximum height of bars as fraction of canvas".to_string(),
                },
                ConfigParameter {
                    name: "gradient_direction".to_string(),
                    label: "Gradient Direction".to_string(),
                    param_type: ParameterType::Enum(vec![
                        "None".to_string(),
                        "Vertical".to_string(),
                        "Horizontal".to_string(),
                        "Radial".to_string(),
                    ]),
                    default_value: PluginValue::String("Vertical".to_string()),
                    min_value: None,
                    max_value: None,
                    description: "Direction of color gradient".to_string(),
                },
            ],
        }
    }

    fn initialize(&mut self, config: &VisualizationConfig) -> Result<(), VisualizerError> {
        self.vis_config = config.clone();
        
        // Initialize bars array with new count if changed
        if self.bars.len() != self.config.bar_count {
            self.bars = vec![BarState::default(); self.config.bar_count];
        }

        // Reset frequency mapping
        self.frequency_bins.clear();
        
        Ok(())
    }

    fn update(&mut self, spectrum_data: &SpectrumData) -> Result<(), VisualizerError> {
        self.update_bars(spectrum_data);
        Ok(())
    }

    fn render(&self, canvas: &mut dyn Canvas) -> Result<(), VisualizerError> {
        self.render_bars(canvas);
        Ok(())
    }

    fn configure(&mut self, settings: &HashMap<String, PluginValue>) -> Result<(), VisualizerError> {
        // Update configuration based on settings
        for (key, value) in settings {
            match key.as_str() {
                "bar_count" => {
                    if let PluginValue::Integer(count) = value {
                        let count = (*count as usize).clamp(8, 256);
                        if count != self.config.bar_count {
                            self.config.bar_count = count;
                            self.bars = vec![BarState::default(); count];
                            self.frequency_bins.clear();
                        }
                    }
                }
                "bar_width_ratio" => {
                    if let PluginValue::Float(ratio) = value {
                        self.config.bar_width_ratio = ratio.clamp(0.1, 1.0);
                    }
                }
                "show_peaks" => {
                    if let PluginValue::Boolean(show) = value {
                        self.config.show_peaks = *show;
                    }
                }
                "logarithmic_scale" => {
                    if let PluginValue::Boolean(log_scale) = value {
                        if *log_scale != self.config.logarithmic_scale {
                            self.config.logarithmic_scale = *log_scale;
                            self.frequency_bins.clear(); // Force remapping
                        }
                    }
                }
                "bar_style" => {
                    if let PluginValue::String(style) = value {
                        self.config.bar_style = match style.as_str() {
                            "Outlined" => BarStyle::Outlined,
                            "Rounded" => BarStyle::Rounded,
                            "Line" => BarStyle::Line,
                            _ => BarStyle::Solid,
                        };
                    }
                }
                "peak_hold_time" => {
                    if let PluginValue::Float(time) = value {
                        self.config.peak_hold_time = time.clamp(0.0, 5.0);
                    }
                }
                "peak_fall_speed" => {
                    if let PluginValue::Float(speed) = value {
                        self.config.peak_fall_speed = speed.clamp(0.1, 2.0);
                    }
                }
                "min_bar_height" => {
                    if let PluginValue::Float(height) = value {
                        self.config.min_bar_height = height.clamp(0.0, 0.5);
                    }
                }
                "max_bar_height" => {
                    if let PluginValue::Float(height) = value {
                        self.config.max_bar_height = height.clamp(0.5, 1.0);
                    }
                }
                "gradient_direction" => {
                    if let PluginValue::String(direction) = value {
                        self.config.gradient_direction = match direction.as_str() {
                            "Horizontal" => GradientDirection::Horizontal,
                            "Radial" => GradientDirection::Radial,
                            "None" => GradientDirection::None,
                            _ => GradientDirection::Vertical,
                        };
                    }
                }
                _ => {
                    // Unknown setting, ignore
                }
            }
        }
        
        Ok(())
    }

    fn reset(&mut self) {
        // Reset all bar states
        for bar in &mut self.bars {
            bar.height = 0.0;
            bar.peak_height = 0.0;
            bar.smoothed_height = 0.0;
            bar.peak_hold_start = Instant::now();
        }

        // Reset timing
        self.last_update = Instant::now();
        self.max_amplitude = 1.0;
    }

    fn supports_realtime(&self) -> bool {
        true
    }

    fn preferred_update_rate(&self) -> u32 {
        60 // 60 FPS
    }
}