# Architecture and ownership

The crate is split into small modules with one responsibility each:

- configuration and normalization mode
- typed processing errors
- Unicode validation and measurement
- NFC normalization
- immutable safe metadata
- prepared password representation
- scalar/UTF-8 indexing
- validation and orchestration

The public crate root re-exports the supported surface; internal helpers remain
private so that validation cannot be bypassed accidentally.

## Processing flow

1. Establish a validated immutable configuration.
2. Count original Unicode scalars and enforce its configured limit.
3. Enforce strict decoding at an external input boundary and measure UTF-8
   bytes.
4. Enforce the original byte limit.
5. Apply only the configured normalization mode.
6. Measure and enforce processed limits.
7. Construct immutable metadata and a prepared representation.

The order mirrors the Python behavioral reference where Rust can represent the
same input. Rust `&str` removes the Python-only runtime type and surrogate
states; byte and integer-array entry points preserve explicit failure behavior
at those boundaries.

## Allocation model

Borrowed `&str` input is retained by reference. Owned `String` input can be
moved into processing. The prepared representation distinguishes “processed is
the original” from an owned normalized result, avoiding a self-reference and an
otherwise unnecessary second copy. Counts and ASCII detection are linear in
the number of scalars, with constant auxiliary space; changed NFC output
requires linear output storage.

Prepared values and configuration are immutable after construction. There is no
global cache or mutable processing state, so independent calls can run safely
on different threads when their inputs satisfy Rust's normal sharing rules.

## Portability

The core uses stable Rust and has no operating-system service dependency. This
makes it suitable for native services and a future WebAssembly consumer. The
crate intentionally contains no Python binding or JavaScript/Wasm wrapper; such
interfaces should decode their host values strictly and call these core entry
points.

