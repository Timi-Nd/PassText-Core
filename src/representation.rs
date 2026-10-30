//! Prepared password ownership and controlled access.

use std::borrow::Cow;
use std::fmt;

use crate::{PassTextConfig, TextMetadata};

/// How processed text is retained without a self-referential borrow.
enum ProcessedText {
    Original,
    Normalized(String),
}

/// Validated password text plus safe metadata.
///
/// Borrowed callers incur no text allocation when normalization is disabled or
/// leaves text unchanged. Owned callers transfer their `String` without a copy.
/// Changed normalized text is stored in one additional `String`.
///
/// This type intentionally does not implement `Clone`, `Display`, or a
/// serialization trait. Its manual `Debug` implementation is metadata-only.
pub struct PreparedPassword<'a> {
    original_text: Cow<'a, str>,
    processed_text: ProcessedText,
    metadata: TextMetadata,
    config: PassTextConfig,
}

impl<'a> PreparedPassword<'a> {
    pub(crate) fn new(
        original_text: Cow<'a, str>,
        normalized_text: Option<String>,
        metadata: TextMetadata,
        config: PassTextConfig,
    ) -> Self {
        let processed_text =
            normalized_text.map_or(ProcessedText::Original, ProcessedText::Normalized);
        Self {
            original_text,
            processed_text,
            metadata,
            config,
        }
    }

    /// Deliberately access the original validated text.
    #[must_use]
    pub fn original_text(&self) -> &str {
        self.original_text.as_ref()
    }

    /// Deliberately access the text after configured normalization.
    #[must_use]
    pub fn processed_text(&self) -> &str {
        match &self.processed_text {
            ProcessedText::Original => self.original_text.as_ref(),
            ProcessedText::Normalized(text) => text,
        }
    }

    /// Access content-free measurements.
    #[must_use]
    pub const fn metadata(&self) -> &TextMetadata {
        &self.metadata
    }

    /// Access the validated configuration used for processing.
    #[must_use]
    pub const fn config(&self) -> &PassTextConfig {
        &self.config
    }
}

impl fmt::Debug for PreparedPassword<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PreparedPassword")
            .field("original_codepoints", &self.metadata.original_codepoints())
            .field(
                "processed_codepoints",
                &self.metadata.processed_codepoints(),
            )
            .field("normalization", &self.metadata.normalization())
            .field(
                "normalization_changed",
                &self.metadata.normalization_changed(),
            )
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use crate::{process_owned_password, process_password, NormalizationMode};

    use super::*;

    #[test]
    fn debug_is_metadata_only() {
        let secret = "SENSITIVE-MARKER";
        let prepared = process_password(secret, PassTextConfig::default()).unwrap();
        let debug = format!("{prepared:?}");
        assert!(!debug.contains(secret));
        assert!(debug.contains("original_codepoints"));
    }

    #[test]
    fn normalized_and_original_text_remain_available_deliberately() {
        let config = PassTextConfig::default().with_normalization(NormalizationMode::Nfc);
        let prepared = process_password("e\u{301}", config).unwrap();
        assert_eq!(prepared.original_text(), "e\u{301}");
        assert_eq!(prepared.processed_text(), "é");
    }

    #[test]
    fn unchanged_borrowed_text_reuses_the_input_storage() {
        let input = "already-normalized-é";
        let config = PassTextConfig::default().with_normalization(NormalizationMode::Nfc);
        let prepared = process_password(input, config).unwrap();

        assert_eq!(prepared.original_text().as_ptr(), input.as_ptr());
        assert_eq!(
            prepared.processed_text().as_ptr(),
            prepared.original_text().as_ptr()
        );
    }

    #[test]
    fn owned_text_moves_without_copying_its_buffer() {
        let input = String::from("owned-password-text");
        let input_pointer = input.as_ptr();
        let prepared = process_owned_password(input, PassTextConfig::default()).unwrap();

        assert_eq!(prepared.original_text().as_ptr(), input_pointer);
        assert_eq!(prepared.processed_text().as_ptr(), input_pointer);
    }

    #[test]
    fn changed_normalization_keeps_original_and_one_distinct_result() {
        let input = "e\u{301}";
        let config = PassTextConfig::default().with_normalization(NormalizationMode::Nfc);
        let prepared = process_password(input, config).unwrap();

        assert_eq!(prepared.original_text().as_ptr(), input.as_ptr());
        assert_ne!(
            prepared.processed_text().as_ptr(),
            prepared.original_text().as_ptr()
        );
        assert!(prepared.metadata().normalization_changed());
    }
}
