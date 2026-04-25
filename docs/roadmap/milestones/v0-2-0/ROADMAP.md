# v0.2.0 - wgpu + Slint 統合の足場

## 概要

Slint ウィンドウ内に wgpu レンダリングを統合する技術的足場を確立する。実際のビジュアライザーはまだ実装せず、単純な三角形/フルスクリーンクワッドを描画してパフォーマンス目標とリサイズ動作を検証する。

## ゴール

- Slint ウィンドウの `visualizer-canvas` 領域に wgpu で描画
- 60fps 以上のフレームレートを安定維持
- ウィンドウリサイズに正しく追従
- 1時間連続動作でメモリリークなし

## スコープ

### In Scope

- wgpu / bytemuck 依存の再導入
- Slint と wgpu の統合方式の確定
- レンダリングループ
- フレームレート計測 / 表示

### Out of Scope

- 音声データの GPU 転送 (v0.3.0)
- 複雑なシェーダー (v0.3.0 以降)
- ポストエフェクト (v0.5.0)

## タスク

### Tech Spike: 統合方式の調査

- [ ] Slint 1.8 の wgpu 連携 API 調査 (`Window::set_rendering_notifier()`)
- [ ] OpenGL / Skia バックエンドとの相互運用性確認
- [ ] 推奨アプローチの決定 (Slint の rendering callback 経由が第一候補)
- [ ] 検証メモを `docs/specs/graphics.md` に追記

### `crates/sonic-shader` 再作成

- [ ] ワークスペースに `wgpu`, `bytemuck`, `pollster` 依存を再追加
- [ ] `crates/sonic-shader/` を新規作成 (空 lib.rs から)
- [ ] `RenderEngine` struct (Device / Queue / Surface 管理)
- [ ] `RenderTarget` 抽象 (Slint から渡されるテクスチャ or 独自サーフェス)
- [ ] エラー型 `ShaderError`

### Slint 統合

- [ ] `app` クレートで `RenderEngine` を保持
- [ ] AppWindow の rendering callback に登録
- [ ] `visualizer-canvas` 領域への描画 (座標変換)
- [ ] リサイズハンドリング (Surface 再構築)

### サンプル描画

- [ ] フルスクリーンクワッドの WGSL (vertex + fragment)
- [ ] 時間ベースで色グラデーション (`uniform time`)
- [ ] FPS 表示 (デバッグオーバーレイ on/off)

### パフォーマンス検証

- [ ] フレームタイム計測 (移動平均, p99)
- [ ] 60fps 安定動作の確認
- [ ] 120fps 目標の検証 (高リフレッシュレート環境)
- [ ] 1時間連続動作テスト (メモリリーク検出)

## 完了条件

- [ ] visualizer canvas 領域にカラーグラデーション描画
- [ ] 60fps 以上維持
- [ ] ウィンドウリサイズ時に追従 (歪みなし)
- [ ] FPS 表示が動く
- [ ] 1時間連続動作でメモリ増加なし

## 依存

- v0.1.0 の完了 (オーディオ再生がデバッグの妨げにならない状態)

## リスク / 技術的検討事項

- **Slint × wgpu の統合 API は版で変わる可能性**: 1.8 / 1.9 で API 改変があったため、最新動向の確認が必須
- **バックエンド選択**: Slint のデフォルトレンダラーと wgpu サーフェスの共存方法によっては、別ウィンドウや別 surface 戦略が必要
- **プラットフォーム差**: Vulkan (Linux/Windows) / Metal (macOS) / DX12 (Windows) でバックエンドが異なる。最低限 Windows での動作確認は必須
