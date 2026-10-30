# API guide

## Configuration

`PassTextConfig::default()` selects:

- `max_utf8_bytes = 4096`
- `max_codepoints = 1024`
- `NormalizationMode::None`

Use `PassTextConfig::new(max_utf8_bytes, max_codepoints, normalization)` for
explicit configuration. Limits must be greater than zero. Configuration fields
are private and exposed through read-only getters, so a validated configuration
cannot later be mutated into an invalid state.

`NormalizationMode` is an enum with `None` and `Nfc` variants. Modeling the
mode as an enum makes unsupported string values unrepresentable in ordinary
Rust code.

## Processing entry points

- `process_password(&str, config)` borrows valid Rust text.
- `process_password_with_default(&str)` uses the default configuration.
- `process_owned_password(String, config)` takes ownership without first
  cloning the string.
- `process_utf8(&[u8], config)` strictly decodes an external UTF-8 buffer.
- `process_codepoints(&[u32], config)` validates every integer as a Unicode
  scalar value before constructing text. In particular, surrogate values such
  as U+D800 are rejected.

All processing functions return `Result<PreparedPassword, PassTextError>`.
Rust's input types replace Python's runtime non-string check; other observable
error categories remain explicit.

Validation checks the original scalar limit before its byte limit, then applies
the selected normalization and checks the processed result. This ordering
matches the Python reference where the two languages can represent the same
input.

## Prepared text and metadata

`PreparedPassword` provides deliberate accessors:

- `original_text()`
- `processed_text()`
- `metadata()`
- `config()`

The original value remains available because later analysis may need it. When
processing borrowed input, the implementation borrows it. NFC output is owned
only when normalization changes the text.

`TextMetadata` exposes read-only getters for:

- original Unicode scalar count
- original UTF-8 byte count
- processed Unicode scalar count
- processed UTF-8 byte count
- whether the original contains non-ASCII text
- normalization mode
- whether normalization changed the text

Metadata does not contain either text representation.

## Indexing

`TextSpan::new(start, end)` constructs a half-open range over Unicode scalar
indices. `start` may equal `end`; `end < start` is rejected.

```rust
use passtext_core::{
    codepoint_index_for_utf8_offset, slice_text,
    utf8_offset_for_codepoint, TextSpan,
};

let text = "éA🙂";
assert_eq!(utf8_offset_for_codepoint(text, 2)?, 3);
assert_eq!(codepoint_index_for_utf8_offset(text, 3)?, 2);
assert_eq!(slice_text(text, TextSpan::new(1, 3)?)?, "A🙂");
# Ok::<(), Box<dyn std::error::Error>>(())
```

An endpoint at the end of the text is valid. A byte offset inside a multibyte
UTF-8 encoding, an out-of-range index, or a span outside the source returns an
`IndexError`. These APIs count scalar values, not user-perceived grapheme
clusters.

## Errors

`PassTextError` separates invalid Unicode, processing-limit failures, invalid
configuration, and unexpected processing failure. Indexing has its own
`IndexError`. Error display and debug output describe only the category and
safe structural detail; they never include input text.

