---
document_type: story
story_id: S-CLAROTY-DTU-PARITY-001
title: "Claroty DTU 14-table parity (post-rc.1) — bring prism-dtu-claroty + demo-server to full 14-table fidelity"
wave: post-v1.0.0-rc.1
epic_id: EPIC-OCSF-ROUTING
priority: P1
status: draft
version: "1.0"
level: "L4"
producer: story-writer
timestamp: "2026-09-04T00:00:00Z"
modified: "2026-09-04"
phase: 3
cycle: post-v1.0.0-rc.1
# Cycle note: POST-v1.0.0-rc.1. Deferred out of rc.1 per human decision 2026-09-04.
# Execute after v1.0.0-rc.1 ships and is validated on the live xDome/monroe RC gate.
behavioral_contracts: []
# BC status: pending PO authorship — behavioral_contracts must be non-empty before status: ready.
# Candidates at dispatch time: BC-2.16.016..BC-2.16.022 (Claroty G2-G6 table BCs).
# BC-2.16.015 (alerts) governs an existing G1 route and is out of scope unless SAP-2 finds drift.
# PO must author and anchor BCs for any G2-G6 surfaces not yet covered before dispatch.
verification_properties: []
holdout_scenarios: []
points: 3
# Points: 3 for governing coordination + demo-server seeding integration (Phase C).
# Sub-story implementation work distributed across 5 child stories (31 pts total sub-stories).
# Batch total: ~34 pts. Governing story own work: Phase C integration only.
estimated_days: 1
crates_touched: [prism-dtu-claroty, prism-dtu-demo-server]
target_module: "prism-dtu-claroty"
subsystems: [SS-12]
# Subsystem anchor justification: SS-12 (Sensor Adapters / DTU) owns prism-dtu-claroty and
# prism-dtu-demo-server per ARCH-INDEX Subsystem Registry. All 5 missing routes are DTU clone
# surfaces within the Claroty xDome sensor adapter boundary.
tdd_mode: strict
assumption_validations: []
risk_mitigations: []
traces_to: []
inputs:
  - "crates/prism-sensors/specs/claroty.sensor.toml"
  - ".factory/objectives/xdome-v1-validation/endpoint-schema-extract.md"
input-hash: "[pending-recompute]"
# input-hash: computed at scheduling/dispatch time.
depends_on: []
# depends_on: empty. This is the entry-point governing story for the Claroty DTU parity batch.
# No upstream story prerequisites. The 5 sub-stories each depend on their respective TOML
# authoring stories (already complete: S-CLAROTY-OT-EVENTS-001, S-CLAROTY-DEVVULNREL-001,
# S-CLAROTY-SERVERS-001, S-CLAROTY-ORGPOLICY-001, S-CLAROTY-ACLPOLICY-001).
blocks: [S-REL-004]
# Dependency anchor justification: S-REL-004 (demo-bundle packaging) cannot function until
# prism-dtu-claroty serves all 14 Claroty tables declared in claroty.sensor.toml. The demo
# bundle runs the DTU-backed demo against the full sensor spec. If the DTU serves only 7 of
# the 14 tables, any demo query touching G2-G6 surfaces fails at runtime. S-REL-004 is gated
# behind this governing story; it must not be dispatched until Phase C integration passes.
---

# S-CLAROTY-DTU-PARITY-001: Claroty DTU 14-table parity (post-rc.1)

**STATUS: DRAFT GOVERNING STORY — DEFERRED post-v1.0.0-rc.1 per human decision 2026-09-04. Execute this story batch AFTER v1.0.0-rc.1 ships and is validated on the live xDome/monroe RC gate. This story BLOCKS S-REL-004 (demo-bundle packaging).**

---

## Origin

`crates/prism-sensors/specs/claroty.sensor.toml` declares 14 `[[tables]]` blocks (G1 through G6).
`crates/prism-dtu-claroty` implements only 7 routes: `alerts`, `audit_log`, `device_alert_relations`,
`devices`, `tags`, `vulnerabilities`, and `mod.rs`. The 5 G2–G6 table surfaces have no DTU routes:

| Gap | Tables | Sub-story |
|-----|--------|-----------|
| G2 | OT activity events | S-CLAROTY-OT-EVENTS-DTU-001 |
| G3 | Device-vulnerability relations | S-CLAROTY-DEVVULNREL-DTU-001 |
| G4 | Servers + server interfaces | S-CLAROTY-SERVERS-DTU-001 |
| G5 | Org policy (zones, zone_policies, fw_groups, fw_group_policies) | S-CLAROTY-ORGPOLICY-DTU-001 |
| G6 | ACL policies | S-CLAROTY-ACLPOLICY-DTU-001 |

**Human decision (2026-09-04):** This entire batch is deferred out of v1.0.0-rc.1. rc.1 ships
WITHOUT the demo bundle (S-REL-004 deferred), validated on the live xDome/monroe RC gate. The
Claroty DTU parity work and demo bundle land post-rc.1.

**Scope constraint (human decision 2026-09-04):** CLAROTY PARITY ONLY. This is NOT a broader
multi-sensor DTU sweep and NOT a drift audit of existing G1 routes. The 5 sub-stories implement
only the 5 missing G2–G6 DTU routes. Existing routes (alerts, audit_log, device_alert_relations,
devices, tags, vulnerabilities) are out of scope unless SAP-2 finds parity defects during this batch.

---

## Narrative

As a demo operator, I want `prism-dtu-claroty` to serve all 14 Claroty xDome table endpoints
and `prism-dtu-demo-server` to seed non-empty fixture data for each, so that the full DTU-backed
demo can run against the complete `claroty.sensor.toml` spec without requiring a live xDome
connection.

---

## Behavioral Contracts

Behavioral contracts assigned at scheduling/dispatch time. PO must author and anchor BCs for
any G2-G6 surface not yet covered before transitioning this story to `ready`.

Candidates from existing Claroty sensor BCs (verify currency at dispatch time):

| BC | Coverage | Relevance |
|----|----------|-----------|
| BC-2.16.016 | OT Activity Events Table | G2 parity target — S-CLAROTY-OT-EVENTS-DTU-001 |
| BC-2.16.017 | Device-Vulnerability Relations Table | G3 parity target — S-CLAROTY-DEVVULNREL-DTU-001 |
| BC-2.16.018 | Servers Table | G4 parity target — S-CLAROTY-SERVERS-DTU-001 |
| BC-2.16.019 | Server Interfaces Table | G4 parity target — S-CLAROTY-SERVERS-DTU-001 |
| BC-2.16.020 | Org Policy (zones, zone_policies) | G5 parity target — S-CLAROTY-ORGPOLICY-DTU-001 |
| BC-2.16.021 | Org Policy (fw_groups, fw_group_policies) | G5 parity target — S-CLAROTY-ORGPOLICY-DTU-001 |
| BC-2.16.022 | ACL Policies Table | G6 parity target — S-CLAROTY-ACLPOLICY-DTU-001 |

---

## Token Budget Estimate (MANDATORY)

| Source | Estimated Tokens |
|--------|-----------------|
| This governing story spec | ~4,000 |
| `claroty.sensor.toml` (all 14 tables, for integration verification) | ~3,000 |
| `endpoint-schema-extract.md` (G2-G6 sections) | ~5,000 |
| BC files for G2-G6 (7 BCs, loaded at dispatch time) | ~14,000 |
| Existing DTU exemplar routes (alerts + devices for pattern reference) | ~4,000 |
| Phase C only (after sub-stories merge — governing story loads sub-story specs) | ~5,000 |
| **Phase C integration agent total** | ~35,000 |

Phase C integration agent (after sub-stories complete) loads only this story + claroty.sensor.toml +
the 5 sub-story completion notes + existing mod.rs. Well within 20-30% of agent context window.

---

## Tasks (MANDATORY)

**Phase A — Materialization (orchestrator step, at scheduling time):**

1. Verify or author BCs for the 5 new table surfaces (BC-2.16.016..BC-2.16.022). PO step.
   Confirm each sub-story `behavioral_contracts:` array is populated before marking `ready`.
2. For each of the 5 sub-stories, populate `RG list` and full `Acceptance Criteria` per SAC-1.
   Each sub-story is `tdd_mode: facade`; Red Gate density check is replaced by mutation testing
   at the wave gate per BC-8.30.001.
3. Verify sub-story `depends_on` references are still valid (TOML stories all shipped; no blockers).
4. Update each sub-story's `# DTU-parity anchor:` comment to reference S-CLAROTY-DTU-PARITY-001
   as the governing story for this batch (in addition to or replacing the S-ADR058 reference).

**Phase B — Execution (dispatch sub-stories, topological order):**

5. Deliver S-CLAROTY-OT-EVENTS-DTU-001 (G2 — OT activity events route, 5 pts).
6. Deliver S-CLAROTY-DEVVULNREL-DTU-001 (G3 — device-vulnerability relations route, 8 pts).
7. Deliver S-CLAROTY-SERVERS-DTU-001 (G4 — servers + server interfaces routes, 5 pts).
8. Deliver S-CLAROTY-ORGPOLICY-DTU-001 (G5 — org policy routes: 4 endpoints, 8 pts).
9. Deliver S-CLAROTY-ACLPOLICY-DTU-001 (G6 — ACL policies route, 5 pts).

Note: All 5 sub-stories have independent `routes/<table>.rs` files and may run in parallel
worktrees. G2 and G3 share no file with G4/G5/G6. The only shared file is `routes/mod.rs`
and `types.rs` — sub-stories must merge sequentially to avoid conflicts on those files, or
coordinate via separate PR branches with rebase sequencing.

**Phase C — Integration + Demo-Server Seeding (this story's own implementation work):**

10. After all 5 sub-stories merge: read `crates/prism-dtu-claroty/src/routes/mod.rs` and verify
    all 5 new routes are registered. Run `just iter prism-dtu-claroty` — zero test failures.
11. Add demo-server fixture seeding for the 5 new table surfaces in `crates/prism-dtu-demo-server/`.
    Each new endpoint must return at least 2 fixture records in the demo scenario layer. Reference
    existing seeding in `prism-dtu-demo-server` for the G1 routes as the pattern.
12. Run the SAP-2 probe across all 5 new routes (per CLAUDE.md §SAP-2):
    - For each new table: grep TOML column names vs DTU types.rs struct fields vs wire-emission site
    - Zero P1-CRITICAL findings. Medium (missing coverage) findings documented in sub-story closeout.
13. Execute a demo run against DTU via `prism-dtu-demo-server` and verify all 14 tables return
    non-empty rows. Capture query output as AC-004 evidence.

---

## Acceptance Criteria

(BCs assigned at scheduling time per Phase A. Traces below use candidate BC IDs; PO confirms.)

### AC-001: prism-dtu-claroty serves all 14 Claroty table endpoints
`crates/prism-dtu-claroty/src/routes/mod.rs` registers all 5 new routes. `just iter prism-dtu-claroty`
passes with zero failures. The 7 existing routes remain unaffected.
(traces to BC-2.16.016..BC-2.16.022 — assigned at scheduling time)

### AC-002: SAP-2 DTU↔TOML column parity for all 5 new routes
For each of the 5 new DTU routes, every column declared in `claroty.sensor.toml` for that table
maps to a field in the DTU response struct AND appears in the wire-emission site of the route
handler. Zero P1-CRITICAL SAP-2 findings. (SAP-2 probe run per CLAUDE.md §SAP-2 rule 6.)
(traces to BC-2.16.016..BC-2.16.022 — assigned at scheduling time)

### AC-003: Demo-server fixture data non-empty for all 5 new table surfaces
After `prism-dtu-demo-server` starts, a POST to each of the 5 new DTU endpoints returns a
non-empty items array with at least 2 fixture records. Tables: OT events (G2), device-vuln
relations (G3), servers + server interfaces (G4), org policy all-4-endpoint-types (G5), ACL
policies (G6).
(traces to BC-2.16.016..BC-2.16.022 — assigned at scheduling time)

### AC-004: Full 14-table demo run completes without errors via DTU
Running the demo end-to-end against `prism-dtu-demo-server`: all 14 Claroty tables return
non-empty rows when queried. No `table not found` or `route not implemented` errors for any
table in `claroty.sensor.toml`. Evidence captured as query output in story closeout.
(traces to BC-2.16.015..BC-2.16.022 — assigned at scheduling time)

---

## Implementation Units (Sub-stories)

These 5 draft stub stories are the execution units for this governing story. All have
`tdd_mode: facade` (DTU mock server pattern). Full materialization (BC authorship, RG list,
ACs) occurs at Phase A scheduling time.

| Story | Gap | Endpoint(s) | Points | Depends On |
|-------|-----|-------------|--------|------------|
| S-CLAROTY-OT-EVENTS-DTU-001 | G2 | POST /api/v1/ot_activity_events/ | 5 | S-CLAROTY-OT-EVENTS-001 |
| S-CLAROTY-DEVVULNREL-DTU-001 | G3 | POST /api/v1/device_vulnerability_relations/ | 8 | S-CLAROTY-DEVVULNREL-001 |
| S-CLAROTY-SERVERS-DTU-001 | G4 | POST /api/v1/servers/ + /api/v1/server_interfaces/ | 5 | S-CLAROTY-SERVERS-001 |
| S-CLAROTY-ORGPOLICY-DTU-001 | G5 | POST /api/v1/organization_zones/ (+ 3 more) | 8 | S-CLAROTY-ORGPOLICY-001 |
| S-CLAROTY-ACLPOLICY-DTU-001 | G6 | POST /api/v1/organization_acl_policies/ | 5 | S-CLAROTY-ACLPOLICY-001 |

Sub-story points total: 31. Governing story Phase C: 3. Batch total: ~34 points.

All 5 sub-stories' `depends_on` TOML stories are already shipped. Sub-stories have no
inter-dependencies and may run in parallel worktrees subject to mod.rs merge sequencing.

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| ot_activity_events route | `crates/prism-dtu-claroty/src/routes/ot_activity_events.rs` | Effectful |
| device_vulnerability_relations route | `crates/prism-dtu-claroty/src/routes/device_vulnerability_relations.rs` | Effectful |
| servers route | `crates/prism-dtu-claroty/src/routes/servers.rs` | Effectful |
| server_interfaces route | `crates/prism-dtu-claroty/src/routes/server_interfaces.rs` | Effectful |
| org_policy routes (4 endpoints) | `crates/prism-dtu-claroty/src/routes/org_policy.rs` | Effectful |
| acl_policies route | `crates/prism-dtu-claroty/src/routes/acl_policies.rs` | Effectful |
| routes/mod.rs (registration) | `crates/prism-dtu-claroty/src/routes/mod.rs` | Pure (router config) |
| Response structs | `crates/prism-dtu-claroty/src/types.rs` | Pure |
| Demo-server seeding (Phase C) | `crates/prism-dtu-demo-server/src/` | Effectful |

Subsystem: SS-12 owns this story's scope per ARCH-INDEX Subsystem Registry because
prism-dtu-claroty and prism-dtu-demo-server are SS-12 (Sensor Adapters / DTU) crates.

---

## Purity Classification

| Module | Classification | Justification |
|--------|----------------|---------------|
| DTU route handlers | Effectful | HTTP server I/O |
| Response structs | Pure | serde-serialized data structs; no I/O |
| Fixture data functions | Pure | Static data construction |
| Demo-server seeding | Effectful | HTTP server startup + fixture data injection |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Sub-story blocked on PO BC authorship | Governing story stays draft; sub-stories remain draft until BCs authored; orchestrator blocks dispatch |
| EC-002 | SAP-2 CRITICAL parity gap found during G2-G6 implementation | Fix in-scope per Canonical Principle Rule 4; sub-story does not ship until gap is closed |
| EC-003 | G5 org-policy has 4 endpoints — mod.rs registration complexity | S-CLAROTY-ORGPOLICY-DTU-001 registers all 4 in a single story; Phase C verifies all 4 are reachable |
| EC-004 | Demo-server seeding produces duplicate fixture IDs across G2-G6 tables | Use per-table ID prefix or uuid; ensure IDs are unique within each table |
| EC-005 | G4 servers + server_interfaces share struct fields — consolidation risk | S-CLAROTY-SERVERS-DTU-001 handles both; Phase C verifies separate route registrations |
| EC-006 | claroty.sensor.toml columns drifted between story authorship and dispatch | Re-run SAP-2 probe at dispatch time; claroty.sensor.toml is the authoritative column list |
| EC-007 | mod.rs merge conflicts across parallel sub-story worktrees | Sequence mod.rs merges; use rebase with --no-ff to preserve individual PRs |

---

## Previous Story Intelligence (MANDATORY)

**Post-v1.0.0-rc.1 context:** This story is the first in the post-rc.1 batch. Before dispatching
the sub-stories, review:

- The rc.1 release retrospective (STATE.md and SESSION-HANDOFF.md post-rc.1) for any xDome API
  shape changes that might affect DTU fidelity for the G2-G6 surfaces.
- The SAP-2 probe results from the G1 routes (alerts, devices, audit_log) for wire-emission patterns
  to replicate in G2-G6 — specifically the `explicit_nulls` / `arrow_json` behavior from D-1715.

Existing G1 routes serve as canonical patterns for G2-G6 implementers:
- `crates/prism-dtu-claroty/src/routes/alerts.rs` — pagination envelope pattern, `offset_limit`
- `crates/prism-dtu-claroty/src/routes/devices.rs` — large-field-count response struct pattern
- `crates/prism-dtu-claroty/src/routes/audit_log.rs` — datetime handling + TOML type parity

Each sub-story implementer MUST read two of these exemplars before writing a new route (per
SAP-2 rule 5: "Adversary MUST read the DTU source directly").

**Note:** The 5 sub-stories currently reference `S-ADR058-DTU-PARITY-MIGRATION-001` as their
governing parent. At Phase A materialization, update each sub-story's `# DTU-parity anchor:`
comment to reference S-CLAROTY-DTU-PARITY-001 as the specific Claroty-parity governing story.
S-ADR058-DTU-PARITY-MIGRATION-001 covers OCSF test migration (a different concern) and remains
separately in flight.

---

## Architecture Compliance Rules (MANDATORY)

| Rule | Source | Enforcement |
|------|--------|-------------|
| DTU routes in `crates/prism-dtu-claroty/src/routes/<table>.rs` | Existing pattern | File structure check |
| Response types in `crates/prism-dtu-claroty/src/types.rs` | Existing pattern | SAP-2 parity probe |
| All public response types require `#[non_exhaustive]` | CLAUDE.md §non_exhaustive | CI compile-fail gate; `check-non-exhaustive-per-symbol.py` EXPECTED_SYMBOLS must be updated |
| `reqwest` deps: `default-features = false, features = ["rustls-tls"]` | ADR-050 | CI deny check |
| Wire-emission site authoritative over struct definition (SAP-2 rule 6) | CLAUDE.md §SAP-2 | AC-002 parity probe |
| Fixture data: wire-shape assertions on serialized JSON output | CLAUDE.md §Wire-shape discipline (D-1715) | AC-003 integration test |
| All routes registered in `routes/mod.rs` before story closes | Existing pattern | AC-001 route-map check |
| No `native-tls` / `default-tls` in Cargo.toml | ADR-050 D2 | CI deny |

---

## Library & Framework Requirements (MANDATORY)

No new Cargo dependencies expected. All sub-stories use the existing `prism-dtu-claroty`
workspace dependency manifest. Required libraries already present:

| Library | Version | Usage |
|---------|---------|-------|
| `axum` | workspace | HTTP route handlers |
| `serde` / `serde_json` | workspace | Response serialization |
| `uuid` | workspace | Fixture record ID generation (if needed) |
| `chrono` | workspace | Datetime fixture values for OT events / audit-type tables |

At dispatch time: verify exact workspace version pins from `dependency-graph.md`; do not
invent version numbers from training data (CLAUDE.md §Lessons Learned).

---

## File Structure Requirements (MANDATORY)

Files created/modified across the 5 sub-stories, verified in Phase C integration:

| Action | File | Sub-story |
|--------|------|-----------|
| CREATE | `crates/prism-dtu-claroty/src/routes/ot_activity_events.rs` | S-CLAROTY-OT-EVENTS-DTU-001 |
| CREATE | `crates/prism-dtu-claroty/src/routes/device_vulnerability_relations.rs` | S-CLAROTY-DEVVULNREL-DTU-001 |
| CREATE | `crates/prism-dtu-claroty/src/routes/servers.rs` | S-CLAROTY-SERVERS-DTU-001 |
| CREATE | `crates/prism-dtu-claroty/src/routes/server_interfaces.rs` | S-CLAROTY-SERVERS-DTU-001 |
| CREATE | `crates/prism-dtu-claroty/src/routes/org_policy.rs` | S-CLAROTY-ORGPOLICY-DTU-001 |
| CREATE | `crates/prism-dtu-claroty/src/routes/acl_policies.rs` | S-CLAROTY-ACLPOLICY-DTU-001 |
| MODIFY | `crates/prism-dtu-claroty/src/routes/mod.rs` | All 5 sub-stories (route registration) |
| MODIFY | `crates/prism-dtu-claroty/src/types.rs` | All 5 sub-stories (response struct additions) |
| MODIFY | `crates/prism-dtu-demo-server/src/` | This governing story Phase C (seeding) |

Governing story Phase C: verify mod.rs registers all 5 new routes, then add demo-server
seeding for the 5 new table surfaces.

---

## Forbidden Dependencies

- No multi-sensor DTU sweep beyond Claroty (out of scope per human decision 2026-09-04)
- No drift audit of existing G1 routes unless SAP-2 finds parity defects during this batch
- No modification to `claroty.sensor.toml` (TOML stories are complete; DTU must match as-is)
- No new Cargo workspace dependencies without architect review
- No `native-tls` / `default-tls` features in any Cargo.toml addition (ADR-050)

---

## Changelog

| Version | Date | Summary |
|---------|------|---------|
| 1.0 | 2026-09-04 | Initial governing tracking story. Scope: Claroty DTU 14-table parity post-rc.1 — 5 missing G2-G6 DTU routes + demo-server seeding. Deferred out of v1.0.0-rc.1 per human decision 2026-09-04. blocks S-REL-004. |
