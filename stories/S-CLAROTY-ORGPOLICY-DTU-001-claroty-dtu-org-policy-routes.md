---
document_type: story
story_id: S-CLAROTY-ORGPOLICY-DTU-001
title: "Claroty DTU — Org Policy Routes (4 endpoints: organization_zones, zone_policies, fw_groups, fw_group_policies, Wave C G5 DTU parity)"
level: "L4"
wave: post-v1-dtu-parity
epic_id: EPIC-OCSF-ROUTING
# DTU-parity anchor: S-ADR058-DTU-PARITY-MIGRATION-001 governs post-v1 DTU-parity batch scheduling.
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
points: 8
# 8 pts — 4 routes across 2 endpoint pairs; Json column types in fixture data add complexity.
# Adjust at scheduling time.
estimated_days: 2
crates_touched: [prism-dtu-claroty]
target_module: "prism-dtu-claroty"
subsystems: [SS-12]
tdd_mode: facade
# facade: DTU clone routes are structural fakes (mock server pattern).
assumption_validations: []
risk_mitigations: []
traces_to: []
inputs: []
input-hash: "[pending-recompute]"
# inputs list and input-hash computed at scheduling time when the story is materialized.
depends_on:
  - S-CLAROTY-ORGPOLICY-001
# Dependency justification: S-CLAROTY-ORGPOLICY-001 defines all 4 TOML blocks (zone 11 cols, zone
# policies 13 cols, fw groups 11 cols, fw group policies 13 cols) plus BCs BC-2.16.020 + BC-2.16.021.
# DTU cannot be built before canonical column sets are finalized and live-validated.
blocks: []
gap: G5
dtu_fidelity: L3
---

# S-CLAROTY-ORGPOLICY-DTU-001: Claroty DTU — Org Policy Routes (G5)

**STATUS: DRAFT STUB — Deferred post-v1. Execute with DTU-parity batch (D-2200 / D-2264).**

RG list and full ACs will be authored at scheduling time when this story is dispatched.

---

## Scope

Build four `prism-dtu-claroty` behavioral clone routes for the Claroty xDome organization policy
tables, replicating real xDome response shapes per `endpoint-schema-extract.md §Organization*`
sections and spike-findings.md §Spike-3. OCSF class `entity_management` (class_uid 3004,
existing arm) for all four routes.

**URL↔envelope asymmetry (critical):** The fw_groups and fw_group_policies endpoints have a
URL/envelope-key mismatch that the DTU routes must replicate correctly:
- URL `/api/v1/organization_fw_groups/` → envelope key `organization_firewall_groups`
- URL `/api/v1/organization_fw_group_policies/` → envelope key `organization_firewall_policies`

**Routes to implement:**

1. `POST /api/v1/organization_zones/` — envelope `$.organization_zones`
   (11 fields; 1 Json: `device_conditions`)
2. `POST /api/v1/organization_zone_policies/` — envelope `$.organization_zone_policies`
   (13 fields; 3 Json: `communication_conditions`, `related_alerts_ids`, `applied_zone_pairs`)
3. `POST /api/v1/organization_fw_groups/` — envelope `$.organization_firewall_groups` (ASYMMETRY)
   (11 fields; 1 Json: `device_conditions`)
4. `POST /api/v1/organization_fw_group_policies/` — envelope `$.organization_firewall_policies` (ASYMMETRY)
   (13 fields; 3 Json: `communication_conditions`, `related_alerts_ids`, `applied_group_pairs`)

**Current state:** None of these 4 routes exist in `crates/prism-dtu-claroty/src/routes/`.
All 4 tables were added by `S-CLAROTY-ORGPOLICY-001` (G5, Wave C) with `SAP-2 N/A` until this
story ships.

---

## Deferral

**DEFERRED post-v1 per D-2200 / D-2264.** Full materialization (RG list, ACs, fixture data
including Json columns, SAP-2 parity alignment) occurs when `S-ADR058-DTU-PARITY-MIGRATION-001`
DTU-parity batch is scheduled. SAP-2 probe status changes from N/A to APPLICABLE once this story
ships.

---

## Authority

- `BC-2.16.020` — Claroty Organization Zones + Zone Policies Table behavioral contracts (parity
  target for routes 1 and 2)
- `BC-2.16.021` — Claroty Organization Firewall Groups + Firewall Group Policies Table behavioral
  contracts (parity target for routes 3 and 4)
- `endpoint-schema-extract.md §OrganizationZones, §OrganizationZonePolicies, §OrganizationFirewallGroups, §OrganizationFirewallGroupPolicies`
  (`.factory/objectives/xdome-v1-validation/endpoint-schema-extract.md`)
- `endpoint-spike-findings.md §Spike-3 — Organization-Policy Nested Field Types` — Json column
  classification + URL↔envelope asymmetry (`.factory/objectives/xdome-v1-validation/endpoint-spike-findings.md`)
- `S-ADR058-DTU-PARITY-MIGRATION-001` — DTU-parity governing story for scheduling this batch
- `xdome-endpoint-expansion-plan.md §Deferred DTU-Creation Stories`
  (`.factory/objectives/xdome-endpoint-expansion-plan.md`)

Section cites only — no line numbers per TD-VSDD-091.

---

## Narrative

As a test infrastructure consumer, I want behavioral clones of the 4 Claroty xDome organization
policy endpoints at fidelity level L3, so that integration tests and DTU-based holdout evaluation
can run for the 4 `claroty_organization_*` tables without hitting the live sensor.

---

## Acceptance Criteria

Acceptance criteria will be authored at scheduling time when BCs are assigned. At minimum, ACs
will cover:

- Clone implements all 4 POST routes with `offset_limit` pagination and `count` field
- URL↔envelope key asymmetry is correctly implemented: fw_groups URL → `organization_firewall_groups`
  envelope; fw_group_policies URL → `organization_firewall_policies` envelope
- Json columns (`device_conditions`, `communication_conditions`, `related_alerts_ids`,
  `applied_zone_pairs`, `applied_group_pairs`) are populated in fixture data as JSON arrays
- SAP-2 parity: all TOML columns have matching fields in each DTU route response
- `last_update` (no trailing d) vs `last_updated` (with trailing d) field naming is consistent
  with TOML spec per spike-findings §Spike-3

(RG list + AC traces authored at scheduling time per SAC-1.)

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| organization_zones route | `crates/prism-dtu-claroty/src/routes/organization_zones.rs` | Effectful |
| organization_zone_policies route | `crates/prism-dtu-claroty/src/routes/organization_zone_policies.rs` | Effectful |
| organization_fw_groups route | `crates/prism-dtu-claroty/src/routes/organization_fw_groups.rs` | Effectful |
| organization_fw_group_policies route | `crates/prism-dtu-claroty/src/routes/organization_fw_group_policies.rs` | Effectful |
| Org policy response types | `crates/prism-dtu-claroty/src/types.rs` | Pure (data structs) |

Subsystem: SS-12 (Sensor Adapters / DTU).

---

## Edge Cases

Edge cases enumerated at scheduling time. Likely candidates:

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Json column is empty array | `"device_conditions": []` (not null) |
| EC-002 | fw_groups URL used instead of fw_group_policies | 404 (route not registered at wrong URL) |
| EC-003 | Empty zone list | `{"organization_zones": [], "count": 0}` |

---

## Purity Classification

- Route handlers: Effectful (HTTP I/O)
- Response struct construction: Pure
- Json field serialization: Pure (serde_json::Value)

---

## Token Budget Estimate (MANDATORY)

Deferred stub — budget estimated at scheduling time.

| Source | Estimated tokens |
|--------|-----------------|
| This story spec | ~3,500 |
| BC-2.16.020 + BC-2.16.021 | ~4,000 |
| Existing DTU route exemplars (2 routes) | ~4,000 |
| endpoint-schema-extract.md §Organization* (4 sections) | ~2,000 |
| endpoint-spike-findings.md §Spike-3 | ~2,000 |
| Total estimate | ~15,500 |

Within 20-30% of agent context window. No split required; 4 routes are structurally parallel.

---

## Tasks (MANDATORY)

Tasks enumerated at scheduling time. High-level sequence:

1. Add 4 response types to `crates/prism-dtu-claroty/src/types.rs` (with Json fields as
   `serde_json::Value`)
2. Create 4 route files in `crates/prism-dtu-claroty/src/routes/`
3. Register all 4 routes in `crates/prism-dtu-claroty/src/routes/mod.rs`
4. Add fixture data for each route including Json-typed columns
5. Write SAP-2 parity tests (one per route)
6. Run `just iter prism-dtu-claroty`

---

## Previous Story Intelligence (MANDATORY)

N/A — first stub for these tables in the DTU-parity batch.

At scheduling time: review `S-CLAROTY-ORGPOLICY-001` merge notes for the final TOML column sets
and especially the `last_update` (no d) vs `last_updated` (with d) field naming that varies
across the 4 tables.

---

## Architecture Compliance Rules (MANDATORY)

At scheduling time, extract from architecture docs and ADRs. Key rules:

- DTU routes in `crates/prism-dtu-claroty/src/routes/<table>.rs`; one file per route
- Response types in `crates/prism-dtu-claroty/src/types.rs`
- All public response types require `#[non_exhaustive]`
- Json columns are `serde_json::Value` in the struct, serialized as JSON arrays in fixtures
- URL↔envelope asymmetry: fw routes MUST NOT use `organization_fw_*` envelope key names;
  use `organization_firewall_*` names
- ADR-058 OCSF routing active post-ROUTING-001

---

## Library & Framework Requirements (MANDATORY)

Library versions confirmed at scheduling time from `dependency-graph.md`.

| Library | Version pin | Usage |
|---------|------------|-------|
| `axum` | workspace | HTTP route handlers |
| `serde` / `serde_json` | workspace | Response serialization including Json columns |

---

## File Structure Requirements (MANDATORY)

| Action | File | Notes |
|--------|------|-------|
| CREATE | `crates/prism-dtu-claroty/src/routes/organization_zones.rs` | POST handler + fixture |
| CREATE | `crates/prism-dtu-claroty/src/routes/organization_zone_policies.rs` | POST handler + fixture |
| CREATE | `crates/prism-dtu-claroty/src/routes/organization_fw_groups.rs` | POST handler + fixture; envelope key = `organization_firewall_groups` |
| CREATE | `crates/prism-dtu-claroty/src/routes/organization_fw_group_policies.rs` | POST handler + fixture; envelope key = `organization_firewall_policies` |
| MODIFY | `crates/prism-dtu-claroty/src/routes/mod.rs` | Register all 4 routes |
| MODIFY | `crates/prism-dtu-claroty/src/types.rs` | Add 4 response struct types |
