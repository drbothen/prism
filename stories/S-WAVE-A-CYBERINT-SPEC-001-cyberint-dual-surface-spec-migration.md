---
document_type: story
story_id: S-WAVE-A-CYBERINT-SPEC-001
title: "Cyberint Dual-Surface Spec Migration — Delete cyberint.sensor.toml; Author cyberint-alerts and cyberint-assets; OpenAPI-Ground Alerts C2-Class Fixes; DTU Route Migration"
version: "1.8"
modified: "2026-07-27"
status: draft
producer: story-writer
phase: 3
wave: wave-a
epic_id: E-WAVE-A-SENSOR-REMEDIATION
priority: P0
points: 8
tdd_mode: strict
target_module: prism-sensors
subsystems: ["SS-06 (SensorSpec)", "SS-07 (SpecEngine)", "SS-12 (DTU-Cyberint)"]
depends_on:
  - S-WAVE-A-ENGINE-001    # header_scheme grammar + Rule 9 must be live before new cyberint specs can load;
                           # S-WAVE-A-CYBERINT-PATCH-001 (the minimal co-land patch) co-lands with ENGINE-001,
                           # so when ENGINE-001 is done, the boot-failure hazard is already closed
blocks: []
behavioral_contracts:
  - BC-2.01.006
  - BC-2.01.018
  - BC-2.06.003
  - BC-2.16.001
  - BC-2.16.002
  - BC-2.16.009
verification_properties:
  - VP-153
estimated_days: 3
# BC status: BC-2.01.006 v1.8 covers Cyberint Assets surface. BC-2.01.018 v1.6 covers Cyberint
# Alerts surface (introduced 2026-07-22; §Story Anchor resolves to this story; v1.6 re-grounded
# on ADR-056 PageNumber pagination in FB66). BC-2.16.001 and BC-2.16.009 are existing contracts.
# BC-2.06.003 covers credential-ref rename. All BCs must be reviewed at status-transition time.
assumption_validations: []
risk_mitigations: []
---

# S-WAVE-A-CYBERINT-SPEC-001: Cyberint Dual-Surface Spec Migration

## Authority

**ADR-053 v0.38 §D3-a** (Cyberint Dual-Surface Schema) is the primary authority for
the two-spec split mandated by this story. ADR-053 §D3-a establishes that
`cyberint.sensor.toml` is superseded and deleted, replaced by `cyberint-alerts.sensor.toml`
(Alerts surface, `/alert` prefix) and `cyberint-assets.sensor.toml` (Assets surface,
`/asset-configuration` prefix), each bound to a distinct server prefix and OpenAPI file.
Read §D3-a in full before implementing:
`.factory/specs/architecture/decisions/ADR-053-wave-a-sensor-fidelity-remediation-openapi-grounding-armis-token-exchange-cyberint-dual-surface.md`

**ADR-056 v0.4** (PageNumber Pagination Variant) is the authority for the
`PaginationConfig::PageNumber` grammar added by T-09 and used in `cyberint-alerts.sensor.toml`.
ADR-056 designates this story (`wiring_deferred_to: S-WAVE-A-CYBERINT-SPEC-001`) as
the implementation site for all `PageNumber` dispatch sites in `prism-spec-engine`. Read
§D3/§D4/§D10 before implementing T-09; §D10 enumerates the compile-error sites
(CE-1 through CE-4) that must be resolved atomically with the `PageNumber` variant:
`.factory/specs/architecture/decisions/ADR-056-page-number-pagination-variant.md`

---

## Scheduling Note (No Co-land Constraint)

This story does NOT need to co-land atomically with `S-WAVE-A-ENGINE-001`.

The boot-failure hazard (E-SPEC-027(c) from `cyberint.sensor.toml` lacking `header_scheme`)
is closed by `S-WAVE-A-CYBERINT-PATCH-001`, which co-lands with ENGINE-001. By the time
this story is dispatched (after ENGINE-001 merges), the existing spec already has
`header_scheme = "cookie:access_token"` and boot is healthy.

This story's `depends_on: [S-WAVE-A-ENGINE-001]` is for implementation ordering only:
the new spec files use `header_scheme = "cookie:access_token"` which requires the
`header_scheme` field grammar that ENGINE-001 adds to `SensorSpec`. The full 8-point
migration can proceed on its own schedule after ENGINE-001 + PATCH-001 are merged.

---

## Narrative

As a Prism maintainer, I want the Cyberint sensor spec split into two surface-scoped spec
files (`cyberint-alerts.sensor.toml` and `cyberint-assets.sensor.toml`) with correct
OpenAPI-grounded endpoint paths, methods, response shapes, pagination, and the
`header_scheme = "cookie:access_token"` declaration — so that (a) Prism boots successfully
after S-WAVE-A-ENGINE-001 ships, (b) the Cyberint Alerts surface uses the correct POST
method and `$.alerts` response path, (c) the DTU behavioral clone reflects the corrected
wire shape, and (d) no spec file in the workspace carries the stale `auth_type =
"cookie_roundtrip"` + absent `header_scheme` combination that triggers E-SPEC-027(c).

---

## Header-Scheme Sweep Report

A sweep of all `*.sensor.toml` files in the workspace was conducted prior to this story's
authoring to identify all files requiring `header_scheme` migration. Directories covered:
`crates/prism-sensors/specs/` (including the `customers/` sub-directory),
`crates/prism-bin/fixtures/sensors/`, and `.prism/specs/sensors/`.

| File | auth_type | header_scheme present? | Action |
|------|-----------|------------------------|--------|
| `crates/prism-sensors/specs/cyberint.sensor.toml` | `cookie_roundtrip` | absent | DELETE — replace with two new specs |
| `crates/prism-sensors/specs/armis.sensor.toml` | `bearer_static` | absent (path A — no field needed) | None |
| `crates/prism-sensors/specs/claroty.sensor.toml` | `bearer_static` | absent (path A — no field needed) | None |
| `crates/prism-sensors/specs/crowdstrike.sensor.toml` | `oauth2_client_credentials` | absent (path A — no field needed) | None |
| `crates/prism-sensors/specs/customers/acme/armis.sensor.toml` | not defined (overlay) | N/A | None |
| `crates/prism-sensors/specs/customers/contoso/armis.sensor.toml` | not defined (overlay) | N/A | None |
| `crates/prism-bin/fixtures/sensors/test-sensor-with-cred-refs.sensor.toml` | `api_key` | absent (path A — no field needed) | None |
| `.prism/specs/sensors/` | directory does not exist on disk | N/A | None |

**Result: Only `cyberint.sensor.toml` is affected.**

---

## Breaking Change Notice (Operator Impact)

This story changes credential environment variable names. Operators must update
secrets management before deploying a build that includes this story.

| Surface | Old env var | New env var |
|---------|------------|-------------|
| Alerts  | `PRISM_CLIENTS_{ID}_SENSORS_CYBERINT_API_KEY` | `PRISM_CLIENTS_{ID}_SENSORS_CYBERINT_ALERTS_ACCESS_TOKEN` |
| Assets  | (new surface — no prior env var) | `PRISM_CLIENTS_{ID}_SENSORS_CYBERINT_ASSETS_ACCESS_TOKEN` |

Resolution chain for each: `…_FILE` path > env var > keyring, per BC-2.06.003 §Resolution Chain.

---

## Acceptance Criteria

### AC-001: Old spec deleted; two new specs load cleanly at startup
(traces to BC-2.16.001 postcondition — all spec files in the bundle directory load without
error at startup)

The file `crates/prism-sensors/specs/cyberint.sensor.toml` is deleted.
The bundled spec load test (part of `just check`) completes without error when both
`cyberint-alerts.sensor.toml` and `cyberint-assets.sensor.toml` are present.
Startup with ENGINE-001's `validate_sensor_spec()` wired in reports no E-SPEC-027(c)
errors for either new spec.

### AC-002: header_scheme = "cookie:access_token" on both new specs
(traces to BC-2.16.009 Rule 9 postcondition — `header_scheme` field present and valid for
`cookie_roundtrip` auth_type)

Both `cyberint-alerts.sensor.toml` and `cyberint-assets.sensor.toml` declare:
```
auth_type = "cookie_roundtrip"
header_scheme = "cookie:access_token"
```
A unit test in `prism-spec-engine` asserts that loading either new spec produces no
E-SPEC-027(c) error when Rule 9 runs.

### AC-003: Alerts surface uses POST /alert/api/v1/alerts with $.alerts response path and page_number pagination
(traces to BC-2.01.018 postcondition — Alerts surface returns the correct data at the
correct endpoint with the correct response extraction path; traces to BC-2.16.002
PageNumber Pagination Dispatch postcondition — POST body injection of `page`/`size` per
ADR-056 §D3)

`cyberint-alerts.sensor.toml` tables.alerts.steps.fetch_alerts declares:
- `method = "POST"`
- `path_template = "/api/v1/alerts"` (relative to base_url which includes `/alert` prefix)
- `response_path = "$.alerts"`
- Pagination type: page_number with `page_size = 100`

The `page_number` variant maps to `PaginationConfig::PageNumber { page_size: 100 }` in
`spec_parser.rs`. For POST method, `build_paged_url_impl` returns the URL unchanged and
`build_request` injects `"page": (offset + 1)` and `"size": 100` as top-level JSON body
keys (ADR-056 §D3). First request emits `page = 1`; advance is `offset += 1` per ADR-056 §D2.

The DTU `get_alerts` handler (in `routes/alerts.rs`) is updated to:
- Register at POST `/alert/api/v1/alerts` (not GET `/api/v1/alerts`)
- Return `{"total": N, "alerts": [...]}` (not `{"data": [...], "next_cursor": "..."}`) — OpenAPI `GetAlertsResponse` has ONLY `{total, alerts}`; no `"page"` key exists in the real API response

A parity test asserts that a POST to `/alert/api/v1/alerts` with valid `access_token` cookie
returns a JSON object with a top-level `"alerts"` key containing an array.

### AC-004: Page/size pagination replaces cursor pagination; prism engine dispatches via POST body
(traces to BC-2.01.018 postcondition — pagination returns complete result sets across
multiple pages; traces to BC-2.16.002 PageNumber Pagination Dispatch postcondition —
POST body carries `page`/`size` keys; first request body asserted on the wire per
CLAUDE.md §Wire-shape assertion discipline)

`ac_6_cursor_pagination.rs` is deleted. A new test `ac_6_page_size_pagination.rs` verifies
(all requests use POST body JSON, not query parameters — the Cyberint API reads `page`/`size`
from `GetAlertsRequest` POST body, not query string):
- POST `/alert/api/v1/alerts` with JSON body `{"page": 1, "size": 10}` returns the first 10 alerts
- POST `/alert/api/v1/alerts` with JSON body `{"page": 2, "size": 10}` returns the next 10 alerts (different records, no overlap)
- POST `/alert/api/v1/alerts` with JSON body `{"page": 1, "size": 1000}` is capped to `max_page_size = 100` (DTU enforces cap)
- A multi-page accumulation via sequential body-keyed requests returns all alerts equal to total reported by `total` field

Wire-shape assertion (CLAUDE.md §Wire-shape assertion discipline): the prism engine
(via `PaginationConfig::PageNumber` + `build_request` POST injection per ADR-056 §D3)
sends a POST body that, on the first request, serializes to a JSON object containing
`"page": 1` and `"size": 100`. The second-page body contains `"page": 2` and `"size": 100`.
Advance rule: `offset += 1` per ADR-056 §D2 (distinct from `OffsetLimit`'s `offset += page_size`).
Termination: `if page_record_count < page_size { break }` per ADR-056 §D4.
At least one test in `ac_6_page_size_pagination.rs` MUST assert on the raw serialized JSON
body sent to the DTU — not only on the Rust `AlertListParams` struct.

`AlertListParams` in `routes/alerts.rs` is updated from `cursor: Option<String>` to
`page: Option<u32>, size: Option<u32>`. The handler MUST extract `AlertListParams` from
the POST request JSON body (not query parameters), matching the real Cyberint
`GetAlertsRequest` POST body contract (ADR-028 §D1 — DTU wire shape mirrors real API).

### AC-005: credential_refs name = "access_token" on both new specs
(traces to BC-2.06.003 postcondition — credential references use the canonical field name
established by ADR-032 and ADR-053 D3)

Both new specs declare:
```toml
[[credential_refs]]
name = "access_token"
description = "Cyberint API access token (injected as access_token cookie)"
```

No spec file in the workspace declares `[[credential_refs]] name = "api_key"` for a
Cyberint surface. Any test fixture that previously referenced `api_key` as the Cyberint
credential name is updated to `access_token`.

### AC-006: cyberint-assets.sensor.toml has correct auth/base_url skeleton
(traces to BC-2.16.001 postcondition — spec file is structurally valid and parseable)

`cyberint-assets.sensor.toml` contains valid TOML that parses without error, including:
- `sensor_id = "cyberint-assets"`
- `auth_type = "cookie_roundtrip"`
- `header_scheme = "cookie:access_token"`
- `base_url = "https://${env.CYBERINT_ENVIRONMENT}.cyberint.io/asset-configuration"`
- At least one `[[tables]]` block (table structure derived from the Cyberint assets
  OpenAPI; see Task T-03 for derivation requirements)

### AC-007: Incidents table removed from cyberint-alerts.sensor.toml
(traces to BC-2.01.006 postcondition — sensor surface accurately reflects the Cyberint
Alerts API scope; the incidents surface has no confirmed API endpoint)

`cyberint-alerts.sensor.toml` contains ONLY the `alerts` table. The `incidents` table
from the old `cyberint.sensor.toml` is NOT carried over. The gap note (EC-016-013-002)
is documented in a comment but no TOML `[[tables]]` block for incidents is created.

### AC-008: No sensor_id = "cyberint" reference remains in test fixtures or docs
(traces to BC-2.01.006 postcondition — stale sensor_id causes misrouted credentials and
wrong probe-table lookups)

A grep of `crates/` and `.factory/stories/` for the literal string `sensor_id.*=.*"cyberint"`
(excluding this story file) returns only:
- The deleted `cyberint.sensor.toml` (already deleted)
- Historical comments referencing the old ID

Any test fixture JSON/TOML or Rust string literal that previously embedded `"cyberint"` as
a sensor_id is updated to `"cyberint-alerts"` or `"cyberint-assets"` as appropriate.

### AC-009: IOC nested fields present in serialized alerts wire output on BOTH paths
(traces to BC-2.01.018 postcondition — wire output includes all TOML-declared IOC
source_path columns; CLAUDE.md §Wire-shape assertion discipline)

The static-fixture path in `get_alerts()` (non-seeded clone, `fixture_gen_seeded == false`)
must emit the nested IOC fields (`ioc`, `iocs`, `alert_data`) in each per-record JSON object
when those fields are populated in the fixture. An implementer following T-05 item 5 MUST NOT
use a hand-built `json!` literal that enumerates only the 8 top-level keys (`alert_id`,
`title`, `severity`, `status`, `created_at`, `source`, `type`, `affected_assets`); instead,
per-record construction must use the full `Alert` struct serialization (e.g.,
`serde_json::to_value(a)`) so IOC fields are included automatically.

At least one test (RG-019) asserts on the serialized JSON response of a static-fixture-path
clone:
- POST to `/alert/api/v1/alerts` against a non-seeded clone whose fixture contains
  IOC-populated records
- The serialized `alerts[N]` contains the `iocs` key (not absent) with at least one element
- The serialized `alerts[N]` contains the `alert_data` key (not absent) when the fixture
  carries an alert with `alert_data`

This is a wire-shape assertion on the per-record level, not only on the envelope. The
generated-records path already emits IOC fields correctly; the static-fixture path parity
is what this AC enforces.

### AC-010: Assets DTU route registered; all 11 assets columns backed by `Asset` struct fields
(traces to BC-2.01.006 postcondition — Cyberint Assets surface is reachable from
spec-driven queries; SAP-2 column parity requirement)

T-04 adds the `Asset` struct to `crates/prism-dtu-cyberint/src/types.rs` with all 11
fields that back the columns in `cyberint-assets.sensor.toml`. T-08 creates
`crates/prism-dtu-cyberint/src/routes/assets.rs` with a handler registered at
`POST /asset-configuration/external/api/v1/assets/`.

Every column declared in `cyberint-assets.sensor.toml` maps to a field in `Asset`:
`id` (i64), `name` (Option<String>), `asset_type` (Option<String>), `status`
(Option<String>), `asset_group` (Option<String>), `created` (String), `updated`
(String), `parent_asset_value` (Option<String>), `discovery_precision` (Option<i64>),
`discovery_reason` (Option<String>), `severity` (Option<String>).

At least one test (RG-020) asserts:
- POST to `/asset-configuration/external/api/v1/assets/` with valid `access_token` cookie
  returns HTTP 200 with a JSON body containing a top-level `"assets"` array key

### AC-011: `affected_assets` column declared in `cyberint-alerts.sensor.toml` alerts table
(traces to BC-2.02.004 §TOML Contract postcondition — `affected_assets` field exposed as
TOML column so `SpecDrivenMapper` can populate `raw_extensions.affected_assets`;
F-WASE-P66-HIGH-003 story leg)

`cyberint-alerts.sensor.toml` `alerts` table declares `affected_assets` column with
`column_type = "json"` and `ocsf_field = "raw_extensions.affected_assets"`.

A Red Gate test (RG-021) asserts the parsed `cyberint-alerts.sensor.toml` spec has an
`affected_assets` column with `ColumnType::Json` in the `alerts` table.

---

## Architecture Mapping

| Component | Module | Pure/Effectful | Architecture Section |
|-----------|--------|---------------|----------------------|
| `cyberint-alerts.sensor.toml` | `crates/prism-sensors/specs/` | Pure (config data) | `architecture/module-decomposition.md §SS-06 SensorSpec` |
| `cyberint-assets.sensor.toml` | `crates/prism-sensors/specs/` | Pure (config data) | `architecture/module-decomposition.md §SS-06 SensorSpec` |
| `PaginationConfig::PageNumber` variant | `crates/prism-spec-engine/src/spec_parser.rs` | Pure (enum variant) | `architecture/module-decomposition.md §SS-07 SpecEngine` |
| `build_paged_url_impl` (PageNumber arm) | `crates/prism-spec-engine/src/pipeline.rs` | Pure (URL construction) | `architecture/module-decomposition.md §SS-07 SpecEngine` |
| `build_request` (PageNumber POST body injection) | `crates/prism-spec-engine/src/pipeline.rs` | Effectful (HTTP request builder) | `architecture/module-decomposition.md §SS-07 SpecEngine` |
| `execute_impl` (PageNumber active_page_size + advance/terminate) | `crates/prism-spec-engine/src/pipeline.rs` | Effectful (pagination loop) | `architecture/module-decomposition.md §SS-07 SpecEngine` |
| `CyberintClone::build_router()` | `crates/prism-dtu-cyberint/src/clone.rs` | Effectful (HTTP server) | `architecture/module-decomposition.md §SS-12 DTU-Cyberint` |
| `get_alerts()` handler | `crates/prism-dtu-cyberint/src/routes/alerts.rs` | Effectful (HTTP handler) | `architecture/module-decomposition.md §SS-12 DTU-Cyberint` |
| `AlertListParams` | `crates/prism-dtu-cyberint/src/routes/alerts.rs` | Pure (body param struct, JSON-extracted) | `architecture/module-decomposition.md §SS-12 DTU-Cyberint` |
| `PaginationType::Page` variant (CE-1) | `crates/prism-spec-engine/src/types.rs` | Pure (enum variant) | `architecture/module-decomposition.md §SS-07 SpecEngine` |
| `sensor_table_descriptor_from_table_spec` (PageNumber → Page arm, CE-1) | `crates/prism-spec-engine/src/types.rs` | Pure (struct conversion) | `architecture/module-decomposition.md §SS-07 SpecEngine` |
| `validate_sensor_spec` §Category 4 (PageNumber arm, CE-2) | `crates/prism-spec-engine/src/validation.rs` | Pure (validation) | `architecture/module-decomposition.md §SS-07 SpecEngine` |
| `Asset` struct (T-04) | `crates/prism-dtu-cyberint/src/types.rs` | Pure (data type) | `architecture/module-decomposition.md §SS-12 DTU-Cyberint` |
| `get_assets()` handler (T-08) | `crates/prism-dtu-cyberint/src/routes/assets.rs` | Effectful (HTTP handler) | `architecture/module-decomposition.md §SS-12 DTU-Cyberint` |

---

## Behavioral Contracts

| BC | Version | Relevance to This Story |
|----|---------|------------------------|
| BC-2.01.006 | v1.8 | Cyberint Assets surface — cookie auth, multi-format timestamp parsing |
| BC-2.01.018 | v1.6 | Cyberint Alerts surface — POST method, $.alerts response path, page/size pagination (re-grounded on ADR-056 PageNumber in FB66; cursor pagination superseded) |
| BC-2.06.003 | v1.12 | Credential refs resolution chain; `access_token` name change |
| BC-2.16.001 | v1.9 | Bundled spec loading at startup — both new specs must pass validation |
| BC-2.16.002 | v2.11 | Multi-Step Fetch Pipeline — PageNumber Pagination Dispatch postcondition (ADR-056 §D3/§D4); `PaginationConfig::PageNumber` wiring in `spec_parser.rs`, `build_paged_url_impl`, `build_request`, and `execute_impl` in `pipeline.rs` |
| BC-2.16.009 | v1.29 | Rule 9: `cookie_roundtrip` requires `header_scheme = "cookie:<name>"` — absence path (c) must NOT trigger |

---

## UX / Operator Surfaces

None — this story produces no user-facing UI changes. The credential env-var rename is
the only operator-visible surface change (documented in Breaking Change Notice above).

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | cyberint.sensor.toml still present after migration | AC-001 fails; MERGE-GATE fires if bundled spec load test is green with old spec still present |
| EC-002 | Both new specs present; S-WAVE-A-ENGINE-001 not yet merged | New specs do NOT require header_scheme validation to pass (Rule 9 not yet live); specs load without error |
| EC-003 | POST /alert/api/v1/alerts with missing access_token cookie | DTU returns HTTP 401 — same auth failure path as before |
| EC-004 | POST `/alert/api/v1/alerts` with body `{"size": 0}` | DTU normalizes to 1 (min page size); returns first result |
| EC-005 | POST `/alert/api/v1/alerts` with body `{"page": 9999, "size": 100}` | DTU returns empty `{"alerts": [], "page": 9999, "total": N}` — not an error |
| EC-006 | cyberint-assets.sensor.toml probe_table points to nonexistent table | BC-2.16.001 spec load fails; implementation must derive probe_table from actual tables block |
| EC-007 | Operator uses old CYBERINT_API_KEY env var after migration | Credential resolver falls through to keyring; if not in keyring, sensor returns E-SENSOR-004 (credential not found) — correct fail-open behavior; NOT a silent 401 |

---

## Tasks

### Red Gate tests (to be written by test-writer BEFORE implementation)

- [ ] **RG-001**: `test_bundled_spec_load_no_sensor_id_cyberint_after_deletion` — AC-001
  _(Runs full bundled spec load after `cyberint.sensor.toml` is deleted; asserts no sensor with `sensor_id = "cyberint"` is present in the registry; verifies AC-008 transitively — the old sensor_id literal is gone entirely)_

- [ ] **RG-002**: `test_cyberint_alerts_toml_header_scheme_cookie_access_token` — AC-002
  _(Reads `crates/prism-sensors/specs/cyberint-alerts.sensor.toml`; asserts top-level `header_scheme = "cookie:access_token"` present and correct)_

- [ ] **RG-003**: `test_cyberint_assets_toml_header_scheme_cookie_access_token` — AC-002
  _(Reads `crates/prism-sensors/specs/cyberint-assets.sensor.toml`; asserts top-level `header_scheme = "cookie:access_token"` present and correct; verifies both surfaces carry the field)_

- [ ] **RG-004**: `test_cyberint_alerts_toml_method_post_response_path_alerts` — AC-003
  _(Parses `cyberint-alerts.sensor.toml` as TOML; asserts alerts table `method = "POST"` and `response_path = "$.alerts"`; EC-005 transitively covered — wrong method/path shape detected here)_

- [ ] **RG-005**: `test_dtu_cyberint_post_alert_api_v1_alerts_returns_alerts_array` — AC-003
  _(SAP-3: sends a real POST request to the DTU cyberint route `/alert/api/v1/alerts`; asserts response contains an `alerts` array key; confirms DTU shape matches TOML `response_path = "$.alerts"` — end-to-end surface reachability from POST method down to response shape)_

- [ ] **RG-006**: `test_dtu_cyberint_alert_response_shape_no_data_no_cursor_keys` — AC-003
  _(Asserts DTU cyberint alerts response does NOT contain old `data` key or `cursor_token` key; confirms old shape is fully replaced; EC-006 transitively covered — cursor key absent)_

- [ ] **RG-007**: `test_pagination_page_number_post_body_first_page_is_page_1_size_100` — AC-004
  _(Drives `PaginationConfig::PageNumber` with POST body mode; asserts first request body contains `"page": 1` and `"size": 100` as top-level JSON integer keys (ADR-056 §D3 canonical wire keys — NOT TOML declaration names `page_number`/`page_size`); covers AC-004 first-page arm)_

- [ ] **RG-008**: `test_pagination_page_number_second_request_body_is_page_2_size_100` — AC-004
  _(Drives same path on second iteration; asserts request body contains `"page": 2` and `"size": 100` as top-level JSON integer keys; covers AC-004 advance arm `offset += 1`)_

- [ ] **RG-009**: `test_pagination_page_number_terminate_on_page_shorter_than_page_size` — AC-004
  _(Returns a page with fewer items than `page_size`; asserts loop terminates and all items accumulated; covers AC-004 termination arm)_

- [ ] **RG-010**: `test_cyberint_alerts_credential_ref_name_is_access_token` — AC-005
  _(Parses `cyberint-alerts.sensor.toml`; asserts `credential_refs` entry has `name = "access_token"`; EC-007 transitively covered — wrong credential name detected here)_

- [ ] **RG-011**: `test_cyberint_assets_credential_ref_name_is_access_token` — AC-005
  _(Parses `cyberint-assets.sensor.toml`; asserts `credential_refs` entry has `name = "access_token"`; verifies both surfaces use the renamed credential key)_

- [ ] **RG-012**: `test_cyberint_assets_toml_skeleton_valid_sensor_id_base_url_probe_table` — AC-006
  _(Parses `cyberint-assets.sensor.toml`; asserts `sensor_id`, `base_url`, and at least one `[[tables]]` block are present and non-empty; bundled spec load passes with no errors for the assets spec)_

- [ ] **RG-013**: `test_cyberint_alerts_toml_no_incidents_table` — AC-007
  _(Parses `cyberint-alerts.sensor.toml`; asserts no `[[tables]]` block has `name = "incidents"`; confirms the incidents table is not carried forward)_

- [ ] **RG-014**: `test_workspace_no_sensor_id_cyberint_literal_in_crates` — AC-008
  _(Greps `crates/` for the literal string `"cyberint"` as a sensor_id value; asserts zero matches for the old monolithic sensor_id; confirms the split names `"cyberint-alerts"` and `"cyberint-assets"` are used in all test fixtures)_

- [ ] **RG-015**: `test_pagination_page_number_get_request_url_contains_page_1_size_100` — AC-004 (GET arm)
  _(Drives `PaginationConfig::PageNumber` with GET URL mode; asserts first request URL contains `page=1` and `size=100` query params; covers GET-method URL injection arm of AC-004)_

- [ ] **RG-016**: `test_pagination_page_number_multi_page_accumulation_equals_total` — AC-004
  _(Simulates 3-page response; asserts accumulated item count equals sum of all three pages; covers AC-004 multi-page collection accumulation arm end-to-end)_

- [ ] **RG-017**: `test_pagination_page_number_page_size_zero_rejected_at_spec_load` — CE-2 (ADR-056 §D10)
  _(Drives `validate_sensor_spec` (or `SpecLoader::parse`) with a step declaring `type = "page_number"` and `page_size = 0` in its `[tables.steps.pagination]` block; asserts `SpecErrorCode::ESpec001` is returned with message `"page_number pagination in step '{}' requires page_size > 0"` (ADR-056 §D10 CE-2, §D3 spec-load layer); confirms spec-load rejection occurs BEFORE any pagination loop execution, eliminating the `ps = 0` runaway path described in ADR-056 §D4 where `page_record_count < 0` is always false and the loop would run to `MAX_PAGES_PER_STEP`)_

- [ ] **RG-018**: `test_sensor_table_descriptor_page_number_pagination_type_is_page_on_wire` — CE-1 wire-shape (ADR-056 §D10)
  _(Builds or loads a sensor spec containing a step with `type = "page_number"` pagination; calls the `list_sensor_specs` MCP tool surface or directly exercises `sensor_table_descriptor_from_table_spec`; asserts the serialized JSON output for that table's descriptor contains `"pagination_type": "Page"` (PascalCase — `PaginationType` carries NO `rename_all` attribute, so `Page` serializes as `"Page"` not `"page"`) — NOT `"offset"`, NOT `"cursor"`, NOT `null` (CLAUDE.md §Wire-shape assertion discipline: MCP-visible surfaces require at least one assertion on the serialized JSON `SensorTableDescriptor` shape, not only on the Rust `PaginationType::Page` variant; LLM agents consuming `list_sensor_specs` must receive correct pagination semantics; ADR-056 v0.3 §D10 established the `"Page"` wire literal); confirms ADR-056 §D10 CE-1 mapping `PaginationConfig::PageNumber { .. } => PaginationType::Page` is correct and wire-visible)_

- [ ] **RG-019**: `test_dtu_cyberint_static_fixture_path_alerts_include_ioc_nested_fields` — AC-009 (F-SAP2-CRIT-001 story leg)
  _(Non-seeded clone (static-fixture path, `fixture_gen_seeded == false`): POST to `/alert/api/v1/alerts` with valid access_token cookie; asserts at least one record in the `alerts` array contains a non-null `iocs` key with at least one element, or a non-null `alert_data` key with at least one populated subfield; confirms static-fixture per-record construction emits IOC fields and does NOT suppress them via the retired 8-key `json!` literal; CLAUDE.md §Wire-shape assertion discipline — asserts on serialized JSON response not Rust struct state; path-dependence is the defect: this test specifically exercises the static-fixture path)_

- [ ] **RG-020**: `test_dtu_cyberint_assets_route_post_returns_assets_array` — AC-010 (F-SAP2-CRIT-002 story leg)
  _(POST to `/asset-configuration/external/api/v1/assets/` with valid `access_token` cookie; asserts HTTP 200 and serialized JSON response contains a top-level `"assets"` array key; confirms assets route is registered in `clone.rs`, handler exists in `routes/assets.rs`, and response shape matches `cyberint-assets.sensor.toml` `response_path = "$.assets"`; CLAUDE.md §Wire-shape assertion discipline: asserts on serialized JSON envelope as the HTTP client receives it)_

- [ ] **RG-021**: `test_cyberint_alerts_toml_affected_assets_column_is_json_type` — AC-011 (F-WASE-P66-HIGH-003 story leg)
  _(Parses `crates/prism-sensors/specs/cyberint-alerts.sensor.toml`; asserts the `alerts` table contains a column named `"affected_assets"` with `column_type = "json"` (i.e., `ColumnType::Json` after spec loading); mirrors the existing IOC column Red Gate test pattern; TOML ground truth: `Alert.affected_assets: Vec<serde_json::Value>` emitted by `routes::alerts::get_alerts` via `Alert` struct serialization per T-05 item 5 — BC-2.02.004 §TOML Contract)_

**Red Gate density check** (BC-5.38.001): **21 failing tests** before implementation begins. RG-001 covers AC-001 (delete/load); RG-002/RG-003 cover AC-002 (header_scheme both surfaces); RG-004/RG-005/RG-006 cover AC-003 (POST + `$.alerts` path, DTU shape); RG-007/RG-008/RG-009/RG-015/RG-016 cover AC-004 (pagination arms); RG-010/RG-011 cover AC-005 (credential_refs access_token); RG-012 covers AC-006 (assets skeleton); RG-013 covers AC-007 (no incidents); RG-014 covers AC-008 (no old sensor_id); RG-017 covers CE-2 (`page_size = 0` spec-load rejection per ADR-056 §D10); RG-018 covers CE-1 wire-shape (`PaginationType::Page` on serialized `SensorTableDescriptor` per ADR-056 §D10); RG-019 covers AC-009 (IOC nested fields on static-fixture path — F-SAP2-CRIT-001 story leg); RG-020 covers AC-010 (assets route registered and response shape correct — F-SAP2-CRIT-002 story leg); RG-021 covers AC-011 (`affected_assets` column `ColumnType::Json` in alerts table — F-WASE-P66-HIGH-003 story leg). RED_RATIO is computed by the orchestrator at Step 3.5 per per-story-delivery.md from actual Red Gate results; BC-5.38.002 and BC-5.38.003 define the exempt test classes (green-by-design and wiring-exempt) that reduce the denominator.

### Implementation tasks

### T-01: Delete cyberint.sensor.toml
**Files:** `crates/prism-sensors/specs/cyberint.sensor.toml` (DELETE)

Verify that the deletion does not break any Rust test that hardcodes the file path by
name (grep for `cyberint.sensor.toml` in `crates/` before deleting). Update any such
references to point to `cyberint-alerts.sensor.toml`.

### T-02: Author cyberint-alerts.sensor.toml
**Files:** `crates/prism-sensors/specs/cyberint-alerts.sensor.toml` (CREATE)

Required fields (authoritative — not optional):

```toml
sensor_id = "cyberint-alerts"
name = "Cyberint Alerts"
auth_type = "cookie_roundtrip"
header_scheme = "cookie:access_token"
probe_table = "alerts"
base_url = "https://${env.CYBERINT_ENVIRONMENT}.cyberint.io/alert"
version = "1.0.0"

[[credential_refs]]
name = "access_token"
description = "Cyberint API access token (injected as access_token cookie)"
```

Alerts table columns — carry over from `cyberint.sensor.toml` alerts table exactly,
preserving all IOC columns (ioc_type, ioc_value_singleton, iocs_type, iocs_value,
iocs_value_first, alert_data_ip, alert_data_domain, alert_data_url), OCSF mappings,
source_path annotations, and timestamp_formats chains. The column schema was
adversarially validated and must not be silently altered.

**Additional required column — `affected_assets` (BC-2.02.004 §TOML Contract, F-WASE-P66-HIGH-003):**
Add `affected_assets` to the alerts table `[[tables.columns]]` block:

```toml
  # F-SAP2-MED-004 / BC-2.02.004 §TOML Contract: affected_assets — wire-key "affected_assets"
  # present in per-record JSON via Alert.affected_assets: Vec<serde_json::Value> (Alert struct
  # serialization per T-05 item 5). Prior deferral (F-LP3-HIGH-001) superseded by ColumnType::Json.
  [[tables.columns]]
  name = "affected_assets"
  column_type = "json"
  ocsf_field = "raw_extensions.affected_assets"
```

This column is covered by AC-011 and RG-021 below.

**Carry-over exclusion (F-SAP2-MED-003):** Do NOT carry over DTU-parity comments that
reference the retired cursor wire shape from the old `cyberint.sensor.toml`. Specifically,
omit or replace any TOML comments mentioning the following cursor-era artifacts:
- `{"data": [...], "next_cursor": ...}` response shape descriptions
- `# Cursor-based pagination` labels on table or step blocks
- `page_size: OMITTED` rationale with `DTU-EXT-005` citation
- `DTU route: GET /api/v1/alerts` (pre-migration route — now POST under `/alert` prefix)
- `next_cursor` key references in pagination commentary

Replace cursor-era pagination rationale with:
`# ADR-056 PageNumber: POST body injects {"page": N, "size": 100}; DTU route: POST /alert/api/v1/alerts`

These stale comments constitute false ground truth for future SAP-2 probes and must not
be carried forward even if the column schema itself is correct.

Alerts fetch step — corrected per ADR-053 C2-class fixes and ADR-056 PageNumber ratification:

```toml
[[tables.steps]]
name = "fetch_alerts"
method = "POST"
path_template = "/api/v1/alerts"
response_path = "$.alerts"
variables_produced = []
[tables.steps.pagination]
type = "page_number"
page_size = 100
```

`type = "page_number"` is the ratified `PaginationConfig::PageNumber { page_size: 100 }` variant
(ADR-056 §D1). For POST method, `build_paged_url_impl` returns the URL unchanged and
`build_request` injects `"page": (offset + 1)` and `"size": 100` as top-level JSON body keys
(ADR-056 §D3). The Cyberint Alerts API `GetAlertsRequest` schema declares `page` (integer,
minimum 1, default 1) and `size` (integer, minimum 10, maximum 100, default 10) as POST body
fields — these are NOT query parameters. The `page_number` variant satisfies this contract
exactly. `page_size = 100` is within the API's accepted range (maximum 100).

The `PaginationConfig::PageNumber` variant must be added to `spec_parser.rs` in T-09
(see §Tasks) before this TOML file can be loaded. Do NOT use `offset_limit` (which
injects `offset`/`limit` keys) or `cursor_token` for this surface.

Do NOT carry over the `incidents` table (see AC-007).

### T-03: Author cyberint-assets.sensor.toml
**Files:** `crates/prism-sensors/specs/cyberint-assets.sensor.toml` (CREATE)

Required header:
```toml
sensor_id = "cyberint-assets"
name = "Cyberint Assets"
auth_type = "cookie_roundtrip"
header_scheme = "cookie:access_token"
base_url = "https://${env.CYBERINT_ENVIRONMENT}.cyberint.io/asset-configuration"
version = "1.0.0"

[[credential_refs]]
name = "access_token"
description = "Cyberint API access token (injected as access_token cookie)"
```

**OpenAPI grounding source (authoritative):** `.factory/reference/api-specs/cyberint_assets_openapi_06.20.2026.json`
The file is present in-repo; no stub is acceptable. The OpenAPI `servers` block declares
`url: "/asset-configuration"` — so `base_url` above is correct. The primary assets endpoint
is `POST /external/api/v1/assets/` (path relative to the server prefix); `path_template`
for the assets fetch step is `/external/api/v1/assets/`.

**Pagination treatment — GAP-ASSETS-PAG-001 [EXPLICIT BLOCKER]:**

The assets OpenAPI `GetAssetsRequest` schema declares `page_number: integer (minimum 1,
default 1)` but NO `page_size` parameter — the server controls how many assets are returned
per page. `GetAssetsResponse` carries `total_assets: integer`, `page_number: integer`, and
`assets: array`.

**Consequence of the current grammar gap: silent data truncation (CWE-390 class).**
Without a multi-page loop, prism sends no `page_number` parameter (the server applies its
default of page 1) and only the first server-default page is retrieved. Every asset beyond
page 1 is silently dropped. `GetAssetsResponse.total_assets` is the evidence field:
if `total_assets > len(assets)` in the first response, assets were missed and the result
set is incomplete. This is the same silent-truncation defect class as F-WASE-P64-CRIT-003
on the alerts surface, which this very burst closes.

**Why no current `PaginationConfig` variant works:**
- `PaginationConfig::PageNumber { page_size: u32 }` (ADR-056) requires a client-specified
  `page_size` for loop termination (`if page_record_count < page_size { break }`). When
  page size is not client-controlled this condition misfires — a half-full server-default
  page would terminate the loop prematurely.
- A correct variant for this endpoint must terminate via `total_assets` comparison
  (e.g., `if accumulated_count >= total_assets { break }`) or by detecting an empty page
  against an API-reported total, not against a client-declared page size. This variant is
  not yet designed and requires a new ADR plus a BC-2.16.002 amendment.

**GAP-ASSETS-PAG-001** — Assets multi-page pagination is blocked on a new
`PaginationConfig` variant supporting server-controlled page size with termination driven
by `GetAssetsResponse.total_assets` or by an empty/short page from an API-reported total.
The follow-up story for this grammar extension does not yet exist; it awaits orchestrator
story creation. The alerts table is unaffected — `GetAlertsRequest` has a client-specifiable
`size` (maximum 100) and uses the ratified `PageNumber` variant (ADR-056).

Until GAP-ASSETS-PAG-001 is resolved, the implementing engineer MUST:
- Author `cyberint-assets.sensor.toml` WITHOUT a `[tables.steps.pagination]` block
  (first-page-only retrieval is the only safe option given the current grammar)
- Add a comment inside `cyberint-assets.sensor.toml` directly above the fetch step:
  `# GAP-ASSETS-PAG-001: pagination block absent — only page 1 retrieved until a`
  `# server-controlled-page-size PaginationConfig variant is designed and ratified.`
  `# GetAssetsResponse.total_assets evidences truncation when total_assets > len(assets).`
- NOT present the first-page-only spec as a complete, production-grade implementation

**Do NOT add a `[tables.steps.pagination]` block to the assets fetch step.**

**Table column schema — derived from `Asset` OpenAPI schema:**
Map the following `Asset` fields to TOML columns with the specified `column_type`:

| API field | column_type | Nullable | Notes |
|-----------|-------------|----------|-------|
| `id` | `Integer` | No | Required field in OpenAPI schema |
| `name` | `String` | Yes | `anyOf: [string, null]` |
| `type` | `String` | Yes | `anyOf: [AssetTypes enum, null]` — serialize as string |
| `status` | `String` | Yes | `anyOf: [string, null]` |
| `asset_group` | `String` | Yes | `anyOf: [string, null]` |
| `created` | `Datetime` | No | Required; `format: date-time`; example `"2024-11-07T12:43:29Z"` |
| `updated` | `Datetime` | No | Required; `format: date-time` |
| `parent_asset_value` | `String` | Yes | `anyOf: [string, null]` |
| `discovery_precision` | `Integer` | Yes | `anyOf: [integer, null]` |
| `discovery_reason` | `String` | Yes | `anyOf: [string, null]` |
| `severity` | `String` | Yes | `anyOf: [string, null]` |

(Omit `compensating_controls` — it is an array of objects; use `Json` column_type if
OCSF mapping requires it, or omit if no downstream mapping exists. The field is nullable
in the OpenAPI schema.)

Response path for the assets fetch step: `$.assets` (from `GetAssetsResponse` which
returns `{ "total_assets": N, "page_number": N, "assets": [...] }`).

`probe_table = "assets"` (BC-2.08.001 postcondition 5 — the LIMIT-0 health probe targets
the first declared table; `assets` is the primary table).

SAP-2 compliance: every column name in `cyberint-assets.sensor.toml` MUST have a
corresponding field in `crates/prism-dtu-cyberint/src/types.rs` (or in the new DTU
types added by T-04). Missing-column-in-DTU = P1 CRITICAL (SAP-2 protocol).

### T-04: Update DTU route registration for /alert prefix and POST method
**Files:** `crates/prism-dtu-cyberint/src/clone.rs` (MODIFY)

In `build_router()` (the axum Router construction method):
- Change the alerts list route from `routing::get(get_alerts)` (or `routing::get(get_alerts).post(get_alerts)`) at `/api/v1/alerts` to `routing::post(get_alerts)` at `/alert/api/v1/alerts`.
- Update the alert detail route (`/api/v1/alerts/{alert_id}`) to `/alert/api/v1/alerts/{alert_id}`.
- Update the alert status patch route to `/alert/api/v1/alerts/{alert_id}/status`.
- Update the alert close route to `/alert/api/v1/alerts/{alert_id}/close`.

All four alert routes gain the `/alert` prefix. The threat intel route (if present) is
NOT in the alerts surface and does NOT gain the `/alert` prefix unless confirmed by the
Assets OpenAPI.

SAP-3 compliance: at least one integration test must POST to `/alert/api/v1/alerts`
(full route including the `/alert` prefix) to confirm reachability from the public surface.

**Additional scope — add `Asset` struct to `types.rs` (F-SAP2-CRIT-002 / MED-001 / MED-002):**
**Files (additional):** `crates/prism-dtu-cyberint/src/types.rs` (MODIFY)

Add the `Asset` struct to `types.rs` so that T-03's SAP-2 column parity check has a
backing struct BEFORE T-08 creates the route handler. Grounding schema: the `Asset`
definition from `.factory/reference/api-specs/cyberint_assets_openapi_06.20.2026.json`.

Type disambiguation (F-SAP2-MED-002): the OpenAPI contains two conflicting type definitions.
Always use the `Asset` schema — do NOT use the sibling schema or the `Threat` schema:
- `id`: `integer` in `Asset` schema (NOT `string` from `Threat` schema) → Rust type `i64`
- `discovery_precision`: `integer|null` in `Asset` schema (NOT `number` from sibling schema)
  → Rust type `Option<i64>`

```rust
/// A single Cyberint asset record.
///
/// Grounding schema: `Asset` from `cyberint_assets_openapi_06.20.2026.json`.
/// Type disambiguation: `id` → i64 (Asset, not Threat); `discovery_precision` → Option<i64>
/// (integer|null from Asset, not number from sibling schema).
#[non_exhaustive]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    pub id: i64,
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub asset_type: Option<String>,
    pub status: Option<String>,
    pub asset_group: Option<String>,
    pub created: String,
    pub updated: String,
    pub parent_asset_value: Option<String>,
    pub discovery_precision: Option<i64>,
    pub discovery_reason: Option<String>,
    pub severity: Option<String>,
}
```

**Non-exhaustive gate — MANDATORY CI requirement:** Adding `Asset` as a new `#[non_exhaustive]`
public type requires updating `scripts/check-non-exhaustive-per-symbol.py` in the same commit
per CLAUDE.md §Conventions (`#[non_exhaustive]` discipline):

Append `"Asset"` to the `EXPECTED_SYMBOLS` list in `scripts/check-non-exhaustive-per-symbol.py`.
This is the ONLY required change — `EXPECTED_COUNT` is derived automatically from the list
length, and `scripts/check-non-exhaustive.sh` reads the count from the Python manifest.
Do NOT update any separate numeric count value in any file. Do NOT update a count sentence
in CLAUDE.md — CLAUDE.md explicitly states "Do NOT restate the count in prose anywhere."

T-03's SAP-2 compliance check ("every column in `cyberint-assets.sensor.toml` MUST have
a corresponding field in `types.rs` or in the new DTU types added by T-04") is satisfied
once T-04 is complete — the `Asset` struct added here backs all 11 assets columns declared
in T-03. T-08 uses this struct to implement the route handler.

### T-05: Update DTU alerts handler for page/size pagination and $.alerts response shape
**Files:** `crates/prism-dtu-cyberint/src/routes/alerts.rs` (MODIFY)

1. Replace `AlertListParams { cursor: Option<String> }` with:
   ```rust
   #[derive(Debug, Deserialize, Default)]
   pub struct AlertListParams {
       pub page: Option<u32>,
       pub size: Option<u32>,
   }
   ```

2. Update `get_alerts()` handler to:
   - Extract `AlertListParams` from the **POST request JSON body** (not query parameters)
     using `axum::extract::Json<AlertListParams>` — the real Cyberint API takes `GetAlertsRequest`
     as a POST body; the DTU must mirror this (ADR-028 §D1 DTU wire-shape parity)
   - Accept `page` (1-indexed, default 1) and `size` (default 25, capped at 100)
   - Return `Json(serde_json::json!({ "total": total_count, "alerts": [...] }))`
     (OpenAPI `GetAlertsResponse` has ONLY `{total, alerts}` — NO `page` field; the DTU
     mirrors the real API per ADR-028 §D1; do NOT add a `"page"` key that the real API
     does not emit)
   - Remove the cursor-based response shape (`"data": [...], "next_cursor": "..."`)

3. Update the state module if needed: remove any `cursor_store` or cursor-generation logic;
   add page/size slicing logic using `alerts_fixture.chunks(size)[page-1]` (or equivalent).

4. The `access_token` cookie auth and rate-limit logic (`check_auth`, `extract_access_token`)
   are NOT changed by this story.

5. **IOC nested fields — static-fixture path (F-SAP2-CRIT-001):** The static-fixture path
   (non-seeded clone, `fixture_gen_seeded == false`) currently builds per-record JSON with a
   hand-crafted `json!` literal that enumerates only 8 top-level keys (`alert_id`, `title`,
   `severity`, `status`, `created_at`, `source`, `type`, `affected_assets`). This silently
   drops `ioc`, `iocs`, and `alert_data` — the IOC fields that `iocs_value` and
   `iocs_value_first` TOML columns depend on for their `source_path` lookups.
   **The 8-key `json!` literal MUST be replaced.** Use `serde_json::to_value(a)` (or
   `serde_json::to_value(a.clone())`) to serialize the full `Alert` struct so all
   populated fields — including `ioc`, `iocs`, and `alert_data` — are included
   automatically. The generated-records path already emits IOC fields (records are served
   as-is); the static-fixture path must be brought into parity.

   Path-dependence is the defect: seeded demo scenarios (generated-records path) pass SAP-2
   checks; unseeded production runs (static-fixture path) silently yield nothing for IOC
   columns because the hand-built literal suppresses them.

Wire-shape assertion (CLAUDE.md §Wire-shape assertion discipline): at least one test in
`tests/f_p3_route_output_tests.rs` or a new test file must assert on the serialized
JSON output — the exact top-level keys and array structure as the HTTP client receives it.
Test must verify `"alerts"` key present, `"data"` key absent, `"next_cursor"` key absent.

Additionally, at least one test must cover the STATIC-FIXTURE PATH (non-seeded clone) and
assert on the per-record IOC fields (RG-019). The path-dependence is the defect: the
generated-records path already emits IOC fields correctly; the static-fixture path was
silently suppressing them via the 8-key `json!` literal.

### T-06: Replace ac_6_cursor_pagination.rs with ac_6_page_size_pagination.rs
**Files:**
- `crates/prism-dtu-cyberint/tests/ac_6_cursor_pagination.rs` (DELETE)
- `crates/prism-dtu-cyberint/tests/ac_6_page_size_pagination.rs` (CREATE)

New test must cover:
- AC-004 page 1 vs page 2 return different records (no overlap)
- AC-004 size cap: `size=1000` capped to 100 results
- AC-004 multi-page accumulation equals total count
- Wire-shape assertion: POST to `/alert/api/v1/alerts` returns JSON with top-level
  `"alerts"` array and no `"data"` or `"next_cursor"` keys

### T-07: Update all test fixtures and references to sensor_id = "cyberint"
**Scope:** grep `crates/` for string literals `"cyberint"` used as a sensor_id value.

Update:
- Any JSON fixture files under `crates/prism-dtu-cyberint/` that embed `sensor_id: "cyberint"` → `sensor_id: "cyberint-alerts"` (or `"cyberint-assets"` where appropriate)
- Any Rust test string literals that pass `"cyberint"` as a sensor_id in a `SpecLoader::load_spec()` or `add_sensor_spec()` call → update to `"cyberint-alerts"` or `"cyberint-assets"`
- Any parity test or fidelity_validator that hard-codes `sensor_id = "cyberint"` in its assertion

Do NOT update references in:
- History comments (leave as documentation of old behavior)
- This story file itself
- Other story files in `.factory/stories/`

### T-08: Create DTU assets route handler and register it in clone.rs
**Files:**
- `crates/prism-dtu-cyberint/src/routes/assets.rs` (CREATE)
- `crates/prism-dtu-cyberint/src/routes/mod.rs` (MODIFY — add `pub mod assets;`)
- `crates/prism-dtu-cyberint/src/clone.rs` (MODIFY — register assets route)

**No stub path (F-SAP2-HIGH-001).** The dead conditional branch ("If Assets OpenAPI
available: full route… else: `assets_stub.rs` returning empty 200") has been removed
entirely. The OpenAPI file IS present at
`.factory/reference/api-specs/cyberint_assets_openapi_06.20.2026.json` (confirmed by T-03).
An empty 200 stub is wire-indistinguishable from "zero assets" (CWE-390 silent-truncation
class) and is explicitly prohibited.

**Grounding schema:** `Asset` from `cyberint_assets_openapi_06.20.2026.json`. Use the
same type resolution as T-04: `id` → `i64`, `discovery_precision` → `Option<i64>`. The
`Asset` struct was added to `types.rs` by T-04; this task uses it without modification.

**Route registration** — in `clone.rs` `build_router()`, add:
```rust
.route(
    "/asset-configuration/external/api/v1/assets/",
    post(routes::assets::get_assets),
)
```
The path includes the `/asset-configuration` base prefix because the DTU serves both
surfaces on a single port. The method is POST, matching `cyberint-assets.sensor.toml`
`method = "POST"`.

**Handler:** implement `get_assets()` in `routes/assets.rs`:
- Auth check via `check_auth()` (same cookie auth as alerts — ADR-031 §D3-a)
- Load fixture via `prism_dtu_common::load_fixture_as(crate_dir, "assets")` → `Vec<Asset>`
- Return: `Json(serde_json::json!({"assets": assets_vec, "total_assets": total, "page_number": 1}))`
- The per-record serialization MUST use the full `Asset` struct (via `serde_json::to_value`
  or direct struct inclusion) — not a hand-built `json!` literal with a field subset.
- If fixture file is absent or empty, return `{"assets": [], "total_assets": 0, "page_number": 1}`
  (not an error — empty is valid for a new clone)

**SAP-2 compliance:** every field emitted in the per-record JSON corresponds to a field
in the `Asset` struct from T-04. All 11 TOML columns have backing struct fields:
`id`, `name`, `asset_type` (`type`), `status`, `asset_group`, `created`, `updated`,
`parent_asset_value`, `discovery_precision`, `discovery_reason`, `severity`.

**Wire-shape assertion (AC-010):** at least one test must POST to
`/asset-configuration/external/api/v1/assets/` with a valid `access_token` cookie and
assert the serialized JSON response contains a top-level `"assets"` array key.

**Non-exhaustive gate:** `Asset` was added as a `#[non_exhaustive]` type by T-04;
the three-site gate update (EXPECTED 92→93) was performed in T-04. T-08 adds no
new public `#[non_exhaustive]` types; no further gate updates required in this task.

**GAP-ASSETS-PAG-001 (unchanged):** the pagination block is intentionally absent from
`cyberint-assets.sensor.toml` (authored in T-03). The assets route in T-08 returns
page 1 only. This is the correct first-page-only retrieval per T-03's GAP-ASSETS-PAG-001
disclosure. Do NOT add multi-page logic here; that awaits the follow-up grammar extension.

### T-09: Implement PaginationConfig::PageNumber variant in prism-spec-engine
**Files:**
- `crates/prism-spec-engine/src/spec_parser.rs` (MODIFY)
- `crates/prism-spec-engine/src/pipeline.rs` (MODIFY)
- `crates/prism-spec-engine/src/types.rs` (MODIFY — CE-1)
- `crates/prism-spec-engine/src/validation.rs` (MODIFY — CE-2)

ADR-056 designates this story (`wiring_deferred_to: S-WAVE-A-CYBERINT-SPEC-001`) as the
implementation site. All implementation obligations from ADR-056 §D10 and §Consequences must be implemented
in a single atomic commit together with the TOML spec from T-02:

**1. `spec_parser.rs` — add `PageNumber { page_size: u32 }` to `PaginationConfig`**

Add the variant adjacent to `OffsetLimit { page_size: u32 }` with a doc comment stating
that `offset` is reused as a 0-based page index and the wire parameter is `offset + 1`.
The enum already carries `#[serde(tag = "type", rename_all = "snake_case")]`; the serde
tag `page_number` is derived automatically — no explicit `#[serde(rename = ...)]` needed.
Do NOT add a second `#[non_exhaustive]` attribute (already present on the enum). Do NOT
append any new symbol to `EXPECTED_SYMBOLS` in `scripts/check-non-exhaustive-per-symbol.py`
for this task — adding a variant to an existing `#[non_exhaustive]` enum does not add a new
annotated symbol (ADR-056 §D9). Note: T-04 in this story already appended `"Asset"` to
`EXPECTED_SYMBOLS` in `scripts/check-non-exhaustive-per-symbol.py`; T-09 does not add
any further annotated symbols.

**2. `pipeline.rs` — `build_paged_url_impl` new match arm (ADR-056 §D3)**

```
Some(PaginationConfig::PageNumber { page_size }) => {
    if step.method.eq_ignore_ascii_case("POST") {
        base_url.to_string()
    } else {
        let page = offset + 1;
        let sep = if base_url.contains('?') { '&' } else { '?' };
        format!("{base_url}{sep}page={page}&size={page_size}")
    }
}
```

**3. `pipeline.rs` — `build_request` POST-body injection block (ADR-056 §D3)**

Add a `PageNumber` injection block parallel to the existing `OffsetLimit` block. Guard:
`step.method.eq_ignore_ascii_case("POST") && matches!(step.pagination, Some(PaginationConfig::PageNumber { .. })) && page_size > 0`.
When guard fires, inject `"page": (offset + 1)` and `"size": page_size` as top-level integer
keys into the JSON body, merged onto the interpolated `body_template`. Merge semantics and
error paths identical to `OffsetLimit` POST dispatch (non-object `body_template` →
`Err(SpecEngineError::HttpRequestFailed { ... })`).

**4. `pipeline.rs` — `execute_impl` `active_page_size` derivation extension (ADR-056 §D3)**

Extend the pattern arm with `|`-syntax:
```
Some(PaginationConfig::OffsetLimit { page_size: ps })
| Some(PaginationConfig::PageNumber { page_size: ps }) => *ps,
```

**5. `pipeline.rs` — `execute_impl` pagination advance/terminate block (ADR-056 §D4)**

```
Some(PaginationConfig::PageNumber { page_size }) => {
    let ps = *page_size as usize;
    if page_record_count < ps {
        break;
    }
    offset += 1;
}
```

Advance is `offset += 1` — MUST NOT be `offset += page_size`. This is a mandatory
distinction from `OffsetLimit` (ADR-056 §D2).

**6. `types.rs` — `PaginationType::Page` variant and `sensor_table_descriptor_from_table_spec` arm (ADR-056 §D10 CE-1)**

Add `Page` variant to the `PaginationType` enum adjacent to `Offset` and `Cursor`:
```
Page,
```
The serde serialization for `PaginationType::Page` produces the string `"Page"` (PascalCase) —
`PaginationType` carries NO `rename_all` attribute on its `#[derive(Serialize, Deserialize)]`,
so variant names serialize verbatim. The LLM agent consuming `list_sensor_specs` MCP output
sees this as `"pagination_type": "Page"` (not `"page"`; ADR-056 v0.3 §D10 established this).
This is a distinct semantic from `Offset` (`"Offset"`) and must NOT be folded into the
existing `Offset` variant (ADR-056 §D10 CE-1 wire-visibility rationale).

Extend the `PaginationConfig` → `PaginationType` mapping in `sensor_table_descriptor_from_table_spec`
(or its equivalent in `types.rs`):
```
PaginationConfig::PageNumber { .. } => PaginationType::Page,
```
This is a compile-error site — `PaginationConfig::PageNumber` is a new variant, so existing
`match` arms over `PaginationConfig` that are non-exhaustive will fail to compile until this
arm is added (CE-1).

**7. `validation.rs` — `validate_sensor_spec` §Category 4 PageNumber arm (ADR-056 §D10 CE-2)**

Extend `validate_sensor_spec` §Category 4 (pagination validation) with a new match arm:
```
PaginationConfig::PageNumber { page_size } if *page_size == 0 => {
    return Err(SpecEngineError::InvalidSpec {
        code: SpecErrorCode::ESpec001,
        message: format!(
            "page_number pagination in step '{}' requires page_size > 0",
            step.name
        ),
    });
}
```
This arm fires at spec-load time — before any pagination loop runs. It eliminates the
`ps = 0` runaway path (ADR-056 §D4: when `page_size = 0`, `page_record_count < 0` is
always false, causing the loop to run until `MAX_PAGES_PER_STEP`). This is a compile-error
site — the `validate_sensor_spec` match over `PaginationConfig` variants (if exhaustive)
requires the `PageNumber` arm (CE-2).

**Test coverage (SAP-3 — end-to-end from `PipelineExecutor::execute`, not synthetic-AST):**
- POST path: first request body contains `"page": 1` and `"size": 100`; second page body
  contains `"page": 2` and `"size": 100`
- GET path: URL contains `?page=1&size=100`; second page URL contains `?page=2&size=100`
- Termination: a page returning fewer records than `page_size` ends the loop; no additional
  request is issued after the terminal page
- Non-object `body_template` with `PageNumber` POST → `Err(SpecEngineError)` returned
- `page_size = 0` → spec-load rejection: `validate_sensor_spec` §Category 4 returns
  `SpecErrorCode::ESpec001` with message `"page_number pagination in step '{}' requires
  page_size > 0"` (ADR-056 §D10 CE-2); the spec-load failure prevents the loop from
  executing with `ps = 0`; RG-017 covers this path
- `PaginationType::Page` wire-shape: serialized `SensorTableDescriptor.pagination_type`
  field = `"page"` (NOT `"offset"`, NOT `"cursor"`, NOT `null`) per CLAUDE.md §Wire-shape
  assertion discipline; at least one test must assert on the serialized JSON output of
  `sensor_table_descriptor_from_table_spec` or the `list_sensor_specs` MCP surface (CE-1);
  RG-018 covers this path

These tests are the Red Gate tests for `PaginationConfig::PageNumber` pipeline behavior.
They live in `crates/prism-spec-engine/tests/` or inline test modules in `src/pipeline.rs`.

---

## Token Budget Estimate

| Context source | Estimated tokens |
|----------------|-----------------|
| This story spec | ~4,500 |
| `cyberint.sensor.toml` (source of alerts column schema) | ~2,500 |
| `crates/prism-dtu-cyberint/src/clone.rs` (route registration) | ~2,000 |
| `crates/prism-dtu-cyberint/src/routes/alerts.rs` (handler + AlertListParams) | ~3,000 |
| `crates/prism-dtu-cyberint/tests/ac_6_cursor_pagination.rs` (to delete) | ~800 |
| `crates/prism-sensors/specs/armis.sensor.toml` (pagination grammar reference) | ~1,000 |
| `crates/prism-sensors/specs/claroty.sensor.toml` (POST pagination reference) | ~1,500 |
| `crates/prism-spec-engine/src/spec_parser.rs` (PaginationConfig definition) | ~1,500 |
| `crates/prism-spec-engine/src/pipeline.rs` (build_paged_url_impl, build_request, execute_impl) | ~4,000 |
| ADR-056 (PageNumber grammar decision — §D1/§D2/§D3/§D4/§D9) | ~2,500 |
| ADR-053 §D3-a and §C2-class fixes | ~1,000 |
| BC-2.16.002 §Postconditions (OffsetLimit + PageNumber dispatch rows) | ~1,500 |
| BC-2.16.009 Rule 9 (header_scheme validation) | ~800 |
| BC-2.06.003 (credential refs) | ~500 |
| BC-2.01.018 v1.6 (Cyberint Alerts contract — PageNumber pagination, ADR-056 re-grounding) | ~800 |
| `.factory/reference/api-specs/cyberint_assets_openapi_06.20.2026.json` (assets schema) | ~1,500 |
| Running test output (nextest per-crate) | ~2,000 |
| **Total estimate** | **~31,400** |

30,600 tokens is at the upper end of the 20–30% context window limit for a standard
100k-token agent context (30.6%). This story is at the boundary; the implementer should
load only the sections of `pipeline.rs` and `spec_parser.rs` relevant to `PaginationConfig`
and `build_paged_url_impl` rather than the full files. If context pressure materializes,
T-09 (prism-spec-engine changes) can be dispatched as a focused sub-burst before the DTU
tasks. No formal story split is required.

---

## Previous Story Intelligence

**From S-WAVE-A-ENGINE-001 (dependency):**
- `header_scheme = "cookie:access_token"` is the exact field value required for
  `auth_type = "cookie_roundtrip"` sensors per BC-2.16.009 Rule 9.
- `SENSOR_ID_RE` CWE-22 path-traversal check runs BEFORE Rule 9. `sensor_id = "cyberint-alerts"`
  and `sensor_id = "cyberint-assets"` must match `^[a-zA-Z0-9_-]{1,64}$` or the equivalent
  regex. Verify both IDs pass before writing the TOML files.
- The co-land MERGE-GATE-CYBERINT in ENGINE-001 blocks ENGINE from merging without this story.
  The wave-scheduler must order ENGINE-001 and S-WAVE-A-CYBERINT-SPEC-001 into the same
  release batch.

**From PLUGIN-MIGRATION-001-D (general lessons):**
- SAP-2 parity check: every column name must match a field in the DTU types.rs struct.
  F-LP3-CRIT-001 in that cascade was caused by column names drifting from DTU struct fields.
  Do not add columns that have no matching DTU field.
- Wire-shape assertions: tests that only assert pre-serialization Rust struct state missed the
  `"data"` → `"alerts"` response-shape bug class. Add JSON-body assertion on the raw response.

---

## Architecture Compliance Rules

Extracted from `architecture/module-decomposition.md` and applicable ADRs:

1. **ADR-028 §D1 — DTU-grounded spec authoring.** TOML spec URLs must be grounded from
   the DTU route registration, not from free-form inference. After this story updates the
   DTU routes, the spec's path_template MUST reflect the DTU routes as modified by T-04,
   not the pre-story DTU routes.

2. **ADR-053 §D3-a — Dual-surface split is required.** A monolithic `cyberint.sensor.toml`
   that covers both alerts and assets surfaces is not an acceptable output. Two separate
   files with two separate `sensor_id` values are mandatory.

3. **ADR-031 §D3-a — Cookie auth model is unchanged.** The `access_token` cookie
   authentication mechanism in the DTU is retained; only the route path and pagination
   parameters change. Do not alter `extract_access_token()` or `check_auth()`.

4. **CLAUDE.md §Wire-shape assertion discipline.** Any test covering the DTU's HTTP
   surface must assert on the serialized JSON output, not only on Rust structs.

5. **BC-2.06.003 §Resolution Chain.** Credential lookup order: file path env var >
   plain-value env var > keyring. The `access_token` name change does not alter the
   lookup order; it only changes the env var suffix from `_API_KEY` to `_ACCESS_TOKEN`.

6. **Non-exhaustive gate.** If new `#[non_exhaustive]` public types are added to
   `prism-dtu-cyberint` by T-08, append the new symbol name to `EXPECTED_SYMBOLS` in
   `scripts/check-non-exhaustive-per-symbol.py` ONLY — `EXPECTED_COUNT` is derived
   automatically from the list length; no numeric count updates elsewhere.

---

## Library & Framework Requirements

| Library | Version | Source of truth |
|---------|---------|----------------|
| `axum` | pinned in workspace `Cargo.toml` | `architecture/dependency-graph.md §External Dependencies` |
| `serde` / `serde_json` | pinned in workspace `Cargo.toml` | same |
| `reqwest` (if used in assets DTU) | `default-features = false, features = ["rustls-tls"]` | ADR-050; CLAUDE.md §reqwest TLS backend |

No new external dependencies are introduced by this story.

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/prism-spec-engine/src/spec_parser.rs` | MODIFY | Task T-09; add `PageNumber { page_size: u32 }` to `PaginationConfig`; serde tag `page_number` automatic; do NOT bump non-exhaustive gate count |
| `crates/prism-spec-engine/src/pipeline.rs` | MODIFY | Task T-09; three dispatch sites — `build_paged_url_impl` new arm, `build_request` POST injection, `execute_impl` active_page_size + advance/terminate per ADR-056 §D3/§D4 |
| `crates/prism-spec-engine/src/types.rs` | MODIFY | Task T-09; CE-1 (ADR-056 §D10): add `PaginationType::Page` variant to `PaginationType` enum; extend `sensor_table_descriptor_from_table_spec` with `PaginationConfig::PageNumber { .. } => PaginationType::Page` arm |
| `crates/prism-spec-engine/src/validation.rs` | MODIFY | Task T-09; CE-2 (ADR-056 §D10): extend `validate_sensor_spec` §Category 4 with `PaginationConfig::PageNumber { page_size }` arm rejecting `page_size == 0` with `SpecErrorCode::ESpec001`, message `"page_number pagination in step '{}' requires page_size > 0"` |
| `crates/prism-sensors/specs/cyberint.sensor.toml` | DELETE | AC-001 red gate: load test fails while this file still exists with old shape after ENGINE-001 merges |
| `crates/prism-sensors/specs/cyberint-alerts.sensor.toml` | CREATE | Task T-02; must include `header_scheme`, POST method, `$.alerts` path, `page_number` pagination, `access_token` cred ref |
| `crates/prism-sensors/specs/cyberint-assets.sensor.toml` | CREATE | Task T-03; `header_scheme` + assets OpenAPI-grounded tables; pagination block ABSENT per GAP-ASSETS-PAG-001 — first-page-only retrieval; `total_assets` in response evidences silent truncation; comment required in TOML |
| `crates/prism-dtu-cyberint/src/types.rs` | MODIFY | Task T-04; add `Asset` struct (11 fields, `#[non_exhaustive]`); append `"Asset"` to `EXPECTED_SYMBOLS` in `scripts/check-non-exhaustive-per-symbol.py` ONLY — derived count; no numeric count updates elsewhere |
| `crates/prism-dtu-cyberint/src/clone.rs` | MODIFY | Task T-04/T-08; alert routes gain `/alert` prefix; assets route `POST /asset-configuration/external/api/v1/assets/` registered |
| `crates/prism-dtu-cyberint/src/routes/alerts.rs` | MODIFY | Task T-05; AlertListParams (body-extracted JSON), response shape, page/size pagination; per-record construction uses full Alert struct serialization (not 8-key json! literal) |
| `crates/prism-dtu-cyberint/src/routes/assets.rs` | CREATE | Task T-08; `get_assets()` handler for POST /asset-configuration/external/api/v1/assets/; uses Asset struct from T-04; auth via check_auth(); fixture-backed response |
| `crates/prism-dtu-cyberint/src/routes/mod.rs` | MODIFY | Task T-08; add `pub mod assets;` |
| `crates/prism-dtu-cyberint/tests/ac_6_cursor_pagination.rs` | DELETE | Task T-06 |
| `crates/prism-dtu-cyberint/tests/ac_6_page_size_pagination.rs` | CREATE | Task T-06; AC-004 coverage; uses POST body params not query params |

---

## Verification Properties

| VP | Description | Applicability |
|----|-------------|---------------|
| VP-153 | SensorAuth Runtime Cross-Composition Prevention | Partial — `cookie_roundtrip` + `header_scheme` on new specs must satisfy VP-153 invariant that StaticCookieAuthProvider is selected. Full VP-153 MERGE-GATE run (with token_exchange arms) is in S-ADR054-WAVE-A-001. |

---

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.8 | 2026-07-27 | story-writer | FB72 leg 1 (5 items): (Item 1) add `## Authority` section citing ADR-053 v0.38 §D3-a and ADR-056 v0.4 — closes F-WASE-P66-MED-002 SAC-2 unverified status. (Item 2) RG-018 wire literal corrected: `"pagination_type": "page"` → `"pagination_type": "Page"` (PascalCase — no `rename_all` on `PaginationType`); T-09 step 6 serde description updated to match `"Page"`. (Item 3) T-05 step 2 and AC-003 DTU return value corrected: removed spurious `"page"` key; `GetAlertsResponse` OpenAPI schema has `{total, alerts}` only, no `page` field (verified from `cyberint_alerts_openapi_06.20.2026.json`). (Item 4) T-02 carry-over list: added `affected_assets` column (`column_type = "json"`, `ocsf_field = "raw_extensions.affected_assets"`); added AC-011 and RG-021; Red Gate density count 20→21. (Item 5) T-04 non-exhaustive gate instructions corrected: removed three-site procedure (forbidden by 2026-07-27 CLAUDE.md amendment); single correct action is append `"Asset"` to `EXPECTED_SYMBOLS` in `scripts/check-non-exhaustive-per-symbol.py` ONLY; same correction applied to §File Structure Requirements, T-09 gate note, and §Architecture Compliance Rules point 6. AC count: 10→11. Red Gate count: 20→21. |
| 1.7 | 2026-07-27 | story-writer | FB69 SAP-2 story-side fixes (6 findings): (F-SAP2-CRIT-001 story leg) T-05 item 5: require per-record IOC nested fields via full Alert struct serialization; extend wire-shape assertion to cover static-fixture path; add AC-009 + RG-019. (F-SAP2-CRIT-002 story leg) T-04 expanded: add Asset struct to types.rs (11 fields, i64/Option discriminations, non-exhaustive gate 92→93); T-08 rewritten: remove dead conditional stub branch entirely (CWE-390), real task body with grounding schema Asset, route registration, fixture loading, SAP-2 compliance clause; add §File Structure entries for types.rs / routes/assets.rs / routes/mod.rs; add AC-010 + RG-020. (F-SAP2-HIGH-001) dead conditional in T-08 deleted. (F-SAP2-MED-001) T-04 now creates Asset struct so T-03's SAP-2 dependency anchor on T-04 is correct. (F-SAP2-MED-002) T-08 names Asset as grounding schema and resolves id/discovery_precision type divergences. (F-SAP2-MED-003) T-02 carry-over exclusion clause for cursor-era comments. AC count: 8→10. Red Gate count: 18→20. |
| 1.6 | 2026-07-27 | story-writer | FB67 Obligations 1/3/4: (Ob-1) fix RG-007/RG-008 wire keys from TOML declaration names `page_number`/`page_size` to ADR-056 §D3 canonical keys `"page"`/`"size"`; (Ob-3) add RG-017/RG-018 for CE-2 (`page_size=0` spec-load ESpec001 rejection, ADR-056 §D10) and CE-1 (`PaginationType::Page` wire-shape on serialized `SensorTableDescriptor`, ADR-056 §D10); add CE-1/CE-2 rows to §Architecture Mapping; add `types.rs` and `validation.rs` to §File Structure Requirements and T-09 §Files; fix T-09 dispatch sites description from "five dispatch sites from §Consequences" to "all implementation obligations from ADR-056 §D10 and §Consequences"; add T-09 steps 6 (CE-1: `types.rs` `PaginationType::Page` + `sensor_table_descriptor_from_table_spec` arm) and 7 (CE-2: `validation.rs` `validate_sensor_spec` §Category 4 PageNumber arm with ESpec001); fix T-09 test coverage `page_size=0` bullet from "no injection (activation gate)" to spec-load ESpec001 rejection language per ADR-056 §D10 CE-2; add `PaginationType::Page` wire-shape test bullet. (Ob-4) propagate BC-2.01.018 v1.5→v1.6 and BC-2.16.009 v1.28→v1.29 in frontmatter comment, §Behavioral Contracts table, and §Token Budget. Red Gate count: 16 → 18. AC count: 8 (unchanged). |
| 1.5 | 2026-07-27 | story-writer | FB63 CRIT-002: add BC-2.01.018 v1.5 (Cyberint Alerts contract) to frontmatter `behavioral_contracts:` (5→6 BCs); re-anchor AC-003/AC-004 Alerts-surface traces from BC-2.01.006 to BC-2.01.018; fix BC-2.01.006 pin v1.x → v1.8 (MED-003 / POL-23); update BC-2.06.003 pin v1.3 → v1.12 (POL-23); add BC-2.01.018 row to §Behavioral Contracts table with correct Alerts scope; rewrite frontmatter comment to name both surface contracts; delete discharged §Product-owner dependency gate (split already landed as BC-2.01.018 introduced 2026-07-22 in PO leg of FB63); add BC-2.01.018 v1.5 row to §Token Budget; update Token Budget total ~30,600 → ~31,400 |
| 1.4 | 2026-07-26 | story-writer | FB61 gate-review DEFECT-1: remove fabricated RED_RATIO formula (Density = 16/8 ACs = 2.0) from §Red Gate density check; replace with orchestrator-computation note per per-story-delivery.md §Step 3.5, citing BC-5.38.002/BC-5.38.003 |
| 1.3 | 2026-07-26 | story-writer | FB61 MED-016: add §Red Gate tests with 16 RGTs (RG-001..RG-016) and BC-5.38.001 density check; §Tasks reordered — test-authoring precedes implementation per ENGINE-001 normative pattern |
| 1.2 | 2026-07-26 | story-writer | FB60 MED-008 + LOW-002: pin BC-2.16.001 from `current` to v1.9 and BC-2.16.009 from `current` to v1.28 in §Behavioral Contracts table; widen §Header-Scheme Sweep Report scope to cover `crates/prism-bin/fixtures/sensors/`; add `test-sensor-with-cred-refs.sensor.toml` row (`api_key`, path A, benign — workspace-wide conclusion confirmed) |
| 1.1 | 2026-07-26 | story-writer | FB53c — F-WASE-P64-CRIT-003: change alerts pagination from `offset_limit` to `page_number` per ADR-056; fix AC-003/AC-004 mutual inconsistency (POST body keys, first page = 1); remove T-02 deferral license; MED-011: fix AC-006 task reference T-06 → T-03; MED-014: fix T-03 OpenAPI pointer to `.factory/reference/api-specs/cyberint_assets_openapi_06.20.2026.json`, removed both stub placeholders, converted assets pagination omission to explicit CWE-390-class blocker GAP-ASSETS-PAG-001 (first-page-only silent truncation; `total_assets` evidences loss; blocked on server-controlled-page-size variant awaiting orchestrator story creation; alerts surface unaffected); add T-09 for engine-side `PaginationConfig::PageNumber` wiring; add BC-2.16.002 to behavioral contracts; add SS-07 SpecEngine to subsystems; add prism-spec-engine entries to Architecture Mapping and File Structure |
| 1.0 | 2026-07-25 | story-writer | Initial authoring post-sweep; co-land constraint with ENGINE-001 encoded |
