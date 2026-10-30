# Testing and cross-language parity

## Shared source of truth

Both implementations read
`passtext/tests/fixtures/rust_portability_vectors.json`. Expected Rust outcomes
must not be copied into a second fixture.

The versioned vectors cover successful processing, exact processed text and all
metadata fields, original limit failures, invalid surrogate code-point arrays,
valid scalar/byte boundary conversions, invalid mid-scalar byte offsets, and
half-open slicing.

The surrogate vector stores U+D800 as an integer. The Rust runner verifies the
entire array before constructing a `String`, then expects the invalid-Unicode
error category.

Native Rust tests add edge cases for configuration, processed limits after NFC,
empty/full spans, confidentiality of diagnostics, deterministic behavior, and
representative scalar/index round trips.

Bounded `proptest` cases add generated valid Unicode, indexing, slicing,
normalization, limit, invalid-byte, and invalid-scalar invariants. Generated
values have redacted diagnostic representations.

## Required Rust checks

Run from the repository root:

```text
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo doc --no-deps
```

Run the property target directly when isolating randomized coverage:

```text
cargo test --test properties
```

If the WebAssembly target is installed, also run:

```text
cargo check --target wasm32-unknown-unknown --all-features
```

Absence of that target is a local verification limitation and must not be
reported as a successful Wasm build. CI installs the target explicitly.

## Python reference checks

```text
cd passtext
python -m pip install -e ".[dev]"
python -m pytest -q
```

Do not claim Python parity from Rust tests alone: report which suites actually
ran and the Python and Unicode database versions used.

With a Unicode-16 Python interpreter, run the fixed-seed structured
differential corpus explicitly:

```text
PASSTEXT_PYTHON=python3 cargo test --test differential -- --ignored
```

GitHub Actions runs this differential test in its Python 3.14 job. The Python
3.10 job continues to validate the shared long-stable fixtures, but its older
Unicode database is intentionally not used for an exhaustive normalization
comparison.

## Benchmarking

Correctness checks come before performance measurement. Any benchmark should
use synthetic inputs, release builds, representative ASCII/Unicode/combining
and near-limit cases, and recorded toolchain/hardware details. Rust and Python
numbers from different environments are not a defensible speedup claim.

The dependency-free local harness covers those representative cases:

```text
cargo bench --bench processing
```

Its wall-clock observations are useful for regressions on the same controlled
machine. They are not statistically rigorous cross-machine results or a claim
of improvement over Python.

The latest same-machine methodology, environment, raw summary statistics, and
interpretation limits are recorded in [performance.md](performance.md).

