//! Explicit Python-to-Rust differential testing.
//!
//! This test is ignored by default because it requires a compatible Python
//! interpreter. Run it with:
//!
//! `cargo test --test differential -- --ignored`

use std::env;
use std::fmt;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};

use passtext_core::{
    process_password, NormalizationMode, PassTextConfig, UNICODE_NORMALIZATION_VERSION,
};
use serde::{Deserialize, Serialize};

const MAX_UTF8_BYTES: usize = 4096;
const MAX_CODEPOINTS: usize = 1024;
const GENERATED_CASES: usize = 96;
const CORPUS_SEED: u64 = 0x50_41_53_53_54_45_58_54;

const PYTHON_PROBE: &str = r#"
import json
import platform
import sys
import unicodedata

sys.path.insert(0, sys.argv[1])

from passtext import PassTextConfig, process_password

request = json.load(sys.stdin)
results = []
for case_index, case in enumerate(request["cases"]):
    codepoints = case["input_codepoints"]
    if any(
        codepoint < 0
        or codepoint > 0x10FFFF
        or 0xD800 <= codepoint <= 0xDFFF
        for codepoint in codepoints
    ):
        json.dump(
            {"case_index": case_index, "error_type": "InvalidScalar"},
            sys.stderr,
            separators=(",", ":"),
        )
        sys.exit(2)
    input_text = "".join(chr(codepoint) for codepoint in codepoints)
    config = PassTextConfig(
        max_utf8_bytes=4096,
        max_codepoints=1024,
        normalization=case["normalization"],
    )
    prepared = process_password(input_text, config=config)
    results.append(
        {
            "processed_text": prepared.processed_text,
            "metadata": prepared.metadata.as_dict(),
        }
    )

json.dump(
    {
        "python_version": platform.python_version(),
        "python_version_info": list(sys.version_info[:3]),
        "unicode_version": unicodedata.unidata_version,
        "results": results,
    },
    sys.stdout,
    ensure_ascii=True,
    separators=(",", ":"),
)
"#;

/// A generated password-shaped value whose diagnostics are always redacted.
struct RedactedText(String);

impl RedactedText {
    fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for RedactedText {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("RedactedText(<redacted>)")
    }
}

#[derive(Serialize)]
struct ProbeRequest {
    cases: Vec<ProbeCase>,
}

#[derive(Serialize)]
struct ProbeCase {
    input_codepoints: Vec<u32>,
    normalization: &'static str,
}

#[derive(Deserialize)]
struct ProbeResponse {
    python_version: String,
    python_version_info: [u32; 3],
    unicode_version: String,
    results: Vec<ProbeResult>,
}

#[derive(Deserialize)]
struct ProbeResult {
    processed_text: String,
    metadata: ProbeMetadata,
}

#[derive(Deserialize)]
struct ProbeMetadata {
    original_codepoints: usize,
    original_utf8_bytes: usize,
    processed_codepoints: usize,
    processed_utf8_bytes: usize,
    contains_non_ascii: bool,
    normalization: String,
    normalization_changed: bool,
}

#[test]
#[ignore = "requires an explicit compatible Python interpreter"]
fn python_and_rust_match_on_deterministic_unicode_corpus() {
    let corpus = deterministic_corpus();
    let request_cases: Vec<_> = corpus
        .iter()
        .flat_map(|text| {
            ["none", "nfc"].map(|normalization| ProbeCase {
                input_codepoints: text.as_str().chars().map(u32::from).collect(),
                normalization,
            })
        })
        .collect();

    let expected_result_count = request_cases.len();
    let response = run_python_probe(&ProbeRequest {
        cases: request_cases,
    });

    assert_supported_python(&response);
    assert_unicode_versions_match(&response);
    assert_eq!(
        response.results.len(),
        expected_result_count,
        "Python probe returned an unexpected result count"
    );

    eprintln!(
        "differential parity environment: Python {}, Python UCD {}, Rust UCD {}.{}.{}",
        response.python_version,
        response.unicode_version,
        UNICODE_NORMALIZATION_VERSION.0,
        UNICODE_NORMALIZATION_VERSION.1,
        UNICODE_NORMALIZATION_VERSION.2,
    );

    for (result_index, python_result) in response.results.iter().enumerate() {
        let case_index = result_index / 2;
        let mode = if result_index % 2 == 0 {
            NormalizationMode::None
        } else {
            NormalizationMode::Nfc
        };
        let config = PassTextConfig::new(MAX_UTF8_BYTES, MAX_CODEPOINTS, mode)
            .expect("differential configuration is valid");
        let rust_result =
            process_password(corpus[case_index].as_str(), config).unwrap_or_else(|error| {
                panic!(
                    "case {case_index} mode {}: Rust processing failed: {error}",
                    mode.as_str()
                )
            });
        let rust_metadata = rust_result.metadata();
        let context = || format!("case {case_index} mode {}", mode.as_str());

        assert!(
            rust_result.processed_text() == python_result.processed_text,
            "{}: processed text mismatch",
            context()
        );
        assert!(
            rust_metadata.original_codepoints() == python_result.metadata.original_codepoints,
            "{}: original code-point count mismatch",
            context()
        );
        assert!(
            rust_metadata.original_utf8_bytes() == python_result.metadata.original_utf8_bytes,
            "{}: original UTF-8 byte count mismatch",
            context()
        );
        assert!(
            rust_metadata.processed_codepoints() == python_result.metadata.processed_codepoints,
            "{}: processed code-point count mismatch",
            context()
        );
        assert!(
            rust_metadata.processed_utf8_bytes() == python_result.metadata.processed_utf8_bytes,
            "{}: processed UTF-8 byte count mismatch",
            context()
        );
        assert!(
            rust_metadata.contains_non_ascii() == python_result.metadata.contains_non_ascii,
            "{}: non-ASCII flag mismatch",
            context()
        );
        assert!(
            rust_metadata.normalization().as_str() == python_result.metadata.normalization,
            "{}: normalization mode mismatch",
            context()
        );
        assert!(
            rust_metadata.normalization_changed() == python_result.metadata.normalization_changed,
            "{}: normalization-changed flag mismatch",
            context()
        );
    }
}

fn run_python_probe(request: &ProbeRequest) -> ProbeResponse {
    let python = env::var_os("PASSTEXT_PYTHON").unwrap_or_else(|| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    });
    let source_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("passtext")
        .join("src");
    let mut child = Command::new(&python)
        .arg("-c")
        .arg(PYTHON_PROBE)
        .arg(source_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap_or_else(|error| {
            panic!("failed to start the configured Python interpreter (PASSTEXT_PYTHON): {error}")
        });

    {
        let stdin = child
            .stdin
            .as_mut()
            .expect("Python probe stdin must be piped");
        serde_json::to_writer(&mut *stdin, request)
            .expect("differential request must serialize as JSON");
        stdin
            .flush()
            .expect("differential request must flush to Python");
    }
    drop(child.stdin.take());

    let output = child
        .wait_with_output()
        .expect("Python differential probe must complete");
    if !output.status.success() {
        let diagnostic = String::from_utf8_lossy(&output.stderr);
        panic!(
            "Python differential probe failed with status {}: {}",
            output.status, diagnostic
        );
    }

    serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
        panic!("Python differential response was not valid structured JSON: {error}")
    })
}

fn assert_supported_python(response: &ProbeResponse) {
    let [major, minor, _patch] = response.python_version_info;
    assert!(
        major > 3 || (major == 3 && minor >= 10),
        "differential parity requires Python 3.10 or newer; found {}",
        response.python_version
    );
}

fn assert_unicode_versions_match(response: &ProbeResponse) {
    let rust_version = format!(
        "{}.{}.{}",
        UNICODE_NORMALIZATION_VERSION.0,
        UNICODE_NORMALIZATION_VERSION.1,
        UNICODE_NORMALIZATION_VERSION.2
    );
    assert!(
        response.unicode_version == rust_version,
        "Python UCD {} is incompatible with Rust UCD {}; use a matching interpreter",
        response.unicode_version,
        rust_version
    );
}

fn deterministic_corpus() -> Vec<RedactedText> {
    let mut corpus = [
        "",
        "ASCII",
        " a\n\tb ",
        "café",
        "e\u{301}",
        "\u{344}",
        "Γειά Привет مرحبا 中",
        "🙂👩\u{200d}💻",
        "Aß🙂123",
        "Å\u{212b}Ω\u{2126}",
        "\0",
    ]
    .into_iter()
    .map(|value| RedactedText(value.to_owned()))
    .collect::<Vec<_>>();

    let alphabet = [
        'a', 'Z', '0', ' ', '\n', 'é', 'ß', 'Γ', 'Ж', 'ع', '中', '🙂', '\u{301}', '\u{200d}',
    ];
    let mut state = CORPUS_SEED;
    for _ in 0..GENERATED_CASES {
        let length = (next_seeded(&mut state) % 49) as usize;
        let mut value = String::with_capacity(length.saturating_mul(2));
        for _ in 0..length {
            let index = (next_seeded(&mut state) % alphabet.len() as u64) as usize;
            value.push(alphabet[index]);
        }
        corpus.push(RedactedText(value));
    }
    corpus
}

fn next_seeded(state: &mut u64) -> u64 {
    *state = state
        .wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(1_442_695_040_888_963_407);
    *state
}
