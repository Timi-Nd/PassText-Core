# Security

Passwords are handled as sensitive data.

## Mandatory safeguards

- Never log raw passwords.
- Never include password content in exceptions or public error text.
- Keep `repr()` output metadata-only.
- Keep `as_dict()` output metadata-only.
- Reject automatic serialization of prepared password objects.
- Treat `PreparedPassword` as a controlled in-memory analysis artifact, not a general-purpose credential container.

## Actual password lifetime

A `PreparedPassword` instance intentionally retains references to the original text and the normalized/processed text while the object is alive so downstream analysis modules can inspect the prepared representation without reprocessing the input.

This design is intentional and limited to the object lifetime. It is not a claim that password values are deleted immediately or that Python can guarantee secure zeroization of immutable strings.

## Known limitations

Python immutable strings are not reliably zeroized from memory after use. Once references are released, the interpreter may reclaim them according to normal garbage-collection behaviour, but the library cannot guarantee that a process memory dump or debugger will never expose a value that was previously live.

This library therefore focuses on the practical security controls it can provide:

- avoiding unnecessary copies where possible
- preventing accidental disclosure through normal diagnostics and serialization paths
- keeping secret-bearing fields out of public metadata and exception text
- documenting the remaining memory and traceback limitations honestly

The project does not claim complete in-memory confidentiality against arbitrary Python introspection or process-level memory acquisition.
