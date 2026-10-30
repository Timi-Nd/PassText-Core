//! Input validation and processing orchestration.

use std::borrow::Cow;

use crate::normalization::apply_normalization;
use crate::unicode::{
    bounded_codepoint_count, contains_non_ascii, decode_validated_codepoints, validate_codepoints,
};
use crate::{
    LimitUnit, PassTextConfig, PassTextError, PreparedPassword, ProcessingStage, TextMetadata,
};

/// Process borrowed UTF-8 password text with an explicit configuration.
///
/// Rust `str` already guarantees strict UTF-8. Use [`process_utf8`] at a byte
/// decoding boundary or [`process_codepoints`] for integer scalar input.
pub fn process_password(
    password: &str,
    config: PassTextConfig,
) -> Result<PreparedPassword<'_>, PassTextError> {
    process_cow(Cow::Borrowed(password), config)
}

/// Process borrowed text with [`PassTextConfig::default`].
pub fn process_password_with_default(
    password: &str,
) -> Result<PreparedPassword<'_>, PassTextError> {
    process_password(password, PassTextConfig::default())
}

/// Process an owned string without copying the original text.
pub fn process_owned_password(
    password: String,
    config: PassTextConfig,
) -> Result<PreparedPassword<'static>, PassTextError> {
    process_cow(Cow::Owned(password), config)
}

/// Strictly decode and process an external UTF-8 byte representation.
///
/// Malformed UTF-8 is classified as [`PassTextError::InvalidUnicode`] before
/// password processing begins; no lossy conversion is performed.
pub fn process_utf8(
    password: &[u8],
    config: PassTextConfig,
) -> Result<PreparedPassword<'_>, PassTextError> {
    let decoded = std::str::from_utf8(password).map_err(|_| PassTextError::InvalidUnicode)?;
    process_password(decoded, config)
}

/// Validate integer code points and process them as owned text.
///
/// The original code-point limit is checked before scalar validity, matching
/// the Python reference's observable validation order. Surrogate and
/// out-of-range integers are rejected before a `String` is allocated.
pub fn process_codepoints(
    password: &[u32],
    config: PassTextConfig,
) -> Result<PreparedPassword<'static>, PassTextError> {
    if password.len() > config.max_codepoints() {
        return Err(limit_error(
            ProcessingStage::Original,
            LimitUnit::UnicodeCodepoints,
        ));
    }
    // Validate the complete external representation before classifying its
    // byte size. This preserves invalid-Unicode precedence without allocating.
    let measured_utf8_bytes = validate_codepoints(password)?;
    let utf8_bytes =
        measured_utf8_bytes.ok_or(limit_error(ProcessingStage::Original, LimitUnit::Utf8Bytes))?;
    if utf8_bytes > config.max_utf8_bytes() {
        return Err(limit_error(ProcessingStage::Original, LimitUnit::Utf8Bytes));
    }

    let decoded = decode_validated_codepoints(password, utf8_bytes)?;
    process_owned_password(decoded, config)
}

fn process_cow<'a>(
    password: Cow<'a, str>,
    config: PassTextConfig,
) -> Result<PreparedPassword<'a>, PassTextError> {
    let original = password.as_ref();
    let original_codepoints = bounded_codepoint_count(original, config.max_codepoints()).ok_or(
        limit_error(ProcessingStage::Original, LimitUnit::UnicodeCodepoints),
    )?;

    // `str` is valid UTF-8, so its byte length is the strict encoded length.
    let original_utf8_bytes = original.len();
    if original_utf8_bytes > config.max_utf8_bytes() {
        return Err(limit_error(ProcessingStage::Original, LimitUnit::Utf8Bytes));
    }

    let normalized = apply_normalization(original, config.normalization());
    let processed = normalized.as_deref().unwrap_or(original);
    let processed_codepoints = bounded_codepoint_count(processed, config.max_codepoints()).ok_or(
        limit_error(ProcessingStage::Processed, LimitUnit::UnicodeCodepoints),
    )?;

    let processed_utf8_bytes = if normalized.is_some() {
        let size = processed.len();
        if size > config.max_utf8_bytes() {
            return Err(limit_error(
                ProcessingStage::Processed,
                LimitUnit::Utf8Bytes,
            ));
        }
        size
    } else {
        original_utf8_bytes
    };

    let metadata = TextMetadata::new(
        original_codepoints,
        original_utf8_bytes,
        processed_codepoints,
        processed_utf8_bytes,
        contains_non_ascii(original),
        config.normalization(),
        normalized.is_some(),
    );

    Ok(PreparedPassword::new(
        password, normalized, metadata, config,
    ))
}

const fn limit_error(stage: ProcessingStage, unit: LimitUnit) -> PassTextError {
    PassTextError::ProcessingLimit { stage, unit }
}

#[cfg(test)]
mod tests {
    use crate::{ConfigField, NormalizationMode};

    use super::*;

    #[test]
    fn empty_and_mixed_unicode_metadata_are_exact() {
        let empty = process_password_with_default("").unwrap();
        assert_eq!(empty.metadata().original_codepoints(), 0);
        assert_eq!(empty.metadata().original_utf8_bytes(), 0);

        let value = "éA🙂";
        let prepared = process_password_with_default(value).unwrap();
        assert_eq!(prepared.metadata().original_codepoints(), 3);
        assert_eq!(prepared.metadata().original_utf8_bytes(), 7);
        assert!(prepared.metadata().contains_non_ascii());
        assert_eq!(prepared.processed_text(), value);
    }

    #[test]
    fn original_codepoint_limit_precedes_scalar_validation() {
        let config = PassTextConfig::new(32, 1, NormalizationMode::None).unwrap();
        let error = process_codepoints(&[0x41, 0xd800], config).unwrap_err();
        assert_eq!(
            error,
            PassTextError::ProcessingLimit {
                stage: ProcessingStage::Original,
                unit: LimitUnit::UnicodeCodepoints
            }
        );
    }

    #[test]
    fn multibyte_codepoint_boundary_is_exact() {
        let config = PassTextConfig::new(64, 3, NormalizationMode::None).unwrap();
        let accepted = process_password("é🙂a", config).unwrap();
        assert_eq!(accepted.metadata().original_codepoints(), 3);

        assert_eq!(
            process_password("é🙂ab", config).unwrap_err(),
            PassTextError::ProcessingLimit {
                stage: ProcessingStage::Original,
                unit: LimitUnit::UnicodeCodepoints
            }
        );
    }

    #[test]
    fn all_entry_points_preserve_oversize_classification() {
        let config = PassTextConfig::new(64, 2, NormalizationMode::None).unwrap();
        let expected = PassTextError::ProcessingLimit {
            stage: ProcessingStage::Original,
            unit: LimitUnit::UnicodeCodepoints,
        };

        assert_eq!(process_password("abc", config).unwrap_err(), expected);
        assert_eq!(
            process_owned_password("abc".to_owned(), config).unwrap_err(),
            expected
        );
        assert_eq!(process_utf8(b"abc", config).unwrap_err(), expected);
        assert_eq!(
            process_codepoints(&[0x61, 0x62, 0x63], config).unwrap_err(),
            expected
        );
    }

    #[test]
    fn utf8_boundary_remains_strict_before_text_processing() {
        let config = PassTextConfig::new(64, 1, NormalizationMode::None).unwrap();
        let invalid_after_oversize_prefix = [b'a', b'b', 0xff];
        assert_eq!(
            process_utf8(&invalid_after_oversize_prefix, config).unwrap_err(),
            PassTextError::InvalidUnicode
        );
    }

    #[test]
    fn codepoint_input_validates_all_scalars_before_byte_classification() {
        let config = PassTextConfig::new(1, 2, NormalizationMode::None).unwrap();
        assert_eq!(
            process_codepoints(&[0x00e9], config).unwrap_err(),
            PassTextError::ProcessingLimit {
                stage: ProcessingStage::Original,
                unit: LimitUnit::Utf8Bytes
            }
        );
        assert_eq!(
            process_codepoints(&[0x00e9, 0xd800], config).unwrap_err(),
            PassTextError::InvalidUnicode
        );
    }

    #[test]
    fn maximum_limits_do_not_overflow() {
        let config = PassTextConfig::new(usize::MAX, usize::MAX, NormalizationMode::None).unwrap();
        let prepared = process_password("é🙂", config).unwrap();
        assert_eq!(prepared.metadata().original_codepoints(), 2);
    }

    #[test]
    fn malformed_utf8_is_rejected_without_lossy_conversion() {
        let error = process_utf8(&[0x66, 0x80], PassTextConfig::default()).unwrap_err();
        assert_eq!(error, PassTextError::InvalidUnicode);
    }

    #[test]
    fn nfc_post_processing_limits_are_checked() {
        // U+0344 decomposes under NFC into U+0308 U+0301.
        let codepoint_limited = PassTextConfig::new(8, 1, NormalizationMode::Nfc).unwrap();
        assert_eq!(
            process_password("\u{344}", codepoint_limited).unwrap_err(),
            PassTextError::ProcessingLimit {
                stage: ProcessingStage::Processed,
                unit: LimitUnit::UnicodeCodepoints
            }
        );

        let byte_limited = PassTextConfig::new(3, 2, NormalizationMode::Nfc).unwrap();
        assert_eq!(
            process_password("\u{344}", byte_limited).unwrap_err(),
            PassTextError::ProcessingLimit {
                stage: ProcessingStage::Processed,
                unit: LimitUnit::Utf8Bytes
            }
        );
    }

    #[test]
    fn configuration_error_remains_content_free() {
        assert_eq!(
            PassTextConfig::new(0, 1, NormalizationMode::None).unwrap_err(),
            PassTextError::InvalidConfiguration {
                field: ConfigField::MaxUtf8Bytes
            }
        );
    }
}
