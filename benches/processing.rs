//! Small dependency-free processing benchmark.
//!
//! This reports local wall-clock observations, not a cross-language speedup.

use std::hint::black_box;
use std::time::Instant;

use passtext_core::{process_password, NormalizationMode, PassTextConfig};

struct Case<'a> {
    name: &'a str,
    input: &'a str,
    config: PassTextConfig,
    iterations: u32,
}

fn main() {
    let ascii_long = "Tr0ub4dor&3".repeat(20);
    let near_limit = "é".repeat(1023);
    let cases = [
        Case {
            name: "ascii-short/none",
            input: "P@ssw0rd!",
            config: PassTextConfig::default(),
            iterations: 200_000,
        },
        Case {
            name: "ascii-long/none",
            input: &ascii_long,
            config: PassTextConfig::default(),
            iterations: 50_000,
        },
        Case {
            name: "accented/none",
            input: "café",
            config: PassTextConfig::default(),
            iterations: 200_000,
        },
        Case {
            name: "mixed-scripts/none",
            input: "Hello世界Γειά",
            config: PassTextConfig::default(),
            iterations: 150_000,
        },
        Case {
            name: "emoji/none",
            input: "Hello🙂👩\u{200d}💻",
            config: PassTextConfig::default(),
            iterations: 150_000,
        },
        Case {
            name: "combining/none",
            input: "e\u{301}e\u{301}e\u{301}",
            config: PassTextConfig::default(),
            iterations: 150_000,
        },
        Case {
            name: "combining/nfc-changed",
            input: "e\u{301}e\u{301}e\u{301}",
            config: PassTextConfig::default().with_normalization(NormalizationMode::Nfc),
            iterations: 100_000,
        },
        Case {
            name: "ascii/nfc-unchanged",
            input: "Example123!",
            config: PassTextConfig::default().with_normalization(NormalizationMode::Nfc),
            iterations: 150_000,
        },
        Case {
            name: "near-codepoint-limit/none",
            input: &near_limit,
            config: PassTextConfig::default(),
            iterations: 20_000,
        },
    ];

    println!("PassText Core synthetic processing benchmark");
    println!("case,iterations,total_ms,ns_per_operation");
    for case in cases {
        let started = Instant::now();
        for _ in 0..case.iterations {
            let prepared = process_password(black_box(case.input), case.config)
                .expect("benchmark cases are valid");
            black_box(prepared.processed_text());
            black_box(prepared.metadata());
        }
        let elapsed = started.elapsed();
        let nanos_per_operation = elapsed.as_nanos() / u128::from(case.iterations);
        println!(
            "{},{},{:.3},{}",
            case.name,
            case.iterations,
            elapsed.as_secs_f64() * 1_000.0,
            nanos_per_operation
        );
    }
}
