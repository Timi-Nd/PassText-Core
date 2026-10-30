import dataclasses
import unicodedata

import pytest
from hypothesis import given
from hypothesis import strategies as st

from passtext import (
    InvalidConfigurationError,
    InvalidInputTypeError,
    InvalidUnicodeError,
    PassTextConfig,
    ProcessingLimitError,
    process_password,
)


def test_empty_password_is_valid():
    result = process_password("", config=PassTextConfig())
    assert result.original_text == ""
    assert result.processed_text == ""
    assert result.metadata.original_codepoints == 0
    assert result.metadata.processed_codepoints == 0
    assert result.metadata.original_utf8_bytes == 0
    assert result.metadata.processed_utf8_bytes == 0


def test_rejects_non_string_input():
    with pytest.raises(InvalidInputTypeError):
        process_password(123, config=PassTextConfig())


def test_rejects_surrogate_code_points():
    with pytest.raises(InvalidUnicodeError):
        process_password("\ud800", config=PassTextConfig())


def test_rejects_input_over_codepoint_limit():
    config = PassTextConfig(max_codepoints=2)
    with pytest.raises(ProcessingLimitError):
        process_password("abc", config=config)


def test_rejects_input_over_utf8_limit():
    config = PassTextConfig(max_utf8_bytes=3)
    with pytest.raises(ProcessingLimitError):
        process_password("éé", config=config)


def test_unicode_codepoint_and_utf8_counts():
    result = process_password("éA🙂", config=PassTextConfig())
    assert result.metadata.original_codepoints == 3
    assert result.metadata.original_utf8_bytes == 7
    assert result.metadata.processed_codepoints == 3
    assert result.metadata.processed_utf8_bytes == 7
    assert result.metadata.contains_non_ascii is True


def test_nfc_normalization_changes_processed_text():
    result = process_password("e\u0301", config=PassTextConfig(normalization="nfc"))
    assert result.original_text == "e\u0301"
    assert result.processed_text == "é"
    assert result.metadata.original_codepoints == 2
    assert result.metadata.processed_codepoints == 1
    assert result.metadata.original_utf8_bytes == 3
    assert result.metadata.processed_utf8_bytes == 2
    assert result.metadata.normalization_changed is True


def test_unicode_error_does_not_expose_secret_data():
    secret = "SECRETMARKER\ud800"
    try:
        process_password(secret, config=PassTextConfig())
    except InvalidUnicodeError as exc:
        payload = str(exc)
        assert "SECRETMARKER" not in payload
        assert "\ud800" not in repr(exc)
        assert exc.__cause__ is None
        assert exc.__context__ is None
    else:
        raise AssertionError("expected InvalidUnicodeError")


def test_bool_values_are_rejected_as_config_integers():
    with pytest.raises(InvalidConfigurationError):
        PassTextConfig(max_codepoints=True)

    with pytest.raises(InvalidConfigurationError):
        PassTextConfig(max_utf8_bytes=False)


def test_invalid_configuration_is_rejected():
    with pytest.raises(InvalidConfigurationError):
        PassTextConfig(max_codepoints=0)

    with pytest.raises(InvalidConfigurationError):
        PassTextConfig(max_utf8_bytes=-1)

    with pytest.raises(InvalidConfigurationError):
        PassTextConfig(normalization="bad")


def test_codepoint_limit_is_checked_before_utf8_validation():
    config = PassTextConfig(max_codepoints=1)
    with pytest.raises(ProcessingLimitError, match="code-point count"):
        process_password("a\ud800", config=config)


def test_prepared_password_is_not_accidentally_dataclass_serializable():
    result = process_password("P@ssw0rd!", config=PassTextConfig())

    with pytest.raises(TypeError):
        dataclasses.asdict(result)


@given(
    st.text(max_size=50).filter(lambda value: not any(0xD800 <= ord(ch) <= 0xDFFF for ch in value))
)
def test_property_metadata_matches_text_invariants(value: str):
    config = PassTextConfig(normalization="nfc")
    result = process_password(value, config=config)

    assert result.metadata.original_codepoints == len(value)
    assert result.metadata.processed_codepoints == len(result.processed_text)
    assert result.metadata.original_utf8_bytes == len(value.encode("utf-8"))
    assert result.metadata.processed_utf8_bytes == len(result.processed_text.encode("utf-8"))
    assert result.metadata.normalization_changed == (result.processed_text != value)
    assert result.metadata.contains_non_ascii == any(ord(ch) > 127 for ch in value)
    assert result.processed_text == unicodedata.normalize("NFC", value)


@given(
    st.text(max_size=50).filter(lambda value: not any(0xD800 <= ord(ch) <= 0xDFFF for ch in value))
)
def test_property_repeated_calls_are_deterministic(value: str):
    config = PassTextConfig(normalization="none")
    first = process_password(value, config=config)
    second = process_password(value, config=config)

    assert first.metadata.as_dict() == second.metadata.as_dict()
    assert first.processed_text == second.processed_text
    assert first.original_text == second.original_text
