# Sonic Flow - Testing Standards

## Testing Philosophy

### Coverage Requirements

- **Unit Tests**: 80%+ coverage for business logic
- **Integration Tests**: All module boundaries
- **Property-Based Tests**: Audio processing algorithms
- **Performance Tests**: Critical path benchmarking
- **UI Tests**: User interaction flows

### Test Categories

```rust
// ✅ Organize tests by category
#[cfg(test)]
mod unit_tests {
    // Fast, isolated tests for individual functions
}

#[cfg(test)]
mod integration_tests {
    // Tests that exercise multiple components together
}

#[cfg(test)]
mod property_tests {
    // Property-based tests using proptest
}

#[cfg(test)]
mod performance_tests {
    // Performance regression tests
}
```

## Unit Testing Patterns

### Audio Processing Tests

```rust
#[cfg(test)]
mod audio_tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_fft_analyzer_initialization() {
        let analyzer = SpectrumAnalyzer::new(1024);

        assert_eq!(analyzer.fft_size(), 1024);
        assert_eq!(analyzer.frequency_bins(), 512); // Nyquist limit
        assert!(analyzer.is_initialized());
    }

    #[test]
    fn test_spectrum_analysis_sine_wave() {
        let mut analyzer = SpectrumAnalyzer::new(1024);

        // Generate 440Hz sine wave
        let sample_rate = 44100.0;
        let frequency = 440.0;
        let samples: Vec<f32> = (0..1024)
            .map(|i| {
                let t = i as f32 / sample_rate;
                (2.0 * std::f32::consts::PI * frequency * t).sin()
            })
            .collect();

        let spectrum = analyzer.analyze(&samples).unwrap();

        // Find peak frequency
        let peak_bin = spectrum.magnitudes
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
            .unwrap().0;

        let detected_frequency = peak_bin as f32 * sample_rate / 1024.0;

        // Allow 5Hz tolerance
        assert_relative_eq!(detected_frequency, frequency, epsilon = 5.0);
    }

    #[test]
    fn test_audio_buffer_bounds() {
        let mut buffer = AudioBuffer::new(2, 1024, 44100);

        // Test channel access bounds
        assert!(buffer.channel(0).is_some());
        assert!(buffer.channel(1).is_some());
        assert!(buffer.channel(2).is_none());

        // Test sample access bounds
        let channel = buffer.channel_mut(0).unwrap();
        assert_eq!(channel.len(), 1024);

        // Test safe access
        channel[0] = 1.0;
        assert_eq!(channel[0], 1.0);
    }

    #[test]
    fn test_volume_application() {
        let mut samples = vec![0.5, -0.5, 1.0, -1.0];
        let gain = 0.5;

        apply_gain(&mut samples, gain);

        let expected = vec![0.25, -0.25, 0.5, -0.5];
        for (actual, expected) in samples.iter().zip(expected.iter()) {
            assert_relative_eq!(actual, expected, epsilon = 1e-6);
        }
    }
}
```

### Visualizer Testing

```rust
#[cfg(test)]
mod visualizer_tests {
    use super::*;
    use mockall::predicate::*;

    // Mock canvas for testing
    mockall::mock! {
        pub Canvas {
            fn draw_rectangle(&mut self, rect: Rect, color: Color) -> Result<()>;
            fn draw_line(&mut self, start: Point, end: Point, color: Color) -> Result<()>;
            fn clear(&mut self, color: Color) -> Result<()>;
            fn size(&self) -> (u32, u32);
        }
    }

    #[test]
    fn test_spectrum_bars_rendering() {
        let mut canvas = MockCanvas::new();
        let mut visualizer = SpectrumBarsVisualizer::new();

        // Set up expectations
        canvas.expect_clear()
            .times(1)
            .with(eq(Color::BLACK))
            .returning(|_| Ok(()));

        canvas.expect_draw_rectangle()
            .times(64) // 64 bars
            .returning(|_, _| Ok(()));

        canvas.expect_size()
            .returning(|| (800, 600));

        // Create test spectrum data
        let spectrum_data = SpectrumData {
            frequencies: (0..64).map(|i| i as f32 * 43.0).collect(),
            magnitudes: vec![0.5; 64],
            sample_rate: 44100.0,
            timestamp: std::time::Instant::now(),
        };

        // Test rendering
        let result = visualizer.render(&spectrum_data, &mut canvas);
        assert!(result.is_ok());
    }

    #[test]
    fn test_visualizer_configuration() {
        let mut visualizer = SpectrumBarsVisualizer::new();

        let mut config = VisualizationConfig::default();
        config.sensitivity = 2.0;
        config.bar_count = 32;

        visualizer.configure(&config).unwrap();

        assert_eq!(visualizer.sensitivity(), 2.0);
        assert_eq!(visualizer.bar_count(), 32);
    }

    #[test]
    fn test_color_interpolation() {
        let start_color = Color::rgb(255, 0, 0); // Red
        let end_color = Color::rgb(0, 255, 0);   // Green

        let mid_color = interpolate_color(start_color, end_color, 0.5);

        assert_eq!(mid_color.red(), 127);
        assert_eq!(mid_color.green(), 127);
        assert_eq!(mid_color.blue(), 0);
    }
}
```

## Property-Based Testing

### Audio Algorithm Properties

```rust
#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_gain_preserves_signal_properties(
            gain in 0.0f32..2.0,
            samples in prop::collection::vec(-1.0f32..1.0, 1..1024)
        ) {
            let mut processed = samples.clone();
            apply_gain(&mut processed, gain);

            // Properties that should hold:

            // 1. Length preservation
            prop_assert_eq!(processed.len(), samples.len());

            // 2. Silence preservation (zero input -> zero output)
            if samples.iter().all(|&x| x == 0.0) {
                prop_assert!(processed.iter().all(|&x| x == 0.0));
            }

            // 3. Linear scaling
            for (original, &processed_sample) in samples.iter().zip(processed.iter()) {
                prop_assert_relative_eq!(processed_sample, original * gain, epsilon = 1e-6);
            }

            // 4. Range bounds (assuming input is normalized)
            prop_assert!(processed.iter().all(|&x| x.abs() <= 2.0));
        }

        #[test]
        fn test_fft_properties(
            samples in prop::collection::vec(-1.0f32..1.0, 512..=2048)
        ) {
            let mut analyzer = SpectrumAnalyzer::new(samples.len());
            let spectrum = analyzer.analyze(&samples)?;

            // Properties of FFT:

            // 1. Frequency bins count (Nyquist)
            prop_assert_eq!(spectrum.frequencies.len(), samples.len() / 2);
            prop_assert_eq!(spectrum.magnitudes.len(), samples.len() / 2);

            // 2. Non-negative magnitudes
            prop_assert!(spectrum.magnitudes.iter().all(|&x| x >= 0.0));

            // 3. Finite values
            prop_assert!(spectrum.magnitudes.iter().all(|&x| x.is_finite()));

            // 4. DC component is at index 0
            prop_assert_eq!(spectrum.frequencies[0], 0.0);
        }

        #[test]
        fn test_smoothing_properties(
            current in prop::collection::vec(0.0f32..1.0, 64),
            target in prop::collection::vec(0.0f32..1.0, 64),
            factor in 0.0f32..1.0
        ) {
            prop_assume!(current.len() == target.len());

            let mut smoothed = current.clone();
            apply_smoothing(&mut smoothed, &target, factor);

            // Smoothing properties:

            // 1. Values move toward target
            for ((&curr, &targ), &smooth) in current.iter().zip(target.iter()).zip(smoothed.iter()) {
                if curr < targ {
                    prop_assert!(smooth >= curr && smooth <= targ);
                } else if curr > targ {
                    prop_assert!(smooth <= curr && smooth >= targ);
                } else {
                    prop_assert_eq!(smooth, curr);
                }
            }

            // 2. Factor of 0 should preserve current values
            if factor == 0.0 {
                prop_assert_eq!(smoothed, current);
            }

            // 3. Factor of 1 should equal target values
            if factor == 1.0 {
                for (&smooth, &targ) in smoothed.iter().zip(target.iter()) {
                    prop_assert_relative_eq!(smooth, targ, epsilon = 1e-6);
                }
            }
        }
    }
}
```

## Integration Testing

### Audio Pipeline Integration

```rust
// tests/integration/audio_pipeline_test.rs
use sonic_flow::prelude::*;
use tokio::time::{timeout, Duration};

#[tokio::test]
async fn test_complete_audio_pipeline() -> Result<()> {
    // Set up test environment
    let temp_dir = tempfile::tempdir()?;
    let test_file = temp_dir.path().join("test.wav");
    create_test_audio_file(&test_file, 440.0, Duration::from_secs(1))?;

    // Initialize audio system
    let config = AudioConfig::test_default();
    let mut audio_system = AudioSystem::new(config).await?;

    // Set up spectrum data receiver
    let mut spectrum_receiver = audio_system.spectrum_receiver();

    // Load and play test file
    audio_system.load_file(&test_file).await?;
    audio_system.play().await?;

    // Verify spectrum data is being produced
    let spectrum_data = timeout(Duration::from_millis(100), spectrum_receiver.recv())
        .await
        .map_err(|_| AudioError::Timeout("No spectrum data received".to_string()))??;

    // Validate spectrum data
    assert!(!spectrum_data.magnitudes.is_empty());
    assert_eq!(spectrum_data.sample_rate, 44100.0);

    // Find 440Hz peak
    let peak_frequency = find_peak_frequency(&spectrum_data);
    assert!((peak_frequency - 440.0).abs() < 10.0,
           "Expected 440Hz, got {}Hz", peak_frequency);

    // Clean shutdown
    audio_system.stop().await?;

    Ok(())
}

#[tokio::test]
async fn test_visualizer_integration() -> Result<()> {
    let mut visualizer_engine = VisualizerEngine::new().await?;

    // Load built-in visualizer
    visualizer_engine.load_visualizer("spectrum_bars").await?;
    visualizer_engine.activate_visualizer("spectrum_bars").await?;

    // Create test spectrum data
    let spectrum_data = create_test_spectrum_data(64);

    // Update visualizer
    visualizer_engine.update_spectrum(spectrum_data).await?;

    // Verify frame was rendered
    let frame_count_before = visualizer_engine.frame_count();
    visualizer_engine.render_frame().await?;
    assert_eq!(visualizer_engine.frame_count(), frame_count_before + 1);

    Ok(())
}

#[tokio::test]
async fn test_playlist_audio_integration() -> Result<()> {
    let temp_dir = tempfile::tempdir()?;

    // Create test playlist with multiple tracks
    let tracks = vec![
        create_test_track(&temp_dir, "track1.wav", 440.0)?,
        create_test_track(&temp_dir, "track2.wav", 880.0)?,
        create_test_track(&temp_dir, "track3.wav", 1320.0)?,
    ];

    let mut playlist_manager = PlaylistManager::new().await?;
    let playlist_id = playlist_manager.create_playlist("Test Playlist").await?;

    for track in tracks {
        playlist_manager.add_track(playlist_id, track).await?;
    }

    // Initialize audio system with playlist
    let mut audio_system = AudioSystem::new(AudioConfig::test_default()).await?;
    audio_system.load_playlist(playlist_id).await?;

    // Test sequential playback
    audio_system.play().await?;
    let initial_track = audio_system.current_track().await;

    audio_system.next_track().await?;
    let next_track = audio_system.current_track().await;

    assert_ne!(initial_track, next_track);

    Ok(())
}
```

### Database Integration Tests

```rust
// tests/integration/database_test.rs
use sonic_flow::storage::*;
use sqlx::sqlite::SqlitePoolOptions;

#[tokio::test]
async fn test_track_repository_operations() -> Result<()> {
    // Set up in-memory database
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await?;

    // Run migrations
    sqlx::migrate!("./migrations").run(&pool).await?;

    let repository = SqliteTrackRepository::new(pool);

    // Test track insertion
    let track = TrackInfo {
        id: TrackId::new(),
        title: Some("Test Track".to_string()),
        artist: Some("Test Artist".to_string()),
        album: Some("Test Album".to_string()),
        duration: Duration::from_secs(180),
        file_path: PathBuf::from("/test/path.mp3"),
        sample_rate: 44100,
        channels: 2,
        ..Default::default()
    };

    repository.save(&track).await?;

    // Test retrieval
    let retrieved = repository.find_by_id(track.id).await?;
    assert_eq!(retrieved.unwrap().title, track.title);

    // Test search
    let search_criteria = SearchCriteria::new()
        .artist("Test Artist");

    let results = repository.find_by_criteria(&search_criteria).await?;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, track.id);

    // Test deletion
    repository.delete(track.id).await?;
    let deleted = repository.find_by_id(track.id).await?;
    assert!(deleted.is_none());

    Ok(())
}

#[tokio::test]
async fn test_playlist_persistence() -> Result<()> {
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let playlist_repo = SqlitePlaylistRepository::new(pool.clone());
    let track_repo = SqliteTrackRepository::new(pool);

    // Create test tracks
    let track1 = create_test_track_info("Track 1");
    let track2 = create_test_track_info("Track 2");

    track_repo.save(&track1).await?;
    track_repo.save(&track2).await?;

    // Create playlist
    let playlist = PlaylistInfo {
        id: PlaylistId::new(),
        name: "Test Playlist".to_string(),
        tracks: vec![track1.id, track2.id],
        ..Default::default()
    };

    playlist_repo.save(&playlist).await?;

    // Retrieve and verify
    let retrieved = playlist_repo.find_by_id(playlist.id).await?.unwrap();
    assert_eq!(retrieved.name, "Test Playlist");
    assert_eq!(retrieved.tracks.len(), 2);

    Ok(())
}
```

## Performance Testing

### Benchmark Integration

```rust
// benches/audio_performance.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use sonic_flow::audio::*;

fn benchmark_fft_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("fft_analysis");

    for &size in [512, 1024, 2048, 4096].iter() {
        group.bench_with_input(
            BenchmarkId::new("spectrum_analysis", size),
            &size,
            |b, &size| {
                let mut analyzer = SpectrumAnalyzer::new(size);
                let test_data = create_test_sine_wave(440.0, size);

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

    let spectrum_data = create_test_spectrum_data(64);
    let mut canvas = TestCanvas::new(1920, 1080);

    for visualizer_type in ["spectrum_bars", "waveform", "circle_spectrum"].iter() {
        group.bench_with_input(
            BenchmarkId::new("render", visualizer_type),
            visualizer_type,
            |b, &viz_type| {
                let mut visualizer = create_visualizer(viz_type);

                b.iter(|| {
                    visualizer.render(black_box(&spectrum_data), black_box(&mut canvas))
                });
            },
        );
    }

    group.finish();
}

fn benchmark_audio_processing_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("audio_pipeline");

    let buffer_sizes = [256, 512, 1024, 2048];

    for &buffer_size in buffer_sizes.iter() {
        group.bench_with_input(
            BenchmarkId::new("full_pipeline", buffer_size),
            &buffer_size,
            |b, &size| {
                let mut processor = AudioProcessor::new(size);
                let test_buffer = create_test_audio_buffer(size, 2);

                b.iter(|| {
                    processor.process_frame(black_box(&test_buffer))
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_fft_sizes,
    benchmark_visualizer_rendering,
    benchmark_audio_processing_pipeline
);
criterion_main!(benches);
```

### Performance Regression Tests

```rust
#[cfg(test)]
mod performance_tests {
    use super::*;
    use std::time::{Duration, Instant};

    #[test]
    fn test_fft_performance_requirement() {
        let mut analyzer = SpectrumAnalyzer::new(2048);
        let test_data = create_test_sine_wave(440.0, 2048);

        // Warm up
        for _ in 0..10 {
            let _ = analyzer.analyze(&test_data);
        }

        // Measure performance
        let start = Instant::now();
        let iterations = 1000;

        for _ in 0..iterations {
            let _ = analyzer.analyze(&test_data);
        }

        let elapsed = start.elapsed();
        let avg_time = elapsed / iterations;

        // Requirement: FFT analysis should complete in under 1ms
        assert!(
            avg_time < Duration::from_millis(1),
            "FFT analysis took {:?}, requirement is < 1ms",
            avg_time
        );
    }

    #[test]
    fn test_visualizer_frame_rate() {
        let mut visualizer = SpectrumBarsVisualizer::new();
        let spectrum_data = create_test_spectrum_data(64);
        let mut canvas = TestCanvas::new(1920, 1080);

        // Measure rendering time
        let start = Instant::now();
        let frames = 120; // 120 frames for 120fps test

        for _ in 0..frames {
            visualizer.render(&spectrum_data, &mut canvas).unwrap();
        }

        let elapsed = start.elapsed();
        let avg_frame_time = elapsed / frames;

        // Requirement: Each frame should render in under 8.33ms (120fps)
        assert!(
            avg_frame_time < Duration::from_micros(8333),
            "Frame rendering took {:?}, requirement is < 8.33ms",
            avg_frame_time
        );
    }

    #[test]
    fn test_memory_usage_bounds() {
        let initial_memory = get_process_memory_usage();

        // Create audio system with typical configuration
        let audio_system = AudioSystem::new(AudioConfig::default()).unwrap();
        let visualizer_engine = VisualizerEngine::new().unwrap();

        // Load test data
        let spectrum_data = create_test_spectrum_data(2048);

        // Process several frames
        for _ in 0..60 {
            visualizer_engine.update_spectrum(spectrum_data.clone());
        }

        let final_memory = get_process_memory_usage();
        let memory_increase = final_memory - initial_memory;

        // Requirement: Memory increase should be under 100MB
        assert!(
            memory_increase < 100 * 1024 * 1024,
            "Memory usage increased by {} bytes, requirement is < 100MB",
            memory_increase
        );
    }
}
```

## UI Testing

### Slint Component Testing

```rust
#[cfg(test)]
mod ui_tests {
    use super::*;
    use slint::testing::*;

    #[test]
    fn test_player_controls_state_changes() {
        let ui = PlayerControls::new().unwrap();

        // Test initial state
        assert_eq!(ui.get_is_playing(), false);
        assert_eq!(ui.get_volume(), 1.0);
        assert_eq!(ui.get_progress(), 0.0);

        // Test play state change
        ui.set_is_playing(true);
        assert_eq!(ui.get_is_playing(), true);

        // Test volume bounds
        ui.set_volume(1.5);
        assert_eq!(ui.get_volume(), 1.0); // Clamped to maximum

        ui.set_volume(-0.1);
        assert_eq!(ui.get_volume(), 0.0); // Clamped to minimum
    }

    #[test]
    fn test_visualizer_canvas_interaction() {
        let ui = VisualizerCanvas::new().unwrap();
        let test_data = create_test_spectrum_data_slint();

        ui.set_spectrum_data(test_data.clone());

        // Verify data binding
        assert_eq!(ui.get_spectrum_data().magnitudes.row_count(), test_data.magnitudes.row_count());

        // Test interaction callbacks
        let mut callback_triggered = false;
        ui.on_sensitivity_changed({
            let mut callback_triggered = callback_triggered;
            move |value| {
                callback_triggered = true;
                assert!(value >= 0.1 && value <= 5.0);
            }
        });

        // Simulate sensitivity change
        ui.invoke_sensitivity_changed(2.0);
        // Note: In real tests, verify callback was triggered through test framework
    }
}
```

## Test Utilities and Helpers

### Test Data Generation

```rust
// tests/common/mod.rs
pub mod test_data {
    use super::*;

    pub fn create_test_sine_wave(frequency: f32, length: usize) -> Vec<f32> {
        let sample_rate = 44100.0;
        (0..length)
            .map(|i| {
                let t = i as f32 / sample_rate;
                (2.0 * std::f32::consts::PI * frequency * t).sin()
            })
            .collect()
    }

    pub fn create_test_spectrum_data(bin_count: usize) -> SpectrumData {
        let frequencies: Vec<f32> = (0..bin_count)
            .map(|i| i as f32 * 43.066)
            .collect();

        let magnitudes: Vec<f32> = (0..bin_count)
            .map(|i| {
                // Create realistic spectrum with decreasing amplitude
                let normalized_freq = i as f32 / bin_count as f32;
                (1.0 - normalized_freq) * 0.8 + 0.1
            })
            .collect();

        SpectrumData {
            frequencies,
            magnitudes,
            sample_rate: 44100.0,
            timestamp: std::time::Instant::now(),
        }
    }

    pub fn create_test_audio_file(path: &Path, frequency: f32, duration: Duration) -> Result<()> {
        let sample_rate = 44100u32;
        let samples = (duration.as_secs_f32() * sample_rate as f32) as usize;

        let audio_data = create_test_sine_wave(frequency, samples);

        // Write WAV file
        let spec = hound::WavSpec {
            channels: 1,
            sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let mut writer = hound::WavWriter::create(path, spec)?;

        for sample in audio_data {
            let amplitude = (sample * i16::MAX as f32) as i16;
            writer.write_sample(amplitude)?;
        }

        writer.finalize()?;
        Ok(())
    }

    pub fn create_test_track_info(title: &str) -> TrackInfo {
        TrackInfo {
            id: TrackId::new(),
            title: Some(title.to_string()),
            artist: Some("Test Artist".to_string()),
            album: Some("Test Album".to_string()),
            duration: Duration::from_secs(180),
            file_path: PathBuf::from(format!("/test/{}.mp3", title)),
            sample_rate: 44100,
            channels: 2,
            bitrate: 320,
            file_size: 5_000_000,
            created_at: chrono::Utc::now(),
            modified_at: chrono::Utc::now(),
            ..Default::default()
        }
    }
}

pub mod mock_objects {
    use super::*;
    use mockall::mock;

    mock! {
        pub AudioRenderer {
            fn initialize(&mut self, config: &AudioConfig) -> Result<()>;
            fn render(&mut self, buffer: &AudioBuffer) -> Result<()>;
            fn set_volume(&mut self, volume: f32) -> Result<()>;
            fn is_playing(&self) -> bool;
        }
    }

    mock! {
        pub VisualizerPlugin {
            fn metadata(&self) -> PluginMetadata;
            fn render(&mut self, data: &SpectrumData, canvas: &mut dyn Canvas) -> Result<()>;
            fn configure(&mut self, config: &VisualizationConfig) -> Result<()>;
        }
    }
}

pub mod assertions {
    use super::*;

    /// Assert that audio data is within valid range
    pub fn assert_audio_range(samples: &[f32]) {
        for (i, &sample) in samples.iter().enumerate() {
            assert!(
                sample >= -1.0 && sample <= 1.0,
                "Sample {} out of range: {} (expected -1.0 to 1.0)",
                i, sample
            );
        }
    }

    /// Assert that spectrum data is valid
    pub fn assert_spectrum_valid(spectrum: &SpectrumData) {
        assert!(!spectrum.frequencies.is_empty(), "Frequencies cannot be empty");
        assert!(!spectrum.magnitudes.is_empty(), "Magnitudes cannot be empty");
        assert_eq!(spectrum.frequencies.len(), spectrum.magnitudes.len(),
                  "Frequencies and magnitudes must have same length");

        // All magnitudes should be non-negative and finite
        for (i, &magnitude) in spectrum.magnitudes.iter().enumerate() {
            assert!(
                magnitude >= 0.0 && magnitude.is_finite(),
                "Invalid magnitude at index {}: {}",
                i, magnitude
            );
        }

        // Frequencies should be ascending
        for window in spectrum.frequencies.windows(2) {
            assert!(
                window[0] <= window[1],
                "Frequencies must be in ascending order"
            );
        }
    }

    /// Assert performance timing requirements
    pub fn assert_performance_timing<F>(operation: F, max_duration: Duration, operation_name: &str)
    where
        F: FnOnce(),
    {
        let start = std::time::Instant::now();
        operation();
        let elapsed = start.elapsed();

        assert!(
            elapsed <= max_duration,
            "{} took {:?}, maximum allowed is {:?}",
            operation_name, elapsed, max_duration
        );
    }
}
```

## Test Configuration

### Test Environment Setup

```rust
// tests/common/setup.rs
use std::sync::Once;

static INIT: Once = Once::new();

pub fn setup_test_environment() {
    INIT.call_once(|| {
        // Initialize logging for tests
        env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("debug"))
            .is_test(true)
            .init();

        // Set test-specific environment variables
        std::env::set_var("SONIC_FLOW_TEST_MODE", "1");
        std::env::set_var("SONIC_FLOW_LOG_LEVEL", "debug");
    });
}

/// Test configuration for integration tests
pub fn test_audio_config() -> AudioConfig {
    AudioConfig {
        sample_rate: 44100,
        buffer_size: 512,
        channels: 2,
        device_name: "test".to_string(),
        ..Default::default()
    }
}

/// Create temporary directory for test files
pub fn create_test_directory() -> tempfile::TempDir {
    tempfile::tempdir().expect("Failed to create test directory")
}
```

## Testing Best Practices

### Test Organization

- Group related tests in modules
- Use descriptive test names that explain what is being tested
- Keep tests focused on single behaviors
- Use property-based testing for algorithm validation
- Include both positive and negative test cases

### Performance Testing

- Establish baseline performance requirements
- Use criterion for micro-benchmarks
- Test performance under various conditions
- Monitor for performance regressions in CI
- Include memory usage testing

### Mock and Stub Usage

- Mock external dependencies (audio devices, file system)
- Use dependency injection for testability
- Prefer fakes over mocks when possible
- Ensure mocks accurately represent real behavior

### Test Data Management

- Create realistic test data
- Use property-based testing for edge cases
- Include boundary condition testing
- Test with various audio formats and configurations
