"""coirocore_rs.coeiro_manager.MetaManager のテスト。

CWD 基準の ./speaker_info/** を走査するため、tmp_path を作って chdir して検証する。
"""

import json
from pathlib import Path

import pytest

from coeirocore_rs import coeiro_manager as cm


def make_metas(speaker_uuid, styles):
    """styles は [(style_id, style_name)] のリスト。"""
    return {
        "speakerName": "話者A",
        "speakerUuid": speaker_uuid,
        "styles": [
            {"styleId": style_id, "styleName": style_name} for style_id, style_name in styles
        ],
        "version": "0.14.0",
    }


def write_speaker(root, dirname, metas, styles_with_models):
    """話者ディレクトリを作り metas.json とダミーモデルを配置する。

    styles_with_models は [(style_id, pth_filename)]。
    """
    speaker_dir = root / "speaker_info" / dirname
    (speaker_dir / "model").mkdir(parents=True)
    (speaker_dir / "metas.json").write_text(
        json.dumps(metas, ensure_ascii=False), encoding="utf-8"
    )
    for style_id, pth_name in styles_with_models:
        style_dir = speaker_dir / "model" / str(style_id)
        style_dir.mkdir(parents=True, exist_ok=True)
        (style_dir / pth_name).write_bytes(b"dummy")
    return speaker_dir


@pytest.fixture()
def speaker_root(tmp_path, monkeypatch):
    monkeypatch.chdir(tmp_path)
    return tmp_path


def test_meta_manager_parses_styles_and_models(speaker_root):
    a = write_speaker(
        speaker_root,
        "tsukuyomi",
        make_metas("uuid-1", [(3, "つくよみ"), (7, "あまあま")]),
        [(3, "a.pth"), (7, "b.pth")],
    )
    b = write_speaker(
        speaker_root, "mana", make_metas("uuid-2", [(1, "マナ")]), [(1, "c.pth")]
    )

    mm = cm.MetaManager()
    metas = mm.get_metas_dict()

    # 最初のスタイルIDでソートされる(元実装と同じ挙動)
    assert [m["speaker_uuid"] for m in metas] == ["uuid-2", "uuid-1"]
    mana, tsuku = metas
    assert mana["name"] == "話者A"
    assert mana["version"] == "0.14.0"
    assert mana["styles"] == [{"id": 1, "name": "マナ"}]
    assert tsuku["styles"] == [
        {"id": 3, "name": "つくよみ"},
        {"id": 7, "name": "あまあま"},
    ]

    # id_model_map 経由でモデルパスが取れる(パスはCWD相対)
    mp = mm.id_model_map[3]
    assert (speaker_root / mp.model_path) == (a / "model" / "3" / "a.pth")
    assert (speaker_root / mp.config_path) == (a / "model" / "3" / "config.yaml")
    assert (speaker_root / mm.id_model_map[1].model_path) == (b / "model" / "1" / "c.pth")


def test_meta_manager_invalid_json_is_skipped(speaker_root, capfd):
    write_speaker(speaker_root, "good", make_metas("uuid-1", [(1, "ノーマル")]), [(1, "a.pth")])
    broken = speaker_root / "speaker_info" / "broken"
    broken.mkdir(parents=True)
    (broken / "metas.json").write_text("{not json", encoding="utf-8")

    mm = cm.MetaManager()
    assert len(mm.get_metas_dict()) == 1
    captured = capfd.readouterr()
    assert "Failed to parse metas.json" in captured.err


def test_meta_manager_duplicate_uuid_is_skipped(speaker_root, capfd):
    """移植元は重複 uuid で例外を投げるが、Rust 版は警告してスキップする(現行挙動を固定)。"""
    write_speaker(speaker_root, "a", make_metas("uuid-same", [(1, "A")]), [(1, "a.pth")])
    write_speaker(speaker_root, "b", make_metas("uuid-same", [(2, "B")]), [(2, "b.pth")])

    mm = cm.MetaManager()
    assert len(mm.get_metas_dict()) == 1
    captured = capfd.readouterr()
    assert "重複" in captured.err


def test_meta_manager_duplicate_style_id_warns(speaker_root, capfd):
    write_speaker(speaker_root, "a", make_metas("uuid-1", [(1, "A")]), [(1, "a.pth")])
    write_speaker(speaker_root, "b", make_metas("uuid-2", [(1, "B")]), [(1, "b.pth")])

    cm.MetaManager()
    captured = capfd.readouterr()
    assert "duplicated" in captured.err


def test_meta_manager_missing_metas_json_raises(speaker_root):
    speaker_root.joinpath("speaker_info", "empty").mkdir(parents=True)
    with pytest.raises(OSError):
        cm.MetaManager()


def test_meta_manager_missing_model_file_skips_style(speaker_root, capfd):
    """.pth が無いスタイルは警告を出し id_model_map には登録されない(元実装は例外)。"""
    write_speaker(speaker_root, "a", make_metas("uuid-1", [(1, "A")]), [])
    mm = cm.MetaManager()
    captured = capfd.readouterr()
    assert "No model file found for style id: 1" in captured.err
    assert 1 not in mm.id_model_map


def test_meta_manager_version_default(speaker_root):
    metas = make_metas("uuid-1", [(1, "A")])
    metas.pop("version")
    write_speaker(speaker_root, "a", metas, [(1, "a.pth")])
    mm = cm.MetaManager()
    assert mm.get_metas_dict()[0]["version"] == "0.0.1"


def test_first_style_id_fallback(speaker_root):
    mm = cm.MetaManager()
    assert mm.get_metas_dict() == []


def test_audio_model_map_default_style_id(speaker_root):
    write_speaker(speaker_root, "a", make_metas("uuid-1", [(0, "A")]), [(0, "a.pth")])
    mm = cm.MetaManager()
    assert mm.default_style_id() == 0
