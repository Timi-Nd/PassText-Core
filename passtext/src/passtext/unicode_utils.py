"""Unicode and UTF-8 validation helpers."""

from __future__ import annotations

from .exceptions import InvalidUnicodeError


def validate_utf8_text(value: str) -> bytes:
    """Reject values that cannot be represented as legal UTF-8.

    Isolated surrogate code points are rejected before any encoding error is produced so the
    validation path does not retain a password-bearing exception object in the call stack.
    """

    for ch in value:
        codepoint = ord(ch)
        if 0xD800 <= codepoint <= 0xDFFF:
            raise InvalidUnicodeError("input contains unsupported surrogate code points")

    try:
        return value.encode("utf-8", errors="strict")
    except (
        UnicodeEncodeError
    ):  # pragma: no cover - Python strings only fail here for unsupported surrogates
        raise InvalidUnicodeError("input cannot be encoded as valid UTF-8") from None


def codepoint_count(value: str) -> int:
    """Return the number of Unicode code points in the string."""

    return len(value)


def utf8_byte_count(value: str) -> int:
    """Return the number of bytes produced by encoding to UTF-8."""

    return len(validate_utf8_text(value))


def contains_non_ascii(value: str) -> bool:
    """Return whether the string contains any non-ASCII code points."""

    return any(ord(ch) > 127 for ch in value)


def normalize_for_config(value: str, normalization: str) -> str:
    """Compatibility alias for the canonical normalization helper."""

    from .normalization import apply_normalization

    return apply_normalization(value, normalization)
