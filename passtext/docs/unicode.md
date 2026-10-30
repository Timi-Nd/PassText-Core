# Unicode behavior

PassText distinguishes between:

- Unicode code points: `len(text)`
- UTF-8 bytes: `len(text.encode('utf-8'))`
- Normalized text: configured through `unicodedata.normalize`

## Supported normalization modes

- `none`: preserves the original value exactly
- `nfc`: applies Unicode NFC normalization

## Current limitation

Grapheme cluster counts are intentionally not implemented in this milestone. This is a design choice to keep the first release simple and Rust-portable while still supporting reliable code-point and byte counting.

## Surrogate handling

Python `str` values may contain isolated surrogate code points. PassText rejects these values because they cannot be represented as valid UTF-8 and would otherwise create inconsistent cross-language behavior.
