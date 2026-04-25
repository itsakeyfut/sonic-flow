# v1.0.0 - 初回安定リリース

## 概要

第一公開リリース。配布パッケージ (Windows/Linux/macOS)、ドキュメント整備、CI/CD、Web プレゼンスを完成させ、ユーザーが安心して使えるバージョンを公開する。

## ゴール

- 3プラットフォームでインストール可能なパッケージ
- ユーザー / 開発者向けドキュメント完成
- CI/CD によるリリース自動化
- ランディングページ
- v1.0.0 タグ + GitHub Release

## スコープ

### In Scope

- 配布パッケージ作成 (各 OS)
- ドキュメント整備
- CI/CD パイプライン
- Web プレゼンス (最小)
- リリース手順書

### Out of Scope

- 国際化 / 多言語 (v1.x)
- 自動アップデーター (v1.x)
- 商業ストア配信 (Microsoft Store, Mac App Store) (v2.x 検討)

## タスク

### 配布パッケージ

- [ ] **Windows**
  - MSI installer (`cargo-wix`)
  - portable .zip (実行ファイル + DLL)
  - コード署名 (オプション、有償証明書)
- [ ] **macOS**
  - DMG (`cargo-bundle`)
  - .app バンドル
  - Apple notarization (有償 Apple Developer 必要 / オプション)
- [ ] **Linux**
  - AppImage (推奨, 単一ファイル)
  - .deb (Debian / Ubuntu)
  - .rpm (Fedora / RHEL)
  - Flatpak (オプション)
- [ ] アイコン作成 (各サイズ + ICO/ICNS)
- [ ] バージョン情報リソースの埋め込み

### CI/CD

- [ ] GitHub Actions workflow
  - PR: cargo fmt / clippy / test
  - main push: 上記 + ビルド確認
  - tag push: マルチプラットフォームビルド + Release 作成
- [ ] release-please で changelog 自動生成
- [ ] アーティファクトの GitHub Release 自動アップロード
- [ ] バッジ (build status, version, license)

### ドキュメント

- [ ] **README.md 充実**
  - スクリーンショット / GIF
  - 機能一覧
  - インストール手順 (各 OS)
  - クイックスタート
  - 貢献ガイドへのリンク
- [ ] **ユーザーガイド** (`docs/user-guide.md`)
  - 基本操作
  - キーボードショートカット
  - ライブラリ管理
  - ビジュアライザーカスタマイズ
  - トラブルシューティング
- [ ] **開発者ドキュメント** (`docs/development.md`)
  - アーキテクチャ概要
  - ビルド手順
  - コーディング規約
  - 貢献の流れ
- [ ] **API ドキュメント**
  - `cargo doc` で生成
  - GitHub Pages に publish (cargo-doc-rs 設定)
- [ ] **CONTRIBUTING.md**
- [ ] **CHANGELOG.md** (release-please 出力)

### 法的事項

- [ ] ライセンス確認 (MIT or Apache-2.0)
- [ ] 依存ライブラリのライセンス互換性監査 (`cargo-license`, `cargo-deny`)
- [ ] 著作権ヘッダの一貫性
- [ ] サードパーティライセンス表示 (`THIRD_PARTY_LICENSES.txt` 自動生成)

### Web プレゼンス

- [ ] ランディングページ (静的サイト, GitHub Pages)
  - ヒーロー (スクリーンショット + キャッチ)
  - 機能ハイライト
  - スクリーンショット / 動画ギャラリー
  - ダウンロードリンク (各 OS)
  - GitHub リポジトリ リンク
- [ ] favicon
- [ ] OGP メタデータ

### リリース運用

- [ ] バージョニング規則 (semver)
- [ ] リリースノート テンプレート
- [ ] アナウンス手順 (Twitter / Reddit / HackerNews 等)
- [ ] Issue テンプレート整備 (Bug / Feature)
- [ ] PR テンプレート整備

### テスト

- [ ] フレッシュ環境でのインストール確認 (各 OS)
- [ ] アンインストール / 再インストールの確認
- [ ] ファイル関連付け (オプション)
- [ ] ユーザーフィードバック1周

## 完了条件

- [ ] 3プラットフォームでインストール可能
- [ ] README + ユーザーガイド + 開発者ドキュメント完成
- [ ] CI が緑
- [ ] v1.0.0 タグ作成 + GitHub Release アーティファクト公開
- [ ] ランディングページ公開
- [ ] 既知のクリティカルバグなし
- [ ] CHANGELOG 完成

## 依存

- v0.9.0 (品質目標達成、クロスプラットフォーム動作確認済み)

## リスク / 技術的検討事項

- **macOS 署名 / 公証**: Apple Developer 登録 ($99/年) が必要。公証なしだと Gatekeeper でブロック。回避手段 (右クリック開く) はあるが UX 悪
- **Linux 配布の多様性**: ディストロごとに依存が異なる。AppImage が最も汎用的
- **Windows SmartScreen**: 署名なし MSI は警告される。EV 証明書は高額
- **ドキュメント工数**: スクリーンショット/動画作成は時間がかかる。早めに取り掛かる
- **依存ライセンス**: GPL 系の依存があると配布形態に制約。事前監査必須
