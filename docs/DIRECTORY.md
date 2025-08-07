# Resonance Player - ディレクトリ構成ガイド

## 📋 目次

1. [プロジェクト構造概要](#プロジェクト構造概要)
2. [ソースコード構成](#ソースコード構成)
3. [モジュール詳細](#モジュール詳細)
4. [ファイル配置ルール](#ファイル配置ルール)
5. [テスト構成](#テスト構成)

## プロジェクト構造概要

```
resonance-player/
├── 📁 src/                    # ソースコード
├── 📁 tests/                  # テスト
├── 📁 benches/                # ベンチマーク
├── 📁 examples/               # 使用例
├── 📁 docs/                   # ドキュメント
├── 📁 assets/                 # 静的リソース
├── 📁 scripts/                # ビルドスクリプト
├── 📄 Cargo.toml              # プロジェクト設定
├── 📄 README.md
└── 📄 CLAUDE.md               # Claude用文脈ファイル
```

## ソースコード構成

### レイヤー別構成

```
src/
├── 📄 main.rs                 # アプリケーションエントリーポイント
├── 📄 lib.rs                  # ライブラリルート（公開API定義）
│
├── 🎨 UI Layer (Slint)
│   └── 📁 ui/
│       ├── 📄 main_window.slint
│       ├── 📁 components/
│       ├── 📁 themes/
│       └── 📄 bindings.rs
│
├── 🎛️ Application Layer
│   └── 📁 app/
│       ├── 📄 controller.rs
│       ├── 📄 state.rs
│       ├── 📄 events.rs
│       └── 📄 lifecycle.rs
│
├── 🏭 Business Logic Layer
│   ├── 📁 audio/              # 音声処理
│   ├── 📁 visualizer/         # ビジュアライザー
│   ├── 📁 playlist/           # プレイリスト
│   └── 📁 library/            # ライブラリ管理
│
└── 🔧 Infrastructure Layer
    ├── 📁 config/             # 設定管理
    ├── 📁 plugin/             # プラグイン
    ├── 📁 utils/              # ユーティリティ
    └── 📁 error/              # エラー処理
```

## モジュール詳細

### 🎵 Audio モジュール (`src/audio/`)

```
audio/
├── 📄 mod.rs                  # モジュール公開API
├── 📄 engine.rs               # メインオーディオエンジン
├── 📄 renderer.rs             # 音声レンダリング
│
├── 📁 decoder/                # デコーダー実装
│   ├── 📄 mod.rs
│   ├── 📄 mp3.rs             # MP3デコーダー
│   ├── 📄 flac.rs            # FLACデコーダー
│   ├── 📄 wav.rs             # WAVデコーダー
│   └── 📄 ogg.rs             # OGGデコーダー
│
├── 📁 effects/                # 音響効果
│   ├── 📄 mod.rs
│   ├── 📄 equalizer.rs       # イコライザー
│   ├── 📄 reverb.rs          # リバーブ
│   └── 📄 crossfade.rs       # クロスフェード
│
└── 📁 analysis/               # 音声解析
    ├── 📄 mod.rs
    ├── 📄 fft.rs             # FFT処理
    ├── 📄 spectrum.rs        # スペクトラム解析
    └── 📄 meter.rs           # レベルメーター
```

**責務**: 音声ファイルの読み込み、デコード、再生、音響効果処理、周波数解析

**主要な型**:

```rust
// src/audio/mod.rs
pub use engine::AudioEngine;
pub use decoder::{AudioDecoder, DecoderError};
pub use effects::{Equalizer, EffectsChain};
pub use analysis::{SpectrumAnalyzer, SpectrumData};
```

### 🎨 Visualizer モジュール (`src/visualizer/`)

```
visualizer/
├── 📄 mod.rs                  # ビジュアライザー公開API
├── 📄 engine.rs               # ビジュアライザーエンジン
├── 📄 traits.rs               # Visualizer trait定義
├── 📄 renderer.rs             # 共通描画処理
├── 📄 config.rs               # 設定管理
│
└── 📁 plugins/                # ビジュアライザープラグイン
    ├── 📄 mod.rs
    ├── 📄 spectrum_bars.rs    # スペクトラムバー
    ├── 📄 waveform.rs         # 波形表示
    ├── 📄 circle_spectrum.rs  # サークルスペクトラム
    ├── 📄 particle_system.rs  # パーティクルシステム
    ├── 📄 spectrum_3d.rs      # 3Dスペクトラム
    └── 📄 vu_meters.rs        # VUメーター
```

**責務**: 音圧データの視覚化、プラグイン管理、描画処理

**主要な型**:

```rust
// src/visualizer/mod.rs
pub use engine::VisualizerEngine;
pub use traits::{Visualizer, VisualizationConfig};
pub use plugins::*;
```

### 📄 UI モジュール (`src/ui/`)

```
ui/
├── 📄 mod.rs                  # UI関連の型定義
├── 📄 bindings.rs             # Rust-Slintバインディング
│
├── 📄 main_window.slint       # メインウィンドウ
│
├── 📁 components/             # 再利用可能コンポーネント
│   ├── 📄 player_controls.slint    # 再生コントロール
│   ├── 📄 playlist_view.slint      # プレイリスト表示
│   ├── 📄 library_browser.slint    # ライブラリブラウザー
│   ├── 📄 visualizer_canvas.slint  # ビジュアライザー描画エリア
│   ├── 📄 settings_panel.slint     # 設定パネル
│   └── 📄 common.slint             # 共通スタイル・コンポーネント
│
└── 📁 themes/                 # テーマ定義
    ├── 📄 dark.slint         # ダークテーマ
    ├── 📄 light.slint        # ライトテーマ
    └── 📄 custom.slint       # カスタムテーマ
```

**責務**: ユーザーインターフェース、ユーザー操作の受付、データバインディング

### 🎛️ App モジュール (`src/app/`)

```
app/
├── 📄 mod.rs                  # アプリケーション制御API
├── 📄 controller.rs           # メインアプリケーションコントローラー
├── 📄 state.rs               # アプリケーション状態管理
├── 📄 events.rs              # イベント処理
└── 📄 lifecycle.rs           # アプリケーションライフサイクル
```

**責務**: UI-ビジネスロジック仲介、アプリケーション状態管理、イベント処理

### 📋 Playlist モジュール (`src/playlist/`)

```
playlist/
├── 📄 mod.rs                  # プレイリスト公開API
├── 📄 manager.rs              # プレイリストマネージャー
├── 📄 shuffle.rs              # シャッフルアルゴリズム
│
└── 📁 formats/                # プレイリスト形式対応
    ├── 📄 mod.rs
    ├── 📄 m3u.rs             # M3U形式
    ├── 📄 pls.rs             # PLS形式
    └── 📄 json.rs            # JSON形式（独自）
```

### 📚 Library モジュール (`src/library/`)

```
library/
├── 📄 mod.rs                  # ライブラリ公開API
├── 📄 manager.rs              # ライブラリマネージャー
├── 📄 scanner.rs              # ファイルスキャン
├── 📄 metadata.rs             # メタデータ処理
├── 📄 database.rs             # データベース操作（SQLite）
└── 📄 artwork.rs              # アルバムアートワーク処理
```

### 🔌 Plugin モジュール (`src/plugin/`)

```
plugin/
├── 📄 mod.rs                  # プラグイン公開API
├── 📄 manager.rs              # プラグインマネージャー
├── 📄 loader.rs               # 動的ライブラリローダー
├── 📄 api.rs                  # プラグインAPI定義
└── 📄 registry.rs             # プラグインレジストリ
```

### ⚙️ Config モジュール (`src/config/`)

```
config/
├── 📄 mod.rs                  # 設定公開API
├── 📄 manager.rs              # 設定マネージャー
├── 📄 schema.rs               # 設定スキーマ定義
├── 📄 defaults.rs             # デフォルト設定値
└── 📄 migration.rs            # 設定マイグレーション
```

### 🛠️ Utils モジュール (`src/utils/`)

```
utils/
├── 📄 mod.rs                  # ユーティリティ公開API
├── 📄 file.rs                # ファイル操作
├── 📄 math.rs                # 数学関数
├── 📄 color.rs               # カラー処理
├── 📄 animation.rs           # アニメーション
└── 📄 platform.rs            # プラットフォーム固有処理
```

### ❌ Error モジュール (`src/error/`)

```
error/
├── 📄 mod.rs                  # エラー公開API
├── 📄 types.rs               # エラー型定義
└── 📄 handling.rs            # エラーハンドリング
```

## ファイル配置ルール

### 1. モジュール構造ルール

```rust
// 各モジュールのmod.rsは公開APIのみ
// src/audio/mod.rs
pub use engine::AudioEngine;
pub use decoder::{AudioDecoder, DecoderResult};
pub use analysis::{SpectrumAnalyzer, SpectrumData};

// 内部実装詳細は非公開
mod engine;
mod decoder;
mod analysis;
```

### 2. ファイル命名規則

- **構造体中心**: `engine.rs`, `manager.rs`, `analyzer.rs`
- **機能中心**: `playback.rs`, `effects.rs`, `rendering.rs`
- **データ中心**: `metadata.rs`, `config.rs`, `types.rs`

### 3. トレイト定義の配置

```rust
// トレイトは独立したファイルまたはtraits.rsに配置
// src/audio/traits.rs
pub trait AudioDecoder {
    fn decode(&self, data: &[u8]) -> Result<AudioData, DecoderError>;
}

pub trait AudioRenderer {
    fn render(&mut self, buffer: &mut [f32]) -> Result<(), RenderError>;
}
```

## テスト構成

### テストディレクトリ構造

```
tests/
├── 📁 integration/            # 結合テスト
│   ├── 📄 audio_engine_test.rs
│   ├── 📄 visualizer_test.rs
│   ├── 📄 playlist_test.rs
│   └── 📄 app_controller_test.rs
│
├── 📁 unit/                   # 単体テスト（不要、src内に配置）
│
├── 📁 fixtures/               # テスト用データ
│   ├── 📁 audio/
│   │   ├── 📄 test.mp3
│   │   ├── 📄 test.flac
│   │   └── 📄 test.wav
│   └── 📁 playlists/
│       ├── 📄 sample.m3u
│       └── 📄 sample.json
│
└── 📁 common/                 # テスト共通処理
    ├── 📄 mod.rs
    ├── 📄 fixtures.rs         # テストデータ生成
    └── 📄 helpers.rs          # テストヘルパー関数
```

### 単体テスト配置

```rust
// src/audio/engine.rs内にテストを配置
impl AudioEngine {
    // 実装
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_playback() {
        // テスト実装
    }
}
```

### ベンチマーク構成

```
benches/
├── 📄 audio_processing.rs     # 音声処理性能測定
├── 📄 fft_performance.rs      # FFT処理性能測定
├── 📄 visualizer_render.rs    # 描画性能測定
└── 📄 memory_usage.rs         # メモリ使用量測定
```

## ドキュメント構成

```
docs/
├── 📄 CLAUDE.md               # Claude用文脈ファイル
├── 📄 ARCHITECTURE.md         # アーキテクチャドキュメント
├── 📄 SPECIFICATION.md        # 機能仕様書
├── 📄 DIRECTORY.md           # このファイル
│
├── 📁 api/                    # API ドキュメント
│   ├── 📄 audio_engine.md
│   ├── 📄 visualizer_api.md
│   └── 📄 plugin_development.md
│
├── 📁 user_guide/            # ユーザーガイド
│   ├── 📄 installation.md
│   ├── 📄 basic_usage.md
│   └── 📄 configuration.md
│
└── 📁 developer_guide/       # 開発者ガイド
    ├── 📄 setup.md
    ├── 📄 contributing.md
    └── 📄 plugin_development.md
```

## 静的リソース構成

```
assets/
├── 📁 themes/                # テーマアセット
│   ├── 📁 dark/
│   │   ├── 📄 colors.json
│   │   └── 📁 images/
│   ├── 📁 light/
│   └── 📁 custom/
│
├── 📁 icons/                 # アイコン
│   ├── 📁 svg/              # ベクター形式
│   │   ├── 📄 play.svg
│   │   ├── 📄 pause.svg
│   │   └── 📄 stop.svg
│   └── 📁 png/              # ラスター形式
│       ├── 📄 app_icon_16.png
│       ├── 📄 app_icon_32.png
│       └── 📄 app_icon_256.png
│
├── 📁 fonts/                 # フォント
│   ├── 📄 inter.woff2       # UI用フォント
│   └── 📄 fira_code.woff2   # モノスペース
│
└── 📁 sounds/                # システムサウンド
    ├── 📄 notification.wav
    └── 📄 error.wav
```

---

**最終更新**: 2025-08-07  
**バージョン**: 1.0  
**管理者**: プロジェクトアーキテクト
