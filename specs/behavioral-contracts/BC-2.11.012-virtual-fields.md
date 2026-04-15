---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T07:00:00
phase: 1a
origin: greenfield
subsystem: "Query Engine & Aliases"
capability: "CAP-015"
---

# BC-2.11.012: Virtual Fields in Queries -- `sensor`, `client_id`, `source`

## Preconditions
- An AxiQL query references `sensor`, `client_id`, or `source` as field names in filter expressions, WHERE clauses, or pipe stages

## Postconditions
- Three virtual fields are available in all AxiQL query modes:
  - **`sensor`**: The sensor type that produced the event (values: `"crowdstrike"`, `"cyberint"`, `"claroty"`, `"armis"`). Injected during OCSF normalization.
  - **`client_id`**: The client ID that owns the sensor instance. Injected during materialization.
  - **`source`**: The data source within the sensor (e.g., `"alerts"`, `"devices"`, `"vulnerabilities"`). Injected during OCSF normalization.
- Virtual fields are usable in all positions where regular OCSF fields are usable:
  - Filter mode: `sensor = "crowdstrike" AND severity >= "high"`
  - SQL mode: `SELECT sensor, count(*) FROM events GROUP BY sensor`
  - Pipe mode: `| where sensor = "claroty" | stats count by client_id`
- Virtual fields are implemented as additional Arrow columns in the materialized RecordBatch
- Virtual field predicates participate in scope intersection:
  - `sensor = "crowdstrike"` in the query intersects with `sensors` tool parameter
  - `client_id = "acme"` in the query intersects with `clients` tool parameter
- Virtual field values are strings; comparison operators (`=`, `!=`, `in`) are supported; numeric comparisons (`>`, `<`) on virtual fields are type errors

## Invariants
- Virtual fields are not part of the OCSF schema; they are Prism-specific metadata fields
- Virtual field names cannot collide with OCSF field names (verified at build time against the OCSF proto schema)

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::QueryType` | Numeric comparison on virtual field (e.g., `sensor > "armis"`) | Type error: "Field 'sensor' is a string virtual field. Use = or != for comparison." |
| `PrismError::QueryType` | Invalid sensor name in predicate | Error with list of valid sensor names |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-11-029 | `sensor = "unknown_sensor"` | No records match; empty result set (not an error -- the filter simply excludes everything) |
| EC-11-030 | `SELECT sensor, client_id, source FROM events` | Valid projection; returns only virtual fields for each event |
| EC-11-031 | Virtual field used in `GROUP BY` | Valid; DataFusion groups by the string column normally |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-015 |
| Related BCs | BC-2.11.005 (virtual fields injected during materialization), BC-2.11.011 (scope intersection) |
| Priority | P0 |
