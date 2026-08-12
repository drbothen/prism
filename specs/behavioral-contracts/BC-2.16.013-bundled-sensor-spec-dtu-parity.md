---
document_type: behavioral-contract
level: L3
version: "1.36"
status: active
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
subsystem: "SS-16"
capability: "CAP-029"
lifecycle_status: active
introduced: "2026-05-20"
modified: "2026-08-11"  # v1.36: device_alert_relations table added to Claroty contracted surface (§Postconditions §1); DTU-EXT-006 gap registered; EC-016-013-009 added; SAP-2 exclusion-documentation for 82 omitted fields (10 contracted, 82 excluded, 92 total); harness envelope shape extended; harness-story S-DEMO-CLAROTY-HARNESS-DAR-001 anchored
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
  - ".factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md"
  - ".factory/specs/architecture/decisions/ADR-028-toml-spec-grounding-vs-dtu-routes.md"
  - ".factory/specs/test-strategy/TS-PLUGIN-PARITY-001-dtu-canonicalization.md"
  - "crates/prism-dtu-crowdstrike/src/routes/mod.rs"
  - "crates/prism-dtu-claroty/src/clone.rs"
  - "crates/prism-dtu-cyberint/src/clone.rs"
  - "crates/prism-dtu-cyberint/src/routes/alerts.rs"
  - "crates/prism-dtu-armis/src/clone.rs"
  - "crates/prism-dtu-armis/src/lib.rs"
input-hash: "36f421b"
traces_to:
  - "CAP-029"
extracted_from: ".factory/specs/prd.md"
---

# BC-2.16.013: Bundled Sensor Spec Authoring and DTU-Parity Verification — 4 Initial Sensors

## Description

The four initial sensors (CrowdStrike, Cyberint, Claroty, Armis) ship as production TOML
spec files bundled at `crates/prism-sensors/specs/` within the prism repository. These files
are reverse-engineered from the existing hardcoded Rust adapter implementations to preserve
exact behavioral parity. Each spec is validated by the existing pipeline (BC-2.16.001,
BC-2.16.009, BC-2.16.002) and paired with a DTU-parity integration test that proves:
spec-driven dispatch against the corresponding DTU clone produces OCSF-normalized output
that is semantically equivalent — per TS-PLUGIN-PARITY-001 Rules A–I — to the reference
output produced by the prior hardcoded Rust adapter path against the same DTU clone for
the same raw API response payload.

This BC is the behavioral anchor for VP-PLUGIN-003 (VP-148) and the correctness gate for
PLUGIN-MIGRATION-001-A (deletion of the 4 hardcoded Rust adapter modules). PLUGIN-MIGRATION-001-A
MUST NOT proceed until VP-PLUGIN-003 is verified (green parity tests) for all 4 sensors.

## Preconditions

- S-PLUGIN-PREREQ-A through S-PLUGIN-PREREQ-E have all merged to develop: `SensorId` newtype,
  `PipelineExecutor` with `AuthProvider`, TOML grammar (`spec_parser.rs` full implementation),
  `PluginRuntime` boot wiring, and `SensorAuth` open trait / `WriteToolInvalidationMap` runtime
  extensibility are all in production code.
- `PluginRegistry` dispatch is wired in `spec_parser.rs` (BC-2.16.012 active).
- The `CustomAdapter` Rust trait has been removed (BC-2.16.011 active; `lifecycle_status: removed`).

### O-001 TOML Grammar Verification — LOCKED Option A (D-FB-IMPL-1-OPT-A, 2026-05-21)

The following four TOML grammar features were verified against the canonical implementation at
`crates/prism-spec-engine/src/spec_parser.rs` and `crates/prism-spec-engine/src/pipeline.rs`:

| Field | Status | Evidence |
|-------|--------|---------|
| `fan_out_batch_size` on `FetchStep` | **SUPPORTED** | Present as `pub fan_out_batch_size: Option<u32>` in the `FetchStep` struct (`FetchStep::fan_out_batch_size` field); handled by `fan_out_batches` in the pipeline executor |
| `${query.filter.KEY}` interpolation | **SUPPORTED** | `FetchContext::query_filters: HashMap<String, String>` seeded into `step_vars` via `PipelineExecutor::execute_impl` `query.filter.{k}` step-vars seeding; Armis AQL must use `${query.filter.aql}`, not the non-existent `${query.aql}` shorthand |
| `timestamp_formats: Vec<String>` | **IN-SCOPE EXTENSION — Option A LOCKED** | Not yet in `spec_parser.rs`. Must be added to `ColumnSpec` with `#[serde(default)]` as part of PLUGIN-MIGRATION-001-D per ADR-028 v1.10 §D8-C. See implementer contract below. |
| `timestamp_fallback_chain: Vec<String>` | **IN-SCOPE EXTENSION — Option A LOCKED** | Not yet in `spec_parser.rs`. Must be added to `ColumnSpec` with `#[serde(default)]` as part of PLUGIN-MIGRATION-001-D per ADR-028 v1.10 §D8-C. See implementer contract below. |

**O-001 LOCKED Option A — see ADR-028 v1.10 §D8.** The WASM transformer plugin path (Option B)
is NOT in scope for PLUGIN-MIGRATION-001-D. The grammar extension (Option A) is in scope and
MUST be implemented by the implementer in the same story. This is not a deferral.

**Implementer contract (complete; no further adjudication needed):**

- Add `timestamp_formats: Vec<String>` and `timestamp_fallback_chain: Vec<String>` to `ColumnSpec`
  in `crates/prism-spec-engine/src/spec_parser.rs`, both with `#[serde(default)]`.
- Both fields default to empty `Vec` — backward compatible: existing TOML specs are unaffected.
- `timestamp_formats` recognized values: `"iso8601"`, `"unix_epoch_seconds"`, `"unix_epoch_millis"`.
  Unrecognized format names → `E-SPEC-001` validation error at load time (BC-2.16.009 gate).
- `timestamp_fallback_chain` lists source field names to try when the primary column field is null.
  After all chain fields are exhausted: use `DateTime::now()` UTC; emit
  `tracing::warn!(event_type = "timestamp.fallback_to_now", column = %col_name)`.
- On multi-format parse failure (all `timestamp_formats` tried, none succeeded): emit `E-SPEC-018`
  (`TimestampParseFailure`) — registered in error-taxonomy.md by this fix-burst.
- **Null-primary passthrough (HIGH-006 adjudication, FB-IMPL-2 PO, Option a):** When a
  `ColumnType::Datetime` column's primary value is null/absent AND `timestamp_fallback_chain`
  is empty (the default), the field passes through to Arrow output as null with no audit signal
  emitted. This is the legitimate path for sensors where null timestamps represent valid data
  (e.g., Cyberint `created_at: null` for alerts in draft/pending state; DTU `Alert.created_at`
  is `serde_json::Value` which accepts JSON `null`). Null is NOT an error in this case — it is
  valid sensor data. Sensors that REQUIRE non-null timestamps MUST declare a non-empty
  `timestamp_fallback_chain`; the chain-exhaustion `DateTime::now()` UTC fallback (with WARN
  emission) handles the "never allow null" contract. The E-SPEC-018 parse-failure path applies
  only when `timestamp_formats` is non-empty and all named formats fail to parse a non-null value.
- Normalization runs inside `PipelineExecutor` during response-to-Arrow materialization for
  `ColumnType::Datetime` columns.

**Cyberint `created_at` canonical formats (DTU-grounded per ADR-028 v1.10 §D8-A):**
`timestamp_formats = ["iso8601", "unix_epoch_seconds"]`. Cyberint DTU `Alert.created_at` is
`serde_json::Value` (accepts ISO 8601, epoch seconds, or epoch milliseconds).

**Armis timestamp fallback chain (DTU-grounded per ADR-028 v1.10 §D8-B, amended FB-IMPL-2):**
`timestamp_fallback_chain = ["first_seen"]` on the primary `last_seen` timestamp column.
DTU `DeviceRecord` has `last_seen: Option<String>` (primary) and `first_seen: Option<String>`
(secondary). Fixture `d-001` has `last_seen: null` + `first_seen: "2024-01-15T10:00:00Z"` to
exercise the fallback path. WARN emission when falling back to `now()` preserves the existing
audit signal. The prior v1.12 chain `["last_seen", "first_seen"]` is corrected: listing the
primary column name as the first fallback element is a semantic no-op (fallback chain only
executes when the primary is already null/absent). See ADR-028 v1.10 §D8-B for full rationale.

- DTU clones for all 4 sensors are built and available in the test harness:
  - `prism-dtu-crowdstrike` (S-6.07): OAuth2 token endpoint + two-step Falcon API (QueryV2 + PostEntities)
  - `prism-dtu-claroty` (S-6.08): Bearer token auth + POST-for-read + offset pagination
  - `prism-dtu-cyberint` (S-6.09): Cookie-roundtrip auth + multi-format timestamp responses
  - `prism-dtu-armis` (S-6.10): Bearer + AQL query forwarding + timestamp fallback chain
  Note: if DTU clones are not yet built, the parity tests are in SKIP status per TS-PLUGIN-PARITY-001
  Rule H (SKIP condition) — this BC still governs the spec authoring obligation.
- Fixture payloads (real-sensor recordings or synthesized per TS-PLUGIN-PARITY-001 Rule I) exist
  at `crates/prism-dtu-{sensor}/fixtures/parity/` with minimum 3 real-sensor recordings AND
  3 synthesized cases per `(sensor_id, table)` pair.

## Postconditions

### 1. Spec Files Authored and Validated

Four production TOML sensor spec files are created at `crates/prism-sensors/specs/`.

**Grounding authority (per ADR-028 §D1):** All URL paths are derived from DTU clone route
registrations, not from the legacy Rust adapter code. The legacy adapters have simplified URL
paths that do not match the real third-party APIs; they are deleted by PLUGIN-MIGRATION-001-A.
**Grounding authority (per ADR-028 §D2):** All `auth_type` values are derived from DTU clone
authentication enforcement behavior, which reflects the real third-party API's auth contract.
**[SUPERSEDED-PENDING for Armis and Cyberint — ADR-053 §D1, effective 2026-07-22]:** The
ADR-028 §D2 auth-grounding authority is superseded by ADR-053 §D1 for Armis and Cyberint; see
ADR-053 §D2 (Armis auth supersession) and ADR-053 §D3-a (Cyberint dual-surface supersession)
for the pending decisions. This statement remains the authority for CrowdStrike and Claroty.
Amendment of the Armis and Cyberint sensor entries below is owned by `S-WAVE-A-CYBERINT-SPEC-001`
per ADR-053 §D5.

- `crowdstrike.sensor.toml` — `sensor_id: "crowdstrike"`, `auth_type: "oauth2_client_credentials"`,
  `base_url = "${env.CROWDSTRIKE_BASE_URL}"` (S-DEMO-CROWDSTRIKE-MULTIREGION-001; replaces hardcoded `https://api.crowdstrike.com` us-1 URL; operator sets `CROWDSTRIKE_BASE_URL` to the tenant's region URL — us-1: `https://api.crowdstrike.com`, us-2: `https://api.us-2.crowdstrike.com`, eu-1: `https://api.eu-1.crowdstrike.com`, gov: `https://api.laggar.gcw.crowdstrike.com`). Missing/empty `CROWDSTRIKE_BASE_URL` → E-SPEC-024 at spec-load time (BC-2.16.009 §Validation Rules 6). Tables:
  - `detections` — QueryV2 step (GET `/detects/queries/detects/v1`) → PostEntities step
    (POST `/detects/entities/summaries/GET/v1`) with batch size ≤ 100 (CROWDSTRIKE_BATCH_SIZE).
    URL grounded: `crates/prism-dtu-crowdstrike/src/routes/mod.rs` route registrations
    (lines 189, 193 per pass-4 adversarial ground truth; exact anchor:
    `"/detects/queries/detects/v1"` and `"/detects/entities/summaries/GET/v1"`).
  - `devices` — QueryV2 step (GET `/devices/queries/devices/v1`) → PostDeviceDetailsV2 step
    (POST `/devices/entities/devices/v2` with body `{"ids": [...]}` per DEFECT-CSDEVICES-EMPTY-PIPELINE-001
    architect ratification 2026-07-10; matches real CrowdStrike `PostDeviceDetailsV2` operation,
    FalconPy v1.2.0+, body `{"ids": ["AID1", "AID2", ...]}` — identical structure to the
    detections PostEntities step; supports up to 5000 IDs vs GET variant's 100).
    URL grounded: `crates/prism-dtu-crowdstrike/src/routes/mod.rs` route registrations
    (`"/devices/queries/devices/v1"` and `"/devices/entities/devices/v2"` — the latter registers
    both GET `get_host_details` (existing; preserved for backward compat) and POST
    `post_host_details` (new handler per DEFECT-CSDEVICES-EMPTY-PIPELINE-001; endpoint count
    8→9: 5 read, 4 write); the spec-driven `fetch_devices` pipeline path is POST).
    **Harness parity (INV-HARNESS-ROUTE-PARITY):** Both `prism-dtu-harness` CrowdStrike router
    builders (in-process and network-mode) MUST register GET `get_host_details` AND POST
    `post_host_details` on `/devices/entities/devices/v2`, mirroring the standalone's shared
    route composition: session-registry filter → org-id guard → containment merge → auth; POST
    handler enforces the same empty-ids 400 guard as the standalone. This parity obligation
    closes F-CSD-P9-001 (v1.26 documented the standalone 9-endpoint surface but omitted the
    corresponding harness-clone update required by this invariant).
  - `incidents` — **See §Known Gaps: DTU-EXT-001 (RETIRED).** Incidents table retired per
    D-1889; CrowdStrike Incidents API removed ~2026-03. Incidents are derived from Alerts via
    `aggregate_id` per S-CROWDSTRIKE-INCIDENTS-RETIREMENT-001.
  Each table's columns match the Arrow schema produced by the prior Rust adapter (BC-2.01.005
  field enumeration); OCSF field mappings reproduce the prior `CrowdStrikeAdapter::fetch()`
  (`SensorAdapter::fetch` trait method) normalization. Version: `"1.0.0"`. Rate limit hints: `requests_per_second: 10.0`.
  Auth grounded: CrowdStrike DTU enforces OAuth2 token endpoint (`/oauth2/token` in
  `crates/prism-dtu-crowdstrike/src/routes/mod.rs`) → `auth_type = "oauth2_client_credentials"`.

- `claroty.sensor.toml` — `sensor_id: "claroty"`, `auth_type: "bearer_static"`,
  base URL from instance_url, tables:
  - `devices` — POST `/api/v1/devices/` (trailing-slash form; S-DEMO-CLAROTY-TRAILING-SLASH-001)
    with offset pagination. The `prism-dtu-claroty` router includes `normalize_path`
    middleware (ADR-031 §D8-b) so that both `/api/v1/devices` and `/api/v1/devices/` are
    accepted. `claroty.sensor.toml` `path_template` uses the trailing-slash form.
    URL grounded: `crates/prism-dtu-claroty/src/clone.rs` `build_router()` route
    registration for `/api/v1/devices`.
  - `assets` — **See §Known Gaps: DTU-EXT-002.** DTU has `/api/v1/devices`, not `/api/v1/assets`.
    This table entry is deferred until the DTU clone is extended (ADR-028 §D5).
  - `alerts` — POST `/api/v1/alerts/` (trailing-slash form; S-DEMO-CLAROTY-TRAILING-SLASH-001)
    with offset pagination. The `prism-dtu-claroty` router includes `normalize_path`
    middleware (ADR-031 §D8-b) so that both `/api/v1/alerts` and `/api/v1/alerts/` are
    accepted. `claroty.sensor.toml` `path_template` uses the trailing-slash form.
    URL grounded: `crates/prism-dtu-claroty/src/clone.rs` `build_router()` route
    registration for `/api/v1/alerts`.
  - `audit_logs` — POST `/api/v1/audit_log/get`; DTU route registered by
    S-DEMO-CLAROTY-AUDIT-DTU-001 (Gap-CL-006 CLOSED). Method is POST-for-read,
    consistent with Claroty xDome API pattern. NO `/xdome` prefix — the production
    Claroty xDome API does not use that path prefix.
    **Trailing-slash form (S-DEMO-CLAROTY-TRAILING-SLASH-001):** The canonical
    `path_template` in `claroty.sensor.toml` is `/api/v1/audit_log/get/` (with trailing
    slash), matching the real Claroty xDome API. The `prism-dtu-claroty` router MUST
    include `normalize_path` middleware (or equivalent) so that both
    `/api/v1/audit_log/get` and `/api/v1/audit_log/get/` are accepted (ADR-031 §D8-b).
  - `device_alert_relations` — POST `/api/v1/device_alert_relations/` (trailing-slash form
    per ADR-031 §D8-b / S-DEMO-CLAROTY-TRAILING-SLASH-001 convention). Method is POST-for-read,
    consistent with the Claroty xDome API pattern. The `fields` array is **required** in the
    request body (`GetDeviceAlertsParameters.fields`, `required: true`, `minItems: 1`);
    this distinguishes it from `audit_log/get/`, whose `GetAuditLogParameters` has no `fields`
    property at all. Pagination is offset/limit; API maximum `limit: 5000`. Response top-level
    key is `devices_alerts` (not the path stem `device_alert_relations` — verified against
    `GetDeviceAlertsResponse.properties` in the authoritative xDome OpenAPI). The
    `prism-dtu-claroty` router MUST include `normalize_path` middleware (ADR-031 §D8-b) so both
    `/api/v1/device_alert_relations` and `/api/v1/device_alert_relations/` are accepted.
    **Contracted column subset (deliberate partial projection of the 92-field enum):**
    The xDome `AlertedDevicesPairs__fields_enum` exposes 92 fields. The contracted subset for
    `claroty.sensor.toml` `device_alert_relations` is exactly 10 columns: `device_uid`,
    `alert_id`, `device_alert_detected_time`, `device_risk_score`,
    `network_signature_severity`, `network_signature_confidence`, `malicious_ip_severity`,
    `alert_note`, `external_ip`, `device_alert_status`. All 10 are confirmed present in the
    92-field enum (verified against the authoritative xDome OpenAPI).
    The remaining 82 fields are deliberately excluded.
    **SAP-2 exclusion documentation (SAP-2 rule 4):** Future SAP-2 passes MUST NOT mint
    "field in API with no TOML column" findings for the 82 excluded fields — the deliberate
    exclusion is documented here. Any story that adds columns to this table MUST amend this BC
    and update this exclusion count before the implementing PR merges.
    **Table rationale:** The xDome `alerts` surface (the 20-value `Alert__fields_enum`, verified
    against the authoritative OpenAPI) carries no `severity` field. Risk and severity signal for
    the Claroty sensor surface resides exclusively on `device_alert_relations` rows, via
    `device_risk_score`, `network_signature_severity`, `network_signature_confidence`, and
    `malicious_ip_severity`. This table is therefore both the alert→device investigation path
    and the sole prioritization source for the Claroty sensor surface.
    **URL grounding (ADR-028 §D1) — pending story merge:** `crates/prism-dtu-claroty/src/clone.rs`
    `build_router()` to register `POST /api/v1/device_alert_relations/`. Route does not yet
    exist on develop as of 2026-08-11 (see §Known Gaps DTU-EXT-006). Gap closes on merge of
    S-DEMO-CLAROTY-DAR-001 (`status: draft`, wave 5) per its AC-006. The corresponding
    `prism-dtu-harness::clones::claroty::router()` route parity obligation is tracked by
    S-DEMO-CLAROTY-DAR-001 AC-007 but is NOT in scope for S-DEMO-CLAROTY-DAR-001 — harness
    parity is delivered by **S-DEMO-CLAROTY-HARNESS-DAR-001** (`status: draft`, wave 5;
    depends on S-DEMO-CLAROTY-DAR-001 merge); see §Invariants INV-HARNESS-ROUTE-PARITY
    for the anchored MUST.
  Polymorphic ID handling: `ClarotyId` (int or UUID string) expressed as column type `string`
  with OCSF `raw_extensions` passthrough. Version: `"1.0.0"`.
  Auth grounded: Claroty DTU (`crates/prism-dtu-claroty/src/routes/devices.rs` and
  `routes/alerts.rs`) enforces `Authorization: Bearer {non-empty}` header →
  `auth_type = "bearer_static"`. (The legacy `ClarotyAuth::auth_type_name()` incorrectly
  returned `"cookie_roundtrip"` — this is a latent label bug deleted by PLUGIN-MIGRATION-001-A.
  Per ADR-028 §D2 supersession of ADR-026 §D3 (D-747), this TOML value diverges from the live
  `ClarotyAuth::auth_type_name()` return until PLUGIN-MIGRATION-001-A migrates the code per
  ADR-028 §D6 scope.)

- `cyberint.sensor.toml` — `sensor_id: "cyberint"`, `auth_type: "cookie_roundtrip"`,
  base URL from environment (`https://{environment}.cyberint.io`), tables:
  - `alerts` — GET `/api/v1/alerts` with cursor pagination.
    URL grounded: `crates/prism-dtu-cyberint/src/clone.rs` `build_router()` (line 115:
    `"/api/v1/alerts"` registered as GET route).
  - `incidents` — Cyberint DTU gap: parity tests in SKIP per TS-PLUGIN-PARITY-001 Cyberint
    DTU Gap Note until DTU coverage of `incidents` pagination behavior is verified.
  Multi-format timestamp parsing is expressed via column `type: "datetime"` with
  `timestamp_formats = ["iso8601", "unix_epoch_seconds"]` on the `created_at` column
  (O-001 LOCKED Option A per ADR-028 v1.10 §D8-A; grammar extension in `ColumnSpec` implemented
  by this story's implementer). Version: `"1.0.0"`.
  Auth grounded: Cyberint DTU (`crates/prism-dtu-cyberint/src/routes/alerts.rs::extract_session_token()`)
  enforces cookie-based session auth — extracts `cyberint_session` cookie from `Cookie` header
  → `auth_type = "cookie_roundtrip"`. (The legacy `CyberintAuth::auth_type_name()` incorrectly
  returned `"bearer_static"` — this is a latent label bug deleted by PLUGIN-MIGRATION-001-A.
  Per ADR-028 §D2 supersession of ADR-026 §D3 (D-747), this TOML value diverges from the live
  `CyberintAuth::auth_type_name()` return until PLUGIN-MIGRATION-001-A migrates the code per
  ADR-028 §D6 scope.)
  **[SUPERSEDED-PENDING — ADR-053 §D3-a, effective 2026-07-22]:** The single-surface
  `cyberint.sensor.toml` entry and its ADR-028 §D2 grounding are superseded; see ADR-053 §D3-a
  for the pending dual-surface split decision. This entry reflects current `develop` state.
  Amendment owned by `S-WAVE-A-CYBERINT-SPEC-001` per ADR-053 §D5.

- `armis.sensor.toml` — `sensor_id: "armis"`, `auth_type: "bearer_static"`,
  base URL from instance_url, tables:
  - `devices` — **DTU-EXT-003: implementation COMPLETE on `feature/S-DEMO-ARMIS-AQL-001`
    (`crates/prism-dtu-armis/src/clone.rs` `build_router()` registers `GET /api/v1/search`
    per ADR-031 §D8-a); gap CLOSES on merge of S-DEMO-ARMIS-AQL-001 (status: in-progress)
    to develop. Until then the gap remains OPEN on develop.** The devices table uses
    `GET /api/v1/search` with AQL discriminator `in:devices` forwarded via
    `${query.filter.aql}`.
  - `alerts` — **DTU-EXT-004: implementation COMPLETE on `feature/S-DEMO-ARMIS-AQL-001`
    (`crates/prism-dtu-armis/src/clone.rs` `build_router()` registers `GET /api/v1/search`
    per ADR-031 §D8-a); gap CLOSES on merge of S-DEMO-ARMIS-AQL-001 (status: in-progress)
    to develop. Until then the gap remains OPEN on develop.** The alerts table uses
    `GET /api/v1/search` with AQL discriminator `in:alerts` forwarded via
    `${query.filter.aql}`.
  Timestamp fallback chain: `last_seen` → `first_seen` → `DateTime::now()` (UTC) expressed via
  `timestamp_fallback_chain = ["first_seen"]` on the primary `last_seen` timestamp column
  (O-001 LOCKED Option A per ADR-028 v1.10 §D8-B amended; `ColumnSpec::timestamp_fallback_chain`
  field implemented by this story's implementer). WARN emission when falling back to `now()`
  preserves the existing `tracing::warn!(event_type = "timestamp.fallback_to_now")` audit signal.
  Version: `"1.0.0"`.
  Auth grounded: Armis DTU (per `crates/prism-dtu-armis/src/lib.rs` module documentation) enforces
  `Authorization: Bearer {non-empty}` header with HTTP 403 on missing/invalid token
  (Armis Centrix API spec behavior) → `auth_type = "bearer_static"`. (The legacy
  `ArmisAuth::auth_type_name()` returned `"api_key"` — per ADR-028 §D2 supersession of ADR-026 §D3
  (D-747), this TOML value diverges from the live `ArmisAuth::auth_type_name()` return until
  PLUGIN-MIGRATION-001-A migrates the code per ADR-028 §D6 scope.)
  **[SUPERSEDED-PENDING — ADR-053 §D2, effective 2026-07-22]:** The DTU-based grounding in
  ADR-028 §D2 for the Armis auth model is superseded; see ADR-053 §D2 for the pending decision.
  This entry reflects current `develop` state. Amendment owned by `S-WAVE-A-CYBERINT-SPEC-001`
  per ADR-053 §D5.

### Known Gaps (DTU Extension Required — ADR-028 §D5)

Per ADR-028 §D5, a TOML spec entry for a URL path that has no corresponding DTU route
registration is an architectural violation. The following gaps are identified, cataloged,
and surfaced to the orchestrator for follow-up story creation. They are NOT blockers for
PLUGIN-MIGRATION-001-D cascade convergence (pass-5 will independently verify status).

| Gap ID | Sensor | Table | BC Entry | DTU Status | Recommended Resolution |
|--------|--------|-------|----------|------------|----------------------|
| DTU-EXT-001 | ~~CrowdStrike~~ | ~~`incidents`~~ | **RETIRED (2026-07-22, D-1889)** — No DTU route was registered | ~~No incidents route in `prism-dtu-crowdstrike/src/routes/mod.rs`~~ CrowdStrike Incidents API removed ~2026-03; incidents table retired; incidents derived from Alerts via `aggregate_id`. | ~~Extend `prism-dtu-crowdstrike` with incidents routes.~~ Retired per D-1889 (incidents=retire+derive). S-DTU-CROWDSTRIKE-INCIDENTS-ROUTE-001 RETIRED. Superseded by S-CROWDSTRIKE-INCIDENTS-RETIREMENT-001. |
| DTU-EXT-002 | Claroty | `assets` | DTU has `/api/v1/devices`; BC had `/api/v1/assets` | `prism-dtu-claroty/src/clone.rs` line 85: `/api/v1/devices` registered | Extend `prism-dtu-claroty` with `/api/v1/assets` route OR reconcile that Claroty "assets" table maps to `/api/v1/devices` (table name vs endpoint may differ per xDome API) |
| DTU-EXT-003 | Armis | `devices` | ~~DTU had `/api/v1/devices` (GET); BC had `/api/v1/search` w/ AQL~~ **Implementation COMPLETE on `feature/S-DEMO-ARMIS-AQL-001`; OPEN on develop until story merges** | `prism-dtu-armis/src/clone.rs` `build_router()` registers `GET /api/v1/search` (AQL-search endpoint) per ADR-031 §D8-a on feature branch. Not yet on develop (S-DEMO-ARMIS-AQL-001 status: in-progress). | Gap closes automatically when S-DEMO-ARMIS-AQL-001 merges to develop. No separate DTU extension story needed. |
| DTU-EXT-004 | Armis | `alerts` | ~~DTU had `/api/v1/alerts` (GET); BC had `/api/v1/search` w/ AQL~~ **Implementation COMPLETE on `feature/S-DEMO-ARMIS-AQL-001`; OPEN on develop until story merges** | `prism-dtu-armis/src/clone.rs` `build_router()` registers `GET /api/v1/search` (AQL-search endpoint) per ADR-031 §D8-a on feature branch. Not yet on develop (S-DEMO-ARMIS-AQL-001 status: in-progress). | Gap closes automatically when S-DEMO-ARMIS-AQL-001 merges to develop. No separate DTU extension story needed. |
| DTU-EXT-005 | ~~Cyberint~~ | ~~`alerts` pagination `page_size`~~ | **RETIRED (2026-07-22, D-1889)** — `AlertListParams` struct (in `crates/prism-dtu-cyberint/src/routes/alerts.rs`) had no `page_size` field; `page_size = 100` in cyberint.sensor.toml was unvalidated by DTU | ~~Confirmed: struct has only `cursor: Option<String>` field. `page_size` REMOVED from TOML per ADR-028 §D9 (FB-IMPL-2).~~ Retired per D-1889; Cyberint alerts endpoint is `POST /alert/api/v1/alerts` with `page/size` pagination per ADR-053 §Finding-1. | ~~Extend `AlertListParams` + DTU handler to accept `page_size`, then restore the TOML field.~~ Retired per D-1889 (Cyberint=retire+reclone). DEFECT-CYBERINT-SPEC-FIDELITY-001 supersedes. S-DEMO-CYBERINT-INCIDENTS-SEEDING-001 also RETIRED. |
| DTU-EXT-006 | Claroty | `device_alert_relations` | BC entry added 2026-08-11; `crates/prism-dtu-claroty/src/clone.rs` `build_router()` does not yet register `POST /api/v1/device_alert_relations/` on develop as of 2026-08-11 | No DTU route for `/api/v1/device_alert_relations/` in `crates/prism-dtu-claroty/src/clone.rs` `build_router()` | **S-DEMO-CLAROTY-DAR-001** (`status: draft`, wave 5) — adds DTU route (`POST /api/v1/device_alert_relations/`), TOML `device_alert_relations` table spec, and TS-PLUGIN-PARITY-001 parity tests in the same PR (AC-001 through AC-006). Gap closes on merge of S-DEMO-CLAROTY-DAR-001 to develop per AC-006. Harness route parity (`prism-dtu-harness::clones::claroty::router()` and `network_router()` registering `POST /api/v1/device_alert_relations/`) is delivered by **S-DEMO-CLAROTY-HARNESS-DAR-001** (`status: draft`, wave 5; depends on S-DEMO-CLAROTY-DAR-001 merge per its `depends_on` constraint; anchored MUST now in §Invariants INV-HARNESS-ROUTE-PARITY covering both routers). |

All four specs pass BC-2.16.009 validation (no schema errors, no variable reference errors)
and are loaded by BC-2.16.001 at startup when `sensor_specs_dir` includes `crates/prism-sensors/specs/`.

### 2. DTU-Parity Tests Pass (VP-PLUGIN-003)

For each `(sensor_id, table)` pair with non-SKIP status:

- A parity integration test in `crates/prism-spec-engine/tests/` or
  `crates/prism-sensors/tests/parity/` exercises the spec-driven path against the DTU clone:
  1. Start DTU clone server by constructing the clone struct and calling
     `BehavioralClone::start_on(bind, shutdown, tls)` (from `prism_dtu_common::BehavioralClone`
     trait, implemented by `CrowdstrikeClone`, `ClarotyClone`, `CyberintClone`, `ArmisClone`):
     ```rust
     // Signature (all 4 clones — identical via BehavioralClone trait):
     async fn start_on(
         &mut self,
         bind: SocketAddr,                              // typically "127.0.0.1:0" for ephemeral
         shutdown: Option<broadcast::Receiver<()>>,
         #[cfg(feature = "tls")] tls: Option<Arc<axum_server::tls_rustls::RustlsConfig>>,
         #[cfg(not(feature = "tls"))] tls: Option<()>,
     ) -> anyhow::Result<SocketAddr>
     ```
     The returned `SocketAddr` is used to construct the test-override base URL.
  2. Load the bundled TOML spec via `SpecLoader::parse(toml_input: &str)` (spec_parser.rs::SpecLoader::parse)
     — read the spec file content to a string, then parse via `SpecLoader::parse(&content)`;
     override the spec's `base_url` field to the DTU `SocketAddr` via test-only config injection
  3. Execute `PipelineExecutor::execute()` with a `NullAuthProvider` (DTU does not validate tokens)
     or the DTU's mock auth provider:
     ```rust
     // Actual signature (crates/prism-spec-engine/src/pipeline.rs):
     pub async fn execute(
         spec: &SensorSpec,
         table: &TableSpec,
         context: &FetchContext,
         http_client: &reqwest::Client,
         auth_provider: &dyn AuthProvider,
     ) -> Result<PipelineResult, SpecEngineError>
     ```
  4. Load reference OCSF output from committed fixture JSON at
     `crates/prism-dtu-{sensor}/fixtures/parity/reference-ocsf/<table>.json`
     (e.g., `crates/prism-dtu-crowdstrike/fixtures/parity/reference-ocsf/detections.json`).
     Parse via `serde_json::from_str::<serde_json::Value>(&content)`.
     **Fixture provenance:** recorded once by running the legacy adapter against the DTU clone
     before PLUGIN-MIGRATION-001-A deletes the adapter — captures real-API-shaped responses,
     not adapter-bug-simplified responses (per ADR-028 §D3). After 001-A deletes the adapters,
     the committed fixture JSON is the permanent parity reference. Fixtures are NEVER
     regenerated automatically at test runtime.
     **No `prism-sensors` dev-dep on `prism-spec-engine`** — the fixture mechanism eliminates
     any need to call `CrowdStrikeAdapter::fetch()` (etc.) from `prism-spec-engine` test code.
     Story §Forbidden Dependencies (blocking `prism-sensors` as a `prism-spec-engine` dependency)
     remains intact (per ADR-028 §D3).
  5. Canonicalize both values: serialize to JSON with sorted keys, trim whitespace.
     Apply TS-PLUGIN-PARITY-001 Rules A–I canonicalization and compare
     plugin-output OCSF (from step 3 in-memory result) against reference OCSF
     (from step 4 fixture JSON). Comparison is byte-identical after canonical
     JSON serialization (sorted keys, whitespace-trimmed).
  6. Assert parity verdict is PASS or WARN (zero FAILs) for the test case

- Minimum coverage per `(sensor_id, table)` pair: 3 real-sensor fixture cases + 3 synthesized cases
  (happy-path, null-field, unrecognized-enum, empty-result as applicable).

- The `crowdstrike.detections` table parity test specifically exercises the two-step pipeline:
  the DTU stub returns a detection IDs page from QueryV2 and full records from PostEntities;
  the spec-driven output must match the reference OCSF record set byte-by-byte on required
  fields (Rule A) and within timestamp tolerance (Rule C).

### 3. Behavioral Fidelity Preserved

The OCSF output of the spec-driven path is semantically equivalent to the prior hardcoded
adapter path for all test cases:
- Arrow schema column names and types match (string/integer/float/boolean/datetime/json); Arrow
  Datetime columns are nullable — when sensor data supplies a null primary timestamp with no
  fallback chain, the Arrow field contains null. This is correct and expected for sensors where
  null timestamps are valid data (see §Preconditions O-001 null-primary passthrough rule).
- Virtual fields `sensor = "{sensor_id}"` and `source = "{table_name}"` are injected
  (BC-2.16.001 postcondition)
- OCSF field mappings from `ocsf_field` entries reproduce the prior per-adapter normalization
- The parity verdict is PASS or WARN for all non-SKIP test cases; zero FAILs

## Invariants

- **INV-PARITY-001 (Replacement-before-deletion):** PLUGIN-MIGRATION-001-A (deletion of
  hardcoded Rust adapter modules) MUST NOT proceed until VP-PLUGIN-003 is verified GREEN
  for all 4 sensors. This invariant is enforced by the STORY-INDEX dependency graph
  (PLUGIN-MIGRATION-001-A depends_on PLUGIN-MIGRATION-001-D) and by the VP-PLUGIN-003
  gate in the PLUGIN-MIGRATION-001-A story pre-flight check.

- **INV-PARITY-002 (Spec file immutability of sensor_id):** Once a spec file is committed
  as a bundled spec, its `sensor_id` value is immutable. Changing the `sensor_id` in the
  TOML file changes the DataFusion table namespace (`{sensor_id}.{table_name}`) and is
  therefore a breaking change requiring a new BC. (Spec files may be amended for non-ID
  fields without a new BC.)

- **INV-PARITY-003 (Spec file is the source of truth for table schema):** After PLUGIN-MIGRATION-001-D
  merges, the TOML spec files (not the Rust adapter source) are the source of truth for
  the schema of the 4 initial sensor tables. Schema changes require amending the spec file
  and re-validating parity tests.

- **INV-HARNESS-ROUTE-PARITY (Harness clone route surface must mirror standalone DTU):**
  The in-process clone modules in `prism-dtu-harness` (under `src/clones/`) MUST expose
  the same HTTP route surface as their corresponding standalone `prism-dtu-*` crates.
  Specifically:
  - `prism-dtu-harness` CrowdStrike clone (both in-process and network-mode router builders)
    MUST register GET `get_host_details` AND POST `post_host_details` on
    `/devices/entities/devices/v2`, mirroring the standalone's 9-endpoint surface (5 read,
    4 write) as updated by DEFECT-CSDEVICES-EMPTY-PIPELINE-001. The POST handler must enforce
    the same empty-ids 400 guard as the standalone (closes F-CSD-P9-001).
    Example: `router.route("/devices/entities/devices/v2", get(get_host_details).post(post_host_details))`.
  - `prism-dtu-harness::clones::armis::router()` MUST include `GET /api/v1/search`
    after S-DEMO-ARMIS-AQL-001 merges to develop (closes F-P6-DEFER-001).
  - `prism-dtu-harness::clones::claroty::router()` MUST include
    `POST /api/v1/audit_log/get` after S-DEMO-CLAROTY-AUDIT-DTU-001 merges to develop
    (closes F-P10-LOW-001).
  - `prism-dtu-harness::clones::claroty::router()` and
    `prism-dtu-harness::clones::claroty::network_router()` MUST register
    `POST /api/v1/device_alert_relations/` after S-DEMO-CLAROTY-DAR-001 merges to develop
    (closes INV-HARNESS-ROUTE-PARITY for Claroty device_alert_relations).
    Response envelope: `{"devices_alerts": [...], "count": N}` — `devices_alerts` key is
    required; `count` is optional (per `GetDeviceAlertsResponse`).
    Implemented by: **S-DEMO-CLAROTY-HARNESS-DAR-001**
    (AC-001 — `router()` returns HTTP 200 on valid Bearer, 401 on missing Bearer, RG-001
    `test_BC_2_16_013_claroty_harness_dar_router_returns_200_with_bearer_401_without`;
    AC-002 — response body uses `devices_alerts` key, NOT path stem `device_alert_relations`,
    RG-002 `test_BC_2_16_013_claroty_harness_dar_response_envelope_uses_devices_alerts_key_not_stem`;
    AC-003 — `network_router()` returns HTTP 200 on valid Bearer, 401 on missing Bearer,
    RG-003 `test_BC_2_16_013_claroty_harness_dar_network_router_returns_200_with_bearer_401_without`).
  - Auth model per sensor MUST match the standalone DTU: Armis → HTTP 403 on missing
    Bearer; Claroty → HTTP 401 on missing Bearer. These are NOT interchangeable.
  - Response envelope shapes MUST match standalone DTU responses:
    Armis search: `{"data": {"results": [...], "total": N}}`;
    Claroty audit_log: `{"audit_log": [...], "total": N}`;
    Claroty device_alert_relations: `{"devices_alerts": [...], "count": N}` —
    `devices_alerts` key is required; `count` is optional (per `GetDeviceAlertsResponse`
    where only `devices_alerts` is in `required:`). The DTU route MUST use `devices_alerts`
    as the response key, not the path stem `device_alert_relations`. (Standalone DTU route
    implemented by S-DEMO-CLAROTY-DAR-001; harness route implemented by
    S-DEMO-CLAROTY-HARNESS-DAR-001 — see DTU-EXT-006 and the anchored MUST above.)
  - CrowdStrike `detection_detail()` response shape: the `prism-dtu-harness` CrowdStrike
    clone's `detection_detail()` handler MUST include all top-level fields required by
    `crowdstrike.sensor.toml` detections columns: `detection_id`, `status`, `severity`,
    `created_timestamp`, `tactic`, `technique`, and `device_id` (top-level field, distinct
    from nested device sub-objects). The response MUST include a non-empty `behaviors`
    array where each element contains at least `ioc_type`, `ioc_value`, `ioc_source`, and
    `ioc_description` keys (`ioc_value` is nullable — null is valid; the key MUST be
    present). The `device_id` value MUST be a valid host ID from
    `generate_host_ids(org_slug, seed)`, computed as
    `generate_host_ids(org_slug, seed)[det_index % HOST_COUNT]`. `det_index` is the
    canonical detection index parsed from the `detection_id` trailing integer
    (format `det-{org_slug}-{seed}-{NNN}` → NNN): this makes the detection→device
    mapping STABLE across all request batch shapes — the same `detection_id` MUST
    always map to the same `device_id` regardless of batch position.
    Batch-position-derived indices (e.g., a plain `.enumerate()` counter that
    resets with each request's result slice) are forbidden because they break
    mapping stability when the caller issues partial-page or re-ordered requests.
    Literal placeholder strings that do not appear in the harness host pool are
    forbidden; a harness-mode
    JOIN `crowdstrike_detections JOIN crowdstrike_devices ON device_id = device_id`
    MUST return non-empty rows when both tables have data.
    Field TYPES in the `detection_detail()` response MUST match the standalone DTU
    generator's emission types (parity reference): `severity` MUST be a string label
    from the standalone generator's set (`"Low"` / `"Medium"` / `"High"` /
    `"Critical"`) matching `crowdstrike.sensor.toml` `column_type = "string"`;
    numeric severity values (e.g., `1`, `2`, `3`, `4`) are forbidden in the harness
    clone. Governs F-CSD-P30-OBS-003 (architect Option A ruling 2026-07-11).
    Governs F-CSD-P31-OBS-001 (det_index semantic disambiguation — stable host-pool
    mapping, 2026-07-11). Governs F-CSD-P31-MED-001 (severity string-type
    enforcement per standalone DTU parity, 2026-07-11). Governs F-CSD-P29-006
    (architect IN-SCOPE-FIX ruling 2026-07-11; devices-table `host_detail()` 6/6
    field-completeness precedent).
  - **Admin-token bearer comparison MUST use constant-time equality (`ct_compare_tokens`):** Every `Authorization: Bearer <token>` comparison in `prism-dtu-harness` that checks the provided token value against the stored `admin_token` MUST use constant-time byte comparison via the shared helper `ct_compare_tokens(provided: &str, expected: &str) -> bool` (implemented with `subtle::ConstantTimeEq`). Non-constant-time `!=` / `==` string equality leaks timing information about where the first differing byte occurs (CWE-208 timing side-channel). All 13 comparison sites across `src/builder.rs` (`check_bearer`), `src/clone_server.rs`, and the per-clone modules (`src/clones/armis.rs`, `src/clones/claroty.rs`, `src/clones/cyberint.rs`, `src/clones/crowdstrike.rs`, `src/clones/jira.rs`, `src/clones/pagerduty.rs`, `src/clones/slack.rs`) MUST call `ct_compare_tokens`. The admin token is a UUID-v4 string in test contexts; constant-time comparison is the correct default to prevent future promotion of the harness into security-sensitive contexts without regression. Closed by: DRIFT-HARNESS-ADMIN-TOKEN-CT-001 in S-DRIFT-SAP2-DEVICES-TOML-SURFACE-001 (D-1666, 2026-07-10).
  - Route parity is verified by multi-tenant harness tests (BC-3.5.001/BC-3.5.002 consumers).
  This invariant is governed by ADR-031 (DTU=true-DTU) and is implemented by story
  S-DEMO-HARNESS-CLONE-PARITY-001 (closes F-P6-DEFER-001, F-P10-LOW-001) and by the
  DEFECT-CSDEVICES-EMPTY-PIPELINE-001 fix lane for CrowdStrike (closes F-CSD-P9-001).

- **DI-030 (partial-failure isolation):** A parity failure for one `(sensor_id, table)` pair
  does NOT block other sensor tables from loading. Each parity test is isolated.

## Route Coverage Table (POL-33)

> **Cross-reference:** INV-HARNESS-ROUTE-PARITY (§Invariants above). This table satisfies POL-33
> (`route_coverage_table_required_for_stagemask_changes`) for the CrowdStrike DTU surface modified
> by DEFECT-CSDEVICES-EMPTY-PIPELINE-001. All three route registration sites are covered: standalone
> `prism-dtu-crowdstrike`, harness in-process (`build_crowdstrike_router`), and harness network-mode
> (`build_crowdstrike_network_router`). Scope: CrowdStrike `containment_status`-projecting routes
> only. Claroty, Cyberint, and Armis sensor routes carry no `containment_status` field and have no
> scenario-state-dependent response fields in the spec-driven parity path; they are EXEMPT from this
> table and will be added if StageMask-relevant routes are introduced in those crates.

| StageMask field | Clone crate | Route file | HTTP route | Guard mechanism | Status |
|-----------------|-------------|-----------|------------|-----------------|--------|
| `containment_status` | `prism-dtu-crowdstrike` | `src/routes/hosts.rs` (registered in `src/routes/mod.rs::build_router`) | `GET /devices/entities/devices/v2` | session-registry filter + containment-store merge in `host_details_inner`; absent `X-DTU-Session-Id` header → empty result (EC-003) | GUARDED |
| `containment_status` | `prism-dtu-crowdstrike` | `src/routes/hosts.rs` (registered in `src/routes/mod.rs::build_router`) | `POST /devices/entities/devices/v2` | shared `host_details_inner` (same session-registry filter + containment-store merge as GET); empty `ids` array → HTTP 400 | GUARDED |
| `containment_status` (write) | `prism-dtu-crowdstrike` | `src/routes/writes.rs` (registered in `src/routes/mod.rs::build_router`) | `POST /devices/entities/devices-actions/v2` | action_name guard (`contain` / `lift_containment`); writes containment-store consumed by GET/POST hosts routes; empty-ids → HTTP 400 | GUARDED |
| `containment_status` | `prism-dtu-harness` | `src/clones/crowdstrike.rs::build_crowdstrike_router` (in-process) | `GET /devices/entities/devices/v2` | session-registry filter + containment-store merge in `host_details_inner` | GUARDED |
| `containment_status` | `prism-dtu-harness` | `src/clones/crowdstrike.rs::build_crowdstrike_router` (in-process) | `POST /devices/entities/devices/v2` | shared `host_details_inner`; empty `ids` → HTTP 400 | GUARDED |
| `containment_status` (write) | `prism-dtu-harness` | `src/clones/crowdstrike.rs::build_crowdstrike_router` (in-process) | `POST /devices/entities/devices-actions/v2` | action_name guard; writes containment-store; empty-ids → HTTP 400 | GUARDED |
| `containment_status` | `prism-dtu-harness` | `src/clones/crowdstrike.rs::build_crowdstrike_network_router` (network-mode) | `GET /devices/entities/devices/v2` | session-registry filter + containment-store merge; bearer-guard applied only to list routes (`/devices/queries/`, `/detects/queries/`), not this detail route | GUARDED |
| `containment_status` | `prism-dtu-harness` | `src/clones/crowdstrike.rs::build_crowdstrike_network_router` (network-mode) | `POST /devices/entities/devices/v2` | shared `host_details_inner` (identical to in-process path); empty-ids → HTTP 400 | GUARDED |
| `containment_status` (write) | `prism-dtu-harness` | `src/clones/crowdstrike.rs::build_crowdstrike_network_router` (network-mode) | `POST /devices/entities/devices-actions/v2` | action_name guard; writes containment-store; empty-ids → HTTP 400 | GUARDED |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-016-013-001 | DTU clone not started when parity test runs | Test skipped (SKIP verdict per TS-PLUGIN-PARITY-001 Rule H); marked as DTU debt in parity report |
| EC-016-013-002 | `cyberint.incidents` table parity test | SKIP status per TS-PLUGIN-PARITY-001 Cyberint DTU Gap Note until DTU `incidents` pagination coverage is verified |
| EC-016-013-003 | CrowdStrike two-step batch boundary (exactly 100 IDs, triggering batch split) | Parity test includes a synthesized fixture with 100 IDs to exercise batch capping at CROWDSTRIKE_BATCH_SIZE (100); spec MUST NOT produce a 101-item batch |
| EC-016-013-004 | Claroty polymorphic ID (integer vs UUID string) | Parity test includes one integer-ID fixture and one UUID-string-ID fixture; spec column `type: "string"` normalizes both; OCSF output must match reference for each |
| EC-016-013-005 | Armis timestamp fallback to `now()` | When `firstSeen` and `lastSeen` are absent, spec produces a fetch-time timestamp; reference does too (same fallback path); TS-PLUGIN-PARITY-001 Rule C "both took same fallback path" → PASS by convention |
| EC-016-013-006 | Spec file present but DTU clone not in scope (Wave 1 test run without all DTUs built) | Individual parity tests that require their DTU clone are `#[ignore]` tagged with the message `"requires prism-dtu-{sensor} DTU clone"` until the DTU story (S-6.07–6.10) merges |
| EC-016-013-007 | Null OCSF field in reference output absent from actual (Rule B null vs absent) | Parity WARN (not FAIL); logged in parity report; does not block VP-PLUGIN-003 verification |
| EC-016-013-008 | Spec loaded successfully but no `sensor_specs_dir` configured to include bundled path | The implementation test must set `sensor_specs_dir` to `crates/prism-sensors/specs/` (or equivalent test path) explicitly; mis-configuration in test is a test authoring defect, not a BC violation |
| EC-016-013-009 | `device_alert_relations` DTU route or TOML `response_path` uses path-stem key `device_alert_relations` instead of actual API key `devices_alerts` | All relation rows are silently lost at normalization time; the correct top-level response key is `devices_alerts` per `GetDeviceAlertsResponse` (verified against authoritative xDome OpenAPI). The DTU handler and `response_path` in `claroty.sensor.toml` MUST use `devices_alerts`. This is a silent-failure mode: the pipeline returns an empty result set with no error rather than a parse error. |

## Error Conditions

| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-SPEC-001` | Bundled spec file fails BC-2.16.009 validation at CI time | CI fails; spec file must be corrected before merge; this is a pre-merge gate |
| `E-SPEC-009` | Duplicate `sensor_id` across two spec files (e.g., two files both declare `sensor_id: "crowdstrike"`) | BC-2.16.001 rejects the second file with `E-SPEC-009` per error-taxonomy.md; first file wins. E-SPEC-009 covers ONLY the duplicate-sensor_id case — it does NOT cover filename-stem-vs-sensor_id mismatch (see E-SPEC-017 below). |
| `E-SPEC-017` | Spec `sensor_id` does not case-sensitively match the filename stem (e.g., `crowdstrike.sensor.toml` with `sensor_id: "falcon"`) | BC-2.16.001 rejects the offending file with `E-SPEC-017` per error-taxonomy.md v1.45. Bundled spec naming convention is `{sensor_id}.sensor.toml`; mismatch indicates a rename without sensor_id update or vice versa; reject at load time to prevent silent namespace drift. (Registered as new code E-SPEC-017 in FB-IMPL-P2-PO 2026-05-20 — prior pass-1 incorrectly cited E-SPEC-009 for this case; E-SPEC-009 has distinct duplicate-sensor_id semantics.) |
| `E-SPEC-018` | `ColumnSpec::timestamp_formats` is non-empty and no format successfully parsed the column value (multi-format timestamp parse failure) | `PipelineExecutor` emits `E-SPEC-018` (`TimestampParseFailure`) per error-taxonomy.md v1.45. Only emitted when `timestamp_formats` is explicitly set on a `ColumnType::Datetime` column; columns with empty `timestamp_formats` (default) use ISO 8601 exclusively and emit a different error on parse failure. Registered FB-IMPL-1 2026-05-21 per ADR-028 v1.10 §D8-C. |

**Note on parity FAIL verdict (test verdict, not runtime error):** A parity test FAIL verdict
(where `PipelineExecutor` output does not match the reference OCSF output for a test case) is
a **test verdict**, not a runtime error code. When a parity test fails, the integration test
itself `assert!`s false — no runtime error code is emitted. The fix is to correct the TOML spec's
field mapping or step pipeline until the parity test passes. (The previously cited fabricated code
`E-SPEC-015` has been removed per F-004 fix-burst-1 FB-IMPL-P1-PO 2026-05-20; `E-SPEC-015` was
never registered in error-taxonomy.md and does not exist as a runtime error.)

## Canonical Test Vectors

| Scenario | Sensor | Input | Expected Outcome |
|----------|--------|-------|-----------------|
| Happy-path CrowdStrike detections | crowdstrike | DTU stub: QueryV2 returns 3 detection IDs from `GET /detects/queries/detects/v1`; PostEntities returns 3 full records from `POST /detects/entities/summaries/GET/v1` | Parity PASS: spec-driven OCSF matches reference OCSF for all 3 detections; `request_count >= 2` (single-page QueryV2 assumption: exactly 2 if response fits one page; > 2 if QueryV2 paginates) |
| CrowdStrike batch cap | crowdstrike | DTU stub: QueryV2 returns 100 detection IDs in one page | Parity PASS: spec produces one PostEntities batch of 100 (not 101+); `batch_size` cap respected |
| Claroty integer ID | claroty | DTU stub: asset record with `"id": 12345` (integer) | Parity PASS: `id` column value `"12345"` (string-normalized) matches reference |
| Claroty UUID string ID | claroty | DTU stub: asset record with `"id": "550e8400-e29b-41d4-a716-446655440000"` | Parity PASS: `id` column value `"550e8400-..."` matches reference |
| Cyberint alerts happy path | cyberint | DTU stub: 5 alert records from `GET /api/v1/alerts` with ISO-8601 timestamps; auth via `cyberint_session` cookie | Parity PASS: timestamps normalized to UTC per Rule C; OCSF fields match; reference loaded from `crates/prism-dtu-cyberint/fixtures/parity/reference-ocsf/alerts.json` |
| Armis devices timestamp fallback | armis | DTU stub: device record with no `firstSeen` or `lastSeen` fields | Parity PASS by Rule C convention (both sides take fetch-time timestamp fallback path); WARN logged |
| Armis AQL forwarding | armis | Query with custom AQL expression passed in `${query.filter.aql}` (caller sets `FetchContext::query_filters["aql"]`) | Parity PASS: DTU receives verbatim AQL expression in `aql` parameter; response matches reference |
| Spec load validation — crowdstrike | crowdstrike | `crowdstrike.sensor.toml` content passed through `SpecLoader::parse(toml_input: &str)` (spec_parser.rs::SpecLoader::parse) — NOTE: `SpecLoader::parse` has no filename context; filename-stem vs sensor_id validation requires `SpecLoader::load_all()` or `parse_spec_directory()` which supply the filename. See F-LP4-MED-002 closure in §Error Conditions note. | `Ok(SensorSpec)` with `sensor_id == "crowdstrike"`, `auth_type == "oauth2_client_credentials"`, correct table count (2 verified tables: detections, devices; incidents table RETIRED per D-1889) |
| Empty SKIP — cyberint.incidents | cyberint | Parity test targeting `cyberint.incidents` table | Test returns SKIP with message "cyberint incidents DTU gap — see TS-PLUGIN-PARITY-001 Cyberint DTU Gap Note" |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| VP-148 (VP-PLUGIN-003) | DTU parity: TOML+plugin path output matches deleted Rust adapter path per sensor — per TS-PLUGIN-PARITY-001 canonicalization. This BC is the primary source contract for VP-PLUGIN-003. Parity test must achieve zero FAILs across all non-SKIP `(sensor_id, table)` pairs for VP-PLUGIN-003 to be verified. |

## Related BCs

- BC-2.16.001: Sensor Spec File Loading — Parse TOML, Validate Schema, Register Tables — the mechanism by which bundled specs are discovered and loaded (composing with)
- BC-2.16.002: Multi-Step Fetch Pipeline Execution — Sequential Steps with Variable Interpolation — the execution engine for the CrowdStrike two-step spec (depends on)
- BC-2.16.009: Spec File Validation — Schema Validation, Variable Reference Resolution, OCSF Field Validation — the validator that each bundled spec must pass at load time (depends on)
- BC-2.16.012: PluginRegistry Dispatch in spec_parser.rs — Hardcoded Sensor Names Replaced with Registry Lookup — the dispatch mechanism whose behavioral output this BC asserts parity for (depends on)
- BC-2.01.013: DataSource Trait Eliminates Per-Sensor Code Duplication — the runtime adapter contract that TOML specs satisfy post-migration (composes with)
- BC-2.01.005: CrowdStrike OAuth2 Authentication and Two-Step Fetch — the prior Rust implementation whose behavior this BC preserves (supersedes within spec-driven scope)
- BC-2.01.006: Cyberint Assets Cookie-Based Authentication and Multi-Format Timestamp Parsing — prior implementation preserved by cyberint.sensor.toml (supersedes within spec-driven scope)
- BC-2.01.007: Claroty Bearer Token Auth with Polymorphic ID Handling — prior implementation preserved by claroty.sensor.toml (supersedes within spec-driven scope)
- BC-2.01.008: Armis Token Exchange Auth with AQL Query Forwarding and Timestamp Fallback — prior implementation preserved by armis.sensor.toml (supersedes within spec-driven scope)
- BC-2.01.017: StaticCookieAuthProvider Contract — No-Login-Roundtrip Cookie Injection: sibling contract specifying the corrected Cyberint auth behavior per ADR-031 §D3. The DTU-parity test family for Cyberint (VP-PLUGIN-003 / VP-148) MUST assert `Cookie: access_token=...` header shape per BC-2.01.017 TV-BC-2.01.017-002/003; test vectors that pass with `cyberint_session` are not DTU-parity evidence under ADR-031 §D5.

## Architecture Anchors

- ADR-028 v1.10 §D1 (URL grounding rule — TOML spec URL paths derived from DTU clone route registrations, not production Rust adapter code)
- ADR-028 v1.10 §D2 (auth_type grounding rule — TOML spec auth_type derived from DTU clone enforcement behavior, which reflects the real third-party API's auth contract; §D2 supersedes ADR-026 §D3 per D-747)
- ADR-028 v1.10 §D3 (parity reference OCSF grounding rule — committed fixture JSON at `crates/prism-dtu-{sensor}/fixtures/parity/reference-ocsf/<table>.json`; no prism-sensors dev-dep required)
- ADR-028 v1.10 §D5 (DTU extension prerequisite — spec entry for a URL path with no DTU route registration is an architectural violation; DTU-EXT-001..005 identified; documented-gap exception per §D9; DTU-EXT-001 and DTU-EXT-005 RETIRED per D-1889)
- ADR-028 v1.10 §D6 (scope expansion — PLUGIN-MIGRATION-001-A migrates live `*Auth::auth_type_name()` to match DTU-grounded auth_type values; auth divergence between TOML spec and live adapter return is intentional and tracked until 001-A merges)
- ADR-028 v1.10 §D8 (O-001 LOCKED Option A — `ColumnSpec::timestamp_formats` + `ColumnSpec::timestamp_fallback_chain` grammar extension; Cyberint `["iso8601", "unix_epoch_seconds"]`; Armis `["first_seen"] → now()` (amended FB-IMPL-2 — prior `["last_seen", "first_seen"]` was semantic no-op); E-SPEC-018 registered)
- ADR-028 v1.10 §D9 (documented-gap exception — incidents table retained in crowdstrike.sensor.toml as documented-gap entry; AC-001 `tables.len() == 3` stands; §D9 scope clarified: table-level gaps only, NOT parameter-level projections; `page_size` removed from cyberint.sensor.toml pagination block per F-LP2-MEDIUM-001)
- ADR-028 v1.10 §D10 (co-merge contract — 001-D + 001-A must deploy to production simultaneously; E-SPEC-012 regression prevention for Claroty bearer_static vs live cookie_roundtrip)
- ADR-023 §Decision Rules — Rule 3 (VP-PLUGIN-003 parity gate — replacement-before-deletion prerequisite)
- ADR-023 §Decision Rules — Rule 1 (four initial sensors ship as pure TOML specs; no in-repo .prx plugin required for the four initial sensors; OCSF complex-transform plugins are a separate concern per Rule 1)
- TS-PLUGIN-PARITY-001 (canonicalization rules for parity comparison: Rules A–I, Rule I fixture minimum, Cyberint DTU Gap Note)
- ADR-023 §Architectural Constraints — C2 (PipelineExecutor as the spec-driven execution engine, replacing the `Ok(Vec::new())` stub; real implementation in PLUGIN-PREREQ-B)
- CLAUDE.md §Source-of-Truth Precedence #7 (spec wins on code-vs-spec conflict; legacy adapter URLs and auth_type_name() strings are bugs in code deleted by PLUGIN-MIGRATION-001-A)

## Story Anchor

PLUGIN-MIGRATION-001-D (implementing story; planned → draft after PO authoring complete)

## VP Anchors

- VP-148 (VP-PLUGIN-003): DTU parity verification property anchored to this BC

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-029 ("Config-Driven Sensor Adapters") per capabilities.md §CAP-029 |
| Capability Anchor Justification | CAP-029 ("Config-Driven Sensor Adapters") per capabilities.md §CAP-029 — this BC describes authoring the 4 production TOML sensor spec files that ARE the config-driven sensor adapter artifacts for the 4 initial sensors, plus their DTU parity verification. CAP-029 defines exactly this: "All sensor tables — including the four initial sensors (CrowdStrike, Cyberint, Claroty, Armis) shipped as bundled TOML spec files — are registered with DataFusion uniformly and queryable via the same `query` MCP tool (CAP-015)." |
| L2 Invariants | DI-008 (client scoping — specs do not cross client boundaries), DI-030 (partial-failure isolation — one spec failure does not block others), DI-012 (auth composition prevention — each spec declares exactly one auth_type) |
| L2 Entities | SensorSpec, TableSpec, ColumnSpec, PipelineResult |
| Priority | P0 |
| ADR anchors | ADR-028 v1.10 §D1 (URL grounding), §D2 (auth_type grounding; supersedes ADR-026 §D3 per D-747), §D3 (fixture-JSON parity reference), §D5 (DTU extension prerequisite; documented-gap exception per §D9; DTU-EXT-001 + DTU-EXT-005 RETIRED D-1889), §D6 (001-A auth migration scope), §D8 (O-001 Option A LOCKED: timestamp_formats + timestamp_fallback_chain grammar extension; Armis chain amended to `["first_seen"]` per FB-IMPL-2), §D9 (documented-gap entries permitted with DTU-EXT-NNN blocker ref; parameter-level projections NOT covered), §D10 (co-merge contract: 001-D + 001-A must deploy simultaneously); ADR-023 §Decision Rules — Rule 1, §Decision Rules — Rule 3; ADR-023 §Architectural Constraints — C2; TS-PLUGIN-PARITY-001 Rules A–I |
| Subsystem | SS-16 (Spec Engine) |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.36 | device-alert-relations-amendment | 2026-08-11 | product-owner | New `device_alert_relations` table added to §Postconditions §1 Claroty `claroty.sensor.toml` entry. Covers: endpoint `POST /api/v1/device_alert_relations/` (trailing-slash form, ADR-031 §D8-b); response key `devices_alerts` (not the path stem); `fields` REQUIRED (`GetDeviceAlertsParameters.fields`, `minItems: 1`); offset/limit pagination, API maximum `limit: 5000`; contracted column subset of 9 fields from the 92-value `AlertedDevicesPairs__fields_enum`; SAP-2 exclusion documentation for the remaining 83 fields; table rationale (xDome `alerts` surface carries no `severity` field; `device_alert_relations` is the sole prioritization source). DTU-EXT-006 registered in §Known Gaps (in-progress, story ID unassigned). `Claroty device_alert_relations` response envelope shape added to INV-HARNESS-ROUTE-PARITY §Response envelope shapes clause. EC-016-013-009 added (response key mismatch edge case). Frontmatter v1.35 → v1.36. Claim correction: orchestrator asserted default `sort_by` was `(device_alert_detected_time, alert_id, device_uid)`; authoritative xDome OpenAPI shows `(device_uid: asc, alert_id: asc)` — 2 fields only, no `device_alert_detected_time` in the default. Contract does not specify sort_by default; prism queries use explicit `fields` projection. **Story anchor fold (same v1.36, 2026-08-11):** DTU-EXT-006 row updated: story ID `S-DEMO-CLAROTY-DAR-001` (`status: draft`, wave 5) named as implementing story; harness route parity noted as separate follow-up per AC-007 and explicit `crates/prism-dtu-harness/` exclusion in that story's File Structure Requirements. §Postconditions §1 `device_alert_relations` URL grounding note updated to name S-DEMO-CLAROTY-DAR-001 AC-006 as the gap-close anchor. INV-HARNESS-ROUTE-PARITY: harness-parity tracking bullet added for `POST /api/v1/device_alert_relations/` — phrased as pending follow-up (NOT anchored as MUST to S-DEMO-CLAROTY-DAR-001 because that story explicitly excludes harness modifications; story-writer's proposed MUST wording rejected per TD-VSDD-097 mandate-anchor dimension 3). §Response envelope shapes note updated to reference S-DEMO-CLAROTY-DAR-001 and AC-007. **Harness-story anchor fold (same v1.36, 2026-08-11):** S-DEMO-CLAROTY-HARNESS-DAR-001 now exists (`status: draft`, wave 5, `depends_on: [S-DEMO-CLAROTY-DAR-001]`). Four normative sites updated: (1) INV-HARNESS-ROUTE-PARITY tracking bullet replaced with anchored MUST — both `router()` and `network_router()` named; AC-001/RG-001 (`router()` 200/401 `test_BC_2_16_013_claroty_harness_dar_router_returns_200_with_bearer_401_without`), AC-002/RG-002 (response key `devices_alerts` `test_BC_2_16_013_claroty_harness_dar_response_envelope_uses_devices_alerts_key_not_stem`), AC-003/RG-003 (`network_router()` 200/401 `test_BC_2_16_013_claroty_harness_dar_network_router_returns_200_with_bearer_401_without`); two-router claim verified against `clones/claroty.rs` (`router()` and `network_router()` both exist, neither registers `device_alert_relations` on develop). (2) §Response envelope shapes "pending separate follow-up story" → S-DEMO-CLAROTY-HARNESS-DAR-001. (3) §Postconditions §1 URL grounding "follow-up story is required" → S-DEMO-CLAROTY-HARNESS-DAR-001. (4) DTU-EXT-006 "harness parity MUST will be anchored when follow-up story created" → S-DEMO-CLAROTY-HARNESS-DAR-001. Stale-phrasing sweep: all four pending-follow-up forms in normative text resolved; changelog prior-fold record text grandfathered as historical record. **Column-list reconciliation (same v1.36, 2026-08-11):** Implementer's verified list is 10 columns, not 9 — `device_alert_status` confirmed present in `AlertedDevicesPairs__fields_enum` (92 values, authoritative xDome OpenAPI, coordinator-verified). Three sites updated: (1) §Postconditions §1 contracted column subset: 9 → 10 columns, `device_alert_status` appended to named list, "All 9" → "All 10", "83 fields" → "82 fields" (two occurrences in the block); (2) §Postconditions §1 SAP-2 exclusion documentation: "83 excluded fields" → "82 excluded fields"; (3) frontmatter `modified` comment: "83 omitted fields" → "82 omitted fields (10 contracted, 82 excluded, 92 total)". Arithmetic: 82 excluded + 10 contracted = 92 total enum values ✓. Note: `device_alert_status` is an individual field in the enum; no `{device_uid, alert_id, device_alert_detected_time, device_alert_status}` 4-tuple exists in the API — those are independent claims. Branch `fix/claroty-live-api-fidelity` at `0d80cbeac` — not yet pushed, not merged; merge-state language in §Known Gaps DTU-EXT-006 remains "pending". |
| 1.35 | MED-008-annotation-burst | 2026-08-03 | product-owner | MED-008 (PR #234 adversarial review): annotation-only amendment to §Postconditions §1 flagging three stale ADR-028 §D2 authority citations. **(1) Grounding-authority intro** — added `[SUPERSEDED-PENDING for Armis and Cyberint — ADR-053 §D1]` qualification after the ADR-028 §D2 statement; scopes the supersession to Armis and Cyberint; CrowdStrike and Claroty authorities unchanged. **(2) Armis entry** — added `[SUPERSEDED-PENDING — ADR-053 §D2]` annotation after the auth-grounding sentence; `auth_type = "bearer_static"` value preserved (live test binding: `test_HS_016_BC_2_16_013_armis_spec_declares_bearer_static_auth`). **(3) Cyberint entry** — added `[SUPERSEDED-PENDING — ADR-053 §D3-a]` annotation after the auth-grounding sentence; single-surface `cookie_roundtrip` entry preserved (live test binding: `test_HS_015_BC_2_16_013_cyberint_spec_declares_cookie_roundtrip_auth`). No `auth_type` value rewritten. CrowdStrike and Claroty entries untouched. Full amendment execution (value rewrites, dual-surface split) owned by `S-WAVE-A-CYBERINT-SPEC-001` per ADR-053 §D5. **Defect-class sweep (same burst):** struck false DTU-precedes-spec grounding direction assertions from three sensor auth-grounding sentences. Removed `CLAUDE.md §Source-of-Truth Precedence #7 applies: spec follows DTU, not adapter code.` from Claroty and Cyberint entries; removed `Spec follows DTU, not adapter code.` from Armis entry. CLAUDE.md §Source-of-Truth Precedence #7 governs code-vs-spec conflicts in favour of the SPEC — the opposite of "spec follows DTU" — making this clause false independent of ADR-053. Preceding ADR-028 §D2/§D6 context already explains the intentional divergence; the struck sentence was redundant and false. CrowdStrike entry clean (no direction assertion present). |
| 1.34 | wave-a-spec-evolution-fix-burst-17 | 2026-07-23 | product-owner | F-WASE-P17-MED-001: §Related BCs — 9 of 10 entries corrected to canonical H1s (POL-7 bc_h1_is_title_source_of_truth class sweep). (1) BC-2.16.009 "Spec File Validation" → "Spec File Validation — Schema Validation, Variable Reference Resolution, OCSF Field Validation"; (2) BC-2.16.001 "Sensor Spec File Loading" → "Sensor Spec File Loading — Parse TOML, Validate Schema, Register Tables"; (3) BC-2.16.002 "Multi-Step Fetch Pipeline" → "Multi-Step Fetch Pipeline Execution — Sequential Steps with Variable Interpolation"; (4) BC-2.16.012 "PluginRegistry Dispatch" → "PluginRegistry Dispatch in spec_parser.rs — Hardcoded Sensor Names Replaced with Registry Lookup"; (5) BC-2.01.013 "DataSource Trait" → "DataSource Trait Eliminates Per-Sensor Code Duplication"; (6) BC-2.01.005 "CrowdStrike OAuth2 Auth and Two-Step Fetch" → "CrowdStrike OAuth2 Authentication and Two-Step Fetch"; (7) BC-2.01.006 "Cyberint Cookie-Based Auth" → "Cyberint Assets Cookie-Based Authentication and Multi-Format Timestamp Parsing"; (8) BC-2.01.007 "Claroty Bearer Token Auth" → "Claroty Bearer Token Auth with Polymorphic ID Handling"; (9) BC-2.01.008 "Armis Bearer Token Auth" → "Armis Token Exchange Auth with AQL Query Forwarding and Timestamp Fallback". BC-2.01.017 pre-existing CLEAN. input-hash updated at commit time. |
| 1.33 | D-1889-wrong-direction-retirements | 2026-07-22 | story-writer | **D-1889 wrong-direction story retirements.** §Known Gaps: DTU-EXT-001 (CrowdStrike incidents) marked RETIRED — Incidents API removed ~2026-03; incidents table retired from crowdstrike.sensor.toml per S-CROWDSTRIKE-INCIDENTS-RETIREMENT-001; incidents derived from Alerts `aggregate_id`. DTU-EXT-005 (Cyberint alerts pagination page_size) marked RETIRED — Cyberint alerts endpoint is `POST /alert/api/v1/alerts` with `page/size` pagination per ADR-053 §Finding-1; DEFECT-CYBERINT-SPEC-FIDELITY-001 supersedes. §Postconditions §1 CrowdStrike `incidents` row updated to reference DTU-EXT-001 RETIRED. §Canonical Test Vectors Spec load validation row updated to reflect incidents RETIRED. TD-VSDD-091 fixes: §Known Gaps DTU-EXT-005 `alerts.rs:38-40` → `alerts.rs::AlertListParams`; §Postconditions §2 step 2 + §Canonical Test Vectors `spec_parser.rs:655` → `spec_parser.rs::SpecLoader::parse`. BC v1.32→v1.33. POL-32. |
| 1.32 | S-DRIFT-SAP2-DEVICES-TOML-SURFACE-001-PO-CT-amendment | 2026-07-11 | product-owner | **DRIFT-HARNESS-ADMIN-TOKEN-CT-001 constant-time token comparison requirement (D-1666, 2026-07-10) — BC amendment closing OQ-001 (S-DRIFT-SAP2-DEVICES-TOML-SURFACE-001).** §Invariants INV-HARNESS-ROUTE-PARITY: added explicit **Admin-token bearer comparison MUST use constant-time equality (`ct_compare_tokens`)** clause — every `Authorization: Bearer <token>` comparison in `prism-dtu-harness` that checks the provided token against the stored `admin_token` MUST use constant-time byte comparison via shared `ct_compare_tokens(provided: &str, expected: &str) -> bool` helper (implemented with `subtle::ConstantTimeEq`). Applies to all 13 comparison sites: `src/builder.rs` (`check_bearer`), `src/clone_server.rs`, and 7 per-clone modules. Addresses CWE-208 timing side-channel — non-constant-time `!=` / `==` string comparison leaks information about where the first differing byte occurs. Rationale: constant-time is the correct default even for test-context UUID tokens to prevent future promotion into security-sensitive contexts without regression. Frontmatter v1.31→v1.32; modified: 2026-07-11. |
| 1.31 | F-CSD-P31-clarifications-PO-burst | 2026-07-11 | product-owner | F-CSD-P31-OBS-001 + F-CSD-P31-MED-001 clarifications — INV-HARNESS-ROUTE-PARITY detection_detail() clause: (1) `det_index` defined as canonical detection index parsed from `detection_id` trailing integer (`det-{org_slug}-{seed}-{NNN}` → NNN); detection→device mapping STABLE across all request batch shapes; batch-position-derived indices forbidden; same `detection_id` MUST always map to same `device_id`. (2) `severity` field MUST be a string label (`"Low"` / `"Medium"` / `"High"` / `"Critical"`) matching standalone DTU generator emission types and `crowdstrike.sensor.toml` `column_type = "string"`; numeric severity values forbidden in harness clone. BC v1.30 → v1.31. POL-27/POL-32. |
| 1.30 | F-CSD-P30-OBS-003-PO-spec-note | 2026-07-11 | product-owner | F-CSD-P30-OBS-003 (architect Option A ruling 2026-07-11) — INV-HARNESS-ROUTE-PARITY CrowdStrike detection_detail() response-shape clause: added `device_id` host-pool constraint. `device_id` MUST be a valid host ID from `generate_host_ids(org_slug, seed)`, computed as `generate_host_ids(org_slug, seed)[det_index % HOST_COUNT]`. Literal placeholder strings not in the harness host pool are forbidden. A harness-mode JOIN `crowdstrike_detections JOIN crowdstrike_devices ON device_id = device_id` MUST return non-empty rows when both tables have data. Appended after the `ioc_description` sentence in the detection_detail() clause (v1.29). BC v1.29 → v1.30. POL-27/POL-32. |
| 1.29 | F-CSD-P29-006-PO-spec-note | 2026-07-11 | product-owner | F-CSD-P29-006 (architect IN-SCOPE-FIX ruling 2026-07-11) — INV-HARNESS-ROUTE-PARITY: added explicit CrowdStrike `detection_detail()` response-shape clause parallel to existing Armis search and Claroty audit_log envelope-shape clauses. Clause requires `prism-dtu-harness` CrowdStrike clone's `detection_detail()` handler to include all top-level fields required by `crowdstrike.sensor.toml` detections columns (`detection_id`, `status`, `severity`, `created_timestamp`, `tactic`, `technique`, `device_id` top-level), plus a non-empty `behaviors` array with `ioc_type`, `ioc_value` (nullable), `ioc_source`, `ioc_description` keys per element. Precedent: devices-table `host_detail()` 6/6 field-completeness ruling. BC v1.28 → v1.29. POL-27/POL-32. |
| 1.28 | F-CSD-P25-006-PO-burst | 2026-07-11 | product-owner | F-CSD-P25-006 (OBS) closure — POL-33 (`route_coverage_table_required_for_stagemask_changes`) compliance: added §Route Coverage Table (POL-33) with 9 rows covering all 3 CrowdStrike DTU registration sites (standalone `prism-dtu-crowdstrike::build_router`, harness in-process `build_crowdstrike_router`, harness network-mode `build_crowdstrike_network_router`). StageMask field: `containment_status` — read routes GET + POST `/devices/entities/devices/v2` on all 3 sites (GUARDED via shared `host_details_inner` session-registry filter + containment-store merge); write route `POST /devices/entities/devices-actions/v2` on all 3 sites (GUARDED via action_name guard + containment-store write). All 9 rows GUARDED. Claroty/Cyberint/Armis EXEMPT (no scenario-state-dependent fields in spec-driven parity path). Seeded from DEFECT-CSDEVICES-EMPTY-PIPELINE-001 worktree code truth. INV-HARNESS-ROUTE-PARITY cross-referenced in section intro. BC v1.27 → v1.28. POL-27/POL-32/POL-33. |
| 1.27 | F-CSD-P9-001-closure-PO-burst | 2026-07-10 | product-owner | F-CSD-P9-001 (HIGH) closure — INV-HARNESS-ROUTE-PARITY CrowdStrike parity gap: v1.26 documented the standalone `prism-dtu-crowdstrike` gaining POST `/devices/entities/devices/v2` (`post_host_details`, endpoint count 8→9) but omitted the corresponding harness-clone obligation. §Postconditions §1 CrowdStrike `devices` row: added **Harness parity (INV-HARNESS-ROUTE-PARITY)** note — both `prism-dtu-harness` CrowdStrike router builders (in-process and network-mode) MUST register GET `get_host_details` AND POST `post_host_details` on `/devices/entities/devices/v2`, mirroring the standalone's shared route composition (session-registry filter, org-id guard, containment merge, auth, empty-ids 400 on POST). §Invariants INV-HARNESS-ROUTE-PARITY: added CrowdStrike bullet alongside existing Armis and Claroty bullets; implementation story reference updated to include DEFECT-CSDEVICES-EMPTY-PIPELINE-001 fix lane as the CrowdStrike closure vehicle. No other sections changed. POL-27/POL-32. BC v1.26 → v1.27. |
| 1.26 | DEFECT-CSDEVICES-EMPTY-PIPELINE-001-PO-burst | 2026-07-10 | product-owner | CrowdStrike `devices` table step-2 corrected from GET to POST per architect ratification of DEFECT-CSDEVICES-EMPTY-PIPELINE-001 (research/defect-csdevices-empty-pipeline-rootcause-2026-07-10.md §Architect Ratification, D-1650). `fetch_devices` step 2 is now POST `/devices/entities/devices/v2` with body `{"ids": [...]}` — matching real CrowdStrike `PostDeviceDetailsV2` (FalconPy v1.2.0+; same body structure as existing `fetch_detections` POST step; supports up to 5000 IDs vs GET's 100). §Postconditions §1 CrowdStrike `devices` row: `(GET \`/devices/entities/devices/v2\`)` → `(POST \`/devices/entities/devices/v2\` with body \`{"ids": [...]}\`)`. URL grounding updated: `/devices/entities/devices/v2` now registers both `get_host_details` (GET; preserved) and `post_host_details` (POST; new); spec-driven path is POST; DTU endpoint count 8→9 (5 read, 4 write). Parity tests for `crowdstrike.devices` must exercise the POST path. BC v1.25 → v1.26. |
| 1.25 | Wave-5-Phase-A-PO-burst | 2026-06-03 | product-owner | Gate 4 (S-DEMO-CLAROTY-SPEC-PROSE-FIX-001 / F-P2-DEFER-001 closure): §Postconditions §1 Claroty `audit_logs` clause corrected from stale "GET /api/v1/audit_logs via offset pagination. No DTU route registered." to "POST /api/v1/audit_log/get; DTU route registered by S-DEMO-CLAROTY-AUDIT-DTU-001 (Gap-CL-006 CLOSED)". Combined with Gate 2 + Gate 3 in same burst. BC v1.24 → v1.25. |
| 1.24 | Wave-5-Phase-A-PO-burst | 2026-06-03 | product-owner | Gate 3 (S-DEMO-HARNESS-CLONE-PARITY-001 / closes F-P6-DEFER-001 + F-P10-LOW-001): Added INV-HARNESS-ROUTE-PARITY invariant specifying that `prism-dtu-harness::clones::armis::router()` must include `GET /api/v1/search` and `prism-dtu-harness::clones::claroty::router()` must include `POST /api/v1/audit_log/get` to mirror standalone DTU route surfaces per ADR-031. Rationale for amendment to BC-2.16.013 (not BC-3.5.001/BC-3.5.002): BC-3.5.001/002 describe harness isolation mechanics (port allocation, data segregation) — they do NOT specify which routes the in-process clones must register. BC-2.16.013 is the DTU-parity contract and is the correct home for route-surface coverage obligations. BC v1.23 → v1.24. |
| 1.23 | Wave-5-Phase-A-PO-burst | 2026-06-03 | product-owner | Gate 2 (S-DEMO-CLAROTY-TRAILING-SLASH-001 / ADR-031 §D8-b): Added trailing-slash parity clause to Claroty `alerts`, `devices`, and `audit_logs` postconditions — `path_template` values use trailing-slash form (`/api/v1/alerts/`, `/api/v1/devices/`, `/api/v1/audit_log/get/`); `prism-dtu-claroty` router includes `normalize_path` middleware so both slash-variant forms return 200. Added `devices` table entry (was missing from Claroty spec body; existed in TOML as DTU-grounded route). BC v1.22 → v1.23. |
| 1.22 | F-LP12R-MED-001 closure burst | 2026-06-01 | product-owner | F-LP12R-MED-001 closure: AQL discriminator divergence flagged in v1.21 as "reported for separate reconciliation — not fixed here" is now CLOSED. The implementation on `feature/S-DEMO-ARMIS-AQL-001` was conformed to `in:devices`/`in:alerts` (the discriminator strings specified in this BC's §Postconditions §1) per impl commit 26267916. Research artifact `.factory/research/armis-aql-discriminator-syntax-2026-06.md` records the disposition: "BC stands; implementation conforms." BC §Postconditions §1 Armis `devices` and `alerts` bullets and §Canonical Test Vectors were already correct and unchanged — no normative content edited. This row closes the open item in v1.21 that would otherwise read as an unresolved finding. |
| 1.21 | orchestration-correction burst | 2026-06-01 | product-owner | Orchestration correction of premature v1.20 gap-closure. §Postconditions §1 Armis `devices` and `alerts` bullets changed from flat "CLOSED by S-DEMO-ARMIS-AQL-001" to dual-state wording: implementation COMPLETE on `feature/S-DEMO-ARMIS-AQL-001` (`build_router()` anchor per TD-VSDD-091), gap CLOSES on story merge, gap remains OPEN on develop until then. §Known Gaps DTU-EXT-003 and DTU-EXT-004 rows updated to match dual-state form. No other sensor rows (CrowdStrike/Cyberint/Claroty) touched. AQL discriminator strings (`in:devices`/`in:alerts`) preserved as-specified in BC; story-vs-BC discriminator convention divergence (`in:type=Device`/`in:type=Alert` in implementation) reported for separate reconciliation — not fixed here. |
| 1.20 | F-LP11-MED-001 closure burst | 2026-06-01 | product-owner | F-LP11-MED-001 closure: §Postconditions §1 Armis `devices` and `alerts` bullets updated from deferred/OPEN to CLOSED by S-DEMO-ARMIS-AQL-001. §Known Gaps DTU-EXT-003 and DTU-EXT-004 rows marked CLOSED with resolution summary — `GET /api/v1/search` (AQL-search) registered in `prism-dtu-armis/src/clone.rs` `build_router()` per ADR-031 §D8-a. Volatile line-number citations (`line 143`, `line 150`) replaced with function-name anchor `build_router()` per TD-VSDD-091. No other sensor rows (CrowdStrike/Cyberint/Claroty) touched. |
| 1.19 | S-DEMO-CROWDSTRIKE-MULTIREGION-001 BC attachment burst | 2026-05-31 | product-owner | §Postconditions §1 CrowdStrike row: replaced stale `base URL pattern https://api.{cloud_region}.crowdstrike.com` with `base_url = "${env.CROWDSTRIKE_BASE_URL}"` per ADR-031 §D8-c. Added region runbook (us-1/us-2/eu-1/gov canonical URLs). Added E-SPEC-024 cross-reference (BC-2.16.009 §Validation Rules 6) for missing/empty env var behavior at spec-load time. No other postconditions changed — DTU parity tests pass `base_url` as a DTU SocketAddr override regardless of the spec's base_url value. |
| 1.18 | D-870 F-LP10-MED-001 comprehensive sweep | 2026-05-30 | product-owner | F-LP10-MED-001 changelog hygiene: v1.11 row was out of order (appeared between v1.16 and v1.15). Moved v1.11 to correct position between v1.12 and v1.10. Pre-existing defect deferred at D-LP9-001 (pass 9) as pre-existing/out-of-scope; promoted to in-scope under comprehensive sweep per POL-32 codification. No semantic content change. BC-INDEX v5.60→v5.61. |
| 1.17 | D-849 | 2026-05-29 | product-owner | §Related BCs: added BC-2.01.017 (StaticCookieAuthProvider — No-Login-Roundtrip Cookie Injection) cross-reference. DTU-parity tests for Cyberint (VP-148) must assert `Cookie: access_token=...` per ADR-031 §D5 and BC-2.01.017 TV-002/003. `cyberint_session` parity evidence is no longer sufficient per ADR-031 §D3. |
| 1.16 | D-776-post-merge | 2026-05-22 | state-manager | POL-14 auto-promotion at merge: PR #153 (PLUGIN-MIGRATION-001-D) squash-merged to develop@3f2de889 at 2026-05-22T09:05:47Z; status draft→active, lifecycle_status draft→active. |
| 1.15 | FB-IMPL-9 | 2026-05-21 | state-manager | F-LP10-LOW-001 closure — §Error Conditions lines 357-358 transitive cite-pin sweep: `error-taxonomy.md v1.42` → `v1.44` (E-SPEC-017 row) and `error-taxonomy.md v1.43` → `v1.44` (E-SPEC-018 row) per FB-IMPL-P22 PREREQ-E precedent + implementer current-authority code-comment pattern. 5th POL-29 axis recurrence (transitive cite-pin chain). No semantic content change. |
| 1.14 | FB-IMPL-2 PO | 2026-05-21 | product-owner | F-LP2-HIGH-006 closure (Option a — document null-primary passthrough): §O-001 implementer contract extended with null-primary passthrough rule — when a Datetime column primary value is null/absent with empty `timestamp_fallback_chain`, the field passes through to Arrow as null with no audit signal; this is valid sensor data (Cyberint `Alert.created_at: serde_json::Value` accepts JSON `null` per DTU types.rs). §Postconditions §3 (Behavioral Fidelity Preserved) first bullet extended to document Arrow Datetime nullable contract. No new error codes, no new tracing events — documentation-only closure. No implementer handoff required. BC-2.16.013 v1.13→v1.14. |
| 1.13 | FB-IMPL-2 | 2026-05-21 | architect | F-LP2-HIGH-004 closure (Option a): §O-001 Armis fallback chain corrected from `["last_seen", "first_seen"]` to `["first_seen"]` — the self-referential primary column name as first chain element is a semantic no-op; doc-comment "Skip the primary field itself" was false (no skip guard in code). Implementer must: (1) update `armis.sensor.toml` chain to `["first_seen"]`, (2) add defensive skip guard `if fb_field == &col.name { continue; }` in pipeline.rs fallback loop, (3) fix the false doc-comment at pipeline.rs:1495. ADR-028 v1.9→v1.10 §D8-B amended. F-LP2-MEDIUM-001 closure (Option b): DTU-EXT-005 added to §Known Gaps — `page_size` parameter removed from cyberint.sensor.toml per ADR-028 §D9 scope clarification (parameter-level projections not covered by documented-gap exception; `AlertListParams` struct at `crates/prism-dtu-cyberint/src/routes/alerts.rs:38-40` has no `page_size` field). §Architecture Anchors ADR-028 cite-pin advanced v1.9→v1.10 at §D8 + §D9 rows. §ADR anchors Traceability row updated. |
| 1.12 | FB-IMPL-1 | 2026-05-21 | architect | (D-FB-IMPL-1-OPT-A) F-LP1-HIGH-002/003 closure: §O-001 LOCKED Option A — grammar extension in `ColumnSpec` (`timestamp_formats: Vec<String>` + `timestamp_fallback_chain: Vec<String>`, both `#[serde(default)]`). Full implementer contract specified: recognized formats, normalization pipeline location, backward compat, E-SPEC-018 registered. Cyberint canonical formats `["iso8601", "unix_epoch_seconds"]` documented (DTU-grounded). Armis fallback chain `["last_seen", "first_seen"] → now()` locked (DTU-grounded, corrected to `["first_seen"]` in v1.13). §Postconditions §1 Cyberint + Armis rows updated: WASM plugin references replaced with Option A grammar. ADR-028 v1.8→v1.9 cite-pin sweep across 6 §Architecture Anchors sites (§D1/D2/D3/D5/D6) + §ADR anchors Traceability row. |
| 1.11 | FB-IMPL-P22-PO | 2026-05-21 | product-owner | F-LP22-MED-001 closure (16th coherence-axis: same-line dual-format cite-pin escape): swept `error-taxonomy.md v1.41` → `v1.42` at 1 active-prose site (§Error Conditions E-SPEC-017 row line 331). BC-2.16.013 v1.10→v1.11. |
| 1.10 | FB-IMPL-P17-PO | 2026-05-20 | product-owner | F-LP17-HIGH-002 propagation closure (POL-29 fixed-point per F-LP16-OBS-001): ADR-028 v1.7→v1.8 cite-pin sweep across 6 active-prose sites (lines 375-379, 403). Architect FB-IMPL-P17-ARCH reverted ADR-028 §Changelog to descending + bumped v1.7→v1.8 + added §D7 (Per-File Convention Lock rule); cites bump only, no structural change. |
| 1.9 | FB-IMPL-P16-PO | 2026-05-20 | product-owner | F-LP16-MED-001 propagation closure (POL-29 fixed-point per F-LP16-OBS-001): ADR-028 v1.6→v1.7 cite-pin sweep across 6 active-prose sites (lines 375-379, 403). Same-burst sweep avoiding leak-into-next-pass per fixed-point iteration discipline. |
| 1.8 | FB-IMPL-P15-PO | 2026-05-20 | product-owner | F-LP15-MED-001 closure: ADR-028 v1.5→v1.6 cite-pin sweep across 6 active-prose sites (lines 375-379, 403) + any other discovered sites. POL-29 cross-file sweep applied per F-LP15-OBS-001 process-gap (closure scope of this burst). |
| 1.7 | FB-IMPL-P13-PO | 2026-05-20 | product-owner | Closes pass-13 findings F-LP13-MED-002 (propagate ADR-028 v1.5 pin): §Architecture Anchors updated from bare ADR-028 to versioned ADR-028 v1.5 citations throughout; §D6 anchor added (PLUGIN-MIGRATION-001-A auth migration scope). §Postconditions §1 Claroty, Cyberint, Armis auth-grounding sentences updated with ADR-028 §D2 supersession of ADR-026 §D3 (D-747) context — each row now explicitly notes TOML value diverges from live `*Auth::auth_type_name()` until PLUGIN-MIGRATION-001-A migrates per ADR-028 §D6. Traceability §ADR anchors row updated to v1.5 + §D6. |
| 1.6 | FB-IMPL-P6-PO fix-burst-6 | 2026-05-20 | product-owner | Closes pass-6 finding F-LP6-LOW-001 (TD-VSDD-091 anti-volatile-pin sibling-asymmetric): replaced line-pinned cite `lib.rs:16-17` with module-doc anchor `crates/prism-dtu-armis/src/lib.rs module documentation` in §Postconditions §1 Armis auth-grounding sentence. POL-25 multi-cite sweep — HS-016 updated in same burst. ADR-028 §D2 row not modified (architect scope). |
| 1.5 | FB-IMPL-P5-PO fix-burst-5 | 2026-05-20 | product-owner | Closes pass-5 finding F-LP5-LOW-001 (TD-VSDD-091 anti-volatile-pin): replaced line-pinned cite `alerts.rs:43-46` with symbol anchor `alerts.rs::extract_session_token()` in §Postconditions §1 cyberint auth-grounding sentence. POL-25 multi-cite sweep — HS-015 updated in same burst. ADR-028 §D2 row not modified (architect scope; already fixed in ADR-028 v1.1). |
| 1.4 | FB-IMPL-P4-PO fix-burst-4 | 2026-05-20 | product-owner | Closes pass-4 findings F-LP4-HIGH-001 (URL re-grounding), F-LP4-HIGH-002 (fixture-JSON parity mechanism), F-LP4-HIGH-003 (E-SPEC-017 enforcement — see BC-2.16.001 v1.5), F-LP4-HIGH-004 (auth_type swap), F-LP4-MED-002 (RG-09 test driver clarification in test vector note), F-LP4-MED-003 (request_count fragility — relaxed to >= 2). F-LP4-HIGH-001: all sensor URL paths re-grounded against DTU clone route registrations per ADR-028 §D1 (CrowdStrike: `/detects/queries/detects/v1` + `/detects/entities/summaries/GET/v1`; devices: `/devices/queries/devices/v1` + `/devices/entities/devices/v2`; Cyberint alerts: `/api/v1/alerts`; Claroty alerts: `/api/v1/alerts`). F-LP4-HIGH-002: §Postconditions §2 step 4 rewritten — reference OCSF loaded from committed fixture JSON at `crates/prism-dtu-{sensor}/fixtures/parity/reference-ocsf/<table>.json`; comparison is byte-identical after canonical JSON serialization; no `prism-sensors` dev-dep; ADR-028 §D3 cited. F-LP4-HIGH-003: F-LP4-MED-002 closure: `SpecLoader::parse` lacks filename context; filename-stem validation requires `SpecLoader::load_all()` or `parse_spec_directory()`; noted in Canonical Test Vector row. F-LP4-HIGH-004: auth_type corrected to DTU-grounded values per ADR-028 §D2 — claroty=`bearer_static` (was `cookie_roundtrip`), cyberint=`cookie_roundtrip` (was `bearer_static`), armis=`bearer_static` (was `api_key`), crowdstrike=`oauth2_client_credentials` (unchanged). §Known Gaps section added with DTU-EXT-001..004 for orchestrator follow-up (CrowdStrike incidents, Claroty assets, Armis devices via AQL, Armis alerts via AQL). ADR-028 §D1/D2/D3/D5 cited in §inputs, §Architecture Anchors. Legacy adapter source files removed from §inputs — per ADR-028 §D4, adapter code is NOT a grounding reference. |
| 1.3 | FB-IMPL-P3-PO fix-burst-3 | 2026-05-20 | product-owner | Closes pass-3 findings F-LP3-CRIT-001, F-LP3-CRIT-002, F-LP3-CRIT-003, F-LP3-HIGH-001, F-LP3-HIGH-002. F-LP3-CRIT-001: replaced phantom `spec_parser::parse_spec_file()` with `SpecLoader::parse(toml_input: &str)` in §Postconditions §2 step 2 and §Canonical Test Vectors (CODE-GROUNDED: spec_parser.rs:655). F-LP3-CRIT-002: corrected all CrowdStrike URL paths — `/detects/queries/detects/v1` etc. replaced with actual patterns from crowdstrike.rs:262,315: `/queries/{resource_type}` (QueryV2) and `/entities/{resource_type}/GET` (PostEntities); incidents table corrected to two-step (same pattern); URL derivation via `resource_type_from_spec()` (crowdstrike.rs:369-375) documented. F-LP3-CRIT-003: stripped `/xdome` prefix from all Claroty endpoints — actual pattern is `/api/v1/{resource}s` (claroty.rs:244); `/xdome` was never present in the code. F-LP3-HIGH-001: removed `/v1` segment from Cyberint endpoints — actual pattern is `/api/{resource}s` (cyberint.rs:251); no `/v1` in Cyberint URL construction. F-LP3-HIGH-002: corrected Armis endpoint — single `/api/v1/search` (no trailing slash, armis.rs:517) used for ALL queries including both `devices` and `alerts`; AQL discriminator `in:devices` / `in:alerts` via `DEFAULT_AQL_TEMPLATE` (armis.rs:72) documented; phantom per-resource endpoint paths removed. |
| 1.2 | FB-IMPL-P2-PO fix-burst-2 | 2026-05-20 | product-owner | Closes pass-2 findings F-001, F-002, F-003, F-004, F-005. F-001: swapped auth_type strings in §Postconditions §1 — claroty=`cookie_roundtrip` (was `bearer_static`), cyberint=`bearer_static` (was `cookie_roundtrip`), matching `ClarotyAuth::auth_type_name()` = `"cookie_roundtrip"` and `CyberintAuth::auth_type_name()` = `"bearer_static"` per code-grounded verification of `crates/prism-sensors/src/auth/{claroty,cyberint}.rs`. F-002: corrected §Error Conditions — E-SPEC-009 row now accurately describes ONLY duplicate-sensor_id (not filename-stem mismatch); added E-SPEC-017 row for filename-stem-vs-sensor_id mismatch (newly registered in error-taxonomy.md v1.41 per POL-1 append-only). F-003: replaced phantom `CrowdStrikeAdapter::fetch_page()` (non-existent) with actual `SensorAdapter::fetch()` trait method in §Postconditions §1 and §2 (CODE-GROUNDED: `crowdstrike.rs` has `fetch()` at trait impl, no `fetch_page()`; all 4 sensors use the same `SensorAdapter::fetch()` entry point). F-004: corrected `${query.aql}` → `${query.filter.aql}` in §Canonical Test Vectors Armis AQL forwarding row. F-005 (TD-VSDD-091): replaced `spec_parser.rs:128` → `FetchStep::fan_out_batch_size field` and `pipeline.rs:246-250` → `PipelineExecutor::execute_impl query.filter.{k} step-vars seeding` in §Preconditions O-001 table. |
| 1.1 | FB-IMPL-P1-PO fix-burst-1 | 2026-05-20 | product-owner | Closes pass-1 adversarial findings F-001/F-002/F-004/F-006/F-007/O-001. F-001: replaced fabricated `prism_dtu_{sensor}::server::spawn()` / `DtuHandle` API with actual `BehavioralClone::start_on(bind, shutdown, tls) -> anyhow::Result<SocketAddr>` trait (all 4 clones share via `prism_dtu_common::BehavioralClone`). F-002: replaced fabricated `PipelineExecutor::execute(spec, "<table_name>", &NullAuthProvider, ...)` with actual 5-arg signature `(spec: &SensorSpec, table: &TableSpec, context: &FetchContext, http_client: &reqwest::Client, auth_provider: &dyn AuthProvider) -> Result<PipelineResult, SpecEngineError>`. F-004: retired fabricated `E-SPEC-015` (parity FAIL is a test verdict, not a runtime error code) and replaced fabricated `E-SPEC-016` with `E-SPEC-009` (existing code already covers sensor_id/filename mismatch). F-006: corrected `ADR-023 §Rule 1` / `§Rule 3` phantom anchors to `ADR-023 §Decision Rules — Rule 1` / `§Decision Rules — Rule 3`. F-007: corrected `ADR-022 §C2` phantom anchor (C2 is in ADR-023, not ADR-022) to `ADR-023 §Architectural Constraints — C2`. O-001: added grammar verification table in §Preconditions confirming `fan_out_batch_size` SUPPORTED, `${query.filter.aql}` SUPPORTED (not `${query.aql}`), `timestamp_format = "multi"` NOT SUPPORTED, `timestamp_fallback_chain` NOT SUPPORTED — grammar extension or WASM plugin required as implementer prerequisite. Postconditions updated to reflect grammar gaps. |
| 1.0 | D-731 PLUGIN-MIGRATION-001-D PO authoring | 2026-05-20 | product-owner | Initial draft — BC anchor for PLUGIN-MIGRATION-001-D; DTU-parity contract for VP-PLUGIN-003; authored from ADR-023 §Rule 3 + TS-PLUGIN-PARITY-001 + 4 sensor adapter source surveys |
