# PassText

PassText is a Python reference implementation for password text preparation and Unicode-safe processing.

It focuses on input validation, Unicode correctness, normalization, metadata extraction, safe indexing, and prepared-password representation. It intentionally does not score password strength or implement authentication policy.

## Current status

This project has completed the architecture, validation, and Unicode-processing milestones. It validates string inputs, enforces processing limits, applies explicit NFC normalization, preserves metadata without exposing secret values, and is covered by unit tests.

## Installation

```bash
python -m pip install -e '.[dev]'
```

## Quick usage

```python
from passtext import PassTextConfig, process_password

config = PassTextConfig(max_utf8_bytes=4096, max_codepoints=1024, normalization="none")
result = process_password("Example123!", config=config)
print(result.metadata.as_dict())
```

## Testing

```bash
pytest -q
```

## Benchmarking

```bash
python benchmarks/benchmark_password_processing.py
```

This script measures representative ASCII, Unicode, combining-mark, and near-limit workloads without active tracing overhead during latency measurements. It records a machine-readable JSON payload in [benchmarks/output.txt](benchmarks/output.txt) alongside console output.

## Documentation

See [docs/architecture.md](docs/architecture.md), [docs/api.md](docs/api.md), [docs/unicode.md](docs/unicode.md), [docs/security.md](docs/security.md), [docs/performance.md](docs/performance.md), [docs/rust-portability.md](docs/rust-portability.md), [docs/decisions/adr-001-immutable-config.md](docs/decisions/adr-001-immutable-config.md), and [docs/decisions/adr-002-safe-indexing.md](docs/decisions/adr-002-safe-indexing.md).

Portable cross-language fixtures are stored in [tests/fixtures/rust_portability_vectors.json](tests/fixtures/rust_portability_vectors.json).

## Security note

Passwords are treated as sensitive data. The library never logs raw password contents and hides them from repr output.
