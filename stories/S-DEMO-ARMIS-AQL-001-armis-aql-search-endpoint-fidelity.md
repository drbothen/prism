---
document_type: story
story_id: S-DEMO-ARMIS-AQL-001
title: "prism-dtu-armis + armis.sensor.toml: AQL Search Endpoint Fidelity — Add GET /api/v1/search Route; Update TOML path_template to /api/v1/search; Parity Test AQL Push-Down (ADR-031 §D8-a)"
wave: 5
epic_id: E-DTU-FIDELITY
priority: P1
status: merged
merged_sha: eb3416d1
merged_pr: 168
merged_at: "2026-06-02"
# BC status: D-911 disposition 2026-05-31 — New-BC Flags 1 & 2 resolved as SUFFICIENT.
# BC-2.16.013 (Bundled Sensor Spec Authoring and DTU-Parity Verification, v1.22) is ACTIVE
# and covers both the AQL endpoint pipeline behavior (Flag 1: AQL treated as opaque per
# R-DTU-002 / ADR-031 §D8-a — no syntax validation, no new BC needed) and the AQL push-down
# parity assertion (Flag 2: R-DTU-002 pass-through is a DTU-parity concern under BC-2.16.013).
# No new BC is required. S-7.01 Spec-First Gate satisfied: behavioral_contracts non-empty +
# BC-2.16.013 is active.
# Parity gate note (D-914): parity ACs (AC-005, AC-006 pipeline path) that exercise
# ${env.ARMIS_INSTANCE_URL} resolution are soft-gated on S-SPEC-ENV-VAR-001 (env-var
# resolution story) merging first. The story may be dispatched; Red Gate unit tests
# (AC-001..AC-004) are unblocked. Parity tests requiring full pipeline env-var resolution
# must be #[ignore]-annotated with a code comment citing S-SPEC-ENV-VAR-001 until that
# prereq merges.
version: "1.10"
level: "L4"
producer: story-writer
timestamp: "2026-05-31T00:00:00Z"
modified: "2026-06-02"
tdd_mode: strict
subsystems: [SS-01, SS-16]
# Subsystem anchor justifications:
#   SS-01 (Sensor Adapters) owns prism-dtu-armis and all prism-dtu-* crates per ARCH-INDEX
#     Subsystem Registry v2.105 row; the new /api/v1/search route, AQL filter logic,
#     and AQL log integration are all SS-01 (DTU clone) scope.
#   SS-16 (Spec Engine) owns prism-spec-engine including the pipeline's path_template
#     resolution and AQL parameter forwarding; the TOML step changes flow through
#     SS-16's SensorSpec loading and FetchContext at runtime.
#   SS-17 (WASM Plugin Runtime) is NOT anchored — this story has no WASM plugin changes.
crates_touched: [prism-dtu-armis, prism-sensors, prism-spec-engine]
target_module: prism-dtu-armis
capabilities: [CAP-001, CAP-029]
behavioral_contracts:
  - BC-2.16.013  # Bundled Sensor Spec Authoring and DTU-Parity Verification — 4 Initial Sensors (v1.22, ACTIVE).
                 # Covers the AQL search endpoint pipeline behavior (Flag 1: opaque AQL per
                 # R-DTU-002/ADR-031 §D8-a) and AQL push-down parity (Flag 2: R-DTU-002
                 # pass-through). D-911 disposition: both flags SUFFICIENT, no new BC needed.
verification_properties:
  - VP-148  # DTU parity — parity tests exercise the AQL search pipeline path; a passing
            # parity test after this story proves real-API AQL query behavior.
depends_on: []
# depends_on is empty: Armis AQL gap (Gap-AR-001) has no cross-story dependencies.
# DTU-EXT-003/004 fixes are self-contained within prism-dtu-armis and armis.sensor.toml.
blocks: []
points: 5
# Points justification:
#   DTU-side changes (prism-dtu-armis):
#   - New GET /api/v1/search route handler (search.rs or inline): ~1.5 pts
#     (AQL param extraction, in:devices vs in:alerts filter logic, response envelope)
#   - Register /api/v1/search in build_router() (clone.rs): ~0.25 pts
#   - AQL log capture (existing state.capture_aql() plumbing already present): ~0.25 pts
#   - Direct endpoints (GET /api/v1/devices, GET /api/v1/alerts) remain for back-compat: 0 pts
#   TOML changes (armis.sensor.toml):
#   - devices + alerts table steps: path_template → /api/v1/search + aql param: ~0.5 pts
#   Parity tests:
#   - AQL string captured in DTU aql-log matches AQL prism constructed: ~1 pt
#   - Devices via in:devices AQL: ~0.5 pts
#   - Alerts via AQL: ~0.5 pts
#   Red Gate tests: ~0.5 pts
#   Total: 5 points (~1.5-2 days)
estimated_days: 2
risk: MEDIUM
# Risk justification:
#   AQL filter logic (in:devices vs in:alerts) adds branching in the search handler.
#   The AQL is treated as opaque (R-DTU-002 — not parsed), so filters are applied
#   by inspecting the AQL string for known patterns rather than parsing AQL grammar.
#   TOML step changes require pipeline to forward the AQL param as a query parameter;
#   this depends on the spec engine's FetchContext/query_param forwarding already
#   supporting the ${query.filter.aql} interpolation pattern.
acceptance_criteria_count: 7
red_gate_tests: 11
estimated_passes: "2 LOCAL adversary passes"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "AQL string treatment: the DTU must capture the AQL string verbatim (R-DTU-002
    mitigation is already in place via state.capture_aql()). The search handler does NOT
    parse or validate AQL grammar — it stores it as-is and uses pattern-matching to
    route between devices and alerts fixture data."
  - "Back-compat: GET /api/v1/devices and GET /api/v1/alerts remain registered in
    build_router() after this story — they are not removed. The primary path is now
    /api/v1/search, but existing tests using the direct endpoints must not break."
  - "AQL param forwarding: verify the spec engine forwards the aql parameter as a URL
    query param (?aql=...) on GET requests. The TOML step uses method = 'GET' and
    path_template = '/api/v1/search'; the AQL value must appear as ?aql=<value>."
  - "New BC flag: if AQL syntax validation is added to the DTU (e.g., rejecting
    malformed AQL strings with 400), flag to product-owner for BC-2.01.NNN EC row.
    Per ADR-031 §D8-a: 'If AQL syntax validation is added to the DTU, BC-2.01.NNN
    may need a new EC row — flag to product-owner.'"
inputs:
  - "crates/prism-dtu-armis/src/clone.rs"
  - "crates/prism-dtu-armis/src/routes/devices.rs"
  - "crates/prism-dtu-armis/src/routes/alerts.rs"
  - "crates/prism-dtu-armis/src/state.rs"
  - "crates/prism-dtu-armis/src/types.rs"
  - "crates/prism-sensors/specs/armis.sensor.toml"
  - ".factory/specs/architecture/decisions/ADR-031-dtu-equals-true-dtu-fidelity-principle.md"
  - ".factory/proposals/POLLER-DTU-FIDELITY-AUDIT-2026-05-29.md"
  - ".factory/semport/poller-coaster/poller-coaster-broad-sweep.md"
input-hash: null
traces_to: []
cycle: "v1.0.0-brownfield"
phase: 3
---

# S-DEMO-ARMIS-AQL-001 v1.10 — Armis AQL Search Endpoint Fidelity

**Story ID:** S-DEMO-ARMIS-AQL-001
**Status:** in-progress
**Version:** v1.10
**Wave:** 5
**Priority:** P1
**Points:** 5

---

## Authority

ADR-031 §D8-a is the authoritative design decision for this story. It mandates that the real
Armis production API uses `GET /api/v1/search?aql=<query>` as the single search endpoint
(replacing separate /devices and /alerts routes), and that the DTU clone must faithfully
replicate this endpoint. The `aql` parameter is treated as opaque per R-DTU-002 — no
validation, no parsing. Note: ADR-031 is scope-narrowed by ADR-053 §D3, but §D8-a (AQL
endpoint fidelity) is unaffected. Read §D8-a before implementing the route:
`.factory/specs/architecture/decisions/ADR-031-dtu-equals-true-dtu-fidelity-principle.md`.

ADR-005 defines the AQL trust model and injection mitigation. The opaque passthrough in this
story operates within ADR-005's validated-allowlist trust boundary — the DTU does not validate
AQL syntax, which is correct per R-DTU-002 (the DTU mirrors the real API's accept-any-AQL
behavior). Read ADR-005 §Trust Model before implementing AQL routing:
`.factory/specs/architecture/decisions/ADR-005-aql-injection-mitigation.md`.

---

## Origin

Established by ADR-031 §D8-a (v1.2 amendment, 2026-05-31). The real Armis Centrix
production poller (poller-coaster) uses ONE endpoint for all data: `GET /api/v1/search?aql=<query>`.
Prism's DTU clone (`prism-dtu-armis`) and its TOML spec (`armis.sensor.toml`) currently
use direct REST endpoints (`GET /api/v1/devices`, `GET /api/v1/alerts`) which are a
different call pattern.

Per ADR-031 §D8-a, the AQL divergence was previously classified as a D2-permitted
acceptable divergence (Gap-AR-001, DTU-EXT-003/004). It is reclassified as REQUIRED
fidelity per user directive 2026-05-31 ("all sensors, best-in-class, no scope compromises").

**Real Armis API behavior (canonical reference: poller-coaster):**
The production poller uses `centrix.Search()` with an AQL query string for all 7 data sources.
Devices are queried via `in:devices` AQL; alerts are queried via `in:alerts status:Open` AQL.
The endpoint is always `GET /api/v1/search?aql=<encoded-query>`.

**Current DTU state (grounded from code):**
- `GET /api/v1/devices` — registered in `clone.rs::build_router()` (routes/devices.rs
  `get_or_post_devices`)
- `POST /api/v1/devices` — registered (routes/devices.rs `post_devices`)
- `GET /api/v1/alerts` — registered (routes/alerts.rs `get_alerts`)
- `GET /dtu/aql-log` — registered (state::capture_aql + aql_log already implemented)
- `GET /api/v1/search` — NOT registered (Gap-AR-001)

The AQL infrastructure is already present (`state.capture_aql()`, `state.aql_log()`,
`AqlLogResponse` in types.rs, `GET /dtu/aql-log` route). The missing piece is the
`/api/v1/search` route that serves as the primary data query path.

---

## Narrative

As the Prism platform team, I want `prism-dtu-armis` to expose a `GET /api/v1/search`
endpoint that accepts an `aql=<query>` parameter and returns devices or alerts filtered
by AQL-pattern matching, and want `armis.sensor.toml` table steps to use
`path_template = "/api/v1/search"`, so that the live demo proves prism can query Armis
the same way the production poller does — not just via a non-production direct-endpoint
call pattern.

---

## Story-Level Goal

After this story merges:

1. `prism-dtu-armis::build_router()` registers `GET /api/v1/search` in addition to the
   existing direct-endpoint routes (which remain for back-compat).
2. The `/api/v1/search` handler accepts an `aql` query parameter, captures it via
   `state.capture_aql()`, and routes the response based on the AQL content:
   - AQL containing `in:devices` → returns
     `{"data": {"results": <DeviceRecords>, "total": N}}` envelope.
   - AQL containing `in:alerts` → returns `{"data": {"results": <AlertRecords>, "total": N}}` envelope.
   - AQL absent or unrecognized → returns devices by default (safe fallback).
3. `armis.sensor.toml` `devices` and `alerts` table steps updated:
   - `path_template = "/api/v1/search"` (replaces `/api/v1/devices` and `/api/v1/alerts`)
   - The `aql` query parameter is included via the spec engine's query parameter mechanism.
4. A parity test asserts that the AQL string received by the DTU via `GET /dtu/aql-log`
   matches the AQL prism constructed for the query (validates R-DTU-002 AQL push-down).
5. `GET /api/v1/devices` and `GET /api/v1/alerts` remain registered (back-compat); they
   are NOT the primary path but must not return 404.

---

## Behavioral Contracts

| BC ID | Title | Version | Role in This Story |
|-------|-------|---------|-------------------|
| BC-2.16.013 | Bundled Sensor Spec Authoring and DTU-Parity Verification — 4 Initial Sensors | v1.22 (ACTIVE) | Covers the AQL search endpoint pipeline behavior (Flag 1: opaque AQL, R-DTU-002/ADR-031 §D8-a) and the AQL push-down parity assertion (Flag 2: R-DTU-002 pass-through is a DTU-parity concern). D-911 disposition 2026-05-31: both New-BC Flags SUFFICIENT — no new BC needed. |

**Note:** Per D-911 disposition 2026-05-31, BC-2.16.013 (v1.22, ACTIVE) is sufficient
coverage for both flagged surfaces. ADR-031 §D8-a holds: AQL is treated as opaque (R-DTU-002),
no syntax validation is added to the DTU, and AQL push-down parity is a BC-2.16.013 concern.
S-7.01 Spec-First Gate is satisfied.

---

## New-BC Flags — D-911 Disposition (2026-05-31)

Flag 1 (CLOSED — SUFFICIENT): AQL is treated as opaque per R-DTU-002 / ADR-031 §D8-a.
No syntax validation is added to the DTU. No new BC is needed. BC-2.16.013 (v1.22, ACTIVE)
covers the DTU-parity surface. D-911 disposition: SUFFICIENT.

Flag 2 (CLOSED — SUFFICIENT): AQL push-down parity (R-DTU-002 pass-through) is a
DTU-parity concern already within BC-2.16.013's scope. No new BC-2.01.NNN AC is needed.
D-911 disposition: SUFFICIENT.

Both flags are fully dispositioned. No further PO authorship action required for this story.

---

## Acceptance Criteria

### AC-001: GET /api/v1/search registered in DTU build_router
`crates/prism-dtu-armis/src/clone.rs::build_router()` registers a handler for
`GET /api/v1/search`. A request to `GET /api/v1/search?aql=in:devices` on a running
DTU clone returns 200 (not 404). `Authorization: Bearer {non-empty}` header is required;
missing/empty token returns 403 (matching existing Armis DTU auth pattern — AC-5 per
routes/devices.rs `check_bearer_auth`: Armis returns 403 not 401).
(traces to BC-2.16.013 §Postconditions §2 (DTU-Parity Tests Pass) — DTU must implement the endpoint declared
in the TOML spec; ADR-031 §D8-a is the architectural mandate; VP-148 parity gate)

Red Gate test: `test_armis_aql_search_route_registered_returns_200_for_device_aql`

### AC-002: /api/v1/search with in:devices AQL returns device records
`GET /api/v1/search?aql=in:devices` returns HTTP 200 with response envelope
`{"data": {"results": [...DeviceRecords...], "total": N}}` where the `results` array
contains `DeviceRecord` objects matching the DTU fixture data (same fields as
`crates/prism-dtu-armis/src/types.rs::DeviceRecord`). The AQL string `in:devices`
is captured via `state.capture_aql()` and is visible in the subsequent `GET /dtu/aql-log`
response as `{"aql_strings": ["in:devices"]}`.
(traces to BC-2.16.013 §Postconditions §2 (DTU-Parity Tests Pass); AQL-capture requirement per ADR-031 §D8-a / R-DTU-002)

Red Gate test: `test_armis_aql_search_devices_aql_returns_device_records`

### AC-003: /api/v1/search with in:alerts AQL returns alert records
`GET /api/v1/search?aql=in:alerts` (or the AQL string prism constructs for the alerts
table, e.g. `in:alerts status:Open`) returns HTTP 200 with response envelope
`{"data": {"results": [...AlertRecords...], "total": N}}` where `results` contains
`AlertRecord` objects matching the DTU fixture data. The AQL string is captured in
`GET /dtu/aql-log`.
(traces to BC-2.16.013 §Postconditions §2 (DTU-Parity Tests Pass))

Red Gate test: `test_armis_aql_search_alerts_aql_returns_alert_records`

### AC-004: armis.sensor.toml devices and alerts steps use /api/v1/search path
`crates/prism-sensors/specs/armis.sensor.toml`:
- `devices` table `fetch_devices` step: `path_template = "/api/v1/search"` (was `"/api/v1/devices"`)
- `alerts` table `fetch_alerts` step: `path_template = "/api/v1/search"` (was `"/api/v1/alerts"`)
The `method = "GET"` and existing pagination configuration are preserved. An AQL query
parameter is included in the step configuration to forward the sensor's AQL filter string
to the DTU. The `response_path` fields are updated if the envelope changes from
`$.data.devices` / `$.data.alerts` to `$.data.results` (per the search route's response).
DTU-EXT-003 and DTU-EXT-004 comments in the TOML are updated to reflect that the gap is
now closed.
(traces to BC-2.16.013 §Postconditions §1 (Spec Files Authored and Validated))

### AC-005: Parity test — AQL string prism sends matches DTU-received AQL string
An integration test (using `ArmisClone` from prism-dtu-harness or prism-dtu-armis directly):
1. Run the pipeline fetch for `armis_devices` (or `armis_alerts`) against the running DTU.
2. Query `GET /dtu/aql-log` after the fetch completes.
3. Assert the AQL string in `aql_strings[0]` matches the AQL prism constructed for the
   query (validates R-DTU-002: AQL push-down is end-to-end; prism's constructed AQL is
   what the real Armis API would receive).
(traces to ADR-031 §D5 validation discipline — parity tests must assert endpoint paths
and AQL push-down match; traces to ADR-031 §D8-a requirement 3)

### AC-006: Direct endpoints remain accessible (back-compat)
`GET /api/v1/devices` and `GET /api/v1/alerts` continue to return 200 with their existing
response envelopes after this story's changes. No existing test that uses the direct
endpoints is broken. The search route is the PRIMARY path; the direct routes are retained
for backward compatibility with existing tests and tooling.
(traces to ADR-031 §D8-a note: "existing direct endpoints may remain for back-compat but
aren't the primary path")

### AC-007: No uncatalogued tracing event_type emissions (SAP-1)
If any new `tracing::*!(event_type = ...)` site is introduced in this story's implementation,
it must have a corresponding row in BC-2.16.002 Structured Event Catalog with full field
schema, audit role, and recurrence policy. Zero uncatalogued `event_type` emissions are
permitted per SAP-1 + PG-LP11-001.
(traces to BC-2.16.002 invariant — standing adversary probe SAP-1 enforced on every pass)

---

## Red Gate Tests

The following 11 named tests constitute the canonical Red Gate set for this story.
Count: 9 tests in `crates/prism-dtu-armis/tests/s_demo_armis_aql_001_red_gate.rs` +
2 AC-005 round-trip parity tests in `crates/prism-spec-engine/tests/parity/armis.rs`.

(TD-VSDD-091 justified-citation exception: test names are verbatim load-bearing identifiers
grounded against the delivered source files on feature/S-DEMO-ARMIS-AQL-001.)

| Test Name | AC | File | Description |
|-----------|----|------|-------------|
| `test_armis_aql_search_route_registered_returns_200_for_device_aql` | AC-001 | prism-dtu-armis/tests/s_demo_armis_aql_001_red_gate.rs | GET /api/v1/search?aql=in:devices returns 200 with valid Bearer |
| `test_armis_aql_search_returns_403_without_bearer` | AC-001 / EC-004 | prism-dtu-armis/tests/s_demo_armis_aql_001_red_gate.rs | No Authorization header on /api/v1/search returns 403 (Armis auth model) |
| `test_armis_aql_search_devices_aql_returns_device_records` | AC-002 | prism-dtu-armis/tests/s_demo_armis_aql_001_red_gate.rs | in:devices AQL returns DeviceRecord objects in data.results; total > 0 |
| `test_armis_aql_search_aql_captured_in_aql_log` | AC-002 | prism-dtu-armis/tests/s_demo_armis_aql_001_red_gate.rs | After search, GET /dtu/aql-log aql_strings contains the verbatim AQL sent (R-DTU-002) |
| `test_armis_aql_search_alerts_aql_returns_alert_records` | AC-003 | prism-dtu-armis/tests/s_demo_armis_aql_001_red_gate.rs | in:alerts AQL returns AlertRecord objects in data.results; total > 0 |
| `test_armis_aql_search_no_aql_defaults_to_devices` | AC-001 / EC-001 | prism-dtu-armis/tests/s_demo_armis_aql_001_red_gate.rs | Absent aql param returns devices (safe default per R-DTU-002) |
| `test_armis_aql_search_toml_path_template_updated` | AC-004 | prism-dtu-armis/tests/s_demo_armis_aql_001_red_gate.rs | armis.sensor.toml devices and alerts fetch steps both have path_template starting with /api/v1/search?aql= |
| `test_armis_aql_search_toml_response_path_updated` | AC-004 | prism-dtu-armis/tests/s_demo_armis_aql_001_red_gate.rs | armis.sensor.toml devices and alerts fetch steps both have response_path = $.data.results |
| `test_armis_aql_search_dtu_toml_column_parity` | AC-004 / SAP-2 | prism-dtu-armis/tests/s_demo_armis_aql_001_red_gate.rs | SAP-2: every TOML column in devices and alerts tables maps to a field in DeviceRecord/AlertRecord in types.rs |
| `test_BC_2_16_013_AC_005_aql_roundtrip_devices_pipeline` | AC-005 | prism-spec-engine/tests/parity/armis.rs | Full pipeline: devices fetch via /api/v1/search; aql-log AQL matches constructed AQL |
| `test_BC_2_16_013_AC_005_aql_roundtrip_alerts_pipeline` | AC-005 | prism-spec-engine/tests/parity/armis.rs | Full pipeline: alerts fetch via /api/v1/search; aql-log AQL matches constructed AQL |

---

## Tasks

### DTU-Side Tasks

1. **Read** `crates/prism-dtu-armis/src/clone.rs` — understand `build_router()`; note
   current route registrations (`/api/v1/devices`, `/api/v1/alerts`, `/dtu/aql-log`).
2. **Read** `crates/prism-dtu-armis/src/routes/devices.rs` — understand `get_or_post_devices`;
   note how `state.capture_aql()` is called; understand `paginate_devices()` return shape
   (`DevicesResponse { data: DevicesData { devices, total, page } }`).
3. **Read** `crates/prism-dtu-armis/src/types.rs` — understand `DeviceRecord`,
   `DevicesResponse`, `AlertRecord`, `AlertsResponse`, `AqlLogResponse`.
4. **Read** `crates/prism-dtu-armis/src/state.rs` — understand `capture_aql()`, `aql_log()`,
   `devices_ordered`, the alert fixture loading pattern.
5. **Create** `crates/prism-dtu-armis/src/routes/search.rs` (or add handler inline in
   `routes/mod.rs`) with stub for `get_search`:
   ```rust
   pub async fn get_search(
       State(state): State<Arc<ArmisState>>,
       headers: HeaderMap,
       Query(params): Query<SearchQueryParams>,
   ) -> impl IntoResponse {
       todo!()
   }
   ```
   Where `SearchQueryParams` has `aql: Option<String>`, `page: Option<u32>`,
   `size: Option<u32>`.
6. **Write Red Gate tests** (must ALL FAIL before implementation):
   - `test_armis_aql_search_route_registered_returns_200_for_device_aql`
   - `test_armis_aql_search_devices_aql_returns_device_records`
   - `test_armis_aql_search_alerts_aql_returns_alert_records`
   - `test_armis_aql_search_toml_path_template_updated`
   Verify they FAIL (RED gate confirmed) before proceeding.
7. **Register** `GET /api/v1/search` in `build_router()` in `clone.rs`:
   ```rust
   .route("/api/v1/search", get(routes::search::get_search))
   ```
   Do NOT remove existing routes (`/api/v1/devices`, `/api/v1/alerts`) — they stay.
8. **Implement** `get_search` handler:
   - Call `check_bearer_auth(&headers)` — return 403 on failure (matches Armis auth model).
   - Call `state.capture_aql(aql_str)` for the `aql` query param (R-DTU-002).
   - Inspect `aql` string to determine response type:
     - Contains `in:alerts` → serve alerts from fixture (check alerts FIRST — `in:alerts` is unambiguous)
     - Contains `in:devices` or absent/unknown → serve devices from `state.devices_ordered`
     - Absent or unknown → default to devices (safe fallback per R-DTU-002 opaque model)
   - Return response envelope: `{"data": {"results": [...], "total": N}}`
     (note: real Armis `/api/v1/search` returns `data.results`, not `data.devices` —
     update the response struct or use `serde_json::json!` to build the envelope)
   - Pagination: apply same `page`/`size` logic as `paginate_devices`.
9. **Run** `cargo nextest run -p prism-dtu-armis` — all tests GREEN.

### TOML Tasks

10. **Read** `crates/prism-sensors/specs/armis.sensor.toml` — understand current
    `fetch_devices` and `fetch_alerts` steps; note `path_template`, `response_path`,
    `pagination` fields.
11. **Update** `devices` table `fetch_devices` step:
    - `path_template = "/api/v1/search"` (was `"/api/v1/devices"`)
    - `response_path = "$.data.results"` (was `"$.data.devices"` — search envelope uses `results`)
    - Preserve `method = "GET"` and pagination config.
    - Update the DTU-EXT-003 comment: mark gap as CLOSED by this story.
12. **Update** `alerts` table `fetch_alerts` step:
    - `path_template = "/api/v1/search"` (was `"/api/v1/alerts"`)
    - `response_path = "$.data.results"` (was `"$.data.alerts"` — search envelope uses `results`)
    - Preserve `method = "GET"` and pagination config.
    - Update the DTU-EXT-004 comment: mark gap as CLOSED by this story.
13. **Verify** spec loads cleanly: `cargo nextest run -p prism-sensors` (or the
    appropriate spec-load test crate) — no parse errors on updated armis.sensor.toml.

### Parity Test Task

14. **Write** integration parity test (AC-005):
    - Start `ArmisClone` with test fixture.
    - Execute pipeline fetch for devices (or alerts) via the spec engine.
    - Query `GET /dtu/aql-log`.
    - Assert `aql_strings[0]` is the AQL string prism constructed.
    - This test may be placed in `crates/prism-dtu-armis/tests/` or in the integration
      test suite for `prism-spec-engine` if the full pipeline is needed.
15. **Run** `just check` — final pre-push gate.

---

## File List

| File | Action | Purpose |
|------|--------|---------|
| `crates/prism-dtu-armis/src/routes/search.rs` | CREATE | New `get_search` handler for `GET /api/v1/search` |
| `crates/prism-dtu-armis/src/routes/mod.rs` | MODIFY | Add `pub mod search;` if search.rs is a new file |
| `crates/prism-dtu-armis/src/clone.rs` | MODIFY | Register `GET /api/v1/search` in `build_router()` |
| `crates/prism-sensors/specs/armis.sensor.toml` | MODIFY | Update devices + alerts step path_template and response_path; close DTU-EXT-003/004 comments |
| `crates/prism-dtu-armis/tests/` | MODIFY or CREATE | Add Red Gate tests + parity test |
| `crates/prism-spec-engine/tests/parity/armis.rs` | MODIFY or CREATE | AC-005 round-trip parity tests (`test_BC_2_16_013_AC_005_aql_roundtrip_devices_pipeline` + `..._alerts_pipeline`) |
| `crates/prism-dtu-armis/src/lib.rs` | MODIFY | Module-doc route inventory update (documents `/api/v1/search` alongside existing routes) |

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| DTU AQL capture: `state.capture_aql()` called for EVERY search request | ADR-005 AQL Injection Mitigation; R-DTU-002 | Adversary probes for capture_aql() call per SAP-2 extension |
| AQL treated as opaque — stored verbatim, not parsed or validated | ADR-005 §D1; R-DTU-002 mitigation | No AQL grammar parser may be introduced in the DTU; pattern matching only |
| Response envelope: `{"data": {"results": [...], "total": N}}` | Real Armis API shape per poller-coaster-broad-sweep | SAP-2 probe: response envelope field names must match real API |
| Auth: 403 (not 401) on missing/invalid Bearer | Armis DTU auth model (AC-5, dtu-assessment.md §3.4) | Existing `check_bearer_auth` in devices.rs returns 403 |
| Direct endpoints (/api/v1/devices, /api/v1/alerts) must NOT be removed | Back-compat requirement; AC-006 | build_router() retains both routes post-story |
| No println! in production code | CLAUDE.md Conventions | Use tracing::*! with structured fields only |
| New event_type emissions require BC-2.16.002 catalog row | SAP-1 + PG-LP11-001 | Adversary greps event_type = on every pass |

### Forbidden Dependencies

`prism-sensors` (where `armis.sensor.toml` lives) MUST NOT gain new crate dependencies from
this story — it is a spec-only crate. `prism-dtu-armis` must not gain a dependency on
`prism-spec-engine` (production engine must not import test fixtures).

---

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `axum` | workspace version | DTU route handler for GET /api/v1/search |
| `serde` + `serde_json` | workspace version | Query param deserialization; JSON response |
| `prism-dtu-common` | workspace path | `load_fixture()` for alert fixture loading |
| `tokio` | workspace version | async handler |

Version source: workspace `Cargo.toml`. Do not pin independently.

---

## Previous Story Intelligence

This is the first AQL fidelity story in E-DTU-FIDELITY. Adjacent stories:

- **S-DTU-CYBERINT-AUTH-FIDELITY-001** (merged PR #164): Pattern for DTU route changes —
  read clone.rs `build_router()` first, write Red Gate tests, implement handler, update
  state as needed. The same procedure applies here.

- **S-6.10-dtu-armis** (Wave 6, S-6.10): Delivered the original prism-dtu-armis implementation
  including the AQL log infrastructure (`state.capture_aql()`, `GET /dtu/aql-log`). This
  story builds on that infrastructure — the search route is the missing piece.

- **PLUGIN-MIGRATION-001-D** (merged): Authored `armis.sensor.toml` with the DTU-EXT-003/004
  comments documenting the AQL gap. The TOML changes in this story close those gaps.

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | GET /api/v1/search with no `aql` query parameter | Handler returns devices (safe default); empty AQL not captured (or captured as empty string — consistent with existing `capture_aql` behavior) |
| EC-002 | AQL string contains both `Device` and `Alert` keywords | Handler uses first matched type (devices first); or defaults to devices if ambiguous |
| EC-003 | AQL string requests page beyond fixture size | Returns `{"data": {"results": [], "total": N}}` (N = full fixture count); consistent with paginate_devices EC-004 behavior |
| EC-004 | Missing Authorization header | 403 with `{"error": "invalid or missing bearer token", "code": 403}` per existing check_bearer_auth |
| EC-005 | aql param is very long (>4KB) | Accepted and captured verbatim (R-DTU-002: opaque pass-through); no length limit in DTU |
| EC-006 | AQL string format for alerts differs from assumed pattern | Default fallback to devices; parity test must use the exact AQL string prism constructs (drives the correct alert routing pattern from actual behavior) |
| EC-007 | Direct endpoint tests (/api/v1/devices, /api/v1/alerts) run after TOML update | Direct endpoints still return correct response; response_path in TOML now points to $.data.results which is the search route's envelope — tests using direct endpoints must validate against direct endpoint envelopes ($.data.devices / $.data.alerts) |

---

## Notes for Implementer

**Response envelope shape:** The real Armis `/api/v1/search` API returns
`{"data": {"results": [...], "total": N}}` — note `results`, not `devices` or `alerts`.
This differs from the existing direct-endpoint responses:
- `GET /api/v1/devices` → `{"data": {"devices": [...], "total": N, "page": N}}`
- `GET /api/v1/alerts` → `{"data": {"alerts": [...], "total": N}}`

The search route uses `results` as the array key in the `data` envelope. Update
`response_path` in armis.sensor.toml accordingly: `$.data.results` (not `$.data.devices`
or `$.data.alerts`). If a new `SearchResponse` struct is added to types.rs, it must use
`results` as the field name (grounded against poller-coaster-broad-sweep §API response shape).

**AQL routing in the handler:** Because the DTU does not parse AQL grammar (R-DTU-002 /
ADR-005), the handler must use simple string pattern matching to determine whether to return
devices or alerts. The real Armis AQL discriminators are `in:alerts` and `in:devices`
(per research artifact `.factory/research/armis-aql-discriminator-syntax-2026-06.md` and
poller-coaster-broad-sweep.md §4). Example approach:
```rust
let return_alerts = aql.as_deref().map(|s| s.contains("in:alerts")).unwrap_or(false);
```
Check `in:alerts` first — it is unambiguous. Absent or unrecognized AQL defaults to devices.
The adversary will verify it against the actual AQL string prism constructs in the parity
test (AC-005).

**TOML response_path update:** After changing `path_template` to `/api/v1/search`, the
`response_path` fields must also be updated. The current TOML has:
- `devices.fetch_devices.response_path = "$.data.devices"` — must change to `"$.data.results"`
- `alerts.fetch_alerts.response_path = "$.data.alerts"` — must change to `"$.data.results"`

Failure to update `response_path` while changing `path_template` will cause the pipeline to
look for `$.data.devices` in a response that has `$.data.results` → empty RecordBatch.

---

## Risk Mitigations

| Risk | Mitigation |
|------|-----------|
| response_path mismatch after TOML path_template change | Explicitly validate response_path update in Task 12; AC-005 parity test will catch non-empty results requirement |
| AQL routing ambiguity causing wrong data type returned | Parity test (AC-005) drives the correct AQL string from actual pipeline behavior; update handler to match |
| Direct endpoint tests broken by TOML change | AC-006: run existing tests against direct endpoints before committing TOML change; note that TOML change only affects pipeline fetch, not DTU routes |
| New event_type emission uncatalogued | SAP-1 sweep after implementation: `rg 'event_type\s*=' crates/ --type rust`; zero new emissions without catalog rows |

---

## Token Budget Estimate

| Context source | Estimated tokens |
|----------------|-----------------|
| This story spec | ~4,000 |
| armis.sensor.toml | ~1,500 |
| crates/prism-dtu-armis/src/clone.rs | ~2,000 |
| crates/prism-dtu-armis/src/routes/devices.rs | ~2,500 |
| crates/prism-dtu-armis/src/routes/alerts.rs | ~800 |
| crates/prism-dtu-armis/src/state.rs | ~2,000 |
| crates/prism-dtu-armis/src/types.rs | ~1,000 |
| ADR-031 §D8-a (relevant section) | ~1,500 |
| POLLER-DTU-FIDELITY-AUDIT-2026-05-29.md §3 (Armis section) | ~1,500 |
| Test files (existing prism-dtu-armis/tests/) | ~2,000 |
| crates/prism-spec-engine/tests/parity/armis.rs (AC-005 parity tests) | ~800 |
| Tool outputs (cargo nextest) | ~2,000 |
| **Total estimate** | **~21,600 tokens (~8% of 256K context)** |

Well within the 20-30% budget.

---

## References

- ADR-031 v1.2 §D8-a — Armis AQL Endpoint Fidelity (Gap-AR-001/DTU-EXT-003/004)
- POLLER-DTU-FIDELITY-AUDIT-2026-05-29.md §3 — Armis Centrix fidelity table
- `.factory/research/armis-aql-discriminator-syntax-2026-06.md` — HIGH-confidence research
  establishing `in:devices`/`in:alerts` as the correct Armis AQL entity-discriminator syntax;
  closes F-LP12-HIGH-001 (story-side discriminator conformance to real Armis API and BC-2.16.013)
- `crates/prism-dtu-armis/src/clone.rs` — build_router() current state
- `crates/prism-dtu-armis/src/routes/devices.rs` — existing AQL capture pattern
- `crates/prism-sensors/specs/armis.sensor.toml` — DTU-EXT-003/004 comments

---

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 1.10 | 2026-08-02 | story-writer | Added ## Authority section (DRIFT-STORY-AUTHORITY-ABSENT-CORPUS-001 Round 6, D-2084). Synced stale `**Version:**` pseudo-field and H1 title from v1.8 to v1.10 to match frontmatter (batch-17 cross-slice consistency, orchestrator-authorized). |
| 1.9 | 2026-06-02 | state-manager | D-950 post-merge burst: status in-progress→merged; merge metadata added (merged_sha: eb3416d1, merged_pr: 168, merged_at: 2026-06-02). LOCAL 3/3 CONVERGED + PR-LEVEL 3/3 CONVERGED (BC-5.39.001 D-779). POL-14 BC-2.16.013 already active — idempotent confirm. |
| 1.8 | 2026-06-02 | story-writer | Correct AC-001..AC-004 BC-2.16.013 §Postconditions trace anchors (phantom/inverted §-labels removed; R-DTU-002 re-attributed to ADR-031 §D8-a). Closes ADV-P02-MED-001 (POL-4/POL-21/POL-22). No code/behavior change. |
| 1.7 | 2026-06-01 | product-owner | F-LP12-HIGH-001 closure (spec-side): conformed all AQL discriminator examples from `in:type=Device`/`in:type=Alert` to real Armis syntax `in:devices`/`in:alerts` per research artifact `.factory/research/armis-aql-discriminator-syntax-2026-06.md` (HIGH confidence, 6 convergent sources including real 1898 & Co production poller). Updated: §Origin narrative, §Story-Level Goal routing bullets, AC-001 example URL, AC-002 H2 title + example URLs + aql-log capture value, AC-003 H2 title + example AQL string, Red Gate table descriptions (rows 1/3/5), Task 8 handler routing bullets, Notes for Implementer AQL routing example and discriminator guidance, §References (added research artifact citation). Handler routing now checks `in:alerts` first (unambiguous) then defaults to `in:devices`. BC-2.16.013 was already correct; story now matches it. |
| 1.6 | 2026-06-01 | story-writer | F-P7-MED-001 structural-table-completeness sweep (POL-29 step 3d): added `prism-spec-engine` to `crates_touched:` frontmatter; added §File List rows for `crates/prism-spec-engine/tests/parity/armis.rs` (AC-005 round-trip parity tests) and `crates/prism-dtu-armis/src/lib.rs` (module-doc route inventory); added §Token Budget Estimate row for `prism-spec-engine` parity tests (~800 tokens); updated total estimate to ~21,600 tokens. No semantic content change. |
| 1.5 | 2026-06-01 | story-writer | POL-23 sibling-sweep: BC-2.16.013 version pins swept v1.18→v1.19 in frontmatter comment (line 10), behavioral_contracts comment (line 41), body §Behavioral Contracts table, §Behavioral Contracts note, and §New-BC Flags section. POL-7 title fix: restored full verbatim H1 "Bundled Sensor Spec Authoring and DTU-Parity Verification — 4 Initial Sensors" (the "— 4 Initial Sensors" suffix was dropped in prior versions). POL-13 status fix: frontmatter and H1 block status flipped ready→in-progress (implementation in flight, cascade pending). |
| 1.4 | 2026-06-01 | story-writer | Close F-P2-HIGH-001. Red Gate Tests table names reconciled verbatim against delivered source (feature/S-DEMO-ARMIS-AQL-001). Five phantom names removed/corrected: `test_armis_aql_search_missing_auth_returns_403` → `test_armis_aql_search_returns_403_without_bearer`; two split TOML-path rows collapsed to single `test_armis_aql_search_toml_path_template_updated` (covers both tables); two split SAP-2 rows collapsed to single `test_armis_aql_search_dtu_toml_column_parity` (covers both tables); omitted `test_armis_aql_search_aql_captured_in_aql_log` and `test_armis_aql_search_toml_response_path_updated` added. Count remains 11 (unchanged — v1.3 count reconciliation was correct; v1.4 closes the name-correctness gap that made v1.3 a paper-fix per TD-VSDD-059). |
| 1.3 | 2026-06-01 | story-writer | Close adversary findings F-P1-HIGH-003/F-P1-OBS-001/F-P1-OBS-002. Red Gate Tests table reconciled to 11 named tests (9 in s_demo_armis_aql_001_red_gate.rs + 2 AC-005 parity tests in prism-spec-engine/tests/parity/armis.rs); red_gate_tests frontmatter updated from 4→11. H1 version corrected v1.0→v1.3 (F-P1-OBS-001). Changelog reordered to descending per POL-32 (F-P1-OBS-002). |
| 1.2 | 2026-05-31 | story-writer | Wave 5 dispatch burst: BC-2.16.013 anchor justification confirmed per POL-4/POL-5 (BC-2.16.013 §Postconditions §1 DTU-Parity + §2 fixture-parity + §Known Gaps DTU-EXT-003/004 directly cover the AQL search endpoint fidelity surface; D-911 SUFFICIENT disposition). Story confirmed ready for TDD dispatch. No semantic content change. |
| 1.1 | 2026-05-31 | story-writer | D-911 disposition applied: New-BC Flags 1 & 2 SUFFICIENT — BC-2.16.013 (v1.18, ACTIVE) covers both surfaces. Set behavioral_contracts: [BC-2.16.013], status: draft→ready. AC-001..AC-004 BC traces updated from "pending PO authorship" to BC-2.16.013 postcondition clauses. D-914 parity-gate note added (AC-005/AC-006 parity tests soft-gated on S-SPEC-ENV-VAR-001 env-var prereq; must be #[ignore] until prereq merges). |
| 1.0 | 2026-05-31 | story-writer | Initial materialization from [stub] per ADR-031 §D8-a v1.2 reclassification. 7 ACs, 4 Red Gate tests, 5 pts, wave 5, P1. Grounded against crates/prism-dtu-armis/src/routes/devices.rs (AQL capture pattern), types.rs (DeviceRecord/AlertRecord/AqlLogResponse), state.rs (capture_aql/aql_log), clone.rs (build_router), armis.sensor.toml (DTU-EXT-003/004 comments). New-BC flags provided to product-owner for AQL syntax validation and R-DTU-002 BC coverage evaluation. |
