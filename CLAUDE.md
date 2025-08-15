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

### 実装ワークフロー例

#### ケース 1: シンプルな機能追加

```bash
# ブランチ作成
git checkout -b feature/spectrum-bars-visualizer

# 実装とコミット
git add src/visualizer/plugins/spectrum_bars.rs tests/visualizer/spectrum_bars_test.rs
git commit -m "feat(visualizer): implement spectrum bars visualizer

- Add SpectrumBarsVisualizer with logarithmic scaling
- Support configurable bar count (32-128)
- Implement peak hold with decay animation
- Add comprehensive unit tests with mock data"

# スタイル修正
cargo fmt
cargo clippy --fix --allow-dirty
git add -A
git commit -m "style: apply rustfmt and clippy fixes"

# プッシュとPR作成
git push origin feature/spectrum-bars-visualizer
```

#### ケース 2: 複雑な機能（依存関係含む）

```bash
# ブランチ作成
git checkout -b feature/audio-effects-chain

# 1. 依存関係追加
git add Cargo.toml
git commit -m "deps: add dsp crate for audio effects processing"

# 2. 機能実装
git add src/audio/effects/ tests/audio/effects/
git commit -m "feat(audio): implement audio effects chain

- Add EffectsChain with plugin architecture
- Implement Equalizer, Reverb, Delay effects
- Support real-time parameter adjustment
- Add comprehensive integration tests"

# 3. リファクタリング（必要に応じて）
git add src/audio/buffer.rs src/audio/processor.rs
git commit -m "refactor(audio): extract common audio processing utilities"

# 4. スタイル修正
cargo fmt && cargo clippy --fix --allow-dirty
git add -A
git commit -m "style: apply rustfmt and clippy fixes"

# 5. ドキュメント更新
git add docs/api/audio_effects.md
git commit -m "docs: add audio effects API documentation"
```

### コミット品質チェックリスト

各コミット前に以下を確認：

- [ ] **単一責任**: コミットは 1 つの明確な変更のみ
- [ ] **コンパイル可能**: 各コミット時点でビルドが通る
- [ ] **テスト通過**: 関連するテストが全て通る
- [ ] **適切なメッセージ**: type(scope)形式で内容が明確
- [ ] **適切な粒度**: 大きすぎず小さすぎない変更単位

### 禁止パターン

```bash
# ❌ 避けるべきコミット例
git commit -m "fix stuff"                    # 不明確
git commit -m "feat: add feature and fix bugs"  # 複数の責任
git commit -m "wip: work in progress"        # 未完成をコミット

# ✅ 良いコミット例
git commit -m "fix(audio): resolve buffer overflow in decoder"
git commit -m "feat(ui): add dark theme support"
git commit -m "refactor(visualizer): extract color utilities"
```

Claude が機能実装を行う際は、以下の順序でコミットを提案してください：

#### 1. 実装前準備

```bash
# 依存関係が必要な場合
git commit -m "deps: add [crate-name] for [purpose]"

# 設定変更が必要な場合
git commit -m "config: [configuration change description]"
```

#### 2. メイン実装

```bash
# 機能実装（テスト含む）
git commit -m "feat([scope]): [feature description]

- [implementation detail 1]
- [implementation detail 2]
- [test coverage info]"
```

#### 3. 品質改善

```bash
# フォーマット適用
git commit -m "style: apply rustfmt and clippy fixes"

# リファクタリング（必要な場合）
git commit -m "refactor([scope]): [refactor description]"
```

#### 4. 文書化

```bash
# ドキュメント更新
git commit -m "docs: update [module] documentation"
```

### 実装出力フォーマット

機能実装の回答では、以下の形式で出力してください：

````markdown
## 実装: [機能名]

### 📋 実装概要

[機能の説明と設計判断]

### 📁 ファイル構成

- `src/[module]/[file].rs` - [ファイルの役割]
- `tests/[test_file].rs` - [テストの範囲]

### 🔨 実装

#### 1. 依存関係追加 (必要に応じて)

```toml
# Cargo.toml additions
[dependencies]
new-crate = "1.0"
```
````

**コミット**: `deps: add new-crate for [purpose]`

#### 2. メイン実装

[実際の Rust コード]

**コミット**: `feat([scope]): [feature description]`

#### 3. テスト実装

[テストコード]

**含まれる**: 上記の feat コミットに含める

#### 4. スタイル修正 (必要に応じて)

**コミット**: `style: apply rustfmt and clippy fixes`

### 📝 次のステップ

[この実装後に続けて実装すべき関連機能]

````

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
````

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

src/内のファイル全てを確認してください。

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

### コミット戦略

#### コミット粒度ルール

1. **一機能一コミット**: 各機能は独立したコミットとして実装
2. **品質改善は分離**: リンター・フォーマッター・リファクタは別コミット
3. **テストは機能と同時**: 機能実装とテストは同一コミット内
4. **設定変更は分離**: Cargo.toml 等の依存関係追加は別コミット

#### コミット順序（推奨）

```bash
# 1. 依存関係・設定追加
git commit -m "deps: add symphonia for audio decoding"

# 2. 機能実装（テスト含む）
git commit -m "feat: implement spectrum bars visualizer

- Add SpectrumBarsVisualizer struct
- Implement VisualizerPlugin trait
- Add logarithmic frequency scaling
- Add peak hold functionality
- Include comprehensive unit tests"

# 3. リンター・フォーマッター適用
git commit -m "style: apply rustfmt and clippy fixes"

# 4. リファクタリング（必要に応じて）
git commit -m "refactor: extract common drawing utilities"

# 5. ドキュメント更新
git commit -m "docs: update visualizer API documentation"
```

### コミットメッセージ規則

#### 基本フォーマット

```
<type>(<scope>): <subject>

[optional body]

[optional footer]
```

#### Type 一覧

```
feat:     新機能追加
fix:      バグ修正
refactor: リファクタリング
perf:     パフォーマンス改善
style:    フォーマット・リンター適用
test:     テスト追加/修正（機能変更なし）
docs:     ドキュメント更新
deps:     依存関係追加/更新
config:   設定ファイル変更
build:    ビルドシステム変更
ci:       CI/CD設定変更
revert:   コミット取り消し
```

#### Scope 例

```
audio:      音声処理関連
visualizer: ビジュアライザー関連
playlist:   プレイリスト関連
ui:         UI関連
config:     設定関連
plugin:     プラグインシステム関連
db:         データベース関連
```

#### 例

```bash
# 機能実装
feat(audio): implement FFT spectrum analyzer
feat(visualizer): add waveform display plugin
feat(playlist): add smart playlist functionality

# バグ修正
fix(audio): resolve memory leak in decoder
fix(ui): fix playlist sorting crash

# リファクタリング
refactor(visualizer): extract common rendering logic
refactor(audio): simplify buffer management

# パフォーマンス改善
perf(audio): optimize FFT calculation with SIMD
perf(visualizer): reduce memory allocations in render loop

# スタイル・品質
style: apply rustfmt to all source files
style(audio): fix clippy warnings in decoder module

# 依存関係
deps: add rustfft 6.2 for spectrum analysis
deps: update slint to 1.6 for UI improvements

# 設定変更
config: enable AVX2 optimizations in release build
config: add development vs production database paths
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
