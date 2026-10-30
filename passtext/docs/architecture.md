# Architecture

PassText follows a small, layered architecture designed for correctness and eventual Rust portability.

## Layers

1. Public API
   - Exposes `PassTextConfig` and `process_password`.
2. Validation
   - Confirms that the input is a `str` and that the value respects processing limits.
3. Unicode helpers
   - Validate UTF-8, measure code points and bytes, and detect non-ASCII characters.
4. Normalization
   - Applies the configured normalization mode without altering the original input in memory.
5. Metadata
   - Produces immutable metadata without exposing processed passwords.
6. Prepared representation
   - Stores safe references and metadata for downstream modules.

## Design principles

- Immutable configuration
- No shared mutable processing state
- Explicit error categories
- No password logging or serialization
- Safe, deterministic behavior

## Future Rust mapping

The Python implementation is intentionally close to the Rust ownership model: a configured input produces immutable metadata and a prepared representation that can map to borrowed UTF-8 slices, explicit ownership boundaries, and structured metadata in the future.
