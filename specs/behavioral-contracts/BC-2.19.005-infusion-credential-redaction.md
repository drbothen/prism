---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-16T12:00:00
phase: 3-patch
origin: greenfield
subsystem: "SS-19"
capability: "CAP-031"
lifecycle_status: active
introduced: cycle-1
---

# BC-2.19.005: Infusion Credentials Are Never Logged or Included in Error Messages

## Description

Credential values referenced in `[infusion.credentials]` sections of `.infusion.toml`
specs are treated as secrets and MUST NOT appear in any log output (at any level,
including TRACE), any error message returned to callers, or any MCP response. The
credential field NAME may appear (for diagnostics), but the resolved VALUE is always
redacted. This is INV-INFUSE-005.

## Preconditions

- An `.infusion.toml` spec has `[infusion.credentials]` entries (e.g., API keys for
  plugin-backed infusions)
- The spec is being loaded, hot-reloaded, or used during query execution
- An error occurs (spec validation failure, source lookup failure, etc.)

## Postconditions

- Error messages reference credential FIELD NAMES only, not values:
  - `"Credential 'maxmind_api_key' for infusion 'geoip' could not be resolved."` — CORRECT
  - `"Credential 'maxmind_api_key' = 'akJ3mN...' could not be resolved."` — PROHIBITED
- `tracing` log output at all levels (ERROR, WARN, INFO, DEBUG, TRACE) contains no credential values
- MCP responses do not include credential values in any field
- Debug representations of `InfusionSpec` struct are configured to redact credential
  fields (via `#[derive(Debug)]` with `#[debug = "<redacted>"]` or equivalent)

## Invariants

- INV-INFUSE-005: Infusion credentials are never logged or included in error messages
- This invariant applies to ALL log levels — not just ERROR
- Credential values resolved from env vars or keyring at source-call time are also
  subject to this invariant (they must not appear in any log following resolution)
- The invariant applies equally during development/test runs — no credential bypass in
  non-production modes

## Error Cases

| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-INFUSE-005` | Credential cannot be resolved (env var missing, keyring unavailable) | Error: `"Credential '{field_name}' for infusion '{infusion_id}' could not be resolved. Ensure '{env_var_name}' is set."` — value never included |
| — | Credential resolved but source call fails (e.g., bad API key rejection) | Error includes HTTP status and endpoint URL; credential value NOT included |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-19-017 | `RUST_LOG=trace` tracing level enabled in development | Credential values still redacted at TRACE level; no exceptions for debug builds |
| EC-19-018 | Infusion spec serialized for `list_infusions` MCP tool | Credential section shows field names only with `"<redacted>"` values |
| EC-19-019 | Audit log entry for infusion source call failure | Field name referenced; HTTP status referenced; credential value absent |

## Related BCs

- BC-2.03.007 — Secret Redaction in Logs, Errors, and MCP Responses (sensor credentials; same policy)
- BC-2.18.007 — Action Credential Opaque Reference (same policy for action credentials)
- BC-2.05.003 — Credential Values Are Never Present in Audit Entries (audit-specific enforcement)

## Architecture Anchors

- AD-017: AI-opaque credential management
- AD-020: Infusions — credential handling
- `specs/architecture/infusions.md` — credential references, INV-INFUSE-005
- `specs/architecture/security-architecture.md` — secret redaction policy

## Story Anchor

S-1.14 — prism-spec-engine: Infusion Spec Loading and UDF Registration (INV-INFUSE-005, AC-6)

## VP Anchors

Integration test: `tests/infusion_tests.rs` — "Verify infusion spec with credentials: when logged or error returned, no credential values appear in any log output or error message."

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-031 |
| Story Invariant | INV-INFUSE-005 |
| ADR | AD-017, AD-020 |
| Story | S-1.14 |
| Priority | P0 |
