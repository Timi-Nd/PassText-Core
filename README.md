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

The core has no platform-specific runtime dependency. CI also checks
`wasm32-unknown-unknown`; a local WebAssembly build requires that target to be
installed first.

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
- [Security guarantees and limitations](docs/security.md)
- [Unicode compatibility](docs/unicode-compatibility.md)
- [Testing and portability](docs/testing.md)

## Security boundary

Prepared values retain password text in memory for deliberate downstream
access. Their diagnostics and metadata are redacted, but Rust memory safety is
not secure erasure: borrowed strings cannot be zeroized by this crate, and
owned immutable strings are not promised to be scrubbed. See the
[security guide](docs/security.md) before integrating the library.

## Distribution note

The source assignment did not include a project license. The project owner must
select and add a license before publishing the crate or distributing it for
third-party reuse.

