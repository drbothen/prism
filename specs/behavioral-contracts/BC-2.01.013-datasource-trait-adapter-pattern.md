---
document_type: behavioral-contract
level: L3
version: "1.10"
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
scheduled_amendment_in: ADR-023
amendment_lifecycle: pending
introduced: cycle-1
modified: "2026-06-03"  # v1.10 OCSF-CLASS-MIGRATION-001 Wave-5 Phase-A PO burst
amendment_burst: D-925-arch-adjudication
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

> **PENDING AMENDMENT — ADR-023**: The sealed-trait enforcement of `SensorAuth` described in
> earlier versions of this BC is superseded by spec-driven runtime validation. The `SensorAuth`
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

  **Conformance boundary — test-writer probe:** A test that constructs a `SpecDrivenSensorAdapter` with a spec using `ocsf_class = "security_finding"` (a REAL class-name from an actual sensor spec), drives it against a mock `PipelineExecutor` returning a raw API response containing `"class_uid": 9999`, and then asserts that (a) all spec-declared column names appear in the returned Arrow schema, (b) `class_uid` in the returned batch equals `2001` (the `select_by_class_name("security_finding")` result) and NOT `9999`, and (c) `_sensor` equals the canonical sensor ID — such a test is the minimum conformance gate for this clause. The test fixture MUST NOT use a fake record-type token like `"detection"` as the `ocsf_class` value; it MUST use a real class-name from the production sensor TOML files.

- **SpecDrivenSensorAdapter Pagination / Push-Down Scope Clause (D-924, S-DEMO-001 F-003-R scope-out):**
  `SpecDrivenSensorAdapter::fetch()` returns ALL pages for the requested table, iterating until the sensor API signals exhaustion (empty page or null cursor), subject ONLY to the pipeline's internal caps: `MAX_PAGES_PER_STEP` and `MAX_REQUESTS_PER_PIPELINE` (defined in `PipelineExecutor`). Pagination is handled entirely inside `PipelineExecutor::execute()`; the adapter does not impose additional page limits.

  **Query-param push-down is EXPLICITLY OUT OF SCOPE for this contract.** The adapter does not translate `limit`, `cursor`, or `time_window` values from the query caller's parameters into sensor-native API request parameters. DataFusion applies `LIMIT` predicates and post-filters over the fully materialized Arrow RecordBatch after `fetch()` returns; correctness is preserved because push-down is an optimization, not a correctness requirement (consistent with BC-2.11.007 invariant: "push-down is an optimization only; the query result must be identical whether or not push-down occurs"). This is a deferred feature, tracked as follow-up story `S-DEMO-QUERY-PUSHDOWN-001` (to be created by story-writer). This scope-out is an explicit, documented deferral of an entire feature to a later story — not a silent shortcut — per CLAUDE.md Canonical Principle (deferring a complete feature to a later story is permitted when documented).

  **Implication for test-writer:** Tests for this contract MUST NOT assert that `fetch()` passes `limit` or cursor values to the sensor API. Tests that assert push-down absence (i.e., the pipeline receives no `limit` parameter from the adapter layer) are positive conformance tests.

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
| EC-01-027 | Caller passes a `limit` value to `SpecDrivenSensorAdapter::fetch()` expecting the sensor API request to carry that limit as a query param | Push-down is out of scope per Pagination/Push-Down Scope Clause; DataFusion applies the LIMIT post-materialization; the adapter must not translate limit into the API request params |

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.01.013-001 | CrowdStrike adapter implementing `DataSource<CrowdStrikeAlert>` | `fetch_page()` delegates to adapter; shared infrastructure manages cursor; adapter code has no cursor logic |
| TV-BC-2.01.013-002 | SensorSpec with `auth_type: [oauth2_client_credentials, bearer_static]` | Spec-load rejected with E-SPEC-010; Rule 1 cited in error |
| TV-BC-2.01.013-003 | Adapter returns unrecognized API response structure | `PrismError::Sensor` with sensor name, source, raw response snippet |
| TV-BC-2.01.013-004 | `SpecDrivenSensorAdapter::fetch()` called against a mock PipelineExecutor returning a raw record with 2+ spec-declared columns; spec uses `ocsf_class = "security_finding"` (a REAL class-name from a production sensor spec) | Returned RecordBatch Arrow schema contains all spec-declared column names PLUS `category_uid`, `class_uid`, `_sensor`; no spec-declared column is absent (OCSF Conformance Clause item 1) |
| TV-BC-2.01.013-005 | Spec declares `ocsf_class = "security_finding"`; raw vendor record contains `"class_uid": 9999, "category_uid": 9999` — `SpecDrivenSensorAdapter::fetch()` called | `class_uid` in returned RecordBatch equals `2004` (from `EventClassSelector::select_by_class_name("security_finding")` → transitional alias returns 2004 per BC-2.02.012 v1.4 Option A), NOT `9999` and NOT `2001`. `category_uid` equals `2`. `event_type = "ocsf.deprecated_class_alias"` WARN is emitted. After OCSF-CLASS-MIGRATION-001 merges, production TOML specs will use `"detection_finding"` directly; this test vector covers the alias path for external TOML compatibility. (OCSF Conformance Clause item 2, D-925; updated per OCSF-CLASS-MIGRATION-001 Wave-5 Phase-A) |
| TV-BC-2.01.013-006 | `SpecDrivenSensorAdapter::fetch()` called; pipeline returns 3 pages before API signals exhaustion | All 3 pages concatenated into returned `Vec<RecordBatch>`; no early termination due to limit parameter (Pagination/Push-Down Scope Clause) |

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

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
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
