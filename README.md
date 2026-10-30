# PassText Core

PassText Core is a small Rust library for strict, Unicode-aware password text
preparation. It validates processing limits, optionally applies NFC
normalization, records non-secret metadata, and converts safely between Unicode
scalar indices and UTF-8 byte offsets.

The existing Python package in [`passtext/`](passtext/) is the behavioral
reference. Rust and Python tests consume the same portability vectors at
[`passtext/tests/fixtures/rust_portability_vectors.json`](passtext/tests/fixtures/rust_portability_vectors.json).

PassText Core deliberately does **not** score password strength, check breach
databases, hash credentials, or implement authentication policy.

## Toolchain and installation

PassText Core uses the Rust 2021 edition, and CI tracks the stable Rust
toolchain. The v0.1.0 release candidate is locally verified with `rustc
1.96.0`. The project does not yet declare a minimum supported Rust version
(MSRV); consumers using an older compiler should validate it in their own
build matrix.

After the project owner approves a license and the crate is published, add the
release from crates.io:

```toml
[dependencies]
passtext-core = "0.1.0"
```

Until then, use a local checkout for release-candidate evaluation:

```toml
[dependencies]
passtext-core = { path = "../PassText-Core" }
```

## Quick start

Add this crate to a Rust project, then process borrowed text without copying it
when no normalized representation is needed:

```rust
use passtext_core::{process_password, NormalizationMode, PassTextConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = PassTextConfig::new(4096, 1024, NormalizationMode::Nfc)?;
    let prepared = process_password("e\u{301}", config)?;

    assert_eq!(prepared.processed_text(), "é");
    assert_eq!(prepared.metadata().original_codepoints(), 2);
    assert_eq!(prepared.metadata().processed_codepoints(), 1);
    Ok(())
}
```

`process_password_with_default` uses the conservative defaults of 4096 UTF-8
bytes, 1024 Unicode scalar values, and no normalization. Owned strings, raw
UTF-8 bytes, and integer code-point arrays have dedicated entry points so that
invalid external representations can be rejected at their decoding boundary.

## Try a hidden password prompt

Run the local example from the repository root:

```text
cargo run --example password_prompt
```

It reads the password without echoing it to the terminal and processes it with
the default limits. It does not authenticate, hash, or print the password.

## Behavior at a glance

- Empty strings and whitespace are preserved.
- Limits apply to both Unicode scalar counts and UTF-8 byte counts.
- Normalization is explicit: `NormalizationMode::None` or
  `NormalizationMode::Nfc`.
- Metadata contains lengths and flags, never password text.
- Indexing uses half-open scalar ranges `[start, end)`; it never treats a byte
  offset as a character index.
- Invalid UTF-8, surrogate integer values, out-of-range indices, and byte
  offsets inside a UTF-8 sequence return typed errors without echoing input.

## Build and verify

```text
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
cargo doc --no-deps
```

The library core has no platform-specific runtime dependency. The hidden-input
example uses `rpassword` as a development dependency. CI also checks
`wasm32-unknown-unknown`; a local WebAssembly build requires that target to be
installed first.

The v0.1.0 release-candidate matrix covers 54 ordinary Rust tests (including
shared fixtures, generated properties, and a doctest), 214 explicit
Python–Rust differential comparisons on matching Unicode 16.0.0 data, 35
Python reference tests, documentation and package builds, and a
`wasm32-unknown-unknown` compile check. A RustSec scan of the locked dependency
graph found no known vulnerabilities or informational warnings; CI repeats the
scan weekly and when dependency files change. See the [testing
guide](docs/testing.md) for the commands and compatibility requirements behind
those results.

To run the reference suite:

```text
cd passtext
python -m pip install -e ".[dev]"
python -m pytest -q
```

## Documentation

- [API guide](docs/api.md)
- [Architecture and ownership](docs/architecture.md)
- [Python parity contract](docs/python-parity.md)
- [Performance methodology and results](docs/performance.md)
- [Security guarantees and limitations](docs/security.md)
- [Unicode compatibility](docs/unicode-compatibility.md)
- [Testing and portability](docs/testing.md)
- [Changelog](CHANGELOG.md)
- [v0.1.0 release-note draft](docs/releases/v0.1.0.md)

## Security boundary

Prepared values retain password text in memory for deliberate downstream
access. Their diagnostics and metadata are redacted, but Rust memory safety is
not secure erasure: borrowed strings cannot be zeroized by this crate, and
owned immutable strings are not promised to be scrubbed. See the
[security guide](docs/security.md) before integrating the library.

The crate has not received an independent security audit. It does not estimate
password strength, screen compromised-password sources, hash passwords, or
provide authentication. Its Unicode normalization behavior is intentionally
pinned to Unicode 16.0.0; see the
[Unicode compatibility guide](docs/unicode-compatibility.md) before changing
the normalization dependency or comparing results with older Python versions.

## Distribution note

The source assignment did not include a project license. The project owner must
select and add a license before publishing the crate or distributing it for
third-party reuse.
