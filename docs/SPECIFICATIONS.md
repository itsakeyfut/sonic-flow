# Sonic Flow - 機能仕様書

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

**Sonic Flow** - 音圧ビジュアライザー特化型ミュージックプレイヤー

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
4. **型安全な設計**: コンパイル時エラー検出による堅牢性

## 機能要件

### F1. 音楽再生機能

#### F1.1 基本再生機能

| 機能 ID | 機能名         | 詳細                         | 優先度 |
| ------- | -------------- | ---------------------------- | ------ |
| F1.1.1  | 再生/一時停止  | ワンクリックでの再生制御     | 必須   |
| F1.1.2  | 停止           | 再生を完全停止し、先頭に戻る | 必須   |
| F1.1.3  | 次のトラック   | プレイリスト内の次曲に移動   | 必須   |
| F1.1.4  | 前のトラック   | プレイリスト内の前曲に移動   | 必須   |
| F1.1.5  | シーク         | 任意の時間位置への移動       | 必須   |
| F1.1.6  | クロスフェード | トラック間のスムーズ遷移     | 高     |

```rust
// 実装API例
pub trait PlaybackController: Send + Sync {
    async fn play(&mut self) -> Result<(), PlaybackError>;
    async fn pause(&mut self) -> Result<(), PlaybackError>;
    async fn stop(&mut self) -> Result<(), PlaybackError>;
    async fn seek(&mut self, position: Duration) -> Result<(), PlaybackError>;
    async fn next_track(&mut self) -> Result<(), PlaybackError>;
    async fn previous_track(&mut self) -> Result<(), PlaybackError>;
    async fn set_crossfade_duration(&mut self, duration: Duration) -> Result<(), PlaybackError>;
}
```

#### F1.2 音量制御

| 機能 ID | 機能名             | 詳細                       | 優先度 |
| ------- | ------------------ | -------------------------- | ------ |
| F1.2.1  | 音量調整           | 0-100%の連続音量制御       | 必須   |
| F1.2.2  | ミュート           | 一時的な音声カット         | 必須   |
| F1.2.3  | 音量記憶           | アプリ再起動時の音量復元   | 高     |
| F1.2.4  | チャンネルバランス | L/R チャンネルバランス調整 | 中     |

#### F1.3 対応音声フォーマット

| フォーマット | ビット深度  | サンプリングレート | 優先度 | 実装ライブラリ |
| ------------ | ----------- | ------------------ | ------ | -------------- |
| MP3          | 16bit       | 44.1kHz-48kHz      | 必須   | symphonia      |
| FLAC         | 16/24bit    | 44.1kHz-192kHz     | 必須   | symphonia      |
| WAV          | 16/24/32bit | 44.1kHz-192kHz     | 必須   | symphonia      |
| OGG Vorbis   | 16bit       | 44.1kHz-48kHz      | 高     | symphonia      |
| AAC/M4A      | 16bit       | 44.1kHz-48kHz      | 中     | symphonia      |
| OPUS         | 16bit       | 48kHz              | 中     | symphonia      |

### F2. プレイリスト管理

#### F2.1 プレイリスト操作

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Playlist {
    pub id: PlaylistId,
    pub name: String,
    pub description: Option<String>,
    pub tracks: Vec<TrackInfo>,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub total_duration: Duration,
    pub track_count: usize,
}

pub trait PlaylistManager: Send + Sync {
    async fn create_playlist(&mut self, name: &str, description: Option<&str>)
        -> Result<PlaylistId, PlaylistError>;
    async fn delete_playlist(&mut self, playlist_id: PlaylistId)
        -> Result<(), PlaylistError>;
    async fn add_track(&mut self, playlist_id: PlaylistId, track: TrackInfo)
        -> Result<(), PlaylistError>;
    async fn remove_track(&mut self, playlist_id: PlaylistId, track_id: TrackId)
        -> Result<(), PlaylistError>;
    async fn reorder_tracks(&mut self, playlist_id: PlaylistId, from: usize, to: usize)
        -> Result<(), PlaylistError>;
    async fn duplicate_playlist(&mut self, playlist_id: PlaylistId, new_name: &str)
        -> Result<PlaylistId, PlaylistError>;
}
```

#### F2.2 再生モード

| モード       | 詳細                       | 実装アルゴリズム          |
| ------------ | -------------------------- | ------------------------- |
| 順次再生     | プレイリスト順での再生     | インデックスベース        |
| シャッフル   | ランダム順での再生         | Fisher-Yates アルゴリズム |
| リピート全体 | プレイリスト全体のリピート | 循環インデックス          |
| リピート単曲 | 現在のトラックのリピート   | 固定インデックス          |
| ランダム     | 完全ランダム（重複あり）   | 重み付きランダム選択      |

#### F2.3 スマートプレイリスト

```rust
#[derive(Debug, Clone)]
pub enum PlaylistCriterion {
    Genre(String),
    Artist(String),
    Album(String),
    Year(RangeInclusive<i32>),
    Duration(RangeInclusive<Duration>),
    PlayCount(RangeInclusive<u32>),
    Rating(RangeInclusive<u8>),
    LastPlayed(RangeInclusive<DateTime<Utc>>),
    FileFormat(AudioFormat),
    Custom(Box<dyn Fn(&TrackInfo) -> bool + Send + Sync>),
}

pub struct SmartPlaylist {
    pub criteria: Vec<PlaylistCriterion>,
    pub logic: LogicalOperator, // AND, OR
    pub auto_update: bool,
    pub max_tracks: Option<usize>,
}
```

### F3. 音楽ライブラリ管理

#### F3.1 メタデータ管理

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackInfo {
    pub id: TrackId,
    pub file_path: PathBuf,
    pub file_hash: String, // SHA-256

    // 基本メタデータ
    pub title: Option<String>,
    pub artist: Option<String>,
    pub album: Option<String>,
    pub album_artist: Option<String>,
    pub genre: Option<String>,
    pub year: Option<i32>,
    pub track_number: Option<u32>,
    pub disc_number: Option<u32>,

    // 技術情報
    pub duration: Duration,
    pub bitrate: u32,
    pub sample_rate: u32,
    pub bit_depth: Option<u8>,
    pub channels: u8,
    pub file_size: u64,

    // アートワーク
    pub artwork: Option<ArtworkInfo>,

    // 統計情報
    pub play_count: u32,
    pub skip_count: u32,
    pub rating: Option<u8>, // 0-5
    pub last_played: Option<DateTime<Utc>>,

    // タイムスタンプ
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtworkInfo {
    pub data: Vec<u8>, // JPEG/PNG data
    pub mime_type: String,
    pub width: u32,
    pub height: u32,
    pub hash: String, // SHA-256
}
```

#### F3.2 ライブラリスキャン

```rust
pub struct LibraryScanner {
    pub scan_directories: Vec<PathBuf>,
    pub recursive: bool,
    pub file_extensions: HashSet<String>,
    pub concurrent_scans: usize,
    pub skip_hidden_files: bool,
}

pub enum ScanMode {
    Full,        // 全ファイル再スキャン
    Incremental, // 変更されたファイルのみ
    Verification, // ハッシュ検証
}

pub trait LibraryManager: Send + Sync {
    async fn scan_library(&mut self, mode: ScanMode) -> Result<ScanResult, LibraryError>;
    async fn add_scan_directory(&mut self, path: PathBuf) -> Result<(), LibraryError>;
    async fn remove_scan_directory(&mut self, path: &Path) -> Result<(), LibraryError>;
    async fn get_track_by_id(&self, id: TrackId) -> Result<Option<TrackInfo>, LibraryError>;
    async fn search_tracks(&self, query: &SearchQuery) -> Result<Vec<TrackInfo>, LibraryError>;
    async fn get_tracks_by_criteria(&self, criteria: &[SearchCriterion])
        -> Result<Vec<TrackInfo>, LibraryError>;
}

#[derive(Debug)]
pub struct ScanResult {
    pub tracks_added: usize,
    pub tracks_updated: usize,
    pub tracks_removed: usize,
    pub errors: Vec<ScanError>,
    pub duration: Duration,
}
```

### F4. 音圧ビジュアライザー（メイン機能）

#### F4.1 ビジュアライザータイプ

| タイプ               | 詳細                        | 実装方式       | FFT サイズ |
| -------------------- | --------------------------- | -------------- | ---------- |
| スペクトラムバー     | 周波数帯域別の縦棒表示      | 対数スケール   | 2048 点    |
| 波形表示             | 時間軸波形表示              | リングバッファ | -          |
| サークルスペクトラム | 円形配置スペクトラム        | 極座標変換     | 1024 点    |
| パーティクルシステム | 音圧連動パーティクル        | GPU 計算       | 512 点     |
| 3D スペクトラム      | 立体的スペクトラム表示      | 3D 変換行列    | 2048 点    |
| VU メーター          | アナログ風レベルメーター    | RMS 計算       | -          |
| ウォーターフォール   | 時間-周波数スペクトログラム | 2D 配列        | 2048 点    |
| オシロスコープ       | XY モード波形表示           | Lissajous      | -          |

#### F4.2 ビジュアライザー設定

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationConfig {
    pub sensitivity: f32,        // 0.1 - 5.0
    pub frequency_range: (f32, f32), // Hz範囲 (20Hz - 20kHz)
    pub color_scheme: ColorScheme,
    pub animation_speed: f32,    // 0.1 - 3.0
    pub smoothing: SmoothingConfig,
    pub auto_gain: bool,
    pub peak_hold: bool,
    pub peak_hold_time: Duration,
    pub fall_speed: f32,         // 0.1 - 2.0
    pub logarithmic_frequency: bool,
    pub window_function: WindowFunction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    pub name: String,
    pub primary: Color,
    pub secondary: Color,
    pub gradient: Vec<ColorStop>,
    pub background: Color,
    pub peak_color: Option<Color>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorStop {
    pub position: f32, // 0.0 - 1.0
    pub color: Color,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmoothingConfig {
    pub enabled: bool,
    pub factor: f32,     // 0.0 - 1.0
    pub algorithm: SmoothingAlgorithm,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SmoothingAlgorithm {
    ExponentialMovingAverage,
    SimpleMovingAverage { window_size: usize },
    KalmanFilter { process_noise: f32, measurement_noise: f32 },
    ButterworthFilter { cutoff_frequency: f32, order: u8 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WindowFunction {
    Rectangle,
    Hamming,
    Hanning,
    Blackman,
    BlackmanHarris,
    Kaiser { beta: f32 },
}
```

#### F4.3 リアルタイム処理要件

| 指標         | 目標値     | 測定方法         |
| ------------ | ---------- | ---------------- |
| 更新頻度     | 120 FPS    | フレーム間隔測定 |
| 音声遅延     | 50ms 以下  | バッファ遅延測定 |
| CPU 使用率   | 5%以下     | プロセッサ使用率 |
| メモリ使用量 | +50MB 以下 | プロセスメモリ   |
| GPU 使用率   | 10%以下    | GPU 監視         |

#### F4.4 FFT 処理仕様

```rust
pub struct SpectrumAnalyzer {
    pub fft_size: usize,
    pub window_function: WindowFunction,
    pub overlap: f32, // 0.0 - 0.75
    pub frequency_bins: usize,
    pub sample_rate: f32,
}

impl SpectrumAnalyzer {
    pub fn analyze(&mut self, audio_data: &[f32]) -> Result<SpectrumData, AnalysisError> {
        // 1. ウィンドウ関数適用
        let windowed = self.apply_window_function(audio_data)?;

        // 2. FFT計算
        let complex_spectrum = self.fft.process(&windowed)?;

        // 3. パワースペクトラム計算
        let power_spectrum: Vec<f32> = complex_spectrum
            .iter()
            .map(|c| c.norm_sqr())
            .collect();

        // 4. 対数変換
        let db_spectrum: Vec<f32> = power_spectrum
            .iter()
            .map(|&p| 20.0 * (p + 1e-10).log10())
            .collect();

        Ok(SpectrumData {
            frequencies: self.get_frequency_bins(),
            magnitudes: db_spectrum,
            sample_rate: self.sample_rate,
            timestamp: Instant::now(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct SpectrumData {
    pub frequencies: Vec<f32>,
    pub magnitudes: Vec<f32>,
    pub sample_rate: f32,
    pub timestamp: Instant,
}
```

### F5. 音響効果機能

#### F5.1 イコライザー

```rust
#[derive(Debug, Clone)]
pub struct Equalizer {
    pub bands: Vec<EqualizerBand>,
    pub enabled: bool,
    pub preset: Option<EqualizerPreset>,
}

#[derive(Debug, Clone)]
pub struct EqualizerBand {
    pub frequency: f32,      // 中心周波数
    pub gain: f32,          // -24dB to +24dB
    pub q_factor: f32,      // 0.1 to 30.0
    pub filter_type: FilterType,
}

#[derive(Debug, Clone)]
pub enum FilterType {
    Peak,
    LowShelf,
    HighShelf,
    LowPass,
    HighPass,
    BandPass,
    Notch,
}

// 標準プリセット
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EqualizerPreset {
    Flat,
    Rock,
    Pop,
    Jazz,
    Classical,
    Electronic,
    Vocal,
    Bass,
    Treble,
    Custom(String),
}
```

#### F5.2 音響効果

```rust
pub trait AudioEffect: Send + Sync {
    fn process(&mut self, input: &[f32], output: &mut [f32]) -> Result<(), EffectError>;
    fn set_parameter(&mut self, name: &str, value: f32) -> Result<(), EffectError>;
    fn get_parameter(&self, name: &str) -> Option<f32>;
    fn reset(&mut self);
}

// リバーブエフェクト
#[derive(Debug, Clone)]
pub struct ReverbEffect {
    pub room_size: f32,      // 0.0 - 1.0
    pub damping: f32,        // 0.0 - 1.0
    pub wet_level: f32,      // 0.0 - 1.0
    pub dry_level: f32,      // 0.0 - 1.0
    pub width: f32,          // 0.0 - 1.0
}

// ディレイ/エコー
#[derive(Debug, Clone)]
pub struct DelayEffect {
    pub delay_time: Duration,
    pub feedback: f32,       // 0.0 - 0.95
    pub wet_level: f32,      // 0.0 - 1.0
    pub sync_to_tempo: bool,
    pub ping_pong: bool,     // L/R チャンネル間のピンポン
}

// 3D音響効果
#[derive(Debug, Clone)]
pub struct SpatialAudioEffect {
    pub listener_position: Vec3,
    pub listener_orientation: Vec3,
    pub room_acoustics: RoomAcoustics,
    pub hrtf_enabled: bool, // Head-Related Transfer Function
}

// クロスフェード
#[derive(Debug, Clone)]
pub struct CrossfadeEffect {
    pub duration: Duration,
    pub curve: CrossfadeCurve,
    pub auto_crossfade: bool,
}

#[derive(Debug, Clone)]
pub enum CrossfadeCurve {
    Linear,
    Logarithmic,
    Exponential,
    SCurve,
    Custom(Vec<f32>),
}
```

### F6. プラグインシステム

#### F6.1 ビジュアライザープラグイン

```rust
// プラグインAPI
pub trait VisualizerPlugin: Send + Sync {
    fn metadata(&self) -> PluginMetadata;
    fn initialize(&mut self, config: &VisualizationConfig) -> Result<(), PluginError>;
    fn render(&mut self, spectrum_data: &SpectrumData, render_context: &mut RenderContext)
        -> Result<(), RenderError>;
    fn configure(&mut self, settings: &HashMap<String, PluginValue>) -> Result<(), ConfigError>;
    fn get_settings_schema(&self) -> SettingsSchema;
    fn cleanup(&mut self);
    fn supports_real_time(&self) -> bool;
    fn required_sample_rate(&self) -> Option<u32>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PluginMetadata {
    pub name: String,
    pub version: semver::Version,
    pub author: String,
    pub description: String,
    pub category: PluginCategory,
    pub supported_formats: Vec<String>,
    pub minimum_fft_size: usize,
    pub preferred_fft_size: usize,
    pub gpu_accelerated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginCategory {
    Spectrum,
    Waveform,
    Particle,
    ThreeDimensional,
    Abstract,
    Educational,
    Custom,
}

#[derive(Debug, Clone)]
pub struct RenderContext {
    pub canvas_size: (u32, u32),
    pub target_fps: f32,
    pub elapsed_time: Duration,
    pub audio_position: Duration,
    pub current_track: Option<TrackInfo>,
}
```

#### F6.2 エフェクトプラグイン

```rust
pub trait EffectPlugin: Send + Sync {
    fn metadata(&self) -> EffectMetadata;
    fn initialize(&mut self, sample_rate: f32, max_buffer_size: usize) -> Result<(), PluginError>;
    fn process(&mut self, input: &AudioBuffer, output: &mut AudioBuffer) -> Result<(), PluginError>;
    fn set_parameter(&mut self, id: u32, value: f32) -> Result<(), PluginError>;
    fn get_parameter(&self, id: u32) -> Option<f32>;
    fn get_parameters_info(&self) -> Vec<ParameterInfo>;
    fn save_state(&self) -> Vec<u8>;
    fn load_state(&mut self, state: &[u8]) -> Result<(), PluginError>;
}

#[derive(Debug, Clone)]
pub struct EffectMetadata {
    pub name: String,
    pub category: EffectCategory,
    pub latency_samples: usize,
    pub tail_time: Duration,
}

#[derive(Debug, Clone)]
pub enum EffectCategory {
    Dynamics,    // コンプレッサー、リミッター
    Frequency,   // EQ、フィルター
    Time,        // ディレイ、リバーブ
    Modulation,  // コーラス、フランジャー
    Distortion,  // オーバードライブ、ファズ
    Spatial,     // 3D音響、ステレオ拡張
    Utility,     // ゲイン、フェーズ
}
```

#### F6.3 プラグイン管理

```rust
pub struct PluginManager {
    visualizer_plugins: HashMap<String, Box<dyn VisualizerPlugin>>,
    effect_plugins: HashMap<String, Box<dyn EffectPlugin>>,
    plugin_configs: HashMap<String, PluginConfig>,
    scan_directories: Vec<PathBuf>,
}

impl PluginManager {
    pub async fn scan_plugins(&mut self) -> Result<ScanResult, PluginError> {
        // プラグインディレクトリをスキャン
        // 動的ライブラリの読み込み
        // メタデータ検証
        // 初期化テスト
    }

    pub fn load_plugin(&mut self, path: &Path) -> Result<String, PluginError>;
    pub fn unload_plugin(&mut self, name: &str) -> Result<(), PluginError>;
    pub fn get_plugin_info(&self, name: &str) -> Option<&PluginMetadata>;
    pub fn hot_reload_plugin(&mut self, name: &str) -> Result<(), PluginError>;
    pub fn validate_plugin(&self, path: &Path) -> Result<PluginMetadata, PluginError>;
}
```

## 非機能要件

### NF1. パフォーマンス要件

#### NF1.1 応答性能

| 指標                 | 目標値           | 許容値          | 測定方法              |
| -------------------- | ---------------- | --------------- | --------------------- |
| アプリ起動時間       | 2 秒以内         | 3 秒以内        | プロセス開始〜UI 表示 |
| トラック切り替え     | 200ms 以内       | 500ms 以内      | 操作〜音声出力開始    |
| UI 応答時間          | 16ms 以下        | 33ms 以下       | 60fps 保証            |
| ビジュアライザー更新 | 8.3ms 以下       | 16ms 以下       | 120fps 目標           |
| ライブラリスキャン   | 1000 トラック/秒 | 500 トラック/秒 | スキャン速度          |

#### NF1.2 リソース使用量

| リソース     | アイドル時 | 再生時       | ビジュアライザー有効時 | 制限値  |
| ------------ | ---------- | ------------ | ---------------------- | ------- |
| メモリ       | 30MB 以下  | 80MB 以下    | 120MB 以下             | 200MB   |
| CPU          | 0.5%以下   | 2%以下       | 4%以下                 | 10%     |
| GPU          | 0%         | 0%           | 5%以下                 | 15%     |
| ディスク I/O | 最小限     | 読み込みのみ | 読み込みのみ           | 100MB/s |

### NF2. 音質要件

#### NF2.1 音声品質

```rust
pub struct AudioQualitySpecs {
    pub bit_perfect: bool,           // Bit-perfect再生保証
    pub max_bit_depth: u8,          // 最大32bit
    pub max_sample_rate: u32,       // 最大192kHz
    pub thd_n: f32,                 // THD+N < 0.0005% (-106dB)
    pub snr: f32,                   // SNR > 120dB
    pub dynamic_range: f32,         // > 120dB
    pub channel_separation: f32,    // > 100dB
    pub frequency_response: (f32, f32), // ±0.1dB (20Hz-20kHz)
    pub phase_accuracy: f32,        // ±1°
    pub jitter: Duration,           // < 10ps
    pub latency: Duration,          // < 50ms
}
```

#### NF2.2 対応音声規格

| 規格         | ビット深度 | サンプルレート | 実装状況    |
| ------------ | ---------- | -------------- | ----------- |
| CD Quality   | 16bit      | 44.1kHz        | ✅ 完全対応 |
| DVD Audio    | 24bit      | 48kHz, 96kHz   | ✅ 完全対応 |
| Hi-Res Audio | 24/32bit   | 192kHz         | ✅ 完全対応 |
| DSD64        | 1bit       | 2.8MHz         | 🔄 将来対応 |
| DSD128       | 1bit       | 5.6MHz         | 🔄 将来対応 |
| MQA          | 可変       | 可変           | 🔄 将来対応 |

### NF3. 可用性要件

#### NF3.1 信頼性

| 指標                              | 目標値   | 測定方法               |
| --------------------------------- | -------- | ---------------------- |
| MTBF (Mean Time Between Failures) | 720 時間 | 継続動作テスト         |
| クラッシュ率                      | < 0.001% | エラーレポート分析     |
| メモリリーク                      | なし     | 24 時間メモリ監視      |
| データ破損                        | なし     | ファイル整合性チェック |

#### NF3.2 エラー処理

```rust
// 階層化エラー処理
#[derive(Error, Debug)]
pub enum SonicFlowError {
    // システムレベルエラー
    #[error("System error: {0}")]
    System(#[from] SystemError),

    // アプリケーションレベルエラー
    #[error("Audio error: {0}")]
    Audio(#[from] AudioError),

    #[error("Visualizer error: {0}")]
    Visualizer(#[from] VisualizerError),

    #[error("Plugin error: {0}")]
    Plugin(#[from] PluginError),

    // 復旧可能エラー
    #[error("Recoverable error: {message}")]
    Recoverable { message: String, retry_count: u32 },

    // 設定エラー
    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),
}

// 自動復旧機能
pub struct ErrorRecoveryManager {
    retry_strategies: HashMap<String, RetryStrategy>,
    circuit_breakers: HashMap<String, CircuitBreaker>,
    fallback_handlers: Vec<Box<dyn FallbackHandler>>,
}
```

### NF4. セキュリティ要件

#### NF4.1 プラグインセキュリティ

```rust
pub struct PluginSecurityPolicy {
    pub signature_verification: bool,
    pub sandbox_enabled: bool,
    pub memory_limit: usize,        // MB
    pub cpu_time_limit: Duration,   // 秒
    pub file_access_whitelist: Vec<PathBuf>,
    pub network_access: NetworkPolicy,
}

#[derive(Debug, Clone)]
pub enum NetworkPolicy {
    Disabled,
    RestrictedDomains(Vec<String>),
    Unrestricted,
}

// セキュアプラグイン実行環境
pub struct SecurePluginRuntime {
    memory_pool: MemoryPool,
    execution_monitor: ExecutionMonitor,
    resource_limiter: ResourceLimiter,
}
```

#### NF4.2 データ保護

| 項目                 | 要件           | 実装           |
| -------------------- | -------------- | -------------- |
| 設定ファイル暗号化   | AES-256        | ✅ 実装        |
| プラグイン署名検証   | RSA-2048       | ✅ 実装        |
| メモリ保護           | スタック保護   | ✅ OS 機能利用 |
| ファイルアクセス制限 | サンドボックス | ✅ 実装        |
| ネットワークアクセス | 制限付き       | ✅ 実装        |

## ユーザーインターフェース仕様

### UI1. メインウィンドウ

```slint
// src/ui/main_window.slint
export component MainWindow inherits Window {
    preferred-width: 1200px;
    preferred-height: 800px;
    min-width: 800px;
    min-height: 600px;

    title: "Sonic Flow";
    icon: @image-url("assets/icons/app_icon.png");

    // レイアウト構成
    VerticalLayout {
        padding: 8px;
        spacing: 8px;

        // トップバー（メニュー、タイトル）
        TopBar {
            height: 48px;
            current-track: current_track;
        }

        // メインコンテンツ
        HorizontalLayout {
            spacing: 8px;

            // 左サイドバー（プレイリスト、ライブラリ）
            Sidebar {
                width: 280px;
                min-width: 200px;
                max-width: 400px;
            }

            // 中央エリア（ビジュアライザー）
            VisualizerArea {
                min-width: 400px;
                spectrum-data: spectrum_data;
                current-visualizer: current_visualizer;
            }

            // 右サイドバー（設定、情報）
            if show_right_sidebar: RightPanel {
                width: 280px;
            }
        }

        // ボトムバー（再生コントロール）
        PlayerControls {
            height: 80px;
            is-playing: is_playing;
            current-position: current_position;
            track-duration: track_duration;
            volume: volume;
        }
    }
}
```

### UI2. テーマシステム

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,
    pub colors: ColorPalette,
    pub typography: Typography,
    pub spacing: SpacingScale,
    pub animations: AnimationSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorPalette {
    // プライマリカラー
    pub primary: Color,
    pub primary_hover: Color,
    pub primary_active: Color,

    // セカンダリカラー
    pub secondary: Color,
    pub accent: Color,

    // 背景色
    pub background: Color,
    pub surface: Color,
    pub card: Color,

    // テキスト色
    pub text_primary: Color,
    pub text_secondary: Color,
    pub text_disabled: Color,

    // ステータス色
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub info: Color,

    // ビジュアライザー色
    pub visualizer_primary: Color,
    pub visualizer_secondary: Color,
    pub visualizer_background: Color,
}

// 標準テーマ
pub mod themes {
    pub const DARK: Theme = Theme { /* 定義 */ };
    pub const LIGHT: Theme = Theme { /* 定義 */ };
    pub const HIGH_CONTRAST: Theme = Theme { /* 定義 */ };
    pub const NEON: Theme = Theme { /* 定義 */ };
}
```

### UI3. アクセシビリティ

```rust
pub struct AccessibilityFeatures {
    pub screen_reader_support: bool,
    pub keyboard_navigation: bool,
    pub high_contrast_mode: bool,
    pub large_text_mode: bool,
    pub reduced_motion: bool,
    pub sound_visualization: bool, // 音の視覚化（聴覚障害者向け）
}

// キーボードショートカット
pub struct KeyboardShortcuts {
    pub play_pause: KeyCombination,
    pub next_track: KeyCombination,
    pub previous_track: KeyCombination,
    pub volume_up: KeyCombination,
    pub volume_down: KeyCombination,
    pub toggle_mute: KeyCombination,
    pub toggle_visualizer: KeyCombination,
    pub focus_search: KeyCombination,
    pub toggle_fullscreen: KeyCombination,
}

impl Default for KeyboardShortcuts {
    fn default() -> Self {
        Self {
            play_pause: KeyCombination::new(Key::Space),
            next_track: KeyCombination::new(Key::ArrowRight),
            previous_track: KeyCombination::new(Key::ArrowLeft),
            volume_up: KeyCombination::new(Key::ArrowUp),
            volume_down: KeyCombination::new(Key::ArrowDown),
            toggle_mute: KeyCombination::new(Key::M),
            toggle_visualizer: KeyCombination::new(Key::V),
            focus_search: KeyCombination::with_ctrl(Key::F),
            toggle_fullscreen: KeyCombination::new(Key::F11),
        }
    }
}
```

## API 仕様

### API1. プラグイン開発者向け API

```rust
// プラグイン開発用マクロ
pub use sonic_flow_plugin_api::*;

// サンプルプラグイン実装
use sonic_flow_plugin_api::prelude::*;

pub struct CustomVisualizer {
    config: VisualizationConfig,
    fft_buffer: Vec<f32>,
    animation_time: f32,
}

impl Default for CustomVisualizer {
    fn default() -> Self {
        Self {
            config: VisualizationConfig::default(),
            fft_buffer: Vec::new(),
            animation_time: 0.0,
        }
    }
}

impl VisualizerPlugin for CustomVisualizer {
    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "Custom Visualizer".to_string(),
            version: semver::Version::new(1, 0, 0),
            author: "Developer Name".to_string(),
            description: "A custom visualizer plugin".to_string(),
            category: PluginCategory::Abstract,
            supported_formats: vec!["audio/mpeg".to_string(), "audio/flac".to_string()],
            minimum_fft_size: 512,
            preferred_fft_size: 1024,
            gpu_accelerated: false,
        }
    }

    fn initialize(&mut self, config: &VisualizationConfig) -> Result<(), PluginError> {
        self.config = config.clone();
        self.fft_buffer.resize(config.fft_size, 0.0);
        Ok(())
    }

    fn render(&mut self, spectrum_data: &SpectrumData, context: &mut RenderContext)
        -> Result<(), RenderError> {
        // カスタム描画ロジック
        self.animation_time += context.elapsed_time.as_secs_f32();

        // スペクトラムデータを使用してビジュアル要素を描画
        for (i, &magnitude) in spectrum_data.magnitudes.iter().enumerate() {
            let x = (i as f32 / spectrum_data.magnitudes.len() as f32) * context.canvas_size.0 as f32;
            let height = magnitude * self.config.sensitivity * context.canvas_size.1 as f32 * 0.8;

            // 描画コマンドを発行
            context.draw_rectangle(
                Rectangle::new(x, context.canvas_size.1 as f32 - height, 4.0, height),
                self.config.color_scheme.primary,
            );
        }

        Ok(())
    }

    fn configure(&mut self, settings: &HashMap<String, PluginValue>) -> Result<(), ConfigError> {
        if let Some(sensitivity) = settings.get("sensitivity") {
            if let PluginValue::Float(value) = sensitivity {
                self.config.sensitivity = *value;
            }
        }
        Ok(())
    }

    fn get_settings_schema(&self) -> SettingsSchema {
        SettingsSchema::builder()
            .add_float_parameter("sensitivity", 1.0, 0.1, 5.0, "Sensitivity")
            .add_color_parameter("color", self.config.color_scheme.primary, "Primary Color")
            .build()
    }

    fn cleanup(&mut self) {
        self.fft_buffer.clear();
    }

    fn supports_real_time(&self) -> bool {
        true
    }

    fn required_sample_rate(&self) -> Option<u32> {
        None // どのサンプルレートでも対応
    }
}

// プラグイン登録
register_visualizer!(CustomVisualizer);
```

## データ仕様

### D1. 設定ファイル形式（TOML）

```toml
# ~/.config/sonic-flow/config.toml

[application]
version = "1.0.0"
first_run = false
theme = "dark"
language = "ja"
auto_update_check = true

[audio]
sample_rate = 44100
buffer_size = 512
bit_depth = 24
output_device = "default"
exclusive_mode = false
bit_perfect_mode = true

[visualizer]
default_type = "spectrum_bars"
fft_size = 2048
update_rate = 120
sensitivity = 1.0
auto_gain = true
peak_hold = true
peak_hold_time = 2.0
fall_speed = 0.8

[visualizer.spectrum_bars]
bar_count = 64
logarithmic = true
smoothing = 0.3
color_scheme = "neon_blue"

[visualizer.waveform]
sample_count = 4096
line_width = 2.0
fill_enabled = false

[ui]
window_width = 1200
window_height = 800
always_on_top = false
minimize_to_tray = false
show_fps = false
vsync_enabled = true

[library]
scan_directories = [
    "~/Music",
    "/mnt/music",
    "D:/Music"
]
auto_scan = true
scan_interval = 300
watch_filesystem = true
extract_artwork = true
generate_thumbnails = true

[playlists]
default_format = "json"
auto_save = true
backup_enabled = true
max_backups = 5

[plugins]
auto_load = true
scan_directories = [
    "~/.config/sonic-flow/plugins",
    "/usr/lib/sonic-flow/plugins"
]
security_policy = "strict"
memory_limit = 100  # MB
cpu_limit = 5.0     # seconds

[network]
update_check_url = "https://api.sonicflow.app/updates"
plugin_repository = "https://plugins.sonicflow.app"
telemetry_enabled = false
crash_reports = true

[advanced]
log_level = "info"
performance_monitoring = false
debug_mode = false
experimental_features = false
```

### D2. プレイリストファイル形式

```json
{
  "version": "2.0",
  "format": "sonic_flow_playlist",
  "metadata": {
    "name": "My Awesome Playlist",
    "description": "A collection of favorite tracks",
    "created_at": "2025-08-12T10:30:00Z",
    "modified_at": "2025-08-12T15:45:00Z",
    "created_by": "Sonic Flow v1.0.0",
    "track_count": 25,
    "total_duration": 5400,
    "artwork": {
      "path": "playlists/covers/my_awesome_playlist.jpg",
      "hash": "sha256:abc123..."
    }
  },
  "settings": {
    "shuffle_enabled": false,
    "repeat_mode": "all",
    "crossfade_duration": 3.0,
    "auto_dj": false
  },
  "tracks": [
    {
      "id": "track_001",
      "file_path": "/path/to/music/song1.flac",
      "file_hash": "sha256:def456...",
      "title": "Amazing Song",
      "artist": "Great Artist",
      "album": "Wonderful Album",
      "duration": 240.5,
      "added_at": "2025-08-12T10:30:00Z",
      "play_count": 15,
      "rating": 5,
      "custom_metadata": {
        "bpm": 128,
        "key": "C major",
        "energy": 0.8
      }
    }
  ],
  "smart_criteria": null,
  "checksum": "sha256:playlist_content_hash"
}
```

### D3. データベーススキーマ（SQLite）

```sql
-- スキーマバージョン管理
CREATE TABLE schema_version (
    version INTEGER PRIMARY KEY,
    applied_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    migration_notes TEXT
);

-- トラック情報テーブル
CREATE TABLE tracks (
    id TEXT PRIMARY KEY,           -- UUID
    file_path TEXT UNIQUE NOT NULL,
    file_hash TEXT NOT NULL,       -- SHA-256
    file_size INTEGER NOT NULL,
    file_modified DATETIME NOT NULL,

    -- メタデータ
    title TEXT,
    artist TEXT,
    album TEXT,
    album_artist TEXT,
    composer TEXT,
    genre TEXT,
    year INTEGER,
    track_number INTEGER,
    disc_number INTEGER,
    total_tracks INTEGER,
    total_discs INTEGER,

    -- 技術情報
    duration_ms INTEGER NOT NULL,
    bitrate INTEGER,
    sample_rate INTEGER,
    bit_depth INTEGER,
    channels INTEGER,
    codec TEXT,
    container_format TEXT,

    -- アートワーク
    artwork_hash TEXT,
    artwork_embedded BOOLEAN DEFAULT FALSE,
    artwork_path TEXT,

    -- オーディオ解析結果
    loudness_lufs REAL,           -- ラウドネス (LUFS)
    dynamic_range REAL,           -- ダイナミックレンジ
    peak_db REAL,                 -- ピーク値
    rms_db REAL,                  -- RMS値
    spectral_centroid REAL,       -- スペクトル重心
    zero_crossing_rate REAL,      -- ゼロクロス率
    tempo_bpm REAL,               -- テンポ
    key_signature TEXT,           -- キー

    -- 統計情報
    play_count INTEGER DEFAULT 0,
    skip_count INTEGER DEFAULT 0,
    rating INTEGER CHECK(rating >= 0 AND rating <= 5),
    favorite BOOLEAN DEFAULT FALSE,

    -- タイムスタンプ
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    modified_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    last_played DATETIME,
    first_played DATETIME,

    -- インデックス最適化のための追加カラム
    artist_sort TEXT,
    album_sort TEXT,
    title_sort TEXT
);

-- プレイリストテーブル
CREATE TABLE playlists (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    type TEXT NOT NULL DEFAULT 'static', -- 'static' or 'smart'

    -- スマートプレイリスト設定
    criteria_json TEXT,               -- JSON形式の条件
    auto_update BOOLEAN DEFAULT FALSE,
    max_tracks INTEGER,

    -- 設定
    shuffle_enabled BOOLEAN DEFAULT FALSE,
    repeat_mode TEXT DEFAULT 'off',   -- 'off', 'all', 'one'
    crossfade_duration REAL DEFAULT 0.0,

    -- メタデータ
    artwork_path TEXT,
    color_scheme TEXT,

    -- タイムスタンプ
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    modified_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    last_played DATETIME
);

-- プレイリスト-トラック関連テーブル
CREATE TABLE playlist_tracks (
    playlist_id TEXT NOT NULL,
    track_id TEXT NOT NULL,
    position INTEGER NOT NULL,
    added_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    added_by TEXT DEFAULT 'user',     -- 'user', 'auto', 'import'

    PRIMARY KEY (playlist_id, track_id),
    FOREIGN KEY (playlist_id) REFERENCES playlists(id) ON DELETE CASCADE,
    FOREIGN KEY (track_id) REFERENCES tracks(id) ON DELETE CASCADE
);

-- 再生履歴テーブル
CREATE TABLE playback_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    track_id TEXT NOT NULL,
    played_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    duration_played INTEGER NOT NULL,  -- 再生された時間（ms）
    total_duration INTEGER NOT NULL,   -- トラック総時間（ms）
    completion_percentage REAL NOT NULL,
    source TEXT,                       -- 'playlist', 'library', 'search'

    FOREIGN KEY (track_id) REFERENCES tracks(id) ON DELETE CASCADE
);

-- アルバムアートワークキャッシュ
CREATE TABLE artwork_cache (
    hash TEXT PRIMARY KEY,
    data BLOB NOT NULL,
    mime_type TEXT NOT NULL,
    width INTEGER NOT NULL,
    height INTEGER NOT NULL,
    file_size INTEGER NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    last_accessed DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 設定テーブル
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    type TEXT NOT NULL,              -- 'string', 'int', 'float', 'bool', 'json'
    description TEXT,
    modified_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- プラグイン情報テーブル
CREATE TABLE plugins (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    version TEXT NOT NULL,
    type TEXT NOT NULL,              -- 'visualizer', 'effect', 'decoder'
    file_path TEXT NOT NULL,
    enabled BOOLEAN DEFAULT TRUE,
    config_json TEXT,
    installed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    last_used DATETIME
);

-- インデックス定義
CREATE INDEX idx_tracks_artist ON tracks(artist);
CREATE INDEX idx_tracks_album ON tracks(album);
CREATE INDEX idx_tracks_genre ON tracks(genre);
CREATE INDEX idx_tracks_year ON tracks(year);
CREATE INDEX idx_tracks_play_count ON tracks(play_count DESC);
CREATE INDEX idx_tracks_rating ON tracks(rating DESC);
CREATE INDEX idx_tracks_last_played ON tracks(last_played DESC);
CREATE INDEX idx_tracks_duration ON tracks(duration_ms);
CREATE INDEX idx_tracks_file_hash ON tracks(file_hash);

CREATE INDEX idx_playlist_tracks_position ON playlist_tracks(playlist_id, position);
CREATE INDEX idx_playback_history_played_at ON playback_history(played_at DESC);
CREATE INDEX idx_playback_history_track_id ON playback_history(track_id);

-- フルテキスト検索用仮想テーブル
CREATE VIRTUAL TABLE tracks_fts USING fts5(
    title, artist, album, genre, composer,
    content='tracks',
    content_rowid='rowid'
);

-- トリガー（FTS同期）
CREATE TRIGGER tracks_fts_insert AFTER INSERT ON tracks BEGIN
    INSERT INTO tracks_fts(rowid, title, artist, album, genre, composer)
    VALUES (NEW.rowid, NEW.title, NEW.artist, NEW.album, NEW.genre, NEW.composer);
END;

CREATE TRIGGER tracks_fts_update AFTER UPDATE ON tracks BEGIN
    UPDATE tracks_fts SET
        title = NEW.title,
        artist = NEW.artist,
        album = NEW.album,
        genre = NEW.genre,
        composer = NEW.composer
    WHERE rowid = NEW.rowid;
END;

CREATE TRIGGER tracks_fts_delete AFTER DELETE ON tracks BEGIN
    DELETE FROM tracks_fts WHERE rowid = OLD.rowid;
END;
```

## プラットフォーム要件

### P1. 対応 OS

| OS      | バージョン     | アーキテクチャ | 優先度 | 実装状況      |
| ------- | -------------- | -------------- | ------ | ------------- |
| Windows | 10/11          | x86_64         | 必須   | ✅ 完全対応   |
| macOS   | 12+ (Monterey) | x86_64, ARM64  | 必須   | ✅ 完全対応   |
| Linux   | Ubuntu 20.04+  | x86_64         | 必須   | ✅ 完全対応   |
| Linux   | Arch Linux     | x86_64         | 高     | ✅ テスト済み |
| Linux   | Fedora 35+     | x86_64         | 中     | 🔄 テスト中   |
| Linux   | Debian 11+     | x86_64         | 中     | 🔄 計画中     |

### P2. システム要件

#### P2.1 最小要件

| コンポーネント | 要件                             | 備考               |
| -------------- | -------------------------------- | ------------------ |
| CPU            | Intel i3-4000 / AMD FX-6300 相当 | 2.4GHz 以上        |
| メモリ         | 4GB RAM                          | システム使用量含む |
| ストレージ     | 2GB 空き容量                     | SSD 推奨           |
| グラフィック   | OpenGL 3.3 対応                  | 統合 GPU でも可    |
| オーディオ     | DirectSound/ALSA/CoreAudio       | 標準ドライバー     |

#### P2.2 推奨要件

| コンポーネント | 要件                                  | 備考                         |
| -------------- | ------------------------------------- | ---------------------------- |
| CPU            | Intel i5-8400 / AMD Ryzen 5 2600 相当 | 3.0GHz 以上                  |
| メモリ         | 8GB RAM                               | 大量のライブラリ用           |
| ストレージ     | SSD 10GB 空き容量                     | 高速アクセス                 |
| グラフィック   | 専用 GPU 推奨                         | NVIDIA GTX 1060 / AMD RX 580 |
| オーディオ     | ASIO/専用ドライバー                   | 低レイテンシ                 |

#### P2.3 ハイエンド要件（4K/高 fps ビジュアライザー）

| コンポーネント | 要件                                | 備考                 |
| -------------- | ----------------------------------- | -------------------- |
| CPU            | Intel i7-10700K / AMD Ryzen 7 3700X | 8 コア推奨           |
| メモリ         | 16GB RAM                            | 高解像度ビジュアル用 |
| グラフィック   | RTX 3060 / RX 6600 XT 以上          | GPU 加速必須         |
| ディスプレイ   | 4K/120Hz 対応                       | HDR 対応推奨         |

### P3. 依存関係

#### P3.1 システムライブラリ

**Windows:**

```toml
[target.'cfg(windows)'.dependencies]
winapi = { version = "0.3", features = [
    "winmm", "dsound", "wasapi", "mfapi", "mfreadwrite"
]}
windows = { version = "0.52", features = [
    "Win32_Media_Audio", "Win32_System_Com", "Win32_Graphics_Direct3D11"
]}
```

**macOS:**

```toml
[target.'cfg(target_os = "macos")'.dependencies]
core-audio = "0.11"
core-foundation = "0.9"
coreaudio-sys = "0.2"
metal = "0.27"
cocoa = "0.25"
```

**Linux:**

```toml
[target.'cfg(unix)'.dependencies]
alsa = "0.7"
pulse = "2.5"
libc = "0.2"
x11 = { version = "2.21", features = ["xlib"] }
wayland-client = "0.31"
```

#### P3.2 ランタイム依存関係

| プラットフォーム | 必須ライブラリ                  | オプション                |
| ---------------- | ------------------------------- | ------------------------- |
| Windows          | Visual C++ 2022 Redistributable | DirectX 12                |
| macOS            | なし（静的リンク）              | Metal Performance Shaders |
| Linux            | ALSA/PulseAudio, X11/Wayland    | JACK, Pipewire            |

## テストとデプロイメント仕様

### T1. テスト要件

#### T1.1 ユニットテスト

```rust
// テストカバレッジ目標: 85%以上
#[cfg(test)]
mod tests {
    use super::*;
    use criterion::Criterion;
    use proptest::prelude::*;
    use mockall::predicate::*;

    #[test]
    fn test_audio_engine_basic_playback() {
        let mut engine = AudioEngine::new_with_mock_renderer();
        let test_track = create_test_track();

        engine.load_track(test_track).unwrap();
        engine.play().unwrap();

        assert_eq!(engine.playback_state(), PlaybackState::Playing);
    }

    #[tokio::test]
    async fn test_playlist_manager_operations() {
        let mut manager = PlaylistManager::new_in_memory();

        let playlist_id = manager.create_playlist("Test Playlist", None).await.unwrap();
        let track = create_test_track();

        manager.add_track(playlist_id, track).await.unwrap();

        let playlist = manager.get_playlist(playlist_id).await.unwrap();
        assert_eq!(playlist.track_count, 1);
    }

    // プロパティベーステスト
    proptest! {
        #[test]
        fn test_spectrum_analyzer_stability(
            frequencies in prop::collection::vec(0.0f32..20000.0, 1..2048)
        ) {
            let mut analyzer = SpectrumAnalyzer::new(2048);
            let result = analyzer.analyze(&frequencies);
            prop_assert!(result.is_ok());

            if let Ok(spectrum) = result {
                prop_assert!(spectrum.magnitudes.iter().all(|&x| x.is_finite()));
            }
        }
    }
}
```

#### T1.2 統合テスト

```rust
// tests/integration/audio_pipeline_test.rs
#[tokio::test]
async fn test_complete_audio_pipeline() {
    let config = AudioConfig::test_default();
    let system = AudioSystem::new(config).await.unwrap();

    // テスト用オーディオファイルをロード
    let test_file = test_assets_path().join("test_audio.flac");
    system.load_file(&test_file).await.unwrap();

    // 再生開始
    system.play().await.unwrap();

    // スペクトラムデータの受信を確認
    let spectrum_data = system.get_spectrum_data().await;
    assert!(spectrum_data.magnitudes.len() > 0);

    // 音声出力の確認（モック使用）
    let audio_output = system.get_audio_output_mock();
    audio_output.verify_samples_written().await;
}
```

#### T1.3 パフォーマンステスト

```rust
// benches/audio_performance.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_fft_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("fft_processing");

    for size in [512, 1024, 2048, 4096].iter() {
        group.bench_with_input(
            BenchmarkId::new("rustfft", size),
            size,
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
    let mut visualizer = SpectrumBarsVisualizer::new();
    let mut render_context = create_test_render_context();

    group.bench_function("spectrum_bars", |b| {
        b.iter(|| {
            visualizer.render(black_box(&spectrum_data), black_box(&mut render_context))
        });
    });

    group.finish();
}

criterion_group!(benches, benchmark_fft_processing, benchmark_visualizer_rendering);
criterion_main!(benches);
```

### T2. デプロイメント

#### T2.1 ビルド設定

```toml
# Cargo.toml
[profile.release]
lto = "thin"              # リンク時最適化
codegen-units = 1         # 最適化優先
panic = "abort"           # バイナリサイズ削減
strip = true              # デバッグシンボル削除

[profile.release-with-debug]
inherits = "release"
debug = true             # デバッグ情報保持
strip = false

# プラットフォーム別最適化
[profile.release.package."*"]
opt-level = 3            # 最大最適化

# 音声処理クレート用特別設定
[profile.release.package.rustfft]
opt-level = 3
```

#### T2.2 パッケージング

```bash
# Windows
cargo build --release --target x86_64-pc-windows-msvc
# NSIS installer script で MSI パッケージ作成

# macOS
cargo build --release --target x86_64-apple-darwin
cargo build --release --target aarch64-apple-darwin
# Universal binary 作成 + DMG パッケージ

# Linux
cargo build --release --target x86_64-unknown-linux-gnu
# AppImage, Flatpak, Snap パッケージ作成
```

#### T2.3 継続的インテグレーション

```yaml
# .github/workflows/ci.yml
name: CI/CD Pipeline

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main]

jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        rust: [stable, nightly]

    runs-on: ${{ matrix.os }}

    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: ${{ matrix.rust }}
          components: rustfmt, clippy

      - name: Install dependencies
        run: |
          # OS別の依存関係インストール

      - name: Run tests
        run: |
          cargo test --all-features
          cargo test --release

      - name: Run benchmarks
        run: cargo bench

      - name: Check formatting
        run: cargo fmt -- --check

      - name: Run clippy
        run: cargo clippy -- -D warnings

      - name: Security audit
        run: cargo audit

  build:
    needs: test
    strategy:
      matrix:
        include:
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
          - os: windows-latest
            target: x86_64-pc-windows-msvc
          - os: macos-latest
            target: x86_64-apple-darwin
          - os: macos-latest
            target: aarch64-apple-darwin

    runs-on: ${{ matrix.os }}

    steps:
      - uses: actions/checkout@v3
      - name: Build release
        run: cargo build --release --target ${{ matrix.target }}

      - name: Create package
        run: |
          # プラットフォーム別パッケージング

      - name: Upload artifacts
        uses: actions/upload-artifact@v3
        with:
          name: sonic-flow-${{ matrix.target }}
          path: target/${{ matrix.target }}/release/
```

## 品質保証

### Q1. コード品質指標

| 指標                   | 目標値      | 測定ツール         |
| ---------------------- | ----------- | ------------------ |
| テストカバレッジ       | 85%以上     | tarpaulin          |
| 静的解析               | エラー 0 件 | clippy             |
| セキュリティ監査       | 脆弱性 0 件 | cargo audit        |
| コード複雑度           | 10 以下     | rust-code-analysis |
| ドキュメントカバレッジ | 90%以上     | cargo doc          |

### Q2. パフォーマンス監視

```rust
// パフォーマンスメトリクス収集
pub struct QualityMetrics {
    pub startup_time: Duration,
    pub memory_usage: MemoryUsage,
    pub cpu_usage: CpuUsage,
    pub audio_latency: AudioLatency,
    pub frame_times: FrameTimeStats,
    pub error_rates: ErrorRateStats,
}

impl QualityMetrics {
    pub fn collect(&mut self) -> MetricsReport {
        MetricsReport {
            timestamp: Utc::now(),
            startup_time: self.startup_time,
            peak_memory: self.memory_usage.peak(),
            average_cpu: self.cpu_usage.average(),
            audio_dropouts: self.audio_latency.dropouts_count(),
            missed_frames: self.frame_times.missed_count(),
            error_count: self.error_rates.total_errors(),
        }
    }
}
```

---

**最終更新**: 2025-08-12  
**バージョン**: 2.0  
**レビュアー**: Claude (機能仕様再設計)
