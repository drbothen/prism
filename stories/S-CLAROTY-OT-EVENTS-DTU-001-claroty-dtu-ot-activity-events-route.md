---
document_type: story
story_id: S-CLAROTY-OT-EVENTS-DTU-001
title: "Claroty DTU — OT Activity Events Route (POST /api/v1/ot_activity_events/, Wave A G2 DTU parity)"
level: "L4"
wave: post-v1-dtu-parity
epic_id: EPIC-OCSF-ROUTING
# DTU-parity anchor: S-ADR058-DTU-PARITY-MIGRATION-001 is the parent story governing post-v1 DTU-parity work.
priority: P2
status: draft
# BC status: pending PO authorship — behavioral_contracts must be non-empty before status: ready.
version: "1.0"
producer: story-writer
timestamp: "2026-08-24T00:00:00Z"
modified: "2026-08-24"
phase: 3
cycle: v1.0.0-brownfield
behavioral_contracts: []
# BC status: pending PO authorship
verification_properties: []
holdout_scenarios: []
points: 5
estimated_days: 1
crates_touched: [prism-dtu-claroty]
target_module: "prism-dtu-claroty"
subsystems: [SS-12]
tdd_mode: facade
# facade: DTU clone routes are structural fakes (mock server pattern). Confirmed at scheduling.
assumption_validations: []
risk_mitigations: []
traces_to: []
# traces_to: populated at scheduling time when BCs are authored.
inputs: []
input-hash: "[pending-recompute]"
# inputs list and input-hash computed at scheduling time when the story is materialized.
depends_on:
  - S-CLAROTY-OT-EVENTS-001
# Dependency justification: S-CLAROTY-OT-EVENTS-001 defines the claroty_ot_activity_events TOML block
# (21 columns, BC-2.16.016), which is the schema contract the DTU clone route must replicate. The DTU
# cannot be built before the canonical TOML column set is finalized and live-validated.
blocks: []
gap: G2
dtu_fidelity: L3
# L3 target: stateful route replicating real API error responses + field shapes.
---

# S-CLAROTY-OT-EVENTS-DTU-001: Claroty DTU — OT Activity Events Route (G2)

**STATUS: DRAFT STUB — Deferred post-v1. Execute with DTU-parity batch (D-2200 / D-2264).**

RG list and full ACs will be authored at scheduling time when this story is dispatched.

---

## Scope

Build the `prism-dtu-claroty` behavioral clone route for `POST /api/v1/ot_activity_events/`, replicating
the real xDome response shape (envelope `$.ot_activity_events`, OTActivityEvent 23-field enum per
`endpoint-schema-extract.md §OTActivityEvent`) so that SAP-2 DTU↔TOML parity validation and
DTU-based holdout evaluation can execute for the `claroty_ot_activity_events` table.

**Current state:** No `/api/v1/ot_activity_events/` route exists in
`crates/prism-dtu-claroty/src/routes/`. The table was added by `S-CLAROTY-OT-EVENTS-001` (G2,
Wave A) with `SAP-2 N/A` until this story ships.

**Route contract:** POST, paginated (`offset_limit`, page_size 1000), envelope key
`ot_activity_events`, `count` field present. OCSF class `detection_finding` (class_uid 2004).
23 OTActivityEvent fields: `detection_time`, `event_type`, `related_alert_ids`, `description`,
`dest_asset_id`, `dest_ip`, `dest_device_type`, `dest_device_name`, `dest_site_name`,
`dest_network`, `protocol`, `dest_port`, `source_port`, `source_asset_id`, `source_ip`,
`source_device_type`, `source_username`, `source_device_name`, `source_site_name`,
`source_network`, `mode`, `event_id`, `ip_protocol`.

---

## Deferral

**DEFERRED post-v1 per D-2200 / D-2264.** Full materialization (RG list, ACs, fixture data,
SAP-2 parity alignment) occurs when `S-ADR058-DTU-PARITY-MIGRATION-001` DTU-parity batch is
scheduled. SAP-2 probe status changes from N/A to APPLICABLE once this story ships.

---

## Authority

- `BC-2.16.016` — Claroty OT Activity Events Table behavioral contract (parity target)
- `endpoint-schema-extract.md §OTActivityEvent` — 23-field OTActivityEvent fields_enum
  (`.factory/objectives/xdome-v1-validation/endpoint-schema-extract.md`)
- `S-ADR058-DTU-PARITY-MIGRATION-001` — DTU-parity governing story for scheduling this batch
- `xdome-endpoint-expansion-plan.md §Deferred DTU-Creation Stories`
  (`.factory/objectives/xdome-endpoint-expansion-plan.md`)

Section cites only — no line numbers per TD-VSDD-091.

---

## Narrative

As a test infrastructure consumer, I want a behavioral clone of the Claroty xDome OT Activity
Events endpoint (`POST /api/v1/ot_activity_events/`) at fidelity level L3, so that integration
tests and DTU-based holdout evaluation can run against realistic OT activity event data without
hitting the live monroe sensor.

---

## Acceptance Criteria

Acceptance criteria will be authored at scheduling time when BCs are assigned and this story
transitions from draft to ready. At minimum, ACs will cover:

- Clone implements `POST /api/v1/ot_activity_events/` with the paginated `offset_limit` contract
- Response envelope is `{"ot_activity_events": [...], "count": N}` matching the real API shape
- All 23 OTActivityEvent fields are present in each response item (SAP-2 parity)
- Clone returns appropriate error codes for malformed requests

(RG list + AC traces authored at scheduling time per SAC-1.)

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| ot_activity_events route | `crates/prism-dtu-claroty/src/routes/ot_activity_events.rs` | Effectful (HTTP server route) |
| OtActivityEvent response type | `crates/prism-dtu-claroty/src/types.rs` | Pure (data struct) |

Subsystem: SS-12 (Sensor Adapters / DTU).

---

## Edge Cases

Edge cases will be enumerated at scheduling time. Likely candidates (from SAP-2 probe discipline):

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Malformed request body | 400 matching real API error format |
| EC-002 | Empty result set (no events) | `{"ot_activity_events": [], "count": 0}` |
| EC-003 | Pagination boundary (offset > count) | Empty items array, count unchanged |

---

## Purity Classification

- Route handler: Effectful (HTTP I/O)
- Response struct construction: Pure
- Field serialization: Pure (serde)

---

## Token Budget Estimate (MANDATORY)

Deferred stub — budget estimated at scheduling time.

| Source | Estimated tokens |
|--------|-----------------|
| This story spec | ~3,000 |
| BC-2.16.016 | ~2,000 |
| Existing DTU route exemplars (2 routes) | ~4,000 |
| endpoint-schema-extract.md (relevant section) | ~1,000 |
| Total estimate | ~10,000 |

Well within 20-30% of agent context window. No split required.

---

## Tasks (MANDATORY)

Tasks enumerated at scheduling time. High-level sequence (from DTU clone story pattern):

1. Add `OtActivityEventItem` response struct to `crates/prism-dtu-claroty/src/types.rs`
2. Create `crates/prism-dtu-claroty/src/routes/ot_activity_events.rs` with POST handler
3. Register route in `crates/prism-dtu-claroty/src/routes/mod.rs`
4. Add fixture data (seed events covering all 23 fields)
5. Write SAP-2 parity test (field-presence assertion for each TOML column)
6. Run `just iter prism-dtu-claroty` to verify

---

## Previous Story Intelligence (MANDATORY)

N/A — first DTU-parity stub; no predecessor in this DTU-parity sub-batch.

At scheduling time: review the merge notes from `S-DEMO-CLAROTY-DAR-001` (device_alert_relations
DTU route) and `S-DEMO-CLAROTY-AUDIT-DTU-001` (audit_log DTU route) for lessons on route
registration and fixture patterns in `prism-dtu-claroty`.

---

## Architecture Compliance Rules (MANDATORY)

At scheduling time, extract from `architecture/module-decomposition.md` and relevant ADRs.
Key rules applicable to DTU clone routes (from existing patterns):

- DTU routes live in `crates/prism-dtu-claroty/src/routes/<table>.rs`
- Response types live in `crates/prism-dtu-claroty/src/types.rs`
- All public response types require `#[non_exhaustive]` (CLAUDE.md §non_exhaustive discipline)
- ADR-058 OCSF field-name routing is active post-ROUTING-001; DTU response field names must be
  consistent with the mapped schema
- `reqwest` dependency: `default-features = false, features = ["rustls-tls"]` (ADR-050)

---

## Library & Framework Requirements (MANDATORY)

Library versions confirmed at scheduling time from `dependency-graph.md`. Expected dependencies
(matching existing `prism-dtu-claroty` pattern):

| Library | Version pin | Usage |
|---------|------------|-------|
| `axum` | workspace | HTTP route handler |
| `serde` / `serde_json` | workspace | Response serialization |
| `uuid` | workspace (if needed) | Event ID generation |

---

## File Structure Requirements (MANDATORY)

| Action | File | Notes |
|--------|------|-------|
| CREATE | `crates/prism-dtu-claroty/src/routes/ot_activity_events.rs` | POST handler + fixture |
| MODIFY | `crates/prism-dtu-claroty/src/routes/mod.rs` | Register route |
| MODIFY | `crates/prism-dtu-claroty/src/types.rs` | Add OtActivityEventItem struct |
