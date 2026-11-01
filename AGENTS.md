# AGENTS.md

VOICEVOX/COEIROINK 風 TTS コアの Rust ポート。PyO3 + maturin で Python に公開する（`pyo3` 0.29、Rust edition 2024）。README は存在せず、このファイルが唯一のドキュメント。

## ビルドと検証

- ビルド確認: `cargo check`。Rust 側に `#[test]` は無く、`cargo test` は実行するテストが存在しない（リンク・コンパイル確認には有用）。
- Python テスト: `python -m pytest`（`pyproject.toml` の `testpaths = tests`）。事前に
  1. `maturin develop` で cdylib を pytest を実行するインタプリタにインストール済み
  2. pytest + numpy（`[dependency-groups] dev` は pytest のみ。numpy は入っていないので手動で入れる。無いとテスト収集自体が失敗する）
- 単一テスト: `python -m pytest tests/test_audio_manager.py -k trim` のように `-k` で絞る。
- 移植元 Python 実装 `coeirocore` が import できる環境では query_manager の差分比較テスト 7 件が追加実行され、無い環境では skip する（skip は正常）。
- `maturin develop` は呼んだ maturin のパスではなく、解決先の Python インタプリタ（デフォルトでは conda 等のグローバル環境）にインストールする点に注意。プロジェクト `.venv` に入れるには venv を activate するか `maturin develop -i <.venv の python>` を使う。
- クレートは `cdylib` のみ（`Cargo.toml` の `[lib]`）。単体の Rust バイナリは存在しない。
- `cargo fmt --check` が通る状態を維持する。
- CI（`.github/workflows/CI.yml`）は maturin 自動生成。手で編集しない。再生成は `maturin generate-ci github`。タグ push で全プラットフォームの wheel をビルドし、`PYPI_API_TOKEN` 経由で PyPI に公開する。

## Python API の構造（間違えやすい）

- Python API の表面は `src/lib.rs` で組み立てる。サブモジュール 3 つ（`model`、`query_manager`、`coeiro_manager`）はそれぞれ固有の `#[pymodule]` 関数を持ち、`src/lib.rs` の `register_submodule` で手動登録する。pyclass / pyfunction を追加したら、そのモジュールの `#[pymodule]` 関数を更新する（反映には `maturin develop` の再実行が必要）。
- サブモジュールはフルネーム（`coeirocore_rs.model` など）で `PyModule::new` し、**`sys.modules` 辞書そのもの**に set する必要がある。親への属性追加だけ、や `sys.__dict__` への set では `from coeirocore_rs.model import X` が ModuleNotFoundError になる（`sys.modules` は `sys.__dict__["modules"]` という別辞書）。実装は `src/lib.rs` 参照。
- `#[pyclass(from_py_object)]` 型のメソッドは Python オブジェクトを `&Bound<'_, T>` で受け取り `.extract::<T>()` で取り出す（`src/query_manager.rs` の `query2tokens_prosody` 参照）。
- pyclass のフィールド名は Python 側ではキャメルケース、Rust 側ではスネークケースで、`#[pyo3(get, set, name = "...")]` で exposure される（`src/model.rs`: `speedScale`、`prePhonemeLength` など）。どちらか一方の名前を盲目的にリネームしないこと。Python 側のコンストラクタ呼び出し形は `tests/helpers.py` 参照。
- numpy 配列は `numpy` クレートで往復する（`PyArray1<f64>` をそのまま Python に返す）。

## 実行時 / データレイアウトの落とし穴

- `MetaManager`（`src/coeiro_manager/meta_manager.rs`）は実行時のプロセス CWD 基準で `./speaker_info/**/metas.json` を走査し、各話者ディレクトリ配下にスタイルごとの `model/<style_id>/*.pth` + `config.yaml` を期待する。`MetaManager` を動かす系統はすべて `speaker_info/` を含むディレクトリから実行する必要がある（テストは `monkeypatch.chdir(tmp_path)` で分離している）。
- `metas.json` は `serde(rename_all = "camelCase")` でデシリアライズされる（`speaker_info.rs`）。パースに失敗したエントリはエラーではなく stderr 警告でスキップされる。

## espnet-burn / 後処理（Python 元実装との差異）

- `espnet-burn`（git: hydriod/espnet-burn v0.2、lib 名 `espnet_burn`。API はそちらを正とする）の `tts::inference::Text2Speech` を `burn::backend::flex::Flex`（CPU のみ）で使う。`use_gpu=True` は未対応でエラーを返す（Wgpu/Cuda は未移植）。OpenJTalk の辞書は `OPEN_JTALK_DICT_DIR`（または実行ファイル同階）に無ければ、実行時の初回使用時に自動ダウンロードされる（`espnet-burn` の `openjtalk/dict.rs` 参照）。
- `AudioManager` の後処理は Python 元実装（librosa / pyworld / resampy）を近似実装に置き換えている: `trim` はフレーム RMS 基準の近似、`pitch_intonation` は `rsworld` クレート（WORLD C ライブラリ）、`resampling` は `rubato` の同期 FFT リサンプラー。`tests/test_audio_manager.py` は近似前提の緩い検証になっている。
- synthesis / `EspnetModel` はモデルファイル（`.pth` + `config.yaml`）が必要なためテスト対象外。ローカルにモデルがある環境でのみ手動 E2E。

## 規約

- コミットメッセージ・識別子近くのコメント・Python から見えるエラー文字列は日本語（ベンダード／派生コード内も同様）。
- pyclass には `#[derive(Debug, Clone)]` を付ける（既存の pyclass はすべて使用している）。
