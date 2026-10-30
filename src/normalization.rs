//! Explicit Unicode normalization.

use unicode_normalization::{is_nfc, UnicodeNormalization};

use crate::NormalizationMode;

/// Unicode data version embedded in the normalization dependency.
///
/// Cross-language users can compare this with Python's
/// `unicodedata.unidata_version` when exhaustive parity matters.
pub const UNICODE_NORMALIZATION_VERSION: (u8, u8, u8) = unicode_normalization::UNICODE_VERSION;

/// Return normalized owned text only when normalization changes the input.
pub(crate) fn apply_normalization(text: &str, mode: NormalizationMode) -> Option<String> {
    match mode {
        NormalizationMode::None => None,
        NormalizationMode::Nfc if is_nfc(text) => None,
        NormalizationMode::Nfc => {
            let normalized: String = text.nfc().collect();
            (normalized != text).then_some(normalized)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nfc_allocates_only_for_changed_text() {
        assert_eq!(
            apply_normalization("e\u{301}", NormalizationMode::Nfc),
            Some("é".to_owned())
        );
        assert_eq!(apply_normalization("é", NormalizationMode::Nfc), None);
        assert_eq!(
            apply_normalization("e\u{301}", NormalizationMode::None),
            None
        );
    }

    #[test]
    fn nfc_can_expand_a_single_scalar() {
        assert_eq!(
            apply_normalization("\u{344}", NormalizationMode::Nfc),
            Some("\u{308}\u{301}".to_owned())
        );
    }

    #[test]
    fn normalization_data_version_is_pinned_for_python_parity() {
        assert_eq!(UNICODE_NORMALIZATION_VERSION, (16, 0, 0));
    }
}
