# v0.4.0 - ビジュアライザーバラエティ

## 概要

複数のビジュアライザータイプを実装し、ランタイム切り替えを可能にする。共通シェーダーライブラリを整備し、開発体験を向上させるためのホットリロード機能を導入する。

## ゴール

- 3種類以上のビジュアライザー (Spectrum Bars, Waveform, Circular)
- 切り替え時間 < 100ms
- 各々が視覚的に明確に異なる
- WGSL ホットリロード (debug build) が動作

## スコープ

### In Scope

- Waveform (オシロスコープ) ビジュアライザー
- Circular (リング型) ビジュアライザー
- ランタイム切り替え機構
- 共通 WGSL ライブラリ (ノイズ, SDF, 色変換)
- シェーダーホットリロード

### Out of Scope

- パーティクルシステム (v0.5.0 で検討)
- ポストプロセス (v0.5.0)
- 3D ビジュアライザー (v1.x 以降)

## タスク

### Waveform ビジュアライザー

- [ ] `crates/sonic-shader/src/shaders/waveform.wgsl`
- [ ] 時間領域サンプルを直接描画 (FFT ではなく raw waveform)
- [ ] ライン描画 (line strip + width)
- [ ] グロー / ブルーム前段の発光表現
- [ ] `WaveformVisualizer` (sonic-visualizer)

### Circular ビジュアライザー

- [ ] `circular.wgsl`
- [ ] スペクトラムを極座標で展開 (放射状)
- [ ] 中央ロゴエリアの確保 (将来のアルバムアート用)
- [ ] 周波数帯域ごとに色相変化
- [ ] `CircularVisualizer`

### ランタイム切り替え

- [ ] `VisualizerRegistry` (name → factory)
- [ ] `Command::ChangeVisualizer(String)`
- [ ] ビジュアライザーごとに固有 uniform をサポート
- [ ] スムーズトランジション (フェード in/out)

### 共通 WGSL ライブラリ

- [ ] `common.wgsl` の充実
  - HSV ↔ RGB 変換
  - Color palettes (viridis, plasma, magma)
  - smoothstep, remap, polar 座標変換
- [ ] `noise.wgsl`
  - Simplex noise
  - Perlin noise
  - FBM (Fractal Brownian Motion)
- [ ] `sdf.wgsl`
  - 円, 矩形, ライン, ダイヤモンド
  - 結合演算 (union, smin)
- [ ] WGSL の include 戦略 (build.rs で連結 or naga 前処理)

### ホットリロード

- [ ] debug build でのみ有効化 (`#[cfg(debug_assertions)]`)
- [ ] WGSL ファイル変更検知 (notify crate を再導入)
- [ ] シェーダー再コンパイル
- [ ] エラー時は前のシェーダーを継続使用 (画面が固まらない)
- [ ] ログにシェーダーエラーを表示

### UI 統合

- [ ] Visualizer 選択 ComboBox (3種類)
- [ ] プリセット保存/読み込みの足場 (v0.8.0 で完成)

### テスト

- [ ] 各ビジュアライザーで正常動作
- [ ] 切り替え 100ms 以内
- [ ] hot reload で WGSL 編集 → 即反映 (dev mode)
- [ ] パフォーマンス目標維持

## 完了条件

- [ ] 3種類のビジュアライザー (Spectrum Bars / Waveform / Circular)
- [ ] ComboBox で切り替え可能
- [ ] 切り替え時間 < 100ms
- [ ] 全ビジュアライザーで 60fps 維持
- [ ] WGSL ホットリロード (dev mode) が動く

## 依存

- v0.3.0 (FFT パイプライン + 1つ目のビジュアライザー)

## リスク / 技術的検討事項

- **WGSL の include 仕組み**: WGSL 自体は include 機構を持たないため、build.rs で文字列連結する自前実装が必要
- **uniform レイアウト**: ビジュアライザーごとに固有パラメータが必要だが、共通 uniform を保ちつつ拡張する設計
- **ホットリロードのフィードバック**: コンパイルエラーをユーザーに見せる方法 (ログ? UI overlay?)
