# AGENTS.md

VOICEVOX/COEIROINK 風 TTS コアの Rust ポート。PyO3 + maturin で Python に公開する（`pyo3` 0.29、Rust edition 2024）。README は存在しない。このファイルがこのリポジトリの唯一のドキュメント。

## ビルドと検証

- 開発ループ: `cargo check`（高速・キャッシュ済み。テストはリポジトリに存在しない）。`cargo test` は現在何も実行しない。
- Python パッケージ: `maturin develop` で cdylib を venv にインストール（先に `pip install maturin` が必要）。クレートは `cdylib` のみ（`Cargo.toml` の `[lib]`）。単体の Rust バイナリは存在しない。
- CI（`.github/workflows/CI.yml`）は maturin が自動生成したもの。手で編集しない。再生成は `maturin generate-ci github` で行う。タグ push で wheel をビルドし、`PYPI_API_TOKEN` を使って PyPI に公開する。

## Python 側の構造（間違えやすい）

- Python API の表面は `src/lib.rs` で組み立てている。3 つの Python サブモジュール（`model`、`query_manager`、`coeiro_manager`）があり、それぞれ固有の `#[pymodule]` 関数を持ち、`src/lib.rs` で `PyModule::new` + `add_submodule` により手動で登録する必要がある。pyclass を追加する場合は、そのモジュールの `#[pymodule]` 関数と（新規モジュールなら）`lib.rs` の両方を更新する。
- `#[pyclass(from_py_object)]` 型は Python オブジェクトを `&Bound<'_, T>` で受け取り、`.extract::<T>()` で Rust 構造体を取り出す（`src/query_manager.rs` の `query2tokens_prosody` 参照）。
- pyclass のフィールド名は Python 側ではキャメルケース、Rust 側ではスネークケースで、`#[pyo3(get, set, name = "...")]` によって exposure される（`src/model.rs`: `speedScale`、`prePhonemeLength` など）。どちらか片方の名前を盲目的にリネームしないこと。
- numpy 配列は `numpy` クレートで往復する（`PyArray1<f64>` をそのまま Python に返す）。

## 実行時 / データレイアウトの落とし穴

- `MetaManager`（`src/coeiro_manager/meta_manager.rs`）は実行時のプロセス CWD 基準で `./speaker_info/**/metas.json` を走査し、各話者ディレクトリ配下にスタイルごとの `model/<style_id>/*.pth` + `config.yaml` を期待する。`MetaManager` を動かす系統のものはすべて、`speaker_info/` を含むディレクトリから実行する必要がある。
- `metas.json` は `serde(rename_all = "camelCase")` でデシリアライズされる（`speaker_info.rs`）。パースに失敗したエントリはエラーではなく stderr 警告でスキップされる。

## 死んでいる / WIP コード

- `src/coeiro_manager/espnet_model.rs` は有効。`espnet-burn`（git: hydriod/espnet-burn、lib 名 `espnet_burn`、rev e7aaf97 v0.2 にピン留め。API はそれを正とする）の `tts::inference::Text2Speech<Burnバックエンド>` を使う。バックエンドは `burn::backend::flex::Flex`（CPU のみ）で、`use_gpu=True` は未対応でエラーを返す（Wgpu/Cuda 対応は未移植）。OpenJTalk の辞書は `OPEN_JTALK_DICT_DIR` または実行ファイル同階に置かれる ものが実行時に初めて使われたときに自動ダウンロードされる（`espnet-burn` の `openjtalk/dict.rs` 参照）。
- `AudioManager` の後処理は Python 元実装（librosa / pyworld / resampy）を近似実装に置き換えている：`trim` はフレームRMS 基準の近似、`pitch_intonation` は `rsworld` クレート（WORLD C ライブラリの cc ビルド）、`resampling` は `rubato` の同期 FFT リサンプラー。
- `src/coeiro_manager/model_config.rs` は空。

## 規約

- コミットメッセージ・識別子近くのコメント・エラー文字列は日本語（ベンダード／派生コード内も同様）。コミットと Python から見えるエラーメッセージでもこれに合わせる。
- pyclass には `#[derive(Debug, Clone)]` を付ける：既存の pyclass はすべて使用している。
