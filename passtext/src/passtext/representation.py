"""Prepared password representation for downstream processing."""

from __future__ import annotations

from typing import Any

from .config import PassTextConfig
from .metadata import TextMetadata


class PreparedPassword:
    """Internal representation exposing only safe metadata and controlled access.

    Password contents are intentionally not printed by this object's repr(). The object is not a
    dataclass so ordinary dataclass conversion cannot accidentally serialize the original or
    processed password text.
    """

    __slots__ = ("config", "metadata", "original_text", "processed_text")

    config: PassTextConfig
    metadata: TextMetadata
    original_text: str
    processed_text: str

    def __init__(
        self,
        original_text: str,
        processed_text: str,
        metadata: TextMetadata,
        config: PassTextConfig,
    ):
        object.__setattr__(self, "original_text", original_text)
        object.__setattr__(self, "processed_text", processed_text)
        object.__setattr__(self, "metadata", metadata)
        object.__setattr__(self, "config", config)

    def __setattr__(self, name: str, value: object) -> None:
        raise AttributeError("PreparedPassword is immutable")

    def __repr__(self) -> str:
        return (
            "PreparedPassword("
            f"original_codepoints={self.metadata.original_codepoints}, "
            f"processed_codepoints={self.metadata.processed_codepoints}, "
            f"normalization={self.metadata.normalization!r}, "
            f"normalization_changed={self.metadata.normalization_changed!r})"
        )

    def as_dict(self) -> dict[str, Any]:
        """Return a safe dictionary representation without including the password."""

        return {
            "original_codepoints": self.metadata.original_codepoints,
            "processed_codepoints": self.metadata.processed_codepoints,
            "contains_non_ascii": self.metadata.contains_non_ascii,
            "normalization": self.metadata.normalization,
            "normalization_changed": self.metadata.normalization_changed,
        }

    def __reduce__(self) -> tuple[type[PreparedPassword], tuple[object, ...]]:
        raise TypeError("PreparedPassword is intentionally not serializable")
