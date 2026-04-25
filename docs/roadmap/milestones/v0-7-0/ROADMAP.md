# v0.7.0 - プレイリスト

## 概要

プレイリスト管理 (作成、編集、保存) を実装し、自動次曲再生・シャッフル・リピートを備えた本格的な音楽プレイヤー体験を実現する。

## ゴール

- プレイリストの作成・編集・保存
- 自動的に次曲へ進む
- シャッフル / リピート (Off / One / All)
- M3U 形式の互換性

## スコープ

### In Scope

- プレイリストデータモデル + 永続化
- キュー管理 (現在の再生順)
- シャッフル / リピート ロジック
- 自動曲送り
- M3U import / export
- Now Playing / Up Next の UI

### Out of Scope

- スマートプレイリスト (条件ベース、v1.x)
- 共有プレイリスト (v1.x 以降)
- 外部サービス連携 (Spotify など, v2.x)

## タスク

### データモデル

- [ ] `Playlist` 構造体 (id, name, tracks: Vec<TrackId>, created_at, modified_at)
- [ ] `Queue` 構造体 (元順序 + シャッフル時の現在順序)
- [ ] DB スキーマ (playlists / playlist_tracks)
- [ ] CRUD API (`PlaylistManager`)

### Player Manager 拡張

- [ ] Auto-advance (現在曲終了 → 次曲)
- [ ] Track-end detection (rodio Sink::sleep_until_end の非同期版)
- [ ] Previous Track (履歴 + シャッフル時の戻り順)
- [ ] Now-playing 状態 (current_track + queue position)

### シャッフル

- [ ] Fisher-Yates シャッフル実装
- [ ] シャッフル On/Off 切替
- [ ] 履歴トラッキング (Previous で戻れる)
- [ ] 「真にランダム」(同じ曲が立て続けに来ないように軽い偏り回避)

### リピート

- [ ] Off / One / All の3モード
- [ ] One: 同じ曲をループ
- [ ] All: プレイリスト末尾で先頭に戻る
- [ ] UI トグルボタン (3状態)

### M3U Import / Export

- [ ] `.m3u` / `.m3u8` パース
- [ ] エクスポート (相対パス推奨)
- [ ] エラー処理 (見つからないファイル等)

### UI

- [ ] Now Playing パネル (現在曲 + アルバムアート placeholder)
- [ ] Up Next パネル (次の3〜5曲をプレビュー)
- [ ] Queue ビュー (全キュー閲覧、ドラッグ並び替え)
- [ ] Playlist Selector (左サイドバー or プルダウン)
- [ ] 「Add to Queue」 / 「Play Next」 メニュー
- [ ] 複数プレイリスト切替

### テスト

- [ ] 100曲のプレイリストで自動曲送り
- [ ] シャッフルが履歴付きで動く
- [ ] リピート One で同じ曲がループ
- [ ] M3U 往復変換でデータ損失なし

## 完了条件

- [ ] プレイリスト作成・保存・読み込み
- [ ] 自動曲送り
- [ ] シャッフルが偏りなく動作
- [ ] M3U 互換 (他プレイヤーで読める)
- [ ] Up Next が見える

## 依存

- v0.6.0 (ライブラリ管理 = 個別トラック管理基盤)

## リスク / 技術的検討事項

- **シャッフルの体験**: 純粋ランダムは「同じ曲が連続」「特定曲が出ない」で UX 悪化することがある。Spotify 式の偏り補正アルゴリズムを参考に
- **Track-end detection**: rodio で正確に「曲終わり」を検知するには sleep_until_end + ε 待ちが必要だが、ブロッキング動作を避ける必要
- **大規模キュー**: 1000曲を超えるキューでの UI 応答性 (仮想スクロール検討)
