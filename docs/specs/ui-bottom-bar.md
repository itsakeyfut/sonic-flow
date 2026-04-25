# UI 詳細設計 — Bottom Bar

## 1. 概要

ウィンドウ下部に固定される 72px のコントロールバー。常に表示され、  
アルバムアート・トラック情報・再生コントロール・シーク・音量・設定アクセスを提供する。

---

## 2. レイアウト仕様

```
高さ: 72px (固定)
幅:   ウィンドウ幅 100%
背景: rgba(10, 10, 15, 0.85)
      + border-top: 1px rgba(255, 255, 255, 0.10)

┌──────────┬────────────────────┬───────────────────┬─────────────────────────────┬───────┬────┐
│ Album    │ Title (scroll)     │                   │                             │       │    │
│ Art      │ Artist             │  ◀◀   ▶▶   ■      │ ━━━━━━━━●━━━━━━  2:34/4:12  │ 🔊━━● │ ⚙  │
│ 48×48    │                    │  再生コントロール   │ シークバー + 時間             │ 音量  │設定│
└──────────┴────────────────────┴───────────────────┴─────────────────────────────┴───────┴────┘
   80px          flex (min:120px)       144px                   flex                  88px  44px
   ← 左パディング 16px                                                       右パディング 8px →
```

---

## 3. アルバムアートエリア

### 3.1 サムネイル表示

```
サイズ:    48 × 48px
角丸:      6px
マージン:   左 16px, 右 12px (サムネイルとトラック情報の間)
境界線:    1px rgba(255,255,255,0.12) (アート画像がない時も枠として表示)

未設定時:
  背景: linear-gradient(135deg, #7c3aed 0%, #06b6d4 100%)
  中央: ♪ アイコン (白, 20px)

カーソル: pointer (クリック可能を示す)
ホバー:   brightness(1.1) + scale(1.02) のマイクロアニメーション (150ms)
```

### 3.2 クリック時: アルバムアート拡大オーバーレイ

クリックするとビジュアライザー上に大きなアートワークを表示する。

```
┌────────────────────────────────────────────────┐
│ rgba(0,0,0, 0.75) dim オーバーレイ (全画面)      │
│                                                │
│          ┌────────────────────────┐            │
│          │                        │            │
│          │                        │            │
│          │       Album Art        │            │
│          │    400 × 400 px        │            │
│          │    (画像がなければ      │            │
│          │     グラデーション)     │            │
│          │                        │            │
│          └────────────────────────┘            │
│               Title                            │
│               Artist                           │
│                                                │
│        クリックまたは Esc で閉じる               │
└────────────────────────────────────────────────┘
```

**オーバーレイ仕様:**
```
z-index:   全 UI 最前面
アニメーション: フェードイン 200ms ease-out
アートサイズ: min(400px, ウィンドウ幅-64px, ウィンドウ高さ-200px) の正方形
角丸:      12px
shadow:    0 24px 80px rgba(0,0,0,0.8)
テキスト:   アート下 20px, Title 16px bold / Artist 13px secondary, テキスト中央揃え
閉じる:    オーバーレイ外クリック / Esc キー (フェードアウト 150ms)
```

---

## 4. トラック情報エリア

```
幅: flex (min 120px, max 240px)
垂直方向: 上下中央揃え

┌─────────────────────────────────┐
│ Track One - Long Song Title ...  │  ← 14px, weight 500, color-text-primary
│ Artist Name                      │  ← 12px, weight 400, color-text-secondary
└─────────────────────────────────┘
```

### マーキースクロール仕様

```
発動条件: テキストがエリア幅を超過する場合
待機時間: 表示から 2 秒後にスクロール開始
速度:     40px/秒
末尾処理: テキスト末尾の後に 2 秒停止 → 先頭に戻って再開

Slint 実装:
  clip: true で切り詰め
  Timer を使って x オフセットをアニメーション
  再生停止中はスクロール停止、先頭位置に戻す

未再生時: "--" / "--" を表示 (color-text-muted)
```

---

## 5. 再生コントロール

```
配置: 水平中央, 垂直中央
ボタン間隔: 12px

[◀◀]  [▶▶/⏸]  [■]
  前    再生/停止  停止

各ボタン: 40 × 40px のヒット領域, アイコンサイズ 18px
```

### ボタン状態テーブル

| ボタン     | 通常              | ホバー             | 押下               |
|----------|-----------------|------------------|-------------------|
| ◀◀ 前へ  | secondary グレー  | primary 白        | scale(0.92)       |
| ▶▶ 再生  | secondary グレー  | accent-purple     | scale(0.92)       |
| ⏸ 停止   | accent-purple    | accent-purple+10% | scale(0.92)       |
| ■ 停止   | secondary グレー  | primary 白        | scale(0.92)       |

**再生/停止アイコン切り替え:**
```
▶▶ (再生中でない) ⇔ ⏸ (再生中)
切り替えアニメーション: opacity 0 → 1 の 100ms フェード
Space キーで切り替え時も同じフィードバック (リップルなし)
```

---

## 6. シークバー

### 6.1 通常状態 / ホバー状態

```
通常:
  高さ: 4px
  トラック (未再生部分): rgba(255,255,255,0.18)
  充填 (再生済み部分):   linear-gradient(→, #7c3aed, #06b6d4)
  サム (●):             非表示

ホバー (マウスがシークバーエリアに乗った時):
  高さ: 6px (150ms ease-out で拡大)
  トラック: rgba(255,255,255,0.25)
  充填:     同上 + beat_kick 連動 (後述)
  サム (●): 表示 (直径 12px, 白, 150ms フェードイン)
            position: 現在再生位置
```

### 6.2 Beat 連動 glow

```
シークバー充填の glow:
  glow_alpha = beat_kick × 0.4
  box-shadow: 0 0 8px rgba(6, 182, 212, glow_alpha)  ← cyan

充填グラデーション:
  通常: #7c3aed → #06b6d4
  beat_kick 時: lerp(#7c3aed, #06b6d4, beat_kick) で右端が cyan に傾く
```

### 6.3 ドラッグシーク

```
操作:
  マウスダウン → ドラッグ → マウスアップ でシーク位置を確定
  ドラッグ中: サム位置がマウスに追従
  ドラッグ中: 時間テキストが現在のシーク先時間に更新 (リアルタイム)
  ドラッグ中: 音声はシーク先でプレビュー (実装コスト高い場合は skip)
  マウスアップ: Command::Seek(position) を送信

クリックシーク:
  シークバー上の任意の場所をクリック → 即時 Command::Seek(position)
```

### 6.4 ツールチップ (時間プレビュー)

```
表示条件: シークバーエリアにホバー中
位置:     マウスカーソル直上 8px
内容:     [1:48] (ホバー位置に対応する時間)
スタイル: 背景 rgba(10,10,15,0.9), 角丸 4px, パディング 4px 8px
         フォント: 12px JetBrains Mono, color-text-primary
アニメーション: フェードイン 100ms
```

### 6.5 時間表示テキスト

```
位置: シークバーの右端 (8px ギャップ)
形式: M:SS / M:SS  (例: 2:34 / 4:12)
      または MM:SS / MM:SS (10分以上の曲)
フォント: 12px JetBrains Mono, color-text-secondary
更新頻度: 1秒ごと (seek ドラッグ中はリアルタイム)
```

---

## 7. 音量コントロール

```
┌──────────────────────┐
│ 🔊  ━━━━━●━━━━  80%  │
└──────────────────────┘

アイコン: 音量に応じて変化
  0%:   🔇 (ミュート)
  1-33%:  🔈 (小)
  34-66%: 🔉 (中) ※Slint の絵文字対応に依存
  67-100%: 🔊 (大)
  アイコンクリック: ミュートトグル

スライダー:
  幅: 72px
  高さ: 4px (ホバー時 6px)
  操作: クリック / ドラッグ / マウスホイール (±5%)
  サム: ホバー時のみ表示 (10px, 白)
  パーセント表示: なし (ツールチップで表示)

ツールチップ: ホバー中にパーセント表示 "80%"
```

---

## 8. 設定アイコン

```
[⚙]

サイズ: 40 × 40px ヒット領域, アイコン 18px
位置:  右端 8px

状態:
  通常:   color-text-secondary
  ホバー: color-text-primary
  アクティブ (Settings ドロワー開): accent-purple + 背景 rgba(124,58,237,0.15)

クリック: Settings ドロワーをトグル (開いている場合は閉じる)
```

---

## 9. 空状態 (No Track)

```
┌──────────────────────────────────────────────────────────────────┐
│ [♪ グラデーション]  --                │  ◀◀   ▶▶   ■   │━━━━━━━━━━━  0:00/0:00│🔊━━●│⚙ │
│                    --                │  (ボタン無効)    │ (シーク不可)           │     │   │
└──────────────────────────────────────────────────────────────────┘

無効状態のスタイル:
  opacity: 0.4
  pointer-events: none (クリック無効)

再生/停止ボタン: 無効 (opacity 0.4)
シークバー充填: 0% (トラックのみ表示)
時間表示: "0:00 / 0:00"
```

---

## 10. Slint コンポーネント分割

```slint
// app/ui/components/bottom_bar.slint

component BottomBar inherits Rectangle {
    height: 72px;
    background: rgba(10, 10, 15, 0.85);

    // --- Album Art ---
    AlbumArtThumb { ... }

    // --- Track Info ---
    TrackInfo {
        title: UiState.track-title;
        artist: UiState.track-artist;
    }

    // --- Playback Controls ---
    PlaybackControls {
        is-playing: UiState.is-playing;
        callback play-pause-clicked -> UiState.toggle-playback;
        callback prev-clicked -> UiState.prev-track;
        callback stop-clicked -> UiState.stop;
    }

    // --- Seek Bar ---
    SeekBar {
        position: UiState.position;    // 0.0-1.0
        duration: UiState.duration;    // 秒
        beat-kick: UiState.beat-kick;  // 0.0-1.0
        callback seek(float) -> UiState.seek;
    }

    // --- Volume ---
    VolumeControl {
        volume: UiState.volume;        // 0.0-1.0
        callback volume-changed(float) -> UiState.set-volume;
    }

    // --- Settings Button ---
    SettingsButton {
        active: UiState.settings-open;
        callback clicked -> UiState.toggle-settings;
    }
}
```

---

## 11. パフォーマンス要件

| 項目                    | 目標値            |
|-----------------------|-----------------|
| シークバードラッグ遅延    | ≤ 16ms (1 フレーム) |
| マーキースクロール CPU   | ≤ 0.5% (Timer 駆動) |
| Beat glow 更新          | 60Hz (解析スレッド同期) |
| アルバムアートオーバーレイ フェードイン | 200ms |
