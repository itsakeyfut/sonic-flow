# UI 詳細設計 — Settings ドロワー

## 1. 概要

⚙ ボタンをクリックすると右からスライドインするガラスドロワー。  
EQ・ビジュアライザーパラメーター・ポストプロセスエフェクトの設定を提供する。

---

## 2. ドロワー本体仕様

```
幅:     360px (固定)
高さ:    ウィンドウ高さ 100%
位置:    右端 (x = ウィンドウ幅 - 360px)
背景:    rgba(10, 10, 15, 0.90)
        + backdrop-blur 近似
ボーダー左: 1px rgba(255, 255, 255, 0.12)

アニメーション:
  開く:  x: +360px → 0, duration 250ms, ease-out
  閉じる: x: 0 → +360px, duration 200ms, ease-in

Playlist ドロワーとの重複:
  後から開いた方が最前面 (z-index 上位)
```

---

## 3. 全体レイアウト

```
┌────────────────────────────────────────────┐
│ ⚙ Settings                              ✕  │  ← ヘッダー 48px
├──────────┬─────────────┬──────────────────┤
│    EQ    │  Visualizer │    Effects       │  ← タブバー 40px
├──────────┴─────────────┴──────────────────┤
│                                            │
│  (タブコンテンツ)                            │
│                                            │
│          (スクロール可能)                    │
│                                            │
└────────────────────────────────────────────┘
```

---

## 4. ヘッダー (48px)

```
背景: rgba(255,255,255,0.04) + ボーダー下 1px rgba(255,255,255,0.10)

左端 16px:  "⚙ Settings" (14px, weight 600, color-text-primary)
右端 12px:  ✕ ボタン (32×32px, 閉じるアクション)

✕ ボタン:
  ホバー: rgba(255,255,255,0.08) 背景, 角丸 6px
  クリック: ドロワーを閉じる
```

---

## 5. タブバー (40px)

```
背景: rgba(255,255,255,0.02) + ボーダー下 1px rgba(255,255,255,0.10)

タブ: [EQ] [Visualizer] [Effects]

各タブ状態:
  通常:    12px, color-text-secondary, パディング 8px 16px
  ホバー:  color-text-primary
  アクティブ: color-text-primary, weight 600
             下端 2px accent-purple ライン
             背景 rgba(124,58,237,0.10)

アニメーション: 150ms ease
```

---

## 6. EQ タブ

### 6.1 レイアウト

```
┌─────────────────────────────────────────────────────────┐
│  [プリセット ▼]                        [リセット]          │  ← プリセット行 44px
├─────────────────────────────────────────────────────────┤
│                                                         │
│  +12 ┤                                                  │
│      │  ┃     ┃  ┃     ┃     ┃  ┃  ┃     ┃     ┃       │
│   0  ┼──╂─────╂──╂─────╂─────╂──╂──╂─────╂─────╂──     │
│      │  ┃     ┃  ┃     ┃     ┃  ┃  ┃     ┃     ┃       │
│  -12 ┤                                                  │
│      32 64  125 250 500  1k   2k  4k  8k  16k           │
│      ──────────────────────────────────────── Hz        │
└─────────────────────────────────────────────────────────┘
```

### 6.2 周波数バンド (10本)

| バンド | 周波数 | 音域     |
|------|------|--------|
| 1    | 32 Hz   | サブベース |
| 2    | 64 Hz   | ベース    |
| 3    | 125 Hz  | ローミッド  |
| 4    | 250 Hz  | ミッド     |
| 5    | 500 Hz  | アッパーミッド|
| 6    | 1 kHz   | プレゼンス  |
| 7    | 2 kHz   | プレゼンス  |
| 8    | 4 kHz   | エア      |
| 9    | 8 kHz   | エア      |
| 10   | 16 kHz  | ブリリアンス |

### 6.3 縦型スライダー仕様

```
高さ:   120px
幅:     28px (ヒット領域)
トラック: 4px幅, 角丸 2px
  ゼロより上: rgba(124,58,237,0.6)  (Boost: 紫)
  ゼロより下: rgba(6,182,212,0.6)   (Cut: シアン)
  未変化部分: rgba(255,255,255,0.15)

サム (●):
  直径: 12px, 背景: white
  ホバー: scale(1.2)
  ドラッグ中: drop-shadow 0 0 6px rgba(124,58,237,0.8)

範囲: -12dB ～ +12dB (センターが 0dB)
ステップ: 0.5dB

ゼロライン: 1px rgba(255,255,255,0.25) 横線

周波数ラベル:
  10px, color-text-muted, スライダー下端
  1000Hz以上は "1k" 形式

値ツールチップ:
  ドラッグ / ホバー中: "+3.0 dB" をスライダー上端に表示
  背景: rgba(10,10,15,0.9), 角丸 4px, パディング 3px 6px, 10px Mono
```

### 6.4 プリセットドロップダウン

```
幅:   160px, 高さ: 32px
位置: プリセット行 左端 16px

プリセット一覧:
  - Flat (デフォルト — 全バンド 0dB)
  - Bass Boost
  - Treble Boost
  - Vocal
  - Electronic

展開リスト:
  背景: rgba(10,10,15,0.95)
  ボーダー: 1px rgba(255,255,255,0.15)
  角丸: 8px
  幅: 180px
  アイテム高さ: 36px
  ✓ アクティブプリセットにアイコン表示
  "カスタム" — ユーザーが手動変更した状態を示す (読み取り専用ラベル)
```

### 6.5 リセットボタン

```
位置: プリセット行 右端 16px
テキスト: "リセット", 12px, weight 500
スタイル: 角丸 6px, ボーダー 1px rgba(255,255,255,0.15)
ホバー: rgba(255,255,255,0.08) 背景
クリック: 全バンドを 0dB に戻す + プリセット表示を "Flat" に変更
```

---

## 7. Visualizer タブ

### 7.1 レイアウト

```
┌─────────────────────────────────────────────┐
│  Sensitivity                                │  ← セクションラベル 12px muted
│  ────────────────────────●──────────        │  ← 水平スライダー
│                         0.75               │  ← 現在値 右揃え
│                                             │
│  Smoothing                                  │
│  ──────────────●────────────────────        │
│                0.40                         │
│                                             │
│  Bar Count                                  │
│  ─────────────────────●─────────────        │
│                       64                   │
│                                             │
│  Color Scheme                               │
│  [Purple-Cyan ▼]                            │
│                                             │
└─────────────────────────────────────────────┘
```

### 7.2 各パラメーター

#### Sensitivity (感度)

```
範囲: 0.1 ～ 2.0
デフォルト: 0.75
ステップ: 0.01
説明: FFT 振幅のスケールファクター
リアルタイム反映: ≤ 16ms でビジュアライザーに適用
```

#### Smoothing (スムージング)

```
範囲: 0.0 ～ 0.99
デフォルト: 0.40
ステップ: 0.01
説明: フレーム間の指数移動平均係数 (高いほど残像が長い)
リアルタイム反映: ≤ 16ms
```

#### Bar Count (バー数)

```
範囲: 16 ～ 128
デフォルト: 64
ステップ: 1 (ただし 2 の冪乗: 16/32/64/128 にスナップ)
適用対象: SpectrumBars 系ビジュアライザーのみ
          他のビジュアライザー選択時はグレーアウト
リアルタイム反映: ≤ 16ms
```

#### Color Scheme (カラースキーム)

```
ドロップダウン (幅: 全体幅 - 32px)

プリセット:
  - Purple-Cyan (デフォルト: #7c3aed → #06b6d4)
  - Fire (赤 → オレンジ → 黄)
  - Ocean (濃紺 → 水色 → 白)
  - Monochrome (グレースケール)
  - Rainbow (HSV 全域)

リアルタイム反映: ≤ 16ms でシェーダー uniform 更新
```

### 7.3 水平スライダー仕様

```
高さ (ヒット領域): 24px
トラック高さ: 4px (ホバー時 6px, 150ms ease)
充填: linear-gradient(→, #7c3aed, #06b6d4)
サム (●): ホバー / ドラッグ時のみ表示 (直径 12px, 白)

値表示: スライダー右端下 (12px Mono, color-text-secondary)
ラベル: スライダー上 8px (12px, color-text-muted)
```

---

## 8. Effects タブ

### 8.1 アコーディオン構造

```
各エフェクトは折りたたみ可能な行として表示。

┌────────────────────────────────────────────────────────┐
│ ● Bloom                           [ON/OFF]  ▼         │  ← 折りたたみヘッダー 44px
├────────────────────────────────────────────────────────┤
│   Threshold  ──────────●──────────         0.60       │  ← 詳細パネル (展開時)
│   Intensity  ───────────────●─────         0.80       │
│                                                        │
│ ○ Motion Blur                     [ON/OFF]  ▶         │  ← 折りたたみ (閉じた状態)
│                                                        │
│ ● Chromatic Aberration            [ON/OFF]  ▼         │
├────────────────────────────────────────────────────────┤
│   Strength   ──●──────────────────         0.15       │
│   Beat Boost ────────────●────────         0.50       │
│                                                        │
│ ○ Vignette                        [ON/OFF]  ▶         │
│ ○ CRT Glitch                      [ON/OFF]  ▶         │
│ ● Tonemap                         [ON/OFF]  ▼         │
├────────────────────────────────────────────────────────┤
│   ◉ ACES   ○ Reinhard                                 │
└────────────────────────────────────────────────────────┘
```

### 8.2 アコーディオンヘッダー (44px)

```
左 16px:  エフェクト名 (14px, weight 500)
          ON 時: color-text-primary
          OFF 時: color-text-muted

右 12px:  [ON/OFF トグルスイッチ] + [▼/▶ 展開アイコン]
          トグルとアイコンの間: 8px ギャップ

クリック対象: ヘッダー全体 (ON/OFFトグル以外の領域) → 展開/折りたたみ
             トグルスイッチ → エフェクトの ON/OFF のみ切り替え

ホバー: rgba(255,255,255,0.04) 背景

トグルスイッチ:
  幅: 36px, 高さ: 20px, 角丸: 10px
  OFF: rgba(255,255,255,0.15) 背景
  ON:  accent-purple (#7c3aed) 背景
  サム: 白, 直径 16px, アニメーション 150ms

展開アイコン:
  ▶ (閉じた状態) / ▼ (展開状態)
  16px, color-text-muted
  回転アニメーション: 150ms ease
```

### 8.3 詳細パネル展開アニメーション

```
高さ: 0 → コンテンツ高さ
duration: 200ms ease-out
内部コンテンツ: opacity 0 → 1 (100ms ディレイ後 100ms)
パディング: 0 16px 16px 16px
ボーダー上: 1px rgba(255,255,255,0.06) (展開時のみ表示)
```

### 8.4 各エフェクト詳細

#### Bloom (グロー効果)

```
デフォルト: ON
パラメーター:
  Threshold  範囲 0.0-1.0  デフォルト 0.60  ステップ 0.01
             説明: グロー発生の輝度閾値 (低いほど広範囲に適用)
  Intensity  範囲 0.0-2.0  デフォルト 0.80  ステップ 0.01
             説明: グロー強度
```

#### Motion Blur (モーションブラー)

```
デフォルト: OFF
パラメーター:
  Decay  範囲 0.0-0.99  デフォルト 0.70  ステップ 0.01
         説明: フレーム残像の減衰率 (高いほど残像が長い)
```

#### Chromatic Aberration (色収差)

```
デフォルト: ON
パラメーター:
  Strength   範囲 0.0-1.0  デフォルト 0.15  ステップ 0.01
             説明: 色ずれの強度
  Beat Boost 範囲 0.0-1.0  デフォルト 0.50  ステップ 0.01
             説明: beat_kick に応じた Strength の追加倍率
             エフェクト: Strength × (1.0 + beat_kick × Beat Boost)
```

#### Vignette (ビネット)

```
デフォルト: OFF
パラメーター:
  Inner  範囲 0.0-1.0  デフォルト 0.50  ステップ 0.01
         説明: ビネット効果の開始半径 (画面中心からの比率)
  Outer  範囲 0.0-1.0  デフォルト 0.85  ステップ 0.01
         説明: ビネット効果の終端半径
         制約: Outer > Inner を強制 (Inner を超えたら追従)
```

#### CRT Glitch (CRT ノイズ)

```
デフォルト: OFF
パラメーター:
  Scanline  範囲 0.0-1.0  デフォルト 0.30  ステップ 0.01
            説明: 走査線の濃さ
  Grain     範囲 0.0-1.0  デフォルト 0.20  ステップ 0.01
            説明: フィルムグレインの強度
  Glitch    範囲 0.0-1.0  デフォルト 0.10  ステップ 0.01
            説明: デジタルグリッチの頻度・強度
```

#### Tonemap (トーンマッピング)

```
デフォルト: ON
モード選択 (ラジオボタン):
  ◉ ACES     — 映画的なコントラスト (デフォルト)
  ○ Reinhard — ソフトなロールオフ

ラジオボタンスタイル:
  選択済み: accent-purple 塗り + 白リング
  未選択: rgba(255,255,255,0.20) リング
  ラベル: 13px, color-text-primary
  配置: 水平並列, 間隔 24px
```

---

## 9. スライダー共通仕様 (水平)

```
ヒット領域高さ: 24px
トラック: 4px (ホバー時 6px, 150ms ease-out)
  充填: linear-gradient(→, #7c3aed, #06b6d4)
  未充填: rgba(255,255,255,0.15)
サム (●): 直径 12px, 白
  通常: 非表示
  ホバー / ドラッグ中: フェードイン 100ms
  ドラッグ中: drop-shadow 0 0 6px rgba(124,58,237,0.6)

値ツールチップ:
  ドラッグ中: サム直上 8px に数値表示
  スタイル: rgba(10,10,15,0.9), 角丸 4px, 10px Mono

リアルタイム反映: スライダー移動のたびに ≤ 16ms で wgpu uniform 更新
```

---

## 10. Rust 側のデータモデル

```rust
// app/src/app/command.rs に追加
pub enum Command {
    // EQ
    SetEqBand { band: usize, db: f32 },  // band: 0-9
    SetEqPreset(EqPreset),
    ResetEq,

    // Visualizer
    SetSensitivity(f32),
    SetSmoothing(f32),
    SetBarCount(u32),
    SetColorScheme(ColorScheme),

    // Effects
    SetEffectEnabled { effect: EffectKind, enabled: bool },
    SetBloomThreshold(f32),
    SetBloomIntensity(f32),
    SetMotionBlurDecay(f32),
    SetChromaticStrength(f32),
    SetChromaticBeatBoost(f32),
    SetVignetteInner(f32),
    SetVignetteOuter(f32),
    SetCrtScanline(f32),
    SetCrtGrain(f32),
    SetCrtGlitch(f32),
    SetTonemapMode(TonemapMode),
}

#[derive(Debug, Clone, PartialEq)]
pub enum EqPreset {
    Flat,
    BassBoost,
    TrebleBoost,
    Vocal,
    Electronic,
    Custom,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ColorScheme {
    PurpleCyan,
    Fire,
    Ocean,
    Monochrome,
    Rainbow,
}

#[derive(Debug, Clone, PartialEq)]
pub enum EffectKind {
    Bloom,
    MotionBlur,
    ChromaticAberration,
    Vignette,
    CrtGlitch,
    Tonemap,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TonemapMode {
    Aces,
    Reinhard,
}

// app/src/app/event.rs に追加
pub enum Event {
    // EQ
    EqBandChanged { band: usize, db: f32 },
    EqPresetChanged(EqPreset),

    // Effects
    EffectSettingsChanged(EffectSettings),
}

#[derive(Debug, Clone)]
pub struct EffectSettings {
    pub bloom_enabled: bool,
    pub bloom_threshold: f32,
    pub bloom_intensity: f32,
    pub motion_blur_enabled: bool,
    pub motion_blur_decay: f32,
    pub chromatic_enabled: bool,
    pub chromatic_strength: f32,
    pub chromatic_beat_boost: f32,
    pub vignette_enabled: bool,
    pub vignette_inner: f32,
    pub vignette_outer: f32,
    pub crt_enabled: bool,
    pub crt_scanline: f32,
    pub crt_grain: f32,
    pub crt_glitch: f32,
    pub tonemap_enabled: bool,
    pub tonemap_mode: TonemapMode,
}
```

---

## 11. Slint コンポーネント分割

```
app/ui/components/
├── settings_drawer.slint      # ドロワー本体 + タブ切り替え
├── eq_tab.slint               # EQ タブ (10 バンド縦スライダー)
├── visualizer_tab.slint       # ビジュアライザー設定タブ
├── effects_tab.slint          # エフェクト設定タブ (アコーディオン)
├── vertical_slider.slint      # EQ 用縦スライダーコンポーネント
└── accordion_item.slint       # 汎用アコーディオン行コンポーネント
```

```slint
// settings_drawer.slint の骨格
component SettingsDrawer inherits Rectangle {
    width: 360px;
    background: rgba(10, 10, 15, 0.90);

    in-out property <int> active-tab: 0;  // 0=EQ, 1=Visualizer, 2=Effects

    // Header
    Rectangle { height: 48px; /* ... */ }

    // Tab Bar
    Rectangle {
        height: 40px;
        y: 48px;
        // [EQ] [Visualizer] [Effects] tabs
    }

    // Content area (scrollable)
    Flickable {
        y: 88px;
        height: parent.height - 88px;
        viewport-height: content.preferred-height;

        if active-tab == 0: EqTab { /* ... */ }
        if active-tab == 1: VisualizerTab { /* ... */ }
        if active-tab == 2: EffectsTab { /* ... */ }
    }
}
```

---

## 12. パフォーマンス要件

| 項目                             | 目標値              |
|--------------------------------|-------------------|
| スライダー変更 → wgpu uniform 反映 | ≤ 16ms (1 フレーム)  |
| EQ バンド変更 → 音声フィルター適用  | ≤ 10ms             |
| アコーディオン展開アニメーション     | 200ms ease-out     |
| タブ切り替え                      | ≤ 1 フレーム (即時)  |
| ドロワー開閉アニメーション           | 250ms / 200ms      |
