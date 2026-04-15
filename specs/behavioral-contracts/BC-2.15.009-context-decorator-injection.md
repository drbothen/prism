---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "Platform Infrastructure"
capability: "CAP-026"
---

# BC-2.15.009: Context Decorator Injection — Auto-Inject Metadata into All Results

## Preconditions
- A query execution has produced OCSF-normalized results (either ad-hoc or scheduled)
- The Prism configuration and session context are available

## Postconditions
- Every result record is decorated with the following metadata fields (injected as top-level fields in the response, not modifying the OCSF record):
  - `_client_id`: the client ID that owns this record
  - `_client_name`: human-readable client name from TOML config
  - `_sensor_type`: the sensor type that produced this record (e.g., "crowdstrike", "claroty")
  - `_sensor_instance`: the sensor instance identifier (e.g., "us-1")
  - `_analyst_id`: the analyst identifier from the current session context
  - `_query_source`: provenance of the query (e.g., "interactive", "schedule:check_alerts", "pack:incident-response.recent_detections")
  - `_prism_version`: the running Prism version string
- Decorators are prefixed with `_` to distinguish them from OCSF fields
- Decorators are injected during result materialization (after DataFusion execution, before response serialization)
- Decorators are deterministic: the same query context always produces the same decorator values
- Decorators are included in audit log entries and differential results

## Invariants
- Every result record has all decorator fields present (never partial decoration)
- Decorator field names are reserved: query predicates cannot filter on `_` prefixed fields (they are post-query injected)
- Decorators never modify the OCSF record itself (they are envelope metadata)

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| (none) | Decorator injection cannot fail | Missing context values (e.g., no analyst_id in automated mode) produce `null` values, not errors |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-15-033 | Cross-client query produces records from 3 clients | Each record has its own `_client_id` and `_client_name` matching its source |
| EC-15-034 | Scheduled query execution (no analyst session) | `_analyst_id` is null; `_query_source` is "schedule:{schedule_name}" |
| EC-15-035 | Query returns 0 results | No decoration needed; empty result set |
| EC-15-036 | Client name contains unicode characters | Preserved as-is in `_client_name` |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-026 |
| L2 Invariants | DI-008 |
| Priority | P0 |
