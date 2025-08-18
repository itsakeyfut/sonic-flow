//! Performance monitoring and optimization utilities

use std::collections::VecDeque;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Performance metrics for audio visualization
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Frame rendering time (milliseconds)
    pub frame_time: f32,
    /// FPS (frames per second)
    pub fps: f32,
    /// GPU memory usage (bytes)
    pub gpu_memory: u64,
    /// CPU usage percentage
    pub cpu_usage: f32,
    /// Audio processing time (milliseconds)
    pub audio_processing_time: f32,
    /// Shader compilation time (milliseconds)
    pub shader_compilation_time: f32,
    /// Buffer upload time (milliseconds)
    pub buffer_upload_time: f32,
    /// Draw calls per frame
    pub draw_calls: u32,
    /// Vertex count per frame
    pub vertex_count: u32,
    /// Fragment count per frame
    pub fragment_count: u32,
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            frame_time: 0.0,
            fps: 0.0,
            gpu_memory: 0,
            cpu_usage: 0.0,
            audio_processing_time: 0.0,
            shader_compilation_time: 0.0,
            buffer_upload_time: 0.0,
            draw_calls: 0,
            vertex_count: 0,
            fragment_count: 0,
        }
    }
}

/// Performance monitor for tracking metrics over time
pub struct PerformanceMonitor {
    /// Historical metrics (rolling window)
    metrics_history: VecDeque<PerformanceMetrics>,
    /// Maximum history size
    max_history_size: usize,
    /// Last frame start time
    last_frame_start: Option<Instant>,
    /// Frame count
    frame_count: u64,
    /// Total running time
    total_time: Duration,
    /// Performance thresholds
    thresholds: PerformanceThresholds,
    /// Optimization suggestions
    suggestions: Vec<OptimizationSuggestion>,
}

/// Performance thresholds for triggering optimizations
#[derive(Debug, Clone)]
pub struct PerformanceThresholds {
    /// Target FPS
    pub target_fps: f32,
    /// Maximum frame time (ms)
    pub max_frame_time: f32,
    /// Maximum CPU usage (%)
    pub max_cpu_usage: f32,
    /// Maximum GPU memory usage (MB)
    pub max_gpu_memory_mb: u64,
    /// Maximum audio processing time (ms)
    pub max_audio_processing_time: f32,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            target_fps: 60.0,
            max_frame_time: 16.67, // 60 FPS = 16.67ms per frame
            max_cpu_usage: 80.0,
            max_gpu_memory_mb: 512, // 512 MB
            max_audio_processing_time: 5.0,
        }
    }
}

/// Optimization suggestions
#[derive(Debug, Clone)]
pub enum OptimizationSuggestion {
    /// Reduce shader complexity
    ReduceShaderComplexity,
    /// Lower resolution
    LowerResolution,
    /// Reduce audio buffer size
    ReduceAudioBufferSize,
    /// Disable expensive effects
    DisableExpensiveEffects,
    /// Use simpler blending
    UseSimplerBlending,
    /// Reduce particle count
    ReduceParticleCount,
    /// Enable frame skipping
    EnableFrameSkipping,
    /// Optimize texture usage
    OptimizeTextureUsage,
}

impl OptimizationSuggestion {
    pub fn description(&self) -> &'static str {
        match self {
            Self::ReduceShaderComplexity => "Consider simplifying shader code to reduce GPU load",
            Self::LowerResolution => "Lower rendering resolution to improve performance",
            Self::ReduceAudioBufferSize => "Reduce audio buffer size to decrease processing time",
            Self::DisableExpensiveEffects => "Disable expensive visual effects to improve frame rate",
            Self::UseSimplerBlending => "Use simpler blending modes to reduce GPU overhead",
            Self::ReduceParticleCount => "Reduce particle count in particle system effects",
            Self::EnableFrameSkipping => "Enable frame skipping to maintain target frame rate",
            Self::OptimizeTextureUsage => "Optimize texture usage and reduce texture memory",
        }
    }

    pub fn priority(&self) -> u8 {
        match self {
            Self::ReduceShaderComplexity => 1,
            Self::LowerResolution => 2,
            Self::DisableExpensiveEffects => 3,
            Self::ReduceParticleCount => 4,
            Self::EnableFrameSkipping => 5,
            Self::OptimizeTextureUsage => 6,
            Self::ReduceAudioBufferSize => 7,
            Self::UseSimplerBlending => 8,
        }
    }
}

impl PerformanceMonitor {
    /// Create a new performance monitor
    pub fn new(max_history_size: usize) -> Self {
        Self {
            metrics_history: VecDeque::with_capacity(max_history_size),
            max_history_size,
            last_frame_start: None,
            frame_count: 0,
            total_time: Duration::ZERO,
            thresholds: PerformanceThresholds::default(),
            suggestions: Vec::new(),
        }
    }

    /// Start frame timing
    pub fn start_frame(&mut self) {
        self.last_frame_start = Some(Instant::now());
    }

    /// End frame timing and record metrics
    pub fn end_frame(&mut self, metrics: PerformanceMetrics) {
        if let Some(start_time) = self.last_frame_start {
            let frame_duration = start_time.elapsed();
            self.total_time += frame_duration;
            self.frame_count += 1;

            // Update metrics with actual frame time
            let mut updated_metrics = metrics;
            updated_metrics.frame_time = frame_duration.as_secs_f32() * 1000.0;
            updated_metrics.fps = 1.0 / frame_duration.as_secs_f32();

            // Add to history
            self.metrics_history.push_back(updated_metrics.clone());

            // Maintain history size
            if self.metrics_history.len() > self.max_history_size {
                self.metrics_history.pop_front();
            }

            // Check performance and generate suggestions
            self.check_performance(&updated_metrics);
        }
    }

    /// Get current performance metrics
    pub fn current_metrics(&self) -> Option<&PerformanceMetrics> {
        self.metrics_history.back()
    }

    /// Get average metrics over the last N frames
    pub fn average_metrics(&self, frame_count: usize) -> Option<PerformanceMetrics> {
        if self.metrics_history.is_empty() || frame_count == 0 {
            return None;
        }

        let count = frame_count.min(self.metrics_history.len());
        let recent_metrics: Vec<&PerformanceMetrics> = self
            .metrics_history
            .iter()
            .rev()
            .take(count)
            .collect();

        let mut avg = PerformanceMetrics::default();
        let count_f32 = count as f32;

        for metrics in recent_metrics {
            avg.frame_time += metrics.frame_time;
            avg.fps += metrics.fps;
            avg.gpu_memory += metrics.gpu_memory;
            avg.cpu_usage += metrics.cpu_usage;
            avg.audio_processing_time += metrics.audio_processing_time;
            avg.shader_compilation_time += metrics.shader_compilation_time;
            avg.buffer_upload_time += metrics.buffer_upload_time;
            avg.draw_calls += metrics.draw_calls;
            avg.vertex_count += metrics.vertex_count;
            avg.fragment_count += metrics.fragment_count;
        }

        avg.frame_time /= count_f32;
        avg.fps /= count_f32;
        avg.gpu_memory /= count as u64;
        avg.cpu_usage /= count_f32;
        avg.audio_processing_time /= count_f32;
        avg.shader_compilation_time /= count_f32;
        avg.buffer_upload_time /= count_f32;
        avg.draw_calls /= count as u32;
        avg.vertex_count /= count as u32;
        avg.fragment_count /= count as u32;

        Some(avg)
    }

    /// Check performance against thresholds and generate suggestions
    fn check_performance(&mut self, metrics: &PerformanceMetrics) {
        self.suggestions.clear();

        // Check frame time
        if metrics.frame_time > self.thresholds.max_frame_time {
            self.suggestions.push(OptimizationSuggestion::ReduceShaderComplexity);
            if metrics.frame_time > self.thresholds.max_frame_time * 1.5 {
                self.suggestions.push(OptimizationSuggestion::LowerResolution);
            }
        }

        // Check FPS
        if metrics.fps < self.thresholds.target_fps {
            self.suggestions.push(OptimizationSuggestion::DisableExpensiveEffects);
            if metrics.fps < self.thresholds.target_fps * 0.5 {
                self.suggestions.push(OptimizationSuggestion::EnableFrameSkipping);
            }
        }

        // Check CPU usage
        if metrics.cpu_usage > self.thresholds.max_cpu_usage {
            self.suggestions.push(OptimizationSuggestion::ReduceAudioBufferSize);
        }

        // Check GPU memory
        if metrics.gpu_memory > self.thresholds.max_gpu_memory_mb * 1024 * 1024 {
            self.suggestions.push(OptimizationSuggestion::OptimizeTextureUsage);
        }

        // Check audio processing time
        if metrics.audio_processing_time > self.thresholds.max_audio_processing_time {
            self.suggestions.push(OptimizationSuggestion::ReduceAudioBufferSize);
        }

        // Sort suggestions by priority
        self.suggestions.sort_by_key(|s| s.priority());

        // Log warnings for severe performance issues
        if metrics.fps < 30.0 {
            warn!("Low FPS detected: {:.1} FPS", metrics.fps);
        }
        if metrics.cpu_usage > 90.0 {
            warn!("High CPU usage detected: {:.1}%", metrics.cpu_usage);
        }
    }

    /// Get optimization suggestions
    pub fn suggestions(&self) -> &[OptimizationSuggestion] {
        &self.suggestions
    }

    /// Get performance statistics
    pub fn statistics(&self) -> PerformanceStatistics {
        let frame_count = self.frame_count as f32;
        let total_seconds = self.total_time.as_secs_f32();

        PerformanceStatistics {
            total_frames: self.frame_count,
            total_time: self.total_time,
            average_fps: if total_seconds > 0.0 { frame_count / total_seconds } else { 0.0 },
            min_fps: self.metrics_history.iter().map(|m| m.fps).fold(f32::INFINITY, f32::min),
            max_fps: self.metrics_history.iter().map(|m| m.fps).fold(0.0, f32::max),
            average_frame_time: self.metrics_history.iter().map(|m| m.frame_time).sum::<f32>() / frame_count.max(1.0),
            average_cpu_usage: self.metrics_history.iter().map(|m| m.cpu_usage).sum::<f32>() / frame_count.max(1.0),
            peak_gpu_memory: self.metrics_history.iter().map(|m| m.gpu_memory).max().unwrap_or(0),
        }
    }

    /// Update performance thresholds
    pub fn update_thresholds(&mut self, thresholds: PerformanceThresholds) {
        self.thresholds = thresholds;
        info!("Updated performance thresholds: {:?}", self.thresholds);
    }

    /// Clear performance history
    pub fn clear_history(&mut self) {
        self.metrics_history.clear();
        self.frame_count = 0;
        self.total_time = Duration::ZERO;
        info!("Cleared performance history");
    }

    /// Export performance data for analysis
    pub fn export_data(&self) -> Vec<PerformanceMetrics> {
        self.metrics_history.iter().cloned().collect()
    }
}

/// Performance statistics summary
#[derive(Debug, Clone)]
pub struct PerformanceStatistics {
    /// Total frames rendered
    pub total_frames: u64,
    /// Total running time
    pub total_time: Duration,
    /// Average FPS
    pub average_fps: f32,
    /// Minimum FPS
    pub min_fps: f32,
    /// Maximum FPS
    pub max_fps: f32,
    /// Average frame time (ms)
    pub average_frame_time: f32,
    /// Average CPU usage (%)
    pub average_cpu_usage: f32,
    /// Peak GPU memory usage (bytes)
    pub peak_gpu_memory: u64,
}

/// Performance benchmark runner
pub struct PerformanceBenchmark {
    /// Benchmark duration
    duration: Duration,
    /// Target frame rate
    target_fps: f32,
    /// Performance monitor
    monitor: PerformanceMonitor,
}

impl PerformanceBenchmark {
    /// Create a new performance benchmark
    pub fn new(duration: Duration, target_fps: f32) -> Self {
        Self {
            duration,
            target_fps,
            monitor: PerformanceMonitor::new(1000), // Store 1000 frames
        }
    }

    /// Run a performance benchmark
    pub fn run<F>(&mut self, mut render_fn: F) -> BenchmarkResults
    where
        F: FnMut() -> PerformanceMetrics,
    {
        let start_time = Instant::now();
        let target_frame_time = Duration::from_secs_f32(1.0 / self.target_fps);
        let mut frame_count = 0;

        info!("Starting performance benchmark for {:?}", self.duration);

        while start_time.elapsed() < self.duration {
            let frame_start = Instant::now();

            // Start frame timing
            self.monitor.start_frame();

            // Render frame
            let metrics = render_fn();

            // End frame timing
            self.monitor.end_frame(metrics);

            frame_count += 1;

            // Frame rate limiting
            let frame_duration = frame_start.elapsed();
            if frame_duration < target_frame_time {
                std::thread::sleep(target_frame_time - frame_duration);
            }
        }

        let total_time = start_time.elapsed();
        let statistics = self.monitor.statistics();

        info!("Benchmark completed: {} frames in {:?}", frame_count, total_time);

        BenchmarkResults {
            frame_count,
            total_time,
            statistics,
            suggestions: self.monitor.suggestions().to_vec(),
        }
    }
}

/// Benchmark results
#[derive(Debug, Clone)]
pub struct BenchmarkResults {
    /// Total frames rendered
    pub frame_count: u64,
    /// Total benchmark time
    pub total_time: Duration,
    /// Performance statistics
    pub statistics: PerformanceStatistics,
    /// Optimization suggestions
    pub suggestions: Vec<OptimizationSuggestion>,
}

impl BenchmarkResults {
    /// Print benchmark results
    pub fn print_summary(&self) {
        println!("\n📊 Performance Benchmark Results");
        println!("================================");
        println!("Total frames: {}", self.frame_count);
        println!("Total time: {:.2}s", self.total_time.as_secs_f32());
        println!("Average FPS: {:.1}", self.statistics.average_fps);
        println!("Min FPS: {:.1}", self.statistics.min_fps);
        println!("Max FPS: {:.1}", self.statistics.max_fps);
        println!("Average frame time: {:.2}ms", self.statistics.average_frame_time);
        println!("Average CPU usage: {:.1}%", self.statistics.average_cpu_usage);
        println!("Peak GPU memory: {:.1}MB", self.statistics.peak_gpu_memory as f32 / 1024.0 / 1024.0);

        if !self.suggestions.is_empty() {
            println!("\n💡 Optimization Suggestions:");
            for suggestion in &self.suggestions {
                println!("  - {}", suggestion.description());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_performance_metrics_default() {
        let metrics = PerformanceMetrics::default();
        assert_eq!(metrics.frame_time, 0.0);
        assert_eq!(metrics.fps, 0.0);
        assert_eq!(metrics.gpu_memory, 0);
    }

    #[test]
    fn test_performance_monitor_creation() {
        let monitor = PerformanceMonitor::new(100);
        assert_eq!(monitor.metrics_history.len(), 0);
        assert_eq!(monitor.max_history_size, 100);
    }

    #[test]
    fn test_performance_thresholds_default() {
        let thresholds = PerformanceThresholds::default();
        assert_eq!(thresholds.target_fps, 60.0);
        assert_eq!(thresholds.max_frame_time, 16.67);
        assert_eq!(thresholds.max_cpu_usage, 80.0);
    }

    #[test]
    fn test_optimization_suggestion_priority() {
        let suggestion1 = OptimizationSuggestion::ReduceShaderComplexity;
        let suggestion2 = OptimizationSuggestion::LowerResolution;
        assert!(suggestion1.priority() < suggestion2.priority());
    }
}
