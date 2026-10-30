"""Safe metadata describing processed password text."""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True, slots=True)
class TextMetadata:
    """Immutable metadata for one password evaluation.

    Password contents are intentionally excluded from these records.
    """

    original_codepoints: int
    original_utf8_bytes: int
    processed_codepoints: int
    processed_utf8_bytes: int
    contains_non_ascii: bool
    normalization: str
    normalization_changed: bool

    def as_dict(self) -> dict[str, object]:
        return {
            "original_codepoints": self.original_codepoints,
            "original_utf8_bytes": self.original_utf8_bytes,
            "processed_codepoints": self.processed_codepoints,
            "processed_utf8_bytes": self.processed_utf8_bytes,
            "contains_non_ascii": self.contains_non_ascii,
            "normalization": self.normalization,
            "normalization_changed": self.normalization_changed,
        }
