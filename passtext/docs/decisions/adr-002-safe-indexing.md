# ADR 002: Half-open code-point indexing with explicit UTF-8 boundaries

Date: 2026-10-29

## Status

Accepted

## Context

Downstream password analysis will eventually need to identify spans such as numeric sequences or repeated character regions. A safe indexing scheme must preserve valid boundaries and avoid silent confusion between code-point indices and UTF-8 byte offsets.

## Decision

PassText uses half-open spans in the form `[start, end)` based on Unicode code points. UTF-8 offsets are only used when explicitly requested, and any offset that lands mid-code-point is rejected with `ValueError`.

## Consequences

- Indexing semantics are explicit and stable
- Normalization is prevented from silently invalidating mappings
- Future Rust implementations can mirror the same boundaries using borrowed slices and explicit offsets
- Cross-language fixtures can rely on consistent code-point semantics
