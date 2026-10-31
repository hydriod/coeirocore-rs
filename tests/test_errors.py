"""エラー系のテスト: EspnetModel の use_gpu と、存在しないモデルパス、CWD に speaker_info が無い AudioManager。"""

from pathlib import Path

import pytest

import coeirocore_rs as _rs

AudioManager, EspnetModel = (
    _rs.coeiro_manager.AudioManager,
    _rs.coeiro_manager.EspnetModel,
)



def test_espnet_model_use_gpu_error():
    with pytest.raises(RuntimeError, match="use_gpu=True"):
        EspnetModel("nonexistent.yaml", "nonexistent.pth", 1.0, True)


def test_espnet_model_missing_files_raises_runtime_error():
    root = Path(__file__).parent
    with pytest.raises(RuntimeError):
        EspnetModel(
            str(root / "nonexistent_config.yaml"),
            str(root / "nonexistent_model.pth"),
            1.0,
            False,
        )


def test_audio_manager_without_speaker_info(tmp_path, monkeypatch):
    monkeypatch.chdir(tmp_path)
    with pytest.raises(KeyError, match="style_id"):
        AudioManager(44100, False)
