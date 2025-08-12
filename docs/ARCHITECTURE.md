# Sonic Flow - アーキテクチャドキュメント

## 📋 目次

1. [概要](#概要)
2. [アーキテクチャ原則](#アーキテクチャ原則)
3. [システム構成](#システム構成)
4. [レイヤー設計](#レイヤー設計)
5. [データフロー](#データフロー)
6. [モジュール設計](#モジュール設計)
7. [プラグインシステム](#プラグインシステム)
8. [パフォーマンス設計](#パフォーマンス設計)

## 概要

Sonic Flow は、音圧ビジュアライザーを中心とした高品質ミュージックプレイヤーです。Rust + Slint による高パフォーマンスな実装と、拡張可能なプラグインシステムを特徴とします。

### アーキテクチャスタイル

- **レイヤードアーキテクチャ**: 明確な層分離による保守性向上
- **プラグインアーキテクチャ**: 動的拡張可能な設計
- **イベント駆動アーキテクチャ**: リアクティブな処理フロー

## アーキテクチャ原則

### 1. 関心の分離 (Separation of Concerns)

```rust
// 各レイヤーが明確な責任を持つ
UI Layer        -> プレゼンテーション・ユーザー操作
Application     -> 制御・状態管理・コーディネーション
Business Logic  -> ドメインロジック・ビジネスルール
Infrastructure  -> 外部システム・技術的詳細
```

### 2. 依存性の逆転 (Dependency Inversion)

```rust
// 高水準モジュールは抽象に依存
pub trait AudioDecoder: Send + Sync {
    fn decode(&self, data: &[u8]) -> Result<AudioBuffer, DecoderError>;
    fn supports_format(&self, format: &AudioFormat) -> bool;
}

// 具体的な実装
pub struct Mp3Decoder;
impl AudioDecoder for Mp3Decoder {
    fn decode(&self, data: &[u8]) -> Result<AudioBuffer, DecoderError> {
        // MP3デコード実装
    }
}
```

### 3. 単一責任原則 (Single Responsibility)

```rust
// 各構造体は1つの責任のみを持つ
pub struct AudioEngine;      // 音声再生のみ
pub struct VisualizerEngine; // ビジュアライザー処理のみ
pub struct PlaylistManager;  // プレイリスト管理のみ
pub struct LibraryScanner;   // ライブラリスキャンのみ
```

### 4. 開放閉鎖原則 (Open/Closed)

```rust
// プラグインによる拡張、既存コードの修正なし
pub trait VisualizerPlugin: Send + Sync {
    fn render(&mut self, spectrum: &SpectrumData, config: &RenderConfig) -> RenderResult;
}

// 新しいビジュアライザーはtraitを実装するだけ
pub struct WaveformVisualizer;
impl VisualizerPlugin for WaveformVisualizer { /* ... */ }
```

## システム構成

### 全体構成図

```
┌─────────────────────────────────────────────┐
│                UI Layer (Slint)             │
│  ┌─────────┐ ┌─────────┐ ┌─────────────┐   │
│  │Controls │ │Playlist │ │ Visualizer  │   │
│  │ Panel   │ │  View   │ │   Canvas    │   │
│  └─────────┘ └─────────┘ └─────────────┘   │
└─────────────────┬───────────────────────────┘
                  │ Slint Bindings
┌─────────────────┴───────────────────────────┐
│            Application Layer                │
│  ┌─────────────┐ ┌─────────────────────┐   │
│  │ App Control │ │   State Manager     │   │
│  │    ler      │ │    (Arc<Mutex>)     │   │
│  └─────────────┘ └─────────────────────┘   │
└─────────────────┬───────────────────────────┘
                  │ Command/Event Bus
┌─────────────────┴───────────────────────────┐
│           Business Logic Layer              │
│ ┌──────────┐ ┌──────────┐ ┌──────────────┐ │
│ │  Audio   │ │Playlist  │ │ Visualizer   │ │
│ │ Engine   │ │ Manager  │ │   Engine     │ │
│ └──────────┘ └──────────┘ └──────────────┘ │
│ ┌──────────┐ ┌─────────────────────────────┘ │
│ │ Library  │ │      Plugin Manager        │ │
│ │ Manager  │ └─────────────────────────────┘ │
│ └──────────┘                               │
└─────────────────┬───────────────────────────┘
                  │ Infrastructure Interface
┌─────────────────┴───────────────────────────┐
│          Infrastructure Layer               │
│ ┌──────────┐ ┌──────────┐ ┌──────────────┐ │
│ │ Audio    │ │ File     │ │    Config    │ │
│ │ Drivers  │ │ System   │ │   Manager    │ │
│ └──────────┘ └──────────┘ └──────────────┘ │
│ ┌──────────┐ ┌──────────┐                  │
│ │Database  │ │  Plugin  │                  │
│ │(SQLite)  │ │ Loader   │                  │
│ └──────────┘ └──────────┘                  │
└─────────────────────────────────────────────┘
```

## レイヤー設計

### UI Layer (Slint)

**責任**: ユーザーインターフェース、ユーザー入力処理、データバインディング

```slint
// src/ui/main_window.slint
export component MainWindow inherits Window {
    // コールバック定義
    callback play_requested();
    callback pause_requested();
    callback visualizer_changed(string);
    callback track_selected(int);

    // UI状態プロパティ
    property<bool> is_playing;
    property<string> current_track;
    property<float> playback_position;
    property<[SpectrumData]> spectrum_data;

    // レイアウト
    VerticalLayout {
        PlayerControls {
            is_playing: is_playing;
            play_requested => { play_requested(); }
        }

        HorizontalLayout {
            PlaylistView { /* ... */ }
            VisualizerCanvas {
                spectrum_data: spectrum_data;
            }
        }
    }
}
```

**特徴**:

- 宣言的 UI 定義
- リアクティブなデータバインディング
- 高性能なアニメーション
- テーマシステム対応

### Application Layer

**責任**: アプリケーション制御、状態管理、UI-ビジネスロジック仲介

```rust
// src/app/controller.rs
pub struct AppController {
    state: Arc<RwLock<AppState>>,
    event_bus: EventBus,

    // ビジネスロジックへの参照
    audio_engine: Arc<AudioEngine>,
    visualizer_engine: Arc<VisualizerEngine>,
    playlist_manager: Arc<PlaylistManager>,
    library_manager: Arc<LibraryManager>,
}

impl AppController {
    pub async fn handle_play_request(&self, track_id: TrackId) -> Result<(), AppError> {
        // 1. 状態検証
        let state = self.state.read().await;
        if state.is_loading { return Err(AppError::Busy); }
        drop(state);

        // 2. オーディオエンジンに委譲
        self.audio_engine.load_and_play(track_id).await?;

        // 3. 状態更新
        {
            let mut state = self.state.write().await;
            state.current_track = Some(track_id);
            state.playback_state = PlaybackState::Playing;
        }

        // 4. イベント発火
        self.event_bus.emit(AudioEvent::PlaybackStarted(track_id));

        Ok(())
    }
}
```

**パターン**:

- **Command Pattern**: ユーザーアクション処理
- **Observer Pattern**: 状態変更通知
- **Mediator Pattern**: コンポーネント間仲介

### Business Logic Layer

**責任**: ドメインロジック、ビジネスルール実装

```rust
// src/audio/engine.rs
pub struct AudioEngine {
    decoder_registry: DecoderRegistry,
    renderer: Box<dyn AudioRenderer>,
    effects_chain: EffectsChain,
    spectrum_analyzer: SpectrumAnalyzer,

    // 状態管理
    current_track: Option<TrackInfo>,
    playback_state: Arc<AtomicCell<PlaybackState>>,

    // 通信チャンネル
    command_receiver: mpsc::Receiver<AudioCommand>,
    event_sender: mpsc::Sender<AudioEvent>,
}

impl AudioEngine {
    pub async fn process_audio_frame(&mut self, buffer: &mut AudioBuffer) -> Result<(), AudioError> {
        // 1. デコード処理
        let decoded = self.decoder_registry.decode_next_frame()?;

        // 2. エフェクト適用
        let processed = self.effects_chain.process(decoded)?;

        // 3. 出力レンダリング
        self.renderer.render(&processed, buffer)?;

        // 4. スペクトラム解析（ビジュアライザー用）
        let spectrum_data = self.spectrum_analyzer.analyze(&processed)?;

        // 5. ビジュアライザーエンジンに送信
        self.event_sender.send(AudioEvent::SpectrumUpdated(spectrum_data)).await?;

        Ok(())
    }
}
```

### Infrastructure Layer

**責任**: 外部システム連携、低レベル処理

```rust
// プラットフォーム抽象化
pub trait AudioDriver: Send + Sync {
    fn initialize(&mut self, config: &AudioConfig) -> Result<(), DriverError>;
    fn start_playback(&self) -> Result<(), DriverError>;
    fn write_samples(&self, buffer: &[f32]) -> Result<usize, DriverError>;
    fn stop_playback(&self) -> Result<(), DriverError>;
}

// プラットフォーム別実装
#[cfg(target_os = "windows")]
pub struct WasapiDriver;

#[cfg(target_os = "linux")]
pub struct AlsaDriver;

#[cfg(target_os = "macos")]
pub struct CoreAudioDriver;
```

## データフロー

### 音声処理フロー

```
Audio File → Decoder → Effects Chain → Renderer → Audio Driver → Speakers
     ↓
Spectrum Analyzer → Visualizer Engine → UI Canvas
```

### 制御フロー

```
User Input → UI Layer → App Controller → Business Logic → Infrastructure
     ↑                                          ↓
State Updates ←── Event Bus ←── Domain Events ←──┘
```

### 実装例

```rust
// イベント駆動フロー
#[derive(Debug, Clone)]
pub enum AudioEvent {
    TrackLoaded(TrackInfo),
    PlaybackStarted,
    PlaybackPaused,
    PlaybackStopped,
    SpectrumUpdated(SpectrumData),
    PlaybackPositionChanged(Duration),
    VolumeChanged(f32),
    ErrorOccurred(AudioError),
}

// イベントバス
pub struct EventBus {
    subscribers: Arc<RwLock<HashMap<TypeId, Vec<Box<dyn EventHandler>>>>>,
}

impl EventBus {
    pub async fn emit<T: Clone + Send + 'static>(&self, event: T) {
        let subscribers = self.subscribers.read().await;
        if let Some(handlers) = subscribers.get(&TypeId::of::<T>()) {
            for handler in handlers {
                handler.handle(Box::new(event.clone())).await;
            }
        }
    }
}
```

## モジュール設計

### 音声エンジンモジュール

```rust
// src/audio/mod.rs
pub mod engine;        // メインエンジン
pub mod decoder;       // デコーダー抽象化
pub mod renderer;      // 音声出力
pub mod effects;       // 音響効果
pub mod analysis;      // 周波数解析
pub mod formats;       // 対応フォーマット

// 公開API
pub use engine::{AudioEngine, AudioEngineBuilder};
pub use decoder::{DecoderRegistry, DecoderError};
pub use analysis::{SpectrumAnalyzer, SpectrumData, FrequencyBand};
pub use effects::{EffectsChain, Equalizer, ReverbEffect};

// 内部実装は非公開
use decoder::registry::DecoderRegistryImpl;
use renderer::platform::create_platform_renderer;
```

### ビジュアライザーモジュール

```rust
// src/visualizer/mod.rs
pub mod engine;        // ビジュアライザーエンジン
pub mod traits;        // プラグインAPI
pub mod renderer;      // 描画処理
pub mod config;        // 設定管理
pub mod plugins;       // 標準プラグイン

// プラグインAPI
pub use traits::{VisualizerPlugin, PluginMetadata, RenderConfig};
pub use engine::{VisualizerEngine, PluginManager};
pub use config::{VisualizationConfig, ColorScheme};

// 標準プラグイン
pub use plugins::{
    SpectrumBarsVisualizer,
    WaveformVisualizer,
    CircleSpectrumVisualizer,
    ParticleSystemVisualizer,
};
```

### モジュール間通信

```rust
// トレイトベース通信
pub trait EventHandler: Send + Sync {
    async fn handle(&self, event: Box<dyn Any + Send>) -> Result<(), HandleError>;
}

// 型安全なイベントハンドラー
pub struct VisualizerEventHandler {
    visualizer_engine: Arc<VisualizerEngine>,
}

impl EventHandler for VisualizerEventHandler {
    async fn handle(&self, event: Box<dyn Any + Send>) -> Result<(), HandleError> {
        if let Ok(spectrum_event) = event.downcast::<AudioEvent>() {
            match *spectrum_event {
                AudioEvent::SpectrumUpdated(data) => {
                    self.visualizer_engine.update_spectrum(data).await?;
                }
                _ => {}
            }
        }
        Ok(())
    }
}
```

## プラグインシステム

### プラグイン API 設計

```rust
// src/plugin/api.rs
pub trait VisualizerPlugin: Send + Sync {
    /// プラグインメタデータを返す
    fn metadata(&self) -> PluginMetadata;

    /// 初期化処理
    fn initialize(&mut self, config: &VisualizationConfig) -> Result<(), PluginError>;

    /// フレーム描画処理（60-120 FPS）
    fn render(&mut self, data: &SpectrumData, config: &RenderConfig) -> RenderResult;

    /// 設定変更処理
    fn configure(&mut self, settings: &HashMap<String, PluginValue>) -> Result<(), ConfigError>;

    /// 後処理
    fn cleanup(&mut self);
}

#[derive(Debug, Clone)]
pub struct PluginMetadata {
    pub name: String,
    pub version: semver::Version,
    pub author: String,
    pub description: String,
    pub supported_sample_rates: Vec<u32>,
    pub required_fft_size: usize,
}

// プラグイン登録マクロ
#[macro_export]
macro_rules! register_visualizer {
    ($plugin_type:ty) => {
        #[no_mangle]
        pub extern "C" fn create_visualizer() -> Box<dyn VisualizerPlugin> {
            Box::new(<$plugin_type>::default())
        }

        #[no_mangle]
        pub extern "C" fn get_metadata() -> PluginMetadata {
            <$plugin_type>::metadata()
        }
    };
}
```

### プラグインライフサイクル

```rust
pub struct PluginManager {
    loaded_plugins: HashMap<String, LoadedPlugin>,
    active_visualizers: Vec<String>,
    plugin_configs: HashMap<String, VisualizationConfig>,
}

struct LoadedPlugin {
    plugin: Box<dyn VisualizerPlugin>,
    library: libloading::Library,
    metadata: PluginMetadata,
}

impl PluginManager {
    pub async fn load_plugin(&mut self, path: &Path) -> Result<String, PluginError> {
        // 1. 動的ライブラリロード
        let lib = libloading::Library::new(path)?;

        // 2. メタデータ取得
        let get_metadata: libloading::Symbol<fn() -> PluginMetadata> =
            unsafe { lib.get(b"get_metadata")? };
        let metadata = get_metadata();

        // 3. プラグイン作成
        let create_fn: libloading::Symbol<fn() -> Box<dyn VisualizerPlugin>> =
            unsafe { lib.get(b"create_visualizer")? };
        let mut plugin = create_fn();

        // 4. 初期化
        let default_config = VisualizationConfig::default();
        plugin.initialize(&default_config)?;

        // 5. 登録
        let plugin_name = metadata.name.clone();
        self.loaded_plugins.insert(plugin_name.clone(), LoadedPlugin {
            plugin,
            library: lib,
            metadata,
        });

        tracing::info!("Loaded plugin: {}", plugin_name);
        Ok(plugin_name)
    }

    pub fn activate_visualizer(&mut self, name: &str) -> Result<(), PluginError> {
        if !self.loaded_plugins.contains_key(name) {
            return Err(PluginError::PluginNotFound(name.to_string()));
        }

        // 既存の visualizer を非アクティブ化
        self.active_visualizers.clear();
        self.active_visualizers.push(name.to_string());

        Ok(())
    }
}
```

## パフォーマンス設計

### リアルタイム処理保証

```rust
// Lock-free データ構造の使用
use crossbeam::atomic::AtomicCell;
use crossbeam::queue::SegQueue;

pub struct RealTimeAudioBuffer {
    // Triple buffering for smooth updates
    buffers: [AtomicCell<Option<AudioBuffer>>; 3],
    write_index: AtomicUsize,
    read_index: AtomicUsize,

    // Spectrum data queue (lock-free)
    spectrum_queue: SegQueue<SpectrumData>,
}

impl RealTimeAudioBuffer {
    pub fn write(&self, buffer: AudioBuffer) {
        let write_idx = self.write_index.load(Ordering::Relaxed);
        self.buffers[write_idx].store(Some(buffer));

        // Rotate write index
        let next_idx = (write_idx + 1) % 3;
        self.write_index.store(next_idx, Ordering::Release);
    }

    pub fn read(&self) -> Option<AudioBuffer> {
        let read_idx = self.read_index.load(Ordering::Acquire);
        let buffer = self.buffers[read_idx].take();

        if buffer.is_some() {
            let next_idx = (read_idx + 1) % 3;
            self.read_index.store(next_idx, Ordering::Relaxed);
        }

        buffer
    }
}
```

### メモリプール設計

```rust
// メモリアロケーション最小化
pub struct BufferPool<T> {
    free_buffers: SegQueue<Vec<T>>,
    buffer_size: usize,
    pool_size: usize,
}

impl<T: Default + Clone> BufferPool<T> {
    pub fn new(pool_size: usize, buffer_size: usize) -> Self {
        let free_buffers = SegQueue::new();

        // 事前割り当て
        for _ in 0..pool_size {
            free_buffers.push(vec![T::default(); buffer_size]);
        }

        Self {
            free_buffers,
            buffer_size,
            pool_size,
        }
    }

    pub fn acquire(&self) -> PooledBuffer<T> {
        let buffer = self.free_buffers
            .pop()
            .unwrap_or_else(|| {
                tracing::warn!("Buffer pool exhausted, allocating new buffer");
                vec![T::default(); self.buffer_size]
            });

        PooledBuffer::new(buffer, &self.free_buffers)
    }
}

pub struct PooledBuffer<T> {
    buffer: Option<Vec<T>>,
    return_queue: *const SegQueue<Vec<T>>,
}

impl<T> Drop for PooledBuffer<T> {
    fn drop(&mut self) {
        if let Some(buffer) = self.buffer.take() {
            unsafe {
                (*self.return_queue).push(buffer);
            }
        }
    }
}
```

### 並行処理設計

```rust
// 専用スレッド分離
pub struct ThreadedAudioSystem {
    audio_thread: JoinHandle<Result<(), AudioError>>,
    visualizer_thread: JoinHandle<Result<(), VisualizerError>>,

    // スレッド間通信
    audio_to_visualizer: mpsc::Sender<SpectrumData>,
    control_sender: mpsc::Sender<AudioCommand>,
}

impl ThreadedAudioSystem {
    pub fn new(config: AudioConfig) -> Result<Self, SystemError> {
        let (spectrum_tx, spectrum_rx) = mpsc::channel(1024);
        let (control_tx, control_rx) = mpsc::channel(256);

        // オーディオ処理スレッド（高優先度）
        let audio_thread = {
            let spectrum_tx = spectrum_tx.clone();
            let control_rx = control_rx;

            std::thread::Builder::new()
                .name("audio-engine".to_string())
                .spawn(move || {
                    // スレッド優先度設定
                    #[cfg(unix)]
                    {
                        unsafe {
                            libc::pthread_setschedparam(
                                libc::pthread_self(),
                                libc::SCHED_FIFO,
                                &libc::sched_param { sched_priority: 80 },
                            );
                        }
                    }

                    let mut engine = AudioEngine::new(config)?;
                    engine.run_main_loop(control_rx, spectrum_tx)
                })?
        };

        // ビジュアライザースレッド（中優先度）
        let visualizer_thread = {
            std::thread::Builder::new()
                .name("visualizer-engine".to_string())
                .spawn(move || {
                    let mut engine = VisualizerEngine::new()?;
                    engine.run_main_loop(spectrum_rx)
                })?
        };

        Ok(Self {
            audio_thread,
            visualizer_thread,
            audio_to_visualizer: spectrum_tx,
            control_sender: control_tx,
        })
    }
}
```

### パフォーマンス監視

```rust
// パフォーマンスメトリクス収集
#[derive(Debug, Default)]
pub struct PerformanceMetrics {
    pub audio_latency: MovingAverage,
    pub render_time: MovingAverage,
    pub cpu_usage: f64,
    pub memory_usage: u64,
    pub dropped_frames: AtomicU64,
}

impl PerformanceMetrics {
    pub fn record_audio_latency(&self, latency: Duration) {
        self.audio_latency.add_sample(latency.as_micros() as f64);
    }

    pub fn record_render_time(&self, time: Duration) {
        self.render_time.add_sample(time.as_micros() as f64);

        // パフォーマンス警告
        if time > Duration::from_millis(16) {
            tracing::warn!("Render time exceeded target: {:?}", time);
        }
    }

    pub fn is_performance_acceptable(&self) -> bool {
        self.audio_latency.average() < 50_000.0 && // 50ms
        self.render_time.average() < 16_000.0 &&   // 16ms
        self.cpu_usage < 0.05                       // 5%
    }
}

struct MovingAverage {
    samples: VecDeque<f64>,
    max_samples: usize,
    sum: f64,
}

impl MovingAverage {
    fn new(window_size: usize) -> Self {
        Self {
            samples: VecDeque::with_capacity(window_size),
            max_samples: window_size,
            sum: 0.0,
        }
    }

    fn add_sample(&mut self, value: f64) {
        if self.samples.len() >= self.max_samples {
            if let Some(old_value) = self.samples.pop_front() {
                self.sum -= old_value;
            }
        }

        self.samples.push_back(value);
        self.sum += value;
    }

    fn average(&self) -> f64 {
        if self.samples.is_empty() {
            0.0
        } else {
            self.sum / self.samples.len() as f64
        }
    }
}
```

## セキュリティ設計

### プラグインサンドボックス

```rust
// プラグイン実行制限
pub struct SecurePluginLoader {
    allowed_paths: HashSet<PathBuf>,
    max_memory: usize,
    max_cpu_time: Duration,
}

impl SecurePluginLoader {
    pub fn load_plugin(&self, path: &Path) -> Result<LoadedPlugin, SecurityError> {
        // パス検証
        if !self.is_path_allowed(path) {
            return Err(SecurityError::UnauthorizedPath);
        }

        // ファイル署名検証
        self.verify_plugin_signature(path)?;

        // リソース制限設定
        let _guard = ResourceLimitGuard::new(self.max_memory, self.max_cpu_time);

        // 安全なロード
        self.load_with_restrictions(path)
    }

    fn verify_plugin_signature(&self, path: &Path) -> Result<(), SecurityError> {
        // デジタル署名検証（実装省略）
        Ok(())
    }
}
```

---

**最終更新**: 2025-08-12  
**バージョン**: 2.0  
**レビュアー**: Claude (アーキテクチャ再設計)
