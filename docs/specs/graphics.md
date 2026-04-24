# Sonic Flow - グラフィックス仕様

## 概要

全てのビジュアライザーはGPUシェーダーで描画する。画像素材は使用しない。
音声のFFTデータをGPUに転送し、WGSL シェーダーで映像をプロシージャル生成する。

## 技術スタック

- **シェーダー言語**: WGSL (WebGPU Shading Language)
- **GPU API**: wgpu (クロスプラットフォーム WebGPU 実装)
- **バックエンド**: Vulkan (Windows/Linux), Metal (macOS), DX12 (Windows フォールバック)

> 当初 Slang を検討したが、clang-sys との互換性問題により WGSL に変更済み。

## レンダリングパイプライン

```
FFT周波数データ (CPU)
  |
  v
Uniform Buffer (CPU -> GPU転送)
  |
  v
Vertex Shader (ジオメトリ)
  |
  v
Fragment Shader (色・エフェクト)
  |
  v
wgpu TextureView -> Slint 表示
```

## Uniform Buffer 設計

シェーダーへ渡すデータ構造:

```rust
#[repr(C)]
#[derive(Debug, Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub struct VisualizerUniforms {
    // 時間
    pub time: f32,
    pub delta_time: f32,

    // 画面サイズ
    pub resolution: [f32; 2],

    // FFTデータ
    pub spectrum: [f32; 128],     // 周波数帯域ごとの強度
    pub bass: f32,                // 低音 (20-250Hz)
    pub mid: f32,                 // 中音 (250-4kHz)
    pub treble: f32,              // 高音 (4kHz-20kHz)
    pub volume: f32,              // 全体音量

    // エフェクトパラメータ
    pub sensitivity: f32,
    pub smoothing: f32,
    pub color_scheme: u32,
    pub _padding: u32,
}
```

## シェーダー設計方針

### ファイル構成

```
libs/sonic-shader/src/shaders/
  ├── common.wgsl            # 共通ユーティリティ (ノイズ関数, 色変換等)
  ├── spectrum_bars.wgsl     # スペクトラムバー
  ├── waveform.wgsl          # 波形オシロスコープ
  ├── circular.wgsl          # 円形ビジュアライザー
  ├── particles.wgsl         # パーティクルシステム
  └── effects/
      ├── bloom.wgsl         # ブルーム
      ├── glow.wgsl          # グロー
      ├── glitch.wgsl        # グリッチ
      └── chromatic.wgsl     # クロマティックアバレーション
```

### 共通関数 (common.wgsl)

シェーダー間で再利用する関数:

- **ノイズ**: Simplex noise, Perlin noise, FBM
- **SDF**: 円, 線, 矩形 の Signed Distance Function
- **色変換**: HSV <-> RGB, パレット生成
- **数学**: smoothstep, remap, polar座標変換

### ビジュアライザーごとのシェーダー

各ビジュアライザーは最低限以下を持つ:

1. **Vertex Shader**: フルスクリーン quad (共通)
2. **Fragment Shader**: ビジュアライザー固有の描画ロジック

Fragment Shader は `spectrum`, `bass`, `mid`, `treble`, `time` を使って描画。

## FFT → シェーダー データフロー

```
低音 (20-250Hz)   -> uniforms.bass   -> 大きな動き、背景の脈動
中音 (250-4kHz)   -> uniforms.mid    -> 波形変形、メインビジュアル
高音 (4kHz-20kHz) -> uniforms.treble -> パーティクル、きらめき
全帯域            -> uniforms.spectrum[128] -> 詳細スペクトラム
```

## パフォーマンス要件

| 項目 | 目標値 |
|------|--------|
| フレーム描画時間 | 8.3ms以下 (120fps) |
| シェーダーコンパイル | 100ms以下 (初回ロード時) |
| GPU メモリ使用量 | 50MB以下 / ビジュアライザー |
| GPU 使用率 | 30%以下 (通常動作時) |
| フレームタイム分散 | 2ms以下 (安定した60fps最低保証) |

## Slint との統合

wgpu の描画結果を Slint の UI 内に埋め込む。

方式:
1. wgpu で TextureView にレンダリング
2. Slint のカスタムレンダリング領域に合成

```
+---------------------------+
|  Slint Window             |
|  [再生コントロール]        |
|  +---------------------+  |
|  |  wgpu RenderTarget  |  |
|  |  (ビジュアライザー)  |  |
|  +---------------------+  |
|  [スライダー / 設定]     |
+---------------------------+
```

## エラーハンドリング

```rust
#[derive(Debug, thiserror::Error)]
pub enum GraphicsError {
    #[error("Shader compilation failed: {0}")]
    ShaderCompilation(String),

    #[error("GPU resource allocation failed: {0}")]
    ResourceAllocation(String),

    #[error("Rendering pipeline error: {0}")]
    RenderingError(String),

    #[error("GPU device lost")]
    DeviceLost,

    #[error("GPU out of memory")]
    OutOfMemory,
}
```
