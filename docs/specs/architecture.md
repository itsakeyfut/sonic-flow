# Sonic Flow - Architecture Patterns

## Layered Architecture

### Layer Responsibilities

```rust
// 1. UI Layer (Slint) - Presentation only
// src/ui/main_window.slint
export component MainWindow inherits Window {
    callback play_requested();
    callback track_selected(int);

    property<bool> is_playing;
    property<SpectrumData> spectrum_data;
}

// 2. Application Layer - Orchestration and state management
// src/app/controller.rs
pub struct AppController {
    state: Arc<RwLock<AppState>>,
    audio_engine: Arc<AudioEngine>,
    visualizer_engine: Arc<VisualizerEngine>,
    event_bus: EventBus,
}

// 3. Business Logic Layer - Domain logic
// src/audio/engine.rs
pub struct AudioEngine {
    decoder: Box<dyn AudioDecoder>,
    effects_chain: EffectsChain,
    spectrum_analyzer: SpectrumAnalyzer,
}

// 4. Infrastructure Layer - External systems
// src/storage/database.rs
pub struct DatabaseRepository {
    pool: SqlitePool,
}
```

### Dependency Flow

```rust
// ✅ Higher layers depend on abstractions, not implementations
use crate::audio::traits::AudioDecoder; // trait, not impl
use crate::storage::traits::Repository; // trait, not impl

// ✅ Dependency injection pattern
pub struct AudioEngine {
    decoder: Box<dyn AudioDecoder>,
    storage: Box<dyn TrackRepository>,
}

impl AudioEngine {
    pub fn new(
        decoder: Box<dyn AudioDecoder>,
        storage: Box<dyn TrackRepository>,
    ) -> Self {
        Self { decoder, storage }
    }
}
```

## Event-Driven Communication

### Event System

```rust
// Central event bus for loose coupling
use tokio::sync::broadcast;

#[derive(Debug, Clone)]
pub enum AppEvent {
    // Audio events
    TrackLoaded(TrackInfo),
    PlaybackStarted,
    PlaybackPaused,
    SpectrumUpdated(SpectrumData),

    // UI events
    TrackSelected(TrackId),
    VolumeChanged(f32),

    // System events
    ConfigChanged(ConfigSection),
    ErrorOccurred(AppError),
}

pub struct EventBus {
    sender: broadcast::Sender<AppEvent>,
}

impl EventBus {
    pub fn emit(&self, event: AppEvent) -> Result<()> {
        self.sender.send(event)
            .map(|_| ())
            .map_err(|_| AppError::EventBusFull)
    }

    pub fn subscribe(&self) -> broadcast::Receiver<AppEvent> {
        self.sender.subscribe()
    }
}
```

### Event Handler Pattern

```rust
// ✅ Type-safe event handlers
pub trait EventHandler<T>: Send + Sync {
    async fn handle(&self, event: T) -> Result<()>;
}

// Concrete handler implementation
pub struct VisualizerEventHandler {
    visualizer: Arc<VisualizerEngine>,
}

impl EventHandler<AppEvent> for VisualizerEventHandler {
    async fn handle(&self, event: AppEvent) -> Result<()> {
        match event {
            AppEvent::SpectrumUpdated(data) => {
                self.visualizer.update_spectrum(data).await
            }
            AppEvent::TrackLoaded(info) => {
                self.visualizer.reset_for_new_track(info).await
            }
            _ => Ok(()), // Ignore other events
        }
    }
}
```

## Plugin Architecture

### Plugin Trait Definition

```rust
// Core plugin interface
pub trait VisualizerPlugin: Send + Sync {
    /// Plugin metadata
    fn metadata(&self) -> PluginMetadata;

    /// Initialize plugin with configuration
    fn initialize(&mut self, config: &VisualizationConfig) -> Result<()>;

    /// Render visualization frame
    fn render(&mut self, spectrum: &SpectrumData, canvas: &mut Canvas) -> Result<()>;

    /// Handle configuration changes
    fn configure(&mut self, settings: &HashMap<String, PluginValue>) -> Result<()>;

    /// Cleanup resources
    fn cleanup(&mut self) {}
}

// Plugin metadata
#[derive(Debug, Clone)]
pub struct PluginMetadata {
    pub id: String,
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub category: PluginCategory,
}
```

### Plugin Manager

```rust
pub struct PluginManager {
    loaded_plugins: HashMap<String, LoadedPlugin>,
    active_visualizers: Vec<String>,
}

struct LoadedPlugin {
    plugin: Box<dyn VisualizerPlugin>,
    metadata: PluginMetadata,
}

impl PluginManager {
    pub fn load_plugin(&mut self, plugin: Box<dyn VisualizerPlugin>) -> Result<String> {
        let metadata = plugin.metadata();
        let id = metadata.id.clone();

        // Validate plugin
        self.validate_plugin(&*plugin)?;

        // Store plugin
        self.loaded_plugins.insert(id.clone(), LoadedPlugin {
            plugin,
            metadata,
        });

        Ok(id)
    }

    pub fn activate_visualizer(&mut self, id: &str) -> Result<()> {
        if !self.loaded_plugins.contains_key(id) {
            return Err(PluginError::NotFound(id.to_string()));
        }

        self.active_visualizers.clear();
        self.active_visualizers.push(id.to_string());

        Ok(())
    }
}
```

## State Management

### Application State

```rust
// Centralized application state
#[derive(Debug, Clone)]
pub struct AppState {
    pub current_track: Option<TrackInfo>,
    pub playback_state: PlaybackState,
    pub volume: f32,
    pub current_playlist: Option<PlaylistId>,
    pub visualizer_config: VisualizationConfig,
    pub library_stats: LibraryStats,
}

// State manager with concurrent access
pub struct StateManager {
    state: Arc<RwLock<AppState>>,
    subscribers: Arc<Mutex<Vec<StateSubscriber>>>,
}

impl StateManager {
    pub async fn update<F>(&self, updater: F) -> Result<()>
    where
        F: FnOnce(&mut AppState) -> Result<()>,
    {
        let mut state = self.state.write().await;
        updater(&mut *state)?;

        // Notify subscribers
        self.notify_subscribers(&*state).await;

        Ok(())
    }

    pub async fn get<F, R>(&self, accessor: F) -> R
    where
        F: FnOnce(&AppState) -> R,
    {
        let state = self.state.read().await;
        accessor(&*state)
    }
}
```

### Configuration Management

```rust
// Hierarchical configuration system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub audio: AudioConfig,
    pub visualizer: VisualizationConfig,
    pub ui: UiConfig,
    pub plugins: PluginConfig,
}

impl AppConfig {
    // Load with environment variable overrides
    pub fn load() -> Result<Self> {
        let mut config = config::Config::builder()
            .add_source(config::File::with_name("config/default.toml"))
            .add_source(config::Environment::with_prefix("SONIC_FLOW"))
            .build()?;

        config.try_deserialize()
    }

    // Validate configuration
    pub fn validate(&self) -> Result<()> {
        self.audio.validate()?;
        self.visualizer.validate()?;
        Ok(())
    }
}
```

## Async Patterns

### Structured Concurrency

```rust
// ✅ Use JoinSet for managing multiple tasks
pub struct AudioSystem {
    tasks: JoinSet<Result<()>>,
    shutdown: CancellationToken,
}

impl AudioSystem {
    pub async fn start(&mut self) -> Result<()> {
        // Start audio processing task
        self.tasks.spawn({
            let shutdown = self.shutdown.clone();
            async move {
                self.audio_processing_loop(shutdown).await
            }
        });

        // Start visualizer update task
        self.tasks.spawn({
            let shutdown = self.shutdown.clone();
            async move {
                self.visualizer_update_loop(shutdown).await
            }
        });

        // Wait for shutdown or task failure
        while let Some(result) = self.tasks.join_next().await {
            match result {
                Ok(Ok(())) => {}, // Task completed successfully
                Ok(Err(e)) => return Err(e), // Task failed
                Err(e) => return Err(AppError::TaskPanic(e.to_string())),
            }
        }

        Ok(())
    }

    pub fn shutdown(&self) {
        self.shutdown.cancel();
    }
}
```

### Channel-Based Communication

```rust
// ✅ Prefer channels over shared mutable state
pub struct AudioPipeline {
    // Control flow
    command_tx: mpsc::Sender<AudioCommand>,
    command_rx: mpsc::Receiver<AudioCommand>,

    // Data flow
    spectrum_tx: mpsc::UnboundedSender<SpectrumData>,
    spectrum_rx: mpsc::UnboundedReceiver<SpectrumData>,

    // Shutdown coordination
    shutdown: CancellationToken,
}

#[derive(Debug)]
pub enum AudioCommand {
    Play(TrackInfo),
    Pause,
    Stop,
    SetVolume(f32),
    Seek(Duration),
}
```

## Performance Patterns

### Lock-Free Data Structures

```rust
use crossbeam::atomic::AtomicCell;
use crossbeam::queue::SegQueue;

// Triple buffering for real-time audio
pub struct TripleBuffer<T> {
    buffers: [AtomicCell<Option<T>>; 3],
    write_index: AtomicUsize,
    read_index: AtomicUsize,
}

// Lock-free queue for spectrum data
pub struct SpectrumQueue {
    queue: SegQueue<SpectrumData>,
    max_size: usize,
}

impl SpectrumQueue {
    pub fn push(&self, data: SpectrumData) {
        if self.queue.len() >= self.max_size {
            let _ = self.queue.pop(); // Drop oldest data
        }
        self.queue.push(data);
    }

    pub fn pop(&self) -> Option<SpectrumData> {
        self.queue.pop()
    }
}
```

### Memory Pool Pattern

```rust
// Reuse audio buffers to avoid allocations
pub struct AudioBufferPool {
    small_buffers: SegQueue<Vec<f32>>,  // 512 samples
    large_buffers: SegQueue<Vec<f32>>,  // 2048 samples
}

impl AudioBufferPool {
    pub fn acquire(&self, size: usize) -> PooledBuffer {
        let buffer = if size <= 512 {
            self.small_buffers.pop().unwrap_or_else(|| vec![0.0; 512])
        } else {
            self.large_buffers.pop().unwrap_or_else(|| vec![0.0; 2048])
        };

        PooledBuffer::new(buffer, self)
    }
}

// RAII buffer that returns to pool on drop
pub struct PooledBuffer {
    buffer: Option<Vec<f32>>,
    pool: *const AudioBufferPool,
}

impl Drop for PooledBuffer {
    fn drop(&mut self) {
        if let Some(buffer) = self.buffer.take() {
            unsafe {
                if buffer.capacity() <= 512 {
                    (*self.pool).small_buffers.push(buffer);
                } else {
                    (*self.pool).large_buffers.push(buffer);
                }
            }
        }
    }
}
```

## Repository Pattern

### Data Access Abstraction

```rust
// Abstract repository trait
#[async_trait]
pub trait TrackRepository: Send + Sync {
    async fn find_by_id(&self, id: TrackId) -> Result<Option<TrackInfo>>;
    async fn find_by_criteria(&self, criteria: &SearchCriteria) -> Result<Vec<TrackInfo>>;
    async fn save(&self, track: &TrackInfo) -> Result<()>;
    async fn delete(&self, id: TrackId) -> Result<()>;
    async fn count(&self) -> Result<usize>;
}

// SQLite implementation
pub struct SqliteTrackRepository {
    pool: Arc<SqlitePool>,
}

#[async_trait]
impl TrackRepository for SqliteTrackRepository {
    async fn find_by_id(&self, id: TrackId) -> Result<Option<TrackInfo>> {
        let row = sqlx::query_as!(
            TrackRow,
            "SELECT * FROM tracks WHERE id = ?",
            id
        )
        .fetch_optional(&*self.pool)
        .await?;

        Ok(row.map(TrackInfo::from))
    }

    async fn save(&self, track: &TrackInfo) -> Result<()> {
        sqlx::query!(
            "INSERT OR REPLACE INTO tracks (id, title, artist, album, duration_ms, file_path)
             VALUES (?, ?, ?, ?, ?, ?)",
            track.id,
            track.title,
            track.artist,
            track.album,
            track.duration.as_millis() as i64,
            track.file_path.to_string_lossy()
        )
        .execute(&*self.pool)
        .await?;

        Ok(())
    }
}
```

## Strategy Pattern

### Visualization Strategy

```rust
// Strategy for different visualization algorithms
pub trait VisualizationStrategy: Send + Sync {
    fn process_spectrum(&self, data: &SpectrumData) -> ProcessedSpectrum;
    fn get_parameters(&self) -> StrategyParameters;
}

// Linear frequency scaling
pub struct LinearFrequencyStrategy {
    frequency_range: (f32, f32),
    bin_count: usize,
}

impl VisualizationStrategy for LinearFrequencyStrategy {
    fn process_spectrum(&self, data: &SpectrumData) -> ProcessedSpectrum {
        // Linear frequency binning
        let bins = self.linear_binning(&data.magnitudes, &data.frequencies);
        ProcessedSpectrum { bins }
    }
}

// Logarithmic frequency scaling
pub struct LogarithmicFrequencyStrategy {
    octave_divisions: usize,
    base_frequency: f32,
}

impl VisualizationStrategy for LogarithmicFrequencyStrategy {
    fn process_spectrum(&self, data: &SpectrumData) -> ProcessedSpectrum {
        // Logarithmic frequency binning
        let bins = self.logarithmic_binning(&data.magnitudes, &data.frequencies);
        ProcessedSpectrum { bins }
    }
}

// Context using strategy
pub struct VisualizationProcessor {
    strategy: Box<dyn VisualizationStrategy>,
}

impl VisualizationProcessor {
    pub fn set_strategy(&mut self, strategy: Box<dyn VisualizationStrategy>) {
        self.strategy = strategy;
    }

    pub fn process(&self, data: &SpectrumData) -> ProcessedSpectrum {
        self.strategy.process_spectrum(data)
    }
}
```

## Observer Pattern

### Event Notification System

```rust
// Observer trait for state changes
#[async_trait]
pub trait StateObserver: Send + Sync {
    async fn on_state_changed(&self, old_state: &AppState, new_state: &AppState);
}

// Specific observer implementations
pub struct VisualizerStateObserver {
    visualizer: Arc<VisualizerEngine>,
}

#[async_trait]
impl StateObserver for VisualizerStateObserver {
    async fn on_state_changed(&self, _old: &AppState, new: &AppState) {
        if let Some(ref config) = new.visualizer_config {
            self.visualizer.update_config(config.clone()).await;
        }
    }
}

// Observable state manager
pub struct ObservableStateManager {
    state: Arc<RwLock<AppState>>,
    observers: Arc<Mutex<Vec<Arc<dyn StateObserver>>>>,
}

impl ObservableStateManager {
    pub async fn register_observer(&self, observer: Arc<dyn StateObserver>) {
        let mut observers = self.observers.lock().await;
        observers.push(observer);
    }

    async fn notify_observers(&self, old_state: &AppState, new_state: &AppState) {
        let observers = self.observers.lock().await;

        for observer in observers.iter() {
            let _ = observer.on_state_changed(old_state, new_state).await;
        }
    }
}
```

## Factory Pattern

### Plugin Factory

```rust
// Abstract factory for plugins
pub trait PluginFactory: Send + Sync {
    fn create_visualizer(&self, id: &str) -> Result<Box<dyn VisualizerPlugin>>;
    fn list_available(&self) -> Vec<PluginMetadata>;
}

// Built-in plugin factory
pub struct BuiltinPluginFactory;

impl PluginFactory for BuiltinPluginFactory {
    fn create_visualizer(&self, id: &str) -> Result<Box<dyn VisualizerPlugin>> {
        match id {
            "spectrum_bars" => Ok(Box::new(SpectrumBarsVisualizer::new())),
            "waveform" => Ok(Box::new(WaveformVisualizer::new())),
            "circle_spectrum" => Ok(Box::new(CircleSpectrumVisualizer::new())),
            _ => Err(PluginError::NotFound(id.to_string())),
        }
    }
}

// Dynamic plugin factory (for external plugins)
pub struct DynamicPluginFactory {
    library_path: PathBuf,
}

impl PluginFactory for DynamicPluginFactory {
    fn create_visualizer(&self, id: &str) -> Result<Box<dyn VisualizerPlugin>> {
        // Load dynamic library and create plugin
        let lib = libloading::Library::new(&self.library_path)?;

        let create_fn: libloading::Symbol<fn() -> Box<dyn VisualizerPlugin>> =
            unsafe { lib.get(b"create_visualizer")? };

        Ok(create_fn())
    }
}
```
