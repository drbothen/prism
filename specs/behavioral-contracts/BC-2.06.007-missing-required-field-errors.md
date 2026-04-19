---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-06"
capability: "CAP-009"
lifecycle_status: active
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.06.007: Missing Required Fields Produce Actionable Error Messages

## Preconditions
- TOML configuration is being validated
- A required field is absent from a client or sensor configuration section

## Postconditions
- The validation error message includes:
  - The exact TOML path of the missing field (e.g., `clients.acme.sensors.crowdstrike.api_base`)
  - A description: "Required field missing"
  - A suggestion showing the expected format (e.g., `api_base = "https://api.crowdstrike.com"`)
- Required fields per client: `client_id` (implicit from TOML key)
- Required fields per sensor: `api_base`, `credential_ref`
- The error message is sufficient for the operator to fix the config without consulting documentation

## Invariants
- None specific -- this is a usability contract

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Missing `api_base` | Sensor section lacks `api_base` | "Required field missing: clients.{id}.sensors.{sensor}.api_base. Expected: api_base = \"https://api.example.com\"" |
| Missing `credential_ref` | Sensor section lacks `credential_ref` | "Required field missing: clients.{id}.sensors.{sensor}.credential_ref. Expected: credential_ref = \"api_key\"" |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-06-011 | Client section exists but is completely empty (`[clients.acme]` with no fields) | Error messages list all missing required fields for the client, not just the first one |
| EC-06-012 | Field is present but set to empty string (`api_base = ""`) | Treated as missing/invalid, not as present; the error message specifies "field is empty" rather than "field missing" |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 |
| L2 Invariants | -- |
| Priority | P0 |
