//! Cross-language contract tests.
//!
//! These tests intentionally consume the Python project's fixture in place. Keeping one
//! versioned source of truth prevents the Rust and Python expectations from drifting apart.

use std::fs;
use std::path::PathBuf;

use passtext_core::{
    codepoint_index_for_utf8_offset, process_codepoints, process_password, slice_text,
    utf8_offset_for_codepoint, NormalizationMode, PassTextConfig, PassTextError, PreparedPassword,
    TextSpan,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct Fixtures {
    schema_version: u32,
    unicode_representation: String,
    cases: Vec<ProcessVector>,
    indexing: Vec<IndexVector>,
}

#[derive(Debug, Deserialize)]
struct ProcessVector {
    name: String,
    operation: String,
    input: Option<String>,
    input_codepoints: Option<Vec<u32>>,
    config: FixtureConfig,
    expected: Option<ExpectedProcess>,
    expected_error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct FixtureConfig {
    max_utf8_bytes: usize,
    max_codepoints: usize,
    normalization: String,
}

#[derive(Debug, Deserialize)]
struct ExpectedProcess {
    processed_text: String,
    original_codepoints: usize,
    original_utf8_bytes: usize,
    processed_codepoints: usize,
    processed_utf8_bytes: usize,
    contains_non_ascii: bool,
    normalization: String,
    normalization_changed: bool,
}

#[derive(Debug, Deserialize)]
struct IndexVector {
    name: String,
    text: String,
    codepoint_to_utf8: Vec<usize>,
    utf8_to_codepoint: Vec<usize>,
    invalid_utf8_offsets: Vec<usize>,
    spans: Vec<SpanVector>,
}

#[derive(Debug, Deserialize)]
struct SpanVector {
    start: usize,
    end: usize,
    expected: String,
}

fn fixture_path() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("passtext")
        .join("tests")
        .join("fixtures")
        .join("rust_portability_vectors.json")
}

fn load_fixtures() -> Fixtures {
    let path = fixture_path();
    let payload = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
    serde_json::from_str(&payload)
        .unwrap_or_else(|error| panic!("failed to parse {}: {error}", path.display()))
}

fn normalization(value: &str) -> NormalizationMode {
    match value {
        "none" => NormalizationMode::None,
        "nfc" => NormalizationMode::Nfc,
        unsupported => panic!("fixture contains unsupported normalization mode: {unsupported}"),
    }
}

fn config(value: &FixtureConfig) -> PassTextConfig {
    PassTextConfig::new(
        value.max_utf8_bytes,
        value.max_codepoints,
        normalization(&value.normalization),
    )
    .expect("fixture configuration must be valid")
}

fn assert_success(case_name: &str, prepared: &PreparedPassword<'_>, expected: &ExpectedProcess) {
    let metadata = prepared.metadata();

    assert_eq!(
        prepared.processed_text(),
        expected.processed_text,
        "{case_name}: processed text"
    );
    assert_eq!(
        metadata.original_codepoints(),
        expected.original_codepoints,
        "{case_name}: original code points"
    );
    assert_eq!(
        metadata.original_utf8_bytes(),
        expected.original_utf8_bytes,
        "{case_name}: original UTF-8 bytes"
    );
    assert_eq!(
        metadata.processed_codepoints(),
        expected.processed_codepoints,
        "{case_name}: processed code points"
    );
    assert_eq!(
        metadata.processed_utf8_bytes(),
        expected.processed_utf8_bytes,
        "{case_name}: processed UTF-8 bytes"
    );
    assert_eq!(
        metadata.contains_non_ascii(),
        expected.contains_non_ascii,
        "{case_name}: non-ASCII flag"
    );
    assert_eq!(
        metadata.normalization(),
        normalization(&expected.normalization),
        "{case_name}: normalization mode"
    );
    assert_eq!(
        metadata.normalization_changed(),
        expected.normalization_changed,
        "{case_name}: normalization changed flag"
    );
}

fn assert_error_category(case_name: &str, error: PassTextError, expected: &str) {
    let matches = match expected {
        "InvalidUnicodeError" => matches!(error, PassTextError::InvalidUnicode),
        "ProcessingLimitError" => matches!(error, PassTextError::ProcessingLimit { .. }),
        unsupported => panic!("fixture contains unsupported error category: {unsupported}"),
    };
    assert!(matches, "{case_name}: unexpected error category: {error:?}");
}

#[test]
fn process_vectors_match_python_reference() {
    let fixtures = load_fixtures();
    assert_eq!(fixtures.schema_version, 1);
    assert!(fixtures.unicode_representation.contains("Unicode scalar"));
    assert!(!fixtures.cases.is_empty());

    for vector in &fixtures.cases {
        let config = config(&vector.config);

        match (vector.input.as_deref(), vector.input_codepoints.as_deref()) {
            (Some(input), None) => match vector.operation.as_str() {
                "process" => {
                    let expected = vector.expected.as_ref().unwrap_or_else(|| {
                        panic!("{}: process vector is missing expected output", vector.name)
                    });
                    let result = process_password(input, config).unwrap_or_else(|error| {
                        panic!("{}: unexpected error: {error}", vector.name)
                    });
                    assert_success(&vector.name, &result, expected);
                }
                "process_error" => {
                    let expected_error = vector.expected_error.as_deref().unwrap_or_else(|| {
                        panic!("{}: error vector is missing expected_error", vector.name)
                    });
                    let error = process_password(input, config).unwrap_err();
                    assert_error_category(&vector.name, error, expected_error);
                }
                operation => panic!("{}: unsupported operation {operation}", vector.name),
            },
            (None, Some(codepoints)) => {
                assert_eq!(
                    vector.operation, "process_error",
                    "{}: code-point fixtures currently describe invalid external input",
                    vector.name
                );
                let expected_error = vector.expected_error.as_deref().unwrap_or_else(|| {
                    panic!("{}: error vector is missing expected_error", vector.name)
                });

                // `process_codepoints` validates every integer as a Unicode scalar before it
                // constructs a Rust `String`. In particular, U+D800 must never enter `str`.
                let error = process_codepoints(codepoints, config).unwrap_err();
                assert_error_category(&vector.name, error, expected_error);
            }
            _ => panic!(
                "{}: vector must contain exactly one of input and input_codepoints",
                vector.name
            ),
        }
    }
}

#[test]
fn indexing_vectors_match_python_reference() {
    let fixtures = load_fixtures();
    assert!(!fixtures.indexing.is_empty());

    for vector in &fixtures.indexing {
        let codepoint_count = vector.text.chars().count();
        assert_eq!(
            vector.codepoint_to_utf8.len(),
            codepoint_count + 1,
            "{}: missing code-point boundaries",
            vector.name
        );
        assert_eq!(
            vector.utf8_to_codepoint.len(),
            vector.codepoint_to_utf8.len(),
            "{}: boundary tables differ in length",
            vector.name
        );

        for (codepoint_index, (&expected_byte_offset, &expected_codepoint_index)) in vector
            .codepoint_to_utf8
            .iter()
            .zip(&vector.utf8_to_codepoint)
            .enumerate()
        {
            let byte_offset = utf8_offset_for_codepoint(&vector.text, codepoint_index)
                .unwrap_or_else(|error| panic!("{}: {error}", vector.name));
            assert_eq!(byte_offset, expected_byte_offset, "{}", vector.name);

            let round_trip = codepoint_index_for_utf8_offset(&vector.text, byte_offset)
                .unwrap_or_else(|error| panic!("{}: {error}", vector.name));
            assert_eq!(round_trip, expected_codepoint_index, "{}", vector.name);
            assert_eq!(round_trip, codepoint_index, "{}", vector.name);
        }

        for &byte_offset in &vector.invalid_utf8_offsets {
            assert!(
                codepoint_index_for_utf8_offset(&vector.text, byte_offset).is_err(),
                "{}: byte offset {byte_offset} unexpectedly accepted",
                vector.name
            );
        }

        for span in &vector.spans {
            let span_value = TextSpan::new(span.start, span.end)
                .unwrap_or_else(|error| panic!("{}: {error}", vector.name));
            let actual = slice_text(&vector.text, span_value)
                .unwrap_or_else(|error| panic!("{}: {error}", vector.name));
            assert_eq!(actual, span.expected, "{}", vector.name);
        }
    }
}
