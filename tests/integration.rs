use std::panic;
use std::thread;

use passtext_core::{
    codepoint_index_for_utf8_offset, process_codepoints, process_owned_password, process_password,
    process_password_with_default, process_utf8, slice_text, utf8_offset_for_codepoint,
    NormalizationMode, PassTextConfig, PassTextError, TextSpan,
};

#[test]
fn defaults_accept_empty_and_preserve_text_exactly() {
    let empty = process_password_with_default("").expect("empty input is valid");
    assert_eq!(empty.original_text(), "");
    assert_eq!(empty.processed_text(), "");
    assert_eq!(empty.metadata().original_codepoints(), 0);
    assert_eq!(empty.metadata().original_utf8_bytes(), 0);

    let whitespace = process_password_with_default(" a\nb ").expect("whitespace is valid");
    assert_eq!(whitespace.original_text(), " a\nb ");
    assert_eq!(whitespace.processed_text(), " a\nb ");
    assert!(!whitespace.metadata().normalization_changed());
}

#[test]
fn metadata_uses_unicode_scalars_and_utf8_bytes_not_graphemes() {
    let input = "Aé e\u{301}🙂👩\u{200d}💻";
    let prepared = process_password_with_default(input).expect("representative Unicode is valid");
    let metadata = prepared.metadata();

    assert_eq!(metadata.original_codepoints(), input.chars().count());
    assert_eq!(metadata.original_utf8_bytes(), input.len());
    assert_eq!(metadata.processed_codepoints(), input.chars().count());
    assert_eq!(metadata.processed_utf8_bytes(), input.len());
    assert!(metadata.contains_non_ascii());
    assert_eq!(metadata.normalization(), NormalizationMode::None);
}

#[test]
fn nfc_can_trigger_processed_codepoint_and_byte_limits() {
    // NFC decomposes U+0344 into U+0308 U+0301. This is a useful regression case because
    // normalization grows both the scalar count and the UTF-8 representation.
    let input = "\u{0344}";

    let codepoint_limited = PassTextConfig::new(16, 1, NormalizationMode::Nfc).unwrap();
    let error = process_password(input, codepoint_limited).unwrap_err();
    assert!(matches!(error, PassTextError::ProcessingLimit { .. }));
    assert!(error.to_string().contains("processed"));
    assert!(error.to_string().contains("code-point"));

    let byte_limited = PassTextConfig::new(2, 4, NormalizationMode::Nfc).unwrap();
    let error = process_password(input, byte_limited).unwrap_err();
    assert!(matches!(error, PassTextError::ProcessingLimit { .. }));
    assert!(error.to_string().contains("processed"));
    assert!(error.to_string().contains("UTF-8 byte"));
}

#[test]
fn original_codepoint_limit_is_checked_before_original_byte_limit() {
    let config = PassTextConfig::new(1, 1, NormalizationMode::None).unwrap();
    let error = process_password("éé", config).unwrap_err();

    assert!(matches!(error, PassTextError::ProcessingLimit { .. }));
    assert!(error.to_string().contains("code-point count"));
    assert!(!error.to_string().contains("UTF-8 byte count"));
}

#[test]
fn external_utf8_decoding_is_strict_and_non_lossy() {
    for invalid in [
        &[0xff][..],
        &[0xc3, 0x28][..],
        &[0xe2, 0x82][..],
        &[0xf0, 0x28, 0x8c, 0xbc][..],
    ] {
        let error = process_utf8(invalid, PassTextConfig::default()).unwrap_err();
        assert!(matches!(error, PassTextError::InvalidUnicode));
    }

    let valid = process_utf8("Grüße🙂".as_bytes(), PassTextConfig::default())
        .expect("valid UTF-8 should be borrowed and processed");
    assert_eq!(valid.original_text(), "Grüße🙂");
    assert_eq!(valid.processed_text(), "Grüße🙂");
}

#[test]
fn external_codepoints_reject_non_scalars_without_panicking() {
    for invalid in [&[0xd800][..], &[0xdfff][..], &[0x11_0000][..]] {
        let result = panic::catch_unwind(|| process_codepoints(invalid, PassTextConfig::default()));
        let error = result
            .expect("invalid external input must not panic")
            .unwrap_err();
        assert!(matches!(error, PassTextError::InvalidUnicode));
    }

    let valid = process_codepoints(&[0x41, 0x00e9, 0x1f642], PassTextConfig::default())
        .expect("valid scalar array should process");
    assert_eq!(valid.original_text(), "Aé🙂");
}

#[test]
fn owned_input_moves_into_prepared_representation() {
    let original = String::from("e\u{301}");
    let config = PassTextConfig::new(4096, 1024, NormalizationMode::Nfc).unwrap();
    let prepared = process_owned_password(original, config).expect("owned input should process");

    assert_eq!(prepared.original_text(), "e\u{301}");
    assert_eq!(prepared.processed_text(), "é");
    assert!(prepared.metadata().normalization_changed());
    assert_eq!(prepared.config().normalization(), NormalizationMode::Nfc);
}

#[test]
fn diagnostics_and_errors_do_not_disclose_passwords() {
    let secret = "P@ssw0rd!SECRETMARKER";
    let prepared = process_password_with_default(secret).unwrap();

    let prepared_debug = format!("{prepared:?}");
    let metadata_debug = format!("{:?}", prepared.metadata());
    assert!(!prepared_debug.contains(secret));
    assert!(!prepared_debug.contains("SECRETMARKER"));
    assert!(!metadata_debug.contains(secret));
    assert_eq!(
        prepared.original_text(),
        secret,
        "deliberate access remains available"
    );

    let config = PassTextConfig::new(8, 8, NormalizationMode::None).unwrap();
    let error = process_password(secret, config).unwrap_err();
    let display = error.to_string();
    let debug = format!("{error:?}");
    assert!(!display.contains(secret));
    assert!(!display.contains("SECRETMARKER"));
    assert!(!debug.contains(secret));
    assert!(!debug.contains("SECRETMARKER"));
}

#[test]
fn processing_is_deterministic_and_thread_safe() {
    let values = ["abc", "éé", "Γειά", "e\u{301}", "🙂🙂", "👩\u{200d}💻"];
    let handles: Vec<_> = values
        .into_iter()
        .map(|value| {
            thread::spawn(move || {
                let config = PassTextConfig::new(4096, 1024, NormalizationMode::Nfc).unwrap();
                let first = process_owned_password(value.to_owned(), config).unwrap();
                let second = process_owned_password(value.to_owned(), config).unwrap();
                assert_eq!(first.processed_text(), second.processed_text());
                assert_eq!(first.metadata(), second.metadata());
                (
                    first.original_text().to_owned(),
                    first.processed_text().to_owned(),
                )
            })
        })
        .collect();

    let results: Vec<_> = handles
        .into_iter()
        .map(|handle| handle.join().expect("worker must not panic"))
        .collect();
    assert_eq!(results.len(), values.len());
    assert_eq!(results[3].1, "é");
}

#[test]
fn representative_unicode_invariants_hold_for_both_modes() {
    let values = [
        "",
        "ASCII",
        "é",
        "e\u{301}",
        "Γειά",
        "Привет",
        "مرحبا",
        "中🙂",
        "👩\u{200d}💻",
        " a\n\t b ",
        "\0",
    ];

    for mode in [NormalizationMode::None, NormalizationMode::Nfc] {
        let config = PassTextConfig::new(4096, 1024, mode).unwrap();
        for input in values {
            let first = process_password(input, config).unwrap();
            let second = process_password(input, config).unwrap();
            let metadata = first.metadata();

            assert_eq!(metadata.original_codepoints(), input.chars().count());
            assert_eq!(metadata.original_utf8_bytes(), input.len());
            assert_eq!(
                metadata.processed_codepoints(),
                first.processed_text().chars().count()
            );
            assert_eq!(
                metadata.processed_utf8_bytes(),
                first.processed_text().len()
            );
            assert_eq!(metadata.contains_non_ascii(), !input.is_ascii());
            assert_eq!(metadata.normalization(), mode);
            assert_eq!(
                metadata.normalization_changed(),
                first.processed_text() != input
            );
            assert_eq!(first.processed_text(), second.processed_text());
            assert_eq!(first.metadata(), second.metadata());
        }
    }
}

#[test]
fn indexing_round_trips_every_boundary_in_representative_texts() {
    let values = [
        "",
        "ASCII",
        "éA🙂",
        "e\u{301}",
        "Aé e\u{301}🙂👩\u{200d}💻",
        "Γειά Привет مرحبا 中",
    ];

    for text in values {
        let codepoint_count = text.chars().count();
        let mut valid_offsets = Vec::with_capacity(codepoint_count + 1);

        for codepoint_index in 0..=codepoint_count {
            let byte_offset = utf8_offset_for_codepoint(text, codepoint_index).unwrap();
            valid_offsets.push(byte_offset);
            assert_eq!(
                codepoint_index_for_utf8_offset(text, byte_offset).unwrap(),
                codepoint_index
            );
        }

        for byte_offset in 0..=text.len() {
            let result = codepoint_index_for_utf8_offset(text, byte_offset);
            assert_eq!(result.is_ok(), valid_offsets.contains(&byte_offset));
        }

        let full = TextSpan::new(0, codepoint_count).unwrap();
        assert_eq!(slice_text(text, full).unwrap(), text);
        let empty = TextSpan::new(0, 0).unwrap();
        assert_eq!(slice_text(text, empty).unwrap(), "");
    }
}

#[test]
fn ordinary_invalid_inputs_return_errors_instead_of_panicking() {
    let outcome = panic::catch_unwind(|| {
        assert!(PassTextConfig::new(0, 1, NormalizationMode::None).is_err());
        assert!(PassTextConfig::new(1, 0, NormalizationMode::None).is_err());
        assert!(process_utf8(&[0xff], PassTextConfig::default()).is_err());
        assert!(process_codepoints(&[0xd800], PassTextConfig::default()).is_err());
        assert!(utf8_offset_for_codepoint("é", 2).is_err());
        assert!(codepoint_index_for_utf8_offset("é", 1).is_err());
        assert!(TextSpan::new(2, 1).is_err());
        let outside = TextSpan::new(0, 2).unwrap();
        assert!(slice_text("a", outside).is_err());
    });
    assert!(outcome.is_ok());
}

#[test]
fn invalid_configuration_has_a_typed_category() {
    for result in [
        PassTextConfig::new(0, 1, NormalizationMode::None),
        PassTextConfig::new(1, 0, NormalizationMode::None),
    ] {
        let error = result.unwrap_err();
        assert!(matches!(error, PassTextError::InvalidConfiguration { .. }));
    }
}
