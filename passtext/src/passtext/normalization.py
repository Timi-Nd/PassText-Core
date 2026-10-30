"""Normalization helpers for PassText."""

from __future__ import annotations

import unicodedata


def apply_normalization(text: str, mode: str) -> str:
    """Apply a supported normalization mode."""

    if mode == "none":
        return text
    if mode == "nfc":
        return unicodedata.normalize("NFC", text)
    raise ValueError(f"unsupported normalization mode: {mode}")
