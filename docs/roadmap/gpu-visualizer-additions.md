# GPU ビジュアライザー追加ロードマップ

既存のロードマップ (`docs/roadmap/README.md`) に対する GPU 高品質ビジュアライザー向けの Issue 追加計画。  
ユーザー要求: wgpu + WGSL シェーダー / Beat 検出・BPM / 全 4 ビジュアルスタイル / 全 4 ポストプロセス

---

## 対象マイルストーンと追加 Issue の対応

| マイルストーン       | 追加テーマ                            | Issue 数 |
|--------------------|--------------------------------------|---------|
| v0.2.0             | wgpu 基盤 + sonic-shader/sonic-visualizer クレート | 4       |
| v0.3.0             | SampleTap + GPU ユニフォーム + WGSL 共通ライブラリ  | 3       |
| v0.4.0             | Beat 検出 + BPM + GPU ビジュアライザー × 4        | 6       |
| v0.5.0             | HDR パイプライン + ポストプロセス × 4 + プリセット  | 6       |

**合計: 19 新規 Issue**

---

## v0.2.0 追加 Issue (wgpu Rendering Foundation)

### GPU-01: sonic-shader クレートの新規作成と wgpu 基盤構築

既存ロードマップ `v0.2.0` タスクの実装 Issue。

- `crates/sonic-shader/` 新規作成 (wgpu, bytemuck, pollster 依存)
- `RenderEngine` 構造体 (Device, Queue, Surface 管理)
- Slint `Window::set_rendering_notifier()` による統合
- フルスクリーンクワッドの WGSL (時間ベースグラデーション)
- FPS 計測 + デバッグオーバーレイ
- リサイズ対応 (Surface 再構築)

**依存**: v0.1.0 完了

---

### GPU-02: sonic-visualizer クレートの新規作成

- `crates/sonic-visualizer/` 新規作成
- `Visualizer` trait: `initialize / update_audio / render`
- `VisualizationConfig` (sensitivity, smoothing, band_count)
- `VisualizerRegistry` (name → factory)
- `SpectrumBarsVisualizer` のスケルトン (v0.3.0 で実装)

---

### GPU-03: SampleTap - ロックフリー音声サンプルキャプチャー

- `rodio::Source` ラッパーでサンプルをコピー
- `rtrb` (lock-free ring buffer) でオーディオスレッド → 解析スレッド転送
- バックプレッシャー対応 (FFT が遅れた場合のサンプルスキップ)
- チャネル数・サンプルレートのメタデータ伝播

**技術リスク**: rodio の `Source` trait からの raw サンプル取得タイミングに注意

---

### GPU-04: VisualizerUniforms GPU バッファパイプライン

- `#[repr(C)] VisualizerUniforms` 構造体 (`bytemuck::Pod`)
  - time, delta_time, resolution, spectrum[128], bass, mid, treble, volume, sensitivity, smoothing
- wgpu Uniform Buffer の作成・毎フレーム更新
- Controller → VisualizerEngine へのスペクトラムデータ配信
- bass/mid/treble を帯域別 RMS から算出

---

## v0.3.0 追加 Issue (Spectrum Bars GPU + WGSL Library)

### GPU-05: 共通 WGSL シェーダーライブラリの実装

- `common.wgsl`: HSV⇔RGB, palette (neon/fire/spectrum), remap, polar 座標
- `noise.wgsl`: Simplex noise, Perlin noise, FBM (Fractal Brownian Motion)
- `sdf.wgsl`: 円・矩形・線の SDF + smooth union
- `build.rs` による WGSL ファイル結合 (include 代替)
- shaders カタログ: `docs/specs/shader-catalog.md` 準拠

---

### GPU-06: WGSL シェーダーホットリロードシステム

- `notify` クレートで `.wgsl` ファイル変更を検知 (debug build のみ)
- 変更検知 → シェーダー再コンパイル → パイプライン差し替え
- コンパイルエラー時は旧シェーダーを継続使用 (画面フリーズ防止)
- エラー内容をログ出力 (行番号付き)

---

### GPU-07: 2D Spectrum Bars WGSL シェーダー実装

**注**: 既存 Issue #27 は Slint ベース想定のため、WGSL GPU 版として別途起票。

- `spectrum_bars.wgsl` (vertex + fragment)
- Instanced quad: バー 1 本 = 1 インスタンス
- `spectrum[i]` → バー高さ (sensitivity 倍率適用)
- フレーム間 smoothing (uniform.smoothing で lerp)
- グラデーションカラー (`palette_neon`)
- HDR 値 > 1.0 で Bloom 用ハイライト

---

## v0.4.0 追加 Issue (Beat Detection + GPU Visualizer Variety)

### GPU-08: Beat/Onset 検出の実装

`docs/specs/beat-detection.md` 準拠。

- Spectral Flux 法による帯域別オンセット検出
  - Kick (20-80Hz), Snare (80-300Hz), Hihat (8kHz+)
- 適応的閾値 + ピークピッキング
- `BeatEvent { kick, snare, hihat, intensity }` データ構造
- `OnsetDetector` を `crates/sonic-core/src/audio/onset_detector.rs` に実装
- 単体テスト (既知 BPM の WAV ファイルで検証)

---

### GPU-09: BPM 推定とテンポトラッキング

- IOI ヒストグラム法 (Inter-Onset Interval)
- 60-200 BPM 範囲、1 BPM 分解能
- 指数移動平均でテンポ安定化
- `bpm_phase` 計算 (0.0-1.0、1 ビート周期内位相)
- `BpmResult { bpm, confidence, beat_phase }` データ構造

---

### GPU-10: AudioUniforms ビート・BPM 拡張とシェーダー配信

- `VisualizerUniforms` に beat_kick/beat_snare/beat_hihat/bpm/bpm_phase を追加
- Beat 時 1.0 → 指数減衰 (kick: τ≈80ms, snare: τ≈60ms, hihat: τ≈40ms)
- `docs/specs/beat-detection.md` の WGSL 利用例に準拠
- Uniform Buffer 毎フレーム更新 (Beat データ含む)

---

### GPU-11: 3D Spectrum Bars ビジュアライザー (wgpu インスタンス描画)

`docs/specs/shader-catalog.md` §2.2 準拠。

- `spectrum_bars_3d.wgsl` (vertex + fragment)
- wgpu Instanced Draw: 128 バーを 1 ドローコール
- 3D unit box ジオメトリ、band_idx → X 座標マッピング
- Camera: perspective projection + BPM 位相に応じた Y 軸自動周回
- 反射床: Y < 0 フラグメントで反転サンプリング
- beat_kick でカメラ Z 方向振動

---

### GPU-12: GPU パーティクルシステムビジュアライザー

`docs/specs/shader-catalog.md` §2.3 準拠。

- `particles_compute.wgsl`: コンピュートシェーダー (100,000 パーティクル)
- `particles.wgsl`: パーティクル描画 (ポイントスプライト or 小 quad)
- Particle struct: pos, vel, life, max_life, color, size
- 物理: 重力, ドラッグ, 音声引力 (bass → 中心引力)
- Beat 連動: beat_kick → 爆発的スポーン, beat_hihat → きらめき
- wgpu Storage Buffer (read/write) × 2 (ダブルバッファ)

---

### GPU-13: プロシージャル/フラクタルビジュアライザー

`docs/specs/shader-catalog.md` §2.4 準拠。

- `procedural.wgsl`: フルスクリーンフラグメントシェーダー
- FBM + Domain Warping (bass で歪み量制御)
- BPM 位相で色相回転
- beat_kick でコントラスト/輝度フラッシュ
- 4 モード: FBM, Julia Set, Mandelbrot, Plasma

---

### GPU-14: 3D Waterfall Spectrogram ビジュアライザー

`docs/specs/shader-catalog.md` §2.5 準拠。

- スペクトラム履歴テクスチャ (128×256 px, 毎フレーム 1 行シフト)
- 3D グリッドメッシュ + 頂点変位シェーダー
- 熱マップカラーマップ (青→シアン→緑→黄→赤)
- 時間軸 (Z 方向) に過去スペクトラムが積み重なる

---

## v0.5.0 追加 Issue (Post-processing Pipeline)

### GPU-15: HDR フレームバッファとマルチパスパイプラインの構築

`docs/specs/post-processing-pipeline.md` §1-§2 準拠。

- `RGBA16Float` レンダーターゲット
- `RenderTargetPool`: リサイズ時に自動再作成
- `PostProcessPipeline` トレイトと Pass チェーン管理
- ACES Filmic トーンマッピング (+ Reinhard フォールバック)
- LDR 非対応デバイスへのフォールバック

---

### GPU-16: Bloom / グローポストプロセスエフェクト

`docs/specs/post-processing-pipeline.md` §3.1 / `docs/specs/shader-catalog.md` §3.1 準拠。

- `bloom_threshold.wgsl`: 輝度閾値抽出 (ソフトニー)
- `bloom_downsample.wgsl`: Dual Kawase 5 段階ダウンサンプル
- `bloom_upsample.wgsl`: 5 段階アップサンプル
- `bloom_composite.wgsl`: 加算合成
- 強度パラメーター: threshold, intensity

---

### GPU-17: モーションブラー / フレームトレイルエフェクト

`docs/specs/post-processing-pipeline.md` §3.2 準拠。

- `motion_blur.wgsl`: テンポラル蓄積バッファ
- ブレンド係数 0.85 (設定可能)
- beat_kick 時に decay 加速 (鋭い切れ目)
- accumulation テクスチャ管理 (ping-pong)

---

### GPU-18: 色収差・カラーグレーディング・ビネット

`docs/specs/post-processing-pipeline.md` §3.x / `docs/specs/shader-catalog.md` §3.3 準拠。

- `color_grade.wgsl`: 色収差 + コントラスト/彩度 + ビネット
- beat_kick で CA 強度を一時増幅
- パラメーター: ca_strength, contrast, saturation, vignette_inner/outer

---

### GPU-19: CRT スキャンライン・グリッチ・フィルムグレイン

`docs/specs/shader-catalog.md` §3.4 準拠。

- `crt_glitch.wgsl`: スキャンライン + 水平ピクセルシフト + フィルムグレイン
- beat_kick 連動グリッチ (glitch_strength = beat_kick * 0.03)
- 各エフェクト個別 ON/OFF 設定
- デフォルト OFF (ユーザー好みが分かれるため)

---

### GPU-20: ビジュアルプリセットシステム (エフェクトスタック設定保存)

`docs/specs/post-processing-pipeline.md` §5 準拠。

- 6 組み込みプリセット: Default / Neon / Cyberpunk / Minimal / Cinematic / Retro
- TOML 形式でエフェクトパラメーターを保存
- ユーザーカスタムプリセットの保存・読み込み
- Visualizer 切り替え時にプリセットを自動適用
- UI: プリセット選択ドロップダウン + カスタム保存ボタン

---

## 既存 Issue との関係

| 既存 Issue | 状態      | 理由                                          |
|----------|----------|----------------------------------------------|
| #27 Spectrum Bars (Slint) | **置き換え** | GPU-07 が WGSL 版として完全に代替              |
| #30 VisualizerPlugin trait | **更新要** | `CanvasCommand` 方式は GPU 版と非互換、GPU-02 で設計変更 |
| #31 PluginManager | **更新要** | GPU 版は `VisualizerRegistry` で管理           |
| #32 Waveform (Slint) | **継続**  | waveform.wgsl は v0.4.0 で GPU 版を実装       |
| #46 Criterion ベンチマーク | **拡張**  | GPU シェーダーのベンチマーク項目を追加           |

---

## 依存グラフ

```
GPU-01 (sonic-shader)
  └─ GPU-02 (sonic-visualizer)
       └─ GPU-03 (SampleTap)
            └─ GPU-04 (Uniforms Pipeline)
                 └─ GPU-05 (WGSL Library)
                      ├─ GPU-06 (Hot-reload)
                      └─ GPU-07 (Spectrum Bars 2D)
                           ├─ GPU-08 (Beat Detection)
                           │    └─ GPU-09 (BPM)
                           │         └─ GPU-10 (Beat Uniforms)
                           │              ├─ GPU-11 (3D Spectrum Bars)
                           │              ├─ GPU-12 (Particles)
                           │              ├─ GPU-13 (Procedural)
                           │              └─ GPU-14 (Waterfall)
                           └─ GPU-15 (HDR Pipeline)
                                ├─ GPU-16 (Bloom)
                                ├─ GPU-17 (Motion Blur)
                                ├─ GPU-18 (Color Grade)
                                └─ GPU-19 (CRT Glitch)
                                     └─ GPU-20 (Presets)
```
