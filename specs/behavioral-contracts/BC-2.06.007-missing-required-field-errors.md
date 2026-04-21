---
document_type: behavioral-contract
level: L3
version: "1.1"
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
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "e5de7f9"
traces_to: ["CAP-009"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.06.007: Missing Required Fields Produce Actionable Error Messages

## Description

When a required TOML field is absent from a client or sensor configuration section, the
validation error message includes the exact TOML path of the missing field, a "Required
field missing" description, and a suggestion showing the expected format with an example
value. Required fields per sensor are `api_base` and `credential_ref`. The messages are
self-contained — the operator should not need to consult documentation to fix the error.

Absent fields and empty-string fields are treated distinctly: an empty string produces
"field is empty" rather than "field missing". All errors are collected and reported in one
pass per BC-2.06.005.

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

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vectors for BC-2.06.007.

| Scenario | Missing Field | Expected Error Message Fragment |
|----------|-------------|--------------------------------|
| Missing `api_base` | `clients.acme.sensors.crowdstrike.api_base` absent | "Required field missing: clients.acme.sensors.crowdstrike.api_base. Expected: api_base = ..." |
| Missing `credential_ref` | `clients.acme.sensors.crowdstrike.credential_ref` absent | "Required field missing: clients.acme.sensors.crowdstrike.credential_ref. Expected: credential_ref = ..." |
| Empty string | `api_base = ""` | "field is empty: clients.acme.sensors.crowdstrike.api_base" |
| Both fields missing | No `api_base` and no `credential_ref` | Both errors reported in one pass (per BC-2.06.005) |

## Verification Properties

No VPs in VP-INDEX v1.5 directly verify missing-field error message quality. Placeholder for future VP.

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 |
| L2 Invariants | -- |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
