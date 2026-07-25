---
document_type: story
story_id: S-WAVE-A-CYBERINT-SPEC-001
title: "Cyberint Dual-Surface Spec Migration — Delete cyberint.sensor.toml; Author cyberint-alerts and cyberint-assets; OpenAPI-Ground Alerts C2-Class Fixes; DTU Route Migration"
version: "1.0"
status: draft
producer: story-writer
phase: 3
wave: wave-a
epic_id: E-WAVE-A-SENSOR-REMEDIATION
priority: P0
points: 8
tdd_mode: strict
target_module: prism-sensors
subsystems: ["SS-06 (SensorSpec)", "SS-12 (DTU-Cyberint)"]
depends_on:
  - S-WAVE-A-ENGINE-001    # header_scheme grammar + Rule 9 must be live before new cyberint specs can load;
                           # S-WAVE-A-CYBERINT-PATCH-001 (the minimal co-land patch) co-lands with ENGINE-001,
                           # so when ENGINE-001 is done, the boot-failure hazard is already closed
blocks: []
behavioral_contracts:
  - BC-2.01.006
  - BC-2.06.003
  - BC-2.16.001
  - BC-2.16.009
verification_properties:
  - VP-153
estimated_days: 3
# BC status: BC-2.01.006 v1.x must be amended/split by PO per ADR-053 D5 before this story
# transitions to ready. BC-2.16.001 and BC-2.16.009 are existing contracts. BC-2.06.003 covers
# credential-ref rename. All four must be reviewed against the amended contracts at status-transition time.
assumption_validations: []
risk_mitigations: []
---

# S-WAVE-A-CYBERINT-SPEC-001: Cyberint Dual-Surface Spec Migration

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

A sweep of all spec files in `crates/prism-sensors/specs/` and `.prism/specs/sensors/`
was conducted prior to this story's authoring to identify all files requiring `header_scheme`
migration.

| File | auth_type | header_scheme present? | Action |
|------|-----------|------------------------|--------|
| `crates/prism-sensors/specs/cyberint.sensor.toml` | `cookie_roundtrip` | absent | DELETE — replace with two new specs |
| `crates/prism-sensors/specs/armis.sensor.toml` | `bearer_static` | absent (path A — no field needed) | None |
| `crates/prism-sensors/specs/claroty.sensor.toml` | `bearer_static` | absent (path A — no field needed) | None |
| `crates/prism-sensors/specs/crowdstrike.sensor.toml` | `oauth2_client_credentials` | absent (path A — no field needed) | None |
| `crates/prism-sensors/specs/customers/acme/armis.sensor.toml` | not defined (overlay) | N/A | None |
| `crates/prism-sensors/specs/customers/contoso/armis.sensor.toml` | not defined (overlay) | N/A | None |
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

### AC-003: Alerts surface uses POST /alert/api/v1/alerts with $.alerts response path
(traces to BC-2.01.006 postcondition — Alerts surface returns the correct data at the
correct endpoint with the correct response extraction path)

`cyberint-alerts.sensor.toml` tables.alerts.steps.fetch_alerts declares:
- `method = "POST"`
- `path_template = "/api/v1/alerts"` (relative to base_url which includes `/alert` prefix)
- `response_path = "$.alerts"`
- Pagination type: offset_limit with `page_size = 100`

The DTU `get_alerts` handler (in `routes/alerts.rs`) is updated to:
- Register at POST `/alert/api/v1/alerts` (not GET `/api/v1/alerts`)
- Return `{"alerts": [...], "page": N, "total": N}` (not `{"data": [...], "next_cursor": "..."}`)

A parity test asserts that a POST to `/alert/api/v1/alerts` with valid `access_token` cookie
returns a JSON object with a top-level `"alerts"` key containing an array.

### AC-004: Page/size pagination replaces cursor pagination in DTU
(traces to BC-2.01.006 postcondition — pagination returns complete result sets across
multiple pages)

`ac_6_cursor_pagination.rs` is deleted. A new test `ac_6_page_size_pagination.rs` verifies:
- POST `/alert/api/v1/alerts?page=1&size=10` returns the first 10 alerts
- POST `/alert/api/v1/alerts?page=2&size=10` returns the next 10 alerts (different records)
- POST `/alert/api/v1/alerts?size=1000` is capped to `max_page_size = 100` (DTU enforces cap)
- A multi-page accumulation returns all alerts across pages equal to single `page=1&size=9999` result count

`AlertListParams` in `routes/alerts.rs` is updated from `cursor: Option<String>` to
`page: Option<u32>, size: Option<u32>`.

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
  OpenAPI; see Task T-06 for derivation requirements)

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

---

## Architecture Mapping

| Component | Module | Pure/Effectful | Architecture Section |
|-----------|--------|---------------|----------------------|
| `cyberint-alerts.sensor.toml` | `crates/prism-sensors/specs/` | Pure (config data) | `architecture/module-decomposition.md §SS-06 SensorSpec` |
| `cyberint-assets.sensor.toml` | `crates/prism-sensors/specs/` | Pure (config data) | `architecture/module-decomposition.md §SS-06 SensorSpec` |
| `CyberintClone::build_router()` | `crates/prism-dtu-cyberint/src/clone.rs` | Effectful (HTTP server) | `architecture/module-decomposition.md §SS-12 DTU-Cyberint` |
| `get_alerts()` handler | `crates/prism-dtu-cyberint/src/routes/alerts.rs` | Effectful (HTTP handler) | `architecture/module-decomposition.md §SS-12 DTU-Cyberint` |
| `AlertListParams` | `crates/prism-dtu-cyberint/src/routes/alerts.rs` | Pure (query param struct) | `architecture/module-decomposition.md §SS-12 DTU-Cyberint` |

---

## Behavioral Contracts

| BC | Version | Relevance to This Story |
|----|---------|------------------------|
| BC-2.01.006 | v1.x (see D5 amendment note) | Cyberint sensor behavior — POST method, $.alerts path, page/size pagination |
| BC-2.06.003 | v1.3 | Credential refs resolution chain; `access_token` name change |
| BC-2.16.001 | current | Bundled spec loading at startup — both new specs must pass validation |
| BC-2.16.009 | current | Rule 9: `cookie_roundtrip` requires `header_scheme = "cookie:<name>"` — absence path (c) must NOT trigger |

**Product-owner dependency:** BC-2.01.006 must be amended or split (per ADR-053 §D5
amendment manifest) to reflect the POST method, `$.alerts` response path, and page/size
pagination for the Alerts surface. This story's `status: draft → ready` transition
requires that BC-2.01.006 v2.x (or a new BC-2.01.006a for alerts) is in place and that
`behavioral_contracts:` here references the amended BC version.

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
| EC-004 | POST /alert/api/v1/alerts?size=0 | DTU normalizes to 1 (min page size); returns first result |
| EC-005 | POST /alert/api/v1/alerts?page=9999&size=100 | DTU returns empty `{"alerts": [], "page": 9999, "total": N}` — not an error |
| EC-006 | cyberint-assets.sensor.toml probe_table points to nonexistent table | BC-2.16.001 spec load fails; implementation must derive probe_table from actual tables block |
| EC-007 | Operator uses old CYBERINT_API_KEY env var after migration | Credential resolver falls through to keyring; if not in keyring, sensor returns E-SENSOR-004 (credential not found) — correct fail-open behavior; NOT a silent 401 |

---

## Tasks

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

Alerts fetch step — corrected per ADR-053 C2-class fixes:

```toml
[[tables.steps]]
name = "fetch_alerts"
method = "POST"
path_template = "/api/v1/alerts"
response_path = "$.alerts"
variables_produced = []
[tables.steps.pagination]
type = "offset_limit"
page_size = 100
```

Note: `offset_limit` pagination with `page_size = 100` maps to the Cyberint API's
`page` and `size` query parameters. If the spec engine's `offset_limit` type injects
`offset`/`limit` rather than `page`/`size`, the implementer MUST verify against
`prism-spec-engine/src/pipeline.rs` `build_request()` to confirm the correct pagination
parameter names and either use the matching pagination type or request a spec grammar
extension. Do not assume `offset_limit` matches without verification.

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

Table structure: derive from the Cyberint Assets OpenAPI (located in
`crates/prism-dtu-cyberint/` or the research directory — search for
`cyberint_assets_openapi` or equivalent file). Map each API response field to a column
with the correct `column_type` and an `ocsf_field` reference where an OCSF mapping
exists. Set `probe_table` to the primary table that the LIMIT-0 health probe will use
(BC-2.08.001 postcondition 5).

If the assets OpenAPI file is not present in the codebase, document the table structure
as a stub with explicit `# TBD: requires assets OpenAPI grounding` comments on each
placeholder column, and file a T-TODO note in the story's task tracking for the DTU
validator to complete the OpenAPI grounding before this story's PR merges.

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
   - Accept `page` (1-indexed, default 1) and `size` (default 25, capped at 100)
   - Return `Json(serde_json::json!({ "alerts": [...], "page": page, "total": total_count }))`
   - Remove the cursor-based response shape (`"data": [...], "next_cursor": "..."`)

3. Update the state module if needed: remove any `cursor_store` or cursor-generation logic;
   add page/size slicing logic using `alerts_fixture.chunks(size)[page-1]` (or equivalent).

4. The `access_token` cookie auth and rate-limit logic (`check_auth`, `extract_access_token`)
   are NOT changed by this story.

Wire-shape assertion (CLAUDE.md §Wire-shape assertion discipline): at least one test in
`tests/f_p3_route_output_tests.rs` or a new test file must assert on the serialized
JSON output — the exact top-level keys and array structure as the HTTP client receives it.
Test must verify `"alerts"` key present, `"data"` key absent, `"next_cursor"` key absent.

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

### T-08: Add DTU assets routes (if Assets OpenAPI available)
**Files:** `crates/prism-dtu-cyberint/src/routes/` (CREATE new module if needed)

If the Cyberint Assets OpenAPI is available in the codebase (Task T-03), create a minimal
DTU route for the assets surface under the `/asset-configuration` prefix. The route must
serve a static fixture response in the same shape as the real API to satisfy SAP-2 column
parity validation.

If the Assets OpenAPI is NOT available, add a `routes/assets_stub.rs` module that
returns an empty 200 response (indicating the clone is not yet grounded) and register
it at `/asset-configuration/api/v1/assets` (placeholder path). Mark the stub with a
`// DTU-EXT-CYBERINT-ASSETS-001: placeholder — requires Assets OpenAPI grounding` comment.

---

## Token Budget Estimate

| Context source | Estimated tokens |
|----------------|-----------------|
| This story spec | ~3,500 |
| `cyberint.sensor.toml` (source of alerts column schema) | ~2,500 |
| `crates/prism-dtu-cyberint/src/clone.rs` (route registration) | ~2,000 |
| `crates/prism-dtu-cyberint/src/routes/alerts.rs` (handler + AlertListParams) | ~3,000 |
| `crates/prism-dtu-cyberint/tests/ac_6_cursor_pagination.rs` (to delete) | ~800 |
| `crates/prism-sensors/specs/armis.sensor.toml` (pagination grammar reference) | ~1,000 |
| `crates/prism-sensors/specs/claroty.sensor.toml` (POST pagination reference) | ~1,500 |
| ADR-053 §D3-a and §C2-class fixes | ~1,000 |
| BC-2.16.009 Rule 9 (header_scheme validation) | ~800 |
| BC-2.06.003 (credential refs) | ~500 |
| Running test output (nextest per-crate) | ~2,000 |
| **Total estimate** | **~18,600** |

18,600 tokens is within the 20–30% context window limit for a standard 100k-token agent
context. No story split required.

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
   `prism-dtu-cyberint` by T-08, update `scripts/check-non-exhaustive.sh` and
   `scripts/check-non-exhaustive-per-symbol.py` before the PR merges.

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
| `crates/prism-sensors/specs/cyberint.sensor.toml` | DELETE | AC-001 red gate: load test fails while this file still exists with old shape after ENGINE-001 merges |
| `crates/prism-sensors/specs/cyberint-alerts.sensor.toml` | CREATE | Task T-02; must include `header_scheme`, POST method, `$.alerts` path, `access_token` cred ref |
| `crates/prism-sensors/specs/cyberint-assets.sensor.toml` | CREATE | Task T-03; `header_scheme` + assets OpenAPI-grounded tables (or stub with TBD comment) |
| `crates/prism-dtu-cyberint/src/clone.rs` | MODIFY | Task T-04; route paths gain `/alert` prefix |
| `crates/prism-dtu-cyberint/src/routes/alerts.rs` | MODIFY | Task T-05; AlertListParams, response shape, pagination |
| `crates/prism-dtu-cyberint/tests/ac_6_cursor_pagination.rs` | DELETE | Task T-06 |
| `crates/prism-dtu-cyberint/tests/ac_6_page_size_pagination.rs` | CREATE | Task T-06; AC-004 coverage |

---

## Verification Properties

| VP | Description | Applicability |
|----|-------------|---------------|
| VP-153 | SensorAuth Runtime Cross-Composition Prevention | Partial — `cookie_roundtrip` + `header_scheme` on new specs must satisfy VP-153 invariant that StaticCookieAuthProvider is selected. Full VP-153 MERGE-GATE run (with token_exchange arms) is in S-ADR054-WAVE-A-001. |

---

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.0 | 2026-07-25 | story-writer | Initial authoring post-sweep; co-land constraint with ENGINE-001 encoded |
