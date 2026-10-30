# Python-to-Rust parity contract

The unmodified package under `passtext/` is the behavioral reference. Rust
tests read its versioned portability fixture directly; expected values are not
duplicated in a Rust-only fixture.

## Processing order

For text that both languages can represent, processing follows this order:

1. Establish a valid immutable configuration.
2. Enforce the original Unicode code-point limit.
3. Validate the external Unicode/UTF-8 representation.
4. Enforce the original UTF-8 byte limit.
5. Apply the explicit `none` or `nfc` mode.
6. Enforce the processed code-point limit.
7. If normalization changed the text, enforce its processed byte limit.
8. Build content-free metadata and the prepared representation.

The original limit preceding scalar validation is observable for integer
code-point arrays and has a regression test. An external malformed byte buffer
must be decoded before it becomes `str`, so `process_utf8` rejects malformed
UTF-8 at that boundary before text counts can exist.

## Equivalent values

Rust metadata has the same seven meanings as Python: original and processed
code-point counts, original and processed byte counts, the original non-ASCII
flag, normalization mode, and exact changed flag. Empty input and whitespace
are valid. Code points mean Rust Unicode scalar values, never grapheme clusters.

Index conversion accepts terminal boundaries, rejects byte offsets inside a
UTF-8 sequence, and slices half-open code-point spans. NFC data is pinned to
Unicode 16.0.0, matching the verified CPython 3.14.6 environment.

Error categories map as follows:

- Python `InvalidUnicodeError` to Rust `PassTextError::InvalidUnicode`
- Python `ProcessingLimitError` to Rust `PassTextError::ProcessingLimit`
- Python `InvalidConfigurationError` to Rust
  `PassTextError::InvalidConfiguration`
- Python `ProcessingFailureError` to Rust `PassTextError::ProcessingFailure`

Fixture parity compares categories rather than language-specific exception
wording, as required by the Python portability specification.

## Intentional language-level differences

- Rust `&str` makes a non-string input and malformed UTF-8 unrepresentable in
  the main entry point; dedicated byte and integer-array entry points provide
  strict external decoding.
- `usize` makes negative indices and limits unrepresentable.
- `NormalizationMode` makes an unsupported mode unrepresentable after parsing.
- Password-bearing fields are private and exposed through deliberate methods
  instead of Python attributes.
- Borrowed input stays borrowed; owned input can be moved; changed NFC text is
  the only normal path that allocates a second retained representation.

No defect requiring a change to the Python reference was found during the
port. Python's Unicode database can vary across supported interpreter versions;
that compatibility boundary is detailed in `unicode-compatibility.md`.
