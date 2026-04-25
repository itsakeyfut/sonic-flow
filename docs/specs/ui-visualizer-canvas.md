# UI 詳細設計 — ビジュアライザーキャンバス

## 1. 概要

`docs/specs/ui-design.md` の全体設計に基づき、ビジュアライザーキャンバスエリア (Layer 0: wgpu) と  
その直上に配置される Top Bar (Layer 2: Slint glass) の詳細仕様を定義する。

---

## 2. ビジュアライザーキャンバス

### 2.1 レイアウト原則

```
ウィンドウ全体 (100% × 100%)
└── wgpu Surface (全画面, Layer 0)
    └── ビジュアライザー描画のみ
        Canvas 上に Slint 要素は一切配置しない
        (Top Bar / Bottom Bar はウィンドウ端に固定された別レイヤー)
```

**Canvas 上の常時オーバーレイ: なし**  
FPS カウンター・BPM 表示・VU メーター等、すべてキャンバス上への常時表示は行わない。  
映像体験を最大化するためにビジュアライザー映像のみを表示する。

### 2.2 空状態 (No Track)

プレイリストが空またはファイル未読み込み時、wgpu キャンバスに直接描画する。

```
┌─────────────────────────────────────────────────┐
│ [Top Bar]                                       │
│                                                 │
│                                                 │
│                    ♪                            │
│                                                 │
│             ファイルをここにドロップ               │
│                    または                        │
│             [  ファイルを開く  ]                 │
│                                                 │
│                                                 │
│ [Bottom Bar]                                    │
└─────────────────────────────────────────────────┘
```

**空状態シェーダー仕様:**

```wgsl
// idle_background.wgsl
// 再生停止時: ゆっくりとした色相回転グラデーション (ambientアニメーション)
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let uv = in.uv;
    let hue = uniforms.time * 0.03;  // 非常にゆっくりした色相変化
    let dist = length(uv - 0.5);
    let col = hsv_to_rgb(hue + dist * 0.3, 0.4, 0.15);  // 暗くて彩度低め
    return vec4(col, 1.0);
}
```

- ドロップゾーンはビジュアライザー上にガラスオーバーレイとして表示
- ドラッグオーバー時: ボーダーが accent-purple に変化 + "ドロップしてください" テキスト

---

## 3. ビジュアライザー切り替えトランジション

### 3.1 方式: クロスフェード (300ms)

```
旧 Visualizer: opacity 1.0 → 0.0
新 Visualizer: opacity 0.0 → 1.0
              同時進行 300ms ease-in-out

GPU uniform: switch_alpha (0.0 〜 1.0)
```

### 3.2 実装設計

**CPU 側 (VisualizerEngine):**

```rust
pub struct VisualizerEngine {
    current: Box<dyn Visualizer>,
    previous_frame: Option<wgpu::Texture>,  // 切り替え前フレームのスナップショット
    switch_alpha: f32,          // 0.0 = 完全に旧, 1.0 = 完全に新
    switching: bool,
}

impl VisualizerEngine {
    pub fn switch_to(&mut self, name: &str, registry: &VisualizerRegistry) {
        // 現在フレームを previous_frame テクスチャに保存
        self.capture_current_frame();
        self.current = registry.create(name).unwrap();
        self.switch_alpha = 0.0;
        self.switching = true;
    }

    pub fn tick(&mut self, dt: f32) {
        if self.switching {
            self.switch_alpha += dt / 0.3;  // 300ms
            if self.switch_alpha >= 1.0 {
                self.switch_alpha = 1.0;
                self.switching = false;
                self.previous_frame = None;  // 解放
            }
        }
    }
}
```

**GPU 側 (コンポジットシェーダー):**

```wgsl
// visualizer_composite.wgsl
// switch_alpha が 1.0 になるまでの間だけ使用

@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let new_col = textureSample(current_tex, samp, in.uv);

    if uniforms.switch_alpha >= 1.0 {
        return new_col;  // 切り替え完了時はそのまま
    }

    let old_col = textureSample(previous_tex, samp, in.uv);
    return mix(old_col, new_col, uniforms.switch_alpha);
}
```

**switch_alpha のイージング:**

```rust
// ease-in-out cubic
fn ease_in_out(t: f32) -> f32 {
    if t < 0.5 { 2.0 * t * t } else { -1.0 + (4.0 - 2.0 * t) * t }
}
let eased_alpha = ease_in_out(self.switch_alpha.clamp(0.0, 1.0));
// GPU に転送する switch_alpha はイージング適用後の値
```

### 3.3 タイムライン

```
t=0ms:    ユーザーがタブをクリック
t=0ms:    前フレームを previous_frame テクスチャにコピー
t=0ms:    新しい Visualizer インスタンスを生成 (VisualizerRegistry::create)
t=0ms:    switch_alpha = 0.0 (クロスフェード開始)
t=0-300ms: switch_alpha が 0.0 → 1.0 に増加 (ease-in-out, 毎フレーム更新)
t=300ms:  switch_alpha = 1.0, previous_frame テクスチャを解放
t>300ms:  通常レンダリングに戻る (コンポジットパスをスキップ)
```

### 3.4 パフォーマンス考慮

- `previous_frame` テクスチャ: RGBA16Float, フル解像度 (~15.8MB @ 1080p)
- クロスフェード中のみ確保。終了後即座に解放。
- 切り替え中は旧 Visualizer の `update_audio()` は呼ばない (新しい側のみ更新)

---

## 4. Top Bar コンポーネント

### 4.1 レイアウト仕様

```
高さ: 48px (固定)
幅:   ウィンドウ幅 100%
背景: rgba(10, 10, 15, 0.75)   ← ビジュアライザーが透けて見える
      + backdrop-blur 近似 (Slint 内で重ね重ね半透明 Rectangle で近似)
ボーダー下: 1px rgba(255, 255, 255, 0.10)

┌────────────────────────────────────────────────────────┐  48px
│  [2D Bars] [3D Bars] [Particles] [Fractal] [Wave] [Fall]  │    [Neon ▼]  [≡]
└────────────────────────────────────────────────────────┘
   ←── タブグループ (左パディング 12px, 間隔 4px) ────→      ← 右端 12px →
```

### 4.2 ビジュアライザータブ仕様

```
サイズ:   paddding 8px 14px, height 32px
角丸:     6px
フォント: 12px, weight 500

状態テーブル:
┌─────────┬──────────────────────────────┬───────────────────────────────┐
│ 状態     │ 背景                          │ ボーダー/装飾                  │
├─────────┼──────────────────────────────┼───────────────────────────────┤
│ 通常     │ なし                          │ なし                          │
│ ホバー   │ rgba(255,255,255, 0.06)       │ 1px bottom: rgba(255,255,255,0.2) │
│ アクティブ│ rgba(124,58,237, 0.15)       │ 2px bottom: #7c3aed           │
│         │                              │ + glow: 0 2px 12px rgba(124,58,237,0.4) │
├─────────┼──────────────────────────────┼───────────────────────────────┤
│ + Beat  │ (アクティブ状態に上乗せ)        │ glow 強度: beat_kick × 0.6   │
└─────────┴──────────────────────────────┴───────────────────────────────┘

アニメーション: background, border 150ms ease
Beat glow:    τ≈80ms 指数減衰 (Slint プロパティ beat-kick を毎フレーム更新)
```

**Slint 実装イメージ:**

```slint
component VisualizerTab {
    in property <bool> is-active;
    in property <float> beat-kick;   // 0.0-1.0 指数減衰値
    in property <string> label;
    callback clicked;

    // アクティブ時の下ボーダー glow
    // Slint の drop-shadow は外側シャドウに対応していないため
    // 2px 高の Rectangle を下端に配置して glow を模倣
    Rectangle {
        y: parent.height - 2px;
        height: 2px;
        background: is-active ? #7c3aed : transparent;
        opacity: is-active ? (0.4 + beat-kick * 0.6) : 0;
    }
}
```

### 4.3 プリセットドロップダウン仕様

```
幅:     128px
高さ:    32px
位置:    タブグループ右端から 8px のギャップ

外観:
  背景: rgba(255,255,255,0.08)
  ボーダー: 1px rgba(255,255,255,0.15)
  角丸: 6px
  テキスト: 12px, weight 500, color-text-primary
  矢印: ▼ (展開状態では ▲)

展開リスト:
  位置:  Top Bar 直下 (top: 48px, right: 52px)
  幅:    160px
  背景:  rgba(10, 10, 15, 0.92)
  ボーダー: 1px rgba(255,255,255,0.15)
  角丸:  8px
  shadow: 0 8px 32px rgba(0,0,0,0.6)
  z-index: 最前面

リスト項目:
  高さ: 36px
  テキスト: 12px
  ✓ アイコン: アクティブなプリセットに表示 (color-accent-purple)
  区切り線: カスタムプリセットとの間に 1px rgba(255,255,255,0.10)
  "保存..." 項目: + アイコン付き

閉じる条件:
  - 項目クリック
  - ドロップダウン外クリック
  - Esc キー
```

### 4.4 プレイリストボタン (≡ / ✕) 仕様

```
サイズ:  40×32px
位置:   右端 8px
アイコン: ≡ (ハンバーガー) / ✕ (プレイリスト開時)
アニメーション: アイコン切り替え 200ms cross-fade

ホバー: rgba(255,255,255,0.08) 背景

アクティブ (ドロワー開時):
  背景: rgba(124,58,237,0.15)
  ボーダー: 1px rgba(124,58,237,0.4)
```

### 4.5 タブ数によるレスポンシブ対応

```
ウィンドウ幅:  タブバーの挙動
≥ 1280px      全タブを横一列に表示
800-1279px    タブを横スクロール (スクロールバー非表示, ホイール/スワイプ操作)
< 800px       タブをドロップダウンに折り畳み
              [▼ Visualizer: 3D Bars] の形式でコンパクト表示
```

---

## 5. Beat 連動 UI アニメーション詳細

### 5.1 対象要素と反応仕様

| 要素                  | 反応内容                                              | 計算式                                      |
|---------------------|-----------------------------------------------------|--------------------------------------------|
| アクティブタブ 下ボーダー | glow 強度が増加                                       | `opacity = 0.4 + beat_kick × 0.6`         |
| シークバー充填グラデーション | purple → cyan 方向にシフト + glow 追加              | `color = lerp(#7c3aed, #06b6d4, beat_kick)` |
| シークバー glow       | beat_kick に応じた cyan glow                         | `glow_alpha = beat_kick × 0.4`            |

### 5.2 beat_kick の Rust → Slint 配信

```rust
// Controller の audio イベントループ内
// AudioAnalysisResult を受信するたびに UiState を更新
slint::invoke_from_event_loop(move || {
    UiState::get(&ui).set_beat_kick(analysis.beat.kick_envelope);  // 0.0-1.0
    UiState::get(&ui).set_beat_phase(analysis.beat_phase);
}).unwrap();
```

**更新頻度**: 解析スレッドの 60Hz に同期 (≈ 16ms ごと)

### 5.3 Slint 側の補間

```slint
// 60fps で Slint は自動的にプロパティ変化を補間するわけではないため
// beat_kick の指数減衰は Rust 側 (VisualizerUniforms と同じロジック) で計算し
// 毎フレーム Slint プロパティに書き込む

// Rust 側:
let decay = (-dt / 0.08).exp();  // τ = 80ms
beat_kick_ui *= decay;
if beat_event.kick { beat_kick_ui = 1.0; }
UiState::get(&ui).set_beat_kick(beat_kick_ui);
```

---

## 6. wgpu ← → Slint 統合境界

### 6.1 Slint ウィンドウ設定

```rust
// wgpu が Slint ウィンドウ全体をレンダリングするための設定
let window = slint::Window::new();
window.set_rendering_notifier(move |state, graphics_api| {
    match state {
        RenderingState::RenderingSetup => { /* wgpu デバイス初期化 */ }
        RenderingState::BeforeRendering => {
            // wgpu コマンドを記録し submit
            render_engine.render(&uniforms);
        }
        RenderingState::AfterRendering => {}
        RenderingState::RenderingTeardown => { /* リソース解放 */ }
    }
});
```

### 6.2 Slint の描画領域

Slint の UI 要素は wgpu 描画の**上**に自動的に合成される。  
`AppWindow` のルート `Rectangle` の `background` を `transparent` にすることで wgpu 描画が透けて見える。

```slint
// app/ui/app.slint
component AppWindow inherits Window {
    background: transparent;  // wgpu が描画した背景がそのまま見える

    // Top Bar (ウィンドウ最上部)
    TopBar { y: 0; }

    // Bottom Bar (ウィンドウ最下部)
    BottomBar { y: parent.height - self.height; }

    // Drawers (右端からスライド)
    PlaylistDrawer { ... }
    SettingsDrawer { ... }
}
```

---

## 7. パフォーマンス要件

| 項目                          | 目標値              |
|-----------------------------|-------------------|
| ビジュアライザー切り替えレイテンシ | ≤ 1 フレーム (16ms) |
| クロスフェード GPU 追加負荷      | ≤ 0.5ms/フレーム   |
| previous_frame テクスチャサイズ | ~15.8MB (1080p RGBA16Float) |
| Beat glow Slint 更新          | 60Hz (解析スレッドと同期) |
| Top Bar リサイズ対応            | ≤ 1 フレームで再計算 |
