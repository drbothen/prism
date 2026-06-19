---
document_type: behavioral-contract
level: L3
version: "1.6"
status: active
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
inputs: [domain-spec/capabilities.md, domain-spec/invariants.md]
input-hash: "4a1657f"
traces_to: domain-spec/L2-INDEX.md
origin: greenfield
extracted_from: null
subsystem: "SS-08"
capability: "CAP-008"
lifecycle_status: active
introduced: cycle-1
modified: ["cycle-1-burst-45", "RECONCILIATION-2-health-resource-shape-2026-06-17", "RECONCILIATION-2-EC-08-013-retirement-2026-06-18"]
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.08.006: Health Status MCP Resource

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.6 | RECONCILIATION-2-EC-08-013-retirement-2026-06-18 | 2026-06-18 | product-owner | **RECONCILIATION-2 (architect Ruling 4) — EC-08-013 retired/subsumed into EC-08-011.** EC-08-013 ("zero clients configured → `{\"clients\":{}}`") is marked retired with strikethrough in the edge cases table; ID preserved per append-only numbering policy (must not be reused). Reachability analysis (`crates/prism-mcp/src/server.rs::check_sensor_health`): the tool always returns `not_yet_available_msg` (GAP-002-A; S-5.04 scope) — the health cache is never written, making both EC-08-013 and EC-08-011 paths currently pre-implementation. Post-S-5.04, a zero-client deployment where `check_sensor_health` is explicitly invoked could technically reach `{"clients":{}}`, so an un-retire condition is attached: un-retire when S-5.04 ships with cache write semantics and a zero-client integration test is required. Until then, the correct response for the zero-clients state is the EC-08-011 sentinel (no check has run → empty cache → sentinel). Bump v1.5 → v1.6. |
| 1.5 | RECONCILIATION-2-health-resource-shape-2026-06-17 | 2026-06-17 | product-owner | **RECONCILIATION-2 — Propagated BC-2.08.005 v1.5 two-phase probe model; resolved array-vs-keyed-object shape; reconciled `status`/`last_checked_at` contradiction.** Three fixes: (1) **Two-phase model propagation:** Postcondition 3 rewritten to use `SensorHealthResult` fields from BC-2.08.005 v1.5 (`probe_level`, `reachable: null` for spec-only, `auth_valid: null` for spec-only, `last_successful_query_at`). The retired `status: "up"|"down"|"degraded"|"auth_invalid"|"unknown"` and `last_checked_at` fields are removed — they were the pre-v1.5 shape that BC-2.08.005 superseded. (2) **Keyed-object shape (code change required):** Postcondition 2 explicitly states `sensors` MUST be a JSON object keyed by `sensor_id` (not a JSON array). The current `render_sensors_health_resource` code emits `"sensors": [array]` — this violates postcondition 2. The implementer must fix `render_sensors_health_resource` to emit a `BTreeMap<sensor_id, SensorHealthResult>` under each client key. (3) **Sentinel vs empty-clients disambiguation:** Postcondition 5 explicitly separates the "no health check run" sentinel response (`{ status: "unknown", message: "..." }`) from the normal `clients` keyed-object response — they are different JSON shapes, which the code already implements correctly. EC-08-011/012 updated to remove stale `last_checked_at` references. Canonical test vectors updated to show both S-5.03 and S-5.04 scoped variants. **Implementer must change:** `render_sensors_health_resource` in `resources.rs` must emit `sensors` as a keyed object (`BTreeMap<String, &SensorHealthResult>`) rather than the current `Vec`. **Bumped v1.4→v1.5.** |
| 1.4 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.3 | pass-69-housekeeping | 2026-04-20 | product-owner | Resolved VP-TBD placeholder per decision matrix; normalized changelog schema to canonical 5-col form. |
| 1.2 | cycle-1-burst-45 | 2026-04-20 | product-owner | pre-build-sweep: Template-compliance sweep — appended Changelog row (version bump 1.1→1.2). |
| 1.1 | cycle-1-burst-45 | 2026-04-19 | product-owner | Burst 45 / P3P44-A-HIGH-003: URI changed from `prism://health/{client_id}` to `prism://sensors/health` (global matrix). Per-analyst-stdio deployment makes per-client URI redundant; health is a cross-client matrix per api-surface.md lines 207, 245. Error case updated to remove stale client_id lookup. |
| 1.0 | — | 2026-04-14 | product-owner | Initial draft |

## Description

This BC governs the `prism://sensors/health` MCP resource, which exposes cached sensor connectivity and authentication status as a global health matrix across all configured clients and sensors. The resource is read-only and non-templated — it returns the full `(client_id, sensor_id)` health matrix in one JSON payload. It does not trigger a live health check; it reflects the most recently cached results from `check_sensor_health` tool invocations.

## Preconditions

1. The MCP resource `prism://sensors/health` is registered in `resources/list`
2. The resource is a global (non-templated) URI — no path parameters
3. Prism has loaded configuration and initialized client/sensor mappings

## Postconditions

1. Reading the resource returns the most recent health status for all sensors across all configured clients as a health matrix, grouped by client.
2. The resource content is `application/json` with schema: `{ clients: { [client_id]: { sensors: { [sensor_id]: SensorHealthResult } } } }`. The inner `sensors` value MUST be a JSON object keyed by `sensor_id` — NOT a JSON array. This keyed-object shape is required for AI consumer lookup by sensor ID without scanning an array. Any implementation that emits `sensors` as a JSON array rather than a keyed object violates this postcondition (code change required if array is currently emitted).
3. `SensorHealthResult` fields (aligned with BC-2.08.005 v1.5 two-phase probe model): `sensor_id: String`, `client_id: String`, `probe_level: "spec-only"|"live"`, `reachable: bool|null` (`null` for spec-only scope; `bool` for live probe per S-5.04), `auth_valid: bool|null` (`null` for spec-only scope; `bool` for live probe per S-5.04), `rate_limit: RateLimitInfo|null`, `last_successful_query_at: DateTime<Utc>|null` (`null` for spec-only scope; `DateTime` after a live query per S-5.04), `error: String|null`. **The retired `status: "up"|"down"|"degraded"|"auth_invalid"|"unknown"` and `last_checked_at` fields do NOT appear in the S-5.03+ SensorHealthResult shape — they were replaced by the two-phase probe fields in BC-2.08.005 v1.5.**
4. The resource reflects cached data from the most recent `check_sensor_health` invocation (not a live check). Stale entries (older than the 5-minute TTL) are returned with a top-level `stale: true` flag in the payload.
5. If no `check_sensor_health` has been run at all (empty cache), the resource returns `{ "status": "unknown", "message": "Run check_sensor_health to populate this resource." }` — NOT an error, NOT an empty `clients` object. This is the sentinel "uninitialized" response. Once any health check has run, the `clients` keyed-object schema (postcondition 2) is used.

## Invariants

- DI-008: Client data separation — the matrix includes entries only for clients present in the loaded configuration; no cross-contamination between unrelated client entries

## Error Cases

| Error | Condition | Behavior |
|-------|-----------|----------|
| Resource unavailable | Prism failed to initialize resources | MCP protocol-level resource error; not a 404 (resource has no path parameters to be wrong) |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-08-011 | Resource read immediately after startup, before any `check_sensor_health` call | Returns sentinel `{ "status": "unknown", "message": "Run check_sensor_health to populate this resource." }` — not an error, not a `clients` object |
| EC-08-012 | Health data is stale (last check was 10+ minutes ago) | Resource returns cached data with top-level `stale: true` flag; no automatic expiry; consumer uses `last_successful_query_at` per-sensor field to assess freshness |
| ~~EC-08-013~~ | ~~Zero clients configured~~ | ~~Resource returns `{ "clients": {} }` — empty object, not an error~~ **RETIRED — subsumed by EC-08-011 (architect Ruling 4, 2026-06-18).** With zero clients configured, no `check_sensor_health` call is semantically meaningful, so the health cache remains empty and the resource returns the EC-08-011 sentinel `{ "status": "unknown", "message": "Run check_sensor_health to populate this resource." }`. Code-path reachability analysis (`crates/prism-mcp/src/server.rs::check_sensor_health`): the tool currently always returns `not_yet_available_msg` (GAP-002-A; S-5.04 scope); once S-5.04 ships, a call with zero configured clients iterates over an empty client set and writes `{ clients: {} }` to the cache — this is the same-cache path as a normal run, making EC-08-013's `{"clients":{}}` technically reachable post-S-5.04 if an explicit tool call is made on a zero-client deployment. **Un-retire condition:** un-retire EC-08-013 (remove strikethrough) when S-5.04 ships `check_sensor_health` with cache write semantics and a zero-client integration test is needed to assert the `{"clients":{}}` shape. ID EC-08-013 is reserved and must not be reused. |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Read `prism://sensors/health` after `check_sensor_health("acme")` — S-5.03 scope | `{ "clients": { "acme": { "sensors": { "crowdstrike": { "sensor_id": "crowdstrike", "client_id": "acme", "probe_level": "spec-only", "reachable": null, "auth_valid": null, "last_successful_query_at": null, "rate_limit": null, "error": null } } } }, "stale": false }` | happy-path (S-5.03) |
| Read `prism://sensors/health` after `check_sensor_health("acme")` — S-5.04 scope (live probe) | `{ "clients": { "acme": { "sensors": { "crowdstrike": { "sensor_id": "crowdstrike", "client_id": "acme", "probe_level": "live", "reachable": true, "auth_valid": true, "last_successful_query_at": "<timestamp>", "rate_limit": null, "error": null } } } }, "stale": false }` | happy-path (S-5.04) |
| Read `prism://sensors/health` immediately after startup (no checks run) | `{ "status": "unknown", "message": "Run check_sensor_health to populate this resource." }` — the uninitialized sentinel, NOT a `clients` object | edge-case |
| Read `prism://sensors/health` when Prism failed to register resources | MCP resource error response (protocol-level) | error |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (none) | credential-absence guaranteed by SensorHealthResult type design (no credential fields); timestamp-equals-cache invariant is integration behavior; covered by integration test in tests/health_tests.rs | — |

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
