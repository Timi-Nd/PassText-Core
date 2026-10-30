//! Unicode scalar and strict UTF-8 helpers.

use crate::PassTextError;

/// Count Unicode scalar values, matching the portable code-point contract.
pub(crate) fn codepoint_count(value: &str) -> usize {
    value.chars().count()
}

/// Report whether the value contains any non-ASCII scalar value.
pub(crate) fn contains_non_ascii(value: &str) -> bool {
    !value.is_ascii()
}

/// Decode integer code points only after validating the complete sequence.
///
/// The preflight pass ensures a surrogate or out-of-range value is rejected
/// before a `String` is allocated or partially constructed.
pub(crate) fn decode_codepoints(values: &[u32]) -> Result<String, PassTextError> {
    if values.iter().any(|&value| char::from_u32(value).is_none()) {
        return Err(PassTextError::InvalidUnicode);
    }

    let mut decoded = String::with_capacity(values.len());
    for &value in values {
        // The preflight pass established that every value is a Unicode scalar.
        if let Some(character) = char::from_u32(value) {
            decoded.push(character);
        } else {
            return Err(PassTextError::ProcessingFailure);
        }
    }
    Ok(decoded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_scalars_not_graphemes_or_bytes() {
        assert_eq!(codepoint_count("e\u{301}🙂"), 3);
        assert!(contains_non_ascii("e\u{301}"));
        assert!(!contains_non_ascii("ASCII"));
    }

    #[test]
    fn decoder_rejects_surrogates_and_out_of_range_values() {
        assert_eq!(decode_codepoints(&[0x41, 0x1f642]).unwrap(), "A🙂");
        assert_eq!(
            decode_codepoints(&[0xd800]),
            Err(PassTextError::InvalidUnicode)
        );
        assert_eq!(
            decode_codepoints(&[0x11_0000]),
            Err(PassTextError::InvalidUnicode)
        );
    }
}
