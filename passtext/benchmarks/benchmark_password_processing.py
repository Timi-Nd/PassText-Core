"""Micro-benchmark for representative PassText workloads.

This benchmark separates steady-state latency measurement from memory profiling so the
performance numbers reflect realistic processing cost rather than instrumentation overhead.
"""

from __future__ import annotations

import json
import os
import platform
import statistics
import sys
import time
import tracemalloc
from pathlib import Path

from passtext import PassTextConfig, process_password

WORKLOADS = [
    ("ascii_short_none", "P@ssw0rd!", "none", "Short ASCII"),
    ("ascii_long_none", "Tr0ub4dor&3" * 20, "none", "Long ASCII"),
    ("accented_none", "café", "none", "Accented Latin"),
    ("unicode_mixed_none", "Hello世界Γειά", "none", "Mixed scripts"),
    ("emoji_none", "Hello🙂👩‍💻", "none", "Emoji and emoji sequence"),
    ("combining_none", "e\u0301e\u0301e\u0301", "none", "Combining marks without normalization"),
    ("combining_nfc", "e\u0301e\u0301e\u0301", "nfc", "NFC changes the input"),
    ("ascii_nfc", "Example123!", "nfc", "NFC leaves the input unchanged"),
    ("near_limit_none", "é" * 1023, "none", "One code point below the configured limit"),
]


def benchmark_latency(
    value: str, config: PassTextConfig, *, iterations: int = 400, warmup: int = 50
) -> list[float]:
    for _ in range(warmup):
        process_password(value, config=config)

    timings: list[float] = []
    for _ in range(iterations):
        start = time.perf_counter()
        process_password(value, config=config)
        timings.append(time.perf_counter() - start)
    return timings


def benchmark_memory(value: str, config: PassTextConfig, *, iterations: int = 250) -> int:
    tracemalloc.start()
    try:
        for _ in range(iterations):
            process_password(value, config=config)
        _, peak = tracemalloc.get_traced_memory()
    finally:
        tracemalloc.stop()
    return peak


def summarize_timings(timings: list[float]) -> dict[str, float | int]:
    median = statistics.median(timings)
    average = statistics.fmean(timings)
    p95 = sorted(timings)[int(len(timings) * 0.95) - 1]
    throughput = 1.0 / average if average > 0 else 0.0
    return {
        "median_seconds": median,
        "p95_seconds": p95,
        "average_seconds": average,
        "throughput_per_second": throughput,
    }


def run_suite() -> dict[str, object]:
    workloads: dict[str, object] = {}
    for label, value, normalization, description in WORKLOADS:
        config = PassTextConfig(
            max_utf8_bytes=4096,
            max_codepoints=1024,
            normalization=normalization,
        )
        latency = benchmark_latency(value, config)
        peak_memory = benchmark_memory(value, config)
        workloads[label] = {
            "description": description,
            "input_codepoints": len(value),
            "input_utf8_bytes": len(value.encode("utf-8")),
            "normalization": normalization,
            "configuration": {
                "max_utf8_bytes": config.max_utf8_bytes,
                "max_codepoints": config.max_codepoints,
            },
            "latency": summarize_timings(latency),
            "memory": {
                "iterations": 250,
                "peak_traced_bytes": peak_memory,
            },
        }
    return {
        "methodology": {
            "latency_iterations": 400,
            "warmup_iterations": 50,
            "memory_iterations": 250,
            "memory_tracing_during_latency": False,
        },
        "workloads": workloads,
    }


def print_summary(results: dict[str, object]) -> None:
    print("PassText benchmark summary")
    print("=" * 28)
    workloads = results["workloads"]
    assert isinstance(workloads, dict)
    for label, item in workloads.items():
        assert isinstance(item, dict)
        latency = item["latency"]
        memory = item["memory"]
        assert isinstance(latency, dict)
        assert isinstance(memory, dict)
        print(
            f"{label:>22}: median={latency['median_seconds']:.8f}s "
            f"p95={latency['p95_seconds']:.8f}s avg={latency['average_seconds']:.8f}s "
            f"throughput={latency['throughput_per_second']:.2f}/s "
            f"peak_traced={memory['peak_traced_bytes']} bytes"
        )


if __name__ == "__main__":
    result = run_suite()
    print_summary(result)
    payload = {
        "benchmark_version": 2,
        "python_version": sys.version.split()[0],
        "platform": platform.platform(),
        "system": platform.system(),
        "processor": platform.processor(),
        "cpu_count": os.cpu_count(),
        **result,
    }
    output_path = Path("benchmarks/output.txt")
    output_path.write_text(json.dumps(payload, indent=2, sort_keys=True), encoding="utf-8")
