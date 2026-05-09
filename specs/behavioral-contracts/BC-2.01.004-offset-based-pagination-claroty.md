---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-01"
capability: "CAP-001"
lifecycle_status: active
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "76729b7"
traces_to: ["CAP-001"]
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

# BC-2.01.004: Offset-Based Hybrid Pagination for Claroty Audit Logs

## Description

The Claroty xDome `audit_logs` data source does not support cursor-based pagination, so the Claroty adapter uses offset-based mechanics with a composite `(timestamp, offset)` cursor to maintain forward progress. This hybrid cursor prevents regression while accommodating the limitations of offset-based APIs, including the possibility of duplicate records during concurrent insertions.

## Preconditions
- The query targets the Claroty xDome `audit_logs` data source
- The Claroty adapter is initialized with valid bearer token credentials

## Postconditions
- Pagination uses offset-based mechanics (Claroty audit_log API does not support cursor-based pagination)
- The hybrid cursor combines a timestamp component with an offset count
- Records are returned in the order provided by the Claroty API
- Forward-only progress is maintained via the composite (timestamp, offset) cursor

## Invariants
- DI-001: Cursor forward progress -- composite (timestamp, offset) cursor never regresses

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::Sensor` | Claroty API returns HTTP 400 for invalid offset | Structured error with `category: "api_contract"` and the rejected offset value |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-01-005 | Offset exceeds total record count | Claroty returns empty page; pagination halts with `has_more: false` |
| EC-01-006 | New audit log records inserted during paginated traversal causing offset drift | Accepted as a known limitation of offset pagination; duplicate records possible, deduplicated by record ID at the adapter layer |

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.01.004-001 | First page fetch for Claroty audit_logs with offset=0 | Records returned; composite cursor `(timestamp, offset)` advanced; `has_more: true` |
| TV-BC-2.01.004-002 | Offset equals total record count | Empty page returned; `has_more: false`; pagination halts |
| TV-BC-2.01.004-003 | HTTP 400 from Claroty for invalid offset | `PrismError::Sensor` with `category: "api_contract"` and rejected offset value |
| TV-BC-2.01.004-004 | New records inserted mid-traversal | Possible duplicates; adapter deduplicates by record ID |

## Verification Properties

| VP | Verification Aspect |
|----|---------------------|
| (none) | No VP directly verifies this BC — see VP-INDEX.md for full map |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 |
| L2 Invariants | DI-001 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added inputs/input-hash/traces_to/extracted_from frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors; added ## Verification Properties; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract. |
