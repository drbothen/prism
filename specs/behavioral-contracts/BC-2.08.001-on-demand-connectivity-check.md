---
document_type: behavioral-contract
level: L3
version: "1.5"
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
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.08.001: On-Demand Connectivity Check Per Sensor Per Client

## Description

The `check_sensor_health` tool invokes `verify_connectivity()` on the target sensor adapter, confirming reachability within a 30-second timeout. The check is scoped to the specified client's sensor instance per DI-008, emits exactly one AuditEntry per invocation per DI-004, and returns structured connectivity status rather than raising a tool-level error when the sensor is unreachable.

## Preconditions
- A valid `client_id` is provided in the health check tool call
- The target sensor is configured and `enabled: true` for the specified client
- The sensor adapter for the target sensor type is initialized
- The sensor spec declares a `probe_table` field (naming the probe target table), or the spec has at least one declared `[[tables]]` block (fallback behavior applies). Specs with neither are accepted but the probe is a structural no-op.

## Postconditions
- The `verify_connectivity()` method on the sensor adapter is invoked against the sensor's API endpoint
- The response includes `reachable: true` or `reachable: false` with a reason string
- The check completes within the sensor-specific timeout (default 30s)
- An AuditEntry is emitted for the health check invocation
- The probe routes the `LIMIT 0` fetch request to the table named by `probe_table` in the sensor spec (fully-qualified as `{sensor_id}_{probe_table}`). If `probe_table` is absent, the probe routes to the first declared table (`spec.tables[0].table_name`, fully-qualified as `{sensor_id}_{spec.tables[0].table_name}`). If no tables are declared, the probe is a structural no-op: `SpecDrivenSensorAdapter::fetch()` receives a table name matching no registered table, returns `Ok([])` without making HTTP contact, and connectivity.rs classifies the result as `status: Up`.
- Probes against sensors with no declared read tables (empty `spec.tables` and no `probe_table`) are accepted but guaranteed not to make HTTP contact; `Up` reflects only that the adapter was reachable by the runtime, not that the sensor API was contacted.

## Invariants
- DI-008: Client data separation -- health check targets only the specified client's sensor instance
- DI-004: Audit completeness -- exactly one AuditEntry is emitted
- The probe table MUST be a table declared in the sensor spec when `probe_table` is explicitly set (enforced at parse time via E-SPEC-026; this invariant cannot be violated at runtime because a spec that sets an invalid `probe_table` is rejected before registration).

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::InvalidInput` | `client_id` fails validation | Structured error with rejected value and allowed pattern |
| `PrismError::Config` | Client or sensor not found in config | Structured error: "Sensor '{sensor}' not configured for client '{id}'" |
| `PrismError::Sensor` | HTTP connection refused or timed out | Returns health status `reachable: false` with reason, not a tool-level error |
| `E-SPEC-026` | `probe_table` names a table not in `[[tables]]` (or spec has no `[[tables]]` blocks) | Spec load rejected at parse time (Rule 8); sensor not registered; no probe is ever attempted for this sensor |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-08-001 | Sensor API returns HTTP 503 during health check | Health status reports `reachable: false`, `reason: "service_unavailable"` |
| EC-08-002 | Sensor is configured but `enabled: false` | Health check returns `status: "disabled"` without making any API call |
| EC-08-003 | Health check times out after 30s | Returns `reachable: false`, `reason: "timeout"`, `timeout_seconds: 30` |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Valid `client_id` and `sensor_id`, sensor reachable | `reachable: true`; AuditEntry emitted | happy-path |
| Sensor returns HTTP 503 | `reachable: false`, `reason: "service_unavailable"`; no tool error | error |
| Sensor configured with `enabled: false` | `status: "disabled"`; no API call made | edge-case |
| Health check times out at 30s | `reachable: false`, `reason: "timeout"`, `timeout_seconds: 30` | edge-case |

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vector tables.

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (no matching VP) | Exactly one AuditEntry emitted per tool invocation | integration test |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-008 |
| L2 Invariants | DI-004, DI-008 |
| Priority | P1 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.5 | F-S504-P1-002-spec-reconciliation | 2026-06-23 | product-owner | F-S504-P1-002 spec reconciliation: postcondition 5 qualified-table form dot→underscore to match canonical SpecDrivenSensorAdapter (strip_prefix `{sensor_id}_`) / PrismQL FROM convention. Changed `{sensor_id}.{probe_table}` → `{sensor_id}_{probe_table}`; added explicit underscore form for the first-declared-table fallback clause. |
| 1.4 | S-5.04-spec-prep | 2026-06-22 | product-owner | probe_table field support (D-1260 / probe-table-field-design.md §5): added probe_table Precondition; two Postconditions (LIMIT 0 routes via probe_table → first-declared-table → no-op fallback chain); E-SPEC-026 Error Case row; probe-table parse-time enforcement Invariant. |
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial draft |
