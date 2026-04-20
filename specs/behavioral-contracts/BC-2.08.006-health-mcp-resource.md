---
document_type: behavioral-contract
level: L3
version: "1.2"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
inputs: [domain-spec/capabilities.md, domain-spec/invariants.md]
input-hash: "365fb25"
traces_to: domain-spec/L2-INDEX.md
origin: greenfield
extracted_from: null
subsystem: "SS-08"
capability: "CAP-008"
lifecycle_status: active
introduced: cycle-1
modified: ["cycle-1-burst-45"]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.08.006: Health Status MCP Resource

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 1.2 | 2026-04-20 | product-owner | pre-build-sweep: Template-compliance sweep — appended Changelog row (version bump 1.1→1.2). |
| 1.1 | 2026-04-19 | product-owner | Burst 45 / P3P44-A-HIGH-003: URI changed from `prism://health/{client_id}` to `prism://sensors/health` (global matrix). Per-analyst-stdio deployment makes per-client URI redundant; health is a cross-client matrix per api-surface.md lines 207, 245. Error case updated to remove stale client_id lookup. |
| 1.0 | 2026-04-14 | product-owner | Initial draft |

## Description

This BC governs the `prism://sensors/health` MCP resource, which exposes cached sensor connectivity and authentication status as a global health matrix across all configured clients and sensors. The resource is read-only and non-templated — it returns the full `(client_id, sensor_id)` health matrix in one JSON payload. It does not trigger a live health check; it reflects the most recently cached results from `check_sensor_health` tool invocations.

## Preconditions

1. The MCP resource `prism://sensors/health` is registered in `resources/list`
2. The resource is a global (non-templated) URI — no path parameters
3. Prism has loaded configuration and initialized client/sensor mappings

## Postconditions

1. Reading the resource returns the most recent health status for all sensors across all configured clients as a health matrix keyed by `(client_id, sensor_id)`
2. The resource content is `application/json` with schema: `{ clients: { [client_id]: { sensors: { [sensor_id]: SensorHealthResult } } } }`
3. `SensorHealthResult` fields: `status: "up"|"down"|"degraded"|"auth_invalid"|"unknown"`, `reachable: bool|null`, `auth_valid: bool|null`, `last_checked_at: DateTime<Utc>|null`
4. The resource reflects cached data from the most recent `check_sensor_health` invocation (not a live check)
5. If no health check has been run for a given sensor, that entry reports `status: "unknown"`, `reachable: null`, `auth_valid: null`, `last_checked_at: null`

## Invariants

- DI-008: Client data separation — the matrix includes entries only for clients present in the loaded configuration; no cross-contamination between unrelated client entries

## Error Cases

| Error | Condition | Behavior |
|-------|-----------|----------|
| Resource unavailable | Prism failed to initialize resources | MCP protocol-level resource error; not a 404 (resource has no path parameters to be wrong) |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-08-011 | Resource read immediately after startup, before any `check_sensor_health` call | All sensor entries report `status: "unknown"`, `reachable: null`, `auth_valid: null`, `last_checked_at: null` |
| EC-08-012 | Health data is stale (last check was 10+ minutes ago) | Resource includes `last_checked_at` per entry so the consumer can assess freshness; no automatic expiry |
| EC-08-013 | Zero clients configured | Resource returns `{ "clients": {} }` — empty object, not an error |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Read `prism://sensors/health` after `check_sensor_health("acme")` with CrowdStrike returning up | `{ "clients": { "acme": { "sensors": { "crowdstrike": { "status": "up", "reachable": true, "auth_valid": true, "last_checked_at": "<timestamp>" } } } } }` | happy-path |
| Read `prism://sensors/health` immediately after startup (no checks run) | All sensor entries have `status: "unknown"`, `reachable: null`, `auth_valid: null`, `last_checked_at: null` | edge-case |
| Read `prism://sensors/health` when Prism failed to register resources | MCP resource error response (protocol-level) | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-TBD | Resource never returns credential values or full API URLs | manual / integration test |
| VP-TBD | `last_checked_at` in response always equals timestamp from most recent `check_sensor_health` for that `(client_id, sensor_id)` pair | proptest |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-008 |
| L2 Invariants | DI-008 |
| Architecture Module | SS-08 (filled by architect) |
| Stories | S-5.03 |

## Related BCs

- BC-2.08.005 — depends on: `check_sensor_health` tool produces the cached data this resource exposes
- BC-2.10.008 — composes with: MCP Resources registry lists `prism://sensors/health` alongside other resources

## Architecture Anchors

- `architecture/api-surface.md#event-feed-resources` — `prism://sensors/health` is listed as an Event Feed resource (global, updated on health change)

## Story Anchor

S-5.03 — Resources and Prompts
