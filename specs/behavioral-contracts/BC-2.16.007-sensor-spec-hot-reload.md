---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "SS-16"
capability: "CAP-030"
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

# BC-2.16.007: Sensor Spec Hot Reload — Add/Remove/Update Sensor Tables Without Restart

## Preconditions
- `reload_config` (BC-2.16.005) has been invoked and detected changes in sensor spec files
- The new `ConfigSnapshot` has passed validation

## Postconditions
- **New spec files** (present in directory but not in current snapshot):
  - The new `SensorSpec` is loaded and its tables registered with the DataFusion catalog
  - New tables are immediately queryable via PrismQL
  - A reload result entry is emitted: `"added": ["{sensor_id}.{table_name}", ...]`

- **Removed spec files** (present in current snapshot but absent from directory):
  - The sensor's tables are unregistered from the DataFusion catalog
  - Queries targeting removed tables return `E-QUERY-011: "Table '{sensor_id}.{table_name}' is no longer available. The sensor spec was removed."`
  - Scheduled queries referencing removed tables continue to run but produce empty results with a `sensor_errors` entry
  - A reload result entry is emitted: `"removed": ["{sensor_id}.{table_name}", ...]`

- **Modified spec files** (file hash changed):
  - The sensor's tables are re-registered with updated schemas and fetch pipelines
  - If column definitions changed (added/removed/type changed), the `notifications/tools/list_changed` MCP notification is sent
  - If only non-schema fields changed (e.g., rate_limit_hints, step URLs, pagination config), no notification is sent (the table schema visible to the agent is unchanged)
  - A reload result entry is emitted: `"modified": ["{sensor_id}.{table_name}", ...]` with `"schema_changed": true/false`

- **Unchanged spec files** (file hash identical):
  - No action taken; existing table registrations remain
  - A reload result entry is emitted: `"unchanged": ["{sensor_id}", ...]`

## In-Flight Query Safety (DEC-037)
- In-flight queries that started before the reload continue using the `ConfigSnapshot` captured at query start (via arc-swap guard, BC-2.16.006)
- The old `ConfigSnapshot` (and its table schemas) remains valid until the last in-flight query holding a reference completes
- The next query after the reload uses the new schema

## Cache Interaction
- Response cache entries (CAP-014) for modified or removed sensor tables are invalidated on reload
- Cache entries for unchanged sensors are retained

## Error Handling
- If a modified spec file fails validation, the modification is rejected and the previous version of that spec remains active (DI-030 + DI-031)
- The reload result includes the validation error for the rejected spec alongside successful updates for other specs

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-030 |
| L2 Invariants | DI-030, DI-031 |
| L2 Entities | SensorSpec, ConfigSnapshot |
| Capabilities | CAP-029, CAP-030 |
| Priority | P1 |
