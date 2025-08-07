# Resonance Player - 機能仕様書

## 📋 目次

1. [プロダクト概要](#プロダクト概要)
2. [機能要件](#機能要件)
3. [非機能要件](#非機能要件)
4. [ユーザーインターフェース仕様](#ユーザーインターフェース仕様)
5. [音圧ビジュアライザー仕様](#音圧ビジュアライザー仕様)
6. [API 仕様](#api仕様)
7. [データ仕様](#データ仕様)
8. [プラットフォーム要件](#プラットフォーム要件)

## プロダクト概要

### 製品名

**Resonance Player** - 音圧ビジュアライザー特化型ミュージックプレイヤー

### 製品ビジョン

"音楽を聴く体験から、音楽を視る体験へ" - 高品質な音圧ビジュアライザーにより、音楽をより深く感じられるプレイヤー

### ターゲットユーザー

- **音楽愛好家**: 高音質での音楽視聴を求めるユーザー
- **DJ とミュージシャン**: 音楽分析機能を必要とするプロ
- **開発者**: カスタマイズ可能なプレイヤーを求める技術者

### 競合優位性

1. **リアルタイム高精度 FFT 解析**: 業界最高レベルの音圧解析
2. **拡張可能ビジュアライザー**: プラグインシステムによる無限の拡張性
3. **ハイパフォーマンス**: Rust 製による最適化されたパフォーマンス

## 機能要件

### F1. 音楽再生機能

#### F1.1 基本再生機能

| 機能 ID | 機能名        | 詳細                         | 優先度 |
| ------- | ------------- | ---------------------------- | ------ |
| F1.1.1  | 再生/一時停止 | ワンクリックでの再生制御     | 必須   |
| F1.1.2  | 停止          | 再生を完全停止し、先頭に戻る | 必須   |
| F1.1.3  | 次のトラック  | プレイリスト内の次曲に移動   | 必須   |
| F1.1.4  | 前のトラック  | プレイリスト内の前曲に移動   | 必須   |
| F1.1.5  | シーク        | 任意の時間位置への移動       | 必須   |

```rust
// 実装例
pub trait PlaybackControl {
    async fn play(&mut self) -> Result<(), PlaybackError>;
    async fn pause(&mut self) -> Result<(), PlaybackError>;
    async fn stop(&mut self) -> Result<(), PlaybackError>;
    async fn seek(&mut self, position: Duration) -> Result<(), PlaybackError>;
    async fn next_track(&mut self) -> Result<(), PlaybackError>;
    async fn previous_track(&mut self) -> Result<(), PlaybackError>;
}
```

#### F1.2 音量制御

| 機能 ID | 機能名   | 詳細                     | 優先度 |
| ------- | -------- | ------------------------ | ------ |
| F1.2.1  | 音量調整 | 0-100%の連続音量制御     | 必須   |
| F1.2.2  | ミュート | 一時的な音声カット       | 必須   |
| F1.2.3  | 音量記憶 | アプリ再起動時の音量復元 | 高     |

#### F1.3 対応音声フォーマット

| フォーマット | ビット深度 | サンプリングレート | 優先度 |
| ------------ | ---------- | ------------------ | ------ |
| MP3          | 16bit      | 44.1kHz-48kHz      | 必須   |
| FLAC         | 16/24bit   | 44.1kHz-192kHz     | 必須   |
| WAV          | 16/24bit   | 44.1kHz-192kHz     | 必須   |
| OGG Vorbis   | 16bit      | 44.1kHz-48kHz      | 高     |
| AAC          | 16bit      | 44.1kHz-48kHz      | 中     |

### F2. プレイリスト管理

#### F2.1 プレイリスト操作

```rust
pub struct Playlist {
    id: PlaylistId,
    name: String,
    tracks: Vec<TrackInfo>,
    created_at: DateTime<Utc>,
    modified_at: DateTime<Utc>,
}

pub trait PlaylistManager {
    async fn create_playlist(&mut self, name: &str) -> Result<PlaylistId, PlaylistError>;
    async fn add_track(&mut self, playlist_id: PlaylistId, track: TrackInfo) -> Result<(), PlaylistError>;
    async fn remove_track(&mut self, playlist_id: PlaylistId, track_id: TrackId) -> Result<(), PlaylistError>;
    async fn reorder_tracks(&mut self, playlist_id: PlaylistId, from: usize, to: usize) -> Result<(), PlaylistError>;
}
```

#### F2.2 再生モード

| モード     | 詳細                   | 実装                      |
| ---------- | ---------------------- | ------------------------- |
| 順次再生   | プレイリスト順での再生 | デフォルト                |
| シャッフル | ランダム順での再生     | Fisher-Yates アルゴリズム |
| リピート   | 全体/単曲リピート      | 設定可能                  |

### F3. 音楽ライブラリ管理

#### F3.1 メタデータ管理

```rust
#[derive(Debug, Clone)]
pub struct TrackInfo {
    pub id: TrackId,
    pub file_path: PathBuf,
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub genre: Option<String>,
    pub year: Option<i32>,
    pub track_number: Option<u32>,
    pub duration: Duration,
    pub bitrate: u32,
    pub sample_rate: u32,
    pub artwork: Option<Vec<u8>>, // JPEG/PNG データ
}
```

#### F3.2 ライブラリスキャン

- **自動スキャン**: 指定フォルダの定期監視
- **手動スキャン**: ユーザー操作による即座スキャン
- **増分スキャン**: 変更ファイルのみの効率的更新

### F4. 音圧ビジュアライザー（メイン機能）

#### F4.1 ビジュアライザータイプ

| タイプ               | 詳細                     | 実装方式       |
| -------------------- | ------------------------ | -------------- |
| スペクトラムバー     | 周波数帯域別の縦棒表示   | FFT 1024 点    |
| 波形表示             | 時間軸波形表示           | リングバッファ |
| サークルスペクトラム | 円形配置スペクトラム     | 極座標変換     |
| パーティクルシステム | 音圧連動パーティクル     | GPU 処理       |
| 3D スペクトラム      | 立体的スペクトラム表示   | WebGL          |
| VU メーター          | アナログ風レベルメーター | RMS 計算       |

#### F4.2 ビジュアライザー設定

```rust
#[derive(Debug, Clone)]
pub struct VisualizationConfig {
    pub sensitivity: f32,        // 0.0 - 2.0
    pub frequency_range: (f32, f32), // Hz範囲
    pub color_scheme: ColorScheme,
    pub animation_speed: f32,    // 0.5 - 2.0
    pub smoothing: bool,
    pub auto_gain: bool,
}

pub struct ColorScheme {
    pub primary: Color,
    pub secondary: Color,
    pub gradient: Vec<Color>,
}
```

#### F4.3 リアルタイム処理要件

- **更新頻度**: 60-120 FPS
- **遅延**: 最大 50ms
- **CPU 使用率**: 5%以下（Intel i5-8400 基準）
- **メモリ使用量**: 追加 50MB 以下

### F5. 音響効果機能

#### F5.1 イコライザー

```rust
pub struct Equalizer {
    bands: [EqualizerBand; 10], // 10バンドEQ
}

pub struct EqualizerBand {
    frequency: f32,      // 中心周波数
    gain: f32,          // -12dB to +12dB
    q_factor: f32,      // バンド幅
}

// プリセット
pub enum EqualizerPreset {
    Flat,
    Rock,
    Pop,
    Jazz,
    Classical,
    Electronic,
    Custom(Equalizer),
}
```

#### F5.2 音響効果

- **リバーブ**: 空間系エフェクト、複数ルームタイプ
- **エコー**: ディレイ効果、フィードバック制御
- **3D 音響**: バイノーラル処理による立体音響
- **クロスフェード**: トラック間のスムーズ切り替え（1-10 秒）

### F6. プラグインシステム

#### F6.1 ビジュアライザープラグイン

```rust
// プラグインAPI
pub trait VisualizerPlugin: Send + Sync {
    fn metadata(&self) -> PluginMetadata;
    fn initialize(&mut self, config: &VisualizationConfig) -> Result<(), PluginError>;
    fn render(&mut self, spectrum_data: &SpectrumData, canvas: &mut Canvas) -> Result<(), RenderError>;
    fn configure(&mut self, settings: &HashMap<String, PluginValue>) -> Result<(), ConfigError>;
    fn cleanup(&mut self);
}

#[derive(Debug)]
pub struct PluginMetadata {
    pub name: String,
    pub version: String,
    pub author: String,
    pub description: String,
    pub supported_formats: Vec<String>,
}
```

#### F6.2 プラグイン管理

- **動的ロード**: 実行時プラグイン読み込み
- **ホットリロード**: 開発時の即座反映
- **設定保存**: プラグイン個別設定の永続化
- **エラー処理**: プラグイン障害の分離

## 非機能要件

### NF1. パフォーマンス要件

#### NF1.1 応答性能

| 指標                 | 目標値     | 測定方法              |
| -------------------- | ---------- | --------------------- |
| アプリ起動時間       | 3 秒以内   | プロセス開始〜UI 表示 |
| トラック切り替え     | 0.5 秒以内 | 操作〜音声出力開始    |
| UI 応答時間          | 16ms 以下  | 60fps 保証            |
| ビジュアライザー更新 | 8.3ms 以下 | 120fps 目標           |

#### NF1.2 リソース使用量

| リソース | アイドル時 | 再生時     | ビジュアライザー有効時 |
| -------- | ---------- | ---------- | ---------------------- |
| メモリ   | 50MB 以下  | 100MB 以下 | 150MB 以下             |
| CPU      | 1%以下     | 3%以下     | 5%以下                 |
| GPU      | 0%         | 0%         | 10%以下                |

### NF2. 音質要件

#### NF2.1 音声品質

```rust
pub struct AudioQualitySpecs {
    pub bit_perfect: bool,           // Bit-perfect再生
    pub max_bit_depth: u8,          // 最大24bit
    pub max_sample_rate: u32,       // 最大192kHz
    pub thd_n: f32,                 // THD+N < 0.001%
    pub snr: f32,                   // SNR > 120dB
    pub latency: Duration,          // < 50ms
}
```

#### NF2.2 対応音声規格

- **PCM**: 16/24bit, 44.1/48/88.2/96/176.4/192kHz
- **DSD**: DSD64/128 (将来対応)
- **MQA**: Master Quality Authenticated (将来対応)

### NF3. 可用性要件

#### NF3.1 安定性

- **MTBF**: 平均故障間隔 1000 時間以上
- **クラッシュ率**: 0.1%以下（セッション当たり）
- **メモリリーク**: なし（24 時間連続再生テスト）

#### NF3.2 エラー処理

```rust
#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error("Audio processing failed: {0}")]
    AudioError(#[from] AudioError),

    #[error("File format not supported: {format}")]
    UnsupportedFormat { format: String },

    #[error("Plugin error: {plugin_name} - {message}")]
    PluginError { plugin_name: String, message: String },

    #[error("Configuration error: {0}")]
    ConfigError(#[from] ConfigError),
}

// エラー回復戦略
pub trait ErrorRecovery {
    fn recover_from_audio_error(&mut self) -> Result<(), RecoveryError>;
    fn recover_from_plugin_error(&mut self, plugin_id: &str) -> Result<(), RecoveryError>;
}
```

### NF4. ユーザビリティ要件

#### NF4.1 操作性

- **学習コスト**: 初回使用時 30 分以内で基本操作習得
- **キーボードショートカット**: 全機能対応
- **アクセシビリティ**: スクリーンリーダー対応

#### NF4.2 国際化

```rust
// 多言語対応
pub enum Language {
    Japanese,
    English,
    Korean,
    Chinese,
}

pub trait Localizable {
    fn localize(&self, lang: Language) -> String;
}
```

## ユーザーインターフェース仕様

### UI1. メインウィンドウレイアウト

```
┌─────────────────────────────────────────────────────────────┐
│  Menu Bar                                            [ - □ ✕ ]│
├─────────────────────────────────────────────────────────────┤
│┌─────────────┐┌──────────────────────┐┌──────────────────┐│
││             ││                      ││                  ││
││  Library    ││    Visualizer        ││   Now Playing    ││
││   Browser   ││      Canvas          ││     Info         ││
││             ││                      ││                  ││
││             ││                      ││                  ││
│└─────────────┘└──────────────────────┘└──────────────────┘│
├─────────────────────────────────────────────────────────────┤
│┌───────────────────────────────────────────────────────────┐│
││               Playlist / Queue View                       ││
│└───────────────────────────────────────────────────────────┘│
├─────────────────────────────────────────────────────────────┤
│ ◄◄ ► ❚❚ ►► │████████████░░░░░░░│ 3:24 / 4:18 │ 🔊 ░░░░│
└─────────────────────────────────────────────────────────────┘
```

### UI2. ビジュアライザーコントロール

```rust
// ビジュアライザー制御UI
pub struct VisualizerControls {
    pub visualizer_type: VisualizerType,
    pub sensitivity: f32,           // スライダー
    pub color_theme: ColorTheme,    // ドロップダウン
    pub fullscreen_toggle: bool,    // トグルボタン
    pub settings_button: Button,    // 詳細設定
}

#[derive(Debug, Clone)]
pub enum VisualizerType {
    SpectrumBars,
    Waveform,
    CircleSpectrum,
    ParticleSystem,
    Spectrum3D,
    VUMeters,
    Custom(String), // プラグイン名
}
```

### UI3. テーマシステム

#### UI3.1 カラーパレット

```rust
#[derive(Debug, Clone)]
pub struct ColorTheme {
    pub name: String,
    pub background: Color,
    pub surface: Color,
    pub primary: Color,
    pub secondary: Color,
    pub accent: Color,
    pub text_primary: Color,
    pub text_secondary: Color,
    pub border: Color,
}

// プリセットテーマ
pub const DARK_THEME: ColorTheme = ColorTheme {
    name: "Dark".to_string(),
    background: Color::RGB(18, 18, 18),
    surface: Color::RGB(24, 24, 24),
    primary: Color::RGB(147, 51, 234),  // Purple
    secondary: Color::RGB(59, 130, 246), // Blue
    accent: Color::RGB(34, 197, 94),     // Green
    text_primary: Color::RGB(255, 255, 255),
    text_secondary: Color::RGB(156, 163, 175),
    border: Color::RGB(55, 65, 81),
};
```

#### UI3.2 レスポンシブ設計

- **最小ウィンドウサイズ**: 800x600px
- **推奨サイズ**: 1200x800px
- **4K 対応**: HiDPI/Retina 対応
- **動的レイアウト**: パネル幅可変

## 音圧ビジュアライザー仕様

### V1. FFT 解析エンジン

```rust
pub struct SpectrumAnalyzer {
    fft_size: usize,              // 1024, 2048, 4096
    window_function: WindowType,   // Hann, Blackman, etc.
    overlap_ratio: f32,           // 0.5 - 0.75
    sample_rate: u32,
    frequency_bins: Vec<f32>,
}

impl SpectrumAnalyzer {
    pub fn analyze(&mut self, audio_buffer: &[f32]) -> SpectrumData {
        // 1. 窓関数適用
        let windowed = self.apply_window(audio_buffer);

        // 2. FFT変換
        let fft_result = self.fft.process(&windowed);

        // 3. パワースペクトラム計算
        let power_spectrum = self.calculate_power_spectrum(&fft_result);

        // 4. 周波数ビン統合
        let bands = self.bin_to_bands(&power_spectrum);

        SpectrumData {
            bands,
            peak_level: self.calculate_peak(&power_spectrum),
            rms_level: self.calculate_rms(&power_spectrum),
            timestamp: Instant::now(),
        }
    }
}
```

### V2. ビジュアライザー実装

#### V2.1 スペクトラムバー

```rust
pub struct SpectrumBarsVisualizer {
    config: SpectrumBarsConfig,
    bar_heights: Vec<f32>,
    peak_holds: Vec<f32>,
    fall_speeds: Vec<f32>,
}

#[derive(Debug, Clone)]
pub struct SpectrumBarsConfig {
    pub bar_count: usize,         // 32, 64, 128
    pub bar_width_ratio: f32,     // 0.8 (80% width, 20% gap)
    pub peak_hold_time: Duration, // 500ms
    pub fall_speed: f32,          // pixels/second
    pub logarithmic_scale: bool,
    pub frequency_range: (f32, f32), // 20Hz - 20kHz
}

impl Visualizer for SpectrumBarsVisualizer {
    fn render(&mut self, data: &SpectrumData, canvas: &mut Canvas) -> RenderResult {
        for (i, &amplitude) in data.bands.iter().enumerate() {
            let bar_height = self.calculate_bar_height(amplitude);
            let x = self.calculate_bar_x(i);

            // バー描画
            canvas.draw_rect(
                Rect::new(x, canvas.height() - bar_height, self.bar_width(), bar_height),
                self.config.color_scheme.primary,
            );

            // ピークホールド描画
            if let Some(peak_y) = self.peak_holds.get(i) {
                canvas.draw_line(
                    Point::new(x, *peak_y),
                    Point::new(x + self.bar_width(), *peak_y),
                    self.config.color_scheme.accent,
                );
            }
        }

        self.update_animation();
        Ok(())
    }
}
```

#### V2.2 3D スペクトラム

```rust
pub struct Spectrum3DVisualizer {
    mesh: SpectralMesh,
    camera: Camera3D,
    lighting: LightingConfig,
    history_buffer: RingBuffer<SpectrumData>,
}

impl Visualizer for Spectrum3DVisualizer {
    fn render(&mut self, data: &SpectrumData, canvas: &mut Canvas) -> RenderResult {
        // 1. 履歴バッファ更新
        self.history_buffer.push(data.clone());

        // 2. 3Dメッシュ生成
        let vertices = self.generate_vertices();
        let indices = self.generate_indices();

        // 3. GPU描画
        canvas.render_3d_mesh(
            &vertices,
            &indices,
            &self.camera,
            &self.lighting,
        )
    }
}
```

### V3. パフォーマンス最適化

#### V3.1 GPU 加速

```rust
// GPU処理用のデータ構造
#[repr(C)]
pub struct GpuSpectrumData {
    pub bands: [f32; 128],        // GPU用固定サイズ
    pub time_offset: f32,
    pub amplitude_scale: f32,
}

// シェーダー統合
pub struct VisualizerShader {
    vertex_shader: String,
    fragment_shader: String,
    uniforms: HashMap<String, UniformValue>,
}
```

#### V3.2 並列処理

```rust
// マルチスレッド処理
pub struct ParallelVisualizationEngine {
    fft_thread: JoinHandle<()>,
    render_thread: JoinHandle<()>,
    spectrum_channel: (Sender<SpectrumData>, Receiver<SpectrumData>),
    render_channel: (Sender<RenderCommand>, Receiver<RenderCommand>),
}
```

## API 仕様

### API1. コア API

```rust
// 公開API
pub struct ResonancePlayer {
    engine: AudioEngine,
    visualizer: VisualizerEngine,
    playlist: PlaylistManager,
    config: ConfigManager,
}

impl ResonancePlayer {
    pub fn new() -> Result<Self, PlayerError>;
    pub async fn load_track(&mut self, path: &Path) -> Result<TrackId, PlayerError>;
    pub async fn play(&mut self) -> Result<(), PlayerError>;
    pub async fn pause(&mut self) -> Result<(), PlayerError>;
    pub fn get_current_spectrum(&self) -> Option<SpectrumData>;
    pub fn set_visualizer(&mut self, visualizer_type: VisualizerType) -> Result<(), PlayerError>;
}
```

### API2. プラグイン API

```rust
// プラグイン開発用API
pub trait VisualizerPlugin {
    fn metadata(&self) -> PluginMetadata;
    fn create_instance(&self) -> Box<dyn Visualizer>;
    fn default_config(&self) -> VisualizationConfig;
}

// エクスポートマクロ
#[macro_export]
macro_rules! export_visualizer_plugin {
    ($plugin_type:ty) => {
        #[no_mangle]
        pub extern "C" fn resonance_plugin_create() -> *mut dyn VisualizerPlugin {
            Box::into_raw(Box::new(<$plugin_type>::new()))
        }

        #[no_mangle]
        pub extern "C" fn resonance_plugin_destroy(plugin: *mut dyn VisualizerPlugin) {
            unsafe { Box::from_raw(plugin) };
        }
    };
}
```

## データ仕様

### D1. 設定ファイル形式（TOML）

```toml
# ~/.config/resonance-player/config.toml

[audio]
sample_rate = 44100
buffer_size = 512
bit_depth = 24
device = "default"

[visualizer]
default_type = "spectrum_bars"
fft_size = 2048
update_rate = 60
sensitivity = 1.0

[visualizer.spectrum_bars]
bar_count = 64
logarithmic = true
peak_hold = true
fall_speed = 0.8

[ui]
theme = "dark"
window_width = 1200
window_height = 800
always_on_top = false

[library]
scan_directories = [
    "~/Music",
    "/mnt/music"
]
auto_scan = true
scan_interval = 300  # seconds

[playlists]
default_format = "json"
auto_save = true
```

### D2. プレイリストファイル形式

```json
{
  "version": "1.0",
  "metadata": {
    "name": "My Playlist",
    "created_at": "2025-08-07T12:00:00Z",
    "modified_at": "2025-08-07T12:30:00Z",
    "track_count": 15,
    "total_duration": 3600
  },
  "tracks": [
    {
      "id": "track_001",
      "file_path": "/path/to/music.flac",
      "title": "Song Title",
      "artist": "Artist Name",
      "album": "Album Name",
      "duration": 240,
      "added_at": "2025-08-07T12:00:00Z"
    }
  ]
}
```

### D3. メタデータデータベース（SQLite）

```sql
-- トラック情報テーブル
CREATE TABLE tracks (
    id TEXT PRIMARY KEY,
    file_path TEXT UNIQUE NOT NULL,
    file_hash TEXT,
    title TEXT,
    artist TEXT,
    album TEXT,
    genre TEXT,
    year INTEGER,
    track_number INTEGER,
    duration_ms INTEGER,
    bitrate INTEGER,
    sample_rate INTEGER,
    bit_depth INTEGER,
    file_size INTEGER,
    artwork_hash TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    modified_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    last_played DATETIME,
    play_count INTEGER DEFAULT 0
);

-- プレイリストテーブル
CREATE TABLE playlists (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    modified_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- プレイリスト-トラック関連テーブル
CREATE TABLE playlist_tracks (
    playlist_id TEXT,
    track_id TEXT,
    position INTEGER,
    added_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (playlist_id, track_id),
    FOREIGN KEY (playlist_id) REFERENCES playlists(id),
    FOREIGN KEY (track_id) REFERENCES tracks(id)
);
```

## プラットフォーム要件

### P1. 対応 OS

| OS      | バージョン    | アーキテクチャ | 優先度 |
| ------- | ------------- | -------------- | ------ |
| Windows | 10/11         | x86_64         | 必須   |
| macOS   | 12+           | x86_64, ARM64  | 必須   |
| Linux   | Ubuntu 20.04+ | x86_64         | 必須   |
| Linux   | Arch Linux    | x86_64         | 高     |
| Linux   | Fedora 35+    | x86_64         | 中     |

### P2. システム要件

#### P2.1 最小要件

- **CPU**: 2GHz デュアルコア
- **メモリ**: 4GB RAM
- **ストレージ**: 1GB 空き容量
- **グラフィック**: OpenGL 3.3 対応

#### P2.2 推奨要件

- **CPU**: 3GHz クアッドコア
- **メモリ**: 8GB RAM
- **ストレージ**: SSD 2GB 空き容量
- **グラフィック**: 専用 GPU (NVIDIA GTX 1060 / AMD RX 580 相当)

### P3. 依存関係

```toml
# Cargo.toml dependencies
[dependencies]
slint = "1.0"
tokio = { version = "1.0", features = ["full"] }
rodio = "0.17"           # オーディオ再生
symphonia = "0.5"        # オーディオデコード
rustfft = "6.0"         # FFT処理
sqlite = "0.26"         # データベース
serde = "1.0"           # シリアライゼーション
toml = "0.7"            # 設定ファイル
anyhow = "1.0"          # エラーハンドリング
tracing = "0.1"         # ロギング

[target.'cfg(windows)'.dependencies]
winapi = "0.3"          # Windows API

[target.'cfg(target_os = "macos")'.dependencies]
core-audio = "0.11"     # Core Audio

[target.'cfg(unix)'.dependencies]
alsa = "0.7"            # ALSA (Linux)
```

---

**文書バージョン**: 1.0  
**最終更新**: 2025-08-07  
**承認者**: プロダクトマネージャー  
**次回レビュー**: Phase 1 実装完了時
