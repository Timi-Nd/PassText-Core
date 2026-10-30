"""Safe text indexing contracts for future password analysis modules."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class TextSpan:
    """Half-open span using code-point indexes: [start, end)."""

    start: int
    end: int

    def __post_init__(self) -> None:
        if self.start < 0 or self.end < self.start:
            raise ValueError("invalid text span")


def utf8_offset_for_codepoint(text: str, codepoint_index: int) -> int:
    """Map a code-point index to the corresponding UTF-8 byte offset."""

    if codepoint_index < 0 or codepoint_index > len(text):
        raise ValueError("codepoint index is out of range")
    return len(text[:codepoint_index].encode("utf-8"))


def codepoint_index_for_utf8_offset(text: str, utf8_offset: int) -> int:
    """Map a UTF-8 byte offset to a code-point index.

    The offset must land on a valid UTF-8 code-point boundary; otherwise a ValueError is
    raised to keep the indexing contract explicit and Rust-portable.
    """

    encoded = text.encode("utf-8", errors="strict")
    if utf8_offset < 0 or utf8_offset > len(encoded):
        raise ValueError("UTF-8 offset is out of range")

    try:
        prefix = encoded[:utf8_offset].decode("utf-8", errors="strict")
    except UnicodeDecodeError as exc:
        raise ValueError("UTF-8 offset is not on a valid code-point boundary") from exc

    return len(prefix)


def slice_text(text: str, span: TextSpan) -> str:
    """Safely extract a substring using a half-open code-point range."""

    if span.start < 0 or span.end > len(text):
        raise ValueError("span is outside the source text")
    return text[span.start : span.end]
