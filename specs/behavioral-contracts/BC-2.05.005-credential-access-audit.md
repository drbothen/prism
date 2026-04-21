---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-05"
capability: "CAP-007"
lifecycle_status: active
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "abc4070"
traces_to: ["CAP-007"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.05.005: Credential Access Events Are Audit-Logged with Context

## Description

Every credential access through the `CredentialStore` trait — get, set, delete, or list —
emits a structured log event with `event_type: "credential_access"`, the operation type,
`client_id`, `sensor_id`, `credential_name`, result, and timestamp. The credential value
itself is never present in the log event, satisfying DI-002 (credential isolation) and
DI-004 (audit completeness). This provides ISO 27001 access control evidence for all
credential lifecycle operations.

## Preconditions
- A credential is accessed (read, write, delete) via the `CredentialStore` trait
- The access is performed in the context of a specific `client_id` and `sensor_id`

## Postconditions
- A structured log event is emitted recording:
  - `event_type: "credential_access"`
  - `operation` (`"get"`, `"set"`, `"delete"`, `"list"`)
  - `client_id` (the `TenantId` of the credential being accessed)
  - `sensor_id` (the sensor the credential belongs to)
  - `credential_name` (the name, e.g., `"api_key"`, `"client_secret"`)
  - `result` (`"success"` or `"error"` with category)
  - `timestamp` (ISO 8601 UTC)
- The credential value itself is NEVER present in the log event

## Invariants
- DI-002: Credential isolation per client -- credential values never logged
- DI-004: Audit completeness -- all credential access is audit-logged

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Credential not found | `get()` called for a non-existent credential | Log event includes `result: "not_found"` with the credential name and context |
| Backend error | OS keyring locked or file backend I/O failure | Log event includes `result: "error"` with category but no backend-specific secrets |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-05-008 | Credential `list()` operation for a client | Log event records `operation: "list"`, `credential_name: "*"` (wildcard), and the count of credentials returned |
| EC-05-009 | Credential rotation (`set()` overwriting existing) | Log event records `operation: "set"` with no distinction between create and update; the old value is not logged |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vectors for BC-2.05.005.

| Scenario | Operation | Expected Event Fields |
|----------|-----------|----------------------|
| Successful get | `get("api_key")` for `acme/crowdstrike` | `{event_type: "credential_access", operation: "get", client_id: "acme", sensor_id: "crowdstrike", credential_name: "api_key", result: "success"}` — value absent |
| Not found | `get("nonexistent")` | `result: "not_found"` |
| List | `list()` for `acme/crowdstrike` | `{operation: "list", credential_name: "*", result: "success"}` + count |
| Delete | `delete("api_key")` | `{operation: "delete", result: "success"}` |

## Verification Properties

No VPs in VP-INDEX v1.5 directly verify credential access event emission. Placeholder for future VP.

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-007 |
| L2 Invariants | DI-002, DI-004 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
