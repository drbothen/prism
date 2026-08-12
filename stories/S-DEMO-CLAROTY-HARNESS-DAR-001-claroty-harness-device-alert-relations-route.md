---
document_type: story
story_id: S-DEMO-CLAROTY-HARNESS-DAR-001
title: "prism-dtu-harness: Add POST /api/v1/device_alert_relations/ to Claroty in-process clone (closes INV-HARNESS-ROUTE-PARITY for device_alert_relations)"
wave: wave-5-e-demo-fidelity
epic_id: E-DTU-FIDELITY
priority: P1
status: draft
# BC status: BC-2.16.013 active (v1.36 as of 2026-08-11). S-7.01 gate CLEARED.
version: "1.0"
acceptance_criteria_count: 5
level: "L4"
producer: story-writer
timestamp: "2026-08-11T00:00:00Z"
modified: "2026-08-11"
tdd_mode: strict
subsystems: [SS-01, SS-16]
# Subsystem anchor justifications:
#   SS-01 (Sensor Adapters) owns prism-dtu-harness per ARCH-INDEX Subsystem Registry
#     (SS-01 crate list explicitly includes "prism-dtu-harness (planned per ADR-011)").
#     The claroty.rs router() and network_router() changes are squarely SS-01 Sensor Adapters work,
#     consistent with S-DEMO-HARNESS-CLONE-PARITY-001 dual-subsystem scoping.
#   SS-16 (Spec Engine) is included because BC-2.16.013 (the governing BC) is a Spec Engine
#     behavioral contract covering DTU-Parity Verification — the same dual-subsystem scoping
#     as the precedent story S-DEMO-HARNESS-CLONE-PARITY-001 and BC-2.16.013 itself.
#   SS-17 ("WASM Plugin Runtime" per ARCH-INDEX) has no ownership of prism-dtu-harness
#     or any DTU crate — do NOT use SS-17 here.
crates_touched: [prism-dtu-harness]
target_module: prism-dtu-harness
capabilities: [CAP-029]
behavioral_contracts:
  - BC-2.16.013  # Bundled Sensor Spec Authoring and DTU-Parity Verification — v1.36.
                 # INV-HARNESS-ROUTE-PARITY: prism-dtu-harness::clones::claroty::router() and
                 # network_router() MUST register POST /api/v1/device_alert_relations/ after
                 # S-DEMO-CLAROTY-DAR-001 merges to develop. Response envelope: {"devices_alerts": [...], "count": N}.
                 # Key MUST be "devices_alerts" — using path stem "device_alert_relations" silently drops all rows.
                 # BC-2.16.013 v1.36 active as of 2026-08-11.
# BC status: BC-2.16.013 active (v1.36). S-7.01 gate CLEARED: behavioral_contracts non-empty
# with active BC. Status may transition to ready once AC<->BC bidirectional traces are
# verified at dispatch and depends_on story S-DEMO-CLAROTY-DAR-001 is merged.
verification_properties:
  - VP-148  # VP-PLUGIN-003 DTU parity — VP-148 anchor story is PLUGIN-MIGRATION-001-D.
            # This story extends harness-side coverage to the device_alert_relations route
            # once parity is verified via multi-tenant harness tests.
depends_on:
  - S-DEMO-CLAROTY-DAR-001
  # Dependency anchor: prism-dtu-harness::clones::claroty::router() must mirror
  # prism-dtu-claroty::ClarotyClone after S-DEMO-CLAROTY-DAR-001 registers
  # POST /api/v1/device_alert_relations/ in the standalone clone (DTU-EXT-006 closure).
  # Hard ordering dependency: standalone clone must merge first (ADR-031 DTU=true-DTU pattern).
  # Additionally, crates/prism-dtu-claroty/fixtures/device-alert-relations.json is created
  # by S-DEMO-CLAROTY-DAR-001; the harness include_str! embed requires this file to exist
  # at compile time.
blocks: []
points: 2
estimated_days: 0.5
risk: LOW
# Risk justification:
#   Single route addition following the established harness audit_log pattern
#   (S-DEMO-HARNESS-CLONE-PARITY-001 AC-003/AC-004 precedent, merged PR #180).
#   Standalone DTU route will be tested and merged before dispatch (depends_on guard).
#   Handler uses include_str! embed of an existing fixture file — no new types, no new
#   crate dependencies. The primary silent-failure risk (wrong response key) is guarded
#   by RG-002 specifically.
assumption_validations: []
risk_mitigations: []
phase: 3
cycle: "v1.0.0-brownfield"
inputs:
  - ".factory/specs/behavioral-contracts/BC-2.16.013-bundled-sensor-spec-dtu-parity.md"
  - "crates/prism-dtu-harness/src/clones/claroty.rs"
input-hash: "1a221df"
traces_to:
  - "BC-2.16.013"
---

# S-DEMO-CLAROTY-HARNESS-DAR-001 v1.0 — Claroty Harness Clone: Add `device_alert_relations` Route

**Story ID:** S-DEMO-CLAROTY-HARNESS-DAR-001
**Status:** draft
**Version:** v1.0
**Wave:** wave-5-e-demo-fidelity
**Priority:** P1
**Points:** 2

---

## Origin

This story discharges the harness-parity obligation established by `S-DEMO-CLAROTY-DAR-001` AC-007
and tracked by BC-2.16.013 §Invariants INV-HARNESS-ROUTE-PARITY. It follows the
`S-DEMO-HARNESS-CLONE-PARITY-001` structural precedent (audit_log harness parity for Claroty,
merged PR #180).

`S-DEMO-CLAROTY-DAR-001` adds `POST /api/v1/device_alert_relations/` to `prism-dtu-claroty`
(standalone) and explicitly excludes `crates/prism-dtu-harness/` in its File Structure
Requirements. AC-007 of that story tracks the harness-parity obligation but does not discharge
it — per its AC-007:

> "The corresponding `prism-dtu-harness::clones::claroty::router()` route parity obligation is
> tracked by S-DEMO-CLAROTY-DAR-001 AC-007 but is NOT in scope for S-DEMO-CLAROTY-DAR-001."

BC-2.16.013 §Invariants INV-HARNESS-ROUTE-PARITY states the obligation as a pending bullet
(not yet a MUST — product-owner deliberately deferred anchoring it as a MUST per TD-VSDD-097
dimension-3, because anchoring a MUST to a story that will not discharge it is the failure mode,
not the fix). Once this story is registered and merges, that bullet becomes a properly-anchored
MUST citing `S-DEMO-CLAROTY-HARNESS-DAR-001` — closing the outstanding obligation and completing
the AC-007 tracking chain.

---

## Narrative

As a test infrastructure consumer running multi-tenant harness tests,
I want the in-process harness clone for Claroty to register `POST /api/v1/device_alert_relations/`
with the correct `{"devices_alerts": [...], "count": N}` response envelope,
so that multi-tenant tests using `prism-dtu-harness` exercise the same endpoint surface as the
production DTU integration tests, preserving the DTU=true-DTU fidelity guarantee (ADR-031) for
the `device_alert_relations` table across both clone execution models.

---

## Authority

Primary authority: BC-2.16.013 §Invariants INV-HARNESS-ROUTE-PARITY (`status: active`, v1.36).

INV-HARNESS-ROUTE-PARITY specifies for this obligation:

- `prism-dtu-harness::clones::claroty::router()` MUST register `POST /api/v1/device_alert_relations/`
  after S-DEMO-CLAROTY-DAR-001 merges to develop.
- Response envelope: `{"devices_alerts": [...], "count": N}` — `devices_alerts` key is **required**;
  `count` is optional (per `GetDeviceAlertsResponse` where only `devices_alerts` is in `required:`).
- The DTU route MUST use `devices_alerts` as the response key — NOT the path stem
  `device_alert_relations`. Using the stem causes every row to be silently dropped
  (BC-2.16.013 EC-016-013-009).
- Auth model: Claroty → HTTP 401 on missing/empty Bearer. This is NOT the Armis model (403) — they
  are not interchangeable.
- Structural precedent: `S-DEMO-HARNESS-CLONE-PARITY-001` AC-003/AC-004 (audit_log harness parity,
  same sensor, same auth model, same router/network_router dual-registration requirement).

**Verbatim artifact statuses (verified on disk 2026-08-11):**

- BC-2.16.013: `status: active` (frontmatter `status: active`, v1.36, modified 2026-08-11)
- S-DEMO-HARNESS-CLONE-PARITY-001: `status: ready` (frontmatter); `merged v1.6 — PR #180
  squash-merged develop@64d34967 2026-06-09` (STORY-INDEX §Full Story List row)
- S-DEMO-CLAROTY-DAR-001: `status: draft` (frontmatter, v1.0, authored 2026-08-11)

---

## Behavioral Contracts

| BC ID | Version | Title | Role in This Story |
|-------|---------|-------|-------------------|
| BC-2.16.013 | v1.36 | Bundled Sensor Spec Authoring and DTU-Parity Verification | INV-HARNESS-ROUTE-PARITY governs this story's complete scope: harness claroty clone MUST register `POST /api/v1/device_alert_relations/` in both `router()` and `network_router()`; response envelope `{"devices_alerts": [...], "count": N}`; Claroty auth model (401 on missing Bearer); response key MUST be `devices_alerts` — NOT `device_alert_relations`. AC-001 through AC-004 implement INV-HARNESS-ROUTE-PARITY. AC-005 closes the traceability loop. |

---

## Acceptance Criteria

### AC-001: `router()` registers `POST /api/v1/device_alert_relations/` (logical mode)

`crates/prism-dtu-harness/src/clones/claroty.rs::router()` includes a handler for
`POST /api/v1/device_alert_relations/`. A request with a valid `Authorization: Bearer {non-empty}`
header returns HTTP 200. A request with no Authorization header returns HTTP 401
(Claroty auth model per BC-2.16.013 INV-HARNESS-ROUTE-PARITY — Claroty returns 401, NOT 403).
(traces to BC-2.16.013 v1.36 INV-HARNESS-ROUTE-PARITY — claroty::router() MUST register
POST /api/v1/device_alert_relations/ after S-DEMO-CLAROTY-DAR-001 merges; Claroty auth: 401)

Red Gate test: `test_BC_2_16_013_claroty_harness_dar_router_returns_200_with_bearer_401_without`

### AC-002: Response envelope uses `devices_alerts` key — not the path stem

The response body from `POST /api/v1/device_alert_relations/` is a JSON object where the
top-level key `devices_alerts` is present and is a non-empty JSON array. The key
`device_alert_relations` MUST NOT appear as the response key — using the path stem silently
drops every row from the pipeline (BC-2.16.013 EC-016-013-009). The response also includes
`count` (optional per `GetDeviceAlertsResponse`). Full shape: `{"devices_alerts": [...], "count": N}`.

This AC is the single most critical assertion in the story. RG-002 must assert the presence
of `devices_alerts` AND the absence of `device_alert_relations` as a top-level key.
(traces to BC-2.16.013 v1.36 INV-HARNESS-ROUTE-PARITY — Claroty device_alert_relations
response envelope: `{"devices_alerts": [...], "count": N}`)

Red Gate test: `test_BC_2_16_013_claroty_harness_dar_response_envelope_uses_devices_alerts_key_not_stem`

### AC-003: `network_router()` also registers `POST /api/v1/device_alert_relations/` (network mode)

`crates/prism-dtu-harness/src/clones/claroty.rs::network_router()` also registers
`POST /api/v1/device_alert_relations/` so that the network-mode isolation harness (BC-3.5.002
consumer) has route parity. A valid Bearer returns HTTP 200; missing Bearer returns HTTP 401.
Response envelope matches AC-002. The same `list_device_alert_relations` handler is reused
(following the `list_audit_log` precedent — same handler registered in both routers).
(traces to BC-2.16.013 v1.36 INV-HARNESS-ROUTE-PARITY — Claroty auth model: 401 on missing
Bearer; full route surface required in both router() and network_router())

Red Gate test: `test_BC_2_16_013_claroty_harness_dar_network_router_returns_200_with_bearer_401_without`

### AC-004: Module-doc route tables updated for both logical-mode and network-mode routers

`claroty.rs` module-doc `//! # Routes served` tables list
`| POST | /api/v1/device_alert_relations/ | Device alert relations; Bearer auth required (401 if missing) |`
in **both** the Logical-mode router section and the Network-mode router section. No undocumented
routes. Route parity with standalone `prism-dtu-claroty` verified by inspection.
(traces to BC-2.16.013 v1.36 INV-HARNESS-ROUTE-PARITY — "Route parity is verified by
multi-tenant harness tests (BC-3.5.001/BC-3.5.002 consumers)")

### AC-005: INV-HARNESS-ROUTE-PARITY gains a properly-anchored MUST on merge (traceability closure)

On merge of this story to develop, BC-2.16.013 §Invariants INV-HARNESS-ROUTE-PARITY is amended
(PO-owned amendment in the same PR, per TD-VSDD-097 mandate-anchor dimension-3) to replace the
current tracking bullet ("is required" / "will be anchored when") with a properly-anchored `MUST`
citing `S-DEMO-CLAROTY-HARNESS-DAR-001`. Recommended MUST wording for the product-owner:

> `prism-dtu-harness::clones::claroty::router()` and
> `prism-dtu-harness::clones::claroty::network_router()` MUST register
> `POST /api/v1/device_alert_relations/` after S-DEMO-CLAROTY-DAR-001 merges to develop
> (closes INV-HARNESS-ROUTE-PARITY for Claroty device_alert_relations).
> Response envelope: `{"devices_alerts": [...], "count": N}`.
> Implemented by: S-DEMO-CLAROTY-HARNESS-DAR-001.

This discharges `S-DEMO-CLAROTY-DAR-001` AC-007 — the obligation that AC-007 tracks is now
fully anchored to a real story that delivers it, eliminating the re-mint risk on every
subsequent adversarial pass.
(traces to BC-2.16.013 v1.36 INV-HARNESS-ROUTE-PARITY — "Once created, that follow-up story
anchors the MUST to this invariant using the same POST /api/v1/audit_log/get →
S-DEMO-CLAROTY-AUDIT-DTU-001 pattern above.")

---

## Red Gate List (SAC-1)

| RG ID | Test Name | Target AC | Failing Assertion |
|-------|-----------|-----------|-------------------|
| RG-001 | `test_BC_2_16_013_claroty_harness_dar_router_returns_200_with_bearer_401_without` | AC-001 | `router()`: valid Bearer → 200; no Bearer → 401 on `POST /api/v1/device_alert_relations/` |
| RG-002 | `test_BC_2_16_013_claroty_harness_dar_response_envelope_uses_devices_alerts_key_not_stem` | AC-002 | Response body: `$.devices_alerts` is a JSON array (required key present); `$.device_alert_relations` absent (path-stem key forbidden) |
| RG-003 | `test_BC_2_16_013_claroty_harness_dar_network_router_returns_200_with_bearer_401_without` | AC-003 | `network_router()`: valid Bearer → 200; no Bearer → 401 on `POST /api/v1/device_alert_relations/` |

### Red Gate Density Check (BC-5.38.001)

3 Red Gate tests for 5 acceptance criteria. RED_RATIO = 3/5 = 0.60, which satisfies the
≥ 0.50 threshold per BC-5.38.001. AC-004 (documentation — verified by inspection, not by
a failing test) and AC-005 (traceability closure — a PO-owned BC amendment obligation, not
a code behavior) do not have corresponding Red Gate tests. All three behavioral ACs (001, 002,
003) have one-to-one Red Gate test coverage.

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `claroty.rs::router()` — add `POST /api/v1/device_alert_relations/` | `crates/prism-dtu-harness/src/clones/claroty.rs` | Effectful (HTTP router mutation) |
| `claroty.rs::network_router()` — add `POST /api/v1/device_alert_relations/` | `crates/prism-dtu-harness/src/clones/claroty.rs` | Effectful (HTTP router mutation, network-mode) |
| `DEVICE_ALERT_RELATIONS_FIXTURE` constant | `crates/prism-dtu-harness/src/clones/claroty.rs` | Pure (compile-time `include_str!` embed) |

Architecture section references:
- `architecture/module-decomposition.md` §SS-01 Sensor Adapters
- `architecture/dependency-graph.md` §Wave-5 DTU fidelity stories

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Missing/empty Bearer on `POST /api/v1/device_alert_relations/` | HTTP 401 (Claroty auth model — `check_bearer_auth` returns 401; NOT Armis-style 403) |
| EC-002 | Handler uses path-stem key `device_alert_relations` in response (wrong key) | Silent row loss — every fixture row is silently dropped by the pipeline; BC-2.16.013 EC-016-013-009 guards this; RG-002 catches it |
| EC-003 | Fixture file `device-alert-relations.json` absent at compile time | Compile error from `include_str!` — this story MUST NOT be dispatched until S-DEMO-CLAROTY-DAR-001 has merged to develop |

---

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| `crates/prism-dtu-harness/src/clones/claroty.rs` — `list_device_alert_relations` handler | effectful-shell | Performs HTTP I/O (reads request headers, writes JSON response); bounded by axum router runtime |
| `crates/prism-dtu-harness/src/clones/claroty.rs` — `DEVICE_ALERT_RELATIONS_FIXTURE` constant | pure-core | Compile-time `include_str!` embed; no I/O at runtime |
| `crates/prism-dtu-harness/src/clones/claroty.rs` — `router()` / `network_router()` route registration | effectful-shell | Mutates the axum Router with new route binding; runs at server startup |

---

## Token Budget Estimate

| Item | Estimated tokens |
|------|-----------------|
| This story spec | ~3,000 |
| `crates/prism-dtu-harness/src/clones/claroty.rs` (1,264 lines) | ~3,500 |
| Standalone DTU device_alert_relations route (pattern reference, ~1 file) | ~1,500 |
| BC-2.16.013 v1.36 §Invariants INV-HARNESS-ROUTE-PARITY (relevant section) | ~2,000 |
| BC files (1 BC) | ~500 |
| **Total estimate** | **~10,500 tokens** |

Well within 20-30% of agent context window. No split needed.

---

## Tasks

> Red-before-green ordering enforced per SAC-1 TDD Iron Law — test tasks (T-RED-*) MUST
> precede implementation tasks (T-IMPL-*). Do not write the handler before writing failing tests.

- [ ] **T-RED-01 (Read):** Read `crates/prism-dtu-harness/src/clones/claroty.rs` — understand:
  (a) `list_audit_log` handler structure: `check_bearer_auth(&headers)`, `AUDIT_LOG_FIXTURE`
  `include_str!` path, response `Json(json!({"audit_log": ..., "total": N}))` — this is the
  exact pattern to mirror;
  (b) `router()` registration: `.route("/api/v1/audit_log/get", post(list_audit_log))`;
  (c) `network_router()` registration: same handler, plain `check_bearer_auth` convention per
  S-DEMO-HARNESS-CLONE-PARITY-001 C-4;
  (d) module-doc `//! # Routes served` table structure.

- [ ] **T-RED-02 (Read):** Read the standalone DTU route handler created by S-DEMO-CLAROTY-DAR-001
  at `crates/prism-dtu-claroty/src/routes/device_alert_relations.rs` — verify the fixture filename
  (`device-alert-relations.json`) and response envelope (`{"devices_alerts": [...], "count": N}`).

- [ ] **T-RED-03 (Write tests — verify RED):** Write Red Gate tests RG-001, RG-002, RG-003 in
  `crates/prism-dtu-harness/tests/logical_isolation_test.rs` following the reqwest-over-TcpListener
  idiom (NOT `tower::ServiceExt::oneshot`). Tests must assert:
  - RG-001: `router()` returns 200 with any non-empty Bearer; returns 401 with no Authorization
    header; endpoint is `POST /api/v1/device_alert_relations/`.
  - RG-002: response body `$.devices_alerts` is a JSON array; top-level key `device_alert_relations`
    is absent (assert both directions — presence of correct key AND absence of wrong key).
  - RG-003: `network_router()` returns 200 with any non-empty Bearer; returns 401 with no
    Authorization header; endpoint is `POST /api/v1/device_alert_relations/`.
  Run tests. All three MUST be RED (compile error or test failure) before proceeding.

- [ ] **T-IMPL-01 (Implement constant):** Add:
  ```
  const DEVICE_ALERT_RELATIONS_FIXTURE: &str =
      include_str!("../../../prism-dtu-claroty/fixtures/device-alert-relations.json");
  ```
  Place adjacent to `AUDIT_LOG_FIXTURE` for readability.

- [ ] **T-IMPL-02 (Implement handler):** Add `list_device_alert_relations` handler mirroring
  `list_audit_log`:
  - Auth: `check_bearer_auth(&headers)` — return 401 on error.
  - Fixture: deserialize `DEVICE_ALERT_RELATIONS_FIXTURE` as `Vec<Value>`.
  - Response: `Json(json!({"devices_alerts": dar_items, "count": count}))` — key is
    `devices_alerts` (NOT `device_alert_relations`); `count` = `dar_items.len() as u64`.

- [ ] **T-IMPL-03 (Implement route registration):** Register route in both routers:
  In `router()`: `.route("/api/v1/device_alert_relations/", post(list_device_alert_relations))`
  (place after audit_log route).
  In `network_router()`: same route and handler (consistent with `list_audit_log` dual-registration).

- [ ] **T-GREEN-04 (Verify GREEN):** Run:
  ```
  cargo nextest run -p prism-dtu-harness -E 'test(BC_2_16_013_claroty_harness_dar)'
  ```
  All three Red Gate tests (RG-001, RG-002, RG-003) MUST be GREEN.

- [ ] **T-DOC-05 (Documentation):** Update `claroty.rs` module-doc `//! # Routes served` tables:
  add `| POST | /api/v1/device_alert_relations/ | Device alert relations; Bearer auth required (401 if missing) |`
  to **both** the Logical-mode router section and the Network-mode router section (AC-004).
  SAP-1 sweep: `rg 'event_type\s*=' crates/prism-dtu-harness/ --type rust` — confirm zero new
  `event_type` emissions (none expected for a harness fixture route).

- [ ] **T-FINAL-06 (Gate):** `just check` — final pre-push gate. All workspace tests GREEN.

---

## Previous Story Intelligence

Directly follows `S-DEMO-HARNESS-CLONE-PARITY-001` (merged PR #180, 2026-06-09). Lessons
carried forward:

- **Fixture embed pattern (C-1):** Use `include_str!` to embed fixture as a `const` string.
  Do NOT call `prism_dtu_common::load_fixture` — that is the standalone DTU's runtime pattern.
- **Handler types from prism-dtu-claroty forbidden (C-8):** The harness is intentionally
  self-contained. `Vec<Value>` from the `include_str!` embed; no typed struct imports from
  `prism-dtu-claroty`.
- **Dual-router registration (C-4):** Both `router()` and `network_router()` must register the
  route. `network_router()` uses plain `check_bearer_auth` (same as sibling alert/vuln routes),
  NOT a network-mode variant — confirmed by the existing audit_log registration in both routers.
- **Claroty auth model is 401 (not 403):** `check_bearer_auth(&headers)` returns 401 for
  absent/empty Bearer. Armis is 403. These are not interchangeable.
- **Test idiom (C-5):** reqwest-over-TcpListener in `tests/logical_isolation_test.rs`.
  NOT `tower::ServiceExt::oneshot`.
- **axum pin (C-6):** `axum = "0.7"` pinned literally in `crates/prism-dtu-harness/Cargo.toml`.
  Do not upgrade.

Read `crates/prism-dtu-harness/src/clones/claroty.rs` `list_audit_log` function as the
canonical pattern to mirror. The new handler is structurally identical — different fixture
constant and different response keys (`devices_alerts`/`count` instead of `audit_log`/`total`).

---

## Architecture Compliance Rules

- Harness clones mirror standalone DTU route surfaces (ADR-031 §D7 harness-scope extension;
  ADR-031 §D1-c route-existence fidelity — DTU registers exactly the real API's endpoints)
- Auth model per sensor: Claroty → 401 on missing/empty Bearer (NOT Armis-style 403)
- `prism-dtu-harness` MUST NOT depend on `prism-spec-engine`, `prism-sensors`, or `prism-query`
- `include_str!` is a compile-time file embed — not a Cargo crate dependency; permitted
- New `event_type` emissions require a BC-2.16.002 catalog row (SAP-1); this story adds none
- No `println!` in production code
- Response key MUST be `devices_alerts` — NEVER `device_alert_relations` (BC-2.16.013 EC-016-013-009)
- Admin-token bearer comparisons MUST use constant-time equality (`ct_compare_tokens`) per
  INV-HARNESS-ROUTE-PARITY §Admin-token clause (CWE-208 timing side-channel). Note: Claroty's
  `check_bearer_auth(&headers)` checks for non-empty Bearer only (no stored-token comparison);
  this clause applies if any future handler variant checks a provided token against a stored value

---

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `axum` | **0.7 (pinned literally in `crates/prism-dtu-harness/Cargo.toml`, NOT .workspace)** | Route handler, `Json`, `HeaderMap`, `post()` routing |
| `serde_json` | 1 (workspace) | `json!` macro, `Vec<Value>` fixture deserialization |
| `tokio` | 1 (workspace) | Async test runtime |
| `reqwest` | 0.12 (workspace) | Red Gate test HTTP client — reqwest-over-TcpListener idiom |

Version note: `axum = "0.7"` is pinned literally in `Cargo.toml`, not via `.workspace = true`.
The new route has no path params, so the axum 0.7 colon-syntax migration risk does not apply.
Do not upgrade the axum pin.

---

## File Structure Requirements

| Action | File path | Notes |
|--------|-----------|-------|
| MODIFY | `crates/prism-dtu-harness/src/clones/claroty.rs` | Add `DEVICE_ALERT_RELATIONS_FIXTURE` constant + `list_device_alert_relations` handler + route registrations in `router()` and `network_router()` + module-doc route table updates |

---

## Forbidden Dependencies

`prism-dtu-harness` MUST NOT gain a dependency on:
- `prism-spec-engine` (build MUST fail if this dep appears in `Cargo.toml`)
- `prism-sensors` (build MUST fail if this dep appears in `Cargo.toml`)
- `prism-query` (build MUST fail if this dep appears in `Cargo.toml`)
- `prism-dtu-claroty` as a **Cargo crate dependency** (build MUST fail if this crate appears in
  `Cargo.toml` as a direct dependency). Note: the `include_str!` path
  `"../../../prism-dtu-claroty/fixtures/device-alert-relations.json"` is a compile-time file
  inclusion — it is NOT a Cargo crate dep and is the established harness pattern. The Claroty
  crate's Rust types (`ClarotyDeviceAlertRelation`, `GetDeviceAlertsBody`, etc.) MUST NOT be
  imported into the harness.
- `prism-dtu-armis` (build MUST fail if this appears as a direct Cargo crate dependency)

---

## References

- BC-2.16.013 §Invariants INV-HARNESS-ROUTE-PARITY — primary authority; specifies the mandate
  and response envelope shape
- BC-2.16.013 §Known Gaps DTU-EXT-006 — tracking row for S-DEMO-CLAROTY-DAR-001 (standalone)
- BC-2.16.013 EC-016-013-009 — response key mismatch edge case; guarded by RG-002
- S-DEMO-CLAROTY-DAR-001 AC-007 — harness-parity obligation tracked here; discharged by this story
- S-DEMO-CLAROTY-DAR-001 — prerequisite (standalone DTU route + fixture + TOML; must merge first)
- S-DEMO-HARNESS-CLONE-PARITY-001 — structural precedent (merged PR #180); establishes audit_log
  harness pattern that this story replicates for device_alert_relations
- ADR-031 §D7 — Harness clones in-scope for DTU=true-DTU (harness-scope extension)
- ADR-031 §D1-c — DTU MUST register exactly the real API's endpoints (route-existence fidelity)
- BC-3.5.001 / BC-3.5.002 — harness isolation contracts (logical and network-mode consumers)

---

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 1.0 | 2026-08-11 | story-writer | Initial story. Discharges INV-HARNESS-ROUTE-PARITY pending obligation for Claroty device_alert_relations harness route parity. Scope: single handler + route registration in router() and network_router() + module-doc update. Depends on S-DEMO-CLAROTY-DAR-001. SAC-1: 3 RGTs, density 0.60. |
