---
document_type: behavioral-contract
level: L3
version: "1.2"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "365fb25"
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

## Postconditions
- The `verify_connectivity()` method on the sensor adapter is invoked against the sensor's API endpoint
- The response includes `reachable: true` or `reachable: false` with a reason string
- The check completes within the sensor-specific timeout (default 30s)
- An AuditEntry is emitted for the health check invocation

## Invariants
- DI-008: Client data separation -- health check targets only the specified client's sensor instance
- DI-004: Audit completeness -- exactly one AuditEntry is emitted

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::InvalidInput` | `client_id` fails validation | Structured error with rejected value and allowed pattern |
| `PrismError::Config` | Client or sensor not found in config | Structured error: "Sensor '{sensor}' not configured for client '{id}'" |
| `PrismError::Sensor` | HTTP connection refused or timed out | Returns health status `reachable: false` with reason, not a tool-level error |

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
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col Version | Burst | Date | Author | Change form. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial draft |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
