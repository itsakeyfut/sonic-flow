# Shader Catalog

すべての WGSL シェーダーの設計・入力・アルゴリズム・パラメーターを定義する。

---

## ファイル構成

```
crates/sonic-shader/src/shaders/
├── common.wgsl              # 共通ユーティリティ (全シェーダーが include)
├── noise.wgsl               # ノイズ関数群
├── sdf.wgsl                 # Signed Distance Function
│
├── visualizers/
│   ├── spectrum_bars.wgsl   # 2D Spectrum Bars (v0.3.0 初期実装)
│   ├── spectrum_bars_3d.wgsl # 3D Spectrum Bars (v0.4.0)
│   ├── waveform.wgsl        # 波形オシロスコープ
│   ├── circular.wgsl        # サークルスペクトラム
│   ├── particles.wgsl       # パーティクル描画 (fragment)
│   ├── particles_compute.wgsl # パーティクル物理 (compute)
│   ├── procedural.wgsl      # プロシージャル/フラクタル
│   └── waterfall.wgsl       # 3D Waterfall Spectrogram
│
└── post_process/
    ├── bloom_downsample.wgsl # Bloom ダウンサンプル
    ├── bloom_upsample.wgsl   # Bloom アップサンプル
    ├── motion_blur.wgsl      # モーションブラー (テンポラル蓄積)
    ├── color_grade.wgsl      # 色収差・カラーグレーディング・ビネット
    └── crt_glitch.wgsl       # CRT スキャンライン・グリッチ
```

---

## 1. 共通ライブラリ

### 1.1 common.wgsl

すべてのシェーダーが使用する基本ユーティリティ。

```wgsl
// HSV → RGB 変換
fn hsv2rgb(h: f32, s: f32, v: f32) -> vec3<f32>;

// RGB → HSV
fn rgb2hsv(rgb: vec3<f32>) -> vec3<f32>;

// カラーパレット (Inigo Quilez 式)
fn palette(t: f32, a: vec3<f32>, b: vec3<f32>, c: vec3<f32>, d: vec3<f32>) -> vec3<f32>;

// 組み込みパレット
fn palette_neon(t: f32) -> vec3<f32>;      // シアン→マゼンタ→ブルー
fn palette_fire(t: f32) -> vec3<f32>;      // 黒→赤→オレンジ→白
fn palette_spectrum(t: f32) -> vec3<f32>;  // 虹色

// 数学ユーティリティ
fn remap(v: f32, in_lo: f32, in_hi: f32, out_lo: f32, out_hi: f32) -> f32;
fn polar(uv: vec2<f32>) -> vec2<f32>;      // デカルト → 極座標

// ソフトクランプ (sin ベース)
fn scurve(t: f32) -> f32;
```

### 1.2 noise.wgsl

```wgsl
fn hash(p: vec2<f32>) -> f32;                    // 疑似乱数
fn noise2(p: vec2<f32>) -> f32;                  // Value noise
fn simplex2(p: vec2<f32>) -> f32;                // Simplex noise
fn perlin2(p: vec2<f32>) -> f32;                 // Perlin noise
fn fbm(p: vec2<f32>, octaves: i32) -> f32;       // Fractal Brownian Motion
fn domain_warp(p: vec2<f32>, q: vec2<f32>) -> f32; // Domain warping
```

### 1.3 sdf.wgsl

```wgsl
fn sdf_circle(p: vec2<f32>, r: f32) -> f32;
fn sdf_rect(p: vec2<f32>, b: vec2<f32>) -> f32;
fn sdf_line(p: vec2<f32>, a: vec2<f32>, b: vec2<f32>) -> f32;
fn sdf_union(a: f32, b: f32) -> f32;
fn sdf_smooth_union(a: f32, b: f32, k: f32) -> f32;
```

---

## 2. ビジュアライザーシェーダー

### 2.1 spectrum_bars.wgsl (2D)

**ロードマップ**: v0.3.0  
**入力**: `VisualizerUniforms`  
**アルゴリズム**:
1. フラグメント UV から帯域インデックスを算出
2. `uniforms.spectrum[band_idx]` から高さを取得
3. `smoothstep` でバー境界をソフト化
4. `palette_neon(band_idx / 128.0)` でグラデーション色付け
5. バー高さ超過部分を発光 (HDR 値 > 1.0) して Bloom の入力とする

**パラメーター**:

| 名前           | 型   | 説明                    |
|---------------|------|------------------------|
| sensitivity   | f32  | バー高さの倍率           |
| smoothing     | f32  | フレーム間スムージング係数 |
| band_count    | u32  | 表示帯域数 (32/64/128)   |
| color_scheme  | u32  | パレット選択             |

---

### 2.2 spectrum_bars_3d.wgsl

**ロードマップ**: v0.4.0  
**入力**: `VisualizerUniforms` + Camera uniform  
**描画方式**: wgpu Instanced Draw (バー 1 本 = 1 インスタンス)  
**アルゴリズム**:

```
Vertex Shader:
  1. unit box ジオメトリをインスタンス化
  2. band_idx → X 座標にマッピング
  3. spectrum[i] → Y スケール
  4. Camera VP 行列でクリップ空間へ変換
  5. beat_kick でスケールパルス加算

Fragment Shader:
  1. バーの Y 位置に応じてグラデーション (底=暗、頂=明)
  2. 頂上付近を HDR 値 > 1.5 で Bloom 用ハイライト
  3. 反射床: Y < 0 のフラグメントで反転テクスチャサンプリング

Camera:
  - BPM 位相に応じてゆっくり周回 (Y 軸回転)
  - beat_kick で Z 方向に軽く揺れる
```

---

### 2.3 particles_compute.wgsl (Compute Shader)

**ロードマップ**: v0.4.0  
**ディスパッチ**: `(N_PARTICLES / 256, 1, 1)` ワークグループ  
**パーティクル構造体**:

```wgsl
struct Particle {
    pos:      vec3<f32>,
    vel:      vec3<f32>,
    life:     f32,        // 0.0 = dead, 1.0 = just spawned
    max_life: f32,
    color:    vec4<f32>,
    size:     f32,
    _pad:     vec3<f32>,
}
```

**物理アルゴリズム**:

```wgsl
@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    var p = particles[id.x];

    // ライフ減少
    p.life -= uniforms.delta_time / p.max_life;

    if p.life <= 0.0 {
        // スポーン: Beat イベントで爆発的スポーン
        if uniforms.beat_kick > 0.5 {
            p = spawn_particle(id.x, uniforms);
        }
    } else {
        // 物理: 重力 + ドラッグ + 音声引力
        let audio_force = vec3(
            uniforms.bass * sin(p.pos.x * 2.0),
            uniforms.mid  * 0.5,
            uniforms.treble * cos(p.pos.z * 3.0),
        );
        p.vel += (gravity + audio_force) * uniforms.delta_time;
        p.vel *= 0.98;  // ドラッグ
        p.pos += p.vel * uniforms.delta_time;
    }

    particles[id.x] = p;
}
```

**スポーン戦略**:
- アイドル時: 上部から静かに降下 (少量スポーン)
- `beat_kick` 検出時: 中央から爆発的スポーン (大量スポーン + 高速度)
- `beat_hihat` 検出時: 上部にきらめき (tiny, short-life)

---

### 2.4 procedural.wgsl

**ロードマップ**: v0.4.0  
**描画方式**: フルスクリーンクワッド  
**アルゴリズム**:

```wgsl
@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv * 2.0 - 1.0;
    let t  = uniforms.time;

    // Domain warping: 音量・Bass で歪み量を制御
    let warp_strength = uniforms.bass * 2.0 + 0.3;
    let q = vec2(
        fbm(uv + vec2(t * 0.1, 0.0), 6),
        fbm(uv + vec2(0.0, t * 0.1), 6),
    );
    let warped = fbm(uv + warp_strength * q, 5);

    // BPM 位相で色相回転
    let hue = warped + uniforms.beat_phase * 0.5 + t * 0.05;

    // Beat でコントラスト/輝度爆発
    let brightness = 1.0 + uniforms.beat_kick * 0.8;

    let col = hsv2rgb(hue, 0.8, warped * brightness);
    return vec4(col, 1.0);
}
```

**パラメーター**:

| 名前            | 説明                   |
|----------------|----------------------|
| warp_scale     | ドメインワーピングの強さ  |
| color_speed    | 色相回転速度            |
| octave_count   | FBM オクターブ数        |
| fractal_mode   | 0=FBM, 1=Julia, 2=Mandelbrot |

---

### 2.5 waterfall.wgsl (3D Spectrogram)

**ロードマップ**: v0.4.0  
**描画方式**: 3D グリッドメッシュ + 変位  
**テクスチャ**: スペクトラム履歴テクスチャ (幅=128帯域、高さ=256フレーム)

毎フレームテクスチャを 1 行シフトして最新スペクトラムを書き込む。

**アルゴリズム**:

```
Vertex Shader:
  1. グリッド UV を読む
  2. スペクトラム履歴テクスチャから高さを取得
  3. Y 方向に変位 (高い → 手前に盛り上がる)
  4. 熱マップカラー (低=青→高=赤) を計算

Fragment Shader:
  1. 頂点補間色を出力
  2. エッジ部分に glow 加算
```

---

## 3. ポストプロセスシェーダー

### 3.1 Bloom パイプライン

**ロードマップ**: v0.5.0  
**方式**: Dual Kawase Blur (Gaussian より高速)

```
HDR Color Buffer
  └─ Threshold Pass (輝度 > 1.0 の部分を抽出)
       └─ Downsample x5 (1/2, 1/4, 1/8, 1/16, 1/32 解像度)
            └─ Upsample x5 (Kawase blur kernel で加重平均)
                 └─ Additive Composite (元画像 + Bloom)
```

**bloom_downsample.wgsl**:
```wgsl
// 4 点サンプリング + 平均でダウンサンプル
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let texel = 1.0 / vec2<f32>(textureDimensions(src_tex));
    let c0 = textureSample(src_tex, samp, in.uv + vec2(-1.0, -1.0) * texel);
    let c1 = textureSample(src_tex, samp, in.uv + vec2( 1.0, -1.0) * texel);
    let c2 = textureSample(src_tex, samp, in.uv + vec2(-1.0,  1.0) * texel);
    let c3 = textureSample(src_tex, samp, in.uv + vec2( 1.0,  1.0) * texel);
    return (c0 + c1 + c2 + c3) * 0.25;
}
```

---

### 3.2 motion_blur.wgsl

**方式**: テンポラル蓄積 (Temporal Accumulation)

```wgsl
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let current  = textureSample(current_tex, samp, in.uv);
    let previous = textureSample(accumulation_tex, samp, in.uv);

    // Beat 時は蓄積を速めにリセット (鋭いトレイル)
    let blend = mix(0.85, 0.5, uniforms.beat_kick);

    return mix(current, previous, blend);
}
```

---

### 3.3 color_grade.wgsl

**処理チェーン**: 色収差 → カラーグレーディング → ビネット

```wgsl
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let center = uv - 0.5;

    // 色収差: R/G/B チャンネルを放射方向にオフセット
    let ca_strength = uniforms.beat_kick * 0.005 + 0.002;
    let r = textureSample(scene_tex, samp, uv - center * ca_strength).r;
    let g = textureSample(scene_tex, samp, uv).g;
    let b = textureSample(scene_tex, samp, uv + center * ca_strength).b;

    var col = vec3(r, g, b);

    // カラーグレーディング: シンプルな contrast/saturation
    col = pow(col, vec3(1.0 / 2.2));  // gamma
    col = mix(vec3(dot(col, vec3(0.299, 0.587, 0.114))), col, saturation);
    col = (col - 0.5) * contrast + 0.5;

    // ビネット
    let vignette = 1.0 - smoothstep(0.5, 1.4, length(center));
    col *= vignette;

    return vec4(col, 1.0);
}
```

**パラメーター**:

| 名前             | 型   | 範囲    | 説明            |
|----------------|------|--------|----------------|
| ca_base_strength | f32 | 0-0.02 | 色収差ベース強度  |
| ca_beat_boost  | f32  | 0-0.02 | Beat 時追加強度  |
| contrast       | f32  | 0.5-2.0 | コントラスト     |
| saturation     | f32  | 0-2.0  | 彩度             |
| vignette_inner | f32  | 0-1.0  | ビネット内半径   |
| vignette_outer | f32  | 0-2.0  | ビネット外半径   |

---

### 3.4 crt_glitch.wgsl

**処理**: スキャンライン → グリッチ → フィルムグレイン

```wgsl
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    var uv = in.uv;

    // グリッチ: Beat 時に水平方向ピクセルシフト
    let glitch_strength = uniforms.beat_kick * 0.02;
    let glitch_line = floor(uv.y * 20.0) / 20.0;
    uv.x += (hash(vec2(glitch_line, uniforms.time)) - 0.5) * glitch_strength;

    var col = textureSample(scene_tex, samp, uv).rgb;

    // CRT スキャンライン
    let scanline = sin(uv.y * resolution.y * 3.14159) * 0.04;
    col -= scanline;

    // フィルムグレイン
    let grain = (hash(uv + uniforms.time * 0.01) - 0.5) * grain_strength;
    col += grain;

    return vec4(col, 1.0);
}
```

**パラメーター**:

| 名前            | 説明              |
|----------------|-----------------|
| scanline_strength | スキャンライン強度  |
| grain_strength  | フィルムグレイン強度 |
| glitch_enabled  | グリッチ ON/OFF   |
| glitch_frequency | グリッチ頻度 (Hz) |

---

## 4. シェーダーコンパイル戦略

WGSL は `#include` を持たないため、`build.rs` で事前結合する。

```rust
// crates/sonic-shader/build.rs
fn bundle_shader(name: &str, deps: &[&str]) {
    let mut source = String::new();
    for dep in deps {
        source += &read_shader(dep);
        source += "\n";
    }
    source += &read_shader(name);
    write_bundled_shader(name, &source);
}

// 例: spectrum_bars.wgsl は common + noise に依存
bundle_shader("spectrum_bars", &["common", "noise"]);
```

---

## 5. パフォーマンス目標

| シェーダー                     | 目標フレームタイム |
|------------------------------|-----------------|
| Spectrum Bars 2D              | < 1ms           |
| Spectrum Bars 3D (128 bars)   | < 2ms           |
| GPU Particles (100k)          | < 3ms           |
| Procedural/Fractal            | < 4ms           |
| Waterfall 3D                  | < 3ms           |
| Bloom (5-pass)                | < 2ms           |
| Motion Blur                   | < 0.5ms         |
| Color Grade + Vignette        | < 0.5ms         |
| CRT Glitch                    | < 0.5ms         |
| **全エフェクト合計**            | **< 8.3ms (120fps)** |
