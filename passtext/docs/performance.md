# Performance notes

This project intentionally prioritizes correctness before optimization.

## Current approach

The Python reference implementation uses straightforward O(n) traversal for:
- UTF-8 validation
- code-point counting
- byte counting
- NFC normalization
- metadata extraction

The pipeline has been adjusted to reject oversized inputs before expensive UTF-8 encoding and to avoid re-encoding values that are already validated and unchanged.

## Measurement methodology

`benchmarks/benchmark_password_processing.py` runs 50 warm-up calls followed by 400 individually timed calls for each workload. It computes median, p95, average latency, and reciprocal-average throughput. Each workload uses a single immutable configuration created outside its timed calls, with limits of 4096 UTF-8 bytes and 1024 code points.

Memory is measured in a separate pass of 250 processing calls with `tracemalloc` enabled. These peak values describe traced Python allocations during that pass, not total process memory or secure-memory usage. Memory tracing is not enabled during latency measurements. Results and environment details are written as JSON to `benchmarks/output.txt`.

## Latest baseline

Run on Windows 11, CPython 3.14.6, Intel64 Family 6 Model 170 Stepping 4, with 14 logical CPUs available. The workload results below are copied from the machine-readable output artifact. The `none` and `nfc` modes are measured independently; `combining_nfc` changes the input while `ascii_nfc` does not.

| Workload | Mode | Median (s) | P95 (s) | Average (s) | Throughput (/s) | Peak traced bytes |
| --- | --- | ---: | ---: | ---: | ---: | ---: |
| ASCII short | none | 0.00000300 | 0.00000330 | 0.00000309 | 323180.08 | 432 |
| ASCII long | none | 0.00001540 | 0.00001630 | 0.00001571 | 63654.74 | 432 |
| Accented Latin | none | 0.00000270 | 0.00000300 | 0.00000273 | 366300.39 | 432 |
| Mixed scripts | none | 0.00000330 | 0.00000620 | 0.00000380 | 263140.58 | 492 |
| Emoji sequence | none | 0.00000310 | 0.00000340 | 0.00000316 | 316580.91 | 496 |
| Combining marks | none | 0.00000280 | 0.00000310 | 0.00000282 | 355176.67 | 492 |
| Combining marks | nfc | 0.00000330 | 0.00000350 | 0.00000328 | 304437.17 | 552 |
| ASCII | nfc | 0.00000320 | 0.00000370 | 0.00000343 | 291248.00 | 432 |
| Near limit | none | 0.00003350 | 0.00006240 | 0.00003887 | 25728.60 | 2079 |

The near-limit case uses 1023 `é` code points (2046 UTF-8 bytes), one code point below the configured 1024-code-point maximum. These results establish a reproducible Python baseline; there is no equivalent old-version measurement here, so no speedup is claimed. Wall-clock values vary by machine, interpreter, and system load.

The library is designed for future Rust parity while preserving a simple Python reference model.
