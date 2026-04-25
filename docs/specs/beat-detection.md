# Beat Detection & Advanced Audio Analysis

## 概要

FFT スペクトラム解析に加えて、Beat/Onset 検出・BPM 推定・テンポトラッキングを実装する。  
これらのデータは GPU シェーダーへ `AudioUniforms` として毎フレーム転送され、Beat に同期した視覚表現を実現する。

---

## 1. 解析パイプライン全体像

```
Audio Thread (rodio Source)
  └─ SampleTap (lock-free ring buffer)
       └─ Analysis Thread (60Hz)
            ├─ SpectrumAnalyzer (FFT)
            │    └─ spectrum[128], bass, mid, treble, volume
            ├─ OnsetDetector
            │    └─ beat_kick, beat_snare, beat_hihat (bool)
            ├─ BpmEstimator
            │    └─ bpm (f32), bpm_confidence, beat_phase
            └─ AudioAnalysisResult
                 └─ CPU → GPU (AudioUniforms buffer upload, 60Hz)
```

---

## 2. Onset / Beat 検出

### 2.1 Spectral Flux 法

各フレームの FFT マグニチュード差分の正の増加分を合計してオンセット強度を求める。

```rust
/// スペクトラルフラックス: 正方向のエネルギー増加量
fn spectral_flux(current: &[f32], previous: &[f32]) -> f32 {
    current
        .iter()
        .zip(previous.iter())
        .map(|(c, p)| (c - p).max(0.0))
        .sum()
}
```

### 2.2 帯域別オンセット検出

| 帯域名    | 周波数範囲     | 検出対象       |
|----------|--------------|--------------|
| Kick     | 20 – 80 Hz   | バスドラム      |
| Snare    | 80 – 300 Hz  | スネアドラム    |
| Hihat    | 8,000 Hz +   | ハイハット      |

各帯域でそれぞれ独立した Spectral Flux を計算する。

### 2.3 適応的閾値とピークピッキング

```rust
struct OnsetDetector {
    /// 帯域ごとの Spectral Flux 履歴 (直近 N フレーム)
    flux_history: [VecDeque<f32>; 3],  // [kick, snare, hihat]
    /// 適応的閾値の乗数
    threshold_multiplier: f32,          // デフォルト 1.5
    /// 前回 Beat から最小フレーム数 (BPM 上限 = 200bpm 対応)
    min_interval_frames: usize,         // 60fps で 18 フレーム ≈ 300ms
}

impl OnsetDetector {
    fn detect(&mut self, spectrum: &[f32]) -> BeatEvent {
        // 1. 各帯域の spectral flux 計算
        // 2. 局所平均 + threshold_multiplier で閾値設定
        // 3. flux > threshold かつ min_interval 経過 → Beat 検出
    }
}
```

### 2.4 BeatEvent

```rust
#[derive(Debug, Clone, Default)]
pub struct BeatEvent {
    pub kick:     bool,
    pub snare:    bool,
    pub hihat:    bool,
    pub intensity: f32,   // 0.0-1.0 (フラックスの正規化値)
}
```

---

## 3. BPM 推定とテンポトラッキング

### 3.1 IOI ヒストグラム法 (Inter-Onset Interval)

```
オンセット検出 → タイムスタンプ記録
                   └─ 直近オンセット間の間隔 (IOI) を計算
                        └─ ヒストグラムの最頻値 → BPM 候補
                             └─ 指数移動平均でテンポ安定化
```

- 解析ウィンドウ: 直近 4 秒のオンセット
- BPM 範囲: 60 – 200 BPM
- 分解能: 1 BPM

### 3.2 BpmEstimator

```rust
pub struct BpmEstimator {
    onset_times: VecDeque<Instant>,    // 直近オンセットタイムスタンプ
    current_bpm: f32,                   // 推定 BPM (スムージング済)
    confidence: f32,                    // 0.0-1.0
}

impl BpmEstimator {
    pub fn update(&mut self, event: &BeatEvent, now: Instant) -> BpmResult;
}

pub struct BpmResult {
    pub bpm: f32,         // 推定 BPM
    pub confidence: f32,  // 信頼度
    pub beat_phase: f32,  // 0.0-1.0 (1 ビート周期内の位相)
}
```

---

## 4. AudioAnalysisResult (統合データ構造)

```rust
#[derive(Debug, Clone, Default)]
pub struct AudioAnalysisResult {
    // --- FFT スペクトラム ---
    pub spectrum:    Vec<f32>,   // 128 バンド、0.0-1.0
    pub bass:        f32,        // 20-250Hz RMS
    pub mid:         f32,        // 250-4kHz RMS
    pub treble:      f32,        // 4kHz-20kHz RMS
    pub volume:      f32,        // 全体 RMS
    pub peak_level:  f32,        // ピークレベル

    // --- Beat ---
    pub beat:        BeatEvent,

    // --- BPM ---
    pub bpm:         f32,
    pub bpm_confidence: f32,
    pub beat_phase:  f32,       // 0.0-1.0
}
```

---

## 5. GPU への AudioUniforms 拡張

既存 `docs/specs/graphics.md` の `VisualizerUniforms` に以下を追加:

```rust
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VisualizerUniforms {
    // === 既存フィールド ===
    pub time:         f32,
    pub delta_time:   f32,
    pub resolution:   [f32; 2],
    pub spectrum:     [f32; 128],
    pub bass:         f32,
    pub mid:          f32,
    pub treble:       f32,
    pub volume:       f32,
    pub sensitivity:  f32,
    pub smoothing:    f32,
    pub color_scheme: u32,
    pub _pad0:        u32,

    // === 追加: Beat & BPM ===
    /// キック検出: Beat 時 1.0、指数減衰 (τ ≈ 80ms)
    pub beat_kick:    f32,
    /// スネア検出: Beat 時 1.0、指数減衰 (τ ≈ 60ms)
    pub beat_snare:   f32,
    /// ハイハット検出: Beat 時 1.0、指数減衰 (τ ≈ 40ms)
    pub beat_hihat:   f32,
    /// 推定 BPM (60-200)
    pub bpm:          f32,
    /// ビート位相 0.0-1.0 (sin(2π * time * bpm/60) でシェーダー内アニメーション)
    pub beat_phase:   f32,
    /// BPM 信頼度 0.0-1.0
    pub bpm_confidence: f32,
    pub _pad1:        [f32; 2],
}
```

### Beat ユニフォームのシェーダー利用例 (WGSL)

```wgsl
// Beat に同期したスケールアニメーション
fn beat_scale(uniforms: VisualizerUniforms) -> f32 {
    return 1.0 + uniforms.beat_kick * 0.3;
}

// BPM に同期した周期アニメーション
fn bpm_pulse(uniforms: VisualizerUniforms) -> f32 {
    return sin(uniforms.beat_phase * 6.28318) * 0.5 + 0.5;
}
```

---

## 6. パフォーマンス要件

| 処理                              | 目標レイテンシ |
|----------------------------------|--------------|
| Onset 検出レイテンシ               | ≤ 20ms       |
| BPM 推定収束時間 (曲開始後)         | ≤ 4 秒       |
| AudioAnalysisResult → GPU 転送   | ≤ 1ms        |
| Beat 検出 CPU 使用率増加           | ≤ 1%         |

---

## 7. 実装ファイル

```
crates/sonic-core/src/audio/
├── analysis.rs          # 既存 SpectrumAnalyzer
├── onset_detector.rs    # 新規: OnsetDetector
├── bpm_estimator.rs     # 新規: BpmEstimator
└── analysis_result.rs   # 新規: AudioAnalysisResult (統合)
```
