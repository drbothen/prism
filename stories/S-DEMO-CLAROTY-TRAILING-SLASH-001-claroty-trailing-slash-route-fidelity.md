---
document_type: story
story_id: S-DEMO-CLAROTY-TRAILING-SLASH-001
title: "prism-dtu-claroty + claroty.sensor.toml: Trailing-Slash Route Fidelity — Add normalize_path Middleware to Claroty DTU Router; Update TOML path_template to Trailing-Slash Form (ADR-031 §D8-b)"
wave: 5
epic_id: E-DTU-FIDELITY
priority: P1
status: ready
# BC-2.16.013 v1.25 authored by PO (D-989 Phase-A burst) — trailing-slash parity clause
# confirmed; normalize_path middleware requirement documented. S-7.01 gate CLEARED.
version: "1.4"
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
  - "Outer-service wrapping required: NormalizePathLayer MUST wrap the outer service (not
    Router::layer()). In axum 0.7, Router::layer() runs after routing — NormalizePathLayer
    silently NO-OPs. Both serve sites (~line 168 TLS and ~line 192 non-TLS) must wrap the
    normalized service. Use ServiceExt::<axum::extract::Request>::into_make_service (fully
    qualified — axum#2377)."
  - "NormalizePathLayer is STRIP-ONLY (trim_trailing_slash): rewrites /alerts/ → /alerts.
    It does NOT add slashes. Routes stay registered WITHOUT trailing slash. The intentional
    /api/v1/devices/:device_id/tags/ route (clone.rs ~line 128) WITH trailing slash will be
    broken by strip — either drop its trailing slash from registration, or verify existing
    tag tests pass after middleware addition."
  - "tower-http = 0.5 pinned in crates/prism-dtu-claroty/Cargo.toml [dependencies] (production
    deps, not dev-deps). Match prism-dtu-common Cargo.toml:30. Do NOT use tower-http 0.6."
  - "AC-003 soft dependency RESOLVED: S-DEMO-CLAROTY-AUDIT-DTU-001 already merged
    (develop@e1c632dc); real audit_log handler available. Stub fallback no longer needed."
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

# S-DEMO-CLAROTY-TRAILING-SLASH-001 v1.4 — Claroty Trailing-Slash Route Fidelity

**Story ID:** S-DEMO-CLAROTY-TRAILING-SLASH-001
**Status:** draft
**Version:** v1.4
**Wave:** 5
**Priority:** P1
**Points:** 3

---

## Authority

ADR-031 §D8-b is the authoritative design decision for Claroty trailing-slash route fidelity
(Gap-CL-001). Read it before implementing:
`.factory/specs/architecture/decisions/ADR-031-dtu-equals-true-dtu-fidelity-principle.md`.

ADR-031 §D8-b establishes the trailing-slash fidelity requirement: the real Claroty xDome API uses
trailing slashes on all POST-for-read endpoints; the DTU router must accept trailing-slash variants
(via `NormalizePathLayer::trim_trailing_slash()`); and the TOML `path_template` values must be
updated to trailing-slash form. ADR-031 §D8-b also requires verifying whether `normalize_path`
middleware is already present before assuming Axum handles it automatically.

ADR-031 `status: accepted`. `superseded_by:` cites ADR-053 §D3 scope-narrowing. The §D3
supersession narrows the single-surface assumption for the Assets surface only; the §D8-b
trailing-slash fidelity provisions governing this story are NOT in the superseded scope.

BC-2.16.013 §Postconditions §1 is the governing behavioral contract — the trailing-slash
`path_template` clause and the `normalize_path` middleware requirement were added in the Wave-5
Phase-A PO burst.

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
- `POST /api/v1/alerts` (no trailing slash) continues to return 200 — `trim_trailing_slash()`
  strips only; requests without a trailing slash pass through unmodified and hit the existing
  no-slash route directly.
- `POST /dtu/configure`, `POST /dtu/reset`, `GET /dtu/health` all return their expected
  responses (normalize_path does not disrupt control-plane routes).
- **CRITICAL — intentional tags trailing-slash route:** `clone.rs` (~line 128) registers
  `POST /api/v1/devices/:device_id/tags/` WITH a trailing slash. With global
  `trim_trailing_slash()`, an inbound `.../tags/` request is stripped to `.../tags`, which
  will NOT match the registered `.../tags/` route (404). Resolution required: either
  (a) also drop the trailing slash from the registered tags route (`/api/v1/devices/:device_id/tags`),
  or (b) verify that existing tag tests still pass after middleware addition and document
  why. The implementer MUST verify the existing AC-3/AC-4 tag tests (from `devices.rs`
  test suite) pass before declaring this story done. If they fail, fix (a) applies.
(traces to BC-2.16.013 v1.25 §Postconditions §1 — normalize_path middleware MUST NOT
break existing routes; ADR-031 §D8-b backward-compatibility note)

---

## Red Gate Tests

| Test Name | AC | Crate | Description |
|-----------|----|-------|-------------|
| `test_claroty_trailing_slash_alerts_returns_200` | AC-001 | prism-dtu-claroty | POST /api/v1/alerts/ returns 200 with fixture data; not 301/404 |
| `test_claroty_trailing_slash_devices_returns_200` | AC-002 | prism-dtu-claroty | POST /api/v1/devices/ returns 200 with fixture data; not 301/404 |
| `test_claroty_trailing_slash_audit_log_get_returns_200` | AC-003 | prism-dtu-claroty | POST /api/v1/audit_log/get/ returns 200; S-DEMO-CLAROTY-AUDIT-DTU-001 merged (develop@e1c632dc) |

**Both serve paths requirement (FIX-1 / FIX-6):**
The parity tests must exercise the outer-service builder that BOTH serve sites use. The
test helper (`ClarotyClone` test setup) must construct the service using the same
`NormalizePathLayer::trim_trailing_slash().layer(router)` + `ServiceExt::into_make_service`
pattern as `clone.rs`. If the test helper uses `router.into_make_service()` directly
(bypassing the normalization layer), the Red Gate tests will pass green even though the
production serve sites are broken — catching the "one path ships broken" failure mode
(axum 0.7 outer-service placement requirement). Document in a test comment which serve
site (TLS or non-TLS) the helper emulates, and that both `clone.rs` paths must mirror it.

---

## Tasks

1. **Read** `crates/prism-dtu-claroty/src/clone.rs` — find `build_router()`; confirm
   whether `normalize_path` or any trailing-slash middleware is already in the layer stack.
   (Current state: `Router::new().route(...).route(...)` — no middleware visible in grep
   results from build_router() inspection.)
2. **Confirm** the correct middleware: `tower_http::normalize_path::NormalizePathLayer::trim_trailing_slash()` is the DEFINITIVE approach for axum 0.7 (pinned in `crates/prism-dtu-claroty/Cargo.toml`). axum has NEVER shipped a built-in trailing-slash normalizer in any 0.7 or 0.8 release — tower-http is always required. No research needed; proceed to step 3.
3. **Write Red Gate tests** (must ALL FAIL before implementation):
   - `test_claroty_trailing_slash_alerts_returns_200`
   - `test_claroty_trailing_slash_devices_returns_200`
   - `test_claroty_trailing_slash_audit_log_get_returns_200`
   These tests should fail with 404 if normalize_path is absent (proving the gap is real).
4. **Add `tower-http = "0.5"` to `crates/prism-dtu-claroty/Cargo.toml` `[dependencies]`**
   (production deps — NOT dev-deps). Then apply the OUTER-SERVICE wrapping pattern in
   `clone.rs` — do NOT use `Router::layer()` for `NormalizePathLayer`:

   **WHY:** In axum 0.7, `Router::layer()` runs middleware AFTER routing. The Router
   resolves the path first (404s on `/alerts/` since the route is registered as `/alerts`),
   THEN runs the layer — so `NormalizePathLayer` via `Router::layer()` silently NO-OPS
   for trailing-slash matching. The path must be normalized BEFORE routing.

   **CORRECT pattern — wrap the outer service:**
   ```rust
   use tower_http::normalize_path::NormalizePathLayer;
   use tower::Layer; // for applying the layer to the service
   use axum::ServiceExt; // for into_make_service on the layered service

   // In build_router() — return the Router as today (routes WITHOUT trailing slash).
   // Routes stay WITHOUT trailing slash; trim_trailing_slash() strips inbound /alerts/ → /alerts.
   // After build_router() returns the Router, at the TWO serve sites in clone.rs:

   // --- non-TLS serve site (~line 192): ---
   let app = NormalizePathLayer::trim_trailing_slash().layer(router);
   axum::serve(listener, ServiceExt::<axum::extract::Request>::into_make_service(app)).await?;

   // --- TLS serve site (~line 168): ---
   let app = NormalizePathLayer::trim_trailing_slash().layer(router);
   axum_server::from_tcp_rustls(listener, tls_config)
       .serve(ServiceExt::<axum::extract::Request>::into_make_service(app))
       .await?;
   ```

   The fully-qualified `ServiceExt::<axum::extract::Request>::into_make_service` syntax is
   REQUIRED because the layered service's Service impl has generics the compiler cannot
   infer (axum issue #2377). Do NOT simplify to `app.into_make_service()` — it will not
   compile.

   **BOTH serve sites MUST wrap the normalized outer service** — if only one site is
   updated, the TLS (or non-TLS) path ships broken. Verify both ~line 168 and ~line 192
   are updated before declaring AC-001/002/003 done.
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
| `crates/prism-dtu-claroty/Cargo.toml` | MODIFY — add to `[dependencies]` | Add `tower-http = "0.5"` to PRODUCTION deps (not dev-deps; the middleware ships in router code). Match the existing `prism-dtu-common` pin (Cargo.toml:30). Do NOT use tower-http 0.6 (moved to a newer http-body line, incompatible with axum 0.7 / tower 0.4 / http 1.0). This is a crate-level dep add, NOT a workspace Cargo.toml change. |

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
| `axum` | `"0.7"` (pinned in `crates/prism-dtu-claroty/Cargo.toml`) | Existing DTU router; `ServiceExt::into_make_service` for wrapping the layered outer service |
| `tower-http` | `"0.5"` (add to `crates/prism-dtu-claroty/Cargo.toml` `[dependencies]`) | `normalize_path::NormalizePathLayer::trim_trailing_slash()` — the ONLY supported path normalization mechanism for axum 0.7. axum has NEVER shipped a built-in trailing-slash normalizer in any 0.7 or 0.8 release; tower-http is always required. |
| `tower` | `"0.4"` (pinned in `crates/prism-dtu-claroty/Cargo.toml`) | `tower::Layer` trait for `.layer()` call on `NormalizePathLayer`; `ServiceExt` for `into_make_service` on the layered service |
| `prism-dtu-common` | workspace path | Existing fixture loading; unchanged |

Version source: `crates/prism-dtu-claroty/Cargo.toml` (axum 0.7, tower 0.4, http 1) and
`crates/prism-dtu-common/Cargo.toml:30` (tower-http 0.5). Do NOT use tower-http 0.6.
Do NOT hedge on "axum 0.8.x+ may have built-in normalize_path" — it does not, and the
definitive approach is axum 0.7 + tower-http 0.5 `NormalizePathLayer::trim_trailing_slash()`
wrapped via outer-service `ServiceExt::into_make_service`.

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
| EC-002 | POST /api/v1/alerts/ with missing Authorization header | Returns 401 (existing check_bearer_auth logic unchanged); normalize_path runs before routing and before the handler's `check_bearer_auth` — this ordering is CORRECT only because `NormalizePathLayer` wraps the OUTER service (FIX-1 outer-service pattern). If mis-applied via `Router::layer()`, the request 404s before auth runs, making this EC false. |
| EC-003 | GET /dtu/health with and without trailing slash | normalize_path applies; GET /dtu/health/ returns 200 (same handler); no regression |
| EC-004 | S-DEMO-CLAROTY-AUDIT-DTU-001 not yet merged | AC-003 test uses stub handler; comment cites dependency; story is not blocked |
| EC-005 | NormalizePathLayer direction — strip vs add | `NormalizePathLayer::trim_trailing_slash()` is STRIP-ONLY: it rewrites inbound `/alerts/` → `/alerts`. It does NOT add slashes. These are TWO INDEPENDENT directions: (a) OUTBOUND — prism's TOML `path_template` gains a trailing slash (AC-004) so prism sends `/alerts/` to the real Claroty API; (b) INBOUND — the DTU strips the trailing slash and matches its existing no-slash route. There is no "bidirectionality" to verify for the DTU. Note: `append_trailing_slash()` is the opposite variant — it would ADD slashes to inbound no-slash requests, which would BREAK existing no-slash routes (AC-005). Do NOT use `append_trailing_slash()`. |

---

## Notes for Implementer

**Use the OUTER-SERVICE wrapping pattern — NOT Router::layer():**
The workspace pins `axum = "0.7"` in `crates/prism-dtu-claroty/Cargo.toml`. In axum 0.7,
`Router::layer()` runs middleware AFTER routing — the Router 404s on `/alerts/` before
`NormalizePathLayer` ever sees the request, making the layer silently NO-OP for
trailing-slash matching. The ONLY correct placement is wrapping the OUTER service after
`build_router()` returns. See Tasks step 4 for the exact code pattern.

**axum has no built-in trailing-slash normalizer (any version):**
axum has NEVER shipped a built-in path/trailing-slash normalization middleware in any
0.7 or 0.8 release. `axum::ServiceExt` is NOT a normalizer — its role here is purely
`into_make_service` conversion of the layered service. tower-http's
`NormalizePathLayer::trim_trailing_slash()` is always required. Do NOT hedge on
"axum 0.8.x+ may have built-in normalize_path" — it does not.

**NormalizePathLayer is STRIP-ONLY:**
`NormalizePathLayer::trim_trailing_slash()` rewrites inbound `/alerts/` → `/alerts`.
It does NOT add slashes to requests that lack them. Routes stay registered WITHOUT
trailing slash (current state is correct — do not re-register with slashes).
`append_trailing_slash()` is the OPPOSITE variant — it would ADD slashes to inbound
no-slash requests, which would break EC-001 (existing no-slash tests). Do NOT use it.

**Both serve sites must be updated:**
`clone.rs` has TWO serve sites: TLS path (~line 168, `into_make_service()`) and non-TLS
path (~line 192, `axum::serve`). BOTH must wrap the `NormalizePathLayer`-normalized
outer service. If only one is updated, one transport path ships broken with no test
catching it (unless the Red Gate tests exercise both paths — see Red Gate section).

**Intentional tags trailing-slash route (AC-005):**
`clone.rs` ~line 128 registers `POST /api/v1/devices/:device_id/tags/` WITH a trailing
slash. `trim_trailing_slash()` will strip inbound `.../tags/` to `.../tags`, which misses
the registered `.../tags/` route. Either drop the trailing slash from the registered tags
route (preferred), or document why existing tag tests pass. Check before declaring done.

**TOML change is mandatory regardless of DTU behavior:**
The TOML `path_template` values must still be updated to trailing-slash form (AC-004). The
TOML is what prism sends to the REAL Claroty API in production; the DTU is the test fixture.

**AC-003 soft dependency notation:** The test for `POST /api/v1/audit_log/get/` must
include a code comment: `// S-DEMO-CLAROTY-AUDIT-DTU-001 merged develop@e1c632dc;
// real audit_log handler available; trailing-slash normalization verified against production handler.`

---

## Risk Mitigations

| Risk | Mitigation |
|------|-----------|
| `Router::layer()` outer-service mis-application — `NormalizePathLayer` NO-OPs silently | Always wrap the outer service (Tasks step 4 pattern), NEVER via `Router::layer()`. Verify Red Gate tests fail before impl (proving the gap) and pass after. |
| Intentional `/api/v1/devices/:device_id/tags/` route broken by strip | `trim_trailing_slash()` strips `.../tags/` → `.../tags` which misses the registered `.../tags/` route. Either drop the trailing slash from the registered route, or verify existing tag tests still pass. Check AC-005. |
| One serve site (TLS or non-TLS) updated but not the other | Red Gate tests MUST exercise both serve paths (or the test helper must use the same outer-service builder both paths use). Verify both ~line 168 and ~line 192 are updated. |
| `append_trailing_slash()` confusion | Do NOT use `append_trailing_slash()` — it ADDs slashes to inbound no-slash requests, breaking EC-001. Use only `trim_trailing_slash()`. |
| `tower-http` wrong version | Add `tower-http = "0.5"` (matching `prism-dtu-common` Cargo.toml:30) to `crates/prism-dtu-claroty/Cargo.toml` `[dependencies]`. tower-http 0.6 is incompatible with axum 0.7 / tower 0.4 / http 1.0 — do NOT use it. |
| AC-003 hard-blocks on S-DEMO-CLAROTY-AUDIT-DTU-001 | Explicitly NOT a hard block (see §AC-003 text and notes); S-DEMO-CLAROTY-AUDIT-DTU-001 already merged (develop@e1c632dc). |
| New event_type emission uncatalogued | SAP-1 sweep: `rg 'event_type\s*=' crates/ --type rust`; zero new emissions without catalog rows. |

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
| 1.4 | 2026-08-02 | story-writer | Round 6 DRIFT-STORY-AUTHORITY-ABSENT-CORPUS-001 (D-2084): added §Authority section; bumped stale body version fields (H1 title + bold Version) from v1.0 to v1.4. |
| 1.3 | 2026-06-08 | story-writer | Six uncertainty-removal corrections from remove-uncertainty research pass (Perplexity + axum docs + tower-http source + axum#2377). FIX-1 (HIGH/U1): replaced wrong Router::layer() snippet with correct outer-service wrapping pattern for BOTH serve sites (~line 168 TLS, ~line 192 non-TLS); ServiceExt::<Request>::into_make_service fully-qualified (axum#2377). FIX-2 (HIGH/U2): corrected EC-005 strip-vs-add inversion — trim_trailing_slash() is STRIP-ONLY; two independent directions (OUTBOUND TOML, INBOUND DTU strip); removed wrong "bidirectionality" claim; noted append_trailing_slash() danger. FIX-3 (MEDIUM/U3): concrete crate-level dep instruction — add tower-http = "0.5" to crates/prism-dtu-claroty/Cargo.toml [dependencies] (production, not dev-deps); do not use 0.6. FIX-4 (MEDIUM/U4): removed axum-0.8 dead path and "(if available)" hedges; definitive statement axum has never shipped built-in path normalizer. FIX-5 (MEDIUM/U5): EC-002 auth-ordering tied to outer-service placement from FIX-1. FIX-6 (LOW/U6): AC-005 enumerates intentional /api/v1/devices/:device_id/tags/ trailing-slash route; Red Gate tests require outer-service builder to prevent "one path ships broken" failure. |
| 1.2 | 2026-06-03 | state-manager | D-990 Phase-A-close: status draft→ready; BC-2.16.013 v1.25 active (PO authored D-989); depends_on S-DEMO-CLAROTY-AUDIT-DTU-001 (SOFT, merged PR #167) SATISFIED; S-7.01 gate CLEARED. |
| 1.1 | 2026-06-03 | story-writer | Wave-5 Phase-A BC-array propagation burst (D-989). PO authored BC-2.16.013 v1.25 with trailing-slash parity clause + normalize_path middleware requirement for claroty.sensor.toml. Propagated into story: (1) `behavioral_contracts: []` → `[BC-2.16.013]`; Flag 1 CLOSED. (2) Added §Behavioral Contracts table with BC-2.16.013 v1.25 role. (3) ACs updated: AC-001/002/003/004/005 now cite `BC-2.16.013 v1.25 §Postconditions §1` instead of `ADR-031 §D8-b requirement N (pending formal BC authorship)`. AC-003 soft-dep note updated: S-DEMO-CLAROTY-AUDIT-DTU-001 already merged (develop@e1c632dc); stub fallback no longer needed. Version bump 1.0 → 1.1. |
| 1.0 | 2026-05-31 | story-writer | Initial materialization from [stub] per ADR-031 §D8-b v1.2 reclassification. 5 ACs, 3 Red Gate tests, 3 pts, wave 5, P1. Grounded against crates/prism-dtu-claroty/src/clone.rs (build_router — no normalize_path found), routes/alerts.rs (registered as POST /api/v1/alerts without trailing slash), routes/devices.rs (same pattern), claroty.sensor.toml (Gap-CL-001 comment). Soft dependency on S-DEMO-CLAROTY-AUDIT-DTU-001 for AC-003 explicitly documented with stub-based mitigation. New-BC flag provided to PO for BC-2.16.013 coverage confirmation. |
