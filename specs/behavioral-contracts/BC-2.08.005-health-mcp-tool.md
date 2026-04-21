---
document_type: behavioral-contract
level: L3
version: "1.3"
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

# BC-2.08.005: Health Check MCP Tool

## Description

The `check_sensor_health` tool returns structured health status for one or all sensors for a given client, or a cross-client health matrix when `client_id` is null. Each response includes per-sensor connectivity, auth validity, rate limit state, and last successful query timestamp, plus a `resource_pressure` section with active cursor count and token count. The response uses `structuredContent` for machine-parseable data and `content[].text` prose summary. Trust level is `"internal"` since health data is Prism-generated.

## Preconditions
- The `check_sensor_health` MCP tool is registered in `tools/list`
- The tool accepts `client_id: String` (required) and `sensor_id: Option<SensorId>` (optional -- null means all sensors for client)

## Postconditions
- When `sensor_id` is provided: returns health status for that single sensor
- When `sensor_id` is null: returns health status for all configured sensors for the client
- When `client_id` is null (cross-client): returns health status for all sensors across all configured clients. Each entry includes the `client_id` field so results can be attributed. The `summary` section aggregates counts across all clients. `partial_failures` lists any clients whose health check failed (e.g., credential unavailable) without blocking results from other clients.
- Each sensor health entry contains: `sensor_id`, `client_id` (always present in cross-client responses), `reachable`, `auth_valid`, `rate_limit`, `last_successful_query_at`
- The response includes a `resource_pressure` section with: `active_cursor_count` (current number of non-expired cursors, out of 200 cap) and `active_token_count` (current number of unexpired, unconsumed confirmation tokens, out of 100 cap). This gives the agent visibility into resource pressure without needing a separate tool.
- Response uses `structuredContent` for machine-parseable health data
- Response includes `content[].text` prose summary (e.g., "2 of 3 sensors healthy for client 'acme'")
- Response metadata includes `trust_level: "internal"` (health data is Prism-internal, not sensor-sourced)
- Tool annotations: `readOnlyHint: true`, `destructiveHint: false`, `idempotentHint: true`, `openWorldHint: true`

## Invariants
- DI-004: Audit completeness -- exactly one AuditEntry emitted per tool invocation
- DI-008: Client data separation -- only the specified client's sensors are checked

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::InvalidInput` | Invalid `client_id` format | Structured error with validation details |
| `PrismError::Config` | `client_id` not found in config | Structured error with suggestion to check config |
| `PrismError::InvalidInput` | Invalid `sensor_id` value | Structured error listing valid sensor IDs from loaded spec files |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-004 | Client has zero sensors configured | Returns empty health array with message "Client '{id}' has no sensors configured" |
| EC-08-010 | One sensor healthy, another unreachable | Returns partial health results; does not fail the entire tool call |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| `check_sensor_health("acme", sensor_id: null)` — all sensors healthy | `structuredContent` with all sensors `reachable: true`, `auth_valid: true`; prose "3 of 3 sensors healthy" | happy-path |
| `check_sensor_health(null)` — cross-client | Health matrix across all clients; each entry includes `client_id` | happy-path |
| One sensor healthy, one unreachable | Partial results; healthy sensor shown; unreachable sensor `reachable: false` | edge-case |
| Client with zero sensors configured | Empty array; message "Client 'x' has no sensors configured" | edge-case |

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vector tables.

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (no matching VP) | Exactly one AuditEntry emitted per tool invocation | integration test |
| (no matching VP) | `trust_level: "internal"` always set on health responses | integration test |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-008 |
| L2 Invariants | DI-004, DI-008 |
| Priority | P1 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial draft |
