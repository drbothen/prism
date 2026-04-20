---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-04-14T07:00:00
phase: 1a
origin: greenfield
subsystem: "SS-11"
capability: "CAP-015"
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
input-hash: "365fb25"
traces_to: ["CAP-015"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.11.012: Virtual Fields in Queries — `_sensor`, `_client`, `_source_table`

## Description

Three virtual fields — `_sensor`, `_client`, and `_source_table` — are injected as additional Arrow columns into every materialized RecordBatch, making sensor provenance queryable in all PrismQL modes. The underscore prefix distinguishes them from OCSF data fields. Virtual field predicates participate in scope intersection (same semantics as tool-level `sensors`/`clients` parameters). Virtual fields are string-typed; numeric comparisons are type errors. Their names are verified at build time to not collide with OCSF field names.

## Preconditions
- A PrismQL query references `_sensor`, `_client`, or `_source_table` as field names in filter expressions, WHERE clauses, or pipe stages

## Postconditions
- Three virtual fields are available in all PrismQL query modes:
  - **`_sensor`**: The sensor type that produced the event (values: `"crowdstrike"`, `"cyberint"`, `"claroty"`, `"armis"` for external tables; `"prism"` for internal RocksDB-backed tables). Underscore prefix distinguishes virtual fields from OCSF data fields (per BC-2.15.009).
  - **`_client`**: The client ID (TenantId value) that owns the sensor instance or the Prism record.
  - **`_source_table`**: The data source table name within the sensor (e.g., `"alerts"`, `"devices"`, `"vulnerabilities"` for external tables; `"alerts"`, `"cases"`, `"rules"`, `"schedules"`, `"diff_results"`, `"audit"`, `"aliases"` for internal tables). Injected during OCSF normalization (external) or during internal table materialization (internal).
- Virtual fields are usable in all positions where regular OCSF fields are usable:
  - Filter mode: `_sensor = "crowdstrike" AND severity >= "high"`
  - SQL mode: `SELECT _sensor, count(*) FROM events GROUP BY _sensor`
  - Pipe mode: `| where _sensor = "claroty" | stats count by _client`
- Virtual fields are implemented as additional Arrow columns in the materialized RecordBatch
- Virtual field predicates participate in scope intersection:
  - `_sensor = "crowdstrike"` in the query intersects with `sensors` tool parameter
  - `_client = "acme"` in the query intersects with `clients` tool parameter
- Virtual field values are strings; comparison operators (`=`, `!=`, `in`) are supported; numeric comparisons (`>`, `<`) on virtual fields are type errors

## Invariants
- Virtual fields are not part of the OCSF schema; they are Prism-specific metadata fields
- Virtual field names cannot collide with OCSF field names (verified at build time against the OCSF proto schema)

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-QUERY-002` | Numeric comparison on virtual field (e.g., `_sensor > "armis"`) | Type error: "Field '_sensor' is a string virtual field. Use = or != for comparison." |
| `E-QUERY-002` | Invalid sensor name in predicate | Error with list of valid sensor names (including `"prism"` for internal tables) |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-11-029 | `_sensor = "unknown_sensor"` | No records match; empty result set (not an error -- the filter simply excludes everything) |
| EC-11-030 | `SELECT _sensor, _client, _source_table FROM events` | Valid projection; returns only virtual fields for each event |
| EC-11-031 | Virtual field used in `GROUP BY` | Valid; DataFusion groups by the string column normally |

## Canonical Test Vectors

> See `.factory/specs/prd-supplements/test-vectors.md` for the canonical test vector tables.

| Input | Expected Output | Category |
|-------|----------------|----------|
| `SELECT _sensor, count(*) FROM events GROUP BY _sensor` | One row per sensor type with count | happy-path |
| `_sensor = 'crowdstrike' AND severity = 'critical'` | Events from CrowdStrike with critical severity only | happy-path |
| `_sensor > 'armis'` | `Err(E-QUERY-002)` numeric comparison on string virtual field | error |
| `_sensor = 'unknown_sensor'` | Empty result set; not an error | edge-case |

## Verification Properties

| VP ID | Property | Proof Method |
|-------|----------|-------------|
| VP-021 | PrismQL parser: never panics on arbitrary input | fuzz |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-015 |
| Related BCs | BC-2.11.005 (virtual fields injected during materialization), BC-2.11.011 (scope intersection) |
| Priority | P0 |

## Changelog
| Version | Date | Burst | Change |
|---------|------|-------|--------|
| 1.0 | 2026-04-14 | cycle-1 | Initial contract |
| 1.1 | 2026-04-20 | pre-build-sweep | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
