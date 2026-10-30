# Changelog

All notable changes to PassText Core are documented in this file.

## [0.1.0] - Unreleased

Initial release candidate of the Rust text-processing foundation:

- Strict validation of UTF-8 text, integer Unicode scalar input, configuration,
  processing limits, and scalar/byte indices.
- Explicit preservation or Unicode 16.0.0 NFC normalization, with content-free
  metadata describing the result.
- Safe conversion between Unicode scalar indices and UTF-8 byte offsets, plus
  half-open scalar slicing.
- Borrowed and owned processing paths that reuse storage when normalization
  does not change the input.
- Shared portability fixtures and a deterministic differential corpus for the
  Python reference implementation.
- Generated property tests, content-free diagnostics, WebAssembly compilation
  coverage, and recorded same-machine performance baselines.

The crate deliberately excludes password-strength scoring, guessability or
dictionary analysis, compromised-password screening, authentication, hashing,
and language bindings. It does not promise secure erasure of password text and
has not received an independent security audit.

[0.1.0]: https://github.com/Timi-Nd/PassText-Core/releases/tag/v0.1.0
