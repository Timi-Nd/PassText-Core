//! Content-free metadata for prepared text.

use crate::NormalizationMode;

/// Immutable measurements for original and processed password text.
///
/// The structure never stores or exposes the password itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextMetadata {
    original_codepoints: usize,
    original_utf8_bytes: usize,
    processed_codepoints: usize,
    processed_utf8_bytes: usize,
    contains_non_ascii: bool,
    normalization: NormalizationMode,
    normalization_changed: bool,
}

impl TextMetadata {
    pub(crate) const fn new(
        original_codepoints: usize,
        original_utf8_bytes: usize,
        processed_codepoints: usize,
        processed_utf8_bytes: usize,
        contains_non_ascii: bool,
        normalization: NormalizationMode,
        normalization_changed: bool,
    ) -> Self {
        Self {
            original_codepoints,
            original_utf8_bytes,
            processed_codepoints,
            processed_utf8_bytes,
            contains_non_ascii,
            normalization,
            normalization_changed,
        }
    }

    /// Number of Unicode scalar values before normalization.
    #[must_use]
    pub const fn original_codepoints(self) -> usize {
        self.original_codepoints
    }

    /// Number of strict UTF-8 bytes before normalization.
    #[must_use]
    pub const fn original_utf8_bytes(self) -> usize {
        self.original_utf8_bytes
    }

    /// Number of Unicode scalar values after normalization.
    #[must_use]
    pub const fn processed_codepoints(self) -> usize {
        self.processed_codepoints
    }

    /// Number of strict UTF-8 bytes after normalization.
    #[must_use]
    pub const fn processed_utf8_bytes(self) -> usize {
        self.processed_utf8_bytes
    }

    /// Whether the original text contains any non-ASCII scalar value.
    #[must_use]
    pub const fn contains_non_ascii(self) -> bool {
        self.contains_non_ascii
    }

    /// The explicitly configured normalization mode.
    #[must_use]
    pub const fn normalization(self) -> NormalizationMode {
        self.normalization
    }

    /// Whether processed text differs exactly from original text.
    #[must_use]
    pub const fn normalization_changed(self) -> bool {
        self.normalization_changed
    }
}
