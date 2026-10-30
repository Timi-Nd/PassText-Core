# Unicode and normalization compatibility

## Units are explicit

PassText Core distinguishes three concepts:

1. UTF-8 bytes (`str::len()` in Rust)
2. Unicode scalar values (`str::chars()`)
3. UTF-8 byte offsets at scalar boundaries

It does not count grapheme clusters. For example, `e` followed by U+0301 is two
scalar values even though it is often displayed as one grapheme. Emoji joined
with U+200D similarly remain a sequence of scalars.

Index spans are half-open scalar ranges `[start, end)`. Conversion to a byte
offset scans UTF-8 boundaries; an offset in the middle of an encoded scalar is
rejected rather than rounded or decoded lossily.

## Strict external decoding

Every Rust `str` is already valid UTF-8 and cannot contain an isolated UTF-16
surrogate. External bytes are decoded strictly by `process_utf8`. Integer
code-point arrays are preflighted by `process_codepoints`; values outside the
Unicode scalar range, including U+D800 through U+DFFF, fail before a `String`
is built.

No API performs lossy replacement with U+FFFD.

## Normalization

`NormalizationMode::None` preserves input exactly. `NormalizationMode::Nfc`
applies canonical NFC normalization and records original and processed lengths
separately. Normalization is never implicit.

Rust normalization is provided by the exactly pinned `unicode-normalization`
0.1.24 dependency, which embeds Unicode data 16.0.0. The verified reference
environment is CPython 3.14.6, whose `unicodedata.unidata_version` is also
16.0.0. A unit test asserts the Rust table version, so a dependency change
cannot silently change NFC behavior without a corresponding review.

The Python package supports older interpreter versions whose bundled Unicode
database can be older than 16.0.0. Inputs involving characters whose
normalization properties changed or were first assigned in a later database
can therefore differ on those interpreters. This is a version boundary rather
than a UTF-8 or indexing difference.

The shared portability vectors use long-stable characters and assert exact NFC
output in both implementations. Passing them establishes compatibility for the
declared vector set, not an unsupported promise that every scalar introduced by
every Unicode version normalizes identically. A release should keep the Rust
table assertion and `Cargo.lock` synchronized and add a cross-language vector
before depending on a newly assigned or version-sensitive character.

## Supported cases

The contract covers ASCII, accented text, combining marks, multiple writing
systems, emoji and zero-width-joiner sequences, whitespace, and empty strings.
Whitespace is not trimmed and case is not changed.

