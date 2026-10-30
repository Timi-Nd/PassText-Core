import pytest

from passtext import PassTextConfig, process_password


@pytest.mark.parametrize(
    ("value", "expected_codepoints", "expected_utf8"),
    [
        ("ASCII", 5, 5),
        ("é", 1, 2),
        ("Γειά", 4, 8),
        ("Привет", 6, 12),
        ("مرحبا", 5, 10),
        ("Hello世界🙂", 8, 15),
        ("e\u0301", 2, 3),
    ],
)
def test_unicode_counts_for_common_password_inputs(value, expected_codepoints, expected_utf8):
    result = process_password(value, config=PassTextConfig())
    assert result.metadata.original_codepoints == expected_codepoints
    assert result.metadata.original_utf8_bytes == expected_utf8
    assert result.metadata.processed_codepoints == expected_codepoints
    assert result.metadata.processed_utf8_bytes == expected_utf8


def test_normalization_is_explicit_and_deterministic():
    config = PassTextConfig(normalization="nfc")
    first = process_password("e\u0301", config=config)
    second = process_password("é", config=config)
    assert first.processed_text == "é"
    assert second.processed_text == "é"
    assert first.metadata.normalization_changed is True
    assert second.metadata.normalization_changed is False


def test_mixed_script_input_is_preserved_without_trimming():
    value = "Aß🙂123"
    result = process_password(value, config=PassTextConfig())
    assert result.original_text == value
    assert result.processed_text == value
    assert result.metadata.contains_non_ascii is True
