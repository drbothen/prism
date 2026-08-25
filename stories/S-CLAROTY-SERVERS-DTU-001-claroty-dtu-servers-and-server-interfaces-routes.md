---
document_type: story
story_id: S-CLAROTY-SERVERS-DTU-001
title: "Claroty DTU — Servers + Server Interfaces Routes (POST /api/v1/servers/ + /api/v1/server_interfaces/, Wave C G4 DTU parity)"
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
points: 5
# 5 pts — two endpoints but both have small scalar-only field sets (17 + 10 fields). No complex nesting.
estimated_days: 1
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
  - S-CLAROTY-SERVERS-001
# Dependency justification: S-CLAROTY-SERVERS-001 defines both the claroty_servers TOML block (17 cols,
# BC-2.16.018) and the claroty_server_interfaces TOML block (10 cols, composite PK server_name+interface_name,
# BC-2.16.019). Both DTU routes in this story replicate those TOML schema contracts. DTU cannot be built
# before the canonical column sets are finalized and live-validated.
blocks: []
gap: G4
dtu_fidelity: L3
---

# S-CLAROTY-SERVERS-DTU-001: Claroty DTU — Servers + Server Interfaces Routes (G4)

**STATUS: DRAFT STUB — Deferred post-v1. Execute with DTU-parity batch (D-2200 / D-2264).**

RG list and full ACs will be authored at scheduling time when this story is dispatched.

---

## Scope

Build two `prism-dtu-claroty` behavioral clone routes replicating the real xDome response shapes
for the Claroty servers tables:

1. `POST /api/v1/servers/` — envelope `$.servers`, Server 17-field enum. Parity target BC-2.16.018.
2. `POST /api/v1/server_interfaces/` — envelope `$.server_interfaces`, ServerInterfaces 10-field
   enum. Parity target BC-2.16.019.

**Current state:** Neither route exists in `crates/prism-dtu-claroty/src/routes/`. Both tables
were added by `S-CLAROTY-SERVERS-001` (G4, Wave C) with `SAP-2 N/A` until this story ships.

**Route contracts:** Both endpoints are POST, paginated (`offset_limit`, page_size 1000), with
`count` field present. OCSF class `inventory_info` (class_uid 5001, existing arm).

- servers endpoint fields (17): `server_name`, `server_location`, `server_status`, `site_id`,
  `model`, `os_version`, `serial_number`, `num_of_interfaces`, `management_ip`, `idrac_ip`,
  `management_mac`, `uptime_days`, `avg_traffic_past_month_mbps`, `avg_traffic_past_week_mbps`,
  `avg_traffic_past_hour_mbps`, `num_of_open_incidents`, `notes`

- server_interfaces endpoint fields (10): `server_name`, `interface_name`, `interface_status`,
  `interface_type`, `interface_connection_type`, `site_id`, `avg_traffic_past_month_mbps`,
  `avg_traffic_past_week_mbps`, `avg_traffic_past_hour_mbps`, `notes`

Composite PK for `claroty_server_interfaces`: `server_name` + `interface_name`.

---

## Deferral

**DEFERRED post-v1 per D-2200 / D-2264.** Full materialization (RG list, ACs, fixture data,
SAP-2 parity alignment) occurs when `S-ADR058-DTU-PARITY-MIGRATION-001` DTU-parity batch is
scheduled. SAP-2 probe status changes from N/A to APPLICABLE once this story ships.

---

## Authority

- `BC-2.16.018` — Claroty Servers Table behavioral contract (parity target for servers route)
- `BC-2.16.019` — Claroty Server Interfaces Table behavioral contract (parity target for
  server_interfaces route)
- `endpoint-schema-extract.md §Server, §ServerInterfaces` — field enums extract
  (`.factory/objectives/xdome-v1-validation/endpoint-schema-extract.md`)
- `S-ADR058-DTU-PARITY-MIGRATION-001` — DTU-parity governing story for scheduling this batch
- `xdome-endpoint-expansion-plan.md §Deferred DTU-Creation Stories`
  (`.factory/objectives/xdome-endpoint-expansion-plan.md`)

Section cites only — no line numbers per TD-VSDD-091.

---

## Narrative

As a test infrastructure consumer, I want behavioral clones of the Claroty xDome Servers and
Server Interfaces endpoints (`POST /api/v1/servers/` + `POST /api/v1/server_interfaces/`) at
fidelity level L3, so that integration tests and DTU-based holdout evaluation can run for both
`claroty_servers` and `claroty_server_interfaces` tables without hitting the live sensor.

---

## Acceptance Criteria

Acceptance criteria will be authored at scheduling time when BCs are assigned. At minimum, ACs
will cover:

- Clone implements `POST /api/v1/servers/` with `offset_limit` pagination
- Clone implements `POST /api/v1/server_interfaces/` with `offset_limit` pagination
- servers response: `{"servers": [...], "count": N}` with all 17 Server fields present (SAP-2)
- server_interfaces response: `{"server_interfaces": [...], "count": N}` with all 10 fields (SAP-2)
- Composite PK `server_name` + `interface_name` are non-null in server_interfaces fixture data

(RG list + AC traces authored at scheduling time per SAC-1.)

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| servers route | `crates/prism-dtu-claroty/src/routes/servers.rs` | Effectful (HTTP server route) |
| server_interfaces route | `crates/prism-dtu-claroty/src/routes/server_interfaces.rs` | Effectful (HTTP server route) |
| ServerItem / ServerInterfaceItem types | `crates/prism-dtu-claroty/src/types.rs` | Pure (data structs) |

Subsystem: SS-12 (Sensor Adapters / DTU).

---

## Edge Cases

Edge cases enumerated at scheduling time. Likely candidates:

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Malformed request body (servers) | 400 matching real API format |
| EC-002 | Empty server list | `{"servers": [], "count": 0}` |
| EC-003 | Server with no interfaces | Zero rows in server_interfaces for that server_name |

---

## Purity Classification

- Route handlers: Effectful (HTTP I/O)
- Response struct construction: Pure
- Field serialization: Pure (serde)

---

## Token Budget Estimate (MANDATORY)

Deferred stub — budget estimated at scheduling time.

| Source | Estimated tokens |
|--------|-----------------|
| This story spec | ~3,000 |
| BC-2.16.018 + BC-2.16.019 | ~3,000 |
| Existing DTU route exemplars (2 routes) | ~4,000 |
| endpoint-schema-extract.md §Server, §ServerInterfaces | ~1,000 |
| Total estimate | ~11,000 |

Within 20-30% of agent context window. No split required.

---

## Tasks (MANDATORY)

Tasks enumerated at scheduling time. High-level sequence:

1. Add `ServerItem` (17 fields) + `ServerInterfaceItem` (10 fields) to `crates/prism-dtu-claroty/src/types.rs`
2. Create `crates/prism-dtu-claroty/src/routes/servers.rs` with POST handler + fixture
3. Create `crates/prism-dtu-claroty/src/routes/server_interfaces.rs` with POST handler + fixture
4. Register both routes in `crates/prism-dtu-claroty/src/routes/mod.rs`
5. Write SAP-2 parity tests (one per route)
6. Run `just iter prism-dtu-claroty`

---

## Previous Story Intelligence (MANDATORY)

N/A — first stub for these tables in the DTU-parity batch.

At scheduling time: review `S-CLAROTY-SERVERS-001` merge notes for the final TOML column sets
and any Tier-1/Tier-2 field routing decisions that affect how DTU response fields must be named.

---

## Architecture Compliance Rules (MANDATORY)

At scheduling time, extract from architecture docs and ADRs. Key rules:

- DTU routes in `crates/prism-dtu-claroty/src/routes/<table>.rs`; one file per route
- Response types in `crates/prism-dtu-claroty/src/types.rs`
- All public response types require `#[non_exhaustive]`
- ADR-058 OCSF routing active; DTU field names must match mapped schema
- `reqwest` dependency (if needed): `default-features = false, features = ["rustls-tls"]` (ADR-050)

---

## Library & Framework Requirements (MANDATORY)

Library versions confirmed at scheduling time from `dependency-graph.md`.

| Library | Version pin | Usage |
|---------|------------|-------|
| `axum` | workspace | HTTP route handlers |
| `serde` / `serde_json` | workspace | Response serialization |

---

## File Structure Requirements (MANDATORY)

| Action | File | Notes |
|--------|------|-------|
| CREATE | `crates/prism-dtu-claroty/src/routes/servers.rs` | POST handler + fixture |
| CREATE | `crates/prism-dtu-claroty/src/routes/server_interfaces.rs` | POST handler + fixture |
| MODIFY | `crates/prism-dtu-claroty/src/routes/mod.rs` | Register both routes |
| MODIFY | `crates/prism-dtu-claroty/src/types.rs` | Add ServerItem + ServerInterfaceItem structs |
