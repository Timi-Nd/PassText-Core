# API contract

## Public functions

### `process_password(password, config=None)`

Processes a password-like text value and returns a `PreparedPassword` object.

Inputs:
- `password`: `str` only
- `config`: `PassTextConfig` instance or `None`

Behavior:
- validates the input type
- rejects isolated surrogate code points
- enforces UTF-8 and code-point processing limits
- applies the configured normalization mode
- returns immutable metadata and a prepared representation

Output:
- `PreparedPassword.original_text`
- `PreparedPassword.processed_text`
- `PreparedPassword.metadata`
- `PreparedPassword.config`

### `PassTextConfig`

Configuration is intentionally immutable and explicit. Supported fields are:

- `max_utf8_bytes` — default 4096
- `max_codepoints` — default 1024
- `normalization` — `"none"` or `"nfc"`

## Safe representation rules

Prepared objects do not expose password contents through `repr()`, JSON serialization, logging, or exception text. Only metadata is returned in safe forms.
