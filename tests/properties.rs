//! Bounded property tests for Unicode, normalization, limits, and errors.
//!
//! Generated text and byte buffers deliberately have redacted `Debug` output,
//! so a failing proptest does not echo password-like values in diagnostics.

use std::fmt;
use std::panic;

use passtext_core::{
    codepoint_index_for_utf8_offset, process_codepoints, process_password, process_utf8,
    slice_text, utf8_offset_for_codepoint, IndexError, LimitUnit, NormalizationMode,
    PassTextConfig, PassTextError, ProcessingStage, TextSpan,
};
use proptest::prelude::*;

#[derive(Clone)]
struct RedactedText(String);

impl fmt::Debug for RedactedText {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("<redacted generated Unicode text>")
    }
}

#[derive(Clone)]
struct RedactedBytes(Vec<u8>);

impl fmt::Debug for RedactedBytes {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("<redacted generated invalid bytes>")
    }
}

#[derive(Clone, Copy)]
struct RedactedCodepoint(u32);

impl fmt::Debug for RedactedCodepoint {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("<redacted generated invalid code point>")
    }
}

fn unicode_text() -> impl Strategy<Value = RedactedText> {
    prop::collection::vec(any::<char>(), 0..65)
        .prop_map(|characters| RedactedText(characters.into_iter().collect()))
}

fn invalid_utf8() -> impl Strategy<Value = RedactedBytes> {
    prop_oneof![
        (0x80_u8..=0xbf).prop_map(|byte| RedactedBytes(vec![byte])),
        (0xc2_u8..=0xdf).prop_map(|byte| RedactedBytes(vec![byte])),
        (0xf5_u8..=0xff).prop_map(|byte| RedactedBytes(vec![b'A', byte])),
    ]
}

fn invalid_codepoint() -> impl Strategy<Value = RedactedCodepoint> {
    prop_oneof![
        (0xd800_u32..=0xdfff).prop_map(RedactedCodepoint),
        (0x11_0000_u32..=u32::MAX).prop_map(RedactedCodepoint),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 128,
        max_shrink_iters: 2_048,
        ..ProptestConfig::default()
    })]

    #[test]
    fn unicode_indices_and_utf8_boundaries_round_trip(value in unicode_text()) {
        let text = value.0.as_str();
        let codepoint_count = text.chars().count();

        for codepoint_index in 0..=codepoint_count {
            let byte_offset = utf8_offset_for_codepoint(text, codepoint_index)
                .expect("a generated valid code-point boundary must map to UTF-8");
            let round_trip = codepoint_index_for_utf8_offset(text, byte_offset)
                .expect("a mapped UTF-8 boundary must map back to a code-point index");
            prop_assert!(round_trip == codepoint_index, "index round trip differed");
        }

        for byte_offset in 0..=text.len() {
            let result = codepoint_index_for_utf8_offset(text, byte_offset);
            if text.is_char_boundary(byte_offset) {
                prop_assert!(result.is_ok(), "a valid UTF-8 boundary was rejected");
            } else {
                prop_assert!(
                    matches!(result, Err(IndexError::Utf8OffsetNotCharBoundary)),
                    "an interior UTF-8 byte offset had the wrong result"
                );
            }
        }

        prop_assert!(
            matches!(
                codepoint_index_for_utf8_offset(text, text.len() + 1),
                Err(IndexError::Utf8OffsetOutOfRange)
            ),
            "an out-of-range UTF-8 offset had the wrong error"
        );
        prop_assert!(
            matches!(
                utf8_offset_for_codepoint(text, codepoint_count + 1),
                Err(IndexError::CodepointIndexOutOfRange)
            ),
            "an out-of-range code-point index had the wrong error"
        );
        prop_assert!(
            matches!(
                codepoint_index_for_utf8_offset(text, text.len()),
                Ok(index) if index == codepoint_count
            ),
            "the terminal UTF-8 offset was not accepted"
        );
    }

    #[test]
    fn half_open_slicing_matches_scalar_ranges(
        value in unicode_text(),
        first in any::<u8>(),
        second in any::<u8>(),
    ) {
        let text = value.0.as_str();
        let count = text.chars().count();
        let boundary_count = count + 1;
        let first = usize::from(first) % boundary_count;
        let second = usize::from(second) % boundary_count;
        let start = first.min(second);
        let end = first.max(second);

        let span = TextSpan::new(start, end).expect("ordered generated span");
        let actual = slice_text(text, span).expect("in-range generated span");
        let expected: String = text.chars().skip(start).take(end - start).collect();
        prop_assert!(actual == expected, "valid half-open slice differed");

        let empty = TextSpan::new(start, start).expect("empty span");
        prop_assert!(
            matches!(slice_text(text, empty), Ok(slice) if slice.is_empty()),
            "empty span did not return empty text"
        );
        let full = TextSpan::new(0, count).expect("full span");
        prop_assert!(
            matches!(slice_text(text, full), Ok(slice) if slice == text),
            "full span did not return the original text"
        );
        prop_assert!(
            matches!(TextSpan::new(1, 0), Err(IndexError::InvalidSpan)),
            "reversed span had the wrong error"
        );
        let outside = TextSpan::new(count, count + 1).expect("ordered outside span");
        prop_assert!(
            matches!(slice_text(text, outside), Err(IndexError::SpanOutOfRange)),
            "outside span had the wrong error"
        );
    }

    #[test]
    fn normalization_metadata_and_determinism_hold(value in unicode_text()) {
        let text = value.0.as_str();

        for mode in [NormalizationMode::None, NormalizationMode::Nfc] {
            let config = PassTextConfig::new(4_096, 1_024, mode)
                .expect("fixed property-test configuration");
            let first = process_password(text, config)
                .expect("bounded generated Unicode must process");
            let second = process_password(text, config)
                .expect("repeated bounded generated Unicode must process");
            let metadata = first.metadata();

            prop_assert!(
                metadata.original_codepoints() == text.chars().count(),
                "original code-point metadata differed"
            );
            prop_assert!(
                metadata.original_utf8_bytes() == text.len(),
                "original UTF-8 metadata differed"
            );
            prop_assert!(
                metadata.processed_codepoints() == first.processed_text().chars().count(),
                "processed code-point metadata differed"
            );
            prop_assert!(
                metadata.processed_utf8_bytes() == first.processed_text().len(),
                "processed UTF-8 metadata differed"
            );
            prop_assert!(
                metadata.contains_non_ascii() != text.is_ascii(),
                "non-ASCII metadata differed"
            );
            prop_assert!(metadata.normalization() == mode, "normalization mode differed");
            prop_assert!(
                metadata.normalization_changed() == (first.processed_text() != text),
                "normalization changed flag differed"
            );
            prop_assert!(
                first.processed_text() == second.processed_text(),
                "repeated processing was not deterministic"
            );
            prop_assert!(
                first.metadata() == second.metadata(),
                "repeated metadata was not deterministic"
            );

            if mode == NormalizationMode::Nfc {
                let repeated = process_password(first.processed_text(), config)
                    .expect("NFC output must remain processable");
                prop_assert!(
                    repeated.processed_text() == first.processed_text(),
                    "repeated NFC was not idempotent"
                );
                prop_assert!(
                    !repeated.metadata().normalization_changed(),
                    "already-NFC text was reported as changed"
                );
            }
        }
    }

    #[test]
    fn original_limits_accept_boundaries_and_preserve_precedence(value in unicode_text()) {
        let text = value.0.as_str();
        let exact = PassTextConfig::new(
            text.len().max(1),
            text.chars().count().max(1),
            NormalizationMode::None,
        ).expect("positive exact-limit configuration");
        prop_assert!(
            process_password(text, exact).is_ok(),
            "an input exactly at its nonzero limits was rejected"
        );

        let over_codepoints = format!("{text}xx");
        let over_codepoint_count = over_codepoints.chars().count();
        let codepoint_limited = PassTextConfig::new(
            over_codepoints.len(),
            over_codepoint_count - 1,
            NormalizationMode::None,
        ).expect("positive code-point-limit configuration");
        prop_assert!(
            matches!(
                process_password(&over_codepoints, codepoint_limited),
                Err(PassTextError::ProcessingLimit {
                    stage: ProcessingStage::Original,
                    unit: LimitUnit::UnicodeCodepoints,
                })
            ),
            "original code-point overflow had the wrong precedence or category"
        );

        let over_bytes = format!("{text}é");
        let byte_limited = PassTextConfig::new(
            over_bytes.len() - 1,
            over_bytes.chars().count(),
            NormalizationMode::None,
        ).expect("positive byte-limit configuration");
        prop_assert!(
            matches!(
                process_password(&over_bytes, byte_limited),
                Err(PassTextError::ProcessingLimit {
                    stage: ProcessingStage::Original,
                    unit: LimitUnit::Utf8Bytes,
                })
            ),
            "original UTF-8 overflow had the wrong category"
        );
    }

    #[test]
    fn nfc_expansion_respects_processed_limits(prefix_length in 0_usize..16) {
        // Exclamation marks are stable starters that cannot compose with the
        // two combining marks produced from U+0344.
        let input = format!("{}\u{344}", "!".repeat(prefix_length));
        let original_codepoints = input.chars().count();
        let original_bytes = input.len();

        let codepoint_limited = PassTextConfig::new(
            original_bytes + 8,
            original_codepoints,
            NormalizationMode::Nfc,
        ).expect("positive processed-code-point-limit configuration");
        prop_assert!(
            matches!(
                process_password(&input, codepoint_limited),
                Err(PassTextError::ProcessingLimit {
                    stage: ProcessingStage::Processed,
                    unit: LimitUnit::UnicodeCodepoints,
                })
            ),
            "NFC code-point expansion had the wrong category"
        );

        let byte_limited = PassTextConfig::new(
            original_bytes,
            original_codepoints + 1,
            NormalizationMode::Nfc,
        ).expect("positive processed-byte-limit configuration");
        prop_assert!(
            matches!(
                process_password(&input, byte_limited),
                Err(PassTextError::ProcessingLimit {
                    stage: ProcessingStage::Processed,
                    unit: LimitUnit::Utf8Bytes,
                })
            ),
            "NFC byte expansion had the wrong category"
        );
    }

    #[test]
    fn invalid_external_utf8_returns_typed_errors_without_panics(bytes in invalid_utf8()) {
        let outcome = panic::catch_unwind(|| process_utf8(&bytes.0, PassTextConfig::default()));
        prop_assert!(outcome.is_ok(), "invalid external UTF-8 caused a panic");
        let result = outcome.expect("panic status was checked");
        prop_assert!(
            matches!(result, Err(PassTextError::InvalidUnicode)),
            "invalid external UTF-8 had the wrong result"
        );
    }

    #[test]
    fn invalid_integer_scalars_return_typed_errors_without_panics(value in invalid_codepoint()) {
        let outcome = panic::catch_unwind(|| {
            process_codepoints(&[value.0], PassTextConfig::default())
        });
        prop_assert!(outcome.is_ok(), "invalid integer code point caused a panic");
        let result = outcome.expect("panic status was checked");
        prop_assert!(
            matches!(result, Err(PassTextError::InvalidUnicode)),
            "invalid integer code point had the wrong result"
        );
    }
}
