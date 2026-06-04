---
document_type: story
story_id: S-DEMO-CLAROTY-TRAILING-SLASH-001
title: "prism-dtu-claroty + claroty.sensor.toml: Trailing-Slash Route Fidelity — Add normalize_path Middleware to Claroty DTU Router; Update TOML path_template to Trailing-Slash Form (ADR-031 §D8-b)"
wave: 5
epic_id: E-DTU-FIDELITY
priority: P1
status: draft
# BC status: pending PO authorship.
# behavioral_contracts is empty — this story cannot be set to ready until the PO
# authors or confirms that existing BCs cover trailing-slash path normalization.
# Per ADR-031 §D8-b: "No new behavioral contracts. The trailing-slash change is a
# request-path fidelity fix with no behavioral semantics change."
# PO should confirm whether BC-2.16.013 (DTU-parity) is sufficient or a new AC is needed.
version: "1.1"
level: "L4"
producer: story-writer
timestamp: "2026-05-31T00:00:00Z"
tdd_mode: strict
subsystems: [SS-01, SS-16]
# Subsystem anchor justifications:
#   SS-01 (Sensor Adapters) owns prism-dtu-claroty and all prism-dtu-* crates per ARCH-INDEX
#     Subsystem Registry v2.105 row; the normalize_path middleware addition to the Claroty
#     DTU router is SS-01 (DTU clone) scope.
#   SS-16 (Spec Engine) owns prism-spec-engine and prism-sensors; the TOML path_template
#     changes flow through SS-16's SensorSpec loading and pipeline request building.
#   SS-17 (WASM Plugin Runtime) is NOT anchored — no WASM plugin changes.
crates_touched: [prism-dtu-claroty, prism-sensors]
target_module: prism-dtu-claroty
capabilities: [CAP-001, CAP-029]
behavioral_contracts:
  - BC-2.16.013  # Bundled Sensor Spec Authoring and DTU-Parity Verification — v1.25.
                 # §Postconditions §1 claroty.sensor.toml spec now specifies trailing-slash
                 # path_template form (S-DEMO-CLAROTY-TRAILING-SLASH-001) + normalize_path
                 # middleware clause for prism-dtu-claroty router. INV-HARNESS-ROUTE-PARITY
                 # does not directly govern this story (harness clone changes are in
                 # S-DEMO-HARNESS-CLONE-PARITY-001), but the trailing-slash parity clause
                 # for the standalone DTU is part of BC-2.16.013 §Postconditions §1.
                 # PO confirmed BC-2.16.013 coverage is sufficient per BC-2.16.013 v1.25
                 # (Wave-5 Phase-A PO burst 2026-06-03): Flag 1 CLOSED.
# BC status: BC-2.16.013 is active (lifecycle_status: active — auto-promoted at
# PLUGIN-MIGRATION-001-D merge D-776). S-7.01 gate CLEARED: behavioral_contracts
# is non-empty with active BC. Status may transition to ready once AC↔BC bidirectional
# traces are verified at dispatch.
verification_properties:
  - VP-148  # DTU parity — parity test confirms trailing-slash POST returns 200, not 301/404.
depends_on:
  - S-DEMO-CLAROTY-AUDIT-DTU-001
# depends_on justification:
#   S-DEMO-CLAROTY-AUDIT-DTU-001 adds the /api/v1/audit_log/get route to the Claroty DTU.
#   AC-003 of this story requires that POST /api/v1/audit_log/get/ (trailing slash) returns
#   200. If S-DEMO-CLAROTY-AUDIT-DTU-001 has not merged, that route returns 404 regardless
#   of trailing-slash handling. However, this is a SOFT dependency: AC-001 and AC-002
#   (alerts/, devices/) can be verified independently against the stub or merged audit_log
#   route. Explicitly noted in AC-003: "AC-003 can be written against a stub returning
#   200 + empty body if S-DEMO-CLAROTY-AUDIT-DTU-001 has not merged; it must not be a
#   hard wave-gate block."
blocks: []
points: 3
# Points justification:
#   DTU-side changes (prism-dtu-claroty):
#   - Verify axum normalize_path behavior: ~0.5 pts (research + test)
#   - Add axum::middleware::from_fn(normalize_path) or tower_http::normalize_path to
#     Claroty DTU router in clone.rs: ~0.5 pts
#   - Trailing-slash parity tests (3 endpoints): ~1 pt
#   TOML changes (claroty.sensor.toml):
#   - 3 path_template values updated: ~0.25 pts
#   Red Gate tests: ~0.5 pts
#   Verification sweep: ~0.25 pts
#   Total: 3 points (~1 day)
estimated_days: 1
risk: LOW
# Risk justification:
#   Trailing-slash normalization is a well-understood Axum pattern. The risk is that
#   the Claroty DTU router does NOT already have normalize_path middleware, and adding
#   it could affect other routes (e.g., /dtu/configure, /dtu/reset) — must verify all
#   existing routes still work after middleware addition. The TOML changes are mechanical.
acceptance_criteria_count: 5
red_gate_tests: 3
estimated_passes: "1-2 LOCAL adversary passes"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "Verify normalize_path does NOT break existing routes: after adding normalize_path
    middleware, run the full prism-dtu-claroty test suite to confirm all existing tests
    still pass. normalize_path can affect trailing-slash stripping or addition — verify
    bidirectionality (adds trailing slash if absent AND strips if present)."
  - "AC-003 soft dependency on S-DEMO-CLAROTY-AUDIT-DTU-001: if /api/v1/audit_log/get
    route does not exist in the DTU, the trailing-slash AC for that path cannot be
    verified end-to-end. Write AC-003 against a stub that returns 200 + empty body;
    mark the test with a code comment citing the dependency. Do NOT block this story
    on that dependency."
  - "tower-http version: if normalize_path is added via tower-http crate, verify the
    workspace version pin; do NOT add a new version independently."
inputs:
  - "crates/prism-dtu-claroty/src/clone.rs"
  - "crates/prism-dtu-claroty/src/routes/alerts.rs"
  - "crates/prism-dtu-claroty/src/routes/devices.rs"
  - "crates/prism-sensors/specs/claroty.sensor.toml"
  - ".factory/specs/architecture/decisions/ADR-031-dtu-equals-true-dtu-fidelity-principle.md"
  - ".factory/proposals/POLLER-DTU-FIDELITY-AUDIT-2026-05-29.md"
  - ".factory/semport/poller-bear/poller-bear-broad-sweep.md"
input-hash: null
traces_to: []
cycle: "v1.0.0-brownfield"
phase: 3
---

# S-DEMO-CLAROTY-TRAILING-SLASH-001 v1.0 — Claroty Trailing-Slash Route Fidelity

**Story ID:** S-DEMO-CLAROTY-TRAILING-SLASH-001
**Status:** draft
**Version:** v1.0
**Wave:** 5
**Priority:** P1
**Points:** 3

---

## Origin

Established by ADR-031 §D8-b (v1.2 amendment, 2026-05-31). The real Claroty xDome API
uses trailing slashes on all POST-for-read endpoints:
- `POST /api/v1/alerts/`
- `POST /api/v1/devices/`
- `POST /api/v1/audit_log/get/`

Prism's TOML spec (`claroty.sensor.toml`) and DTU clone (`prism-dtu-claroty`) currently
use paths WITHOUT trailing slashes. This means:
- When prism talks to a real Claroty xDome instance, the request paths lack the trailing
  slash, which may cause 301 redirects or 404s depending on Claroty's server configuration.
- The DTU's Axum route normalization has been ASSUMED to mask this gap, but this has NOT
  been verified by a parity test — ADR-031 §D8-b explicitly states: "Verify whether the
  Claroty DTU router includes `normalize_path` before assuming Axum handles it transparently."

**Current DTU state (grounded from code):**
- `crates/prism-dtu-claroty/src/clone.rs::build_router()` registers:
  - `.route("/api/v1/devices", post(devices::list_devices))` — NO trailing slash
  - `.route("/api/v1/alerts", post(alerts::list_alerts))` — NO trailing slash
  - No `normalize_path` middleware visible in the router chain
- `crates/prism-sensors/specs/claroty.sensor.toml` has:
  - `alerts.fetch_alerts.path_template = "/api/v1/alerts"` — NO trailing slash
  - `devices.fetch_devices.path_template = "/api/v1/devices"` — NO trailing slash
  - `audit_logs.fetch_audit_logs.path_template = "/api/v1/audit_log/get"` — NO trailing slash

**Required fix per ADR-031 §D8-b:**
1. Add axum `normalize_path` middleware (or equivalent) to the Claroty DTU router.
2. Update all 3 `path_template` values in `claroty.sensor.toml` to include trailing slashes.
3. Parity test: assert that POST with trailing slash returns 200, not 301/404.

---

## Narrative

As the Prism platform team, I want `prism-dtu-claroty` to accept trailing-slash POST
paths (`/api/v1/alerts/`, `/api/v1/devices/`, `/api/v1/audit_log/get/`) that match the
real Claroty xDome API, and want `claroty.sensor.toml` path templates updated to use
trailing-slash form, so that the demo proves prism sends the same request paths that
the production poller (poller-bear) sends to real Claroty instances.

---

## Story-Level Goal

After this story merges:

1. `prism-dtu-claroty::build_router()` includes `normalize_path` middleware (from
   axum or tower-http) so that trailing-slash and no-trailing-slash variants of
   `POST /api/v1/alerts`, `POST /api/v1/devices`, and (when available)
   `POST /api/v1/audit_log/get` all return 200.
2. `claroty.sensor.toml` updated:
   - `alerts.fetch_alerts.path_template = "/api/v1/alerts/"` (trailing slash)
   - `devices.fetch_devices.path_template = "/api/v1/devices/"` (trailing slash)
   - `audit_logs.fetch_audit_logs.path_template = "/api/v1/audit_log/get/"` (trailing slash)
3. A parity test asserts that `POST /api/v1/alerts/` and `POST /api/v1/devices/` return
   200 (not 301/404) against a running Claroty DTU clone.
4. All existing tests pass (the normalize_path middleware does not break non-trailing-slash
   routes or `/dtu/*` control routes).

---

## Behavioral Contracts

| BC ID | Version | Title | Role in This Story |
|-------|---------|-------|-------------------|
| BC-2.16.013 | v1.25 | Bundled Sensor Spec Authoring and DTU-Parity Verification | §Postconditions §1 `claroty.sensor.toml` spec now specifies trailing-slash path_template form (`/api/v1/alerts/`, `/api/v1/devices/`, `/api/v1/audit_log/get/`) and the normalize_path middleware clause for prism-dtu-claroty. AC-001 through AC-004 implement these postconditions. |

BC-2.16.013 coverage is sufficient per PO confirmation in Wave-5 Phase-A burst (2026-06-03).
Flag 1 CLOSED. No new BC required for trailing-slash path fidelity.

---

## New-BC Flags for Product-Owner

Flag 1 (CONFIRM): **CLOSED** — PO confirmed BC-2.16.013 v1.25 covers trailing-slash path
fidelity. §Postconditions §1 for claroty.sensor.toml was updated in the Wave-5 Phase-A PO
burst to explicitly specify the trailing-slash `path_template` form and the normalize_path
middleware requirement. No new BC is needed.

---

## Acceptance Criteria

### AC-001: POST /api/v1/alerts/ (trailing slash) returns 200 from Claroty DTU
A `POST /api/v1/alerts/` request with a valid `Authorization: Bearer {non-empty}` header
to a running `ClarotyClone` returns HTTP 200 with the alerts fixture data. It does NOT
return 301 (redirect) or 404 (route not found). This proves the DTU accepts the trailing-slash
form that the real Claroty xDome API requires.
(traces to BC-2.16.013 v1.25 §Postconditions §1 — claroty.sensor.toml `alerts` table
uses trailing-slash path_template `/api/v1/alerts/`; DTU normalize_path middleware accepts both forms)

Red Gate test: `test_claroty_trailing_slash_alerts_returns_200`

### AC-002: POST /api/v1/devices/ (trailing slash) returns 200 from Claroty DTU
A `POST /api/v1/devices/` request with a valid `Authorization: Bearer {non-empty}` header
to a running `ClarotyClone` returns HTTP 200 with the devices fixture data. It does NOT
return 301 or 404.
(traces to BC-2.16.013 v1.25 §Postconditions §1 — claroty.sensor.toml `devices` table
uses trailing-slash path_template `/api/v1/devices/`; DTU normalize_path middleware accepts both forms)

Red Gate test: `test_claroty_trailing_slash_devices_returns_200`

### AC-003: POST /api/v1/audit_log/get/ (trailing slash) returns 200 (soft dep on S-DEMO-CLAROTY-AUDIT-DTU-001)
A `POST /api/v1/audit_log/get/` request with a valid `Authorization: Bearer {non-empty}`
header to a running `ClarotyClone` returns HTTP 200. S-DEMO-CLAROTY-AUDIT-DTU-001 is
already merged (develop@e1c632dc), so the real `/api/v1/audit_log/get` route exists. The
stub fallback is still documented for completeness but is no longer needed.
Code comment required in the test: `// S-DEMO-CLAROTY-AUDIT-DTU-001 merged develop@e1c632dc;
// real audit_log handler available; trailing-slash normalization verified against production handler.`
(traces to BC-2.16.013 v1.25 §Postconditions §1 — claroty.sensor.toml `audit_logs` table
uses trailing-slash path_template `/api/v1/audit_log/get/`; DTU normalize_path middleware
accepts both forms; Gap-CL-006 CLOSED by S-DEMO-CLAROTY-AUDIT-DTU-001)

Red Gate test: `test_claroty_trailing_slash_audit_log_get_returns_200`

### AC-004: claroty.sensor.toml path_template uses trailing-slash form for all three tables
`crates/prism-sensors/specs/claroty.sensor.toml` updated:
- `alerts` table `fetch_alerts` step: `path_template = "/api/v1/alerts/"` (was `"/api/v1/alerts"`)
- `devices` table `fetch_devices` step: `path_template = "/api/v1/devices/"` (was `"/api/v1/devices"`)
- `audit_logs` table `fetch_audit_logs` step: `path_template = "/api/v1/audit_log/get/"` (was `"/api/v1/audit_log/get"`)
The Gap-CL-001 comment in the TOML is updated to mark the gap as CLOSED by this story.
A spec load test verifies no parse errors after the update.
(traces to BC-2.16.013 v1.25 §Postconditions §1 — TOML path_template values in trailing-slash form
match the real Claroty xDome API; grounding authority: ADR-028 §D1 — paths derived from DTU routes)

### AC-005: Existing non-trailing-slash routes and /dtu/* routes unaffected
After adding normalize_path middleware, all existing DTU tests pass without modification.
Specifically:
- `POST /api/v1/alerts` (no trailing slash) continues to return 200 (normalize_path
  normalizes BOTH trailing-slash and no-trailing-slash to the same route).
- `POST /dtu/configure`, `POST /dtu/reset`, `GET /dtu/health` all return their expected
  responses (normalize_path does not disrupt control-plane routes).
(traces to BC-2.16.013 v1.25 §Postconditions §1 — normalize_path middleware MUST NOT
break existing routes; ADR-031 §D8-b backward-compatibility note)

---

## Red Gate Tests

| Test Name | AC | Crate | Description |
|-----------|----|-------|-------------|
| `test_claroty_trailing_slash_alerts_returns_200` | AC-001 | prism-dtu-claroty | POST /api/v1/alerts/ returns 200 with fixture data; not 301/404 |
| `test_claroty_trailing_slash_devices_returns_200` | AC-002 | prism-dtu-claroty | POST /api/v1/devices/ returns 200 with fixture data; not 301/404 |
| `test_claroty_trailing_slash_audit_log_get_returns_200` | AC-003 | prism-dtu-claroty | POST /api/v1/audit_log/get/ returns 200; stub if S-DEMO-CLAROTY-AUDIT-DTU-001 not merged |

---

## Tasks

1. **Read** `crates/prism-dtu-claroty/src/clone.rs` — find `build_router()`; confirm
   whether `normalize_path` or any trailing-slash middleware is already in the layer stack.
   (Current state: `Router::new().route(...).route(...)` — no middleware visible in grep
   results from build_router() inspection.)
2. **Research** axum normalize_path behavior — verify which of these applies to the
   workspace's axum version:
   - `axum::middleware::from_fn(axum::middleware::map_request(...))` approach
   - `tower_http::normalize_path::NormalizePathLayer` (if tower-http is already in workspace)
   Note: per ADR-031 §D8-b: "Axum's default behavior does NOT automatically redirect or
   accept trailing-slash variants unless `axum::middleware::normalize_path()` is in the
   layer stack." Confirm this for the workspace's axum version.
3. **Write Red Gate tests** (must ALL FAIL before implementation):
   - `test_claroty_trailing_slash_alerts_returns_200`
   - `test_claroty_trailing_slash_devices_returns_200`
   - `test_claroty_trailing_slash_audit_log_get_returns_200`
   These tests should fail with 404 if normalize_path is absent (proving the gap is real).
4. **Add normalize_path middleware** to `build_router()` in `clone.rs`. Preferred approach:
   ```rust
   // If tower-http is already a workspace dependency:
   use tower_http::normalize_path::NormalizePathLayer;
   let router = Router::new()
       .route("/api/v1/devices", post(devices::list_devices))
       .route("/api/v1/alerts", post(alerts::list_alerts))
       // ... other routes ...
       .layer(NormalizePathLayer::trim_trailing_slash());
   ```
   Alternative (axum built-in, version-dependent):
   ```rust
   use axum::ServiceExt; // if available in workspace axum version
   ```
   Use whichever approach matches the workspace's dependency set. Do NOT add new crate
   dependencies if the existing workspace already provides normalize_path via axum or tower-http.
5. **Run** `cargo nextest run -p prism-dtu-claroty` — Red Gate tests must now PASS GREEN.
   ALL existing tests must still pass (AC-005 verification).
6. **Update** `crates/prism-sensors/specs/claroty.sensor.toml`:
   - `alerts.fetch_alerts.path_template = "/api/v1/alerts/"` (add trailing slash)
   - `devices.fetch_devices.path_template = "/api/v1/devices/"` (add trailing slash)
   - `audit_logs.fetch_audit_logs.path_template = "/api/v1/audit_log/get/"` (add trailing slash)
   - Update Gap-CL-001 comment: mark CLOSED by S-DEMO-CLAROTY-TRAILING-SLASH-001.
7. **Verify** spec loads: run the prism-sensors spec load test to confirm no parse errors
   after TOML changes.
8. **If any new tracing event_type emission introduced**: add BC-2.16.002 catalog row
   (SAP-1). This story is unlikely to introduce new events — note explicitly if none added.
9. **Run** `just check` — final pre-push gate.

---

## File List

| File | Action | Purpose |
|------|--------|---------|
| `crates/prism-dtu-claroty/src/clone.rs` | MODIFY | Add normalize_path middleware layer to build_router() |
| `crates/prism-sensors/specs/claroty.sensor.toml` | MODIFY | Update 3 path_template values to trailing-slash form; close Gap-CL-001 comment |
| `crates/prism-dtu-claroty/tests/` | MODIFY or CREATE | Add 3 Red Gate trailing-slash parity tests |
| `Cargo.toml` (workspace) | POSSIBLY MODIFY | Add tower-http if not already present; verify version |

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| normalize_path must NOT break existing routes | AC-005 | Run full test suite after adding middleware |
| Trailing-slash form in TOML must match real Claroty API | ADR-031 §D8-b; poller-bear API table | SAP-2 probe: TOML path_template verified against semport |
| No new crate dependencies without workspace version pin | CLAUDE.md Conventions | If tower-http added, use workspace version; do not pin independently |
| No println! in production code | CLAUDE.md Conventions | Use tracing::*! with structured fields only |
| New event_type emissions require BC-2.16.002 catalog row | SAP-1 + PG-LP11-001 | Adversary greps event_type = on every pass |

### Forbidden Dependencies

`prism-dtu-claroty` must NOT gain a dependency on `prism-spec-engine`. The DTU is a
test fixture; the production spec engine must not import it (and the reverse must also hold).

---

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `axum` | workspace version | Existing DTU router; normalize_path middleware (if available in this axum version) |
| `tower-http` | workspace version | `NormalizePathLayer` for trailing-slash normalization (if not in axum directly) |
| `prism-dtu-common` | workspace path | Existing fixture loading; unchanged |

Version source: workspace `Cargo.toml`. Verify tower-http is already a workspace dependency
before adding it. If not present, check axum's built-in normalize_path capability first.

---

## Previous Story Intelligence

This is the second story in E-DTU-FIDELITY (after S-DTU-CYBERINT-AUTH-FIDELITY-001).

- **S-DTU-CYBERINT-AUTH-FIDELITY-001** (merged PR #164): Pattern for DTU route changes —
  read clone.rs `build_router()` first, write Red Gate tests, implement the change, verify
  existing tests pass. Same procedure here for normalize_path middleware.

- **S-6.08-dtu-claroty** (original DTU story): Delivered prism-dtu-claroty. The routes were
  registered without trailing slashes. This story adds the missing middleware.

- **S-DEMO-CLAROTY-AUDIT-DTU-001** (may or may not be merged): AC-003 of this story has
  a soft dependency on that story's `/api/v1/audit_log/get` route. Use a stub if not merged.

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | POST /api/v1/alerts (no trailing slash) after normalize_path added | normalize_path normalizes both forms to the same route; returns 200 (backward compat) |
| EC-002 | POST /api/v1/alerts/ with missing Authorization header | Returns 401 (existing check_bearer_auth logic unchanged); normalize_path runs before auth check |
| EC-003 | GET /dtu/health with and without trailing slash | normalize_path applies; GET /dtu/health/ returns 200 (same handler); no regression |
| EC-004 | S-DEMO-CLAROTY-AUDIT-DTU-001 not yet merged | AC-003 test uses stub handler; comment cites dependency; story is not blocked |
| EC-005 | normalize_path strips trailing slash instead of adding one | If normalize_path behavior is strip-only, route registration must use trailing-slash form; verify bidirectionality before choosing middleware approach |

---

## Notes for Implementer

**Verify normalize_path behavior for your axum version:** The correct middleware choice
depends on the workspace's axum version:
- axum 0.7.x: `tower_http::normalize_path::NormalizePath` is the standard approach.
- axum 0.8.x+: may have built-in normalize_path; check axum changelog.

Run a quick test: `POST /api/v1/alerts/` against the current (pre-change) DTU. If it
returns 404, normalize_path is absent and must be added. If it returns 200, the middleware
is already present (unlikely given ADR-031 §D8-b analysis) and only TOML changes are needed.

**TOML change is mandatory regardless of DTU behavior:** Even if the DTU already accepts
trailing-slash paths (e.g., via an existing middleware not visible in build_router), the
TOML `path_template` values must still be updated to trailing-slash form (AC-004). The TOML
is what prism sends to the REAL Claroty API in production; the DTU is the test fixture.

**AC-003 soft dependency notation:** The test for `POST /api/v1/audit_log/get/` must
include a code comment: `// Soft dep: S-DEMO-CLAROTY-AUDIT-DTU-001 must merge before this
test exercises the real audit_log handler. Using stub for trailing-slash verification.`
This makes the dependency visible during adversarial review and prevents false pass/fail
confusion.

---

## Risk Mitigations

| Risk | Mitigation |
|------|-----------|
| normalize_path strips trailing slashes rather than accepting both forms | Test AC-005 (existing no-trailing-slash tests must pass) before AC-001/AC-002/AC-003; if stripping behavior causes regressions, switch to explicit trailing-slash route registration |
| tower-http not in workspace | Check Cargo.toml workspace dependencies first; use axum built-in if available; add tower-http only if necessary with workspace version pin |
| AC-003 hard-blocks on S-DEMO-CLAROTY-AUDIT-DTU-001 | Explicitly NOT a hard block (see §AC-003 text and notes); write stub route returning empty 200 body in test setup |
| New event_type emission uncatalogued | SAP-1 sweep: `rg 'event_type\s*=' crates/ --type rust`; zero new emissions without catalog rows |

---

## Token Budget Estimate

| Context source | Estimated tokens |
|----------------|-----------------|
| This story spec | ~3,500 |
| claroty.sensor.toml | ~1,500 |
| crates/prism-dtu-claroty/src/clone.rs | ~2,000 |
| crates/prism-dtu-claroty/src/routes/alerts.rs | ~800 |
| crates/prism-dtu-claroty/src/routes/devices.rs | ~3,500 |
| ADR-031 §D8-b (relevant section) | ~1,500 |
| POLLER-DTU-FIDELITY-AUDIT-2026-05-29.md §2 (Claroty section) | ~1,500 |
| axum/tower-http normalize_path docs | ~500 |
| Test files (existing prism-dtu-claroty/tests/) | ~1,500 |
| Tool outputs (cargo nextest) | ~1,500 |
| **Total estimate** | **~17,800 tokens (~7% of 256K context)** |

Well within the 20-30% budget.

---

## References

- ADR-031 v1.2 §D8-b — Claroty Trailing-Slash Route Fidelity (Gap-CL-001)
- POLLER-DTU-FIDELITY-AUDIT-2026-05-29.md §2 — Claroty xDome fidelity table (Gap-CL-001)
- `crates/prism-dtu-claroty/src/clone.rs` — build_router() current state (no normalize_path)
- `crates/prism-dtu-claroty/src/routes/alerts.rs` — route registered as `POST /api/v1/alerts`
- `crates/prism-sensors/specs/claroty.sensor.toml` — Gap-CL-001 comment (path without trailing slash)
- poller-bear-broad-sweep.md API table — real Claroty xDome trailing-slash endpoint paths

---

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 1.1 | 2026-06-03 | story-writer | Wave-5 Phase-A BC-array propagation burst (D-989). PO authored BC-2.16.013 v1.25 with trailing-slash parity clause + normalize_path middleware requirement for claroty.sensor.toml. Propagated into story: (1) `behavioral_contracts: []` → `[BC-2.16.013]`; Flag 1 CLOSED. (2) Added §Behavioral Contracts table with BC-2.16.013 v1.25 role. (3) ACs updated: AC-001/002/003/004/005 now cite `BC-2.16.013 v1.25 §Postconditions §1` instead of `ADR-031 §D8-b requirement N (pending formal BC authorship)`. AC-003 soft-dep note updated: S-DEMO-CLAROTY-AUDIT-DTU-001 already merged (develop@e1c632dc); stub fallback no longer needed. Version bump 1.0 → 1.1. |
| 1.0 | 2026-05-31 | story-writer | Initial materialization from [stub] per ADR-031 §D8-b v1.2 reclassification. 5 ACs, 3 Red Gate tests, 3 pts, wave 5, P1. Grounded against crates/prism-dtu-claroty/src/clone.rs (build_router — no normalize_path found), routes/alerts.rs (registered as POST /api/v1/alerts without trailing slash), routes/devices.rs (same pattern), claroty.sensor.toml (Gap-CL-001 comment). Soft dependency on S-DEMO-CLAROTY-AUDIT-DTU-001 for AC-003 explicitly documented with stub-based mitigation. New-BC flag provided to PO for BC-2.16.013 coverage confirmation. |
