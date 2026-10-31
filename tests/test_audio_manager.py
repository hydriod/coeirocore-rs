"""coirocore_rs.coeiro_manager.AudioManager の静的 後処理 関数 (trim/volume/pitch_intonation/sil/resampling) のテスト。

モデルファイルが必要な synthesis / EspnetModel は対象外 (ローカルにモデルがある環境のみ E2E)。
"""

import numpy as np
import pytest

from coeirocore_rs import coeiro_manager as cm

AudioManager = cm.AudioManager


def sine_wave(freq, fs, seconds, amp=0.5):
    n = int(fs * seconds)
    t = np.arange(n) / fs
    return amp * np.sin(2.0 * np.pi * freq * t).astype(np.float32)


def sine_wave_list(freq, fs, seconds, amp=0.5):
    return sine_wave(freq, fs, seconds, amp).tolist()


def pulse_wave_list(freq, fs, seconds, amp=0.5):
    """WORLDのf0解析が動作するような、倍音を豊富に含む音(矩形波)を作る。

    純粋なサイン波は WORLD (harvest) が有声音として扱えないため使えない。
    """
    n = int(fs * seconds)
    t = np.arange(n) / fs
    return (amp * np.sign(np.sin(2.0 * np.pi * freq * t))).astype(np.float32).tolist()


def autocorr_dominant_lag(x, max_lag=480):
    n = len(x) // 2
    seg = np.ascontiguousarray(x[:n])
    ac = np.correlate(seg, seg, "full")[n - 1 : n - 1 + max_lag]
    return int(np.argmax(ac[5:]) + 5)


def dominant_freq(x, fs):
    spec = np.abs(np.fft.rfft(x))
    freqs = np.fft.rfftfreq(len(x), 1.0 / fs)
    return freqs[int(np.argmax(spec))]


# --- trim ------------------------------------------------------------------


def test_trim_removes_silence_from_both_ends():
    fs = 24000
    wav = np.concatenate(
        [np.zeros(10240, np.float32), sine_wave(600, fs, 10240 / fs), np.zeros(10240, np.float32)]
    )
    out = np.asarray(AudioManager.trim(wav.tolist()), dtype=np.float32)
    assert 0 < len(out) < len(wav)
    # 有音部分は含まれている
    assert np.max(np.abs(out)) == pytest.approx(0.5, abs=0.05)
    # 先頭の無音が詰められている(フレームRMS 近似なので数フレームの誤差を許容)
    first_net = int(np.argmax(np.abs(out) > 0.01))
    assert first_net <= 2048
    # 末尾の無音も詰められている
    last_net = int(len(out) - 1 - np.argmax(np.abs(out[::-1]) > 0.01))
    assert (len(out) - 1) - last_net <= 2048


def test_trim_all_silent_keeps_full_length():
    """librosa と同様、全フレームがしきい値相対で下回っていなければ全長を保持する。"""
    out = AudioManager.trim([0.0] * 22050)
    assert isinstance(out, list)
    assert len(out) == 22050


def test_trim_empty_input():
    assert AudioManager.trim([]) == []


def test_trim_no_silence_keeps_waveform():
    wav = sine_wave_list(440, 24000, 1.0)
    out = np.asarray(AudioManager.trim(wav), dtype=np.float32)
    assert len(out) == len(wav)  # 全フレームがしきい値上 → 全長維持


# --- volume ----------------------------------------------------------------


def test_volume_scales_amplitude():
    out = AudioManager.volume([1.0, -2.0, 0.5], 0.5)
    assert out == pytest.approx([0.5, -1.0, 0.25])


def test_volume_empty_input():
    assert AudioManager.volume([], 0.5) == []


# --- sil -------------------------------------------------------------------


def test_sil_inserts_leading_and_trailing_silence():
    out = AudioManager.sil([1.0, 2.0, 3.0], 16000, 0.5, 0.25)
    arr = np.asarray(out, dtype=np.float32)
    assert len(arr) == 8000 + 3 + 4000
    assert arr[8000] == 1.0
    assert list(arr[:8000]) == [0.0] * 8000
    assert list(arr[8003:]) == [0.0] * 4000


def test_sil_negative_length_no_extra():
    out = AudioManager.sil([1.0, 2.0], 16000, -1.0, -0.5)
    assert out == [1.0, 2.0]


def test_sil_empty_input():
    out = AudioManager.sil([], 16000, 0.1, 0.2)
    assert len(out) == int(16000 * 0.1) + int(16000 * 0.2)


# --- resampling ------------------------------------------------------------


def test_resampling_changes_length_by_ratio():
    fs, out_fs = 44100, 24000
    wav = sine_wave_list(440, fs, 1.0)
    out = np.asarray(AudioManager.resampling(wav, fs, out_fs), dtype=np.float32)
    # 長さは比率どおり(リサンプラーの遅延トリミング分の許容あり)
    expected = len(wav) * out_fs / fs
    assert abs(len(out) - expected) <= 1024
    # 周波数成分が保たれている
    assert dominant_freq(out, out_fs) == pytest.approx(440, abs=10)


def test_resampling_empty_input():
    assert AudioManager.resampling([], 44100, 24000) == []


def test_resampling_same_rate_returns_same_length():
    wav = sine_wave_list(440, 44100, 0.1)
    out = np.asarray(AudioManager.resampling(wav, 44100, 44100), dtype=np.float32)
    assert abs(len(out) - len(wav)) <= 1024
    assert np.abs(np.max(np.abs(out)) - 0.5) < 0.05


# --- pitch_intonation ------------------------------------------------------


def test_pitch_intonation_pitch_down_lower_frequency():
    """pitch_scale=-1 で f0 が半分: 自己相分の優勢周期が2倍になる。"""
    fs = 24000
    wav = np.asarray(pulse_wave_list(220, fs, 0.5), dtype=np.float32)
    base = np.asarray(AudioManager.pitch_intonation(wav.tolist(), fs, 0.0, 1.0), dtype=np.float32)
    down = np.asarray(AudioManager.pitch_intonation(wav.tolist(), fs, -1.0, 1.0), dtype=np.float32)
    base_lag = autocorr_dominant_lag(base)
    down_lag = autocorr_dominant_lag(down)
    # fs/220 ≈ 109 が fs/110 ≈ 218 に変わる
    assert base_lag == pytest.approx(fs / 220, abs=8)
    assert down_lag == pytest.approx(fs / 110, abs=8)
    # pitch_scale=+1 も波形を変化させる
    up = np.asarray(AudioManager.pitch_intonation(wav.tolist(), fs, 1.0, 1.0), dtype=np.float32)
    assert np.abs(base - up).max() > 0.1
    assert np.all(np.isfinite(up))


def test_pitch_intonation_intonation_scale_stretches():
    fs = 24000
    wav = np.asarray(pulse_wave_list(220, fs, 0.6), dtype=np.float32)
    base = np.asarray(AudioManager.pitch_intonation(wav.tolist(), fs, 0.0, 1.0), dtype=np.float32)
    stretched = np.asarray(
        AudioManager.pitch_intonation(wav.tolist(), fs, 0.0, 1.5), dtype=np.float32
    )
    assert len(stretched) == len(base)
    assert np.abs(base - stretched).max() > 0.1
    assert np.all(np.isfinite(stretched))


def test_pitch_intonation_empty_input():
    assert AudioManager.pitch_intonation([], 24000, 1.0, 1.0) == []
