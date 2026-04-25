# v0.3.0 - 初のビジュアライザー (Spectrum Bars)

## 概要

「音声 → FFT → GPU → 描画」の完全なエンドツーエンドパイプラインを構築する。最もオーソドックスなスペクトラムバーズビジュアライザーを WGSL で実装し、音楽再生中に音圧に応じてバーが動く状態を実現する。

このマイルストーンは**プロジェクトの核**。

## ゴール

- 音楽再生中、Spectrum Bars が音圧に反応して動く
- 周波数帯域 (低音=左、高音=右) が視覚的に正しい
- Sensitivity / Smoothing スライダーが効く
- オーディオ遅延 50ms 以下を維持

## スコープ

### In Scope

- `crates/sonic-visualizer` の新規作成
- 音声サンプル取得経路の確立
- FFT 解析パイプライン
- WGSL spectrum_bars シェーダー
- Slint UI の sensitivity / smoothing コントロール統合

### Out of Scope

- 他の visualizer 種類 (v0.4.0)
- ポストプロセス (v0.5.0)
- ビジュアライザー切り替え (v0.4.0)

## タスク

### `crates/sonic-visualizer` 新規作成

- [ ] 空クレート作成 (lib.rs + Cargo.toml)
- [ ] `Visualizer` trait (initialize / update_audio / render)
- [ ] `VisualizationConfig` (sensitivity, smoothing, band_count, etc.)
- [ ] `SpectrumBarsVisualizer` 実装

### 音声サンプル取得

- [ ] rodio Source をラップする `SampleTap` (再生と並行してサンプルをコピー)
- [ ] Ring buffer (lock-free, e.g. `rtrb`) でオーディオスレッド → FFT スレッド転送
- [ ] FFT 解析タスク (60Hz で `SpectrumAnalyzer::analyze`)
- [ ] バックプレッシャー対応 (FFT が遅れた場合のサンプルスキップ)

### WGSL シェーダー実装

- [ ] `crates/sonic-shader/src/shaders/spectrum_bars.wgsl`
- [ ] Vertex shader: instanced bars または segmented quad
- [ ] Fragment shader: バーごとのグラデーション + グロー初期実装
- [ ] 共通ユーティリティ `common.wgsl` (HSV, smoothstep, palette)

### Uniform Buffer

- [ ] `VisualizerUniforms` struct (`#[repr(C)]` + `bytemuck::Pod`)
  - `time: f32`, `delta_time: f32`
  - `resolution: [f32; 2]`
  - `spectrum: [f32; 128]`
  - `bass / mid / treble / volume: f32`
  - `sensitivity / smoothing: f32`
- [ ] CPU → GPU 転送 (毎フレーム or 変更時)

### パイプライン統合

- [ ] Controller → Visualizer への spectrum data 配信
- [ ] FFT 結果から bass/mid/treble 算出 (帯域別 RMS)
- [ ] Smoothing (前フレームとの線形補間)

### UI コントロール統合

- [ ] Sensitivity スライダー → uniform.sensitivity 反映
- [ ] Smoothing スライダー → uniform.smoothing 反映
- [ ] Visualizer Type Combo (現状は "Spectrum Bars" のみ)

### テスト

- [ ] 1kHz サイン波で適切な周波数位置にピーク表示
- [ ] 無音時にバーが沈む
- [ ] Sensitivity 0.1 と 5.0 で見た目が大きく異なる
- [ ] 60fps 維持 (再生 + ビジュアライザー同時)

## 完了条件

- [ ] 音楽再生で Spectrum Bars が動く
- [ ] 周波数帯域が視覚的に正しい
- [ ] Sensitivity / Smoothing スライダーが効く
- [ ] 60fps 維持
- [ ] オーディオ遅延 50ms 以下
- [ ] FFT 処理が再生をブロックしない

## 依存

- v0.1.0 (再生機能の安定)
- v0.2.0 (wgpu レンダリング基盤)

## リスク / 技術的検討事項

- **rodio から raw samples の取り出し**: `Source` trait の実装で `next()` を tap する形が最も自然。ただし f32 への変換タイミング、チャネル数の扱いに注意
- **同期性**: オーディオは再生スレッドで実時間進行。FFT は独立スレッドで動かし、結果を GPU スレッドに渡す。3スレッド間の同期はチャネルベースで
- **FFT サイズ**: 2048 が標準だが、スピーカー応答 (440Hz の解像度 ~21Hz) を考えると 4096 がより精度高い。レイテンシとのトレードオフ
- **ジッター**: 60fps 描画と FFT の解析タイミングが揃わないとバーが脈動して見える。フレーム補間で滑らかに
