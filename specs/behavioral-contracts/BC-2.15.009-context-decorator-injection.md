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
- Result records carry two separate categories of metadata:

  **Virtual fields** (injected pre-DataFusion into Arrow RecordBatch — queryable in WHERE/GROUP BY/ORDER BY, underscore-prefixed):
  - `_sensor`: sensor identifier (e.g., "crowdstrike", "armis", "prism" for internal tables)
  - `_client`: client ID (TenantId value)
  - `_source_table`: specific table name (e.g., "crowdstrike_detections", "prism_alerts")

  **Decorator fields** (injected post-DataFusion into `_meta` response envelope — NOT queryable, no underscore prefix):
  - `client_name`: human-readable client name from TOML config
  - `sensor_instance`: the sensor instance identifier (e.g., "us-1")
  - `analyst_id`: the analyst identifier from the current session context
  - `query_source`: provenance of the query (e.g., "interactive", "schedule:check_alerts", "pack:incident-response.recent_detections")
  - `prism_version`: the running Prism version string

  Response structure:
  ```json
  {
    "events": [
      { "_sensor": "crowdstrike", "_client": "acme", "_source_table": "crowdstrike_detections", "severity_id": 4, "device_hostname": "DESKTOP-X", ... }
    ],
    "_meta": {
      "client_name": "Acme Corp",
      "analyst_id": "joshua",
      "query_source": "interactive",
      "prism_version": "0.1.0",
      "sensor_instance": "us-1"
    }
  }
  ```

- Queryable virtual fields (`_sensor`, `_client`, `_source_table`) are documented in query-engine.md's virtual fields table. Decorator fields (`client_name`, `sensor_instance`, `analyst_id`, `query_source`, `prism_version`) live in the `_meta` envelope and are defined in this BC only.
- Virtual fields are Arrow columns registered in the MemTable schema — they participate in DataFusion execution
- Decorator fields are `_meta` envelope metadata — they are NOT Arrow columns and cannot appear in PrismQL predicates. They have no underscore prefix because they are clearly separated from event data by being in the `_meta` object, not in the event rows.
- Both categories are deterministic: the same query context always produces the same values
- Both categories are included in audit log entries; only virtual fields are included in differential results

## Invariants
- Every response has all decorator fields present in `_meta` (never partial decoration)
- Decorator fields in `_meta` cannot be referenced in PrismQL predicates — they are injected after DataFusion execution. Virtual fields (`_sensor`, `_client`, `_source_table`) CAN be referenced in predicates — they are Arrow columns injected before DataFusion execution (see query-engine.md virtual fields table)
- Decorators never modify the OCSF record itself (they are envelope metadata in `_meta`)

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| (none) | Decorator injection cannot fail | Missing context values (e.g., no analyst_id in automated mode) produce `null` values, not errors |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-15-033 | Cross-client query produces records from 3 clients | Each record has its own `_client` (virtual field in event row, queryable). The `_meta.client_name` shows the name of the queried client context. Per-record client identification uses the `_client` virtual field. |
| EC-15-034 | Scheduled query execution (no analyst session) | `analyst_id` is null; `query_source` is "schedule:{schedule_name}" |
| EC-15-035 | Query returns 0 results | No decoration needed; empty result set |
| EC-15-036 | Client name contains unicode characters | Preserved as-is in `_client_name` |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-026 |
| L2 Invariants | DI-008 |
| Priority | P0 |
