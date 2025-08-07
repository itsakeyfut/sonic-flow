# Resonance Player - アーキテクチャドキュメント

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

Resonance Player は、音圧ビジュアライザーを中心とした高品質ミュージックプレイヤーです。Rust + Slint による高パフォーマンスな実装と、拡張可能なプラグインシステムを特徴とします。

### アーキテクチャスタイル

- **レイヤードアーキテクチャ**: 明確な層分離
- **プラグインアーキテクチャ**: 拡張可能な設計
- **イベント駆動アーキテクチャ**: リアクティブな処理

## アーキテクチャ原則

### 1. 関心の分離 (Separation of Concerns)

```rust
// 各レイヤーが明確な責任を持つ
UI Layer        -> プレゼンテーション
Application     -> 制御・状態管理
Business Logic  -> ドメインロジック
Infrastructure  -> 技術的詳細
```

### 2. 依存性の逆転 (Dependency Inversion)

```rust
// 高水準モジュールは低水準モジュールに依存しない
trait AudioDecoder {
    fn decode(&self, data: &[u8]) -> Result<AudioData, DecoderError>;
}

struct Mp3Decoder;
impl AudioDecoder for Mp3Decoder { /* ... */ }
```

### 3. 単一責任原則 (Single Responsibility)

```rust
// 各構造体は1つの責任のみを持つ
struct AudioEngine;     // 音声再生のみ
struct VisualizerEngine; // ビジュアライザーのみ
struct PlaylistManager; // プレイリスト管理のみ
```

### 4. 開放閉鎖原則 (Open/Closed)

```rust
// プラグインによる拡張、既存コードの修正なし
trait Visualizer {
    fn render(&self, spectrum: &[f32]) -> RenderData;
}
// 新しいビジュアライザーはtraitを実装するだけ
```

## システム構成

### 全体構成図

```
┌─────────────────────────────────────────────┐
│                UI Layer                     │
│  ┌─────────┐ ┌─────────┐ ┌─────────────┐   │
│  │ Controls│ │Playlist │ │  Visualizer │   │
│  │         │ │  View   │ │   Canvas    │   │
│  └─────────┘ └─────────┘ └─────────────┘   │
└─────────────────┬───────────────────────────┘
                  │ Slint Bindings
┌─────────────────┴───────────────────────────┐
│            Application Layer                │
│  ┌─────────────┐ ┌─────────────────────┐   │
│  │ App Control │ │   State Manager     │   │
│  └─────────────┘ └─────────────────────┘   │
└─────────────────┬───────────────────────────┘
                  │ Command/Query
┌─────────────────┴───────────────────────────┐
│           Business Logic Layer              │
│ ┌──────────┐ ┌──────────┐ ┌──────────────┐ │
│ │  Audio   │ │Playlist  │ │ Visualizer   │ │
│ │ Engine   │ │ Manager  │ │   Engine     │ │
│ └──────────┘ └──────────┘ └──────────────┘ │
└─────────────────┬───────────────────────────┘
                  │ Infrastructure Interface
┌─────────────────┴───────────────────────────┐
│          Infrastructure Layer               │
│ ┌──────────┐ ┌──────────┐ ┌──────────────┐ │
│ │ Audio    │ │ File     │ │   Plugin     │ │
│ │ Drivers  │ │ System   │ │   Loader     │ │
│ └──────────┘ └──────────┘ └──────────────┘ │
└─────────────────────────────────────────────┘
```

## レイヤー設計

### UI Layer (Slint)

**責任**: ユーザーインターフェース、ユーザー入力処理

```rust
// src/ui/main_window.slint
export component MainWindow inherits Window {
    // UI定義
    callback play_requested();
    callback visualizer_changed(string);

    // UI状態
    property<bool> is_playing;
    property<string> current_track;
}
```

**特徴**:

- 宣言的 UI 定義
- データバインディング
- アニメーション対応
- テーマシステム

### Application Layer

**責任**: アプリケーション制御、状態管理、UI-ビジネスロジック仲介

```rust
// src/app/controller.rs
pub struct AppController {
    state: StateManager,
    audio_engine: Arc<AudioEngine>,
    visualizer_engine: Arc<VisualizerEngine>,
    playlist_manager: Arc<PlaylistManager>,
}

impl AppController {
    pub async fn handle_play_request(&self) -> Result<(), AppError> {
        // 複数のビジネスロジックを協調
    }
}
```

**パターン**:

- Command Pattern: ユーザーアクション処理
- Observer Pattern: 状態変更通知
- Mediator Pattern: コンポーネント仲介

### Business Logic Layer

**責任**: ドメインロジック、ビジネスルール実装

```rust
// src/audio/engine.rs
pub struct AudioEngine {
    decoder: Box<dyn AudioDecoder>,
    renderer: AudioRenderer,
    effects: EffectsChain,
    analyzer: SpectrumAnalyzer,
}

impl AudioEngine {
    pub async fn process_audio(&self, buffer: &mut [f32]) -> Result<(), AudioError> {
        // 音声処理パイプライン
        self.decoder.decode()?;
        self.effects.apply(buffer)?;
        self.renderer.render(buffer)?;
        self.analyzer.analyze(buffer)?; // ビジュアライザー用
        Ok(())
    }
}
```

### Infrastructure Layer

**責任**: 外部システム連携、低レベル処理

```rust
// システム依存の実装詳細を隠蔽
pub trait AudioDriver {
    fn start(&self) -> Result<(), DriverError>;
    fn write(&self, buffer: &[f32]) -> Result<(), DriverError>;
}

// プラットフォーム別実装
#[cfg(target_os = "windows")]
pub struct WasapiDriver;

#[cfg(target_os = "linux")]
pub struct AlsaDriver;
```

## データフロー

### 音声処理フロー

```
Audio File → Decoder → Effects → Renderer → Audio Driver → Speakers
     ↓
FFT Analyzer → Visualizer Engine → UI Canvas
```

### 制御フロー

```
User Input → UI Layer → App Controller → Business Logic → Infrastructure
     ↑                                          ↓
State Updates ←── Event Bus ←── Domain Events ←──┘
```

### 実装例:

```rust
// 音声データフロー
pub struct AudioPipeline {
    stages: Vec<Box<dyn AudioProcessor>>,
}

impl AudioPipeline {
    pub fn process(&mut self, mut buffer: AudioBuffer) -> Result<AudioBuffer, AudioError> {
        for stage in &mut self.stages {
            buffer = stage.process(buffer)?;
        }
        Ok(buffer)
    }
}

// ビジュアライザーデータフロー
pub struct VisualizationPipeline {
    analyzer: SpectrumAnalyzer,
    active_visualizers: Vec<Box<dyn Visualizer>>,
}
```

## モジュール設計

### 音声エンジンモジュール

```rust
// src/audio/mod.rs
pub mod engine;     // メインエンジン
pub mod decoder;    // デコーダー抽象化
pub mod renderer;   // 音声出力
pub mod effects;    // 音響効果
pub mod analysis;   // 周波数解析

// 公開API
pub use engine::AudioEngine;
pub use decoder::{AudioDecoder, DecoderError};
pub use analysis::{SpectrumAnalyzer, SpectrumData};
```

### ビジュアライザーモジュール

```rust
// src/visualizer/mod.rs
pub mod engine;     // ビジュアライザーエンジン
pub mod traits;     // プラグインAPI
pub mod renderer;   // 描画処理
pub mod plugins;    // 標準プラグイン

// プラグインAPI
pub use traits::{Visualizer, VisualizationConfig};
pub use engine::VisualizerEngine;
```

### モジュール間通信

```rust
// Event-driven communication
pub enum AudioEvent {
    TrackChanged(TrackInfo),
    SpectrumUpdated(SpectrumData),
    PlaybackStateChanged(PlaybackState),
}

pub trait EventHandler {
    fn handle_event(&mut self, event: AudioEvent) -> Result<(), HandleError>;
}
```

## プラグインシステム

### プラグイン API 設計

```rust
// src/plugin/api.rs
pub trait Visualizer: Send + Sync {
    fn name(&self) -> &str;
    fn render(&mut self, data: &SpectrumData, config: &RenderConfig) -> RenderResult;
    fn configure(&mut self, config: VisualizationConfig) -> Result<(), ConfigError>;
}

// プラグイン登録マクロ
#[macro_export]
macro_rules! register_visualizer {
    ($name:ty) => {
        #[no_mangle]
        pub extern "C" fn create_visualizer() -> Box<dyn Visualizer> {
            Box::new(<$name>::new())
        }
    };
}
```

### プラグインライフサイクル

```rust
pub struct PluginManager {
    loaded_plugins: HashMap<String, Box<dyn Visualizer>>,
    plugin_configs: HashMap<String, VisualizationConfig>,
}

impl PluginManager {
    pub fn load_plugin(&mut self, path: &Path) -> Result<(), PluginError> {
        // 動的ライブラリロード
        // プラグイン初期化
        // レジストリ登録
    }

    pub fn get_visualizer(&self, name: &str) -> Option<&dyn Visualizer> {
        self.loaded_plugins.get(name).map(|v| v.as_ref())
    }
}
```

## パフォーマンス設計

### リアルタイム処理保証

```rust
// Lock-freeデータ構造の使用
use crossbeam::atomic::AtomicCell;
use crossbeam::queue::SegQueue;

pub struct RealTimeAudioBuffer {
    buffer: AtomicCell<AudioBuffer>,
    queue: SegQueue<SpectrumData>,
}

// メモリアロケーション最小化
pub struct BufferPool {
    buffers: Vec<AudioBuffer>,
    free_list: Vec<usize>,
}
```

### 並行処理設計

```rust
// 専用スレッド分離
struct ThreadPool {
    audio_thread: JoinHandle<()>,      // 音声処理専用
    visual_thread: JoinHandle<()>,     // ビジュアル処理専用
    ui_thread: JoinHandle<()>,         // UI更新専用
}

// チャンネルベース通信
pub struct AudioVisualizerBridge {
    spectrum_sender: Sender<SpectrumData>,
    spectrum_receiver: Receiver<SpectrumData>,
}
```

### メモリ管理戦略

```rust
// ゼロコピー設計
pub struct ZeroCopyBuffer<T> {
    data: Arc<[T]>,        // 共有データ
    range: Range<usize>,   // 有効範囲
}

// RAII による確実なクリーンアップ
pub struct AudioResource {
    handle: AudioHandle,
}

impl Drop for AudioResource {
    fn drop(&mut self) {
        // 自動クリーンアップ
    }
}
```

---

**最終更新**: 2025-08-07  
**バージョン**: 1.0  
**レビュアー**: プロジェクトアーキテクト
