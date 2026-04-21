---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "e5de7f9"
traces_to: ["CAP-008"]
extracted_from: ".factory/specs/prd.md"
origin: greenfield
subsystem: "SS-08"
capability: "CAP-008"
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

# BC-2.08.004: Last Successful Query Timestamp Per Sensor Per Client

## Description

Each sensor adapter tracks the timestamp of its most recent HTTP 2xx response, scoped to `(client_id, sensor_id)` per DI-008. The timestamp is in-memory only (not persisted across restarts) and null if no successful query has occurred in the current session. The health response exposes this timestamp so the agent can assess data freshness.

## Preconditions
- A valid `client_id` and `sensor_id` are provided
- The sensor adapter tracks the timestamp of its most recent successful API call

## Postconditions
- The health response includes `last_successful_query_at: Option<DateTime<Utc>>`
- If no successful query has been made during this session, the field is `null`
- The timestamp reflects the most recent HTTP 2xx response from the sensor API for this (client_id, sensor_id) pair
- The timestamp is stored in memory only (not persisted across restarts)

## Invariants
- DI-008: Client data separation -- timestamp is scoped to the specific (client_id, sensor_id) pair

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| None | No successful queries in this session | `last_successful_query_at: null` with `status: "no_successful_queries"` |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-08-008 | Prism just started, no queries executed yet | `last_successful_query_at: null`; health check still returns valid status for other fields |
| EC-08-009 | Most recent query succeeded but a subsequent health check runs minutes later | Timestamp reflects the query time, not the health check time |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Health check after one successful query | `last_successful_query_at` equals timestamp of that query (not health-check time) | happy-path |
| Health check immediately after Prism startup (no queries yet) | `last_successful_query_at: null` | edge-case |
| Health check after Prism restart (prior session had queries) | `last_successful_query_at: null` (in-memory only, not persisted) | edge-case |

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vector tables.

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (no matching VP) | Timestamp is scoped per (client_id, sensor_id) pair | integration test |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-008 |
| L2 Invariants | DI-008 |
| Priority | P1 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial draft |
