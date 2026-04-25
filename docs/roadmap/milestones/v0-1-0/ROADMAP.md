# v0.1.0 - オーディオ基盤の完成

## 概要

ビジュアライザーを乗せる前に、オーディオ再生機能を完成させる。Position/Duration/Seek/メタデータの欠損部分を埋め、全フォーマットの動作を確認する。

## ゴール

- ProgressBar が滑らかに動く
- Duration が正確に表示される
- Seek バーで任意位置に移動できる
- トラックメタデータ (Title/Artist/Album/Year/Genre) が UI に表示される
- MP3 / FLAC / WAV / OGG すべて再生確認済み

## スコープ

### In Scope

- `Player` の position / duration / seek 実装
- メタデータ抽出と UI 統合
- フォーマット別の手動テスト

### Out of Scope

- ビジュアライザー (v0.3.0)
- 複数ファイル管理 / プレイリスト (v0.6.0, v0.7.0)
- メタデータの書き込み (v1.x 以降)

## タスク

### `crates/sonic-core` - Player の機能拡張

- [ ] `Player` に position 追跡を実装 (rodio の `SamplesConverted` ラップ または独自タイマー)
- [ ] `Player` に duration 取得を実装 (symphonia でロード時にプローブ)
- [ ] `Player::seek(Duration)` を実装 (rodio Source の `skip_duration` を活用)
- [ ] `Player` から `AudioFormat` (sample_rate / channels / bit_depth) を返す
- [ ] `PlayerStatus` の TODO (position / duration / format) を実値で埋める

### `crates/sonic-core` - メタデータ統合

- [ ] `metadata::extract_metadata(path)` の高レベル API を作成 (フォーマット自動判定)
- [ ] `Player::play_file` でメタデータを内部に保持
- [ ] `PlayerManager::get_metadata()` API 追加

### `app/src/app` - Controller / Event 拡張

- [ ] `Event::TrackLoaded` に `TrackMetadata` を含める
- [ ] `Event::PlaybackStatus` の position/duration を実値で送信
- [ ] `Command::Seek(f32)` を実装し、Player::seek を呼ぶ
- [ ] `Command::SkipForward(f64)` / `SkipBackward(f64)` を seek ベースで実装

### `app/src/ui` - UI 統合

- [ ] UiState の `track_title` / `artist` / `album` / `year` / `genre` を Event 経由で更新
- [ ] ProgressBar の値とテキスト時刻を実値で更新
- [ ] Seek バーのドラッグで実際にシーク
- [ ] `file_format` / `sample_rate` / `bit_depth` / `bitrate` をクイック表示

### テスト

- [ ] 各フォーマット (MP3, FLAC, WAV, OGG) の再生確認
- [ ] 1分以上のトラックで position が正確に進む
- [ ] Seek 後も正確に再生される
- [ ] 壊れたファイルでもクラッシュしない

## 完了条件

- [ ] 4フォーマットすべて再生確認
- [ ] ProgressBar が滑らかに動く (スタッターなし)
- [ ] Seek バーで任意位置に飛べる
- [ ] メタデータが UI に表示される
- [ ] エラー時 (壊れたファイル等) もアプリが応答する

## 依存

なし (現状の足場の上に直接実装)

## リスク / 技術的検討事項

- rodio から position を正確に取得する方法は版により異なる。`Source::total_duration` は前向きで、再生中の現在位置は別途追跡が必要
- seek の実装には rodio の Source を再構築する必要があるため、ファイル再読み込みが発生する可能性
