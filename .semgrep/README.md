# .semgrep — Prism Custom SAST Rules

This directory contains project-specific semgrep rules enforcing Prism's
security policies (see `.factory/specs/` for the authoritative policy sources).

## Rules

- **credential-handling.yml** — Enforces AI-Opaque Credentials policy
  (AD-017). Flags `String` types with names matching credential patterns
  (api_key, secret, token, password, private_key). Error severity.

- **unsafe-patterns.yml** — Flags unsafe Rust patterns beyond clippy coverage.
  Currently stub — real rules land when first crates exist (Wave 0b+).

## Usage

Rules are invoked by the `just check` PR gate via `semgrep --config .semgrep/`.
Local developers: `semgrep --config .semgrep/` from the repo root.

CI: runs as part of the security job in `.github/workflows/ci.yml`.

## Adding rules

1. Create `.semgrep/<category>.yml`
2. Follow existing rule schema (id, message, languages, severity, patterns)
3. Test locally: `semgrep --config .semgrep/ --test`
4. Document the rule here
