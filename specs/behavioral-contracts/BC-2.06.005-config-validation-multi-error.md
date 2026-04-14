---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "Client Configuration"
capability: "CAP-009"
---

# BC-2.06.005: Configuration Validation Reports All Errors in One Pass

## Preconditions
- TOML configuration has been parsed into an unvalidated structure
- The config validator is checking all clients, sensors, credentials, and capabilities

## Postconditions
- Validation collects ALL errors before reporting, rather than failing on the first error
- The error output is a list of individual validation errors, each including:
  - The exact TOML path of the invalid field (e.g., `clients.acme.sensors.crowdstrike.api_base`)
  - A human-readable description of the problem
  - A suggestion for resolution
- If any validation errors exist, Prism exits with a non-zero exit code and prints the full error list
- If no validation errors exist, Prism proceeds to start normally

## Invariants
- None specific -- this is a usability contract for configuration experience

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Multiple errors | Client A has an invalid URL AND Client B has a missing credential ref AND defaults have an invalid capability path | All three errors are reported in a single output; the operator can fix all at once |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-06-007 | Config has 50+ validation errors (e.g., a template was copy-pasted without customization) | All 50+ errors are reported; no truncation. The first few errors should be the most actionable. |
| EC-06-008 | A single field has multiple problems (e.g., `api_base` is both empty and not a valid URL) | Each distinct problem is reported as a separate error entry for that field |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 |
| L2 Invariants | -- |
| Priority | P0 |
