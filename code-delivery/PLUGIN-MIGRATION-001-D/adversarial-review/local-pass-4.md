---
document_type: adversarial-review-pass
story_id: PLUGIN-MIGRATION-001-D
pass_number: 4
pass_scope: LOCAL-SPEC-LEVEL
pass_date: 2026-05-20
adversary_model: Claude Opus 4.7 (1M context, fresh)
streak_before: 0/3
streak_after: 0/3
findings_summary: "4 HIGH + 3 MED + 1 LOW + 1 OBS (9 actionable)"
checkpoint_status: ARCHITECTURAL-INPUT-REQUIRED
---

# PLUGIN-MIGRATION-001-D Pass-4 Adversarial Review (LOCAL Spec-Level)

## Scope
Story v1.3, BC-2.16.013 v1.3, BC-2.16.001 v1.4, BC-2.16.009 v1.4, error-taxonomy.md v1.41, HS-013..018, BC cross-anchors, ADR-022/023, TS-PLUGIN-PARITY-001, full code grounding against prism-sensors auth modules + prism-spec-engine + ALL 4 prism-dtu-* clones.

## Critical Finding — Systemic Regression from Pass-3

### F-LP4-HIGH-001 PolicyViolation:POL-22 Phase C — All 4 BC-declared sensor URLs are 404s against the DTU clones the parity tests target

Pass-3 closures (F-LP3-CRIT-002, -003, -HIGH-001, -HIGH-002) aligned URL paths to the production Rust adapter code at `crates/prism-sensors/src/auth/{sensor}.rs`. The pass-3 closure did NOT verify alignment against the actual DTU clone route registrations. **All 4 sensors' BC-declared URLs do not exist on the DTU clones.**

| Sensor | BC v1.3 URL | DTU clone route | Status |
|---|---|---|---|
| CrowdStrike detections | `/queries/detections` + `/entities/detections/GET` | `/detects/queries/detects/v1` + `/detects/entities/summaries/GET/v1` (prism-dtu-crowdstrike/src/routes/mod.rs:189,193) | MISMATCH |
| CrowdStrike devices | `/queries/devices` + `/entities/devices/GET` | `/devices/queries/devices/v1` + `/devices/entities/devices/v2` (mod.rs:197-198) | MISMATCH |
| CrowdStrike incidents | `/queries/incidents` + `/entities/incidents/GET` | (no incidents routes registered) | MISSING ROUTE |
| Claroty assets | `POST /api/v1/assets` | `POST /api/v1/devices` (prism-dtu-claroty/src/clone.rs:85) | MISSING ROUTE |
| Claroty alerts | `POST /api/v1/alerts` | `POST /api/v1/alerts` | OK |
| Cyberint alerts | `GET /api/alerts` | `GET /api/v1/alerts` (prism-dtu-cyberint/src/clone.rs:115) | MISMATCH (missing /v1) |
| Cyberint auth | `bearer_static` (Bearer header) | cookie auth (Set-Cookie cyberint_session) | AUTH-MECH MISMATCH |
| Armis devices | `GET /api/v1/search` w/ AQL | `GET /api/v1/devices` (no /search) | MISSING ROUTE |
| Armis alerts | `GET /api/v1/search` w/ AQL | `GET /api/v1/alerts` (no /search) | MISSING ROUTE |

**Consequence:** Every RG-04..RG-07 parity test starts a DTU clone, executes against it, receives 404 (or 401 for Cyberint). The reference path (calling production `SensorAdapter::fetch()` against the same DTU) has the SAME problem — the production code itself is not aligned with its own DTU clones. This is a LATENT production bug exposed by pass-4 grounding.

**Root-cause:** The production Rust adapter URLs are simplified vs the real APIs (and DTUs model the real APIs). Pass-3 grounded against the wrong reference.

**Routing:** **architect** (whose reference grounds the BC contract — DTU/real-API OR Rust-adapter-as-built? Architectural decision required) + product-owner (implement chosen reference) + story-writer (propagate).

## Other HIGH Findings

### F-LP4-HIGH-002 — `prism-sensors` dev-dep contradiction
AC-007..010 step 7 requires calling `CrowdStrikeAdapter::fetch()` (etc.) to produce reference OCSF output. This requires importing `prism-sensors`. Story §Forbidden Dependencies (line 798-808) prohibits new `prism-sensors` dep. Internal contradiction.
**Routing:** product-owner (authorize dev-dep with rationale OR re-spec reference mechanism to use committed fixture JSON instead of live adapter call).

### F-LP4-HIGH-003 — E-SPEC-017 implementation scope gap
RG-09 expects filename-stem-vs-sensor_id validation emitting E-SPEC-017. error-taxonomy.md v1.41 registers the code (FB-IMPL-P2). But: no `SpecErrorCode::ESpec017` variant exists in prism-core; no filename-stem check exists in `spec_parser.rs::load_all()`. Story §Forbidden file changes is ambiguous about `prism-core/src/error.rs` modification.
**Routing:** product-owner (clarify scope — extend story task list with explicit `prism-core::SpecErrorCode::ESpec017` + `load_all` validation subtasks, OR defer RG-09 to follow-up story).

### F-LP4-HIGH-004 — Cyberint auth_type mismatch with DTU + actual code behavior
BC v1.3 declares `cyberint.sensor.toml.auth_type: "bearer_static"` per `auth_type_name()` return. But actual Cyberint adapter uses cookie auth (`reqwest` cookie store at cyberint.rs:155); DTU enforces cookie (`routes/alerts.rs:43-46`). Pass-2 deferred this as "code TD" but it is parity-test-defeating. Same inverse problem for Claroty (declared `cookie_roundtrip` but actual uses Bearer).
**Routing:** **architect** (decide whether `auth_type_name()` strings are correct labels for the actual auth behavior, OR whether the spec contract should match observed behavior even when `auth_type_name()` is wrong).

## MED Findings

- F-LP4-MED-001: Story AC-001 line 230 says incidents is "(cursor)"; BC + Task 3 say "2-step pipeline" → story-writer fix
- F-LP4-MED-002: HS-018 cites `parse_spec_directory()` but RG-09 doesn't name driver explicitly → story-writer fix
- F-LP4-MED-003: BC `request_count == 2` test vector is fragile (assumes single-page QueryV2) → product-owner relax assertion

## LOW

- F-LP4-LOW-001: AC examples use `unwrap()` (technically permitted in test body but inconsistent with story style guidance)

## OBS

- F-LP4-OBS-001 [process-gap]: POL-22 Phase C verification did not cross-check DTU clone routes in pass-1/2/3. Codification needed: TOML-spec stories targeting DTU parity require dual code-grounding (production adapter + DTU clone routes).

## Verdict
**BLOCKED-soft with ARCHITECTURAL-INPUT-REQUIRED checkpoint.** 4 HIGH findings include at least 2 architectural decisions (URL grounding source + auth_type semantics) that exceed PO/story-writer scope. Cascade pauses pending human/architect adjudication.

## Streak Update
- streak_before: 0/3
- streak_after: 0/3
- next_action: ARCHITECTURAL CHECKPOINT — orchestrator surfaces decisions to user/architect before FB-IMPL-P4 can proceed
