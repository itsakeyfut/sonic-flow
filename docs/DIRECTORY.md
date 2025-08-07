# Directory

```
sonic-flow/
├── Cargo.toml                    # プロジェクト設定
├── Cargo.lock
├── README.md
├── LICENSE
├── CHANGELOG.md
├── .gitignore
├── .github/
│   └── workflows/
│       ├── ci.yml               # CI/CD設定
│       ├── release.yml
│       └── security.yml
│
├── src/
│   ├── main.rs                  # エントリーポイント
│   ├── lib.rs                   # ライブラリルート
│   │
│   ├── app/                     # アプリケーション層
│   │   ├── mod.rs
│   │   ├── controller.rs        # メインコントローラー
│   │   ├── state.rs            # アプリケーション状態管理
│   │   ├── events.rs           # イベントハンドリング
│   │   └── lifecycle.rs        # アプリライフサイクル
│   │
│   ├── ui/                      # Slint UI層
│   │   ├── mod.rs
│   │   ├── main_window.slint    # メインウィンドウUI
│   │   ├── components/          # UIコンポーネント
│   │   │   ├── player_controls.slint
│   │   │   ├── playlist_view.slint
│   │   │   ├── library_browser.slint
│   │   │   ├── visualizer_canvas.slint
│   │   │   ├── settings_panel.slint
│   │   │   └── common.slint
│   │   ├── themes/             # テーマ定義
│   │   │   ├── dark.slint
│   │   │   ├── light.slint
│   │   │   └── custom.slint
│   │   └── bindings.rs         # Rust-Slint バインディング
│   │
│   ├── audio/                  # オーディオエンジン
│   │   ├── mod.rs
│   │   ├── engine.rs           # メインオーディオエンジン
│   │   ├── decoder/            # デコーダー実装
│   │   │   ├── mod.rs
│   │   │   ├── mp3.rs         # MP3デコーダー
│   │   │   ├── flac.rs        # FLACデコーダー
│   │   │   ├── wav.rs         # WAVデコーダー
│   │   │   └── ogg.rs         # OGGデコーダー
│   │   ├── renderer.rs         # オーディオレンダリング
│   │   ├── effects/            # 音響効果
│   │   │   ├── mod.rs
│   │   │   ├── equalizer.rs   # イコライザー
│   │   │   ├── reverb.rs      # リバーブ
│   │   │   └── crossfade.rs   # クロスフェード
│   │   └── analysis/           # 音声解析
│   │       ├── mod.rs
│   │       ├── fft.rs         # FFT処理
│   │       ├── spectrum.rs    # スペクトラム解析
│   │       └── meter.rs       # レベルメーター
│   │
│   ├── visualizer/             # ビジュアライザーエンジン
│   │   ├── mod.rs
│   │   ├── engine.rs          # ビジュアライザーエンジン
│   │   ├── traits.rs          # ビジュアライザートレイト定義
│   │   ├── renderer.rs        # レンダリング共通処理
│   │   ├── plugins/           # ビジュアライザープラグイン
│   │   │   ├── mod.rs
│   │   │   ├── spectrum_bars.rs    # スペクトラムバー
│   │   │   ├── waveform.rs         # 波形表示
│   │   │   ├── circle_spectrum.rs  # サークルスペクトラム
│   │   │   ├── particle_system.rs  # パーティクルシステム
│   │   │   ├── spectrum_3d.rs      # 3Dスペクトラム
│   │   │   └── vu_meters.rs        # VUメーター
│   │   └── config.rs          # ビジュアライザー設定
│   │
│   ├── library/               # 音楽ライブラリ管理
│   │   ├── mod.rs
│   │   ├── manager.rs         # ライブラリマネージャー
│   │   ├── scanner.rs         # ファイルスキャン
│   │   ├── metadata.rs        # メタデータ処理
│   │   ├── database.rs        # データベース操作
│   │   └── artwork.rs         # アルバムアートワーク
│   │
│   ├── playlist/              # プレイリスト管理
│   │   ├── mod.rs
│   │   ├── manager.rs         # プレイリストマネージャー
│   │   ├── formats/           # プレイリスト形式
│   │   │   ├── mod.rs
│   │   │   ├── m3u.rs        # M3U形式
│   │   │   ├── pls.rs        # PLS形式
│   │   │   └── json.rs       # JSON形式（独自）
│   │   └── shuffle.rs         # シャッフルアルゴリズム
│   │
│   ├── plugin/                # プラグインシステム
│   │   ├── mod.rs
│   │   ├── manager.rs         # プラグインマネージャー
│   │   ├── loader.rs          # プラグインローダー
│   │   ├── api.rs             # プラグインAPI定義
│   │   └── registry.rs        # プラグインレジストリ
│   │
│   ├── config/                # 設定管理
│   │   ├── mod.rs
│   │   ├── manager.rs         # 設定マネージャー
│   │   ├── schema.rs          # 設定スキーマ定義
│   │   ├── defaults.rs        # デフォルト設定
│   │   └── migration.rs       # 設定マイグレーション
│   │
│   ├── utils/                 # 共通ユーティリティ
│   │   ├── mod.rs
│   │   ├── file.rs            # ファイル操作
│   │   ├── math.rs            # 数学関数
│   │   ├── color.rs           # カラー処理
│   │   ├── animation.rs       # アニメーション
│   │   └── platform.rs        # プラットフォーム固有処理
│   │
│   └── error/                 # エラー処理
│       ├── mod.rs
│       ├── types.rs           # エラー型定義
│       └── handling.rs        # エラーハンドリング
│
├── tests/                     # テスト
│   ├── integration/           # 結合テスト
│   │   ├── audio_engine.rs
│   │   ├── visualizer.rs
│   │   └── playlist.rs
│   ├── unit/                  # 単体テスト
│   └── fixtures/              # テスト用データ
│       ├── audio/
│       └── playlists/
│
├── benches/                   # ベンチマーク
│   ├── audio_processing.rs
│   ├── fft_performance.rs
│   └── visualizer_render.rs
│
├── examples/                  # 使用例
│   ├── basic_player.rs
│   ├── custom_visualizer.rs
│   └── plugin_development.rs
│
├── docs/                      # ドキュメント
│   ├── api/                   # API ドキュメント
│   ├── user_guide/           # ユーザーガイド
│   ├── developer_guide/      # 開発者ガイド
│   └── architecture.md       # アーキテクチャドキュメント
│
├── assets/                    # 静的リソース
│   ├── themes/               # テーマアセット
│   │   ├── dark/
│   │   ├── light/
│   │   └── custom/
│   ├── icons/                # アイコン
│   │   ├── svg/
│   │   └── png/
│   ├── fonts/                # フォント
│   └── sounds/               # システムサウンド
│
├── scripts/                  # ビルドスクリプト
│   ├── build.rs             # カスタムビルド
│   ├── package.sh           # パッケージング
│   └── deploy.sh            # デプロイメント
│
├── target/                   # ビルド出力（.gitignore）
├── Cargo.lock
└── .env                     # 環境変数（.gitignore）
```
