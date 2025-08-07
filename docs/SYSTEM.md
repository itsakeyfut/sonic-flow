# Resonance Player - システム設計詳細

## 📋 目次

1. [システム概要](#システム概要)
2. [技術スタック](#技術スタック)
3. [データベース設計](#データベース設計)
4. [並行処理設計](#並行処理設計)
5. [メモリ管理](#メモリ管理)
6. [パフォーマンス最適化](#パフォーマンス最適化)
7. [エラー処理戦略](#エラー処理戦略)
8. [セキュリティ](#セキュリティ)

## システム概要

### アーキテクチャ概要

Resonance Player は、音圧ビジュアライザーを中心とした高性能ミュージックプレイヤーです。Rust の型安全性とゼロコスト抽象化を活用し、リアルタイム音声処理とビジュアル表現を実現します。

### 設計原則

1. **型安全性**: コンパイル時エラー検出による堅牢性
2. **ゼロコピー**: 不要なメモリコピーの排除
3. **並行性**: データ競合のない並列処理
4. **拡張性**: プラグインシステムによる機能追加

## 技術スタック

### コア技術

```toml
[dependencies]
# UI Framework
slint = "1.0"

# 非同期処理
tokio = { version = "1.0", features = ["full", "rt-multi-thread"] }
futures = "0.3"

# 音声処理
rodio = "0.17"           # クロスプラットフォーム音声再生
symphonia = "0.5"        # 音声デコード
cpal = "0.15"            # 低レベル音声I/O
rustfft = "6.0"          # FFT計算

# データ管理
sqlx = { version = "0.7", features = ["sqlite", "chrono", "uuid"] }
serde = { version = "1.0", features = ["derive"] }
toml = "0.7"             # 設定ファイル

# エラーハンドリング
anyhow = "1.0"           # エラーチェーン
thiserror = "1.0"        # カスタムエラー

# ログ・デバッグ
tracing = "0.1"          # 構造化ログ
tracing-subscriber = "0.3"

# ユーティリティ
uuid = { version = "1.0", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
crossbeam = "0.8"        # 並行データ構造
parking_lot = "0.12"     # 高性能Mutex
dashmap = "5.0"          # 並行HashMap
```

### プラットフォーム別依存関係

```toml
# Windows特有
[target.'cfg(windows)'.dependencies]
winapi = { version = "0.3", features = ["winmm", "dsound", "wasapi"] }
windows = { version = "0.48", features = ["Win32_Media_Audio"] }

# macOS特有
[target.'cfg(target_os = "macos")'.dependencies]
core-audio = "0.11"
core-foundation = "0.9"
coreaudio-sys = "0.2"

# Linux特有
[target.'cfg(unix)'.dependencies]
alsa = "0.7"
pulse = "2.5"
libc = "0.2"
```

### 開発・テスト用ツール

```toml
[dev-dependencies]
criterion = "0.5"        # ベンチマーク
proptest = "1.0"         # プロパティベーステスト
mockall = "0.11"         # モック生成
tempfile = "3.0"         # 一時ファイル
wiremock = "0.5"         # HTTPモック

[build-dependencies]
slint-build = "1.0"      # Slintコンパイル
```

## データベース設計

### スキーマ設計

```sql
-- データベースバージョン管理
CREATE TABLE schema_version (
    version INTEGER PRIMARY KEY,
    applied_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- トラック情報
CREATE TABLE tracks (
    id TEXT PRIMARY KEY,           -- UUID
    file_path TEXT UNIQUE NOT NULL,
    file_hash TEXT,               -- SHA-256

    -- メタデータ
    title TEXT,
    artist TEXT,
    album TEXT,
    album_artist TEXT,
    genre TEXT,
    year INTEGER,
    track_number INTEGER,
    disc_number INTEGER,

    -- 技術情報
    duration_ms INTEGER NOT NULL,
    bitrate INTEGER,
    sample_rate INTEGER,
    bit_depth INTEGER,
    channels INTEGER,
    file_size INTEGER,

    -- アートワーク
    artwork_hash TEXT,

    -- 統計情報
    play_count INTEGER DEFAULT 0,
    skip_count INTEGER DEFAULT 0,
    rating INTEGER CHECK(rating >= 0 AND rating <= 5),

    -- タイムスタンプ
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    modified_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    last_played DATETIME,

    -- インデックス用
    title_normalized TEXT,        -- 検索用正規化タイトル
    artist_normalized TEXT        -- 検索用正規化アーティスト
);

-- プレイリスト
CREATE TABLE playlists (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    is_smart BOOLEAN DEFAULT FALSE,  -- スマートプレイリスト
    smart_query TEXT,               -- スマートプレイリストクエリ
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    modified_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- プレイリスト-トラック関連
CREATE TABLE playlist_tracks (
    id TEXT PRIMARY KEY,
    playlist_id TEXT NOT NULL,
    track_id TEXT NOT NULL,
    position INTEGER NOT NULL,
    added_at DATETIME DEFAULT CURRENT_TIMESTAMP,

    FOREIGN KEY (playlist_id) REFERENCES playlists(id) ON DELETE CASCADE,
    FOREIGN KEY (track_id) REFERENCES tracks(id) ON DELETE CASCADE,

    UNIQUE(playlist_id, position),
    UNIQUE(playlist_id, track_id)
);

-- アルバムアートワーク
CREATE TABLE artworks (
    hash TEXT PRIMARY KEY,        -- SHA-256
    data BLOB NOT NULL,           -- 画像データ
    mime_type TEXT NOT NULL,      -- image/jpeg, image/png
    width INTEGER,
    height INTEGER,
    file_size INTEGER,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- 再生履歴
CREATE TABLE play_history (
    id TEXT PRIMARY KEY,
    track_id TEXT NOT NULL,
    played_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    duration_played_ms INTEGER,  -- 実際の再生時間
    completed BOOLEAN DEFAULT FALSE,  -- 最後まで再生したか

    FOREIGN KEY (track_id) REFERENCES tracks(id) ON DELETE CASCADE
);

-- 設定保存
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    type TEXT NOT NULL,          -- 'string', 'integer', 'float', 'boolean', 'json'
    modified_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- インデックス定義
CREATE INDEX idx_tracks_file_path ON tracks(file_path);
CREATE INDEX idx_tracks_artist ON tracks(artist_normalized);
CREATE INDEX idx_tracks_album ON tracks(album);
CREATE INDEX idx_tracks_genre ON tracks(genre);
CREATE INDEX idx_tracks_year ON tracks(year);
CREATE INDEX idx_tracks_last_played ON tracks(last_played);
CREATE INDEX idx_playlist_tracks_playlist ON playlist_tracks(playlist_id, position);
CREATE INDEX idx_play_history_track ON play_history(track_id, played_at);
CREATE INDEX idx_play_history_date ON play_history(played_at);
```

### データアクセス層

```rust
use sqlx::{SqlitePool, Row};
use uuid::Uuid;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    pub async fn new(database_url: &str) -> Result<Self, DatabaseError> {
        let pool = SqlitePool::connect(database_url).await?;

        // マイグレーション実行
        sqlx::migrate!("./migrations").run(&pool).await?;

        Ok(Self { pool })
    }

    pub async fn insert_track(&self, track: &TrackInfo) -> Result<(), DatabaseError> {
        sqlx::query!(
            r#"
            INSERT INTO tracks (
                id, file_path, file_hash, title, artist, album,
                duration_ms, bitrate, sample_rate, title_normalized, artist_normalized
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
            track.id.to_string(),
            track.file_path.to_string_lossy(),
            track.file_hash,
            track.title,
            track.artist,
            track.album,
            track.duration.as_millis() as i64,
            track.bitrate as i64,
            track.sample_rate as i64,
            track.title.as_ref().map(|s| normalize_for_search(s)),
            track.artist.as_ref().map(|s| normalize_for_search(s)),
        )
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn search_tracks(&self, query: &str) -> Result<Vec<TrackInfo>, DatabaseError> {
        let normalized_query = normalize_for_search(query);
        let search_pattern = format!("%{}%", normalized_query);

        let rows = sqlx::query!(
            r#"
            SELECT * FROM tracks
            WHERE title_normalized LIKE ?
               OR artist_normalized LIKE ?
               OR album LIKE ?
            ORDER BY
                CASE
                    WHEN title_normalized LIKE ? THEN 1
                    WHEN artist_normalized LIKE ? THEN 2
                    ELSE 3
                END,
                title
            LIMIT 100
            "#,
            search_pattern, search_pattern, search_pattern,
            format!("{}%", normalized_query), format!("{}%", normalized_query)
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|row| row.into()).collect())
    }
}

fn normalize_for_search(text: &str) -> String {
    // 検索用正規化: 小文字化、アクセント除去、空白正規化
    text.to_lowercase()
        .chars()
        .filter(|c| !c.is_whitespace() || *c == ' ')
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
```

## 並行処理設計

### スレッドアーキテクチャ

```rust
use tokio::sync::{mpsc, broadcast, oneshot};
use crossbeam::queue::SegQueue;
use parking_lot::RwLock;
use std::sync::Arc;

pub struct ThreadManager {
    // 音声処理専用スレッド
    audio_handle: tokio::task::JoinHandle<()>,
    audio_commands: mpsc::UnboundedSender<AudioCommand>,

    // ビジュアライザー処理スレッド
    visualizer_handle: tokio::task::JoinHandle<()>,
    spectrum_channel: broadcast::Sender<SpectrumData>,

    // ファイルI/O専用スレッド
    io_handle: tokio::task::JoinHandle<()>,
    io_commands: mpsc::UnboundedSender<IoCommand>,
}

#[derive(Debug)]
pub enum AudioCommand {
    Play(TrackId),
    Pause,
    Stop,
    Seek(Duration),
    SetVolume(f32),
    Shutdown(oneshot::Sender<()>),
}

impl ThreadManager {
    pub async fn new() -> Result<Self, SystemError> {
        // 音声処理スレッド
        let (audio_tx, audio_rx) = mpsc::unbounded_channel();
        let audio_handle = tokio::spawn(Self::audio_thread(audio_rx));

        // ビジュアライザースレッド
        let (spectrum_tx, _) = broadcast::channel(100);
        let visualizer_handle = tokio::spawn(Self::visualizer_thread(spectrum_tx.clone()));

        // ファイルI/Oスレッド
        let (io_tx, io_rx) = mpsc::unbounded_channel();
        let io_handle = tokio::spawn(Self::io_thread(io_rx));

        Ok(Self {
            audio_handle,
            audio_commands: audio_tx,
            visualizer_handle,
            spectrum_channel: spectrum_tx,
            io_handle,
            io_commands: io_tx,
        })
    }

    async fn audio_thread(mut commands: mpsc::UnboundedReceiver<AudioCommand>) {
        let mut audio_engine = AudioEngine::new().expect("Failed to initialize audio engine");

        while let Some(command) = commands.recv().await {
            match command {
                AudioCommand::Play(track_id) => {
                    if let Err(e) = audio_engine.play(track_id).await {
                        tracing::error!("Failed to play track: {:?}", e);
                    }
                }
                AudioCommand::Pause => {
                    audio_engine.pause().await;
                }
                AudioCommand::Stop => {
                    audio_engine.stop().await;
                }
                AudioCommand::Seek(position) => {
                    if let Err(e) = audio_engine.seek(position).await {
                        tracing::error!("Failed to seek: {:?}", e);
                    }
                }
                AudioCommand::SetVolume(volume) => {
                    audio_engine.set_volume(volume);
                }
                AudioCommand::Shutdown(response) => {
                    audio_engine.shutdown().await;
                    let _ = response.send(());
                    break;
                }
            }
        }
    }

    async fn visualizer_thread(spectrum_sender: broadcast::Sender<SpectrumData>) {
        // リアルタイムFFT処理とビジュアライザー更新
        let mut analyzer = SpectrumAnalyzer::new(2048);
        let mut interval = tokio::time::interval(Duration::from_millis(16)); // 60fps

        loop {
            interval.tick().await;

            // 音声バッファからFFTデータ取得
            if let Some(audio_data) = get_current_audio_buffer() {
                let spectrum = analyzer.analyze(&audio_data);

                // ビジュアライザーに送信（失敗は無視）
                let _ = spectrum_sender.send(spectrum);
            }
        }
    }
}
```

### Lock-Free データ構造

```rust
use crossbeam::{
    atomic::AtomicCell,
    queue::{ArrayQueue, SegQueue},
    channel::{bounded, unbounded, Receiver, Sender},
};

// 音声バッファ用Ring Buffer
pub struct AudioRingBuffer {
    buffer: Vec<AtomicCell<f32>>,
    write_pos: AtomicUsize,
    read_pos: AtomicUsize,
    capacity: usize,
}

impl AudioRingBuffer {
    pub fn new(capacity: usize) -> Self {
        let buffer = (0..capacity)
            .map(|_| AtomicCell::new(0.0))
            .collect();

        Self {
            buffer,
            write_pos: AtomicUsize::new(0),
            read_pos: AtomicUsize::new(0),
            capacity,
        }
    }

    pub fn write(&self, data: &[f32]) -> usize {
        let mut written = 0;
        let start_pos = self.write_pos.load(Ordering::Acquire);

        for &sample in data {
            let pos = (start_pos + written) % self.capacity;
            self.buffer[pos].store(sample);
            written += 1;

            // バッファフル検出
            if written >= self.available_write_space() {
                break;
            }
        }

        self.write_pos.store(
            (start_pos + written) % self.capacity,
            Ordering::Release,
        );

        written
    }

    pub fn read(&self, output: &mut [f32]) -> usize {
        let mut read = 0;
        let start_pos = self.read_pos.load(Ordering::Acquire);

        for sample in output.iter_mut() {
            if read >= self.available_read_space() {
                break;
            }

            let pos = (start_pos + read) % self.capacity;
            *sample = self.buffer[pos].load();
            read += 1;
        }

        self.read_pos.store(
            (start_pos + read) % self.capacity,
            Ordering::Release,
        );

        read
    }

    fn available_write_space(&self) -> usize {
        let write_pos = self.write_pos.load(Ordering::Relaxed);
        let read_pos = self.read_pos.load(Ordering::Relaxed);

        if write_pos >= read_pos {
            self.capacity - (write_pos - read_pos) - 1
        } else {
            read_pos - write_pos - 1
        }
    }

    fn available_read_space(&self) -> usize {
        let write_pos = self.write_pos.load(Ordering::Relaxed);
        let read_pos = self.read_pos.load(Ordering::Relaxed);

        if write_pos >= read_pos {
            write_pos - read_pos
        } else {
            self.capacity - (read_pos - write_pos)
        }
    }
}
```

## メモリ管理

### Zero-Copy 設計

```rust
use std::sync::Arc;
use std::ops::Range;

// ゼロコピーバッファ
#[derive(Debug, Clone)]
pub struct ZeroCopyBuffer<T> {
    data: Arc<Vec<T>>,
    range: Range<usize>,
}

impl<T> ZeroCopyBuffer<T> {
    pub fn new(data: Vec<T>) -> Self {
        let len = data.len();
        Self {
            data: Arc::new(data),
            range: 0..len,
        }
    }

    pub fn slice(&self, range: Range<usize>) -> Self {
        let start = self.range.start + range.start;
        let end = self.range.start + range.end.min(self.range.len());

        Self {
            data: Arc::clone(&self.data),
            range: start..end,
        }
    }

    pub fn as_slice(&self) -> &[T] {
        &self.data[self.range.clone()]
    }
}

// メモリプール
pub struct BufferPool<T> {
    free_buffers: SegQueue<Vec<T>>,
    buffer_size: usize,
}

impl<T: Default + Clone> BufferPool<T> {
    pub fn new(pool_size: usize, buffer_size: usize) -> Self {
        let free_buffers = SegQueue::new();

        for _ in 0..pool_size {
            free_buffers.push(vec![T::default(); buffer_size]);
        }

        Self {
            free_buffers,
            buffer_size,
        }
    }

    pub fn get(&self) -> Vec<T> {
        self.free_buffers
            .pop()
            .unwrap_or_else(|| vec![T::default(); self.buffer_size])
    }

    pub fn return_buffer(&self, mut buffer: Vec<T>) {
        buffer.clear();
        buffer.resize(self.buffer_size, T::default());
        self.free_buffers.push(buffer);
    }
}
```

### RAII リソース管理

```rust
// 音声リソースの自動管理
pub struct AudioResource {
    handle: Option<AudioHandle>,
    _phantom: std::marker::PhantomData<*const ()>,
}

impl AudioResource {
    pub fn new(handle: AudioHandle) -> Self {
        Self {
            handle: Some(handle),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn handle(&self) -> &AudioHandle {
        self.handle.as_ref().expect("Resource already consumed")
    }
}

impl Drop for AudioResource {
    fn drop(&mut self) {
        if let Some(handle) = self.handle.take() {
            // 確実なクリーンアップ
            handle.stop();
            handle.release_resources();
        }
    }
}

// スコープガード
pub struct ScopeGuard<F: FnOnce()> {
    cleanup: Option<F>,
}

impl<F: FnOnce()> ScopeGuard<F> {
    pub fn new(cleanup: F) -> Self {
        Self {
            cleanup: Some(cleanup),
        }
    }
}

impl<F: FnOnce()> Drop for ScopeGuard<F> {
    fn drop(&mut self) {
        if let Some(cleanup) = self.cleanup.take() {
            cleanup();
        }
    }
}

// 使用例
fn process_audio_file(path: &Path) -> Result<(), AudioError> {
    let file = File::open(path)?;
    let _guard = ScopeGuard::new(|| {
        // ファイルクリーンアップ処理
        cleanup_temp_files();
    });

    // ファイル処理...
    Ok(())
}
```

## パフォーマンス最適化

### CPU 最適化

```rust
// SIMD処理によるFFT高速化
use std::arch::x86_64::*;

#[target_feature(enable = "avx2")]
unsafe fn simd_multiply_add(a: &[f32], b: &[f32], c: &mut [f32]) {
    assert_eq!(a.len(), b.len());
    assert_eq!(a.len(), c.len());
    assert_eq!(a.len() % 8, 0);

    for i in (0..a.len()).step_by(8) {
        let va = _mm256_loadu_ps(a.as_ptr().add(i));
        let vb = _mm256_loadu_ps(b.as_ptr().add(i));
        let vc = _mm256_loadu_ps(c.as_ptr().add(i));

        let result = _mm256_fmadd_ps(va, vb, vc);
        _mm256_storeu_ps(c.as_mut_ptr().add(i), result);
    }
}

// 分岐予測最適化
#[inline(always)]
pub fn likely(condition: bool) -> bool {
    #[cold]
    fn cold() {}

    if !condition {
        cold();
    }
    condition
}

#[inline(always)]
pub fn unlikely(condition: bool) -> bool {
    #[cold]
    fn cold() {}

    if condition {
        cold();
    }
    condition
}
```

### メモリ最適化

```rust
// キャッシュ効率の良いデータレイアウト
#[repr(C)]
pub struct PackedAudioFrame {
    pub left: f32,
    pub right: f32,
    pub timestamp: u64,
} // 64-bit境界でアライン

// プリフェッチ
use std::intrinsics::prefetch_read_data;

pub fn optimized_process(data: &[f32]) {
    const PREFETCH_DISTANCE: usize = 64;

    for (i, chunk) in data.chunks(32).enumerate() {
        // 次のデータをプリフェッチ
        if i * 32 + PREFETCH_DISTANCE < data.len() {
            unsafe {
                prefetch_read_data(
                    data.as_ptr().add(i * 32 + PREFETCH_DISTANCE),
                    1, // 低局所性
                );
            }
        }

        // 実際の処理
        process_chunk(chunk);
    }
}
```

## エラー処理戦略

### エラー階層

```rust
use thiserror::Error;

// アプリケーションレベルエラー
#[derive(Error, Debug)]
pub enum ApplicationError {
    #[error("Audio engine error: {0}")]
    Audio(#[from] AudioError),

    #[error("Database error: {0}")]
    Database(#[from] DatabaseError),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("Plugin error: {plugin} - {message}")]
    Plugin { plugin: String, message: String },
}

// 音声エラー
#[derive(Error, Debug)]
pub enum AudioError {
    #[error("Decoder error: {0}")]
    Decoder(#[from] DecoderError),

    #[error("Device error: {0}")]
    Device(String),

    #[error("Format not supported: {format}")]
    UnsupportedFormat { format: String },

    #[error("Buffer underrun")]
    BufferUnderrun,

    #[error("Invalid state transition: {from} -> {to}")]
    InvalidState { from: String, to: String },
}

// エラー回復戦略
pub trait ErrorRecovery {
    type Error;

    fn can_recover(&self, error: &Self::Error) -> bool;
    fn recover(&mut self, error: Self::Error) -> Result<(), Self::Error>;
}

impl ErrorRecovery for AudioEngine {
    type Error = AudioError;

    fn can_recover(&self, error: &AudioError) -> bool {
        match error {
            AudioError::BufferUnderrun => true,
            AudioError::Device(_) => true,
            AudioError::UnsupportedFormat { .. } => false,
            _ => false,
        }
    }

    fn recover(&mut self, error: AudioError) -> Result<(), AudioError> {
        match error {
            AudioError::BufferUnderrun => {
                self.reset_buffers()?;
                Ok(())
            }
            AudioError::Device(_) => {
                self.reinitialize_device()?;
                Ok(())
            }
            _ => Err(error),
        }
    }
}
```

### 障害対応

```rust
// サーキットブレーカーパターン
#[derive(Debug)]
pub struct CircuitBreaker<F, E> {
    failure_threshold: usize,
    recovery_timeout: Duration,
    failure_count: AtomicUsize,
    last_failure: AtomicCell<Option<Instant>>,
    state: AtomicCell<CircuitState>,
    _phantom: std::marker::PhantomData<(F, E)>,
}

#[derive(Debug, Clone, Copy)]
enum CircuitState {
    Closed,   // 正常状態
    Open,     // 障害状態
    HalfOpen, // 回復試行状態
}

impl<F, E> CircuitBreaker<F, E>
where
    F: Fn() -> Result<(), E>,
    E: std::fmt::Debug,
{
    pub fn new(failure_threshold: usize, recovery_timeout: Duration) -> Self {
        Self {
            failure_threshold,
            recovery_timeout,
            failure_count: AtomicUsize::new(0),
            last_failure: AtomicCell::new(None),
            state: AtomicCell::new(CircuitState::Closed),
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn execute(&self, f: F) -> Result<(), CircuitBreakerError<E>> {
        match self.state.load() {
            CircuitState::Open => {
                if self.should_attempt_recovery() {
                    self.state.store(CircuitState::HalfOpen);
                } else {
                    return Err(CircuitBreakerError::CircuitOpen);
                }
            }
            CircuitState::Closed | CircuitState::HalfOpen => {}
        }

        match f() {
            Ok(()) => {
                self.on_success();
                Ok(())
            }
            Err(e) => {
                self.on_failure();
                Err(CircuitBreakerError::Execution(e))
            }
        }
    }

    fn should_attempt_recovery(&self) -> bool {
        if let Some(last_failure) = self.last_failure.load() {
            last_failure.elapsed() >= self.recovery_timeout
        } else {
            true
        }
    }

    fn on_success(&self) {
        self.failure_count.store(0, Ordering::Relaxed);
        self.state.store(CircuitState::Closed);
    }

    fn on_failure(&self) {
        self.last_failure.store(Some(Instant::now()));
        let failures = self.failure_count.fetch_add(1, Ordering::Relaxed) + 1;

        if failures >= self.failure_threshold {
            self.state.store(CircuitState::Open);
        }
    }
}
```

## セキュリティ

### ファイルシステムセキュリティ

```rust
use std::path::{Path, PathBuf};

pub struct SecurePathValidator {
    allowed_extensions: HashSet<&'static str>,
    max_file_size: u64,
    allowed_directories: Vec<PathBuf>,
}

impl SecurePathValidator {
    pub fn new() -> Self {
        Self {
            allowed_extensions: ["mp3", "flac", "wav", "ogg", "m4a"]
                .iter().copied().collect(),
            max_file_size: 500 * 1024 * 1024, // 500MB
            allowed_directories: vec![
                dirs::home_dir().unwrap_or_default().join("Music"),
            ],
        }
    }

    pub fn validate_audio_file(&self, path: &Path) -> Result<(), SecurityError> {
        // パストラバーサル攻撃防止
        let canonical = path.canonicalize()
            .map_err(|_| SecurityError::InvalidPath)?;

        // ディレクトリ制限チェック
        let allowed = self.allowed_directories.iter().any(|allowed_dir| {
            canonical.starts_with(allowed_dir)
        });

        if !allowed {
            return Err(SecurityError::UnauthorizedDirectory);
        }

        // 拡張子チェック
        if let Some(extension) = canonical.extension().and_then(|s| s.to_str()) {
            if !self.allowed_extensions.contains(extension.to_lowercase().as_str()) {
                return Err(SecurityError::UnsupportedFileType);
            }
        } else {
            return Err(SecurityError::NoFileExtension);
        }

        // ファイルサイズチェック
        let metadata = std::fs::metadata(&canonical)?;
        if metadata.len() > self.max_file_size {
            return Err(SecurityError::FileTooLarge);
        }

        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("Invalid file path")]
    InvalidPath,

    #[error("Unauthorized directory access")]
    UnauthorizedDirectory,

    #[error("Unsupported file type")]
    UnsupportedFileType,

    #[error("File has no extension")]
    NoFileExtension,

    #[error("File too large")]
    FileTooLarge,

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
```

### メモリセキュリティ

```rust
// 機密データの安全な消去
pub struct SecureBuffer {
    data: Vec<u8>,
}

impl SecureBuffer {
    pub fn new(size: usize) -> Self {
        Self {
            data: vec![0u8; size],
        }
    }

    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.data
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data
    }
}

impl Drop for SecureBuffer {
    fn drop(&mut self) {
        // メモリを確実に0クリア
        unsafe {
            std::ptr::write_bytes(
                self.data.as_mut_ptr(),
                0,
                self.data.len(),
            );
        }

        // コンパイラ最適化を防ぐ
        std::sync::atomic::fence(std::sync::atomic::Ordering::SeqCst);
    }
}
```

---

**最終更新**: 2025-08-07  
**バージョン**: 1.0  
**レビュアー**: システムアーキテクト
