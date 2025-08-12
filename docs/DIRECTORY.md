# Sonic Flow - ディレクトリ構成ガイド

## 📋 目次

1. [プロジェクト構造概要](#プロジェクト構造概要)
2. [ソースコード構成](#ソースコード構成)
3. [モジュール詳細](#モジュール詳細)
4. [ファイル配置ルール](#ファイル配置ルール)
5. [テスト構成](#テスト構成)
6. [ビルドとデプロイメント](#ビルドとデプロイメント)

## プロジェクト構造概要

```
sonic-flow/
├── 📁 src/                    # ソースコード
├── 📁 tests/                  # 統合テスト
├── 📁 benches/                # ベンチマーク
├── 📁 examples/               # 使用例
├── 📁 docs/                   # ドキュメント
├── 📁 assets/                 # 静的リソース
├── 📁 scripts/                # ビルドスクリプト
├── 📁 tools/                  # 開発ツール
├── 📁 plugins/                # 標準プラグイン
├── 📁 packages/               # パッケージング用ファイル
├── 📄 Cargo.toml              # プロジェクト設定
├── 📄 Cargo.lock              # 依存関係ロック
├── 📄 README.md               # プロジェクト概要
├── 📄 CHANGELOG.md            # 変更履歴
├── 📄 LICENSE                 # ライセンス
└── 📄 .gitignore              # Git除外設定
```

## ソースコード構成

### レイヤード・アーキテクチャ別構成

```
src/
├── 📄 main.rs                 # アプリケーションエントリーポイント
├── 📄 lib.rs                  # ライブラリルート（公開API定義）
│
├── 🎨 UI Layer (Slint)
│   └── 📁 ui/
│       ├── 📄 mod.rs
│       ├── 📄 bindings.rs     # Rust-Slintバインディング
│       ├── 📄 theme.rs        # テーマ管理
│       ├── 📄 components.rs   # UI コンポーネント定義
│       │
│       ├── 📄 main_window.slint       # メインウィンドウ
│       │
│       ├── 📁 components/             # 再利用可能コンポーネント
│       │   ├── 📄 player_controls.slint    # 再生コントロール
│       │   ├── 📄 playlist_view.slint      # プレイリスト表示
│       │   ├── 📄 library_browser.slint    # ライブラリブラウザー
│       │   ├── 📄 visualizer_canvas.slint  # ビジュアライザー描画エリア
│       │   ├── 📄 settings_panel.slint     # 設定パネル
│       │   ├── 📄 track_info.slint         # トラック情報表示
│       │   ├── 📄 search_box.slint         # 検索ボックス
│       │   └── 📄 common.slint             # 共通スタイル・コンポーネント
│       │
│       └── 📁 themes/                 # テーマ定義
│           ├── 📄 dark.slint         # ダークテーマ
│           ├── 📄 light.slint        # ライトテーマ
│           ├── 📄 high_contrast.slint # 高コントラストテーマ
│           └── 📄 custom.slint       # カスタムテーマ
│
├── 🎛️ Application Layer
│   └── 📁 app/
│       ├── 📄 mod.rs                  # アプリケーション制御API
│       ├── 📄 controller.rs           # メインアプリケーションコントローラー
│       ├── 📄 state.rs               # アプリケーション状態管理
│       ├── 📄 events.rs              # イベント処理・イベントバス
│       ├── 📄 lifecycle.rs           # アプリケーションライフサイクル
│       ├── 📄 commands.rs            # コマンドパターン実装
│       └── 📄 services.rs            # アプリケーションサービス
│
├── 🏭 Business Logic Layer
│   ├── 📁 audio/                     # 音声処理
│   │   ├── 📄 mod.rs                 # 音声モジュール公開API
│   │   ├── 📄 engine.rs              # メインオーディオエンジン
│   │   ├── 📄 renderer.rs            # 音声レンダリング
│   │   ├── 📄 buffer.rs              # オーディオバッファ管理
│   │   ├── 📄 stream.rs              # 音声ストリーミング
│   │   │
│   │   ├── 📁 decoder/               # デコーダー実装
│   │   │   ├── 📄 mod.rs
│   │   │   ├── 📄 registry.rs        # デコーダーレジストリ
│   │   │   ├── 📄 traits.rs          # デコーダートレイト
│   │   │   ├── 📄 mp3.rs             # MP3デコーダー
│   │   │   ├── 📄 flac.rs            # FLACデコーダー
│   │   │   ├── 📄 wav.rs             # WAVデコーダー
│   │   │   ├── 📄 ogg.rs             # OGGデコーダー
│   │   │   └── 📄 aac.rs             # AACデコーダー
│   │   │
│   │   ├── 📁 effects/               # 音響効果
│   │   │   ├── 📄 mod.rs
│   │   │   ├── 📄 chain.rs           # エフェクトチェーン
│   │   │   ├── 📄 equalizer.rs       # イコライザー
│   │   │   ├── 📄 reverb.rs          # リバーブ
│   │   │   ├── 📄 delay.rs           # ディレイ/エコー
│   │   │   ├── 📄 crossfade.rs       # クロスフェード
│   │   │   ├── 📄 spatial.rs         # 3D音響効果
│   │   │   └── 📄 filters.rs         # デジタルフィルター
│   │   │
│   │   └── 📁 analysis/              # 音声解析
│   │       ├── 📄 mod.rs
│   │       ├── 📄 fft.rs             # FFT処理
│   │       ├── 📄 spectrum.rs        # スペクトラム解析
│   │       ├── 📄 meter.rs           # レベルメーター
│   │       ├── 📄 tempo.rs           # テンポ検出
│   │       ├── 📄 pitch.rs           # ピッチ検出
│   │       └── 📄 loudness.rs        # ラウドネス測定
│   │
│   ├── 📁 visualizer/                # ビジュアライザー
│   │   ├── 📄 mod.rs                 # ビジュアライザー公開API
│   │   ├── 📄 engine.rs              # ビジュアライザーエンジン
│   │   ├── 📄 traits.rs              # ビジュアライザートレイト
│   │   ├── 📄 renderer.rs            # 共通描画処理
│   │   ├── 📄 config.rs              # 設定管理
│   │   ├── 📄 canvas.rs              # 描画キャンバス
│   │   ├── 📄 colors.rs              # カラーシステム
│   │   │
│   │   └── 📁 plugins/               # ビジュアライザープラグイン
│   │       ├── 📄 mod.rs
│   │       ├── 📄 spectrum_bars.rs    # スペクトラムバー
│   │       ├── 📄 waveform.rs         # 波形表示
│   │       ├── 📄 circle_spectrum.rs  # サークルスペクトラム
│   │       ├── 📄 particle_system.rs  # パーティクルシステム
│   │       ├── 📄 spectrum_3d.rs      # 3Dスペクトラム
│   │       ├── 📄 vu_meters.rs        # VUメーター
│   │       ├── 📄 waterfall.rs        # ウォーターフォール
│   │       └── 📄 oscilloscope.rs     # オシロスコープ
│   │
│   ├── 📁 playlist/                  # プレイリスト
│   │   ├── 📄 mod.rs                 # プレイリスト公開API
│   │   ├── 📄 manager.rs             # プレイリストマネージャー
│   │   ├── 📄 playlist.rs            # プレイリスト実装
│   │   ├── 📄 smart_playlist.rs      # スマートプレイリスト
│   │   ├── 📄 shuffle.rs             # シャッフルアルゴリズム
│   │   ├── 📄 repeat.rs              # リピート機能
│   │   │
│   │   └── 📁 formats/               # プレイリスト形式対応
│   │       ├── 📄 mod.rs
│   │       ├── 📄 traits.rs          # フォーマットトレイト
│   │       ├── 📄 m3u.rs             # M3U形式
│   │       ├── 📄 pls.rs             # PLS形式
│   │       ├── 📄 xspf.rs            # XSPF形式
│   │       └── 📄 json.rs            # JSON形式（独自）
│   │
│   └── 📁 library/                   # ライブラリ管理
│       ├── 📄 mod.rs                 # ライブラリ公開API
│       ├── 📄 manager.rs             # ライブラリマネージャー
│       ├── 📄 scanner.rs             # ファイルスキャン
│       ├── 📄 metadata.rs            # メタデータ処理
│       ├── 📄 database.rs            # データベース操作（SQLite）
│       ├── 📄 artwork.rs             # アルバムアートワーク処理
│       ├── 📄 search.rs              # 検索機能
│       ├── 📄 indexer.rs             # インデックス作成
│       ├── 📄 watcher.rs             # ファイルシステム監視
│       └── 📄 cache.rs               # キャッシュ管理
│
└── 🔧 Infrastructure Layer
    ├── 📁 config/                    # 設定管理
    │   ├── 📄 mod.rs                 # 設定公開API
    │   ├── 📄 manager.rs             # 設定マネージャー
    │   ├── 📄 schema.rs              # 設定スキーマ定義
    │   ├── 📄 defaults.rs            # デフォルト設定値
    │   ├── 📄 migration.rs           # 設定マイグレーション
    │   ├── 📄 validation.rs          # 設定検証
    │   └── 📄 persistence.rs         # 設定永続化
    │
    ├── 📁 plugin/                    # プラグインシステム
    │   ├── 📄 mod.rs                 # プラグイン公開API
    │   ├── 📄 manager.rs             # プラグインマネージャー
    │   ├── 📄 loader.rs              # 動的ライブラリローダー
    │   ├── 📄 api.rs                 # プラグインAPI定義
    │   ├── 📄 registry.rs            # プラグインレジストリ
    │   ├── 📄 security.rs            # セキュリティ・サンドボックス
    │   ├── 📄 discovery.rs           # プラグイン自動発見
    │   └── 📄 validation.rs          # プラグイン検証
    │
    ├── 📁 storage/                   # データストレージ
    │   ├── 📄 mod.rs
    │   ├── 📄 database.rs            # データベース接続
    │   ├── 📄 migrations.rs          # スキーママイグレーション
    │   ├── 📄 repository.rs          # リポジトリパターン
    │   ├── 📄 cache.rs               # キャッシュレイヤー
    │   └── 📄 backup.rs              # バックアップ機能
    │
    ├── 📁 utils/                     # ユーティリティ
    │   ├── 📄 mod.rs                 # ユーティリティ公開API
    │   ├── 📄 file.rs                # ファイル操作
    │   ├── 📄 math.rs                # 数学関数
    │   ├── 📄 color.rs               # カラー処理
    │   ├── 📄 animation.rs           # アニメーション
    │   ├── 📄 platform.rs            # プラットフォーム固有処理
    │   ├── 📄 time.rs                # 時間処理
    │   ├── 📄 encoding.rs            # 文字エンコーディング
    │   └── 📄 crypto.rs              # 暗号化機能
    │
    ├── 📁 error/                     # エラー処理
    │   ├── 📄 mod.rs                 # エラー公開API
    │   ├── 📄 types.rs               # エラー型定義
    │   ├── 📄 handling.rs            # エラーハンドリング
    │   ├── 📄 recovery.rs            # エラー復旧
    │   └── 📄 reporting.rs           # エラーレポート
    │
    ├── 📁 logging/                   # ログシステム
    │   ├── 📄 mod.rs
    │   ├── 📄 logger.rs              # ロガー実装
    │   ├── 📄 filters.rs             # ログフィルター
    │   └── 📄 formatters.rs          # ログフォーマッター
    │
    └── 📁 telemetry/                 # テレメトリー
        ├── 📄 mod.rs
        ├── 📄 metrics.rs             # メトリクス収集
        ├── 📄 monitoring.rs          # パフォーマンス監視
        └── 📄 diagnostics.rs         # 診断機能
```

**責務**: 外部システム連携、低レベル処理、インフラストラクチャ管理

## モジュール詳細

### 🎵 Audio モジュール (`src/audio/`)

**主要な型・トレイト**:

```rust
// src/audio/mod.rs
pub use engine::{AudioEngine, AudioEngineBuilder, AudioEngineError};
pub use decoder::{AudioDecoder, DecoderRegistry, DecoderError, AudioFormat};
pub use effects::{EffectsChain, EffectProcessor, EffectError};
pub use analysis::{SpectrumAnalyzer, SpectrumData, FrequencyBand, AnalysisError};
pub use renderer::{AudioRenderer, RenderError};

// 内部実装は非公開
mod engine;
mod decoder;
mod effects;
mod analysis;
mod renderer;
mod buffer;
mod stream;
```

**設計パターン**:

- **Strategy Pattern**: 異なるデコーダー・エフェクトの切り替え
- **Chain of Responsibility**: エフェクトチェーン処理
- **Observer Pattern**: 音声イベント通知

### 🎨 Visualizer モジュール (`src/visualizer/`)

**主要な型・トレイト**:

```rust
// src/visualizer/mod.rs
pub use engine::{VisualizerEngine, PluginManager};
pub use traits::{VisualizerPlugin, PluginMetadata, RenderConfig, RenderError};
pub use config::{VisualizationConfig, ColorScheme, AnimationSettings};
pub use canvas::{Canvas, DrawCommand, Primitive};

// 標準プラグイン
pub use plugins::{
    SpectrumBarsVisualizer,
    WaveformVisualizer,
    CircleSpectrumVisualizer,
    ParticleSystemVisualizer,
    Spectrum3DVisualizer,
    VUMetersVisualizer,
    WaterfallVisualizer,
    OscilloscopeVisualizer,
};
```

**設計パターン**:

- **Plugin Architecture**: 拡張可能なビジュアライザー
- **Command Pattern**: 描画コマンドキュー
- **Factory Pattern**: プラグインインスタンス作成

### 📄 UI モジュール (`src/ui/`)

**Slint コンポーネント構成**:

```slint
// src/ui/main_window.slint
import { PlayerControls } from "components/player_controls.slint";
import { PlaylistView } from "components/playlist_view.slint";
import { VisualizerCanvas } from "components/visualizer_canvas.slint";
import { LibraryBrowser } from "components/library_browser.slint";
import { SettingsPanel } from "components/settings_panel.slint";

export component MainWindow inherits Window {
    // コンポーネント構成
    VerticalLayout {
        TopBar { /* ... */ }
        HorizontalLayout {
            Sidebar { /* ... */ }
            VisualizerArea { /* ... */ }
            if show_right_panel: RightPanel { /* ... */ }
        }
        PlayerControls { /* ... */ }
    }
}
```

### 🎛️ App モジュール (`src/app/`)

**アプリケーション制御層**:

```rust
// src/app/mod.rs
pub use controller::{AppController, ControllerError};
pub use state::{AppState, StateManager, StateError};
pub use events::{EventBus, AudioEvent, UIEvent, SystemEvent};
pub use commands::{Command, CommandProcessor, CommandError};
pub use services::{ServiceRegistry, ServiceError};

// 内部実装
mod controller;
mod state;
mod events;
mod commands;
mod lifecycle;
mod services;
```

**責務**: UI-ビジネスロジック仲介、アプリケーション状態管理、イベント処理

### 📋 Playlist モジュール (`src/playlist/`)

**プレイリスト管理**:

```rust
// src/playlist/mod.rs
pub use manager::{PlaylistManager, PlaylistError};
pub use playlist::{Playlist, PlaylistId, PlaylistMetadata};
pub use smart_playlist::{SmartPlaylist, PlaylistCriterion, LogicalOperator};
pub use shuffle::{ShuffleAlgorithm, FisherYatesShuffle};
pub use formats::{PlaylistFormat, FormatError};

// フォーマット対応
pub use formats::{M3uFormat, PlsFormat, XspfFormat, JsonFormat};
```

**設計パターン**:

- **Repository Pattern**: プレイリストデータアクセス
- **Strategy Pattern**: シャッフル・リピートアルゴリズム
- **Builder Pattern**: スマートプレイリスト構築

### 📚 Library モジュール (`src/library/`)

**ライブラリ管理**:

```rust
// src/library/mod.rs
pub use manager::{LibraryManager, LibraryError};
pub use scanner::{LibraryScanner, ScanResult, ScanMode};
pub use metadata::{MetadataExtractor, TrackInfo, ArtworkInfo};
pub use search::{SearchEngine, SearchQuery, SearchResult};
pub use database::{DatabaseRepository, QueryBuilder};
pub use watcher::{FileSystemWatcher, WatchEvent};

// 検索機能
pub use search::{SearchCriterion, SortOrder, FilterOptions};
```

### 🔌 Plugin モジュール (`src/plugin/`)

**プラグインシステム**:

```rust
// src/plugin/mod.rs
pub use manager::{PluginManager, PluginError};
pub use loader::{PluginLoader, LoadResult};
pub use api::{VisualizerPlugin, EffectPlugin, PluginValue};
pub use registry::{PluginRegistry, PluginInfo};
pub use security::{SecurityPolicy, PluginSandbox};

// プラグイン開発支援
pub mod development {
    pub use super::api::*;
    pub use crate::register_visualizer;
    pub use crate::register_effect;
}
```

## ファイル配置ルール

### 1. モジュール構造ルール

```rust
// 各モジュールのmod.rsは公開APIのみ定義
// src/audio/mod.rs
pub use engine::AudioEngine;
pub use decoder::{AudioDecoder, DecoderRegistry, DecoderResult};
pub use analysis::{SpectrumAnalyzer, SpectrumData};
pub use effects::{EffectsChain, Equalizer};

// 内部実装詳細は非公開
mod engine;
mod decoder;
mod analysis;
mod effects;
mod renderer;
mod buffer;

// 再エクスポートで階層構造を隠蔽
pub mod prelude {
    pub use super::{AudioEngine, SpectrumAnalyzer};
    pub use super::decoder::{AudioDecoder, AudioFormat};
}
```

### 2. ファイル命名規則

**カテゴリ別命名**:

- **構造体中心**: `engine.rs`, `manager.rs`, `analyzer.rs`, `scanner.rs`
- **機能中心**: `playback.rs`, `effects.rs`, `rendering.rs`, `search.rs`
- **データ中心**: `metadata.rs`, `config.rs`, `types.rs`, `schema.rs`
- **抽象化**: `traits.rs`, `api.rs`, `interface.rs`
- **実装**: `implementation.rs`, `backend.rs`, `platform.rs`

### 3. トレイト定義の配置

```rust
// トレイトは独立したファイルに配置
// src/audio/traits.rs
pub trait AudioDecoder: Send + Sync {
    fn decode(&self, data: &[u8]) -> Result<AudioBuffer, DecoderError>;
    fn supports_format(&self, format: &AudioFormat) -> bool;
    fn metadata(&self) -> DecoderMetadata;
}

pub trait AudioRenderer: Send + Sync {
    fn initialize(&mut self, config: &AudioConfig) -> Result<(), RenderError>;
    fn render(&mut self, buffer: &AudioBuffer) -> Result<(), RenderError>;
    fn flush(&mut self) -> Result<(), RenderError>;
}

// または機能ごとにtraitsディレクトリ内に分割
// src/audio/traits/
//   ├── decoder.rs
//   ├── renderer.rs
//   ├── effect.rs
//   └── mod.rs
```

### 4. 設定ファイル管理

```rust
// src/config/schema.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    pub sample_rate: u32,
    pub buffer_size: u32,
    pub bit_depth: u8,
    pub device: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationConfig {
    pub default_type: String,
    pub fft_size: usize,
    pub update_rate: f32,
    pub sensitivity: f32,
}

// 設定バリデーション
impl AudioConfig {
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.sample_rate < 8000 || self.sample_rate > 192000 {
            return Err(ConfigError::InvalidSampleRate(self.sample_rate));
        }
        // 他の検証...
        Ok(())
    }
}
```

## テスト構成

### テストディレクトリ構造

```
tests/
├── 📁 integration/               # 統合テスト
│   ├── 📄 audio_pipeline_test.rs      # 音声パイプラインテスト
│   ├── 📄 visualizer_integration_test.rs # ビジュアライザー統合テスト
│   ├── 📄 playlist_management_test.rs  # プレイリスト管理テスト
│   ├── 📄 library_scan_test.rs        # ライブラリスキャンテスト
│   ├── 📄 plugin_system_test.rs       # プラグインシステムテスト
│   ├── 📄 ui_interaction_test.rs      # UI操作テスト
│   └── 📄 performance_test.rs         # パフォーマンステスト
│
├── 📁 fixtures/                  # テスト用データ
│   ├── 📁 audio/
│   │   ├── 📄 test_mono_16bit_44khz.wav
│   │   ├── 📄 test_stereo_24bit_96khz.flac
│   │   ├── 📄 test_mp3_320kbps.mp3
│   │   ├── 📄 test_ogg_vorbis.ogg
│   │   └── 📄 silent_1sec.wav
│   │
│   ├── 📁 playlists/
│   │   ├── 📄 sample_m3u.m3u
│   │   ├── 📄 sample_pls.pls
│   │   └── 📄 sample_json.json
│   │
│   ├── 📁 metadata/
│   │   ├── 📄 with_artwork.mp3
│   │   ├── 📄 unicode_metadata.flac
│   │   └── 📄 corrupt_metadata.mp3
│   │
│   └── 📁 plugins/
│       ├── 📄 test_visualizer.so
│       ├── 📄 test_effect.dll
│       └── 📄 invalid_plugin.so
│
├── 📁 common/                    # テスト共通処理
│   ├── 📄 mod.rs
│   ├── 📄 fixtures.rs            # テストデータ生成
│   ├── 📄 helpers.rs             # テストヘルパー関数
│   ├── 📄 mocks.rs               # モックオブジェクト
│   ├── 📄 assertions.rs          # カスタムアサーション
│   └── 📄 setup.rs               # テスト環境セットアップ
│
└── 📁 e2e/                       # End-to-Endテスト
    ├── 📄 complete_workflow_test.rs    # 完全ワークフローテスト
    ├── 📄 stress_test.rs              # ストレステスト
    └── 📄 regression_test.rs          # リグレッションテスト
```

### 単体テスト配置

```rust
// src/audio/engine.rs内にテストを配置
impl AudioEngine {
    // 実装...
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_common::*;

    #[test]
    fn test_engine_initialization() {
        let config = AudioConfig::test_default();
        let engine = AudioEngine::new(config).unwrap();
        assert_eq!(engine.state(), EngineState::Initialized);
    }

    #[tokio::test]
    async fn test_track_loading() {
        let mut engine = AudioEngine::new_with_mock();
        let test_track = create_test_track();

        engine.load_track(test_track).await.unwrap();
        assert!(engine.is_track_loaded());
    }

    #[test]
    fn test_volume_control() {
        let mut engine = AudioEngine::new_mock();

        engine.set_volume(0.5).unwrap();
        assert_eq!(engine.volume(), 0.5);

        engine.set_volume(1.5).unwrap_err(); // 範囲外エラー
    }
}
```

### ベンチマーク構成

```
benches/
├── 📄 audio_processing.rs        # 音声処理性能測定
├── 📄 fft_performance.rs         # FFT処理性能測定
├── 📄 visualizer_render.rs       # 描画性能測定
├── 📄 memory_usage.rs            # メモリ使用量測定
├── 📄 plugin_loading.rs          # プラグインロード性能
├── 📄 database_operations.rs     # データベース操作性能
└── 📄 file_scanning.rs           # ファイルスキャン性能
```

```rust
// benches/audio_processing.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};

fn benchmark_audio_decoding(c: &mut Criterion) {
    let mut group = c.benchmark_group("audio_decoding");

    for format in ["mp3", "flac", "wav"].iter() {
        let test_file = format!("tests/fixtures/audio/test.{}", format);
        let decoder = create_decoder_for_format(format);

        group.bench_with_input(
            BenchmarkId::new("decode_frame", format),
            &test_file,
            |b, path| {
                let data = std::fs::read(path).unwrap();
                b.iter(|| {
                    decoder.decode_frame(black_box(&data))
                });
            },
        );
    }

    group.finish();
}

fn benchmark_spectrum_analysis(c: &mut Criterion) {
    let mut group = c.benchmark_group("spectrum_analysis");

    for &size in [512, 1024, 2048, 4096].iter() {
        group.bench_with_input(
            BenchmarkId::new("fft", size),
            &size,
            |b, &size| {
                let mut analyzer = SpectrumAnalyzer::new(size);
                let test_data = generate_test_audio(size);

                b.iter(|| {
                    analyzer.analyze(black_box(&test_data))
                });
            },
        );
    }

    group.finish();
}

criterion_group!(benches, benchmark_audio_decoding, benchmark_spectrum_analysis);
criterion_main!(benches);
```

## ビルドとデプロイメント

### スクリプト構成

```
scripts/
├── 📄 build.sh                   # ユニバーサルビルドスクリプト
├── 📄 test.sh                    # テスト実行スクリプト
├── 📄 benchmark.sh               # ベンチマーク実行
├── 📄 format.sh                  # コードフォーマット
├── 📄 lint.sh                    # 静的解析実行
├── 📄 release.sh                 # リリースビルド
├── 📄 package.sh                 # パッケージング
├── 📄 deploy.sh                  # デプロイメント
└── 📄 clean.sh                   # クリーンアップ
```

### パッケージ構成

```
packages/
├── 📁 windows/                   # Windows用パッケージング
│   ├── 📄 installer.nsi          # NSISインストーラー
│   ├── 📄 wix.wxs                # WiX Toolsetファイル
│   ├── 📄 manifest.xml           # アプリマニフェスト
│   └── 📁 assets/                # アイコン等
│
├── 📁 macos/                     # macOS用パッケージング
│   ├── 📄 Info.plist             # アプリ情報
│   ├── 📄 create_dmg.sh          # DMG作成スクリプト
│   ├── 📄 codesign.sh            # コード署名
│   └── 📁 Resources/             # アプリリソース
│
├── 📁 linux/                     # Linux用パッケージング
│   ├── 📄 appimage.yml           # AppImage設定
│   ├── 📄 flatpak.yml            # Flatpak設定
│   ├── 📄 snap.yaml              # Snap設定
│   ├── 📄 debian.control         # Debian package制御
│   └── 📄 rpm.spec               # RPM specファイル
│
└── 📁 universal/                 # 共通パッケージング
    ├── 📄 README.md              # リリースノート
    ├── 📄 LICENSE                # ライセンス
    ├── 📄 CHANGELOG.md           # 変更履歴
    └── 📁 docs/                  # ドキュメント
```

### プラグイン開発用テンプレート

```
plugins/
├── 📁 template/                  # プラグイン開発テンプレート
│   ├── 📄 Cargo.toml
│   ├── 📄 src/
│   │   ├── 📄 lib.rs
│   │   └── 📄 visualizer.rs
│   ├── 📄 README.md
│   └── 📄 examples/
│
├── 📁 spectrum_bars/             # 標準プラグイン例
│   ├── 📄 Cargo.toml
│   └── 📄 src/
│
└── 📁 sdk/                       # プラグイン開発SDK
    ├── 📄 Cargo.toml
    ├── 📄 src/
    │   ├── 📄 lib.rs
    │   ├── 📄 api.rs
    │   ├── 📄 macros.rs
    │   └── 📄 utils.rs
    ├── 📄 examples/
    └── 📄 docs/
```

### 開発ツール

```
tools/
├── 📄 plugin_validator.rs        # プラグイン検証ツール
├── 📄 metadata_extractor.rs      # メタデータ抽出ツール
├── 📄 config_migrator.rs         # 設定マイグレーションツール
├── 📄 performance_profiler.rs    # パフォーマンスプロファイラー
├── 📄 audio_analyzer.rs          # 音声解析ツール
├── 📄 theme_generator.rs         # テーマ生成ツール
└── 📄 localization_helper.rs     # 多言語化支援ツール
```

### 静的リソース構成

```
assets/
├── 📁 themes/                    # テーマアセット
│   ├── 📁 dark/
│   │   ├── 📄 colors.json
│   │   ├── 📄 typography.json
│   │   ├── 📄 spacing.json
│   │   └── 📁 images/
│   │       ├── 📄 background.png
│   │       └── 📄 patterns/
│   ├── 📁 light/
│   ├── 📁 high_contrast/
│   └── 📁 neon/
│
├── 📁 icons/                     # アイコンセット
│   ├── 📁 svg/                   # ベクター形式
│   │   ├── 📄 play.svg
│   │   ├── 📄 pause.svg
│   │   ├── 📄 stop.svg
│   │   ├── 📄 next.svg
│   │   ├── 📄 previous.svg
│   │   ├── 📄 shuffle.svg
│   │   ├── 📄 repeat.svg
│   │   ├── 📄 volume.svg
│   │   └── 📄 settings.svg
│   │
│   ├── 📁 png/                   # ラスター形式
│   │   ├── 📄 app_icon_16.png
│   │   ├── 📄 app_icon_32.png
│   │   ├── 📄 app_icon_64.png
│   │   ├── 📄 app_icon_128.png
│   │   ├── 📄 app_icon_256.png
│   │   └── 📄 app_icon_512.png
│   │
│   └── 📁 ico/                   # Windows用アイコン
│       └── 📄 app_icon.ico
│
├── 📁 fonts/                     # フォント
│   ├── 📄 inter-regular.woff2    # UI用フォント
│   ├── 📄 inter-bold.woff2
│   ├── 📄 fira_code-regular.woff2 # モノスペース
│   └── 📄 noto_sans_cjk.woff2   # 多言語対応
│
├── 📁 sounds/                    # システムサウンド
│   ├── 📄 notification.wav
│   ├── 📄 error.wav
│   ├── 📄 success.wav
│   └── 📄 warning.wav
│
├── 📁 shaders/                   # GPU シェーダー
│   ├── 📄 spectrum.vert
│   ├── 📄 spectrum.frag
│   ├── 📄 particle.compute
│   └── 📄 blur.frag
│
└── 📁 localization/              # 多言語化ファイル
    ├── 📄 en.json                # 英語
    ├── 📄 ja.json                # 日本語
    ├── 📄 zh.json                # 中国語
    ├── 📄 ko.json                # 韓国語
    └── 📄 template.json          # テンプレート
```

## 品質管理と CI/CD

### ドキュメント構成

```
docs/
├── 📄 CLAUDE.md                  # Claude用文脈ファイル
├── 📄 ARCHITECTURE.md            # アーキテクチャドキュメント
├── 📄 SPECIFICATION.md           # 機能仕様書
├── 📄 DIRECTORY.md              # このファイル
├── 📄 SYSTEM.md                 # システム設計詳細
│
├── 📁 api/                       # API ドキュメント
│   ├── 📄 audio_engine.md
│   ├── 📄 visualizer_api.md
│   ├── 📄 plugin_development.md
│   └── 📄 rust_docs/            # cargo doc 出力
│
├── 📁 user_guide/               # ユーザーガイド
│   ├── 📄 installation.md
│   ├── 📄 basic_usage.md
│   ├── 📄 advanced_features.md
│   ├── 📄 troubleshooting.md
│   └── 📄 keyboard_shortcuts.md
│
├── 📁 developer_guide/          # 開発者ガイド
│   ├── 📄 getting_started.md
│   ├── 📄 building.md
│   ├── 📄 contributing.md
│   ├── 📄 plugin_development.md
│   ├── 📄 testing.md
│   └── 📄 releasing.md
│
└── 📁 design/                   # 設計ドキュメント
    ├── 📄 ui_mockups/
    ├── 📄 wireframes/
    ├── 📄 design_system.md
    └── 📄 accessibility.md
```

### CI/CD ファイル構成

```
.github/
├── 📁 workflows/
│   ├── 📄 ci.yml                # 継続的インテグレーション
│   ├── 📄 release.yml           # リリースワークフロー
│   ├── 📄 security.yml          # セキュリティチェック
│   ├── 📄 docs.yml              # ドキュメント生成
│   └── 📄 performance.yml       # パフォーマンステスト
│
├── 📁 ISSUE_TEMPLATE/
│   ├── 📄 bug_report.md
│   ├── 📄 feature_request.md
│   └── 📄 plugin_request.md
│
└── 📄 PULL_REQUEST_TEMPLATE.md
```

---

**最終更新**: 2025-08-12  
**バージョン**: 2.0  
**管理者**: Claude (ディレクトリ構成再整理)
