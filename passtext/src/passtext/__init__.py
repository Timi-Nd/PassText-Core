"""PassText public API."""

from .config import PassTextConfig
from .exceptions import (
    InvalidConfigurationError,
    InvalidInputTypeError,
    InvalidUnicodeError,
    PassTextError,
    ProcessingFailureError,
    ProcessingLimitError,
)
from .indexing import (
    TextSpan,
    codepoint_index_for_utf8_offset,
    slice_text,
    utf8_offset_for_codepoint,
)
from .metadata import TextMetadata
from .representation import PreparedPassword
from .validation import process_password

__all__ = [
    "InvalidConfigurationError",
    "InvalidInputTypeError",
    "InvalidUnicodeError",
    "PassTextConfig",
    "PassTextError",
    "PreparedPassword",
    "ProcessingFailureError",
    "ProcessingLimitError",
    "TextMetadata",
    "TextSpan",
    "codepoint_index_for_utf8_offset",
    "process_password",
    "slice_text",
    "utf8_offset_for_codepoint",
]
