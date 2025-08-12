# Sonic Flow - システム設計詳細

## 📋 目次

1. [システム概要](#システム概要)
2. [技術スタック](#技術スタック)
3. [データベース設計](#データベース設計)
4. [並行処理設計](#並行処理設計)
5. [メモリ管理](#メモリ管理)
6. [パフォーマンス最適化](#パフォーマンス最適化)
7. [エラー処理戦略](#エラー処理戦略)
8. [セキュリティ設計](#セキュリティ設計)
9. [監視システム](#監視システム)

## システム概要

### アーキテクチャ概要

Sonic Flow は、音圧ビジュアライザーを中心とした高性能ミュージックプレイヤーです。Rust の型安全性とゼロコスト抽象化を活用し、リアルタイム音声処理とビジュアル表現を実現します。

### 設計原則

1. **型安全性**: コンパイル時エラー検出による堅牢性
2. **ゼロコピー**: 不要なメモリコピーの排除
3. **並行性**: データ競合のない並列処理
4. **拡張性**: プラグインシステムによる機能追加
5. **リアルタイム性**: 音声処理の低レイテンシ保証

### システム品質要件

```rust
pub struct SystemRequirements {
    pub max_audio_latency: Duration,      // 50ms
    pub min_frame_rate: f32,              // 60fps
    pub max_memory_usage: usize,          // 200MB
    pub max_cpu_usage: f32,               // 10%
    pub startup_time: Duration,           // 3s
}
```

## 技術スタック

### コア依存関係

```toml
[dependencies]
# UI Framework
slint = "1.6"

# 非同期処理
tokio = { version = "1.35", features = ["full", "rt-multi-thread"] }
futures = "0.3"

# 音声処理
rodio = "0.17"           # クロスプラットフォーム音声再生
symphonia = "0.5"        # 音声デコード
cpal = "0.15"            # 低レベル音声I/O
rustfft = "6.2"          # FFT計算

# データ管理
sqlx = { version = "0.7", features = ["sqlite", "chrono", "uuid"] }
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"             # 設定ファイル

# エラーハンドリング
anyhow = "1.0"
thiserror = "1.0"

# 並行処理
crossbeam = "0.8"        # 並行データ構造
parking_lot = "0.12"     # 高性能Mutex
dashmap = "5.5"          # 並行HashMap
rayon = "1.8"            # データ並列処理

# セキュリティ
ring = "0.17"            # 暗号化
sha2 = "0.10"            # ハッシュ関数
```

### プラットフォーム別依存関係

```toml
# Windows
[target.'cfg(windows)'.dependencies]
winapi = { version = "0.3", features = ["winmm", "wasapi"] }
windows = { version = "0.52", features = ["Win32_Media_Audio"] }

# macOS
[target.'cfg(target_os = "macos")'.dependencies]
core-audio = "0.11"
coreaudio-sys = "0.2"

# Linux
[target.'cfg(unix)'.dependencies]
alsa = "0.7"
pulse = "2.5"
```

## データベース設計

### スキーマ最適化

```sql
-- 高速検索用インデックス
CREATE INDEX idx_tracks_album_order ON tracks(artist, album, track_number);
CREATE INDEX idx_tracks_rated ON tracks(rating DESC) WHERE rating IS NOT NULL;
CREATE INDEX idx_tracks_title_lower ON tracks(LOWER(title));

-- FTS検索
CREATE VIRTUAL TABLE tracks_fts USING fts5(
    title, artist, album, genre,
    content='tracks', content_rowid='rowid'
);

-- 統計テーブル
CREATE TABLE playback_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    track_id TEXT NOT NULL,
    played_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    duration_played INTEGER NOT NULL,
    completion_percentage REAL NOT NULL
);
```

### データベース最適化

```rust
pub struct DatabaseManager {
    pool: Arc<SqlitePool>,
    cache: Arc<DashMap<String, CachedQuery>>,
}

impl DatabaseManager {
    pub async fn new(database_url: &str) -> Result<Self, DatabaseError> {
        let pool = SqlitePoolOptions::new()
            .max_connections(10)
            .connect(database_url).await?;

        // WALモード + 最適化
        sqlx::query("PRAGMA journal_mode = WAL").execute(&pool).await?;
        sqlx::query("PRAGMA synchronous = NORMAL").execute(&pool).await?;
        sqlx::query("PRAGMA cache_size = -64000").execute(&pool).await?;

        Ok(Self {
            pool: Arc::new(pool),
            cache: Arc::new(DashMap::new()),
        })
    }

    // バッチ処理による高速挿入
    pub async fn batch_insert_tracks(&self, tracks: Vec<TrackInfo>) -> Result<(), DatabaseError> {
        let mut tx = self.pool.begin().await?;

        for chunk in tracks.chunks(100) {
            // バルクインサート実装
        }

        tx.commit().await?;
        Ok(())
    }
}
```

## 並行処理設計

### スレッドプール構成

```rust
pub struct AudioThreadPool {
    // 音声処理専用（高優先度）
    audio_runtime: Runtime,
    // ビジュアライザー処理（中優先度）
    visual_runtime: Runtime,
    // I/O処理（低優先度）
    io_runtime: Runtime,
}

impl AudioThreadPool {
    pub fn new() -> Result<Self, ThreadPoolError> {
        // 音声処理用高優先度ランタイム
        let audio_runtime = Builder::new_multi_thread()
            .worker_threads(2)
            .thread_name("audio-worker")
            .on_thread_start(|| {
                #[cfg(unix)]
                unsafe {
                    libc::pthread_setschedparam(
                        libc::pthread_self(),
                        libc::SCHED_FIFO,
                        &libc::sched_param { sched_priority: 80 },
                    );
                }
            })
            .build()?;

        Ok(Self { audio_runtime, /* ... */ })
    }
}
```

### Lock-Free データ構造

```rust
// Triple Buffering for Audio
pub struct TripleBuffer<T> {
    buffers: [AtomicCell<Option<T>>; 3],
    write_index: AtomicUsize,
    read_index: AtomicUsize,
}

impl<T> TripleBuffer<T> {
    pub fn write(&self, data: T) {
        let write_idx = self.write_index.load(Ordering::Relaxed);
        self.buffers[write_idx].store(Some(data));
        self.rotate_indices();
    }

    pub fn read(&self) -> Option<T> {
        let read_idx = self.read_index.load(Ordering::Acquire);
        self.buffers[read_idx].take()
    }
}

// 高性能リングバッファ
pub struct LockFreeRingBuffer<T> {
    buffer: Vec<AtomicCell<MaybeUninit<T>>>,
    write_pos: AtomicUsize,
    read_pos: AtomicUsize,
    capacity: usize,
}
```

## メモリ管理

### メモリプール

```rust
pub struct AudioMemoryPool {
    small_blocks: SegQueue<Vec<f32>>,    // 1KB
    medium_blocks: SegQueue<Vec<f32>>,   // 4KB
    large_blocks: SegQueue<Vec<f32>>,    // 16KB
}

impl AudioMemoryPool {
    pub fn allocate(&self, size: usize) -> PooledBuffer {
        let buffer = match size {
            0..=256 => self.small_blocks.pop().unwrap_or_else(|| vec![0.0f32; 256]),
            257..=1024 => self.medium_blocks.pop().unwrap_or_else(|| vec![0.0f32; 1024]),
            _ => self.large_blocks.pop().unwrap_or_else(|| vec![0.0f32; 4096]),
        };

        PooledBuffer::new(buffer, self)
    }
}

// RAII メモリ管理
pub struct PooledBuffer {
    buffer: Option<Vec<f32>>,
    pool: *const AudioMemoryPool,
}

impl Drop for PooledBuffer {
    fn drop(&mut self) {
        if let Some(buffer) = self.buffer.take() {
            unsafe { (*self.pool).return_buffer(buffer); }
        }
    }
}
```

### Zero-Copy バッファ

```rust
#[derive(Clone)]
pub struct SharedAudioBuffer {
    data: Arc<[f32]>,
    offset: usize,
    length: usize,
    sample_rate: u32,
    channels: u8,
}

impl SharedAudioBuffer {
    pub fn slice(&self, start: usize, end: usize) -> Self {
        Self {
            data: Arc::clone(&self.data),
            offset: self.offset + start,
            length: end - start,
            sample_rate: self.sample_rate,
            channels: self.channels,
        }
    }
}
```

## パフォーマンス最適化

### SIMD 最適化

```rust
pub struct SIMDAudioProcessor {
    #[cfg(target_arch = "x86_64")]
    supports_avx2: bool,
}

impl SIMDAudioProcessor {
    #[cfg(target_arch = "x86_64")]
    #[target_feature(enable = "avx2")]
    unsafe fn mix_channels_avx2(&self, left: &mut [f32], right: &[f32], volume: f32) {
        use std::arch::x86_64::*;

        let volume_vec = _mm256_set1_ps(volume);

        for chunk in left.chunks_exact_mut(8).zip(right.chunks_exact(8)) {
            let left_vec = _mm256_loadu_ps(chunk.0.as_ptr());
            let right_vec = _mm256_loadu_ps(chunk.1.as_ptr());
            let mixed = _mm256_fmadd_ps(right_vec, volume_vec, left_vec);
            _mm256_storeu_ps(chunk.0.as_mut_ptr(), mixed);
        }
    }
}
```

### キャッシュ最適化

```rust
#[repr(align(64))] // キャッシュライン境界
pub struct CacheOptimizedSpectrumData {
    pub magnitudes: Vec<f32>,      // ホットデータ
    pub timestamp: Instant,

    _padding: [u8; 32],            // false sharing防止
    pub frequencies: Vec<f32>,      // コールドデータ
    pub metadata: SpectrumMetadata,
}
```

## エラー処理戦略

### 階層化エラー処理

```rust
#[derive(Error, Debug)]
pub enum SonicFlowError {
    #[error("Critical system error: {0}")]
    Critical(#[from] CriticalError),

    #[error("Recoverable error: {message} (attempt {attempt}/{max_attempts})")]
    Recoverable {
        message: String,
        attempt: u32,
        max_attempts: u32,
        #[source] source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("Audio error: {0}")]
    Audio(#[from] AudioError),
}
```

### サーキットブレーカー

```rust
pub struct CircuitBreaker {
    state: Arc<Mutex<CircuitState>>,
    failure_threshold: usize,
    failure_count: AtomicUsize,
}

impl CircuitBreaker {
    pub async fn execute<F, T, E>(&self, operation: F) -> Result<T, CircuitBreakerError<E>>
    where F: Future<Output = Result<T, E>>
    {
        match self.current_state().await {
            CircuitState::Open => Err(CircuitBreakerError::CircuitOpen),
            _ => {
                match operation.await {
                    Ok(result) => {
                        self.on_success().await;
                        Ok(result)
                    }
                    Err(error) => {
                        self.on_failure().await;
                        Err(CircuitBreakerError::OperationFailed(error))
                    }
                }
            }
        }
    }
}
```

## セキュリティ設計

### プラグインサンドボックス

```rust
pub struct PluginSandbox {
    security_policy: SecurityPolicy,
    memory_limiter: MemoryLimiter,
    cpu_limiter: CpuTimeLimiter,
    file_access_controller: FileAccessController,
}

impl PluginSandbox {
    pub async fn execute_plugin<T>(
        &mut self,
        plugin: &mut dyn VisualizerPlugin,
        input: T,
    ) -> Result<T::Output, SecurityError> {
        // 1. プラグイン署名検証
        self.verify_plugin_signature(plugin)?;

        // 2. リソース制限設定
        let _guards = (
            self.memory_limiter.create_guard()?,
            self.cpu_limiter.create_guard()?,
            self.file_access_controller.create_guard()?,
        );

        // 3. 隔離された実行
        let result = tokio::task::spawn_blocking(move || {
            plugin.execute(input)
        }).await??;

        Ok(result)
    }
}
```

### 暗号化とハッシュ

```rust
pub struct SecurityManager {
    rng: SystemRandom,
    signing_key: Vec<u8>,
}

impl SecurityManager {
    pub fn verify_plugin_signature(&self, plugin_path: &Path) -> Result<(), SecurityError> {
        use ring::signature;

        let public_key = signature::UnparsedPublicKey::new(
            &signature::RSA_PKCS1_2048_8192_SHA256,
            &self.signing_key
        );

        let plugin_hash = self.calculate_file_hash(plugin_path)?;
        let signature = self.read_signature(plugin_path)?;

        public_key.verify(&plugin_hash, &signature)
            .map_err(|_| SecurityError::InvalidSignature)?;

        Ok(())
    }
}
```

## 監視システム

### メトリクス収集

```rust
pub struct MetricsCollector {
    cpu_usage: MovingAverage,
    memory_usage: MovingAverage,
    audio_latency: Histogram,
    frame_render_time: Histogram,
    error_rates: HashMap<String, Counter>,
}

impl MetricsCollector {
    pub async fn start_collection(&mut self, interval: Duration) {
        let mut timer = tokio::time::interval(interval);

        loop {
            timer.tick().await;
            self.collect_system_metrics().await;
            self.export_metrics().await;
        }
    }

    pub fn record_audio_latency(&self, latency: Duration) {
        self.audio_latency.record(latency.as_millis() as f64);
    }
}
```

### アラートシステム

```rust
pub struct AlertManager {
    alert_rules: Vec<AlertRule>,
    active_alerts: DashMap<String, ActiveAlert>,
}

#[derive(Debug, Clone)]
pub enum AlertCondition {
    CpuUsageHigh { threshold: f64 },
    MemoryUsageHigh { threshold: f64 },
    AudioLatencyHigh { threshold_ms: f64 },
    ErrorRateHigh { threshold_per_minute: f64 },
}

impl AlertManager {
    pub async fn evaluate_metrics(&self, metrics: &MetricsSnapshot) {
        for rule in &self.alert_rules {
            let should_fire = rule.condition.evaluate(metrics);

            if should_fire && !self.active_alerts.contains_key(&rule.id) {
                self.fire_alert(rule).await;
            } else if !should_fire && self.active_alerts.contains_key(&rule.id) {
                self.resolve_alert(rule).await;
            }
        }
    }
}
```

### 診断システム

```rust
pub struct DiagnosticsEngine {
    system_profiler: SystemProfiler,
    performance_analyzer: PerformanceAnalyzer,
    error_analyzer: ErrorAnalyzer,
}

impl DiagnosticsEngine {
    pub async fn run_diagnosis(&self) -> DiagnosisReport {
        let (system_info, performance, errors) = tokio::join!(
            self.system_profiler.collect_info(),
            self.performance_analyzer.analyze(),
            self.error_analyzer.analyze_recent()
        );

        DiagnosisReport {
            timestamp: Utc::now(),
            system_info: system_info?,
            performance: performance?,
            errors: errors?,
            recommendations: self.generate_recommendations(&performance),
        }
    }
}
```

---

**最終更新**: 2025-08-12  
**バージョン**: 2.0 (コンパクト版)  
**管理者**:Claude (ディレクトリ構成再整理)
