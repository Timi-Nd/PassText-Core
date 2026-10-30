//! Reproducible, dependency-free processing benchmark.
//!
//! The harness uses the same nine synthetic workloads as the Python reference.
//! It reports each implementation separately; it does not claim a controlled
//! cross-language speedup.

use std::env;
use std::fs;
use std::hint::black_box;
use std::process::Command;
use std::time::Instant;

use passtext_core::{
    process_password, NormalizationMode, PassTextConfig, UNICODE_NORMALIZATION_VERSION,
};

const SAMPLE_COUNT: usize = 31;

struct Case<'a> {
    name: &'a str,
    input: &'a str,
    config: PassTextConfig,
    warmup_iterations: u32,
    iterations_per_sample: u32,
}

struct Statistics {
    mean: f64,
    median: f64,
    p95: f64,
    standard_deviation: f64,
    minimum: f64,
    maximum: f64,
}

fn main() {
    let ascii_long = "Tr0ub4dor&3".repeat(20);
    let near_limit = "é".repeat(1023);
    let cases = [
        Case {
            name: "ascii-short/none",
            input: "P@ssw0rd!",
            config: PassTextConfig::default(),
            warmup_iterations: 20_000,
            iterations_per_sample: 100_000,
        },
        Case {
            name: "ascii-long/none",
            input: &ascii_long,
            config: PassTextConfig::default(),
            warmup_iterations: 10_000,
            iterations_per_sample: 50_000,
        },
        Case {
            name: "accented/none",
            input: "café",
            config: PassTextConfig::default(),
            warmup_iterations: 20_000,
            iterations_per_sample: 100_000,
        },
        Case {
            name: "mixed-scripts/none",
            input: "Hello世界Γειά",
            config: PassTextConfig::default(),
            warmup_iterations: 20_000,
            iterations_per_sample: 100_000,
        },
        Case {
            name: "emoji/none",
            input: "Hello🙂👩\u{200d}💻",
            config: PassTextConfig::default(),
            warmup_iterations: 20_000,
            iterations_per_sample: 100_000,
        },
        Case {
            name: "combining/none",
            input: "e\u{301}e\u{301}e\u{301}",
            config: PassTextConfig::default(),
            warmup_iterations: 20_000,
            iterations_per_sample: 100_000,
        },
        Case {
            name: "combining/nfc-changed",
            input: "e\u{301}e\u{301}e\u{301}",
            config: PassTextConfig::default().with_normalization(NormalizationMode::Nfc),
            warmup_iterations: 10_000,
            iterations_per_sample: 50_000,
        },
        Case {
            name: "ascii/nfc-unchanged",
            input: "Example123!",
            config: PassTextConfig::default().with_normalization(NormalizationMode::Nfc),
            warmup_iterations: 20_000,
            iterations_per_sample: 100_000,
        },
        Case {
            name: "near-codepoint-limit/none",
            input: &near_limit,
            config: PassTextConfig::default(),
            warmup_iterations: 2_000,
            iterations_per_sample: 10_000,
        },
    ];

    print_environment();
    println!("methodology.timer=std::time::Instant");
    println!("methodology.samples={SAMPLE_COUNT}");
    println!("methodology.measurement=batched release-mode operations after per-case warm-up");
    println!(
        "methodology.compiler_barrier=std::hint::black_box on inputs, configuration, and outputs"
    );
    println!("methodology.allocations=not measured; no allocator instrumentation was available");
    println!(
        "case,warmup_iterations,samples,iterations_per_sample,total_iterations,mean_ns_per_op,median_ns_per_op,p95_ns_per_op,stddev_ns_per_op,min_ns_per_op,max_ns_per_op,total_measured_ms"
    );

    for case in &cases {
        run_operations(case, case.warmup_iterations);

        let mut samples = Vec::with_capacity(SAMPLE_COUNT);
        let suite_started = Instant::now();
        for _ in 0..SAMPLE_COUNT {
            let started = Instant::now();
            run_operations(case, case.iterations_per_sample);
            let elapsed = started.elapsed();
            samples.push(
                elapsed.as_secs_f64() * 1_000_000_000.0 / f64::from(case.iterations_per_sample),
            );
        }
        let total_measured_ms = suite_started.elapsed().as_secs_f64() * 1_000.0;
        let statistics = summarize(&mut samples);
        let total_iterations = u64::from(case.iterations_per_sample)
            * u64::try_from(SAMPLE_COUNT).expect("sample count fits in u64");

        println!(
            "{},{},{},{},{},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},{:.3}",
            case.name,
            case.warmup_iterations,
            SAMPLE_COUNT,
            case.iterations_per_sample,
            total_iterations,
            statistics.mean,
            statistics.median,
            statistics.p95,
            statistics.standard_deviation,
            statistics.minimum,
            statistics.maximum,
            total_measured_ms,
        );
    }
}

fn run_operations(case: &Case<'_>, iterations: u32) {
    for _ in 0..iterations {
        let prepared = process_password(black_box(case.input), black_box(case.config))
            .expect("synthetic benchmark case must remain valid");
        black_box(prepared.processed_text());
        black_box(prepared.metadata());
    }
}

fn summarize(samples: &mut [f64]) -> Statistics {
    assert_eq!(samples.len(), SAMPLE_COUNT);
    samples.sort_by(f64::total_cmp);

    let mean = samples.iter().sum::<f64>() / samples.len() as f64;
    let variance = samples
        .iter()
        .map(|sample| (sample - mean).powi(2))
        .sum::<f64>()
        / samples.len() as f64;
    let p95_index = ((samples.len() as f64 * 0.95).ceil() as usize)
        .saturating_sub(1)
        .min(samples.len() - 1);

    Statistics {
        mean,
        median: samples[samples.len() / 2],
        p95: samples[p95_index],
        standard_deviation: variance.sqrt(),
        minimum: samples[0],
        maximum: samples[samples.len() - 1],
    }
}

fn print_environment() {
    let (unicode_major, unicode_minor, unicode_patch) = UNICODE_NORMALIZATION_VERSION;
    println!("benchmark=PassText Core processing");
    println!("environment.os={}", os_description());
    println!("environment.arch={}", env::consts::ARCH);
    println!("environment.processor={}", processor_description());
    println!(
        "environment.rustc={}",
        command_first_line("rustc", &["--version"]).unwrap_or_else(|| "unavailable".to_owned())
    );
    println!("environment.rust_unicode={unicode_major}.{unicode_minor}.{unicode_patch}");
    println!(
        "environment.build_mode={}",
        if cfg!(debug_assertions) {
            "debug (results are not release-quality)"
        } else {
            "release/bench"
        }
    );

    let python_command = env::var("PASSTEXT_PYTHON").unwrap_or_else(|_| {
        if cfg!(windows) {
            "python".to_owned()
        } else {
            "python3".to_owned()
        }
    });
    match python_versions(&python_command) {
        Some((python_version, unicode_version)) => {
            println!("environment.python_command={python_command}");
            println!("environment.python={python_version}");
            println!("environment.python_unicode={unicode_version}");
        }
        None => {
            println!("environment.python_command={python_command}");
            println!("environment.python=unavailable");
            println!("environment.python_unicode=unavailable");
        }
    }
}

fn os_description() -> String {
    let detected = if cfg!(windows) {
        command_first_line("cmd", &["/C", "ver"])
    } else {
        command_first_line("uname", &["-srv"])
    };
    detected.unwrap_or_else(|| env::consts::OS.to_owned())
}

fn processor_description() -> String {
    if let Ok(identifier) = env::var("PROCESSOR_IDENTIFIER") {
        let identifier = identifier.trim();
        if !identifier.is_empty() {
            return identifier.to_owned();
        }
    }

    if cfg!(target_os = "linux") {
        if let Ok(cpu_info) = fs::read_to_string("/proc/cpuinfo") {
            if let Some(model) = cpu_info.lines().find_map(|line| {
                let (key, value) = line.split_once(':')?;
                matches!(key.trim(), "model name" | "Hardware").then(|| value.trim().to_owned())
            }) {
                return model;
            }
        }
    }

    if cfg!(target_os = "macos") {
        if let Some(model) = command_first_line("sysctl", &["-n", "machdep.cpu.brand_string"]) {
            return model;
        }
    }

    "unavailable".to_owned()
}

fn python_versions(program: &str) -> Option<(String, String)> {
    let script = "import platform, unicodedata; print(platform.python_version()); print(unicodedata.unidata_version)";
    let output = Command::new(program).args(["-c", script]).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8(output.stdout).ok()?;
    let mut lines = stdout.lines();
    let python_version = lines.next()?.trim();
    let unicode_version = lines.next()?.trim();
    if python_version.is_empty() || unicode_version.is_empty() {
        return None;
    }
    Some((python_version.to_owned(), unicode_version.to_owned()))
}

fn command_first_line(program: &str, arguments: &[&str]) -> Option<String> {
    let output = Command::new(program).args(arguments).output().ok()?;
    if !output.status.success() {
        return None;
    }
    let stdout = String::from_utf8(output.stdout).ok()?;
    let first_line = stdout.lines().find(|line| !line.trim().is_empty())?.trim();
    (!first_line.is_empty()).then(|| first_line.to_owned())
}
