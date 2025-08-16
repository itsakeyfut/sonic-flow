# Sonic Flow - Rust Code Standards

## Core Rust Guidelines

### Error Handling (Mandatory)

```rust
// ✅ Use Result types consistently
pub fn load_audio_file(path: &Path) -> Result<AudioBuffer, AudioError> {
    let file = std::fs::File::open(path)
        .map_err(|e| AudioError::FileRead(format!("Failed to open {}: {}", path.display(), e)))?;
    // ... processing
    Ok(buffer)
}

// ❌ NEVER use unwrap() in production code
let file = std::fs::File::open(path).unwrap(); // FORBIDDEN

// ✅ Use expect() only for truly impossible cases with explanation
let config = CONFIG.get().expect("Config must be initialized at startup");
```

### Error Type Structure

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AudioError {
    #[error("File reading error: {0}")]
    FileRead(String),

    #[error("Decoding error: {0}")]
    Decoding(#[from] DecodingError),

    #[error("Device error: {0}")]
    Device(#[from] DeviceError),

    #[error("Invalid audio format: expected {expected}, got {actual}")]
    InvalidFormat { expected: String, actual: String },
}

// Module-level Result alias
pub type Result<T> = std::result::Result<T, AudioError>;
```

### Async Patterns

```rust
// ✅ Use tokio consistently
use tokio::sync::{mpsc, oneshot, RwLock};
use std::sync::Arc;

// ✅ Structured concurrency with JoinSet
use tokio::task::JoinSet;

pub async fn process_audio_pipeline(&self) -> Result<()> {
    let mut tasks = JoinSet::new();

    tasks.spawn(self.audio_decoder_task());
    tasks.spawn(self.visualizer_update_task());
    tasks.spawn(self.ui_update_task());

    // Wait for any task to complete/fail
    while let Some(result) = tasks.join_next().await {
        result??; // Handle both join error and task error
    }

    Ok(())
}

// ✅ Prefer channels over shared state
pub struct AudioPipeline {
    spectrum_tx: mpsc::UnboundedSender<SpectrumData>,
    control_rx: mpsc::Receiver<AudioCommand>,
}
```

### Memory Management

```rust
// ✅ Use Arc for shared ownership, Rc for single-thread
use std::sync::Arc;

#[derive(Clone)]
pub struct SharedAudioConfig {
    inner: Arc<AudioConfigInner>,
}

// ✅ Use Box for heap allocation when needed
pub trait AudioProcessor: Send + Sync {
    fn process(&mut self, input: &[f32], output: &mut [f32]) -> Result<()>;
}

pub type BoxedProcessor = Box<dyn AudioProcessor>;

// ✅ Prefer borrowing over cloning
pub fn analyze_spectrum(data: &[f32]) -> SpectrumData {
    // Process borrowed data
}
```

### Performance-Critical Code

```rust
// ✅ Use inline for hot paths
#[inline]
pub fn mix_samples(left: &mut [f32], right: &[f32], volume: f32) {
    for (l, &r) in left.iter_mut().zip(right.iter()) {
        *l += r * volume;
    }
}

// ✅ Use SIMD when beneficial (with fallback)
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

pub fn apply_gain_simd(samples: &mut [f32], gain: f32) {
    #[cfg(target_arch = "x86_64")]
    if is_x86_feature_detected!("avx2") {
        return unsafe { apply_gain_avx2(samples, gain) };
    }

    // Fallback implementation
    for sample in samples {
        *sample *= gain;
    }
}

// ✅ Minimize allocations in audio thread
pub struct AudioBuffer {
    samples: Vec<f32>,
    channels: u8,
    sample_rate: u32,
}

impl AudioBuffer {
    // Reuse buffer instead of allocating
    pub fn clear(&mut self) {
        self.samples.clear(); // Don't deallocate capacity
    }
}
```

### Module Organization

```rust
// ✅ Clear module structure with re-exports
// src/audio/mod.rs
pub mod engine;
pub mod decoder;
pub mod effects;
pub mod analysis;

// Public API - only export what's needed
pub use engine::{AudioEngine, AudioEngineBuilder};
pub use decoder::{AudioDecoder, DecodingError};
pub use analysis::{SpectrumAnalyzer, SpectrumData};

// Internal modules remain private
mod buffer;
mod utils;

// ✅ Prelude module for common imports
pub mod prelude {
    pub use super::{AudioEngine, SpectrumAnalyzer};
    pub use super::decoder::{AudioDecoder, AudioFormat};
    pub use super::effects::EffectsChain;
}
```

### Plugin System Patterns

```rust
// ✅ Trait-based plugin system
pub trait VisualizerPlugin: Send + Sync {
    fn metadata(&self) -> PluginMetadata;
    fn initialize(&mut self, config: &VisualizationConfig) -> Result<()>;
    fn render(&mut self, data: &SpectrumData, canvas: &mut Canvas) -> Result<()>;
    fn configure(&mut self, settings: &HashMap<String, PluginValue>) -> Result<()>;

    // Default implementations for optional methods
    fn cleanup(&mut self) {}
    fn supports_real_time(&self) -> bool { true }
}

// ✅ Registration macro for plugins
#[macro_export]
macro_rules! register_visualizer {
    ($plugin_type:ty) => {
        #[no_mangle]
        pub extern "C" fn create_visualizer() -> Box<dyn VisualizerPlugin> {
            Box::new(<$plugin_type>::new())
        }

        #[no_mangle]
        pub extern "C" fn get_metadata() -> PluginMetadata {
            <$plugin_type>::metadata()
        }
    };
}
```

### Testing Patterns

```rust
// ✅ Comprehensive test structure
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;

    #[test]
    fn test_audio_engine_initialization() {
        let config = AudioConfig::test_default();
        let engine = AudioEngine::new(config).unwrap();
        assert_eq!(engine.state(), EngineState::Initialized);
    }

    #[tokio::test]
    async fn test_spectrum_analysis() {
        let mut analyzer = SpectrumAnalyzer::new(1024);
        let test_data = create_test_sine_wave(440.0, 1024);

        let spectrum = analyzer.analyze(&test_data).unwrap();

        // Verify 440Hz peak
        let peak_bin = spectrum.find_peak_frequency();
        assert!((peak_bin - 440.0).abs() < 5.0);
    }

    // ✅ Property-based testing for audio
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_gain_application(
            gain in 0.0f32..2.0,
            samples in prop::collection::vec(-1.0f32..1.0, 1..1024)
        ) {
            let mut buffer = samples.clone();
            apply_gain(&mut buffer, gain);

            // Verify all samples are in valid range
            prop_assert!(buffer.iter().all(|&x| x.abs() <= 2.0));
        }
    }
}

// ✅ Test utilities module
#[cfg(test)]
pub mod test_utils {
    use super::*;

    pub fn create_test_sine_wave(frequency: f32, length: usize) -> Vec<f32> {
        (0..length)
            .map(|i| (2.0 * std::f32::consts::PI * frequency * i as f32 / 44100.0).sin())
            .collect()
    }

    pub fn create_mock_spectrum_data() -> SpectrumData {
        SpectrumData {
            frequencies: (0..512).map(|i| i as f32 * 43.0).collect(),
            magnitudes: vec![0.5; 512],
            sample_rate: 44100.0,
            timestamp: std::time::Instant::now(),
        }
    }
}
```

### Documentation Standards

````rust
//! Audio processing module
//!
//! This module provides high-performance audio processing capabilities
//! including real-time decoding, effects processing, and spectrum analysis.
//!
//! # Examples
//!
//! ```rust
//! use sonic_flow::audio::AudioEngine;
//!
//! let engine = AudioEngine::builder()
//!     .sample_rate(44100)
//!     .buffer_size(512)
//!     .build()?;
//! ```

/// Real-time spectrum analyzer using FFT
///
/// Provides frequency domain analysis of audio signals with configurable
/// window functions and overlap ratios for optimal time-frequency resolution.
///
/// # Performance
///
/// - Optimized for real-time processing (< 1ms latency)
/// - SIMD acceleration when available
/// - Lock-free operation for audio thread safety
///
/// # Examples
///
/// ```rust
/// let mut analyzer = SpectrumAnalyzer::new(2048);
/// let spectrum = analyzer.analyze(&audio_samples)?;
/// println!("Peak frequency: {} Hz", spectrum.peak_frequency());
/// ```
pub struct SpectrumAnalyzer {
    // ... fields
}
````

## Code Quality Enforcement

- **Always run**: `cargo fmt` before committing
- **Always run**: `cargo clippy -- -D warnings` before committing
- **Test coverage**: Aim for 80%+ with `cargo tarpaulin`
- **No unsafe code**: Unless absolutely necessary and well-documented
- **Prefer iterators**: Over manual loops for better performance and clarity
