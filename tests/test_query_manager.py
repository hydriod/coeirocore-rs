"""coirocore_rs.query_manager.query2tokens_prosody のテスト。

期待値は Python 元実装 (coeirocore.query_manager) の挙動に基づく固定値で、
coeirocore が import 可能な環境では差分一致テストも行う。
"""

import pytest

from coeirocore_rs import query_manager

from helpers import (
    mora_dict,
    phrase_dict,
    pause_mora_dict,
    query_dict,
    rs_query,
    py_modules,
    py_query,
)

# 基本: 子音/母音 + アクセント2、pause_mora あり(1文目なので "_" が出る)
QUERY_ACCENT2_PAUSE = query_dict(
    [
        phrase_dict(
            moras=[mora_dict("k", "o"), mora_dict(None, "o")],
            accent=2,
            pause_mora=pause_mora_dict(),
        ),
        phrase_dict(moras=[mora_dict(None, "o")], accent=1),
    ]
)
EXPECTED_ACCENT2_PAUSE = ["^", "k", "o", "[", "o", "_", "o", "$"]

# 疑問形: 最後に "?" が付く
QUERY_INTERROGATIVE = query_dict(
    [phrase_dict(moras=[mora_dict(None, "o")], accent=1, is_interrogative=True)]
)
EXPECTED_INTERROGATIVE = ["^", "o", "?"]

# アクセント位置 == 最終モーラ: "]" は付かない
QUERY_ACCENT_END = query_dict(
    [phrase_dict(moras=[mora_dict("k", "o"), mora_dict(None, "o")], accent=2)]
)
EXPECTED_ACCENT_END = ["^", "k", "o", "[", "o", "$"]

# pause_mora なしの複数フレーズ: "#" が付く(アクセントフレーズごとに1つ)
QUERY_NO_PAUSE = query_dict(
    [
        phrase_dict(moras=[mora_dict("k", "o"), mora_dict(None, "o")], accent=1),
        phrase_dict(moras=[mora_dict(None, "o")], accent=1),
    ]
)
EXPECTED_NO_PAUSE = ["^", "k", "o", "]", "o", "#", "o", "$"]

# 複数モーラ + pause_mora: "_" はアクセントフレーズごとに「1つ」だけ
QUERY_PAUSE_MULTI_MORA = query_dict(
    [
        phrase_dict(
            moras=[mora_dict("k", "o"), mora_dict(None, "o"), mora_dict(None, "o")],
            accent=2,
            pause_mora=pause_mora_dict(),
        ),
        phrase_dict(moras=[mora_dict(None, "o")], accent=1),
    ]
)
EXPECTED_PAUSE_MULTI_MORA = ["^", "k", "o", "[", "o", "]", "o", "_", "o", "$"]

# 空のアクセントフレーズ一覧: 元実装は IndexError を除いて ['$'] で返す
QUERY_EMPTY = query_dict([])
EXPECTED_EMPTY = ["^", "$"]

# 母音 "N" は小文字化されず、子音は小文字化される(元実装と同じ)
QUERY_VOLTAGE = query_dict(
    [phrase_dict(moras=[mora_dict("N", "N"), mora_dict("K", "O")], accent=1)]
)
EXPECTED_VOLTAGE = ["^", "n", "N", "]", "k", "o", "$"]


ALL_CASES = [
    (QUERY_ACCENT2_PAUSE, EXPECTED_ACCENT2_PAUSE),
    (QUERY_INTERROGATIVE, EXPECTED_INTERROGATIVE),
    (QUERY_ACCENT_END, EXPECTED_ACCENT_END),
    (QUERY_NO_PAUSE, EXPECTED_NO_PAUSE),
    (QUERY_PAUSE_MULTI_MORA, EXPECTED_PAUSE_MULTI_MORA),
    (QUERY_EMPTY, EXPECTED_EMPTY),
    (QUERY_VOLTAGE, EXPECTED_VOLTAGE),
]


@pytest.mark.parametrize("query_dict_,expected", ALL_CASES)
def test_query2tokens_prosody_fixed_values(query_dict_, expected):
    tokens = query_manager.query2tokens_prosody(rs_query(query_dict_))
    assert tokens == expected


def test_pause_token_per_accent_phrase():
    """pause_mora がある複数モーラのフレーズで '_' が各フレーズ1回だけ出ることを統合的に確認。"""
    tokens = query_manager.query2tokens_prosody(rs_query(QUERY_PAUSE_MULTI_MORA))
    assert tokens.count("_") == 1


@pytest.mark.skipif(
    py_modules()[0] is None, reason="coeirocore (移植元) が無い環境ではスキップ"
)
@pytest.mark.parametrize("query_dict_,_", ALL_CASES)
def test_query2tokens_prosody_parity_with_original(query_dict_, _):
    """移植元 Python 実装との差分一致を確認する。"""
    _, conv = py_modules()
    original = conv(py_query(py_modules()[0], query_dict_))
    ported = query_manager.query2tokens_prosody(rs_query(query_dict_))
    assert ported == original
