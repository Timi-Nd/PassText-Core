//! Safe conversions between Unicode scalar-value indices and UTF-8 byte offsets.
//!
//! All spans are half-open (`[start, end)`) and use Unicode scalar-value
//! ("code-point") indices. They are deliberately distinct from UTF-8 byte offsets.

use core::fmt;

/// A validated half-open span expressed in Unicode scalar-value indices.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TextSpan {
    start: usize,
    end: usize,
}

impl TextSpan {
    /// Creates a span `[start, end)`.
    ///
    /// The span is independent of a particular string, so this only verifies
    /// that `start <= end`. [`slice_text`] checks the source-text bounds.
    pub fn new(start: usize, end: usize) -> Result<Self, IndexError> {
        if end < start {
            return Err(IndexError::InvalidSpan);
        }
        Ok(Self { start, end })
    }

    /// Returns the inclusive start code-point index.
    #[must_use]
    pub const fn start(self) -> usize {
        self.start
    }

    /// Returns the exclusive end code-point index.
    #[must_use]
    pub const fn end(self) -> usize {
        self.end
    }

    /// Returns the span length in Unicode scalar values.
    #[must_use]
    pub const fn len(self) -> usize {
        self.end - self.start
    }

    /// Returns whether the span is empty.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.start == self.end
    }
}

/// An error produced by a Unicode-index or UTF-8-offset operation.
///
/// Error values contain only a classification. They never retain or display
/// the source text.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum IndexError {
    /// A span's end precedes its start.
    InvalidSpan,
    /// A code-point index is beyond the source's terminal boundary.
    CodepointIndexOutOfRange,
    /// A UTF-8 offset is beyond the source's terminal boundary.
    Utf8OffsetOutOfRange,
    /// A UTF-8 offset falls inside a multi-byte scalar value.
    Utf8OffsetNotCharBoundary,
    /// A valid span extends beyond the source text.
    SpanOutOfRange,
}

impl fmt::Display for IndexError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::InvalidSpan => "invalid text span",
            Self::CodepointIndexOutOfRange => "codepoint index is out of range",
            Self::Utf8OffsetOutOfRange => "UTF-8 offset is out of range",
            Self::Utf8OffsetNotCharBoundary => "UTF-8 offset is not on a valid code-point boundary",
            Self::SpanOutOfRange => "span is outside the source text",
        })
    }
}

impl std::error::Error for IndexError {}

/// Maps a Unicode scalar-value index to its UTF-8 byte offset.
///
/// The terminal index (the number of scalar values) maps to `text.len()`.
/// This operation does not allocate.
pub fn utf8_offset_for_codepoint(text: &str, codepoint_index: usize) -> Result<usize, IndexError> {
    if codepoint_index == 0 {
        return Ok(0);
    }

    match text.char_indices().nth(codepoint_index) {
        Some((offset, _)) => Ok(offset),
        None if text.chars().count() == codepoint_index => Ok(text.len()),
        None => Err(IndexError::CodepointIndexOutOfRange),
    }
}

/// Maps a UTF-8 byte offset to its Unicode scalar-value index.
///
/// The offset must be at a scalar-value boundary. The terminal byte offset is
/// accepted. This operation does not allocate.
pub fn codepoint_index_for_utf8_offset(
    text: &str,
    utf8_offset: usize,
) -> Result<usize, IndexError> {
    if utf8_offset > text.len() {
        return Err(IndexError::Utf8OffsetOutOfRange);
    }
    if !text.is_char_boundary(utf8_offset) {
        return Err(IndexError::Utf8OffsetNotCharBoundary);
    }

    Ok(text[..utf8_offset].chars().count())
}

/// Returns the substring selected by a half-open code-point span.
///
/// The returned value borrows from `text`; no password copy is made.
pub fn slice_text(text: &str, span: TextSpan) -> Result<&str, IndexError> {
    let start =
        utf8_offset_for_codepoint(text, span.start).map_err(|_| IndexError::SpanOutOfRange)?;
    let end = utf8_offset_for_codepoint(text, span.end).map_err(|_| IndexError::SpanOutOfRange)?;
    Ok(&text[start..end])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn span_validation_and_accessors() {
        let span = TextSpan::new(2, 5).expect("ordered span");
        assert_eq!(span.start(), 2);
        assert_eq!(span.end(), 5);
        assert_eq!(span.len(), 3);
        assert!(!span.is_empty());

        let empty = TextSpan::new(4, 4).expect("empty span");
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);
        assert_eq!(TextSpan::new(5, 4), Err(IndexError::InvalidSpan));
    }

    #[test]
    fn maps_all_boundaries_in_mixed_unicode_text() {
        let text = "Aé e\u{301}🙂👩\u{200d}💻";
        let expected = [0, 1, 3, 4, 5, 7, 11, 15, 18, 22];

        for (codepoint_index, byte_offset) in expected.into_iter().enumerate() {
            assert_eq!(
                utf8_offset_for_codepoint(text, codepoint_index),
                Ok(byte_offset)
            );
            assert_eq!(
                codepoint_index_for_utf8_offset(text, byte_offset),
                Ok(codepoint_index)
            );
        }
    }

    #[test]
    fn rejects_out_of_range_and_mid_scalar_offsets() {
        let text = "éA🙂";
        assert_eq!(
            utf8_offset_for_codepoint(text, 4),
            Err(IndexError::CodepointIndexOutOfRange)
        );
        assert_eq!(
            codepoint_index_for_utf8_offset(text, text.len() + 1),
            Err(IndexError::Utf8OffsetOutOfRange)
        );

        for offset in [1, 4, 5, 6] {
            assert_eq!(
                codepoint_index_for_utf8_offset(text, offset),
                Err(IndexError::Utf8OffsetNotCharBoundary)
            );
        }
    }

    #[test]
    fn slices_half_open_codepoint_spans_without_allocating() {
        let text = "Aé e\u{301}🙂👩\u{200d}💻";
        assert_eq!(slice_text(text, TextSpan::new(0, 0).unwrap()), Ok(""));
        assert_eq!(slice_text(text, TextSpan::new(0, 9).unwrap()), Ok(text));
        assert_eq!(
            slice_text(text, TextSpan::new(1, 6).unwrap()),
            Ok("é e\u{301}🙂")
        );
        assert_eq!(
            slice_text(text, TextSpan::new(9, 10).unwrap()),
            Err(IndexError::SpanOutOfRange)
        );
    }

    #[test]
    fn every_boundary_round_trips_for_representative_inputs() {
        let inputs = [
            "",
            "ASCII",
            "é",
            "e\u{301}",
            "🙂👩\u{200d}💻",
            "Γειά",
            "中文",
        ];

        for text in inputs {
            let count = text.chars().count();
            for index in 0..=count {
                let offset = utf8_offset_for_codepoint(text, index).unwrap();
                assert_eq!(codepoint_index_for_utf8_offset(text, offset), Ok(index));
            }

            for offset in 0..=text.len() {
                let result = codepoint_index_for_utf8_offset(text, offset);
                assert_eq!(result.is_ok(), text.is_char_boundary(offset));
            }
        }
    }

    #[test]
    fn errors_do_not_contain_source_text() {
        let secret = "INDEX-SECRET-🙂";
        let errors = [
            utf8_offset_for_codepoint(secret, 100).unwrap_err(),
            codepoint_index_for_utf8_offset(secret, 14).unwrap_err(),
            slice_text(secret, TextSpan::new(0, 100).unwrap()).unwrap_err(),
        ];

        for error in errors {
            let diagnostic = format!("{error:?} {error}");
            assert!(!diagnostic.contains(secret));
            assert!(!diagnostic.contains("INDEX-SECRET"));
        }
    }
}
