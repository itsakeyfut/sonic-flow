# CLAUDE.md - Sonic Flow プロジェクト文脈

このファイルは、Sonic Flow プロジェクトに関する Claude への指示と文脈情報を含みます。

## 🎯 プロジェクト概要

**プロジェクト名**: Sonic Flow  
**技術スタック**: Rust + Slint  
**目標**: リリースレベル品質の音圧ビジュアライザー重視ミュージックプレイヤー  
**開発ステータス**: 設計フェーズ完了、実装開始準備中

## 🏗️ 設計哲学

### コア原則

1. **ビジュアライザー優先**: 音圧ビジュアライザーが最重要機能
2. **拡張性**: プラグインシステムによる機能追加
3. **品質重視**: リリースレベルの品質基準
4. **パフォーマンス**: リアルタイム処理とスムーズなアニメーション
5. **モジュラー設計**: 明確な責任分離

### 技術的制約

- **フレームワーク**: Slint UI を必須使用
- **言語**: Rust のみ使用
- **アーキテクチャ**: レイヤード・アーキテクチャ
- **設計パターン**: Repository, Strategy, Plugin パターン適用

## 📋 Claude への指示

### コーディング規則

```rust
// 1. モジュール構造の維持
// - 各機能は独立したモジュールに配置
// - pub(crate) を適切に使用
// - mod.rs でモジュール公開を管理

// 2. エラーハンドリング
// - カスタムエラー型を使用（thiserror使用）
// - Result<T, E> を一貫して使用
// - unwrap() は原則禁止（テスト以外）

// 3. 非同期処理
// - tokio を使用（マルチスレッドランタイム）
// - async/await パターンを適用
// - デッドロックを避ける設計

// 4. ドキュメンテーション
// - 全pub関数に /// コメント
// - 複雑なアルゴリズムに説明追加
// - 例を含むドキュメント
```

### 命名規則

- **構造体**: PascalCase (例: `AudioEngine`)
- **関数/変数**: snake_case (例: `process_audio`)
- **定数**: SCREAMING_SNAKE_CASE (例: `DEFAULT_SAMPLE_RATE`)
- **モジュール**: snake_case (例: `audio_engine`)
- **トレイト**: PascalCase + 動詞形 (例: `Visualizable`)

### 重要な設計決定

1. **音圧解析**: FFT 処理はリアルタイムで実行（RustFFT 使用）
2. **ビジュアライザー**: trait-based プラグインシステム
3. **UI 更新**: 120fps 目標、60fps 最低保証
4. **メモリ管理**: 音声データは効率的にストリーミング
5. **設定**: TOML 形式で外部化
6. **データベース**: SQLite + sqlx による型安全なクエリ

## 📁 重要ファイル参照

### 必読ドキュメント

- `docs/ARCHITECTURE.md`: 詳細アーキテクチャ設計
- `docs/SPECIFICATION.md`: 機能仕様書
- `docs/DIRECTORY.md`: ディレクトリ構成ガイド
- `docs/SYSTEM.md`: システム設計詳細

### 実装優先度

1. **Phase 1**: 基本音楽再生 + シンプルビジュアライザー
   - AudioEngine, BasicDecoder, SpectrumBarsVisualizer
2. **Phase 2**: プラグインシステム + 複数ビジュアライザー
   - PluginManager, VisualizerTraits, 追加ビジュアライザー
3. **Phase 3**: 高度な音響効果 + UI 改善
   - Equalizer, Effects, 改良された UI
4. **Phase 4**: 最適化 + リリース準備
   - パフォーマンス最適化、テスト、ドキュメント完成

## 🛠️ 開発ガイドライン

### 技術スタック詳細

```toml
[dependencies]
# UI Framework
slint = "1.0"

# 非同期処理
tokio = { version = "1.0", features = ["full", "rt-multi-thread"] }
futures = "0.3"

# 音声処理
rodio = "0.17"           # クロスプラットフォーム音声再生
symphonia = "0.5"        # 音声デコード
cpal = "0.15"            # 低レベル音声I/O
rustfft = "6.0"          # FFT計算

# データ管理
sqlx = { version = "0.7", features = ["sqlite", "chrono", "uuid"] }
serde = { version = "1.0", features = ["derive"] }
toml = "0.7"             # 設定ファイル

# エラーハンドリング
anyhow = "1.0"           # エラーチェーン
thiserror = "1.0"        # カスタムエラー

# 並行処理
crossbeam = "0.8"        # 並行データ構造
parking_lot = "0.12"     # 高性能Mutex
dashmap = "5.0"          # 並行HashMap
```

### コード品質

- **テストカバレッジ**: 80%以上維持
- **パフォーマンス**: Criterion ベンチマーク必須
- **メモリリーク**: 定期的な検証
- **並行性**: データ競合の徹底チェック

### ブランチ戦略

- `main`: 安定版
- `develop`: 開発版
- `feature/*`: 機能ブランチ
- `hotfix/*`: 緊急修正

### コミットメッセージ

```
feat: 新機能追加
fix: バグ修正
refactor: リファクタリング
perf: パフォーマンス改善
docs: ドキュメント更新
test: テスト追加/修正
```

## 🚨 重要な注意事項

### 絶対に避けること

1. **パニック**: unwrap()の多用（テスト以外）
2. **ブロッキング**: UI スレッドでの重い処理
3. **メモリリーク**: 不適切なライフタイム管理
4. **データ競合**: 不適切な並行処理
5. **デッドロック**: 複数ロック時の順序不整合

### パフォーマンス要件

- **音声遅延**: 50ms 以下
- **UI 応答性**: 16ms 以下（60fps）
- **ビジュアライザー**: 8.3ms 以下（120fps 目標）
- **メモリ使用量**: 100MB 以下（アイドル時）
- **CPU 使用率**: 5%以下（再生時）

## 🔧 具体的な実装指針

### モジュール間通信

```rust
// イベント駆動アーキテクチャ
pub enum AudioEvent {
    TrackChanged(TrackInfo),
    SpectrumUpdated(SpectrumData),
    PlaybackStateChanged(PlaybackState),
}

// チャンネルベース通信
use tokio::sync::mpsc;
type EventSender = mpsc::UnboundedSender<AudioEvent>;
type EventReceiver = mpsc::UnboundedReceiver<AudioEvent>;
```

### エラー処理パターン

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SonicFlowError {
    #[error("Audio engine error: {0}")]
    Audio(#[from] AudioError),
    #[error("Visualizer error: {0}")]
    Visualizer(#[from] VisualizerError),
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}
```

### プラグインシステム

```rust
// プラグインAPI定義
pub trait VisualizerPlugin: Send + Sync {
    fn metadata(&self) -> PluginMetadata;
    fn initialize(&mut self, config: &VisualizationConfig) -> Result<(), PluginError>;
    fn render(&mut self, data: &SpectrumData, canvas: &mut Canvas) -> Result<(), RenderError>;
    fn configure(&mut self, settings: &HashMap<String, PluginValue>) -> Result<(), ConfigError>;
}

// プラグイン登録マクロ
#[macro_export]
macro_rules! register_visualizer {
    ($plugin:ty) => {
        #[no_mangle]
        pub extern "C" fn create_visualizer() -> Box<dyn VisualizerPlugin> {
            Box::new(<$plugin>::default())
        }
    };
}
```

## 🔄 更新指示

このファイルは以下の場合に更新：

1. 設計方針の変更
2. 新しい技術的制約の追加
3. 実装中の重要な発見
4. パフォーマンス要件の調整
5. 依存関係の変更

---

**最終更新**: 2025-08-12  
**更新者**: Claude (プロジェクト再整理)  
**次回レビュー**: フェーズ 1 実装完了時
