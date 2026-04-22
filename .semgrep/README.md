# .semgrep/ — Prism SAST Rules

TODO: S-0.02 stub — placeholder directory for Semgrep rules.

## Rules

- `credential-handling.yml` — detects credentials stored as plain `String` and potential credential logging
- `unsafe-patterns.yml` — detects `unsafe` blocks without explicit `#[allow(unsafe_code)]` and `.unwrap()` in library crates

Run with: `semgrep --config .semgrep/ .`
