# Security guarantees and limitations

PassText Core treats every processed value as sensitive, including synthetic or
test passwords in production-shaped code.

## Controls provided by the crate

- Raw password text is never logged by the core.
- `PreparedPassword` has a manual, metadata-only `Debug` implementation.
- Password contents are absent from error values and their `Display`/`Debug`
  output.
- `TextMetadata` contains counts and flags only.
- Secret-bearing types do not implement general-purpose serialization.
- Processing uses no global password store or shared mutable state.
- Borrowed input remains borrowed, and normalization allocates only when a
  changed representation must be retained.

Storage-reuse tests verify that borrowed text keeps the caller's buffer, owned
text moves without cloning its allocation, already-normalized NFC text reuses
the original representation, and changed NFC retains one distinct result.

Text access is nevertheless intentional. `original_text()` and
`processed_text()` return the values required by downstream password analysis.
Application code must not log, format, serialize, or persist those returned
slices.

## Memory lifetime

A prepared value keeps the original text and, when necessary, an owned NFC
result alive for the lifetime of the value. Dropping it releases ownership or
references according to normal Rust rules.

This is **not** a secure-erasure guarantee:

- Rust memory safety prevents many classes of invalid access, but does not wipe
  memory automatically.
- The crate cannot erase a caller-owned borrowed `str`.
- Allocator behavior, copies made by callers, crash dumps, swap, debuggers, and
  compiler optimizations are outside this guarantee.
- Immutable text is not claimed to be zeroized after drop.

Integrations with a hard zeroization requirement need an end-to-end secret
storage design. Adding a wrapper at this layer alone cannot erase upstream
copies or borrowed memory.

## Diagnostics and failures

Errors intentionally avoid input excerpts. Applications should log the error
category only and should not attach the rejected value as structured context.
Panics are not part of the ordinary invalid-input contract: malformed UTF-8,
invalid scalar arrays, limit violations, and invalid indices return typed
errors.

Property-test inputs use wrappers with redacted `Debug` implementations, and
the differential runner identifies failures only by corpus index and mode.

## Out of scope

The crate does not hash passwords, authenticate users, estimate guessability,
screen breach corpora, or enforce site password policy. Those responsibilities
need separate components and threat models.

