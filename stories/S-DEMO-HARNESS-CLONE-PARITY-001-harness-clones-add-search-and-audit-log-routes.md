---
document_type: story
story_id: S-DEMO-HARNESS-CLONE-PARITY-001
title: "prism-dtu-harness: Bring In-Process Armis + Claroty Clones to Route Parity with Standalone DTUs (closes F-P6-DEFER-001 / F-P10-LOW-001)"
wave: wave-5-e-demo-fidelity
epic_id: E-DTU-FIDELITY
priority: P2
status: ready
# BC-2.16.013 v1.25 authored by PO (D-989 Phase-A burst) with INV-HARNESS-ROUTE-PARITY invariant.
# S-7.01 gate CLEARED.
version: "1.2"
level: "L3"
producer: story-writer
timestamp: "2026-06-01T00:00:00Z"
tdd_mode: strict
subsystems: [SS-17]
# Subsystem anchor justification:
#   SS-17 (DTU Clones) owns all prism-dtu-* crates including prism-dtu-harness per
#   ARCH-INDEX Subsystem Registry v2.105. Both router changes are in harness clone modules
#   (armis.rs and claroty.rs inside prism-dtu-harness), not in the standalone DTU crates.
crates_touched: [prism-dtu-harness]
target_module: prism-dtu-harness
behavioral_contracts:
  - BC-2.16.013  # Bundled Sensor Spec Authoring and DTU-Parity Verification — v1.25.
                 # INV-HARNESS-ROUTE-PARITY (added in Wave-5 Phase-A PO burst 2026-06-03):
                 # prism-dtu-harness clones MUST expose the same HTTP route surface as
                 # their corresponding standalone prism-dtu-* crates. Specifically:
                 # - armis::router() MUST include GET /api/v1/search after S-DEMO-ARMIS-AQL-001 merges
                 # - claroty::router() MUST include POST /api/v1/audit_log/get after S-DEMO-CLAROTY-AUDIT-DTU-001 merges
                 # Auth model: Armis→403, Claroty→401. Response envelopes per BC-2.16.013 v1.25.
                 # BC-2.16.013 is active (lifecycle_status: active — auto-promoted at PLUGIN-MIGRATION-001-D merge D-776).
# BC status: BC-2.16.013 is active. S-7.01 gate CLEARED: behavioral_contracts non-empty
# with active BC. Status may transition to ready once AC↔BC bidirectional traces are
# verified at dispatch and both depends_on stories are merged.
verification_properties: []
depends_on:
  - S-DEMO-ARMIS-AQL-001
  # Dependency anchor: prism-dtu-harness::clones::armis::router() must mirror
  # prism-dtu-armis::ClarotyClone after S-DEMO-ARMIS-AQL-001 adds GET /api/v1/search.
  # The standalone clone must merge first so the harness can mirror it. This is a
  # hard ordering dependency, not conceptual relatedness.
  - S-DEMO-CLAROTY-AUDIT-DTU-001
  # Dependency anchor: prism-dtu-harness::clones::claroty::router() must mirror
  # prism-dtu-claroty::ClarotyClone after S-DEMO-CLAROTY-AUDIT-DTU-001 adds
  # POST /api/v1/audit_log/get (Gap-CL-006). The standalone clone must merge first.
blocks: []
points: 3
estimated_days: 1
risk: LOW
# Risk justification:
#   The standalone DTU routes being mirrored will already be tested and merged before
#   this story is dispatched (depends_on both predecessors). The harness clone changes
#   are structural additions only — new route registrations pointing to handler stubs
#   that delegate to the same in-process state. No new types needed (they share or
#   re-export from the standalone DTU crates).
assumption_validations: []
risk_mitigations: []
# Deferred-finding closure:
#   Closes F-P6-DEFER-001 (out-of-perimeter finding: prism-dtu-harness armis clone missing
#   GET /api/v1/search, surfaced during S-DEMO-ARMIS-AQL-001 lane cascade 2026-05-31).
#   Closes F-P10-LOW-001 (out-of-perimeter finding: prism-dtu-harness claroty clone missing
#   POST /api/v1/audit_log/get, surfaced during S-DEMO-CLAROTY-AUDIT-DTU-001 lane cascade
#   2026-05-31).
#   Promoted to goal task per user direction 2026-06-02 (demo goal fidelity).
---

# S-DEMO-HARNESS-CLONE-PARITY-001 v1.0 — Harness In-Process Clone Route Parity

**Story ID:** S-DEMO-HARNESS-CLONE-PARITY-001
**Status:** draft (pending PO BC authorship)
**Version:** v1.0
**Wave:** wave-5-e-demo-fidelity
**Priority:** P2
**Points:** 3

---

## Origin

Two out-of-perimeter findings were deferred during Wave 5 lane cascades:

- **F-P6-DEFER-001** — Armis harness clone (`prism-dtu-harness::clones::armis`) missing
  `GET /api/v1/search` after S-DEMO-ARMIS-AQL-001 adds the route to the standalone
  `prism-dtu-armis` clone. Surfaced 2026-05-31 during the S-DEMO-ARMIS-AQL-001 cascade;
  deferred out-of-perimeter (armis.rs is not in scope for the standalone-DTU story).

- **F-P10-LOW-001** — Claroty harness clone (`prism-dtu-harness::clones::claroty`) missing
  `POST /api/v1/audit_log/get` after S-DEMO-CLAROTY-AUDIT-DTU-001 adds it to the standalone
  `prism-dtu-claroty` clone (Gap-CL-006). Surfaced 2026-05-31; deferred out-of-perimeter.

Harness clones are the in-process behavioral clones used for multi-tenant harness tests
(S-3.4.01 / BC-3.5.001 / BC-3.5.002). They must remain route-faithful to the standalone
DTU clones per ADR-031 (DTU=true-DTU). After S-DEMO-ARMIS-AQL-001 and
S-DEMO-CLAROTY-AUDIT-DTU-001 merge, the harness clones will diverge from the standalone
clones until this story closes the gap.

Promoted to goal task per user direction 2026-06-02.

---

## Narrative

As a test infrastructure consumer running multi-tenant harness tests,
I want the in-process harness clones for Armis and Claroty to expose the same routes as
their standalone DTU counterparts,
so that multi-tenant tests using `prism-dtu-harness` exercise the same endpoint surface as
the production DTU integration tests — preserving the DTU=true-DTU fidelity guarantee
(ADR-031) across both clone execution models.

---

## Scope

### Armis harness clone — add `GET /api/v1/search`

File: `crates/prism-dtu-harness/src/clones/armis.rs`

- Add `GET /api/v1/search` handler to `router()` mirroring `prism-dtu-armis::ArmisClone`
  per ADR-031 §D8-a.
- Handler must call `check_bearer_auth` (403 on missing/invalid Bearer — Armis auth model).
- Handler must accept `aql: Option<String>` query param; capture via `state.capture_aql()`
  if the harness state carries an AQL log (or use equivalent in-process capture mechanism).
- Response envelope: `{"data": {"results": [...], "total": N}}` matching standalone DTU.
- Update `armis.rs` module-doc route table to include the new endpoint.

### Claroty harness clone — add `POST /api/v1/audit_log/get`

File: `crates/prism-dtu-harness/src/clones/claroty.rs`

- Add `POST /api/v1/audit_log/get` handler to `router()` mirroring
  `prism-dtu-claroty::ClarotyClone` (Gap-CL-006 CLOSED by S-DEMO-CLAROTY-AUDIT-DTU-001).
- Handler must call `check_bearer_auth` (401 on missing/invalid Bearer — Claroty auth model).
- Response envelope: `{"audit_log": [...], "total": N}` matching standalone DTU (AC-003
  of S-DEMO-CLAROTY-AUDIT-DTU-001).
- Update `claroty.rs` module-doc route table to include the new endpoint.

---

## Behavioral Contracts

| BC ID | Version | Title | Role in This Story |
|-------|---------|-------|-------------------|
| BC-2.16.013 | v1.25 | Bundled Sensor Spec Authoring and DTU-Parity Verification | INV-HARNESS-ROUTE-PARITY (added Wave-5 Phase-A PO burst 2026-06-03) governs this story's complete scope: harness armis clone MUST include `GET /api/v1/search`; harness claroty clone MUST include `POST /api/v1/audit_log/get`; auth model per sensor (Armis→403, Claroty→401); response envelopes must match standalone DTU. AC-001 through AC-005 implement INV-HARNESS-ROUTE-PARITY. |

---

## Acceptance Criteria

### AC-001: Armis harness clone registers GET /api/v1/search
`crates/prism-dtu-harness/src/clones/armis.rs::router()` includes a handler for
`GET /api/v1/search`. A request with a valid `Authorization: Bearer {non-empty}` header
returns 200. A request with no/invalid Bearer returns 403 (Armis auth model per BC-2.16.013
v1.25 INV-HARNESS-ROUTE-PARITY — Armis returns 403, NOT 401).
(traces to BC-2.16.013 v1.25 INV-HARNESS-ROUTE-PARITY — armis::router() MUST include
GET /api/v1/search; Armis auth model: 403 on missing/invalid Bearer)

Red Gate test: `test_armis_harness_search_returns_200_with_bearer_403_without`

### AC-002: Armis harness clone AQL routing matches standalone
`GET /api/v1/search?aql=in:devices` returns device records; `GET /api/v1/search?aql=in:alerts`
returns alert records. Response envelope `{"data": {"results": [...], "total": N}}` matching
standalone `prism-dtu-armis` clone (BC-2.16.013 v1.25 INV-HARNESS-ROUTE-PARITY response
envelope shape requirement).
(traces to BC-2.16.013 v1.25 INV-HARNESS-ROUTE-PARITY — Armis search response envelope:
`{"data": {"results": [...], "total": N}}`)

Red Gate test: `test_armis_harness_search_aql_in_devices_returns_device_records`

### AC-003: Claroty harness clone registers POST /api/v1/audit_log/get
`crates/prism-dtu-harness/src/clones/claroty.rs::router()` includes a handler for
`POST /api/v1/audit_log/get`. A request with a valid `Authorization: Bearer {non-empty}`
header returns 200. A request with no/invalid Bearer returns 401 (Claroty auth model per
BC-2.16.013 v1.25 INV-HARNESS-ROUTE-PARITY — Claroty returns 401, NOT 403).
(traces to BC-2.16.013 v1.25 INV-HARNESS-ROUTE-PARITY — claroty::router() MUST include
POST /api/v1/audit_log/get; Claroty auth model: 401 on missing/invalid Bearer)

Red Gate test: `test_claroty_harness_audit_log_returns_200_with_bearer_401_without`

### AC-004: Claroty harness clone audit_log response matches standalone
Response envelope `{"audit_log": [...], "total": N}` where `audit_log` is non-empty and
all 5 TOML-declared columns are present in each entry (id, action, actor, timestamp, resource).
This matches the standalone `prism-dtu-claroty` response shape per BC-2.16.013 v1.25
INV-HARNESS-ROUTE-PARITY response envelope requirement.
(traces to BC-2.16.013 v1.25 INV-HARNESS-ROUTE-PARITY — Claroty audit_log response:
`{"audit_log": [...], "total": N}`)

Red Gate test: `test_claroty_harness_audit_log_response_envelope_matches_standalone`

### AC-005: Module-doc route tables updated
Both `armis.rs` and `claroty.rs` module-doc route inventory tables list the new routes.
No undocumented routes. Route parity with standalone DTU routes verified by inspection.
(traces to BC-2.16.013 v1.25 INV-HARNESS-ROUTE-PARITY — "route parity is verified by
multi-tenant harness tests (BC-3.5.001/BC-3.5.002 consumers)")

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `armis.rs::router()` — add GET /api/v1/search | `crates/prism-dtu-harness/src/clones/armis.rs` | Effectful (HTTP router mutation) |
| `claroty.rs::router()` — add POST /api/v1/audit_log/get | `crates/prism-dtu-harness/src/clones/claroty.rs` | Effectful (HTTP router mutation) |

Architecture section references:
- `architecture/module-decomposition.md` §SS-17 DTU Clones
- `architecture/dependency-graph.md` §Wave-5 DTU fidelity stories

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Armis harness: missing Bearer on /api/v1/search | 403 (matches Armis auth model — standalone returns 403 not 401) |
| EC-002 | Claroty harness: missing Bearer on /api/v1/audit_log/get | 401 (matches Claroty auth model) |
| EC-003 | Armis harness: absent aql param | Returns devices (safe default, mirrors standalone behavior) |
| EC-004 | Harness in-process state has no AQL capture mechanism | Route still serves fixture data; AQL capture is best-effort for harness context |

---

## Token Budget Estimate

| Item | Estimated tokens |
|------|-----------------|
| This story spec | ~2,000 |
| `crates/prism-dtu-harness/src/clones/armis.rs` (existing) | ~2,500 |
| `crates/prism-dtu-harness/src/clones/claroty.rs` (existing) | ~2,500 |
| Standalone DTU route handlers (pattern reference: ~2 files) | ~4,000 |
| BC-2.16.013 v1.25 (INV-HARNESS-ROUTE-PARITY section) | ~3,000 |
| **Total estimate** | **~14,000 tokens** |

Well within 20-30% of agent context window. No split needed.

---

## Tasks (stub — expand at dispatch)

- [ ] **Task 1:** Read `crates/prism-dtu-harness/src/clones/armis.rs` — understand current
  `router()` structure, state type, and auth pattern used.
- [ ] **Task 2:** Read `crates/prism-dtu-armis/src/routes/search.rs` (delivered by
  S-DEMO-ARMIS-AQL-001) — understand handler signature, AQL routing logic, response envelope.
- [ ] **Task 3:** Add `GET /api/v1/search` to harness armis `router()`. Write Red Gate test
  asserting 200 for device AQL + 403 for missing Bearer. Verify RED before implementing.
- [ ] **Task 4:** Implement harness armis search handler. All Red Gate tests GREEN.
- [ ] **Task 5:** Read `crates/prism-dtu-harness/src/clones/claroty.rs` — understand current
  `router()` structure and auth pattern.
- [ ] **Task 6:** Read `crates/prism-dtu-claroty/src/routes/audit_log.rs` (delivered by
  S-DEMO-CLAROTY-AUDIT-DTU-001) — understand handler and response envelope.
- [ ] **Task 7:** Add `POST /api/v1/audit_log/get` to harness claroty `router()`. Write Red
  Gate test asserting 200 + non-empty `audit_log` + 401 on missing Bearer. Verify RED.
- [ ] **Task 8:** Implement harness claroty audit_log handler. All Red Gate tests GREEN.
- [ ] **Task 9:** Update module-doc route tables in both files. SAP-1 sweep: `rg 'event_type\s*=' crates/ --type rust`.
- [ ] **Task 10:** `just check` — final pre-push gate.

---

## Previous Story Intelligence

N/A — first story in E-DTU-FIDELITY touching `prism-dtu-harness`. Related predecessors
that establish the patterns to mirror:
- S-DEMO-ARMIS-AQL-001 — delivers standalone `GET /api/v1/search` (must merge before dispatch)
- S-DEMO-CLAROTY-AUDIT-DTU-001 — delivers standalone `POST /api/v1/audit_log/get` (must merge)

Read those stories' delivered source files before writing any harness code.

---

## Architecture Compliance Rules

- Harness clones mirror standalone DTU route surfaces (ADR-031 DTU=true-DTU)
- Auth model per sensor: Armis→403, Claroty→401 (not interchangeable)
- `prism-dtu-harness` must NOT depend on `prism-spec-engine`, `prism-sensors`, or `prism-query`
- New `event_type` emissions require BC-2.16.002 catalog row (SAP-1)
- No `println!` in production code

---

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `axum` | workspace version | Route handler, `State`, `Json`, `HeaderMap`, `Query` |
| `serde_json` | workspace version | `json!` macro, response envelopes |
| `tokio` | workspace version | Async test runtime |
| `prism-dtu-common` | workspace path | `load_fixture`, auth helpers |

Version source: workspace `Cargo.toml`. Do not pin independently.

---

## File Structure Requirements

| Action | File path | Notes |
|--------|-----------|-------|
| MODIFY | `crates/prism-dtu-harness/src/clones/armis.rs` | Add GET /api/v1/search handler + route registration + module-doc update |
| MODIFY | `crates/prism-dtu-harness/src/clones/claroty.rs` | Add POST /api/v1/audit_log/get handler + route registration + module-doc update |

---

## Forbidden Dependencies

`prism-dtu-harness` MUST NOT gain a dependency on:
- `prism-spec-engine` (build MUST fail if this dep appears)
- `prism-sensors` (build MUST fail if this dep appears)
- `prism-query` (build MUST fail if this dep appears)

---

## References

- ADR-031 §D8-a — Armis AQL endpoint fidelity (harness parity obligation)
- ADR-031 §D1 — DTU clone isolation
- S-DEMO-ARMIS-AQL-001 — standalone Armis AQL route (depends on)
- S-DEMO-CLAROTY-AUDIT-DTU-001 — standalone Claroty audit_log route (depends on)
- BC-3.5.001 / BC-3.5.002 — harness isolation contracts (candidate BCs, pending PO)
- F-P6-DEFER-001 — deferred finding: harness armis missing /api/v1/search
- F-P10-LOW-001 — deferred finding: harness claroty missing /api/v1/audit_log/get

---

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 1.2 | 2026-06-03 | state-manager | D-990 Phase-A-close: status draft→ready; BC-2.16.013 v1.25 active (PO authored D-989, INV-HARNESS-ROUTE-PARITY); depends_on S-DEMO-ARMIS-AQL-001 (merged PR #168) + S-DEMO-CLAROTY-AUDIT-DTU-001 (merged PR #167) BOTH SATISFIED; S-7.01 gate CLEARED. |
| 1.1 | 2026-06-03 | story-writer | Wave-5 Phase-A BC-array propagation burst (D-989). PO authored BC-2.16.013 v1.25 with INV-HARNESS-ROUTE-PARITY invariant governing this story's full scope. Propagated into story: (1) `behavioral_contracts: []` → `[BC-2.16.013]`; status stays draft (depends_on stories both merged, but AC↔BC traces need dispatch-time verification). (2) Added §Behavioral Contracts table with BC-2.16.013 v1.25 + INV-HARNESS-ROUTE-PARITY role. (3) ACs rewritten from INV-HARNESS-ROUTE-PARITY: AC-001 (armis GET /api/v1/search, 403 on missing Bearer), AC-002 (AQL routing, response envelope), AC-003 (claroty POST /api/v1/audit_log/get, 401 on missing Bearer), AC-004 (response envelope), AC-005 (module-doc). Red Gate test names added. (4) Token budget updated to include BC-2.16.013 v1.25 read. Version bump 1.0 → 1.1. |
| 1.0 | 2026-06-01 | story-writer | Initial stub. Captures scope (armis.rs + claroty.rs route additions), gating (depends_on S-DEMO-ARMIS-AQL-001 + S-DEMO-CLAROTY-AUDIT-DTU-001), and finding closure (F-P6-DEFER-001 + F-P10-LOW-001). Status draft pending PO BC authorship per S-7.01. |
