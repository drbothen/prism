---
document_type: behavioral-contract
level: L3
version: "1.6"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-03"
capability: "CAP-004"
lifecycle_status: active
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "3ff257e"
traces_to: ["CAP-004"]
extracted_from: ".factory/specs/prd.md"
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.03.005: Credential CRUD Operations via MCP Tools (Mutations Require Confirmation Token)

## Description

Credential management is exposed via four MCP tools: `configure_credential_source` (create/update source references), `delete_credential`, `list_credentials`, and `credential_status`. Mutation tools accept only source-type references (`env`, `file`, `vault`, `keyring`) — never raw credential values. Initial creation is immediate and non-destructive; updates and deletions are gated behind the confirmation token flow (BC-2.04.009). `list_credentials` requires a non-null `client_id` to prevent MSSP portfolio disclosure, and always returns metadata only (never values). All operations are audit-logged.

## Preconditions
- The credential management MCP tools are registered (`configure_credential_source`, `delete_credential`, `list_credentials`, `credential_status`)
- The analyst provides a valid `client_id`, `sensor_id`, and `credential_name`
- Credential mutation tools (`configure_credential_source`, `delete_credential`) are subject to feature flag gating under capability path `credential.write`. They follow the hidden-tools pattern (BC-2.04.005): if `credential.write` is denied for ALL configured clients, `configure_credential_source` and `delete_credential` are omitted from `tools/list`. If denied for a specific client, invocation with that `client_id` returns `E-FLAG-001`. `list_credentials` and `credential_status` are read-only tools and are always visible regardless of feature flags.

## Postconditions
- `configure_credential_source` (create): When no credential exists for the given `(client_id, sensor_id, credential_name)` tuple, the credential source reference is created immediately and returns `status: "created"`. No confirmation token is required for initial creation — the operation is non-destructive (nothing is being overwritten). The tool accepts source type references only (`env`, `file`, `vault`, `keyring`); raw credential values are never accepted.
- `configure_credential_source` (update): When a credential source reference already exists, updating is gated behind the confirmation token flow (same as irreversible write operations per BC-2.04.009) — the tool returns a `ConfirmationToken` with `status: "confirmation_required"` and the caller must call `confirm_action` to execute the update. This prevents accidental credential source replacement. The updated value must also be a source type reference, never a raw credential value.
- `delete_credential`: Removes the credential from the backend; idempotent. Deletion is gated behind the confirmation token flow (same as irreversible write operations per BC-2.04.009) — the tool returns a `ConfirmationToken` and the caller must call `confirm_action` to execute the deletion.
- `list_credentials`: Returns all credential entries for a specific client/sensor combination (metadata only, never credential values). Requires a non-null `client_id` — cross-client credential listing (`client_id: null`) is not supported to prevent MSSP client portfolio disclosure. Returns `E-FLAG-006` if `client_id` is null.
- All CRUD operations are audit-logged with client_id, sensor_id, credential_name, and operation type
- Credential values are never included in MCP responses, logs, or error messages

## Invariants
- DI-002: Credential isolation
- DI-004: Audit completeness -- every credential operation produces an audit entry
- Credential values are never included in MCP responses, logs, or error messages

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::InvalidInput` | `credential_name` fails validation | Structured error with the rejected name and the allowed pattern `[a-zA-Z0-9_\-\.]+` |
| `E-FLAG-006` | `client_id` is null | Credential mutation tools (`configure_credential_source`, `delete_credential`) require a non-null `client_id`. Cross-client credential writes are not permitted. Structured error: "Write operation with client_id: null not supported". **Note:** This error case is defense-in-depth for direct callers or schema validation bypass scenarios. Under normal MCP invocation, JSON Schema validation rejects null `client_id` before the tool handler runs. |
| `PrismError::Credential` | Backend write fails | Structured error with backend type and suggestion |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-03-012 | Agent attempts to read a credential value (not just metadata) | No MCP tool exposes credential values; `list_credentials` returns metadata only (backend type, last_modified) but never the credential value itself |
| EC-03-013 | Resolved credential value from backend (env var content, file content, or vault retrieval) contains special characters (newlines, null bytes) | Resolution layer (credential provider) handles arbitrary byte sequences; value integrity preserved through the credential-provider abstraction; raw bytes never enter MCP tool input/output surface |

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.03.005-001 | `configure_credential_source` (create) with no existing credential | Returns `status: "created"` immediately; no confirmation token |
| TV-BC-2.03.005-002 | `configure_credential_source` (update) on existing credential | Returns `ConfirmationToken` with `status: "confirmation_required"` |
| TV-BC-2.03.005-003 | `delete_credential` | Returns `ConfirmationToken`; deletion executes only after `confirm_action` |
| TV-BC-2.03.005-004 | `list_credentials(client_id: null)` | Returns `E-FLAG-006`; cross-client listing rejected |
| TV-BC-2.03.005-005 | `credential_status` for existing credential | Returns metadata (backend type, last_modified); never the value |
| TV-BC-2.03.005-006 | Mutation tool called when `credential.write` denied for all clients | Tool absent from `tools/list` (hidden-tools pattern) |

## Verification Properties

| VP | Verification Aspect |
|----|---------------------|
| VP-011 | Credential name sanitization: rejects path traversal (kani) |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-004, CAP-005 |
| L2 Invariants | DI-002, DI-003, DI-004 |
| Addresses | ADV-5-001 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.6 | pass-71-fix | 2026-04-20 | product-owner | MED-001/MED-002: fixed column swap on pre-build-sweep row (Date was in Burst column); sorted rows to fully descending version order. |
| 1.5 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.4 | pass-61-fix | 2026-04-20 | product-owner | Renumbered duplicate pre-build-sweep Changelog row for version monotonicity (MED-002). |
| 1.3 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added inputs/input-hash/traces_to/extracted_from frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors; added ## Verification Properties. |
| 1.2 | Burst 44 | 2026-04-19 | product-owner | P3P43-A-LOW-001: reframed EC-03-013 under AI-opaque model. Scenario now describes backend-level byte handling during resolution (env var content, file content, vault retrieval) rather than tool-level value acceptance, which is impossible under the source-reference-only model. |
| 1.1 | Burst 43 | 2026-04-19 | product-owner | P3P41-A-HIGH-001: renamed `set_credential` → `configure_credential_source` throughout. Preconditions updated to include `credential_status` in registered tool list. Postconditions rewritten to reflect AI-opaque source-type reference semantics (tool accepts `env`/`file`/`vault`/`keyring` references only, never raw credential values). |
| 1.0 | Phase 1 | 2026-04-14 | product-owner | Initial contract |
