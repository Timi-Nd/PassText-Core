import pytest
from hypothesis import assume, given
from hypothesis import strategies as st

from passtext import PassTextConfig, process_password
from passtext.indexing import (
    TextSpan,
    codepoint_index_for_utf8_offset,
    slice_text,
    utf8_offset_for_codepoint,
)


def test_metadata_as_dict_excludes_secret_contents():
    result = process_password("éA🙂", config=PassTextConfig())
    payload = result.metadata.as_dict()

    assert payload["original_codepoints"] == 3
    assert payload["processed_utf8_bytes"] == 7
    assert "éA🙂" not in str(payload)
    assert "🙂" not in str(payload)


def test_indexing_round_trip_for_utf8_and_codepoint_offsets():
    text = "éA🙂"

    assert utf8_offset_for_codepoint(text, 0) == 0
    assert utf8_offset_for_codepoint(text, 1) == 2
    assert utf8_offset_for_codepoint(text, 2) == 3
    assert utf8_offset_for_codepoint(text, 3) == 7

    assert codepoint_index_for_utf8_offset(text, 0) == 0
    assert codepoint_index_for_utf8_offset(text, 2) == 1
    assert codepoint_index_for_utf8_offset(text, 3) == 2
    assert codepoint_index_for_utf8_offset(text, 7) == 3

    assert slice_text(text, TextSpan(1, 3)) == "A🙂"


def test_indexing_rejects_mid_codepoint_utf8_offsets():
    with pytest.raises(ValueError, match="UTF-8 offset"):
        codepoint_index_for_utf8_offset("éA", 1)


UNICODE_TEXT = st.text(
    alphabet=st.characters(blacklist_categories=("Cs",)),
    max_size=40,
)


@given(UNICODE_TEXT)
def test_property_codepoint_and_utf8_boundaries_round_trip(text: str):
    encoded = text.encode("utf-8")
    for codepoint_index in range(len(text) + 1):
        byte_offset = utf8_offset_for_codepoint(text, codepoint_index)
        assert codepoint_index_for_utf8_offset(text, byte_offset) == codepoint_index

    valid_offsets = {
        utf8_offset_for_codepoint(text, codepoint_index) for codepoint_index in range(len(text) + 1)
    }
    for byte_offset in range(len(encoded) + 1):
        if byte_offset in valid_offsets:
            assert codepoint_index_for_utf8_offset(text, byte_offset) == len(
                encoded[:byte_offset].decode("utf-8")
            )
        else:
            with pytest.raises(ValueError, match="UTF-8 offset"):
                codepoint_index_for_utf8_offset(text, byte_offset)


@given(UNICODE_TEXT, st.integers(min_value=0, max_value=40), st.integers(min_value=0, max_value=40))
def test_property_safe_slicing_matches_half_open_codepoint_ranges(
    text: str, first: int, second: int
):
    assume(first <= second <= len(text))
    span = TextSpan(first, second)
    assert slice_text(text, span) == text[first:second]


@given(UNICODE_TEXT)
def test_property_invalid_offsets_and_spans_are_rejected(text: str):
    with pytest.raises(ValueError, match="codepoint index"):
        utf8_offset_for_codepoint(text, -1)
    with pytest.raises(ValueError, match="codepoint index"):
        utf8_offset_for_codepoint(text, len(text) + 1)
    with pytest.raises(ValueError, match="UTF-8 offset"):
        codepoint_index_for_utf8_offset(text, -1)
    with pytest.raises(ValueError, match="UTF-8 offset"):
        codepoint_index_for_utf8_offset(text, len(text.encode("utf-8")) + 1)
    with pytest.raises(ValueError, match="span"):
        TextSpan(1, 0)
    with pytest.raises(ValueError, match="span"):
        slice_text(text, TextSpan(len(text), len(text) + 1))


@given(UNICODE_TEXT)
def test_property_empty_and_full_spans(text: str):
    assert slice_text(text, TextSpan(0, 0)) == ""
    assert slice_text(text, TextSpan(0, len(text))) == text
