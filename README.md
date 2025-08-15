# Sonic Flow

A high-quality music player with advanced audio spectrum visualizers, built with Rust and Slint.

## ✨ Features

### 🎵 Audio Playback

- **High-Quality Audio**: Support for FLAC, WAV, MP3, OGG, and AAC formats
- **Bit-Perfect Playback**: Up to 24-bit/192kHz audio support
- **Low Latency**: Sub-50ms audio latency for responsive playback
- **Advanced Effects**: 10-band equalizer, reverb, 3D audio processing

### 🎨 Advanced Visualizers

- **Spectrum Bars**: Classic frequency domain visualization
- **Waveform Display**: Time domain oscilloscope view
- **Circle Spectrum**: Circular frequency visualization
- **Particle System**: Dynamic particle-based visualization
- **3D Spectrum**: Three-dimensional frequency landscape
- **VU Meters**: Professional-style level meters

### 🔧 Extensibility

- **Plugin System**: Load custom visualizers at runtime
- **Theme Support**: Dark, light, and custom themes
- **Configurable**: Extensive customization options
- **Cross-Platform**: Windows, macOS, and Linux support

## 🚀 Getting Started

### Prerequisites

- Rust 1.70.0 or later
- Git

### Installation

#### From Source

```bash
git clone https://github.com/sonic-flow/sonic-flow.git
cd sonic-flow
cargo build --release
```

#### From Crates.io (Coming Soon)

```bash
cargo install sonic-flow
```

### Quick Start

```bash
# Run the application
cargo run

# Or run the release build
./target/release/sonic-flow
```

## 🏗️ Architecture

Sonic Flow follows a layered architecture:

```
┌─────────────────────────────────────────────┐
│                UI Layer (Slint)             │
├─────────────────────────────────────────────┤
│            Application Layer                │
├─────────────────────────────────────────────┤
│           Business Logic Layer              │
├─────────────────────────────────────────────┤
│          Infrastructure Layer               │
└─────────────────────────────────────────────┘
```

## Project structure

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
