---
document_type: behavioral-contract
level: L3
version: "1.16"
status: active
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-01"
capability: "CAP-001"
lifecycle_status: active
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "76729b7"
traces_to: ["CAP-001"]
extracted_from: ".factory/specs/prd.md"
scheduled_amendment_in: null
amendment_lifecycle: null
introduced: cycle-1
modified: "2026-07-24"
amendment_burst: S-DEMO-QUERY-PUSHDOWN-001-v2-armis-aql-full-wiring
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.01.013: DataSource Trait Eliminates Per-Sensor Code Duplication

## Description

The `DataSource<T>` generic trait provides a uniform interface for all sensor data feeds, with
shared infrastructure handling cursor management, forward-progress enforcement, and page assembly.
Each adapter implements only sensor-specific concerns: API call construction, response
deserialization, and field extraction. Adapter implementations are produced from TOML SensorSpec
declarations at runtime; runtime validation (not compile-time sealing) prevents cross-sensor auth
composition per the three rules in ADR-023 Rule 2. Record types follow the `<sensor>_<entity>`
naming convention (e.g., `crowdstrike_alert`, `armis_device`).

> **Amendment — ADR-023 (PREREQ-F, v1.4, 2026-05-11):** The sealed-trait enforcement of `SensorAuth` described in
> earlier versions of this BC was superseded in v1.4 (PREREQ-F) by spec-driven runtime validation. The `SensorAuth`
> trait is no longer sealed. Cross-sensor auth-composition prevention is enforced at spec-load
> time via three runtime rejection rules (see Rule 2 of ADR-023 and the amended DI-012).

## Preconditions
- A sensor adapter implements the `SensorAdapter` trait and one or more `DataSource<T>` implementations
- Each `DataSource<T>` corresponds to a single sensor data feed (e.g., `crowdstrike_alert`, `claroty_device`)
- A valid TOML SensorSpec declaration exists for the sensor, specifying a single `auth_type` value

## Postconditions
- The generic `DataSource<T>` trait provides `fetch_page()` and `cursor_from_record()` methods
- All pagination logic (cursor management, forward-progress enforcement, page assembly) is handled by shared infrastructure, not per-adapter code
- Each adapter only implements sensor-specific concerns: API call construction, response deserialization, field extraction
- Adapter implementations are produced from TOML SensorSpec declarations at runtime; no hand-written adapter code outside `prism-sensors` is required for TOML-expressible sensors
- `record_type` follows the `<sensor>_<entity>` naming convention (e.g., `crowdstrike_alert`, `armis_device`)
- Cross-sensor auth-composition is prevented by three runtime validation rules enforced at spec-load time (ADR-023 Rule 2):
  1. `SensorSpec.auth_type` accepts exactly one value from the enumerated set; arrays or mixed types are rejected at spec-load
  2. Each auth method declares exactly one `credential_ref` binding; multiple credential bindings per auth method are rejected at spec-load
  3. The credential record schema must structurally match the declared `auth_type`; mismatches are rejected at spec-load
- **Adapter Identity Method (SensorAdapter::sensor_type → SensorId, S-PLUGIN-PREREQ-A):**
  The trait method `SensorAdapter::sensor_type(&self) -> SensorId` is the canonical adapter identity accessor. The method name `sensor_type` is preserved through the S-PLUGIN-PREREQ-A keystone migration (despite the type rename `SensorType` enum → `SensorId(Arc<str>)` newtype) to maintain caller-side idiomatic readability: `let sensor_id = adapter.sensor_type()`. Implementations construct the SensorId from the canonical lowercase sensor name (e.g., `SensorId::from("crowdstrike")`). The trait method is sealed-by-convention: implementations live in `prism-sensors` or are plugin-generated via PluginRuntime; direct user-code impls are discouraged but not enforced at the type level (DI-012 v1.6 amendment downgraded sealed-supertrait to runtime validation).
- **SpecDrivenSensorAdapter OCSF Conformance Clause (D-924, S-DEMO-001 F-001-R closure; amended D-925-arch-adjudication):**
  `SpecDrivenSensorAdapter::fetch()` MUST return OCSF-normalized Arrow RecordBatches in which:
  1. Every column declared in the sensor's TOML `[[tables.columns]]` spec (hereafter "spec-declared data columns") MUST survive into the returned RecordBatch via `ColumnMapper` field-by-field mapping. Dropping spec-declared data columns — emitting a RecordBatch that contains only OCSF envelope fields while discarding the actual sensor payload — is NON-CONFORMANT.
  2. The OCSF envelope fields `category_uid` and `class_uid` MUST be derived from the table's `ocsf_class` class-name string (e.g., `"security_finding"`, `"device"`, `"audit_activity"`, `"incident_finding"`) via `EventClassSelector::select_by_class_name(class_name) -> Result<u32, PrismError>`. This function MUST be added to `class_selector.rs` in `prism-ocsf` as a compile-time constant table mapping OCSF class-name strings to their canonical `class_uid` values. `class_uid / 1000` yields `category_uid`. The implementation MUST NOT call `EventClassSelector::select(sensor_id, &table.ocsf_class)` with an `ocsf_class` class-name as the `record_type` argument — `select()` is keyed on record-type tokens (`"detection"`, `"alert"`, `"device"`, `"audit_log"`), which are a DIFFERENT namespace from class-name strings. An implementation that calls `select(sensor_id, class_name_string)` will yield `class_uid = 0` (BASE_EVENT) for every real sensor table except those coincidentally matching a record-type token (only `"device"` coincides). An implementation that copies `category_uid`/`class_uid` directly from the raw vendor JSON is also NON-CONFORMANT.
  3. The `_sensor` virtual column MUST be present and set to the sensor's canonical `SensorId` string (e.g., `"crowdstrike"`), injected as a virtual column by the normalization layer (consistent with BC-2.11.005 postcondition — virtual fields injected by the engine). The raw record's `_sensor` field (if any) MUST be ignored — the spec `sensor_id` is the authoritative value.

  **`EventClassSelector::select_by_class_name` specification (must-build, D-925):**
  The implementer MUST add this function to `crates/prism-ocsf/src/class_selector.rs`:
  ```rust
  /// Returns the OCSF `class_uid` for the given OCSF class-name string.
  ///
  /// `class_name` is the snake_case OCSF class identifier as declared in TOML
  /// sensor specs (e.g., `"security_finding"`, `"detection_finding"`, `"device"`,
  /// `"audit_activity"`, `"incident_finding"`).
  ///
  /// This is DISTINCT from `EventClassSelector::select(sensor, record_type)` which
  /// maps vendor record-type tokens; this function maps OCSF schema class names.
  pub fn select_by_class_name(class_name: &str) -> Result<u32, PrismError> { ... }
  ```
  The mapping table MUST include at minimum the class names used by the four production
  sensor specs (verified against `crates/prism-sensors/specs/*.sensor.toml`):

  | `ocsf_class` value (TOML) | `class_uid` | Notes |
  |--------------------------|-------------|-------|
  | `"detection_finding"` | 2004 | OCSF v1.1 canonical; PRIMARY. Production sensor TOMLs use this post-OCSF-CLASS-MIGRATION-001. |
  | `"security_finding"` | 2004 | Transitional alias (Option A per BC-2.02.012 v1.4). Maps to 2004 (NOT 2001) with deprecation WARN emission `event_type = "ocsf.deprecated_class_alias"`. External TOML specs not under Prism control may use this string. Production sensor TOMLs MUST use `"detection_finding"` after OCSF-CLASS-MIGRATION-001 merges. |
  | `"incident_finding"` | 2005 | CrowdStrike incidents, Cyberint incidents |
  | `"vulnerability_finding"` | 2002 | Claroty vulnerabilities |
  | `"device"` | 5001 | Claroty/Armis devices (InventoryInfo) |
  | `"audit_activity"` | 3001 | Claroty/Armis audit logs (AccountChange — closest OCSF v1.7.0 class) |

  Returns `Err(PrismError::OcsfUnknownEventClass { sensor: "".into(), record_type: class_name.to_owned() })` for unmapped class names. The caller (`pipeline_result_to_record_batch`) falls back to `class_uid = 0` on `Err`, identical to the existing `unwrap_or(0)` behavior. This fallback is acceptable for tables whose `ocsf_class` is not in the mapping table.

  **Conformance boundary — test-writer probe:** A test that constructs a `SpecDrivenSensorAdapter` with a spec using `ocsf_class = "security_finding"` (a REAL class-name from an actual sensor spec), drives it against a mock `PipelineExecutor` returning a raw API response containing `"class_uid": 9999`, and then asserts that (a) all spec-declared column names appear in the returned Arrow schema, (b) `class_uid` in the returned batch equals `2004` (the `select_by_class_name("security_finding")` result — transitional alias returns 2004 per BC-2.02.012 v1.4 Option A, NOT 2001) and NOT `9999`, and (c) `_sensor` equals the canonical sensor ID — such a test is the minimum conformance gate for this clause. The test fixture MUST NOT use a fake record-type token like `"detection"` as the `ocsf_class` value; it MUST use a real class-name from the production sensor TOML files.

- **SpecDrivenSensorAdapter Pagination / Push-Down Scope Clause (D-924 initial scope-out; superseded by S-DEMO-QUERY-PUSHDOWN-001 approved story v1.1 — F-PUSHDOWN-008):**
  `SpecDrivenSensorAdapter::fetch()` returns ALL pages for the requested table, iterating until the sensor API signals exhaustion (empty page or null cursor), subject ONLY to the pipeline's internal caps: `MAX_PAGES_PER_STEP` and `MAX_REQUESTS_PER_PIPELINE` (defined in `PipelineExecutor`). Pagination is handled entirely inside `PipelineExecutor::execute()`; the adapter does not impose additional page limits.

  **Query-param push-down IS performed by `SpecDrivenSensorAdapter::fetch()` as of story S-DEMO-QUERY-PUSHDOWN-001 (approved v1.1, CLAUDE.md Source-of-Truth Precedence Rule 1 — the more-specific approved story supersedes this BC's earlier scope-out on implementation scope).** The adapter translates `limit`, `cursor`, `start_time`, and `end_time` values from the query caller's `FetchContext` into sensor-native API request parameters via `build_request()` (module-level free function in `crates/prism-spec-engine/src/pipeline.rs`). Push-down is applied on the first / query-plan pipeline step only; hydration and entity-fetch steps receive `FetchContext::default()` (all fields `None`) and do not push down. Unsupported params for a given sensor are silently ignored (never sent as API query strings). DataFusion still applies `LIMIT` predicates and post-filters over the materialized Arrow RecordBatch as a correctness backstop, consistent with BC-2.11.007 invariant: "push-down is an optimization only; the query result must be identical whether or not push-down occurs."

  **Per-sensor push-down translation (corrected in v1.13 per pushdown-redesign.md §6 + §1, ADR-033):**

  > **v1.12 table SUPERSEDED (2026-06-05, S-DEMO-QUERY-PUSHDOWN-001 v2 re-spec).** The v1.12 table claimed Cyberint POST-body `from_date`/`to_date`+`page_size`, Claroty POST-body `limit`/`offset`, and Armis `timeFrame`+`maxResults` as real push-down translations. These claims are FACTUALLY WRONG against production DTU structs and have been corrected below. They are retained here append-only per POLICY 1. Superseded by: pushdown-redesign.md §6 + §1 mechanism table (architect design note, 2026-06-05); ADR-033 §Decision.

  | Sensor | Push-Down Mechanism | Dimensions Reachable in v2 | What is NOT available |
  |--------|--------------------|--------------------------|-----------------------|
  | CrowdStrike | `filter` FQL query param (Step 1 `query_detection_ids` only) + `limit` query param. `start_time` → FQL `created_timestamp:>'<ISO8601>'`; `end_time` → `created_timestamp:<'<ISO8601>'`; both combined with `+` when present. `start_time`/`end_time` are now populated by `run_materialization_pipeline` via ADR-033 Option T1 time-window extraction from the PrismQL AST. Step 2 (`fetch_detections`) receives `FetchContext::default()` — push-down does NOT apply to Step 2. | `limit` (query param), `filter` FQL time-window (start and/or end) | Cursor: none — `DetectionListParams` uses `offset`, not cursor-token. Devices table shares same mechanism. |
  | Armis | AQL-clause augmentation (BC-2.11.007 Mechanism B extended, v1.14). The pipeline appends a canonical Armis AQL time clause to the user's base AQL string when time-window bounds are extracted from the PrismQL WHERE clause via ADR-033 Option T1. The augmented AQL string is forwarded verbatim via `${query.filter.aql}` in the path_template: `GET /api/v1/search?aql=<augmented_value>`. Confirmed AQL time syntax (research-doc `armis-aql-time-window-syntax-2026-06.md`, HIGH confidence): bounded range `after:<ts> before:<ts>` (space-separated keywords; bare, unquoted, timezone-naive `YYYY-MM-DDTHH:MM:SS`); lower bound only `after:<ts>`; relative `timeFrame:"<N> <unit>"` (e.g. `timeFrame:"3 Hours"`). Anti-double-filter guard: if the base AQL already contains `after:`, `before:`, or `timeFrame:`, the string is forwarded verbatim without augmentation (user's explicit time scope is preserved). The prism-dtu-armis clone MUST honor the AQL time clause by filtering its dataset on `last_seen` (devices) / `created_at` (alerts), making push-down scenarios load-bearing (§8.3 of pushdown-redesign.md). `offset`/`limit` pagination is handled by the existing OffsetLimit pipeline. | AQL passthrough (existing) + AQL-clause time-window augmentation (new, v2 scope) | Cursor: NONE — Armis `/api/v1/search` is OffsetLimit only, no cursor-token. `maxResults`: NONE — `SearchQueryParams` has no such field. |
  > **v1.13 Armis row SUPERSEDED (2026-06-05, S-DEMO-QUERY-PUSHDOWN-001 v2 Armis full-wiring directive).** The v1.13 row stated "AQL verbatim passthrough only; no time-window wiring." This was the correct assessment before the human directive "fully wire Armis AQL into our DTU and our scenarios" (2026-06-05). That statement is now superseded: Armis time-window push-down IS in scope via AQL-clause augmentation. Preserved append-only per POLICY 1. Superseded by: pushdown-redesign.md §8 (architect design note, 2026-06-05); research-doc `armis-aql-time-window-syntax-2026-06.md` (HIGH confidence AQL syntax); ADR-033 §Decision extended to cover Armis AQL augmentation path.
  | Cyberint | Cursor passthrough only. `AlertListParams` has exactly one field: `cursor: Option<String>`. No time-window push-down possible against the current DTU. `page_size`/`limit` are NOT available (DTU-EXT-005 open). | Cursor (existing pagination, not a new push-down dimension) | Time-window: NONE — `AlertListParams` has no `from_date`/`to_date`/`start_time`/`end_time`. The endpoint is GET (no body_template); the v1.12 POST-body injection claim was WRONG. `page_size`: NONE — deferred to DTU-EXT-005. |
  | Claroty | OffsetLimit URL params (`?offset=N&limit=M`) via the existing OffsetLimit pipeline — this is not a new push-down dimension but correct existing behavior. `body_template: '{}'` is always an empty object; no body-based push-down fields are injected. | OffsetLimit pagination (existing) | Time-window: NONE — no native time-window param in any Claroty DTU route struct. Body-based offset/limit: deferred to `S-DEMO-CLAROTY-PAGINATION-001` (Gap-CL-004). The v1.12 POST-body `limit`/`offset` claim was WRONG. |

  **`FetchContext` push-down fields (additive, introduced in S-DEMO-QUERY-PUSHDOWN-001):**
  `FetchContext` gains four optional fields: `cursor: Option<String>`, `limit: Option<u32>`, `start_time: Option<DateTime<Utc>>`, `end_time: Option<DateTime<Utc>>`. `FetchContext::default()` sets all four to `None`; all existing callers are unaffected.

  **Implication for test-writer:** Tests for this contract that exercise the first/query-plan pipeline step SHOULD assert that `fetch()` passes the expected `limit` and time-window values to the sensor API request. Tests for hydration or entity-fetch steps MUST assert that no push-down params appear in the API request (those steps receive `FetchContext::default()`). Tests that assert push-down ABSENCE on a query-plan step are NON-CONFORMANT with this contract as of S-DEMO-QUERY-PUSHDOWN-001. See BC-2.11.007 for the result-equivalence invariant that must be verified alongside push-down.

## Invariants
- Each `DataSource<T>` produces records of a single type
- The `SensorAuth` trait is NOT sealed — it is open for plugin implementations (ADR-023 Rule 2). Cross-sensor auth-composition is prevented by three runtime rejection rules (see Postconditions), not by compile-time sealed-supertrait enforcement (DI-012 amended)

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-SPEC-010` | SensorSpec declares multiple auth types | Rejected at spec-load with error citing Rule 1 |
| `E-SPEC-011` | Auth method has multiple credential_ref bindings | Rejected at spec-load with error citing Rule 2 |
| `E-SPEC-012` | Credential schema does not structurally match declared auth_type | Rejected at credential-resolution time with error citing Rule 3 |
| `PrismError::Sensor` | Adapter's `fetch_page()` encounters an unrecognized API response structure | Structured error with the sensor name, source, and raw response snippet for debugging |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-01-020 | A sensor API adds a new data source type not yet implemented | The new source is ignored; `list_capabilities` shows only implemented sources |
| EC-01-021 | Adapter bound to one (Client, Sensor) pair is accidentally shared | Type system prevents this: `SensorAdapter` requires `tenant_id()` returning the bound `TenantId` |
| EC-01-022 | SensorSpec declares `auth_type: [oauth2_client_credentials, bearer_static]` (array) | Rejected at spec-load with E-SPEC-010 citing Rule 1 (single auth_type required) |
| EC-01-023 | SensorSpec auth method has two credential_ref entries | Rejected at spec-load with E-SPEC-011 citing Rule 2 (single credential_ref per method) |
| EC-01-024 | SensorSpec declares `auth_type: oauth2_client_credentials` but credential is a cookie record | Rejected at credential-resolution time with E-SPEC-012 citing Rule 3 (structural mismatch) |
| EC-01-025 | `SpecDrivenSensorAdapter::fetch()` returns a RecordBatch containing only `category_uid`, `class_uid`, and `_sensor` (no spec-declared data columns) | NON-CONFORMANT per OCSF Conformance Clause; test-writer must assert this as a failure case; the ColumnMapper step is missing |
| EC-01-026 | Raw vendor JSON contains a field named `category_uid` — implementation copies it verbatim into Arrow output | NON-CONFORMANT per OCSF Conformance Clause item 2; `category_uid` must be derived by `EventClassSelector::select_by_class_name(table.ocsf_class)` (the class-name→uid function in `class_selector.rs`), not read from raw response and not by calling `select(sensor_id, ocsf_class_string)` |
| EC-01-028 | Implementation calls `EventClassSelector::select(sensor_id, &table.ocsf_class)` passing the `ocsf_class` class-name string (`"security_finding"`, etc.) as the `record_type` argument | NON-CONFORMANT: `select()` maps record-type tokens, not class-name strings. This yields `class_uid = 0` (BASE_EVENT) for every real sensor table except coincidental matches. Must use `EventClassSelector::select_by_class_name(&table.ocsf_class)` instead. (D-925 arch-adjudication) |
| EC-01-027 | `FetchContext` with `limit = Some(N)` is passed on a hydration or entity-fetch pipeline step (not the first / query-plan step) | Push-down is restricted to the first / query-plan pipeline step per F-PUSHDOWN-001 invariant (S-DEMO-QUERY-PUSHDOWN-001). The hydration and entity-fetch steps receive `FetchContext::default()` (all fields `None`); they MUST NOT propagate `limit`, `cursor`, or time-window values to the sensor API request even if those fields are set on the caller's FetchContext. DataFusion applies the LIMIT post-materialization as the correctness backstop. For CrowdStrike specifically, push-down applies to Step 1 (`query_detection_ids`) only — Step 2 (`fetch_detections`) receives `FetchContext::default()` regardless. (v1.13 amendment: added CrowdStrike two-step clarification per pushdown-redesign.md §1.1; supersedes v1.8 assertion of push-down absence on ALL steps, narrowed to hydration/entity-fetch steps only per S-DEMO-QUERY-PUSHDOWN-001, further narrowed to Step-2-only for CrowdStrike in v1.13.) |

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.01.013-001 | CrowdStrike adapter implementing `DataSource<CrowdStrikeAlert>` | `fetch_page()` delegates to adapter; shared infrastructure manages cursor; adapter code has no cursor logic |
| TV-BC-2.01.013-002 | SensorSpec with `auth_type: [oauth2_client_credentials, bearer_static]` | Spec-load rejected with E-SPEC-010; Rule 1 cited in error |
| TV-BC-2.01.013-003 | Adapter returns unrecognized API response structure | `PrismError::Sensor` with sensor name, source, raw response snippet |
| TV-BC-2.01.013-004 | `SpecDrivenSensorAdapter::fetch()` called against a mock PipelineExecutor returning a raw record with 2+ spec-declared columns; spec uses `ocsf_class = "security_finding"` (a REAL class-name from a production sensor spec) | Returned RecordBatch Arrow schema contains all spec-declared column names PLUS `category_uid`, `class_uid`, `_sensor`; no spec-declared column is absent (OCSF Conformance Clause item 1) |
| TV-BC-2.01.013-005 | Spec declares `ocsf_class = "security_finding"`; raw vendor record contains `"class_uid": 9999, "category_uid": 9999` — `SpecDrivenSensorAdapter::fetch()` called | `class_uid` in returned RecordBatch equals `2004` (from `EventClassSelector::select_by_class_name("security_finding")` → transitional alias returns 2004 per BC-2.02.012 v1.4 Option A), NOT `9999` and NOT `2001`. `category_uid` equals `2`. `event_type = "ocsf.deprecated_class_alias"` WARN is emitted. After OCSF-CLASS-MIGRATION-001 merges, production TOML specs will use `"detection_finding"` directly; this test vector covers the alias path for external TOML compatibility. (OCSF Conformance Clause item 2, D-925; updated per OCSF-CLASS-MIGRATION-001 Wave-5 Phase-A) |
| TV-BC-2.01.013-006 | `run_materialization_pipeline` executes a PrismQL query with `WHERE created_timestamp > '2026-01-01T00:00:00Z' AND created_timestamp < '2026-06-01T00:00:00Z' LIMIT 50` against a CrowdStrike sensor; `start_time` and `end_time` are extracted from the AST by ADR-033 Option T1 heuristic and populated into `QueryParams`. | (a) `run_materialization_pipeline` populates `QueryParams.start_time = Some("2026-01-01T00:00:00Z")` and `QueryParams.end_time = Some("2026-06-01T00:00:00Z")` before fan-out; (b) the CrowdStrike DTU receives `DetectionListParams.filter` containing `created_timestamp:>'2026-01-01T00:00:00Z'+created_timestamp:<'2026-06-01T00:00:00Z'`; (c) `DetectionListParams.limit = Some(50)` is present; (d) Step 2 (`fetch_detections` POST) receives NO `filter` or `limit` params — `FetchContext::default()`. Wiring occurs via `run_materialization_pipeline` (NOT by constructing `FetchContext` directly at a callsite). DataFusion applies LIMIT post-materialization as correctness backstop (BC-2.11.007 result-equivalence invariant). Test MUST use production `crowdstrike.sensor.toml` spec shape (GET step, no body_template) per SAP-2. (v1.13 re-cast: asserts both start_time AND end_time reach the FQL filter, and asserts wiring occurs via `run_materialization_pipeline` per ADR-033; supersedes v1.12 which only asserted start_time. Prior v1.8 asserted push-down absence as positive conformance — that is now obsolete. Append-only: v1.8 assertion was: "No early termination due to limit; FetchContext push-down absent on all steps.") |
| TV-BC-2.01.013-007 | PrismQL query `SELECT * FROM armis_devices WHERE aql = 'in:devices' AND last_seen > '2026-01-01T00:00:00'`; `last_seen` is declared `column_type = "datetime"` + `options = ["INDEX"]` in `armis.sensor.toml`; ADR-033 T1 heuristic extracts `start_time = Some("2026-01-01T00:00:00")`. AQL-clause augmentation appends `after:2026-01-01T00:00:00` to the base AQL string. | (a) `QueryParams.filters["aql"]` = `"in:devices after:2026-01-01T00:00:00"` (base AQL + time clause, bare unquoted timezone-naive ISO8601 per research-doc); (b) Armis DTU receives `GET /api/v1/search?aql=in:devices+after:2026-01-01T00:00:00`; (c) DTU returns a PROPER SUBSET of the unfiltered device fixture (filtered_count < unfiltered_count — load-bearing assertion per §8.3 of pushdown-redesign.md); (d) DataFusion post-filter on `last_seen > '2026-01-01T00:00:00'` also applies as correctness backstop (result-equivalence invariant). Test MUST use production `armis.sensor.toml` spec shape per SAP-2. (v1.14 new TV) |
| TV-BC-2.01.013-008 | PrismQL query `WHERE aql = 'in:devices after:2024-01-01T00:00:00' AND last_seen > '2026-01-01T00:00:00'` — base AQL already contains `after:` keyword; anti-double-filter guard fires. | `QueryParams.filters["aql"]` = `"in:devices after:2024-01-01T00:00:00"` — base AQL forwarded verbatim, no second `after:` clause appended. The guard detects that the base AQL already contains the `after:` keyword and passes through unchanged. (v1.14 new TV — anti-double-filter guard) |

## Verification Properties

| VP | Verification Aspect |
|----|---------------------|
| VP-PLUGIN-006 | OCSF column mapping fixture catalog verifying SpecDrivenMapper correctness |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 |
| Capability Anchor Justification | CAP-001 ("Enumerate and fetch data from sensor APIs") per capabilities.md §CAP-001 |
| L2 Invariants | DI-012 (amended — runtime enforcement replaces compile-time sealed trait per ADR-023 Rule 2) |
| Priority | P0 |
| Related ADRs | ADR-023 (SensorAuth open trait, runtime auth-composition prevention), ADR-033 (push-down time-window extraction strategy — pre-fan-out heuristic T1; covers both CrowdStrike FQL injection and Armis AQL-clause augmentation path) |
| Related Research | `.factory/research/armis-aql-time-window-syntax-2026-06.md` (HIGH confidence; canonical Armis AQL time-window syntax — `after:<ts>` / `before:<ts>` / `timeFrame:"<N> <unit>"` forms confirmed across 6 independent sources; `lastSeen:>"T"` form NOT confirmed; `"Last 3 Hours"` value-phrasing NOT confirmed) |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.16 | wave-a-spec-evolution-fix-burst-41 | 2026-07-24 | product-owner | POL-29 sweep (F-WASE-P52-LOW-001 companion): replaced phantom `PipelineExecutor::build_request()` qualifier in live-body §Pagination/Push-Down Scope Clause with `build_request()` (module-level free function in `crates/prism-spec-engine/src/pipeline.rs`). Changelog row at v1.12 retains original phrasing (exempt per changelog-row exemption). As-built verified: pipeline.rs:975 is a module-level free function, no `&self`. |
| 1.15 | wave-a-spec-evolution-fix-burst-38 | 2026-07-24 | product-owner | F-WASE-P49-LOW-001 sibling-sweep extension (adjudicated): ADR-023 amendment was substantively applied in v1.4 (PREREQ-F, 2026-05-11) — sealed-trait language removed, spec-driven runtime pattern installed; no PLUGIN-MIGRATION-001-G entry because this BC's amendment was front-loaded into PREREQ-F rather than Wave 2/G. Evidence: v1.4 changelog "removed sealed-trait language; replaced with spec-driven adapter pattern" is the full content of the ADR-023 amendment; body postconditions confirm spec-driven runtime validation is in place. Clearing `scheduled_amendment_in: null` and `amendment_lifecycle: null`. PENDING AMENDMENT banner converted to completion note (removed "PENDING", changed "is superseded" → "was superseded in v1.4 (PREREQ-F)"). `modified` inline comment (v1.14 reference) removed. |
| 1.14 | S-DEMO-QUERY-PUSHDOWN-001-v2-armis-aql-full-wiring | 2026-06-05 | product-owner | Human directive 2026-06-05: "fully wire Armis AQL into our DTU and our scenarios." Armis row in per-sensor push-down translation table updated: Armis time-window push-down IS in scope via AQL-clause augmentation (v1.13 "passthrough only; no time-window" text superseded, preserved append-only per POLICY 1). New Armis mechanism: query-engine appends canonical Armis AQL time clause (`after:<ts>` / `before:<ts>` / `timeFrame:"<N> <unit>"`) to user's base AQL string; combined AQL forwarded via existing `${query.filter.aql}` path; prism-dtu-armis clone MUST honor time clause by filtering dataset (§8.3 of pushdown-redesign.md). Confirmed AQL syntax from research-doc `armis-aql-time-window-syntax-2026-06.md` (HIGH confidence): bare unquoted timezone-naive `YYYY-MM-DDTHH:MM:SS` timestamps; space-separated `after:` / `before:` keywords; `timeFrame:"<N> <unit>"` for relative. Anti-double-filter guard specified: if base AQL already contains `after:`, `before:`, or `timeFrame:`, no augmentation. Added TV-BC-2.01.013-007 (Armis bounded-range augmentation; load-bearing assertion: filtered_count < unfiltered_count). Added TV-BC-2.01.013-008 (anti-double-filter guard). Added research-doc to Traceability §Related Research. ADR-033 traceability note updated to cover Armis augmentation path. |
| 1.13 | S-DEMO-QUERY-PUSHDOWN-001-v2-bc-respec | 2026-06-05 | product-owner | S-DEMO-QUERY-PUSHDOWN-001 v2 re-spec (LOCAL adversary passes 5/6 factual correction). Per-sensor push-down translation table corrected per pushdown-redesign.md §6+§1 and ADR-033 §Decision: (1) CrowdStrike — CORRECT direction preserved; table now makes explicit that both `start_time` AND `end_time` reach the FQL filter, and that wiring occurs via `run_materialization_pipeline` (ADR-033 Option T1 pre-fan-out heuristic), NOT via direct FetchContext construction; Step 2 (`fetch_detections`) receives no push-down. (2) Armis — REMOVED false `maxResults`/`timeFrame` claims; real mechanism is AQL verbatim passthrough only (BC-2.11.007 Mechanism B) with no separate time-window param; `SearchQueryParams` has no such fields. (3) Cyberint — REMOVED false POST-body `from_date`/`to_date`+`page_size` claims; real: GET endpoint, cursor-only (`AlertListParams.cursor`), no body_template, no time-window; `page_size` deferred to DTU-EXT-005. (4) Claroty — REMOVED false POST-body `limit`/`offset` claims; real: OffsetLimit URL params (existing behavior); body-based pagination deferred to `S-DEMO-CLAROTY-PAGINATION-001`; no time-window param. Superseded v1.12 table preserved append-only per POLICY 1 with rationale header. EC-01-027 updated: added CrowdStrike two-step clarification (Step 1 push-down only; Step 2 always `FetchContext::default()`). TV-BC-2.01.013-006 re-cast: now asserts BOTH `start_time` AND `end_time` reach the CrowdStrike FQL filter, and that wiring occurs via `run_materialization_pipeline` (not direct FetchContext construction at call site); SAP-2 production-TOML fixture mandate added to TV. ADR-033 added to Traceability §Related ADRs. |
| 1.12 | F-PUSHDOWN-008-po-fix-burst | 2026-06-05 | product-owner | F-PUSHDOWN-008 closure (LOCAL adversary pass 1, S-DEMO-QUERY-PUSHDOWN-001 fix-burst): amended Pagination/Push-Down Scope Clause — the prior "EXPLICITLY OUT OF SCOPE" language is superseded by approved story S-DEMO-QUERY-PUSHDOWN-001 v1.1 (CLAUDE.md Source-of-Truth Precedence Rule 1; the more-specific approved story supersedes BC on implementation scope). Push-down IS now performed by `SpecDrivenSensorAdapter::fetch()` on the first/query-plan pipeline step: `limit`, `cursor`, `start_time`, `end_time` are threaded via `FetchContext` into `PipelineExecutor::build_request()` with per-sensor translation (CrowdStrike FQL, Cyberint POST-body, Claroty POST-body, Armis AQL). Hydration and entity-fetch steps receive `FetchContext::default()` and do NOT push down (F-PUSHDOWN-001 invariant). Unsupported params silently ignored. BC-2.11.007 result-equivalence invariant preserved. EC-01-027 re-cast: prior "push-down absence on all steps" conformance → new "hydration/entity-fetch steps do not push down even when FetchContext fields are set." TV-BC-2.01.013-006 re-cast: prior "no early termination due to limit" → new "CrowdStrike request carries limit and time-window on first/query-plan step." BC v1.11 → v1.12. |
| 1.11 | Wave-5-Phase-B-gate-F-001 | 2026-06-03 | product-owner | F-001 consistency fix (D-989): corrected stale `2001` assertion in the conformance boundary test-writer probe paragraph (§SpecDrivenSensorAdapter OCSF Conformance Clause) — changed "equals `2001`" to "equals `2004` (transitional alias returns 2004 per BC-2.02.012 v1.4 Option A, NOT 2001)". The TV-BC-2.01.013-005 table row was already correct at v1.10; this fix makes the prose consistent with the table and with BC-2.02.012 v1.4. An implementer or test-writer reading the stale prose would have written a non-conformant assertion. BC v1.10 → v1.11. |
| 1.10 | Wave-5-Phase-A-PO-burst | 2026-06-03 | product-owner | OCSF-CLASS-MIGRATION-001 amendment: updated `select_by_class_name` mapping table — `"security_finding"` now maps to 2004 (NOT 2001) as a transitional alias per BC-2.02.012 v1.4 Option A decision; `"detection_finding"` confirmed as PRIMARY entry returning 2004; deprecation WARN `event_type = "ocsf.deprecated_class_alias"` specified for alias path. Updated TV-BC-2.01.013-005 to assert class_uid == 2004 (not 2001) for `"security_finding"` input. This removes the prior "use until migration to detection_finding" note now that the migration has been specified (OCSF-CLASS-MIGRATION-001). BC v1.9 → v1.10. |
| 1.9 | D-925-arch-adjudication | 2026-05-31 | architect | Architecture adjudication for S-DEMO-001 F-001-R-RECUR + F-DOC-001: resolved the ocsf_class namespace collision. TOML `ocsf_class` carries OCSF class-name strings (e.g., `"security_finding"`, `"device"`) — NOT record-type tokens (`"detection"`, `"alert"`). `EventClassSelector::select(sensor, record_type)` is keyed on record-type tokens only. Decision: add `EventClassSelector::select_by_class_name(class_name) -> Result<u32, PrismError>` to `crates/prism-ocsf/src/class_selector.rs`. `pipeline_result_to_record_batch` MUST call `select_by_class_name(&table.ocsf_class)` — NOT `select(sensor_id, &table.ocsf_class)`. Amended Conformance Clause item 2 to name this function explicitly; added class-name→uid mapping table with all production sensor ocsf_class values; corrected TV-BC-2.01.013-005 to use real `ocsf_class = "security_finding"` (not fake record-type token `"detection"`); added EC-01-028 for the wrong-function anti-pattern. `category_uid = class_uid / 1000` rule confirmed. |
| 1.8 | D-924-bc-amendment | 2026-05-31 | product-owner | S-DEMO-001 adversary pass-2 findings F-001-R and F-003-R closure: added SpecDrivenSensorAdapter OCSF Conformance Clause (items 1–3 — spec-declared data columns must survive via ColumnMapper; category_uid/class_uid must be derived by OcsfNormalizer not read from raw record; _sensor virtual column required); added SpecDrivenSensorAdapter Pagination/Push-Down Scope Clause (fetch returns ALL pages bounded only by MAX_PAGES_PER_STEP/MAX_REQUESTS_PER_PIPELINE; query-param push-down explicitly out of scope deferred to S-DEMO-QUERY-PUSHDOWN-001 per D-924); added EC-01-025/EC-01-026/EC-01-027; added TV-BC-2.01.013-004/005/006. Error taxonomy unchanged. |
| 1.7 | D-776-post-merge | 2026-05-22 | state-manager | POL-14 verification (no-op confirm): PR #153 (PLUGIN-MIGRATION-001-D) squash-merged to develop@3f2de889 at 2026-05-22T09:05:47Z; status already active (promoted draft→active D-398 per POL-14 PR #142) — idempotent confirm. |
| 1.6 | D-398-post-merge | 2026-05-11 | state-manager | Status promoted draft→active per POL-14 (anchor story S-PLUGIN-PREREQ-A merged at develop@90d7c80f via PR #142, squash-merged 2026-05-11T16:37:14Z). lifecycle_status was already active; status frontmatter now matches. |
| 1.5 | pass-6-closures | 2026-05-11 | product-owner | S-PLUGIN-PREREQ-A pass-6 closure (F-LP6-MED-002): added Adapter Identity Method postcondition block documenting SensorAdapter::sensor_type() → SensorId canonical adapter identity accessor, name-preservation rationale through S-PLUGIN-PREREQ-A keystone migration, SensorId construction convention, and sealed-by-convention provenance (DI-012 v1.6 amendment). Closes story anchor claim that BC body "drives the open dispatch requirement." |
| 1.4 | prereq-f | 2026-05-11 | product-owner | ADR-023 v1.17 PREREQ-F amendment: removed sealed-trait language; replaced with spec-driven adapter pattern where implementations are produced from TOML SensorSpec declarations at runtime; replaced compile-time SensorAuth sealing with three runtime cross-sensor auth-composition rejection rules per ADR-023 Rule 2; updated Error Cases, Edge Cases, Canonical Test Vectors, and Verification Properties accordingly. DI-012 reference updated to reflect amended runtime enforcement. |
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added inputs/input-hash/traces_to/extracted_from frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors; added ## Verification Properties; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract. |
