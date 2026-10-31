"""テスト共通ヘルパー: dict から coeirocore_rs / coeirocore(移植元) の AudioQuery を構築する。"""

import coeirocore_rs as _rs

rs_model = _rs.model


def mora_dict(consonant, vowel="a", pitch=5.0):
    return {
        "text": (consonant or "") + vowel,
        "consonant": consonant,
        "consonant_length": 0.1 if consonant else None,
        "vowel": vowel,
        "vowel_length": 0.2,
        "pitch": pitch,
    }


def phrase_dict(moras, accent=1, pause_mora=None, is_interrogative=False):
    return {
        "moras": moras,
        "accent": accent,
        "pause_mora": pause_mora,
        "is_interrogative": is_interrogative,
    }


def pause_mora_dict():
    return mora_dict(None, vowel="pau")


def query_dict(accent_phrases, **overrides):
    d = {
        "accent_phrases": accent_phrases,
        "speedScale": 1.0,
        "pitchScale": 0.0,
        "intonationScale": 1.0,
        "volumeScale": 1.0,
        "prePhonemeLength": 0.0,
        "postPhonemeLength": 0.0,
        "outputSamplingRate": 44100,
        "outputStereo": False,
        "kana": None,
    }
    d.update(overrides)
    return d


# --- coeirocore_rs (Rust) 側の構築 ---------------------------------------------


def rs_mora(d):
    return rs_model.Mora(
        d["text"],
        d.get("consonant"),
        d.get("consonant_length"),
        d["vowel"],
        d["vowel_length"],
        d["pitch"],
    )


def rs_phrase(d):
    return rs_model.AccentPhrase(
        [rs_mora(m) for m in d["moras"]],
        d["accent"],
        rs_mora(d["pause_mora"]) if d.get("pause_mora") else None,
        d["is_interrogative"],
    )


def rs_query(d):
    return rs_model.AudioQuery(
        [rs_phrase(p) for p in d["accent_phrases"]],
        d["speedScale"],
        d["pitchScale"],
        d["intonationScale"],
        d["volumeScale"],
        d["prePhonemeLength"],
        d["postPhonemeLength"],
        d["outputSamplingRate"],
        d["outputStereo"],
        d.get("kana"),
    )


# --- coeirocore (Python移植元) 側の構築（差分比較用、無ければ None） -------------


def py_modules():
    try:
        from coeirocore import model as py_model
        from coeirocore.query_manager import query2tokens_prosody as conv
    except ImportError:
        return None, None
    return py_model, conv


def py_mora(mod, d):
    return mod.Mora(**d)


def py_phrase(mod, d):
    return mod.AccentPhrase(
        moras=[py_mora(mod, m) for m in d["moras"]],
        accent=d["accent"],
        pause_mora=py_mora(mod, d["pause_mora"]) if d.get("pause_mora") else None,
        is_interrogative=d["is_interrogative"],
    )


def py_query(mod, d):
    return mod.AudioQuery(
        accent_phrases=[py_phrase(mod, p) for p in d["accent_phrases"]],
        speedScale=d["speedScale"],
        pitchScale=d["pitchScale"],
        intonationScale=d["intonationScale"],
        volumeScale=d["volumeScale"],
        prePhonemeLength=d["prePhonemeLength"],
        postPhonemeLength=d["postPhonemeLength"],
        outputSamplingRate=d["outputSamplingRate"],
        outputStereo=d["outputStereo"],
        kana=d.get("kana"),
    )
