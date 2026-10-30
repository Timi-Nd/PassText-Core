"""Input validation and process orchestration for PassText."""

from __future__ import annotations

from .config import PassTextConfig
from .exceptions import (
    InvalidInputTypeError,
    InvalidUnicodeError,
    ProcessingFailureError,
    ProcessingLimitError,
)
from .metadata import TextMetadata
from .normalization import apply_normalization
from .representation import PreparedPassword
from .unicode_utils import codepoint_count, contains_non_ascii, validate_utf8_text


def validate_password_input(password: object, config: PassTextConfig) -> tuple[str, int]:
    """Validate the raw password input and return it as a string with its UTF-8 byte count.

    Empty strings are valid inputs; they produce valid metadata without modification.
    """

    if not isinstance(password, str):
        raise InvalidInputTypeError("password must be a str instance")

    if len(password) > config.max_codepoints:
        raise ProcessingLimitError("password exceeds configured maximum Unicode code-point count")

    try:
        encoded = validate_utf8_text(password)
    except UnicodeError:
        raise InvalidUnicodeError("input contains unsupported Unicode") from None

    utf8_size = len(encoded)
    if utf8_size > config.max_utf8_bytes:
        raise ProcessingLimitError("password exceeds configured maximum UTF-8 byte count")

    return password, utf8_size


def build_metadata(
    original: str,
    processed: str,
    config: PassTextConfig,
    *,
    original_utf8_bytes: int | None = None,
    processed_utf8_bytes: int | None = None,
) -> TextMetadata:
    """Create safe metadata without exposing any password content."""

    original_codepoints = codepoint_count(original)
    processed_codepoints = codepoint_count(processed)

    if original_utf8_bytes is None:
        original_utf8_bytes = len(validate_utf8_text(original))
    if processed_utf8_bytes is None:
        processed_utf8_bytes = len(validate_utf8_text(processed))

    return TextMetadata(
        original_codepoints=original_codepoints,
        original_utf8_bytes=original_utf8_bytes,
        processed_codepoints=processed_codepoints,
        processed_utf8_bytes=processed_utf8_bytes,
        contains_non_ascii=contains_non_ascii(original),
        normalization=config.normalization,
        normalization_changed=(processed != original),
    )


def process_password(password: object, config: PassTextConfig | None = None) -> PreparedPassword:
    """Process one password value and return a prepared representation.

    This is the public API entry point for PassText.
    """

    if config is None:
        config = PassTextConfig()

    validated, validated_utf8_bytes = validate_password_input(password, config)
    processed = apply_normalization(validated, config.normalization)

    processed_codepoints = codepoint_count(processed)
    if processed_codepoints > config.max_codepoints:
        raise ProcessingLimitError(
            "processed password exceeds configured maximum Unicode code-point count"
        )

    if processed == validated:
        processed_utf8_bytes = validated_utf8_bytes
    else:
        try:
            processed_utf8_bytes = len(validate_utf8_text(processed))
        except UnicodeError:
            raise ProcessingFailureError("processed password contains invalid Unicode") from None
        if processed_utf8_bytes > config.max_utf8_bytes:
            raise ProcessingLimitError(
                "processed password exceeds configured maximum UTF-8 byte count"
            )

    metadata = build_metadata(
        validated,
        processed,
        config,
        original_utf8_bytes=validated_utf8_bytes,
        processed_utf8_bytes=processed_utf8_bytes,
    )

    from .representation import PreparedPassword

    return PreparedPassword(
        original_text=validated,
        processed_text=processed,
        metadata=metadata,
        config=config,
    )
