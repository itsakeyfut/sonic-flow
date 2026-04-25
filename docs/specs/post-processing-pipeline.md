# Post-processing Pipeline

## 概要

すべてのビジュアライザーの出力に適用されるマルチパスポストプロセスパイプラインを定義する。  
HDR レンダリング → Bloom → モーションブラー → 色収差/カラーグレーディング → CRT → トーンマップ → LDR 出力 の順で処理する。

---

## 1. パイプライン全体構成

```
┌─────────────────────────────────────────────────────────────────┐
│                    Frame Rendering Flow                          │
│                                                                 │
│  Visualizer Pass                                                │
│    └─ HDR Color Buffer (RGBA16Float, フル解像度)                │
│         │                                                       │
│         ├─ [Pass 1] Bloom                                       │
│         │    ├─ Threshold Extract (輝度 > threshold の部分)      │
│         │    ├─ Downsample × 5 (1/2 → 1/4 → 1/8 → 1/16 → 1/32) │
│         │    └─ Upsample × 5   (Dual Kawase Blur)              │
│         │                                                       │
│         ├─ [Pass 2] Motion Blur                                 │
│         │    └─ Temporal Accumulation Buffer                    │
│         │         (前フレームとブレンド)                           │
│         │                                                       │
│         ├─ [Pass 3] Composite                                   │
│         │    └─ HDR Scene + Bloom additive blend               │
│         │                                                       │
│         ├─ [Pass 4] Color Grade + Chromatic Aberration          │
│         │    ├─ RGB チャンネルオフセット (色収差)                  │
│         │    ├─ Contrast / Saturation                           │
│         │    └─ Vignette                                        │
│         │                                                       │
│         ├─ [Pass 5] Tonemap (HDR → LDR)                        │
│         │    └─ ACES Filmic / Reinhard                          │
│         │                                                       │
│         └─ [Pass 6] CRT Scanline / Glitch / Film Grain          │
│              └─ Final LDR Output → Screen                       │
└─────────────────────────────────────────────────────────────────┘
```

---

## 2. レンダーターゲット構成

| バッファ名               | フォーマット      | 解像度    | 用途                         |
|------------------------|----------------|---------|------------------------------|
| `rt_hdr`               | RGBA16Float    | フル     | ビジュアライザー出力 (HDR)      |
| `rt_bloom_threshold`   | RGBA16Float    | フル     | Bloom 閾値抽出               |
| `rt_bloom_0`           | RGBA16Float    | 1/2     | Bloom ダウンサンプル 1         |
| `rt_bloom_1`           | RGBA16Float    | 1/4     | Bloom ダウンサンプル 2         |
| `rt_bloom_2`           | RGBA16Float    | 1/8     | Bloom ダウンサンプル 3         |
| `rt_bloom_3`           | RGBA16Float    | 1/16    | Bloom ダウンサンプル 4         |
| `rt_bloom_4`           | RGBA16Float    | 1/32    | Bloom ダウンサンプル 5         |
| `rt_accumulation`      | RGBA16Float    | フル     | モーションブラー蓄積バッファ      |
| `rt_composite`         | RGBA16Float    | フル     | Bloom 合成後                 |
| `rt_color_grade`       | RGBA8Unorm     | フル     | カラーグレーディング後 (LDR)    |
| `rt_final`             | RGBA8Unorm     | フル     | 最終出力 (Swapchain へ)       |

**GPU メモリ概算** (1920×1080):
- RGBA16Float フル: 1920 × 1080 × 8 bytes = ~15.8 MB
- 全バッファ合計: ~120 MB (ダウンサンプルバッファ含む)

---

## 3. 各パスの詳細

### 3.1 Bloom

#### Phase A: 閾値抽出

```wgsl
// bloom_threshold.wgsl
fn luminance(col: vec3<f32>) -> f32 {
    return dot(col, vec3(0.2126, 0.7152, 0.0722));
}

@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let col = textureSample(hdr_tex, samp, in.uv);
    let lum = luminance(col.rgb);
    // 輝度 > threshold の部分だけ残す (ソフトニー)
    let bloom_val = col.rgb * max(lum - bloom_threshold, 0.0) / (lum + 1e-6);
    return vec4(bloom_val, 1.0);
}
```

#### Phase B: Dual Kawase Blur

Kawase Blur は Gaussian より少ないサンプル数で同等の品質を実現する。

```wgsl
// bloom_downsample.wgsl (各ダウンサンプルパスで使用)
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let t = 1.0 / vec2<f32>(textureDimensions(src_tex));
    let uv = in.uv;
    // 13-tap filter
    var col = textureSample(src_tex, samp, uv) * 4.0;
    col += textureSample(src_tex, samp, uv + vec2(-1.0, -1.0) * t);
    col += textureSample(src_tex, samp, uv + vec2( 1.0, -1.0) * t);
    col += textureSample(src_tex, samp, uv + vec2(-1.0,  1.0) * t);
    col += textureSample(src_tex, samp, uv + vec2( 1.0,  1.0) * t);
    // ... (計 13 タップ)
    return col / 8.0;
}
```

#### Phase C: 加算合成

```wgsl
// bloom_composite.wgsl
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let scene = textureSample(scene_tex, samp, in.uv);
    let bloom = textureSample(bloom_tex, samp, in.uv);
    return scene + bloom * bloom_intensity;
}
```

---

### 3.2 Motion Blur (Temporal Accumulation)

前フレームとの指数移動平均で残像を生成する。

```wgsl
// motion_blur.wgsl
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let current  = textureSample(current_tex, samp, in.uv);
    let previous = textureSample(accum_tex,   samp, in.uv);

    // Beat 時は蓄積をリセット気味にして「ビート=鋭い残像」表現
    let decay = 0.85 - uniforms.beat_kick * 0.35;

    return mix(current, previous, decay);
}
```

> **注意**: テンポラル蓄積はカメラ/ビジュアライザーが静止しているときに最も効果的。  
> 高速移動時は ghosting が起きるため、速度しきい値で自動 ON/OFF する。

---

### 3.3 Tonemap (HDR → LDR)

#### ACES Filmic (推奨)

```wgsl
fn tonemap_aces(x: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return clamp((x * (a * x + b)) / (x * (c * x + d) + e), vec3(0.0), vec3(1.0));
}
```

#### Reinhard (シンプルフォールバック)

```wgsl
fn tonemap_reinhard(x: vec3<f32>) -> vec3<f32> {
    return x / (x + vec3(1.0));
}
```

---

### 3.4 CRT Glitch の Beat 連動

```
beat_kick > 0.5 (Beat 検出時)
  → glitch_strength = beat_kick * 0.03 (水平ピクセルシフト量)
  → scanline_darkness 一時増加 (フラッシュ感)
  → grain_strength 一時増加
  → 100ms で通常値に指数減衰
```

---

## 4. エフェクトスタック設定

```toml
# config.toml の [visualizer.effects] セクション
[visualizer.effects]
bloom_enabled    = true
bloom_threshold  = 0.8     # HDR 輝度閾値
bloom_intensity  = 0.6     # 合成時の Bloom 強度

motion_blur_enabled = true
motion_blur_decay   = 0.85

chromatic_aberration_enabled = true
ca_strength      = 0.003
ca_beat_boost    = 0.008   # Beat 時追加量

vignette_enabled = true
vignette_inner   = 0.5
vignette_outer   = 1.2

crt_enabled      = false   # デフォルト OFF (ユーザー好みが分かれる)
crt_scanline     = 0.03
crt_grain        = 0.02
crt_glitch       = true

tonemap_mode     = "aces"  # "aces" | "reinhard" | "none"
```

---

## 5. プリセット定義

| プリセット名 | Bloom | Motion Blur | CA  | CRT | 特徴                    |
|------------|-------|-------------|-----|-----|------------------------|
| Default    | ON    | OFF         | ON  | OFF | バランス型               |
| Neon       | ON強  | ON          | ON強| OFF | 強いグロー・残像          |
| Cyberpunk  | ON    | ON          | ON  | ON  | グリッチ・ノイズあり       |
| Minimal    | OFF   | OFF         | OFF | OFF | クリーンな映像            |
| Cinematic  | ON    | ON          | ON  | OFF | ACES トーンマップ重視     |
| Retro      | OFF   | OFF         | OFF | ON  | CRT スキャンライン中心     |

---

## 6. パフォーマンス制約

- **解像度スケーリング**: Bloom のダウンサンプルは 1/2 解像度から開始するため GPU 負荷は低い
- **HDR 対応確認**: `wgpu::TextureFormat::Rgba16Float` を `wgpu::Features::TEXTURE_ADAPTER_SPECIFIC_FORMAT_FEATURES` でサポート確認
- **非対応デバイスフォールバック**: `Rgba8Unorm` で LDR モードにフォールバック
- **ビート判定のスキップ**: `beat_kick` が 0.0 の場合、Glitch パスをスキップして負荷削減

---

## 7. 実装ファイル

```
crates/sonic-shader/src/
├── pipeline/
│   ├── mod.rs              # PostProcessPipeline (全パスを管理)
│   ├── bloom.rs            # BloomPass
│   ├── motion_blur.rs      # MotionBlurPass
│   ├── color_grade.rs      # ColorGradePass
│   ├── tonemap.rs          # TonemapPass
│   └── crt_glitch.rs       # CrtGlitchPass
└── render_targets.rs       # RenderTargetPool (リサイズ対応)
```
