---
document_type: behavioral-contract
level: L3
version: "1.4"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "c36ec87"
traces_to: ["CAP-008"]
extracted_from: ".factory/specs/prd.md"
origin: greenfield
subsystem: "SS-08"
capability: "CAP-008"
lifecycle_status: active
introduced: cycle-1
modified: 2026-06-23
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.08.004: Last Successful Query Timestamp Per Sensor Per Client

## Description

Each sensor adapter tracks the timestamp of its most recent HTTP 2xx response, scoped to `(client_id, sensor_id)` per DI-008. The timestamp is persisted to RocksDB `StorageDomain::Default` under a known key prefix and survives server restarts; it is null only if no successful query has ever been made for that `(client_id, sensor_id)` pair. The health response exposes this timestamp so the agent can assess data freshness.

## Preconditions
- A valid `client_id` and `sensor_id` are provided
- The sensor adapter tracks the timestamp of its most recent successful API call

## Postconditions

1. The health response includes `last_successful_query_at: Option<DateTime<Utc>>`; the timestamp reflects the most recent HTTP 2xx response from the sensor API for this `(client_id, sensor_id)` pair, regardless of whether it occurred in the current server session or a prior one.
2. After a server restart, `last_successful_query` returns the persisted timestamp from before the restart (NOT `None`) for any `(client_id, sensor_id)` pair that had at least one successful query prior to the restart. The timestamp is persisted in RocksDB `StorageDomain::Default` under a known key prefix and survives restarts.
3. If no successful query has ever been recorded for this `(client_id, sensor_id)` pair (neither in the current session nor in any persisted prior session), the field is `null`.

## Invariants
- DI-008: Client data separation -- timestamp is scoped to the specific (client_id, sensor_id) pair

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| None | No successful queries in this session | `last_successful_query_at: null` with `status: "no_successful_queries"` |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-08-008 | Prism just started, no queries have EVER been made for this pair | `last_successful_query_at: null`; health check still returns valid status for other fields |
| EC-08-009 | Most recent query succeeded but a subsequent health check runs minutes later | Timestamp reflects the query time, not the health check time |
| EC-08-010 | Prism restarted after a prior successful query for this `(client_id, sensor_id)` pair | `last_successful_query_at` returns the persisted timestamp from before the restart (NOT `null`) — postcondition 2 |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Health check after one successful query | `last_successful_query_at` equals timestamp of that query (not health-check time) | happy-path |
| Health check immediately after Prism startup (no prior queries ever recorded for this pair) | `last_successful_query_at: null` | edge-case |
| Health check after Prism restart (prior session had a successful query for this pair) | `last_successful_query_at` equals the persisted timestamp from before the restart (NOT `null`) | edge-case (postcondition 2) |

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
| Stories | S-5.04 (implements timestamp persistence + restart-survival; AC-5 and EC-006 trace to postcondition 2) |
| Capability Anchor Justification | CAP-008 ("Track last-successful-query timestamp per sensor per client") per capabilities.md §CAP-008 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.4 | S-5.04-LP3P3-option-a-ratify | 2026-06-23 | product-owner | **Persistence ratified per human Option-A decision 2026-06-23; closes F-S504-LP3P3-HIGH-001 + F-S504-LP3P3-MED-001.** §Description: replaced "in-memory only (not persisted across restarts)" with RocksDB `StorageDomain::Default` persistence claim. §Postconditions: replaced 4 unnumbered postconditions with 3 explicitly-numbered postconditions; postcondition 2 now asserts restart-survival (RocksDB persistence), providing the real anchor for AC-5 and EC-006 in S-5.04 (phantom "postcondition 2" finding resolved). §Canonical Test Vectors: replaced "restart → `null` (in-memory only, not persisted)" vector with "restart → persisted timestamp returned (postcondition 2)". §Edge Cases: updated EC-08-008 to "no prior queries ever"; added EC-08-010 (restart case, references postcondition 2). §Traceability: added Stories row (S-5.04) and Capability Anchor Justification row. |
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial draft |
