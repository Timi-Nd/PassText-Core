//! Unicode scalar and strict UTF-8 helpers.

use crate::PassTextError;

/// Count Unicode scalar values up to an inclusive configured maximum.
///
/// `None` means the input contains more than `limit` scalar values. The
/// traversal consumes at most `limit + 1` values without computing that sum,
/// so `usize::MAX` cannot overflow.
pub(crate) fn bounded_codepoint_count(value: &str, limit: usize) -> Option<usize> {
    bounded_count(value.chars(), limit)
}

fn bounded_count(values: impl IntoIterator, limit: usize) -> Option<usize> {
    let mut count = 0;
    for _ in values {
        if count == limit {
            return None;
        }
        count += 1;
    }
    Some(count)
}

/// Report whether the value contains any non-ASCII scalar value.
pub(crate) fn contains_non_ascii(value: &str) -> bool {
    !value.is_ascii()
}

/// Validate integer code points and measure their UTF-8 representation.
///
/// A `None` byte count means the theoretical sum overflowed `usize`. Scalar
/// validation continues after that point so invalid-Unicode precedence is not
/// changed by an oversized representation.
pub(crate) fn validate_codepoints(values: &[u32]) -> Result<Option<usize>, PassTextError> {
    let mut utf8_bytes = Some(0usize);
    for &value in values {
        let character = char::from_u32(value).ok_or(PassTextError::InvalidUnicode)?;
        if let Some(current) = utf8_bytes {
            utf8_bytes = current.checked_add(character.len_utf8());
        }
    }
    Ok(utf8_bytes)
}

/// Decode a sequence already checked by [`validate_codepoints`].
pub(crate) fn decode_validated_codepoints(
    values: &[u32],
    utf8_bytes: usize,
) -> Result<String, PassTextError> {
    let mut decoded = String::with_capacity(utf8_bytes);
    for &value in values {
        let character = char::from_u32(value).ok_or(PassTextError::ProcessingFailure)?;
        decoded.push(character);
    }
    Ok(decoded)
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    #[test]
    fn counts_scalars_not_graphemes_or_bytes() {
        assert_eq!(bounded_codepoint_count("e\u{301}🙂", 3), Some(3));
        assert_eq!(bounded_codepoint_count("e\u{301}🙂", 2), None);
        assert!(contains_non_ascii("e\u{301}"));
        assert!(!contains_non_ascii("ASCII"));
    }

    #[test]
    fn bounded_count_stops_after_the_first_excess_value() {
        let visited = Cell::new(0);
        let values = (0..100).inspect(|_| visited.set(visited.get() + 1));

        assert_eq!(bounded_count(values, 3), None);
        assert_eq!(visited.get(), 4);
    }

    #[test]
    fn maximum_limit_cannot_overflow() {
        assert_eq!(bounded_codepoint_count("é🙂", usize::MAX), Some(2));
    }

    #[test]
    fn decoder_rejects_surrogates_and_out_of_range_values() {
        let valid = [0x41, 0x1f642];
        let byte_count = validate_codepoints(&valid).unwrap().unwrap();
        assert_eq!(byte_count, 5);
        assert_eq!(
            decode_validated_codepoints(&valid, byte_count).unwrap(),
            "A🙂"
        );
        assert_eq!(
            validate_codepoints(&[0xd800]),
            Err(PassTextError::InvalidUnicode)
        );
        assert_eq!(
            validate_codepoints(&[0x11_0000]),
            Err(PassTextError::InvalidUnicode)
        );
    }
}
