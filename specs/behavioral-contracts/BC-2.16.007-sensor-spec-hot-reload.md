---
document_type: behavioral-contract
level: L3
version: "1.3"
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
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "8e43eb2"
traces_to:
  - "CAP-030"
extracted_from: ".factory/specs/prd.md"
---

# BC-2.16.007: Sensor Spec Hot Reload — Add/Remove/Update Sensor Tables Without Restart

## Description

When `reload_config` detects changed sensor spec files, this BC governs how the
DataFusion table catalog is updated: new specs register tables, removed specs
unregister tables, and modified specs re-register tables with updated schemas or fetch
pipelines. All transitions are file-hash-driven.

In-flight queries that started before the reload use their captured `ConfigSnapshot`
(via ArcSwap guard, BC-2.16.006) and are unaffected. Response cache entries for
modified or removed sensors are invalidated on reload. If a modified spec file fails
validation, it is rejected and the previous version remains active.

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
  - If only non-schema fields changed (e.g., rate_limit_hints, step URLs, pagination config), no notification is sent
  - A reload result entry is emitted: `"modified": ["{sensor_id}.{table_name}", ...]` with `"schema_changed": true/false`

- **Unchanged spec files** (file hash identical):
  - No action taken; existing table registrations remain
  - A reload result entry is emitted: `"unchanged": ["{sensor_id}", ...]`

## In-Flight Query Safety (DEC-037)
- In-flight queries that started before the reload continue using the `ConfigSnapshot` captured at query start (via arc-swap guard, BC-2.16.006)
- The old `ConfigSnapshot` (and its table schemas) remains valid until the last in-flight query holding a reference completes
- The next query after the reload uses the new schema

## Invariants
- File-hash-based change detection: only actually-changed files trigger re-registration
- If a modified spec file fails validation, the previous version remains active (DI-030 + DI-031)
- Cache entries for modified or removed sensors are invalidated on reload; unchanged sensor caches are retained

## Error Conditions
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-QUERY-011` | Query targets a removed sensor table after reload | Structured error: "Table '{sensor_id}.{table_name}' is no longer available. The sensor spec was removed." |
| (validation failure) | Modified spec file fails validation | Previous version retained; validation error included in reload result |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| In-flight query during reload | query started before reload; reload completes mid-query | Query uses pre-reload schema; completes normally |
| Scheduled query on removed table | schedule references `removed_sensor.table` | Schedule runs; produces empty results with sensor_errors entry; schedule not disabled |
| Non-schema spec change | rate_limit_hints changed; column definitions unchanged | Tables re-registered; no `notifications/tools/list_changed` |
| Schema change | column added to spec | Tables re-registered; `notifications/tools/list_changed` sent |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for full canonical vectors.

| Scenario | Input | Expected Output |
|----------|-------|-----------------|
| Add new spec | new `vendor.sensor.toml` appears in directory | Tables registered; `"added": ["vendor.table1"]` in result |
| Remove spec | existing spec file deleted | Tables unregistered; queries on removed tables get `E-QUERY-011` |
| Modify spec — schema change | column added | Tables re-registered; `notifications/tools/list_changed` sent; `schema_changed: true` |
| Modify spec — no schema change | rate limit changed | Tables re-registered; no notification; `schema_changed: false` |
| Modified spec fails validation | invalid TOML after edit | Previous version retained; validation error in result |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| (none) | In-flight query safety transitively covered by VP-032 and BC-2.16.006; cache invalidation for modified/removed sensors is a behavioral integration test; no additional formal VP. |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-030 |
| L2 Invariants | DI-030, DI-031 |
| L2 Entities | SensorSpec, ConfigSnapshot |
| Related BCs | BC-2.16.005 (reload_config), BC-2.16.006 (ArcSwap config access) |
| Priority | P1 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-74-fix | 2026-04-20 | product-owner | Resolved (placeholder) row in ## Verification Properties per pass-74 VP-TBD decision matrix extension. |
| 1.2 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description; added ## Invariants; added ## Error Conditions (extracted from body); added ## Canonical Test Vectors; added ## Verification Properties; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial draft |
