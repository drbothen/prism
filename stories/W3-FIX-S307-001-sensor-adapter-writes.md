---
story_id: W3-FIX-S307-001
title: "prism-sensors: Concrete Sensor Adapter Write Overrides (4 Built-In Sensors)"
wave: 4
target_module: prism-sensors
subsystems: [SS-01, SS-11]
priority: P0
depends_on: []
blocks: [W3-FIX-S307-002-write-capability-and-observability, S-5.01-FOLLOWUP-MCP-BOOT]
estimated_days: 3
points: 5
risk: MEDIUM
status: draft
document_type: story
version: "1.0"
level: "L4"
producer: story-writer
timestamp: "2026-05-08T00:00:00Z"
input-hash: "[md5]"
traces_to: []
cycle: "v1.0.0-greenfield"
epic_id: "E-CLEANUP-02"
phase: 3
behavioral_contracts: [BC-2.04.007, BC-2.04.008]
verification_properties: []
assumption_validations: []
risk_mitigations: []
anchor_bcs: [BC-2.04.007, BC-2.04.008]
anchor_capabilities: [CAP-006]
anchor_subsystem: ["SS-01", "SS-11"]
# BC status: existing BCs reused. New BC for per-sensor write boundary may be needed
# if product-owner determines write contracts need per-sensor granularity beyond BC-2.04.007.
# Surfaced as open question OQ-1.
inputs:
  - ".factory/specs/architecture/decisions/ADR-022-production-runtime-wiring.md"
  - ".factory/stories/S-3.07-write-execution.md"
  - ".factory/specs/behavioral-contracts/BC-INDEX.md"
  - ".factory/specs/architecture/sensor-adapters.md"
  - ".factory/specs/architecture/write-operations.md"
---

# W3-FIX-S307-001 — prism-sensors: Concrete Sensor Adapter Write Overrides

## Narrative

As the Prism write execution pipeline, I want each of the four built-in sensor adapters
(CrowdStrike, Armis, Claroty, Cyberint) to override `SensorAdapter::write()` with a real
implementation that calls the sensor's write API endpoint, so that write operations
declared in sensor TOML specs can actually dispatch to the sensor and return structured
results instead of `WriteNotImplemented` for every write verb.

## Objective

Implement `fn write(...)` override in each built-in sensor adapter. Replace the
`WriteNotImplemented` default in `adapter.rs:365` with concrete HTTP dispatch calls for
at least 1-2 write verbs per sensor (e.g., `acknowledge`, `contain`, `isolate`,
`update_status`). The default `WriteNotImplemented` body in `adapter.rs:365` MUST remain
for sensors with no declared write endpoints — it is the correct fallback.

Per ADR-022 §G Story 3 and audit finding F-AUD-D1-04, this story resolves the gap where
`SensorAdapter::write()` returns `WriteNotImplemented` for all four built-in sensors
regardless of what write endpoints their TOML specs declare.

No `todo!()` or `unimplemented!()` may remain in the concrete write override methods
before this story merges. Per POL-12.

---

## Behavioral Contracts

| BC ID | Title |
|-------|-------|
| BC-2.04.007 | Three-Tier Risk Classification for Operations |
| BC-2.04.008 | Dry-Run Default for Reversible Write Operations |

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec | ~3,000 |
| `adapter.rs` (trait extension review) | ~1,000 |
| `crowdstrike/write.rs` (contain + isolate verbs) | ~2,500 |
| `armis/write.rs` (update_device_status + acknowledge_alert verbs) | ~2,500 |
| `claroty/write.rs` (update_alert_status verb) | ~1,800 |
| `cyberint/write.rs` (acknowledge_alert verb) | ~1,800 |
| BC files (2 BCs) | ~1,000 |
| Integration tests (per-sensor write dispatch) | ~3,000 |
| Total | ~16,600 |

Within the 30% context window budget.

---

## Tasks

1. Review `SensorAdapter::write()` trait signature at `crates/prism-sensors/src/adapter.rs:365`:
   ```rust
   async fn write(
       &self,
       endpoint: &WriteEndpointSpec,
       records: &RecordBatch,
       params: &HashMap<String, String>,
       org_id: &OrgId,
   ) -> Result<Vec<RecordWriteResult>, SensorError> {
       Err(SensorError::WriteNotImplemented { sensor: self.name().to_string() })
   }
   ```
   This default MUST remain. Only concrete sensor structs override it.

   Note: The signature uses `OrgId` not `TenantId` (per ADR-006 rename). Verify the
   actual signature in `adapter.rs` before implementing overrides — this story must match
   the existing trait definition exactly.

2. Read each sensor's TOML spec to identify available write endpoints:
   - CrowdStrike: look for `[[endpoints]]` entries with `method = "POST"` or `"PATCH"`.
     Expected write verbs: `contain_host`, `lift_containment`, `isolate_device`.
   - Armis: look for write endpoints. Expected: `update_device_status`, `acknowledge_alert`.
   - Claroty: look for write endpoints. Expected: `update_alert_status`.
   - Cyberint: look for write endpoints. Expected: `acknowledge_alert`.

   If any sensor's TOML spec lacks a `[[endpoints.write]]` section, document the gap
   in the story's Open Questions and coordinate with Bundle C (sensor TOML completion).
   Do NOT block merge on TOML gaps — implement what is specified and flag what is missing.

3. Implement CrowdStrike write override at `crates/prism-sensors/src/crowdstrike/write.rs`:
   ```rust
   impl SensorAdapter for CrowdStrikeAdapter {
       async fn write(
           &self,
           endpoint: &WriteEndpointSpec,
           records: &RecordBatch,
           params: &HashMap<String, String>,
           org_id: &OrgId,
       ) -> Result<Vec<RecordWriteResult>, SensorError> {
           match endpoint.verb.as_str() {
               "contain_host" => self.contain_hosts(endpoint, records, params, org_id).await,
               "lift_containment" => self.lift_containment(endpoint, records, params, org_id).await,
               verb => Err(SensorError::WriteNotImplemented {
                   sensor: "crowdstrike".into(),
                   // Note: do NOT use the generic default here — be explicit about unsupported verbs
               }),
           }
       }
   }
   ```
   `contain_hosts` must:
   - Extract `device_id` column values from `records`.
   - Call `POST /devices/entities/devices-actions/v2?action_name=contain` with a JSON
     body `{ "ids": [...] }`.
   - Parse response per `endpoint.response_path` (JSONPath).
   - Map HTTP 200 → `RecordWriteResult { status: Success }`.
   - Map 4xx → `RecordWriteResult { status: Failed, error: Some(...) }`.

4. Implement Armis write override at `crates/prism-sensors/src/armis/write.rs`:
   - Verb `update_device_status`: `PATCH /api/v1/devices/{device_id}/` with `{ "status": "{status}" }`.
   - Verb `acknowledge_alert`: `POST /api/v1/alerts/{alert_id}/mark-as-seen/`.
   - Per endpoint, extract the record ID column (`endpoint.record_id_field`) from `records`.
   - Use the existing `ArmisAdapter` authenticated HTTP client (not a new client).

5. Implement Claroty write override at `crates/prism-sensors/src/claroty/write.rs`:
   - Verb `update_alert_status`: `PUT /api/v3/alerts/{alert_id}` with `{ "status": "{status}" }`.
   - Use existing Claroty bearer-token authenticated client (BC-2.01.007).

6. Implement Cyberint write override at `crates/prism-sensors/src/cyberint/write.rs`:
   - Verb `acknowledge_alert`: `PUT /api/v1/alerts/{alert_id}/acknowledge`.
   - Use existing Cyberint cookie-auth client (BC-2.01.006).

7. Write integration tests in `crates/prism-sensors/tests/write_dispatch_tests.rs`:
   - Test CrowdStrike `contain_host`: DTU clone returns 200 → `RecordWriteResult { status: Success }`.
   - Test CrowdStrike `contain_host`: DTU clone returns 400 → `RecordWriteResult { status: Failed }`.
   - Test Armis `acknowledge_alert`: DTU clone 200 → Success.
   - Test Claroty `update_alert_status`: DTU clone 200 → Success.
   - Test Cyberint `acknowledge_alert`: DTU clone 200 → Success.
   - Test default `WriteNotImplemented` for a sensor with no write endpoint override (use a
     test-only `MinimalSensorAdapter` that does not override `write()`).

---

## Acceptance Criteria

**AC-1:** Given a `CrowdStrikeAdapter::write(contain_host_endpoint, records, params, org_id)`,
When the DTU clone returns HTTP 200 for the contain request, Then a `Vec<RecordWriteResult>`
is returned with `status: Success` for each input record ID.
(traces to BC-2.04.007 postcondition — write dispatch through risk classification pipeline)

**AC-2:** Given a `CrowdStrikeAdapter::write(contain_host_endpoint, records, params, org_id)`,
When the DTU clone returns HTTP 400 (bad device ID), Then the returned `RecordWriteResult`
has `status: Failed` with `error: Some("...")` and the operation does NOT panic or return `Err`.
Partial failure is not an `Err` return — it is a `Failed` result per S-3.07 design.
(traces to BC-2.04.008 postcondition — dry-run and partial-failure handling)

**AC-3:** Given an `ArmisAdapter` (or any adapter) invoked with a verb not listed in its
write override match arms, When `write()` is called, Then `SensorError::WriteNotImplemented`
is returned for that specific verb — not for all Armis writes.

**AC-4:** Given the default `SensorAdapter::write()` (unarranged struct with no override),
When `write()` is called, Then `WriteNotImplemented` is returned. The default at `adapter.rs:365`
remains unchanged by this story.

**AC-5:** Given all four sensor adapters with write overrides, When the write overrides are
called with a valid `WriteEndpointSpec` and non-empty `RecordBatch`, Then each override uses
the sensor's **existing authenticated HTTP client** — no second HTTP client is constructed.
(traces to BC-2.04.007 — writes route through the same authenticated path as reads)

**AC-6:** No `todo!()`, `unimplemented!()`, or `panic!("stub")` may remain in any
concrete write override method (CrowdStrike, Armis, Claroty, Cyberint) before merge.
Per POL-12. The default `WriteNotImplemented` in `adapter.rs:365` is NOT a stub — it is
the permanent correct fallback and is exempt from this rule.

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|----------------|
| CrowdStrike write override | `prism-sensors/src/crowdstrike/write.rs` (SS-01) | Effectful |
| Armis write override | `prism-sensors/src/armis/write.rs` (SS-01) | Effectful |
| Claroty write override | `prism-sensors/src/claroty/write.rs` (SS-01) | Effectful |
| Cyberint write override | `prism-sensors/src/cyberint/write.rs` (SS-01) | Effectful |
| `SensorAdapter::write()` trait | `prism-sensors/src/adapter.rs` (SS-01) | Mixed (pure trait; effectful override) |

Per `architecture/module-decomposition.md`, `prism-sensors` (SS-01, COMP-004) owns
sensor adapter orchestration and auth traits. Write overrides live within each sensor's
sub-module, consuming the existing authenticated HTTP client from the adapter struct.

---

## Purity Classification

| Module | Classification | Justification |
|--------|----------------|---------------|
| `adapter.rs` (trait default) | Pure | Default `WriteNotImplemented` is a pure const return. |
| Concrete write overrides | Effectful | HTTP POST/PATCH/PUT/DELETE to sensor APIs. |

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| Write overrides MUST use the existing authenticated HTTP client from the adapter struct | ADR-022 §C (wiring contract) + S-3.07 Architecture Compliance Rules | Code review; grep for `reqwest::Client::new()` in write.rs files |
| The default `WriteNotImplemented` at `adapter.rs:365` MUST remain unchanged | ADR-022 §G Story 3 explicitly states this | Code review; test AC-4 verifies default behavior |
| HTTP error responses (4xx/5xx) MUST map to `RecordWriteResult { status: Failed }`, NOT to `Err(...)` | S-3.07 design — partial failure is not an error | AC-2 test; code review |
| Write overrides MUST NOT create a second HTTP client | S-3.07 Architecture Compliance Rules | Code review |
| Credential resolution for write calls MUST use the same credential store path as read calls | AD-017 (AI-opaque credentials) | Code review; no inline credential values |

**Forbidden Dependencies:** `prism-sensors` write overrides MUST NOT depend on `prism-query`,
`prism-mcp`, or `prism-bin`. The dependency direction is: `prism-query` → `prism-sensors`,
never the reverse.

---

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| reqwest | 0.12.x (workspace) | HTTP client (existing; reuse adapter's client, do not add new dep) |
| serde_json | 1.x (workspace) | Request body serialization; response parsing |
| arrow | 53.x (workspace) | `RecordBatch` column extraction for write payloads |
| jsonpath-rust | 0.5.x (workspace, per S-3.07) | `response_path` JSONPath evaluation |
| prism-core | workspace | `OrgId`, `PrismError`, `SensorError` |
| prism-spec-engine | workspace | `WriteEndpointSpec`, `RiskTier` |

---

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/prism-sensors/src/crowdstrike/write.rs` | Create | CrowdStrike `SensorAdapter::write()` override (contain_host, lift_containment) |
| `crates/prism-sensors/src/armis/write.rs` | Create | Armis `SensorAdapter::write()` override (update_device_status, acknowledge_alert) |
| `crates/prism-sensors/src/claroty/write.rs` | Create | Claroty `SensorAdapter::write()` override (update_alert_status) |
| `crates/prism-sensors/src/cyberint/write.rs` | Create | Cyberint `SensorAdapter::write()` override (acknowledge_alert) |
| `crates/prism-sensors/src/crowdstrike/mod.rs` | Modify | Add `mod write;` |
| `crates/prism-sensors/src/armis/mod.rs` | Modify | Add `mod write;` |
| `crates/prism-sensors/src/claroty/mod.rs` | Modify | Add `mod write;` |
| `crates/prism-sensors/src/cyberint/mod.rs` | Modify | Add `mod write;` |
| `crates/prism-sensors/tests/write_dispatch_tests.rs` | Create | Per-sensor write dispatch integration tests using DTU clones |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | CrowdStrike API returns 401 (OAuth token expired) mid-write | SensorError::AuthFailure; caller (WriteExecutor) handles retry logic |
| EC-002 | Armis `update_device_status` — device_id column is null for some rows | Skip null rows; return `RecordWriteResult { status: Skipped }` for those rows |
| EC-003 | Claroty returns 404 for an alert_id that no longer exists | `RecordWriteResult { status: Failed, error: Some("Alert not found") }` |
| EC-004 | Sensor TOML does not declare any write endpoints | Caller dispatches to default `WriteNotImplemented`; override is never called |
| EC-005 | `records` RecordBatch has zero rows | Return `Ok(vec![])` immediately; no HTTP calls made |
| EC-006 | Write verb in `endpoint.verb` does not match any match arm in the override | Return `SensorError::WriteNotImplemented` for that specific verb |

---

## Previous Story Intelligence

**S-3.07 (Write Execution Pipeline — partial-merge predecessor):** This story implements
the concrete write dispatch that S-3.07 designed but left as `WriteNotImplemented`. Key
design decisions from S-3.07 to preserve:
- `RecordWriteResult` struct shape: `{ record_id, status, sensor_response, error }`.
- Partial batch failure is `Ok(vec![..RecordWriteResult..])` with `status: Failed` items,
  NOT `Err(...)`. This distinction is load-bearing.
- HTTP dispatch uses the adapter's existing auth client — not a new client per story.
- The `record_id_field` from `WriteEndpointSpec` identifies which column to extract from
  `RecordBatch` for building the write request body.

**S-2.07 (Sensor Adapters — merged):** The authenticated HTTP client is already implemented
for each sensor. DO NOT construct a new `reqwest::Client` — use `self.client` (or
equivalent field name in each adapter struct). Read the adapter struct definition before
implementing write overrides to confirm the client field name.

---

## Dev Notes

- Each sensor sub-module currently likely has a structure like `crowdstrike/mod.rs`,
  `crowdstrike/auth.rs`, `crowdstrike/fetch.rs`. Add `crowdstrike/write.rs` as a sibling.
- The `WriteEndpointSpec` struct (from S-1.13 / prism-spec-engine) contains `steps` array,
  `record_id_field`, `body_template`, `path_template`, `response_path`, and `success_status`
  list. Read the current definition in `prism-spec-engine` before implementing.
- For MVP: implement the most impactful 1-2 write verbs per sensor. Additional verbs can be
  added in follow-up stories without architectural changes. The important thing is that the
  `WriteNotImplemented` gap for all four sensors is closed by at least one real verb each.
- Coordinate with Bundle C (sensor TOML completion): if a sensor's TOML `[[endpoints]]`
  write section is absent or incomplete, document the gap in Open Questions and implement
  what IS specified. Do not block on Bundle C.

## Open Questions

| OQ | Question | Owner | Resolution Target |
|----|----------|-------|------------------|
| OQ-1 | Do write operations for each sensor need a distinct BC (BC-2.04.0XX per sensor) or does BC-2.04.007 (three-tier risk) + BC-2.04.008 (dry-run default) cover them fully? | Product Owner | Phase B-1b |
| OQ-2 | CrowdStrike TOML spec: does the current version include `[[endpoints.write]]` entries, or does Bundle C need to add them? | Bundle C team | Phase B-2 |
| OQ-3 | Armis write API: does the TOML spec include `update_device_status` and `acknowledge_alert` endpoints? | Bundle C team | Phase B-2 |

---

## Changelog

| Version | Burst | Date | Author | Changes |
|---------|-------|------|--------|---------|
| 1.0 | Bundle-B-Phase-B-1 | 2026-05-08 | story-writer | Initial story creation from ADR-022 §G seed (Story 3). |
