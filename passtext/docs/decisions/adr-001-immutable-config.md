# ADR 001: Immutable processing configuration

Date: 2026-10-29

## Status

Accepted

## Context

PassText must be deterministic and safe for repeated independent processing. Password-related evaluations should not share mutable configuration state across calls.

## Decision

Configuration is implemented as a frozen dataclass with explicit validation in `PassTextConfig`.

## Consequences

- Thread-safe by construction within a single processing call
- Easier to reason about in future Rust porting
- Prevents accidental mutation of limits or normalization mode
- Avoids hidden cross-call state
