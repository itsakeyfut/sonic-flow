# UI Design Specification — Sonic Flow

## 1. 設計コンセプト

**Immersive Glassmorphism**: wgpu ビジュアライザーが画面全体を占め、すべての UI 要素はその上に浮かぶ半透明のガラスパネルとして重なる。音楽体験・映像体験を最優先し、UI は必要なときだけ存在感を持つ。

---

## 2. レイヤー構造

```
┌─────────────────────────────────────────────────────────┐
│  Layer 3: Drawer 系 (Playlist / Settings)  ← 最前面      │
│  Layer 2: Top Bar + Bottom Bar             ← 常時表示    │
│  Layer 1: Slint UI (透明ルート)                          │
│  Layer 0: wgpu Surface (全画面レンダリング) ← 最背面      │
└─────────────────────────────────────────────────────────┘
```

wgpu は `Window::set_rendering_notifier()` 経由で Slint ウィンドウ内に統合される。  
Slint の UI 要素はすべてガラス効果付きオーバーレイとして wgpu 描画面の上に重なる。

---

## 3. カラー / テーマ仕様 (Glassmorphism Dark)

### 3.1 ベースカラー

| トークン               | 値                           | 用途                          |
|----------------------|------------------------------|-------------------------------|
| `color-bg`           | `#0a0a0f`                    | ウィンドウ背景 (ビジュアライザーなし時) |
| `color-glass`        | `rgba(255, 255, 255, 0.05)`  | ガラスパネル背景                |
| `color-glass-hover`  | `rgba(255, 255, 255, 0.08)`  | ホバー時のガラスパネル           |
| `color-glass-active` | `rgba(255, 255, 255, 0.12)`  | アクティブ/選択状態              |
| `color-border`       | `rgba(255, 255, 255, 0.12)`  | パネルボーダー                  |
| `color-border-bright`| `rgba(255, 255, 255, 0.20)`  | 強調ボーダー (アクティブタブ)    |

### 3.2 アクセントカラー

| トークン               | 値          | 用途                          |
|----------------------|-------------|-------------------------------|
| `color-accent-purple`| `#7c3aed`   | プライマリアクセント (アクティブ状態) |
| `color-accent-cyan`  | `#06b6d4`   | セカンダリアクセント (Beat 連動グロー) |
| `color-accent-glow`  | `rgba(124, 58, 237, 0.3)` | 紫アクセントのグロー効果 |

### 3.3 テキストカラー

| トークン           | 値          | 用途              |
|------------------|-------------|------------------|
| `color-text-primary` | `#e2e8f0` | メインテキスト      |
| `color-text-secondary` | `#94a3b8` | サブテキスト、ラベル |
| `color-text-muted`   | `#475569`  | 非アクティブ要素    |

### 3.4 ガラスエフェクト (Slint 実装)

```slint
// パネル共通スタイル
Rectangle {
    background: rgba(255, 255, 255, 0.05);
    border-color: rgba(255, 255, 255, 0.12);
    border-width: 1px;
    // Slint の backdrop-filter は未サポートのため、
    // ガラス効果は半透明 background で近似する
}
```

### 3.5 タイポグラフィ

| 用途              | フォント         | サイズ   | ウェイト |
|-----------------|----------------|--------|--------|
| 曲名             | Inter / System | 14px   | 500    |
| アーティスト名     | Inter / System | 12px   | 400    |
| タブラベル         | Inter / System | 12px   | 500    |
| 時間表示          | JetBrains Mono | 12px   | 400    |
| ドロワーヘッダー   | Inter / System | 14px   | 600    |

---

## 4. 全体レイアウト

### 4.1 通常ウィンドウ時

```
┌─ OS タイトルバー ──────────────────────────────────────────┐
│                                                            │
│ ┌──────────────────────────────────────────────────────┐  │
│ │ [2D Bars][3D Bars][Particles][Fractal][Wave][Fall]   │  │  ← Top Bar (glass, 48px)
│ │                                        [Neon ▼] [≡] │  │
│ │                                                      │  │
│ │                                                      │  │
│ │              wgpu Visualizer (全画面)                 │  │  ← Layer 0
│ │                                                      │  │
│ │                                                      │  │
│ │                                                      │  │
│ ├──────────────────────────────────────────────────────┤  │
│ │ [🎵] Title ─────── Artist  │ ◀◀ ▶▶ ■ │━━━━●━━━ 2:34/4:12│ 🔊━━●━ │ ⚙ │
│ └──────────────────────────────────────────────────────┘  │  ← Bottom Bar (glass, 72px)
└────────────────────────────────────────────────────────────┘
```

### 4.2 フルスクリーン時 (F11)

```
┌──────────────────────────────────────────────────────────────┐ ← OS 枠なし
│ [2D Bars][3D Bars][Particles][Fractal][Wave][Fall][Neon▼][≡] │ ← glass, 48px
│                                                              │
│                 wgpu Visualizer (全画面最大化)                 │
│                                                              │
│ [🎵] Title ─── Artist │ ◀◀ ▶▶ ■ │━━━●━━ 2:34/4:12│🔊━●━│ ⚙ │ ← glass, 72px
└──────────────────────────────────────────────────────────────┘
```

---

## 5. Top Bar

### 5.1 レイアウト

```
高さ: 48px
幅:   100% (ウィンドウ幅)
背景: rgba(255,255,255,0.05) + border-bottom: 1px rgba(255,255,255,0.12)

┌─────────────────────────────────────────────────────────────┐
│ [2D Bars] [3D Bars] [Particles] [Fractal] [Wave] [Waterfall] │    [Neon ▼]  [≡]
└─────────────────────────────────────────────────────────────┘
  ← タブグループ (左寄せ、間隔 4px) →              ← 右寄せ →
```

### 5.2 ビジュアライザータブ

| 状態         | 背景                        | ボーダーボトム                    | テキスト           |
|------------|-----------------------------|---------------------------------|------------------|
| 通常         | なし                        | なし                             | `color-text-secondary` |
| ホバー       | `color-glass-hover`         | 1px `color-border-bright`       | `color-text-primary` |
| アクティブ   | `color-glass-active`        | 2px `color-accent-purple` (glow) | `color-text-primary` |

```
パディング: 8px 16px
角丸:       6px (上のみ)
アニメーション: background, border-color 150ms ease
```

タブ数が 8 本以上になった場合、タブバーは横スクロール可能になる（スクロールバー非表示、スワイプ or マウスホイールで操作）。

### 5.3 プリセットドロップダウン

```
幅:   120px
高さ: 32px
位置: 右端から 52px

[Neon ▼]  ← 現在のプリセット名 + 矢印アイコン

クリックで展開:
┌──────────────┐
│ ✓ Neon       │
│   Default    │
│   Cyberpunk  │
│   Minimal    │
│   Cinematic  │
│   Retro      │
│ ── ─────── ──│
│   My Preset  │
│ + 保存...    │
└──────────────┘
```

### 5.4 プレイリストボタン (≡)

```
幅:   40px
高さ: 32px
位置: 右端 8px

[≡]  ← ハンバーガーアイコン (開時は × に変化、200ms cross-fade)
```

---

## 6. Bottom Bar

### 6.1 レイアウト (72px 固定高)

```
┌────────┬─────────────────────┬────────────────────┬──────────────────┬────┐
│ Album  │ Title (scroll)      │                    │                  │    │
│ Art    │ Artist              │ ◀◀  ▶▶  ■          │ ━━━━●━━━ 2:34/4:12│🔊━●│ ⚙ │
│ 48×48  │                     │  再生コントロール    │ シークバー + 時間  │音量│設定│
└────────┴─────────────────────┴────────────────────┴──────────────────┴────┘
  80px        ~200px (flex)           140px                flex           80px 44px
```

### 6.2 各要素仕様

#### アルバムアート

```
サイズ:     48×48px
角丸:       4px
マージン:   12px 左右
未設定時:   グラデーション四角形 (♪ アイコン付き、accent-purple)
クリック:   アルバムアートの拡大表示 (フルスクリーンオーバーレイ、将来機能)
```

#### 曲名 / アーティスト名

```
曲名:
  フォント:   14px, weight 500, color-text-primary
  overflow:  テキストが溢れる場合は marquee スクロール (3秒後開始、速度 40px/s)

アーティスト名:
  フォント:   12px, weight 400, color-text-secondary
  overflow:  省略記号 (...) で切り詰め

最大幅:     flex, min 120px, max 240px
```

#### 再生コントロール

```
[◀◀ 前のトラック]  [▶▶ 再生/一時停止]  [■ 停止]

ボタンサイズ: 32×32px
アイコンサイズ: 18px
間隔: 8px

▶▶ ボタン:
  通常: color-text-secondary
  ホバー: color-text-primary, scale(1.1)
  再生中: color-accent-purple
  アニメーション: 150ms ease
```

#### シークバー + 時間表示

```
┌────────────────────────────────────┐
│ ━━━━━━━━━━●━━━━━━━━━━━━━  2:34 / 4:12 │
└────────────────────────────────────┘

シークバー:
  高さ:     4px (ホバー時 6px に拡大)
  トラック: rgba(255,255,255,0.2)
  充填:     linear-gradient(→, color-accent-purple, color-accent-cyan)
  サム:     白い円 (直径 12px, ホバー時に表示)
  操作:     クリックでシーク、ドラッグでシーク

時間表示:
  フォント: 12px, JetBrains Mono, color-text-secondary
  形式:    MM:SS / MM:SS
```

#### 音量スライダー

```
[🔊] ━━━●━━

アイコン: 🔊 (音量に応じて 🔇/🔈/🔊 に変化)
スライダー幅: 72px
操作: クリック、ドラッグ、マウスホイール (±5%)
```

#### 設定アイコン

```
[⚙]

サイズ: 32×32px
位置: 右端 8px
クリック: Settings ドロワーを開く (右からスライドイン)
```

---

## 7. Playlist ドロワー

### 7.1 開閉アニメーション

```
幅:            320px
開く:          right: -320px → right: 0, duration 250ms, ease-out
閉じる:        right: 0 → right: -320px, duration 200ms, ease-in
背景:          rgba(10, 10, 15, 0.85) + blur近似
ボーダー左:    1px rgba(255,255,255,0.15)
```

### 7.2 レイアウト

```
┌────────────────────────────────────┐
│ ≡ Playlist                      ✕ │  ← ヘッダー 48px
│ ─────────────────────────────────  │
│ + ファイル追加                 🔀   │  ← アクションバー 36px
│ ─────────────────────────────────  │
│                                    │
│ 🎵 01. Track One                   │  ← リストアイテム 56px
│    Artist Name                  3:21│
│ ─────────────────────────────────  │
│ ▶ 02. Track Two (再生中)      ●   │  ← アクティブ (accent-purple)
│    Artist Name                  4:12│
│ ─────────────────────────────────  │
│ 🎵 03. Track Three                 │
│    Artist Name                  2:55│
│                                    │
│         (スクロール可能)             │
└────────────────────────────────────┘
```

### 7.3 リストアイテム仕様

```
高さ:   56px
パディング: 12px 16px

通常:
  背景: なし
  テキスト: color-text-primary (曲名) / color-text-secondary (アーティスト)

ホバー:
  背景: rgba(255,255,255,0.05)
  右端に [▶ 再生] ボタンが出現 (32px)

再生中:
  左端に紫のアクティブインジケーター (3px 縦線)
  テキスト: color-accent-purple
  右端: ● 音楽波形アニメーションアイコン

右クリック / 長押し:
  コンテキストメニュー: [再生] [プレイリストから削除] [ファイルの場所を開く]
```

---

## 8. Settings ドロワー

### 8.1 開閉アニメーション

Playlist ドロワーと同仕様 (幅 360px, 右からスライドイン)。  
Playlist ドロワーと同時に開く場合は Settings が最前面に重なる。

### 8.2 レイアウト

```
┌──────────────────────────────────────┐
│ ⚙ Settings                        ✕ │  ← ヘッダー 48px
│ ─────────────────────────────────── │
│ [EQ]  [Visualizer]  [Effects]        │  ← セクションタブ 40px
│ ─────────────────────────────────── │
│                                      │
│  ▼ EQ タブ選択時:                    │
│                                      │
│  10-Band Equalizer                   │
│  ▁▃▅▇▅▃▁▃▅▇  ← ドラッグ可能バー    │
│  32 64 125 250 500 1k 2k 4k 8k 16k │
│                                      │
│  ─────────────────────────────────  │
│  ▼ Visualizer タブ選択時:            │
│                                      │
│  Sensitivity    ━━━━●━━━  1.5        │
│  Smoothing      ━━━●━━━━  0.85       │
│  Bar Count      ━━━━━●━━  128        │
│  Color Scheme   [Neon ▼]             │
│                                      │
│  ─────────────────────────────────  │
│  ▼ Effects タブ選択時:               │
│                                      │
│  Bloom          [ON/OFF] ━━━●━  0.6  │
│  Motion Blur    [ON/OFF] ━━●━━  0.85 │
│  Chroma Aberr.  [ON/OFF] ━●━━━  0.003│
│  Vignette       [ON/OFF]             │
│  CRT Glitch     [ON/OFF]             │
│                                      │
│  Tonemap  ● ACES  ○ Reinhard         │
│                                      │
└──────────────────────────────────────┘
```

### 8.3 スライダー共通仕様

```
高さ:   4px (ホバー時 6px)
幅:     160px
トラック: rgba(255,255,255,0.15)
充填:   color-accent-purple
サム:   白い円 (12px)
右に数値ラベル表示 (color-text-secondary)
ダブルクリックで数値直接入力
```

---

## 9. アニメーション仕様

### 9.1 遷移一覧

| 操作              | 種別        | Duration | Easing      |
|-----------------|------------|----------|-------------|
| タブ切り替え (ビジュアライザー) | ビジュアライザー側フェード | 300ms | ease-in-out |
| Playlist 開く    | スライドイン   | 250ms    | ease-out    |
| Playlist 閉じる  | スライドアウト  | 200ms    | ease-in     |
| Settings 開く    | スライドイン   | 250ms    | ease-out    |
| Settings 閉じる  | スライドアウト  | 200ms    | ease-in     |
| タブホバー        | 背景フェード   | 150ms    | ease        |
| 再生ボタン押下    | scale 変化   | 100ms    | ease-out    |
| シークバーホバー   | 高さ拡大      | 150ms    | ease-out    |
| ≡ → ✕ アイコン変化| cross-fade  | 200ms    | ease        |
| プリセット切り替え | (wgpu 側)    | 300ms    | ease-in-out |

### 9.2 Beat 連動 UI アニメーション

Beat 検出 (`beat_kick`) に連動して UI 要素もわずかに反応させる。過度にならないよう微妙な範囲に留める。

```
対象要素         反応                              強度
シークバー充填   cyan グロー強度一時増加            beat_kick × 0.3
アクティブタブ   accent-purple glow 一時増加        beat_kick × 0.2
音量アイコン     scale(1.0 + beat_kick × 0.05)     微細
```

---

## 10. キーボードショートカット

| キー          | 操作                              |
|-------------|----------------------------------|
| `Space`     | 再生/一時停止                       |
| `←` / `→`   | 5秒シーク                          |
| `↑` / `↓`   | 音量 +5% / -5%                   |
| `V`         | 次のビジュアライザーに循環切り替え    |
| `P`         | 次のプリセットに循環切り替え          |
| `L`         | プレイリストドロワー 開/閉           |
| `S`         | 設定ドロワー 開/閉                   |
| `F11`       | フルスクリーントグル                 |
| `Esc`       | 開いているドロワーを閉じる            |

---

## 11. ウィンドウリサイズ対応

| ウィンドウ幅  | 挙動                                              |
|------------|---------------------------------------------------|
| ≥ 1280px   | フルレイアウト (タブ全表示)                          |
| 800-1279px | タブを横スクロール可能に (overflow: hidden + scroll) |
| < 800px    | タブをドロップダウンに折り畳み                        |

- Bottom Bar は常に 72px 固定高
- Top Bar は常に 48px 固定高
- Playlist / Settings ドロワーは幅固定 (320px / 360px)、ウィンドウ幅 < 480px では全幅に変更

---

## 12. 空状態 (プレイリスト未読み込み時)

```
┌─────────────────────────────────────────┐
│ [Tab Bar]                   [≡]         │
│                                         │
│                                         │
│              ♪                          │
│                                         │
│     ファイルをドロップ                    │  ← wgpu キャンバスに直接表示
│     または                               │
│     [ファイルを開く]                     │
│                                         │
│                                         │
│ ─────────────── No Track ─────────────  │  ← Bottom Bar
│ [🎵]  --  /  --  │ ◀◀  ▶▶  ■ │━━━0:00/0:00│ 🔊━━●━ │ ⚙ │
└─────────────────────────────────────────┘
```

- ドラッグ＆ドロップで音楽ファイル (.mp3 / .flac / .wav / .ogg / .aac) を読み込み可能
- [ファイルを開く] ボタンは `rfd` ファイルダイアログを起動
- wgpu キャンバスにはアイドル時のグラデーションアニメーション（ゆっくりとした色相回転）を表示

---

## 13. Slint コンポーネント設計

### 13.1 コンポーネント一覧

```
app/ui/
├── app.slint               # AppWindow ルート
├── components/
│   ├── top_bar.slint       # TopBar コンポーネント
│   ├── bottom_bar.slint    # BottomBar コンポーネント
│   ├── visualizer_tab.slint# VisualizerTab (タブ1枚)
│   ├── playlist_drawer.slint # PlaylistDrawer
│   ├── playlist_item.slint # PlaylistItem (リスト行)
│   ├── settings_drawer.slint # SettingsDrawer
│   ├── seek_bar.slint      # SeekBar
│   ├── volume_knob.slint   # VolumeKnob
│   └── glass_panel.slint   # GlassPanel (共通ガラスコンテナ)
└── themes/
    └── dark_glass.slint    # カラートークン定義
```

### 13.2 UiState グローバル拡張 (既存 + 追加)

```slint
export global UiState {
    // === 既存 ===
    in-out property <bool> is-playing;
    in-out property <float> volume;
    in-out property <float> position;
    in-out property <float> duration;
    in property <string> track-path;

    // === 追加: ビジュアライザー ===
    in property <[string]> available-visualizers;
    in-out property <string> active-visualizer: "2d_spectrum_bars";
    callback visualizer-changed(string);

    // === 追加: プリセット ===
    in property <[string]> available-presets;
    in-out property <string> active-preset: "Default";
    callback preset-changed(string);

    // === 追加: ドロワー ===
    in-out property <bool> playlist-open: false;
    in-out property <bool> settings-open: false;

    // === 追加: Beat (ビート連動アニメーション用) ===
    in property <float> beat-kick;     // 0.0-1.0 指数減衰値
    in property <float> beat-phase;    // 0.0-1.0 BPM 位相

    // === 追加: フルスクリーン ===
    in-out property <bool> fullscreen: false;
    callback fullscreen-toggled();
}
```

---

## 14. 実装優先度

| Priority | コンポーネント             | 対応 Issue |
|----------|--------------------------|-----------|
| P1       | GlassPanel / カラートークン | #35       |
| P1       | BottomBar (静的)          | #35       |
| P1       | TopBar + VisualizerTab   | #35       |
| P2       | PlaylistDrawer            | #28, #35  |
| P2       | SeekBar / VolumeKnob     | #29       |
| P3       | SettingsDrawer (EQ タブ)  | #38, #42  |
| P3       | SettingsDrawer (Visualizer タブ) | #35 |
| P4       | Beat 連動アニメーション      | #60       |
| P4       | SettingsDrawer (Effects タブ) | #70    |
