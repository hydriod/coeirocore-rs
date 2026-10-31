"""coirocore_rs.model の pyclass (Mora/AccentPhrase/AudioQuery) のテスト。

Python 側から構築でき、camelCase / snake_case の attribute 名が想定どおり出ていること。
"""

from coeirocore_rs import model as rs

from helpers import mora_dict, phrase_dict, pause_mora_dict, query_dict, rs_query


def test_mora_construction_and_fields():
    m = rs.Mora("コ", "k", 0.1, "オ", 0.2, 5.0)
    assert m.text == "コ"
    assert m.consonant == "k"
    assert m.consonant_length == 0.1
    assert m.vowel == "オ"
    assert m.vowel_length == 0.2
    assert m.pitch == 5.0

    m.consonant = None
    m.pitch = 6.5
    assert m.consonant is None
    assert m.pitch == 6.5


def test_accent_phrase_construction_and_fields():
    m = rs.Mora("コ", "k", 0.1, "オ", 0.2, 5.0)
    p = rs.AccentPhrase([m], 2, m, True)
    assert len(p.moras) == 1
    assert p.moras[0].text == m.text
    assert p.moras[0].pitch == m.pitch
    assert p.accent == 2
    assert p.pause_mora.text == m.text
    assert p.is_interrogative is True

    p.is_interrogative = False
    assert p.is_interrogative is False


def test_audio_query_construction_and_camelcase_fields():
    m = rs.Mora("コ", "k", 0.1, "オ", 0.2, 5.0)
    p = rs.AccentPhrase([m], 2, None, False)
    q = rs.AudioQuery([p], 1.5, 0.1, 0.2, 0.3, 0.4, 0.5, 24000, False, kana="テスト")
    # camelCase名で見えていること
    assert len(q.accentPhrases) == 1
    assert q.accentPhrases[0].accent == 2
    assert q.accentPhrases[0].moras[0].text == "コ"
    assert q.speedScale == 1.5
    assert q.pitchScale == 0.1
    assert q.intonationScale == 0.2
    assert q.volumeScale == 0.3
    assert q.prePhonemeLength == 0.4
    assert q.postPhonemeLength == 0.5
    assert q.outputSamplingRate == 24000
    assert q.outputStereo is False
    assert q.kana == "テスト"

    q.outputSamplingRate = 48000
    assert q.outputSamplingRate == 48000


def test_audio_query_kana_default_none():
    m = rs.Mora("コ", None, None, "オ", 0.2, 5.0)
    p = rs.AccentPhrase([m], 1, None, False)
    q = rs.AudioQuery([p], 1.0, 0.0, 1.0, 1.0, 0.0, 0.0, 44100, False)
    assert q.kana is None


def test_query_roundtrip_via_dict():
    d = query_dict(
        [
            phrase_dict(
                moras=[mora_dict("k", "o"), mora_dict(None, "o")],
                accent=2,
                pause_mora=pause_mora_dict(),
            )
        ]
    )
    q = rs_query(d)
    assert len(q.accentPhrases) == 1
    assert q.accentPhrases[0].moras[0].consonant == "k"
    assert q.accentPhrases[0].moras[1].consonant is None
    assert q.accentPhrases[0].pause_mora.vowel == "pau"
