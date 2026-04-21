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
input-hash: "47125c0"
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

# BC-2.01.007: Claroty Bearer Token Auth with Polymorphic ID Handling

## Description

The Claroty xDome adapter authenticates via bearer token and uses Claroty's POST-for-read pattern (POST requests for read operations). A key implementation detail is polymorphic ID normalization: Claroty returns IDs inconsistently as either JSON numbers (`12345`) or JSON strings (`"12345"`), which the adapter normalizes to a `PolymorphicId` enum for deterministic cursor comparison and deduplication. Cursor arity varies by source: 2-tuple for most, 3-tuple where required.

## Preconditions
- Claroty xDome sensor is configured with a bearer token credential
- The target data source is one of the 9 Claroty endpoints (alerts, ot_activity_events, audit_logs, device_alert_relations, device_vulnerability_relations, servers, sites, devices, vulnerabilities)

## Postconditions
- All Claroty API requests use the bearer token in the Authorization header
- Claroty's POST-for-read pattern is followed (POST requests used for read operations)
- Polymorphic ID fields (JSON number `12345` or JSON string `"12345"`) are normalized to a `PolymorphicId` enum
- Cursor arity matches the source: 2-tuple for most sources, 3-tuple for sources requiring it

## Invariants
- DI-012: Sealed auth trait
- DI-001: Cursor forward progress across polymorphic ID types

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::Sensor` | Bearer token rejected (HTTP 401) | `category: "authentication"`, suggestion: "Verify Claroty API token in credential store" |
| `PrismError::Sensor` | Claroty API returns HTTP 500 during paginated query | Return pages already fetched as partial result; cursor advances to last successful page |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-010 | Same ID appears as JSON number `12345` and string `"12345"` in the same response | `PolymorphicId` enum normalizes both to string for cursor comparison and deduplication; both representations treated as equivalent |
| EC-01-010 | Claroty API changes field name or nesting structure in a minor version | Adapter logs unmapped fields as warnings; preserves them in `raw_extensions`; does not fail on unknown fields |

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.01.007-001 | Claroty alerts endpoint with bearer token; IDs as JSON numbers | All records fetched; IDs normalized to `PolymorphicId`; 2-tuple cursor set |
| TV-BC-2.01.007-002 | Same response with IDs as JSON strings | Normalized identically to number form; cursor comparison treats as equivalent |
| TV-BC-2.01.007-003 | HTTP 401 bearer token rejection | `PrismError::Sensor` with `category: "authentication"` and Claroty token store suggestion |
| TV-BC-2.01.007-004 | HTTP 500 mid-pagination | Previously fetched pages returned as partial result; cursor at last successful page |

## Verification Properties

| VP | Verification Aspect |
|----|---------------------|
| (none) | No VP directly verifies this BC — see VP-INDEX.md for full map |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 |
| L2 Invariants | DI-001, DI-012 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added inputs/input-hash/traces_to/extracted_from frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors; added ## Verification Properties; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract. |
