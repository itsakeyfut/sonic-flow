# Sonic Flow ロードマップ

GPUシェーダー駆動の音圧ビジュアライザーを核とする音楽プレイヤーを、v1.0.0 (初回安定リリース) まで段階的に構築するための計画。

## 現状

- **段階**: v0.0.x (リファクタ完了、機能未実装の足場段階)
- **動作確認済み**: 基本再生 (Player + PlayerManager)、Slint UI 統合 (Command/Event + UiState global)
- **未実装**: ビジュアライザー、Position/Duration、メタデータ統合、ライブラリ、プレイリスト

## マイルストーン一覧

| バージョン | テーマ | フェーズ |
|------------|--------|----------|
| [v0.1.0](milestones/v0-1-0/ROADMAP.md) | オーディオ基盤の完成 | Phase 1: Foundation |
| [v0.2.0](milestones/v0-2-0/ROADMAP.md) | wgpu + Slint 統合の足場 | Phase 1: Foundation |
| [v0.3.0](milestones/v0-3-0/ROADMAP.md) | 初のビジュアライザー (Spectrum Bars) | Phase 1: Foundation |
| [v0.4.0](milestones/v0-4-0/ROADMAP.md) | ビジュアライザーバラエティ | Phase 2: Visual Variety |
| [v0.5.0](milestones/v0-5-0/ROADMAP.md) | ポストプロセス & エフェクト | Phase 2: Visual Variety |
| [v0.6.0](milestones/v0-6-0/ROADMAP.md) | ライブラリ管理 | Phase 3: Music Management |
| [v0.7.0](milestones/v0-7-0/ROADMAP.md) | プレイリスト | Phase 3: Music Management |
| [v0.8.0](milestones/v0-8-0/ROADMAP.md) | 設定 & テーマ | Phase 3: Music Management |
| [v0.9.0](milestones/v0-9-0/ROADMAP.md) | パフォーマンス & 品質 | Phase 4: Release |
| [v1.0.0](milestones/v1-0-0/ROADMAP.md) | 初回安定リリース | Phase 4: Release |

## 設計方針

### 各マイルストーンの粒度

- **1 マイルストーン = 1 PR にはしない**。複数の Issue → 複数の PR を経て完了する単位
- ゴールが明確で、テスト可能で、独立してリリース可能な機能セット
- スコープ外を明記し、次バージョンへの先送りを正当化する

### 順序の根拠

1. **Phase 1 (v0.1-0.3)**: 「動く再生 + 動くビジュアライザー」が最小プロダクト。ビジュアライザーがプロジェクトの核なので最優先
2. **Phase 2 (v0.4-0.5)**: ビジュアライザーを「魅力的」にする。複数種類 + ポストエフェクトで差別化
3. **Phase 3 (v0.6-0.8)**: 「日常使い」できるプレイヤーにする。ライブラリ・プレイリスト・設定
4. **Phase 4 (v0.9-1.0)**: 品質・配布

ライブラリ管理 (v0.6) を後ろに置いた理由は、ファイルダイアログ単体でビジュアライザー開発はテスト可能で、ライブラリは大きい作業だが緊急ではないため。

### スコープ外 (v1.0 以降に先送り)

以下は v1.x または v2.x で検討:

- **プラグインシステム** (外部作者によるカスタムビジュアライザー)
- **DSP エフェクト** (Equalizer, Compressor, Reverb)
- **ストリーミング** (Webradio, ネットワーク再生)
- **メタデータ書き込み** (タグ編集)
- **ライブ入力** (マイク → ビジュアライザー)
- **MIDI 連動**
- **配信機能** (OBS 連携など)

## パフォーマンス目標 (v1.0 達成必須)

| 項目 | 目標値 |
|------|--------|
| オーディオ遅延 | ≤ 50ms |
| UI 応答性 | ≤ 16ms (60fps) |
| ビジュアライザー描画 | ≤ 8.3ms (120fps) |
| メモリ (idle) | ≤ 100MB |
| メモリ (active) | ≤ 200MB |
| CPU 使用率 (再生時) | ≤ 5% |
| GPU 使用率 (ビジュアライザー時) | ≤ 30% |

## 関連ドキュメント

- [プロジェクトビジョン](../specs/vision.md)
- [グラフィックス仕様](../specs/graphics.md)
- [アーキテクチャパターン](../specs/architecture.md)
- [UI 設計仕様](../specs/ui.md)
