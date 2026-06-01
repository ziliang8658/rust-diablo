# Security Policy

Rust Diablo parses legacy binary game formats and archive files. Malformed files can expose parser bugs, panics, excessive allocation, or unsafe FFI behavior.

## Reporting A Vulnerability

Please open a private security advisory on GitHub if available, or contact the maintainer through the repository owner profile.

Include:

- A minimal reproduction case.
- The affected command, test, or example.
- Whether the input file is safe to share publicly.
- Expected behavior and actual behavior.

Do not attach proprietary Diablo assets or extracted commercial resources to public issues.

## Scope

Security-sensitive areas include:

- MPQ archive parsing and decompression.
- PCX, CLX, CL2, CEL, TRN, MIN, TIL, and SOL parsers.
- FFI bindings and unsafe Rust.
- Asset loading paths and file extraction tools.

