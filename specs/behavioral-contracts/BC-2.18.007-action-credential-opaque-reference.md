---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-16T12:00:00
phase: 2-patch
origin: greenfield
subsystem: "SS-18"
capability: "CAP-033"
lifecycle_status: active
introduced: cycle-1
modified: 2026-04-20
deprecated: ~
deprecated_by: ~
replacement: ~
retired: ~
removed: ~
removal_reason: ~
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "76729b7"
traces_to: ["CAP-033"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.18.007: Action Credentials Must Use AI-Opaque Reference Model — No Inline Values (E-ACTION-001)

## Description

All credentials referenced in `.action.toml` files (webhook URLs, API keys, SMTP
passwords, etc.) MUST be declared using the AI-opaque reference model:
`{ source = "env", key = "SLACK_WEBHOOK_URL" }` or equivalent reference forms.
Inline credential values (plain string values in TOML) are rejected at spec load time
with error `E-ACTION-001`. This prevents credential values from entering the AI context
via MCP tool responses or error messages. This is INV-ACTION-007.

## Preconditions

- The `.action.toml` spec loader is parsing a spec file
- A credential field (e.g., `routing_key`, `webhook_url`, `smtp_password`) contains
  an inline string value (e.g., `routing_key = "my-secret-123"`)

## Postconditions

- The spec loader detects the inline value on the credential field
- The spec is rejected: NOT registered in `ActionRegistry`
- Error returned: `E-ACTION-001`:
  `"Credential '{field}' in action '{id}' must use a reference (source = 'env', key = 'KEY_NAME'), not an inline value."`
- The error is logged at `ERROR` level
- Other action specs in the directory continue loading (this spec's failure is isolated)
- The inline value is NEVER logged or included in any error message or audit entry
  (the value itself is treated as sensitive; only the field name is referenced)

## Invariants

- INV-ACTION-007: Action credentials MUST use AI-opaque reference model — never inline values in TOML
- The validation runs at spec load time (startup + hot reload), not at delivery time
- If a previously-valid spec is hot-reloaded with an inline credential added, it is rejected;
  the previously-registered valid spec is retained (CI-002 hot reload invariant)
- Inline value detection applies to any field listed under `[action.credentials]` in the spec

## Error Conditions

| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-ACTION-001` | Inline credential value detected at load time | Spec rejected; error logged (field name only, not value); other specs continue |
| — | Credential reference key does not exist in environment at delivery time | Delivery fails with `E-ACTION-009: "Credential '{field}' references environment variable '{key}' which is not set."` |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-18-023 | `routing_key = { source = "env", key = "PD_KEY" }` | Valid reference form; spec accepted; `E-ACTION-001` NOT raised |
| EC-18-024 | `routing_key = ""` (empty inline value) | Rejected with `E-ACTION-001`; empty string is still an inline value |
| EC-18-025 | Credential field is entirely absent from spec | Not an `E-ACTION-001` error; may be a missing-required-field error (`E-ACTION-010`) depending on destination type |
| EC-18-026 | Hot reload replaces valid spec with inline-credential version | Loader rejects new version; old registered spec retained; `ERROR` log |

## Canonical Test Vectors

| ID | Input | Expected Output | Notes |
|----|-------|----------------|-------|
| TV-18-007-happy | `routing_key = { source = "env", key = "PD_KEY" }` | Spec accepted; no `E-ACTION-001` | EC-18-023 |
| TV-18-007-inline | `routing_key = "my-secret-123"` | `E-ACTION-001`; spec not registered; value not in log | Baseline rejection |
| TV-18-007-empty | `routing_key = ""` | `E-ACTION-001`; empty string is still inline | EC-18-024 |
| TV-18-007-hotreload | Hot reload adds inline credential to valid spec | Loader rejects; old spec retained | EC-18-026 |

## Verification Properties

| VP ID | Description | Verification Method |
|-------|-------------|---------------------|
| VP-046 | For any ActionSpec where a credential field contains an inline string value (non-reference form), `validate_credential_fields()` returns Err(E-ACTION-001); the error message contains the field name and does not contain the field value; reference-form values always return Ok | Proptest |

## Related BCs

- BC-2.03.009 — resolve_secret() for env var and file-based credential references (shared pattern)
- BC-2.03.007 — Secret Redaction (inline values, if accidentally present, must not appear in logs)
- BC-2.17.002 — Plugin Sandbox (plugin action destinations also use credential references, not inline values)

## Architecture Anchors

- AD-017: AI-opaque credential management — credential values never transit AI context
- AD-021: Actions — credential reference model
- `specs/architecture/actions.md` — `E-ACTION-001`, credential validation in loader
- `specs/architecture/security-architecture.md` — AI-opaque model
- S-4.08 Task 2: `action/loader.rs` — credential inline value detection
- S-4.08 AC-9: "routing_key with inline value → E-ACTION-001 and action NOT registered"

## Story Anchor

S-4.08 — prism-operations: Action Delivery Framework (INV-ACTION-007, AC-9)

## VP Anchors

Integration test: `tests/action_tests.rs` — "Load action with inline credential value → verify `E-ACTION-001` rejection."

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-033 |
| Story Invariant | INV-ACTION-007 |
| ADR | AD-017, AD-021 |
| Story | S-4.08 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Resolved VP-TBD placeholder per decision matrix (ADD-VP-046); normalized changelog schema to canonical 5-col form. |
| 1.1 | Wave-6-pre-build-sweep | 2026-04-20 | product-owner | Added frontmatter (inputs, input-hash, traces_to, extracted_from, lifecycle fields); renamed Error Cases → Error Conditions; added Canonical Test Vectors, Verification Properties, Changelog |
| 1.0 | Phase-2 | 2026-04-16 | product-owner | Initial contract |
