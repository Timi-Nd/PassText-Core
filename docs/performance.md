# Performance verification

These measurements were executed on 2026-10-30 as a stabilization check. They
are reproducible observations from one development machine, not a general
speedup claim or service-level guarantee.

## Environment

- OS: Microsoft Windows 10.0.26200.9711 (Python reports Windows 11 build 26200)
- Processor: AMD64 Family 26 Model 68 Stepping 0, AuthenticAMD
- Rust: `rustc 1.96.0 (ac68faa20 2026-05-25)`
- Python: CPython 3.14.6
- Rust and Python Unicode normalization data: 16.0.0
- Rust build mode: optimized `cargo bench` profile

## Rust methodology and results

`benches/processing.rs` uses the same nine synthetic inputs and configurations
as the Python reference benchmark. Every workload has an explicit warm-up,
followed by 31 batched samples. Inputs, configuration, and results pass through
`std::hint::black_box`. The table reports nanoseconds per operation.

| Workload | Warm-up | Operations/sample | Median ns | p95 ns | Stddev ns |
|---|---:|---:|---:|---:|---:|
| Short ASCII, none | 20,000 | 100,000 | 12.197 | 12.738 | 0.197 |
| Long ASCII, none | 10,000 | 50,000 | 135.324 | 238.924 | 46.374 |
| Accented, none | 20,000 | 100,000 | 11.789 | 18.151 | 2.569 |
| Mixed scripts, none | 20,000 | 100,000 | 13.050 | 27.365 | 5.656 |
| Emoji, none | 20,000 | 100,000 | 15.119 | 16.821 | 0.704 |
| Combining marks, none | 20,000 | 100,000 | 11.934 | 12.297 | 0.196 |
| Combining marks, NFC changed | 10,000 | 50,000 | 151.196 | 267.798 | 40.399 |
| ASCII, NFC unchanged | 20,000 | 100,000 | 14.938 | 15.414 | 0.774 |
| Near code-point limit, none | 2,000 | 10,000 | 794.580 | 1,681.910 | 271.472 |

Run the same harness with:

```text
PASSTEXT_PYTHON=python cargo bench --bench processing
```

Rust allocation counts were not recorded because no reliable allocator
instrumentation was available. The ownership tests verify reuse behavior, but
they are not a substitute for allocation telemetry.

## Python reference methodology and results

The existing Python harness uses 50 warm-up operations, 400 individually timed
operations, and a separate 250-iteration `tracemalloc` pass. Times below are
microseconds per operation; memory is peak traced bytes for that separate pass.

| Workload | Median µs | p95 µs | Peak traced bytes |
|---|---:|---:|---:|
| Short ASCII, none | 2.40 | 2.40 | 432 |
| Long ASCII, none | 11.20 | 11.40 | 432 |
| Accented, none | 2.20 | 2.30 | 432 |
| Mixed scripts, none | 2.50 | 2.60 | 492 |
| Emoji, none | 2.40 | 2.50 | 496 |
| Combining marks, none | 2.20 | 2.30 | 492 |
| Combining marks, NFC changed | 2.60 | 2.70 | 552 |
| ASCII, NFC unchanged | 2.50 | 2.60 | 432 |
| Near code-point limit, none | 25.90 | 26.00 | 2,079 |

The Python suite can be measured without overwriting its recorded reference
file by importing `run_suite` from
`passtext/benchmarks/benchmark_password_processing.py`.

## Interpretation limits

The inputs and machine match, but timer placement and batching do not. Python
times each call; Rust amortizes timer overhead across a batch. Background load,
frequency scaling, and scheduler effects were not controlled, as visible in
the long-input variability. The measurements should therefore be used as
separate implementation baselines. No Rust-to-Python speedup ratio is claimed.
