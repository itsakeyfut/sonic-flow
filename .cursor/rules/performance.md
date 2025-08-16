# Sonic Flow - Performance Optimization Rules

## 📚 Performance References

- **Rust Performance Book**: https://nnethercote.github.io/perf-book/
- **Tokio Performance**: https://tokio.rs/tokio/topics/performance
- **RustFFT Documentation**: https://docs.rs/rustfft/latest/rustfft/
- **CPAL Documentation**: https://docs.rs/cpal/latest/cpal/
- **Crossbeam Documentation**: https://docs.rs/crossbeam/latest/crossbeam/
- **Rayon Documentation**: https://docs.rs/rayon/latest/rayon/
- **Criterion Benchmarking**: https://bheisler.github.io/criterion.rs/book/

## Critical Performance Requirements

### Non-Negotiable Targets

- **Audio Latency**: ≤ 50ms total pipeline latency
- **UI Responsiveness**: ≤ 16ms per frame (60fps minimum)
- **Visualizer Rendering**: ≤ 8.3ms per frame (120fps target)
- **Memory Usage**: ≤ 100MB idle, ≤ 200MB active
- **CPU Usage**: ≤ 5% during playback

### Measurement and Monitoring

```rust
// Performance monitoring integration
use std::time::Instant;

pub struct PerformanceMonitor {
    frame_times: RingBuffer<Duration>,
    audio_latency: MovingAverage,
    memory_usage: AtomicU64,
}

impl PerformanceMonitor {
    #[inline]
    pub fn measure_frame_time<F, R>(&self, operation: F) -> R
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = operation();
        let duration = start.elapsed();

        self.frame_times.push(duration);

        // Alert if frame time exceeds budget
        if duration > Duration::from_millis(16) {
            tracing::warn!("Frame time budget exceeded: {:?}", duration);
        }

        result
    }
}

// Use in hot paths
#[inline]
pub fn render_visualizer(&mut self, data: &SpectrumData) -> Result<()> {
    self.perf_monitor.measure_frame_time(|| {
        self.internal_render(data)
    })
}
```

## Audio Processing Optimization

### Lock-Free Audio Pipeline

```rust
// ✅ Use lock-free structures for audio thread
use crossbeam::atomic::AtomicCell;
use crossbeam::queue::SegQueue;

pub struct RealTimeAudioProcessor {
    // Triple buffering for audio data
    input_buffers: [AtomicCell<Option<AudioBuffer>>; 3],
    output_buffers: [AtomicCell<Option<AudioBuffer>>; 3],
    write_index: AtomicUsize,
    read_index: AtomicUsize,

    // Lock-free command queue
    commands: SegQueue<AudioCommand>,

    // Pre-allocated buffers to avoid allocation
    scratch_buffer: Vec<f32>,
    fft_buffer: Vec<Complex<f32>>,
}

impl RealTimeAudioProcessor {
    // ✅ Non-blocking audio processing
    #[inline]
    pub fn process_audio_frame(&mut self, samples: &[f32]) -> Option<SpectrumData> {
        // Process commands without blocking
        while let Some(command) = self.commands.pop() {
            self.handle_command_nonblocking(command);
        }

        // Reuse scratch buffer
        self.scratch_buffer.clear();
        self.scratch_buffer.extend_from_slice(samples);

        // Apply window function in-place
        self.apply_window_function(&mut self.scratch_buffer);

        // FFT processing with pre-allocated buffer
        self.fft_processor.process_in_place(&mut self.fft_buffer);

        // Return spectrum data
        Some(self.extract_spectrum_data())
    }
}
```

### SIMD Optimization

```rust
// ✅ Use SIMD for audio processing hot paths
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

pub struct SIMDAudioProcessor;

impl SIMDAudioProcessor {
    /// Apply gain with AVX2 acceleration
    #[target_feature(enable = "avx2")]
    #[inline]
    pub unsafe fn apply_gain_avx2(samples: &mut [f32], gain: f32) {
        let gain_vec = _mm256_set1_ps(gain);

        // Process 8 samples at a time
        for chunk in samples.chunks_exact_mut(8) {
            let data = _mm256_loadu_ps(chunk.as_ptr());
            let result = _mm256_mul_ps(data, gain_vec);
            _mm256_storeu_ps(chunk.as_mut_ptr(), result);
        }

        // Handle remaining samples
        let remainder = samples.len() % 8;
        if remainder > 0 {
            let start = samples.len() - remainder;
            for sample in &mut samples[start..] {
                *sample *= gain;
            }
        }
    }

    /// Mix stereo channels with SIMD
    #[target_feature(enable = "avx2")]
    #[inline]
    pub unsafe fn mix_stereo_avx2(left: &mut [f32], right: &[f32], balance: f32) {
        let balance_vec = _mm256_set1_ps(balance);
        let inv_balance_vec = _mm256_set1_ps(1.0 - balance);

        for (l_chunk, r_chunk) in left.chunks_exact_mut(8).zip(right.chunks_exact(8)) {
            let left_data = _mm256_loadu_ps(l_chunk.as_ptr());
            let right_data = _mm256_loadu_ps(r_chunk.as_ptr());

            let mixed = _mm256_add_ps(
                _mm256_mul_ps(left_data, inv_balance_vec),
                _mm256_mul_ps(right_data, balance_vec)
            );

            _mm256_storeu_ps(l_chunk.as_mut_ptr(), mixed);
        }
    }

    /// Runtime feature detection
    pub fn apply_gain_optimized(samples: &mut [f32], gain: f32) {
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx2") {
                return unsafe { Self::apply_gain_avx2(samples, gain) };
            }
        }

        // Fallback implementation
        for sample in samples {
            *sample *= gain;
        }
    }
}
```

## Memory Management

### Buffer Pool Pattern

```rust
// ✅ Reuse buffers to avoid allocation in audio thread
use std::sync::Arc;
use parking_lot::Mutex;

pub struct BufferPool<T> {
    free_buffers: Arc<Mutex<Vec<Vec<T>>>>,
    buffer_size: usize,
    max_pool_size: usize,
}

impl<T: Default + Clone> BufferPool<T> {
    pub fn new(buffer_size: usize, max_pool_size: usize) -> Self {
        let mut free_buffers = Vec::with_capacity(max_pool_size);

        // Pre-allocate buffers
        for _ in 0..max_pool_size / 2 {
            free_buffers.push(vec![T::default(); buffer_size]);
        }

        Self {
            free_buffers: Arc::new(Mutex::new(free_buffers)),
            buffer_size,
            max_pool_size,
        }
    }

    pub fn acquire(&self) -> PooledBuffer<T> {
        let buffer = {
            let mut free_buffers = self.free_buffers.lock();
            free_buffers.pop().unwrap_or_else(|| {
                tracing::debug!("Buffer pool empty, allocating new buffer");
                vec![T::default(); self.buffer_size]
            })
        };

        PooledBuffer::new(buffer, Arc::clone(&self.free_buffers))
    }
}

pub struct PooledBuffer<T> {
    buffer: Option<Vec<T>>,
    pool: Arc<Mutex<Vec<Vec<T>>>>,
}

impl<T> PooledBuffer<T> {
    fn new(buffer: Vec<T>, pool: Arc<Mutex<Vec<T>>>) -> Self {
        Self {
            buffer: Some(buffer),
            pool,
        }
    }
}

impl<T> std::ops::Deref for PooledBuffer<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        self.buffer.as_ref().unwrap()
    }
}

impl<T> std::ops::DerefMut for PooledBuffer<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.buffer.as_mut().unwrap()
    }
}

impl<T> Drop for PooledBuffer<T> {
    fn drop(&mut self) {
        if let Some(mut buffer) = self.buffer.take() {
            buffer.clear(); // Keep capacity, clear contents

            let mut pool = self.pool.lock();
            if pool.len() < pool.capacity() {
                pool.push(buffer);
            }
        }
    }
}
```

### Zero-Copy Data Sharing

```rust
// ✅ Use Arc for immutable data sharing
use std::sync::Arc;

#[derive(Clone)]
pub struct SharedSpectrumData {
    data: Arc<SpectrumDataInner>,
}

struct SpectrumDataInner {
    frequencies: Vec<f32>,
    magnitudes: Vec<f32>,
    sample_rate: f32,
    timestamp: Instant,
}

impl SharedSpectrumData {
    pub fn new(frequencies: Vec<f32>, magnitudes: Vec<f32>, sample_rate: f32) -> Self {
        Self {
            data: Arc::new(SpectrumDataInner {
                frequencies,
                magnitudes,
                sample_rate,
                timestamp: Instant::now(),
            }),
        }
    }

    // Zero-copy slice access
    pub fn magnitudes(&self) -> &[f32] {
        &self.data.magnitudes
    }

    pub fn frequencies(&self) -> &[f32] {
        &self.data.frequencies
    }
}
```

## Threading and Concurrency

### Real-Time Thread Setup

```rust
// ✅ Configure audio thread for real-time processing
pub fn setup_audio_thread() -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        // Set real-time priority on Linux
        unsafe {
            let param = libc::sched_param {
                sched_priority: 80,
            };

            if libc::sched_setscheduler(0, libc::SCHED_FIFO, &param) != 0 {
                tracing::warn!("Failed to set real-time priority");
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Set high priority class on Windows
        unsafe {
            let handle = kernel32::GetCurrentThread();
            if kernel32::SetThreadPriority(handle, winapi::um::winbase::THREAD_PRIORITY_TIME_CRITICAL) == 0 {
                tracing::warn!("Failed to set thread priority");
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        // Set high priority on macOS
        unsafe {
            let policy = mach::thread_policy::THREAD_EXTENDED_POLICY;
            let mut info = mach::thread_policy::thread_extended_policy_data_t {
                timeshare: 0, // Disable timesharing
            };

            mach::traps::thread_policy_set(
                mach::traps::mach_thread_self(),
                policy,
                &mut info as *mut _ as *mut i32,
                1,
            );
        }
    }

    Ok(())
}
```

### Lock-Free Communication

```rust
// ✅ Use lock-free queues for real-time communication
use crossbeam::queue::{ArrayQueue, SegQueue};

pub struct AudioCommandQueue {
    commands: ArrayQueue<AudioCommand>,
}

impl AudioCommandQueue {
    pub fn new(capacity: usize) -> Self {
        Self {
            commands: ArrayQueue::new(capacity),
        }
    }

    /// Non-blocking push (returns error if full)
    pub fn try_push(&self, command: AudioCommand) -> Result<(), AudioCommand> {
        self.commands.push(command)
    }

    /// Non-blocking pop
    pub fn try_pop(&self) -> Option<AudioCommand> {
        self.commands.pop()
    }

    /// Force push (overwrites oldest if full)
    pub fn force_push(&self, command: AudioCommand) {
        if self.commands.push(command).is_err() {
            // Queue is full, drop oldest command
            let _ = self.commands.pop();
            let _ = self.commands.push(command);
        }
    }
}
```

## Visualization Performance

### Efficient Rendering Pipeline

```rust
// ✅ Optimize visualizer rendering
pub struct OptimizedSpectrumRenderer {
    // Pre-allocated vertex buffers
    vertex_buffer: Vec<Vertex>,
    index_buffer: Vec<u16>,

    // Cached calculations
    frequency_bins: Vec<f32>,
    bar_positions: Vec<f32>,

    // Previous frame data for interpolation
    previous_magnitudes: Vec<f32>,

    // Configuration
    bar_count: usize,
    smoothing_factor: f32,
}

impl OptimizedSpectrumRenderer {
    pub fn render(&mut self, spectrum_data: &SpectrumData, canvas: &mut Canvas) -> Result<()> {
        // Interpolate with previous frame for smooth animation
        self.interpolate_magnitudes(&spectrum_data.magnitudes);

        // Update vertex buffer in-place (no allocation)
        self.update_vertices();

        // Batch render all bars
        canvas.draw_triangles(&self.vertex_buffer, &self.index_buffer)?;

        Ok(())
    }

    #[inline]
    fn interpolate_magnitudes(&mut self, new_magnitudes: &[f32]) {
        for (prev, &new) in self.previous_magnitudes.iter_mut().zip(new_magnitudes.iter()) {
            *prev += (new - *prev) * self.smoothing_factor;
        }
    }

    #[inline]
    fn update_vertices(&mut self) {
        // Update vertex positions based on magnitude data
        // Minimize branching in the loop
        for (i, &magnitude) in self.previous_magnitudes.iter().enumerate() {
            let bar_height = magnitude * self.bar_height_scale;
            let x = self.bar_positions[i];

            // Update 4 vertices per bar (2 triangles)
            let base_idx = i * 4;
            self.vertex_buffer[base_idx].position.y = 0.0;
            self.vertex_buffer[base_idx + 1].position.y = bar_height;
            self.vertex_buffer[base_idx + 2].position.y = bar_height;
            self.vertex_buffer[base_idx + 3].position.y = 0.0;
        }
    }
}
```

### Cache-Friendly Data Layout

```rust
// ✅ Structure data for cache efficiency
#[repr(C)]
#[derive(Copy, Clone)]
pub struct CacheOptimizedVertex {
    pub position: [f32; 2],    // Hot data - accessed every frame
    pub color: [f32; 4],       // Hot data - accessed every frame
    pub uv: [f32; 2],         // Cold data - accessed less frequently
}

// Separate hot and cold data
pub struct SpectrumVisualizerData {
    // Hot data - accessed every frame (cache-friendly layout)
    pub bar_heights: Vec<f32>,     // Current heights
    pub target_heights: Vec<f32>,  // Target heights for animation
    pub positions: Vec<f32>,       // X positions

    // Cold data - accessed rarely
    pub config: VisualizationConfig,
    pub metadata: PluginMetadata,
}
```

## Performance Profiling Integration

### Built-in Profiling

```rust
// ✅ Integrated performance profiling
use std::time::{Duration, Instant};

pub struct ProfiledFunction {
    name: &'static str,
    call_count: AtomicU64,
    total_time: AtomicU64,
    max_time: AtomicU64,
}

impl ProfiledFunction {
    pub fn profile<F, R>(&self, f: F) -> R
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = f();
        let elapsed = start.elapsed();

        self.call_count.fetch_add(1, Ordering::Relaxed);
        self.total_time.fetch_add(elapsed.as_nanos() as u64, Ordering::Relaxed);

        // Update max time if necessary
        let elapsed_nanos = elapsed.as_nanos() as u64;
        self.max_time.fetch_max(elapsed_nanos, Ordering::Relaxed);

        result
    }

    pub fn report(&self) -> ProfileReport {
        let call_count = self.call_count.load(Ordering::Relaxed);
        let total_nanos = self.total_time.load(Ordering::Relaxed);
        let max_nanos = self.max_time.load(Ordering::Relaxed);

        ProfileReport {
            name: self.name,
            call_count,
            average_time: Duration::from_nanos(total_nanos / call_count.max(1)),
            max_time: Duration::from_nanos(max_nanos),
            total_time: Duration::from_nanos(total_nanos),
        }
    }
}

// Macro for easy profiling
macro_rules! profile_function {
    ($profiler:expr, $body:expr) => {{
        static PROFILER: ProfiledFunction = ProfiledFunction::new(concat!(file!(), ":", line!()));
        PROFILER.profile(|| $body)
    }};
}
```

## Benchmark Integration

### Performance Regression Testing

```rust
// ✅ Benchmark critical paths
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn benchmark_spectrum_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("spectrum_analysis");

    for &size in [512, 1024, 2048, 4096].iter() {
        group.bench_with_input(
            BenchmarkId::new("fft_processing", size),
            &size,
            |b, &size| {
                let mut analyzer = SpectrumAnalyzer::new(size);
                let test_data: Vec<f32> = (0..size).map(|i| (i as f32).sin()).collect();

                b.iter(|| {
                    analyzer.analyze(black_box(&test_data))
                });
            },
        );
    }

    group.finish();
}

fn benchmark_visualizer_rendering(c: &mut Criterion) {
    let mut group = c.benchmark_group("visualizer_rendering");

    let spectrum_data = create_test_spectrum_data(1024);
    let mut renderer = SpectrumBarsVisualizer::new();
    let mut canvas = MockCanvas::new(1920, 1080);

    group.bench_function("spectrum_bars", |b| {
        b.iter(|| {
            renderer.render(black_box(&spectrum_data), black_box(&mut canvas))
        });
    });

    group.finish();
}

criterion_group!(benches, benchmark_spectrum_analysis, benchmark_visualizer_rendering);
criterion_main!(benches);
```
