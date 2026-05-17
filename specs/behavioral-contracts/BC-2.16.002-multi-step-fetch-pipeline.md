---
document_type: behavioral-contract
level: L3
version: "1.24"
status: active
producer: product-owner
timestamp: 2026-05-16T14:00:00Z
phase: 1a
origin: greenfield
subsystem: "SS-16"
capability: "CAP-029"
lifecycle_status: active
introduced: cycle-1
modified: 2026-05-17
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "76729b7"
traces_to:
  - "CAP-029"
extracted_from: ".factory/specs/prd.md"
---

# BC-2.16.002: Multi-Step Fetch Pipeline Execution — Sequential Steps with Variable Interpolation

## Description

Config-driven sensor tables may require multi-step fetch sequences where each step's
output feeds into subsequent steps via variable interpolation. Steps execute sequentially
in spec-declared order; variables produced by a step are available to all later steps
but not earlier ones. When a variable resolves to an array, the step is executed in
batches (default batch size 100), with all batches concatenated into a single result set.

The final step's response records are collected into an Arrow RecordBatch using the
table's column definitions. The 10K materialization limit (DI-019) applies to the
final collected records, not intermediate step results. Rate limit hints from the
`SensorSpec` apply between API calls.

## Preconditions
- A spec-driven table has been registered (BC-2.16.001) with one or more `FetchStep` entries in its `steps` array
- A query targeting this table has been dispatched by the query engine (CAP-015)
- An `AuthProvider` implementation is available to resolve credentials for the sensor's `auth_type`. `PipelineExecutor::execute` invokes `AuthProvider::acquire_token` EAGERLY at pipeline start for any sensor whose `auth_type` requires credentials (currently: all real `AuthType` variants — `Oauth2ClientCredentials`, `BearerStatic`, `CookieRoundtrip`, `ApiKey`). The acquired token is used on the FIRST HTTP request of the pipeline. If a subsequent HTTP request receives a 401-Unauthorized, `AuthProvider::acquire_token` is invoked again (refresh) and the failed step is retried once. If the retry also returns 401, the pipeline aborts with `SpecEngineError::AuthRefreshFailed`.
  - **Rationale (F-LP5-LOW-003 closure):** The prior lazy-token-on-401 design forced a guaranteed 401 round-trip on every production execution against bearer-auth APIs, polluting the audit signal (every legitimate execution emitted `auth_refresh_triggered`) and inflating both `request_count` and API quota usage. Eager-token acquisition restores the audit signal to its intended semantic ("refresh event = rare mid-pipeline token expiry") while preserving the 401-retry path for genuine token expiry events.

## Postconditions
- Steps are executed sequentially in the order defined in the spec's `[[table.steps]]` array
- Each step produces an HTTP request using:
  - `method`: GET or POST as declared
  - `path_template`: interpolated against variables from prior steps and query parameters (e.g., `${query_ids.resource_ids}` resolves to the `resource_ids` field from the step named `query_ids`)
  - `body_template` (if present): interpolated identically to `path_template`, then sent as the request body
- Variable interpolation uses the syntax `${step_name.field}` where:
  - `step_name` is the `name` attribute of a prior step
  - `field` is a JSONPath-like dot-notation path into that step's response (e.g., `${query_ids.resources[*].id}` extracts all IDs from the array)
  - Array-valued variables trigger fan-out: the step is executed once per batch of values (batch size configurable per step, default 100)
- Each step's response is parsed according to:
  - `response_path`: a JSONPath expression pointing to the results array in the JSON response (e.g., `$.resources`, `$.data.items`)
  - `pagination_cursor_path` (if present): a JSONPath expression pointing to the pagination cursor in the response for automatic page iteration
  - `variables_produced`: a list of variable names that downstream steps can reference
- Pagination within a step follows the sensor spec's declared pagination config (cursor/offset/none), iterating until the API returns an empty page or the cursor is null
- The final step's response records are collected into an Arrow RecordBatch using the table's column definitions
- Rate limit hints from the `SensorSpec` are applied between API calls: inter-request delay = `1 / requests_per_second`, with burst allowance from `burst_size`
- **Adapter abstraction** — The auth-resolution mechanism is provided via a dyn-compatible `AuthProvider` trait (defined in `prism-spec-engine/src/auth_provider.rs`). `PipelineExecutor` accepts `&dyn AuthProvider`; the trait is object-safe (`Send + Sync` + manually-boxed Future return type per Rust stable RPITIT limitations). This enables sensor-spec-driven adapter dispatch at runtime, replacing compile-time-keyed `SensorAuth` enum dispatch.
- **Record truncation** — When the cumulative `PipelineResult.records.len()` would exceed the DI-019 cap of 10,000, execution truncates the final-step accumulator to exactly 10,000 records and sets `PipelineResult.truncated = true`. The truncation flag is the user-facing signal that data was lost; it does NOT propagate to the per-step `request_count`. The outer materialization-layer cap (in `prism-query/src/materialization.rs`) does NOT double-apply when the executor cap fires.
- **Request count semantics (v1.5)** — `PipelineResult.request_count` is the number of HTTP requests issued by the pipeline steps (NOT including `AuthProvider::acquire_token` calls, which use the AuthProvider's own transport). With the v1.5 eager-token semantic, a single-step single-page pipeline produces `request_count == 1` (not 2 as in v1.4, where a 401 probe request was required before the token was acquired).
- **Auth initial acquisition audit signal (v1.5, superseded by Structured Event Catalog v1.8)** — `PipelineExecutor::execute` and `PipelineExecutor::execute_step` each emit exactly one of three `tracing` events when `AuthProvider::acquire_token` is called eagerly at pipeline start. The field schema differs between the two entry points: `execute()` omits `step_name` (pipeline-level call site); `execute_step()` includes `step_name` (per-step call site). Full field-level enumeration and trigger conditions are in the **Structured Event Catalog (v1.8)** postcondition below. Token value is NEVER included in any event.
- **Auth refresh audit signal (superseded by Structured Event Catalog v1.8)** — When `AuthProvider::acquire_token` is invoked on a 401 retry (mid-pipeline token expiry), `issue_request_with_retry` emits one of four `auth_refresh_*` events (triggered, succeeded, failed, double_401). All four include `step_name` because they fire from within a step execution context. Full field-level enumeration is in the **Structured Event Catalog (v1.8)** postcondition below. Token value is NEVER included in any event. This satisfies VP-PLUGIN-005 assertion (d) (ADR-023 §E).
- **Partial-record discard on mid-pipeline HTTP failure** — When any fetch step's HTTP request fails with a non-401 non-200 status (e.g., 500, 503, network timeout, JSON parse error, page-cap exceeded, cursor non-advance), `PipelineExecutor::execute` returns `Err(SpecEngineError::HttpRequestFailed{...})`. The `PipelineResult` is NOT returned to the caller. ALL records accumulated from prior successfully-completed steps are discarded. This is the "all-or-nothing" semantic: callers must not assume partial data on Err return. Rationale: a partial PipelineResult could mislead downstream OCSF mappers into producing schema-mismatched rows; explicit Err propagation forces the caller to handle the failure mode. The 401-retry path is the exception (handled internally per the auth-refresh postcondition family).
- **Canonical Structured Event Catalog (v1.21)** — This catalog is the single source of truth for every `tracing::*!(event_type=…)` site within the `prism-spec-engine` crate (including `PipelineExecutor`, plugin runtime emissions via `PluginRuntime` and `host_functions::host_http_request`) AND `prism-bin` boot-step plugin-load emissions (step 7.5 per ADR-023 §C4) AND `prism-query` plugin write-tool registration (per ADR-026 D7). PG-LP11-001 requires that new `event_type` sites added to ANY of these scopes MUST be enumerated here as a BC amendment in the same commit as the implementation. The catalog currently contains 33 structured events. Each event sets `event_type` as a structured field. All events are emitted via `tracing` macros; no event includes a token value. (Path B adjudication, fix-burst-8 stage 1: BC-2.16.002's prior narrow scope — "PipelineExecutor and helpers / pipeline.rs" — was a historical artifact from when only PipelineExecutor emitted structured events. PG-LP11-001 always intended a universal catalog; Path B aligns the scope statement to that de-facto role, avoiding new BC IDs (Path A cost) and catalog scatter across 4 BCs (Path C cost).)

| event_type | level | function | fields (beyond event_type) | trigger condition |
|---|---|---|---|---|
| `auth_initial_acquired` | info | `PipelineExecutor::execute` | `sensor_id`, `client_id` | `acquire_token` returns `Ok(tok)` where `tok` is non-empty; eager call at pipeline start before the steps loop |
| `auth_initial_acquired_empty` | debug | `PipelineExecutor::execute` | `sensor_id`, `client_id` | `acquire_token` returns `Ok(tok)` where `tok` is empty string; typically `NullAuthProvider` (test-only) or a buggy production provider; pipeline continues with empty credential |
| `auth_initial_failed` | error | `PipelineExecutor::execute` | `sensor_id`, `client_id`, `detail` | `acquire_token` returns `Err`; pipeline aborts immediately, no fetch steps attempted |
| `auth_initial_acquired` | info | `PipelineExecutor::execute_step` | `sensor_id`, `client_id`, `step_name` | Same Ok-non-empty outcome as execute() variant; includes `step_name` because execute_step is a per-step entry point (field schema differs from the execute() emission) |
| `auth_initial_acquired_empty` | debug | `PipelineExecutor::execute_step` | `sensor_id`, `client_id`, `step_name` | Same Ok-empty outcome as execute() variant; includes `step_name` |
| `auth_initial_failed` | error | `PipelineExecutor::execute_step` | `sensor_id`, `client_id`, `step_name`, `detail` | Same Err outcome as execute() variant; includes `step_name` |
| `auth_refresh_triggered` | warn | `issue_request_with_retry` (called by both execute and execute_step) | `sensor_id`, `client_id`, `step_name` | HTTP 401 received on first attempt; refresh path entered; `acquire_token` about to be called again |
| `auth_refresh_succeeded` | info | `issue_request_with_retry` (called by both execute and execute_step) | `sensor_id`, `client_id`, `step_name` | `acquire_token` on refresh path returns `Ok`; fresh token acquired; retry will proceed |
| `auth_refresh_failed` | error | `issue_request_with_retry` (called by both execute and execute_step) | `sensor_id`, `client_id`, `step_name`, `detail` | `acquire_token` on refresh path returns `Err`; pipeline aborts |
| `auth_refresh_double_401` | error | `issue_request_with_retry` (called by both execute and execute_step) | `sensor_id`, `client_id`, `step_name` | Retry after refresh also returns 401; pipeline aborts with `SpecEngineError::AuthRefreshFailed` |
| `pipeline_truncated` | warn | `PipelineExecutor::execute` (records accumulation loop) | `sensor_id`, `client_id`, `step_name`, `max_records`, `accumulated` | Cumulative `all_records.len()` reaches or exceeds the DI-019 cap of 10,000; records truncated to 10K and `PipelineResult.truncated` set true |
| `pagination_cursor_unsupported_type` | warn | `extract_cursor` (called from execute pagination loop) | `cursor_path`, `actual_type`, `cursor_preview` | Cursor value at `cursor_response_path` resolves to Array, Object, or Bool rather than String or Number; pagination treated as terminal |
| `fanout_invalid_source_type` | warn | `find_fan_out_array` (called from execute fan-out detection) | `step_name`, `var_name`, `actual_type` ("Object") | A template variable reference resolves to an Object-typed value; will be stringified into URL or body; likely a spec authoring bug |
| `fanout_ambiguous_multi_array` | warn | `find_fan_out_array` (called from execute fan-out detection) | `step_name`, `array_vars_count`, `first_var`, `other_vars` | Two or more array-valued variables are referenced in a step's templates; only the first array drives batching; semantics are ambiguous |
| `jsonpath_extraction_failed` | warn | `PipelineExecutor::execute` and `PipelineExecutor::execute_step` — JSON extraction call sites for a step's response_path | `sensor_id`, `step_name`, `path`, `detail` | `extract_at_path` fails for a step's `response_path` (e.g., path not found, bracket index out of bounds, malformed JSONPath syntax); `detail` carries the descriptive error string from `extract_at_path` |
| `jsonpath_size_cap_exceeded` | warn | `extract_with_tokens` — wildcard enumeration recursive descent | `path`, `max_size` | Nested wildcard extraction (e.g., `$.a[*].b[*]`) would exceed `MAX_JSONPATH_RESULT_SIZE` = 100,000 total elements before completion; extraction aborted and `Err` returned to caller |
| `plugin_load_unsigned` | warn | `PluginRuntime::load_all_plugins` | `plugin_path`, `plugin_hash` | Each successfully loaded plugin emits this event at WARN (v1.0 unsigned; audit-channel routing encoded by `event_type` field per ADR-023 §C4) |
| `plugin_load_disabled_via_envvar` | warn | `boot::plugin_load_step` (`prism-bin/src/boot.rs` step-7.5 function) | `env_var: "PRISM_DISABLE_PLUGIN_LOAD"` | `PRISM_DISABLE_PLUGIN_LOAD=1` detected at boot before plugin-load step; plugin loading administratively disabled; emitted before skip to preserve DI-004 audit completeness |
| `plugin_load_failed_manifest_no_allowed_urls` | error | `PluginRuntime::load_plugin` | `plugin_path`, `error: E-PLUGIN-013` | Plugin manifest missing required `allowed_urls` field; plugin rejected; remaining plugins continue loading |
| `plugin_load_failed_format_version_exceeded` | error | `PluginRuntime::load_plugin` | `plugin_path`, `format_version`, `max_supported` | Plugin `format_version` exceeds `CURRENT_SUPPORTED_VERSION`; plugin rejected; remaining plugins continue loading |
| `plugin_load_failed_manifest_name_missing` | error | `PluginRuntime::load_plugin` | `plugin_path`, `error: E-PLUGIN-015` | Manifest `name` field absent or empty string; plugin rejected at manifest gate before WIT compilation; remaining plugins continue loading (n-1 survivor) |
| `plugin_load_failed_manifest_version_malformed` | error | `PluginRuntime::load_plugin` | `plugin_path`, `version_value`, `error: E-PLUGIN-016` | Manifest `version` field absent or not valid semver; plugin rejected at manifest gate before WIT compilation; remaining plugins continue loading (n-1 survivor) |
| `plugin_load_failed_wit_invalid` | error | `PluginRuntime::load_plugin` | `plugin_path`, `missing_export`, `error: E-PLUGIN-001` | WIT validation failure — plugin component is missing one or more required WIT exports; plugin rejected; remaining plugins continue loading |
| `plugin_http_request_blocked` | warn | `host_http_request` (`prism-spec-engine/src/plugin/host_functions.rs`) | `plugin_id`, `url`, `reason: allowlist_mismatch` | Plugin attempted an outbound HTTP request to a URL not present in its manifest `allowed_urls` list; request blocked; plugin execution continues |
| `pipeline_max_requests_exceeded` | error | `PipelineExecutor` executor loop | `sensor_id`, `total_requests`, `max: MAX_REQUESTS_PER_PIPELINE` | Cumulative HTTP request count across all pipeline steps reaches `MAX_REQUESTS_PER_PIPELINE` (10,000); pipeline aborts; no further steps attempted |
| `plugin_directory_not_found` | info | `PluginRuntime::load_all_plugins` | `plugin_dir` | Plugin directory does not exist at the configured path; boot continues with zero plugins loaded (EC-D-001: non-error, expected on first-time or plugin-less deployments) |
| `plugin_load_failed_read_error` | error | `PluginRuntime::load_all_plugins` | `plugin_path`, `error` | Filesystem read of a `.prx` file failed (permissions, I/O error); plugin is skipped; remaining plugins continue loading (n-1 survivor) |
| `plugin_load_failed_compilation` | error | `PluginRuntime::load_all_plugins` | `plugin_path`, `error: E-PLUGIN-008`, `message` | WASM Component Model compilation of a `.prx` file failed (corrupt binary, bad magic, component wrapping failed); plugin is skipped; remaining plugins continue loading (n-1 survivor) |
| `plugin_load_failed_manifest_not_found` | error | `PluginRuntime::load_all_plugins` | `plugin_path`, `expected_manifest_path`, `error: E-PLUGIN-018` | Plugin `.prx` file found but no companion `.manifest.toml` exists at expected path; plugin rejected (n-1 survivor) |
| `plugin_load_failed_manifest_parse_error` | error | `PluginRuntime::load_all_plugins` | `plugin_path`, `error: E-PLUGIN-017`, `detail` | Companion `.manifest.toml` is present but fails TOML parse; plugin rejected (n-1 survivor); distinct from E-PLUGIN-015 which applies only when TOML parses but `name` field is absent |
| `plugin_load_failed_format_version_missing` | error | `PluginRuntime::load_all_plugins` | `plugin_path`, `supported`, `error: E-PLUGIN-019` | Manifest `format_version` field is absent entirely; plugin rejected (n-1 survivor); AC-5: absent `format_version` is a hard rejection, not silently treated as 0 |
| `plugin_log_level_unrecognized` | warn | `register_host_functions` → `host::log` callback (`host_functions.rs`) | `plugin_id`, `received_name` | Plugin sent a `log-level` enum variant name not recognized by the current host implementation (e.g., a future log-level added to a newer WIT IDL); host defaults to `LogLevel::Info` to preserve forward-compatibility. Emitted BEFORE the default so the downgrade is always observable in tracing. Not a trap — trapping on unrecognized future enum names would break forward-compat. Audit role: operational observability (not security). Recurrence: once per unrecognized log call from the plugin. |
| `write_tool_registration_after_boot` | warn | `register_write_tool` (`crates/prism-query/src/invalidation.rs`) | `plugin_name: String` (source: `entry.plugin_name` — `WriteToolInvalidationMap` struct field; set by `PluginRuntime` from plugin manifest `name` field at step 7.5 plugin-load per ADR-026 D7 v1.17), `tool_name: String` (source: `entry.tool_name` — `WriteToolInvalidationMap` struct field), `error: "E-PLUGIN-020"` (source: literal string constant) | `register_write_tool` called after query-engine init starts (step 8+, ADR-026 D7); the `AtomicBool` query-phase flag is set at step 8 start (first act of step 8, before QueryEngine construction proceeds), gating the write; registration rejected with `Err(SpecEngineError::WriteToolRegistrationAfterBoot)`. Audit role: forensic-trace (error event tied to E-PLUGIN-020; matches audit role of sibling WARN-level post-boot rejection events). Recurrence: one emission per post-boot registration attempt; not retried. Retention: per organization audit policy. |

## Variable Scope and Lifetime
- Variables produced by a step are available to all subsequent steps but not to prior steps
- Variables from the most recent execution of a step overwrite previous values (relevant when a step is re-executed due to fan-out)
- Query-time variables are available to all steps: `${query.client_id}`, `${query.filter.*}` (push-down filter values extracted by the query planner)

## Fan-Out Behavior
- When a variable interpolation resolves to an array, the step is executed in batches
- Batch size is configurable per step via `fan_out_batch_size` (default 100)
- Fan-out results are concatenated into a single result set for the step
- Fan-out respects rate limit hints — each batch counts as a separate request

## Invariants
- Steps execute sequentially; no parallel step execution within a single table fetch (simplifies variable scoping)
- The 10K materialization limit (DI-019) applies to the final collected records, not to intermediate step results
- No BC-specific invariants beyond DI-019 and rate-limit behavior defined above.

## Error Conditions
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-SPEC-010` | Variable interpolation failure at runtime (variable exists but field path does not match response structure) | `E-SPEC-010` with the step name, variable reference, and actual response structure hint |
| (sensor_error) | HTTP error on any step (non-401, non-200) | Pipeline aborts; ALL accumulated records discarded; `Err(HttpRequestFailed)` propagated to caller. NO partial `PipelineResult` returned. See partial-record-discard postcondition for rationale. |
| (no error) | Empty response from a non-final step | Subsequent steps receive empty arrays; produces zero results (not an error) |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| Fan-out batch of 250 IDs | 250 IDs; batch_size=100 | 3 executions: 100, 100, 50; all results concatenated |
| Empty non-final step | step 1 returns empty; step 2 needs step 1 output | Step 2 receives empty variable; produces zero records |
| Pagination | step has cursor pagination | Iterates pages until cursor=null or empty page |
| Rate limiting | spec declares 5 req/s | 200ms delay between requests; burst allowed |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for full canonical vectors.

| Scenario | Input | Expected Output |
|----------|-------|-----------------|
| Happy path — 2-step | step 1 fetches IDs; step 2 fetches details by ID | Final RecordBatch with detail records |
| Fan-out | step 2 depends on array from step 1 (250 items) | 3 batched requests; all results concatenated |
| HTTP error on step 1 | step 1 returns 500 | Pipeline aborts; sensor_error in response |
| Empty step 1 | step 1 returns zero records | Step 2 produces zero records; no error |
| Interpolation failure | step 2 references `${step1.missing_field}` | `E-SPEC-010` with step name and field path |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| (none) | Fan-out batch concatenation requires HTTP mock integration; forward-reference scoping rejection is covered by VP-059 (BC-2.16.009 validation); no additional formal VP for runtime pipeline execution. |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-029 |
| L2 Invariants | DI-019 |
| Related BCs | BC-2.16.001 (spec loading), BC-2.01.014 (exponential backoff on HTTP calls), BC-2.01.010 (partial failure) |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.24 | FB50 | 2026-05-17 | architect | POL-23 sibling-sweep OBS-LP62-002 interpretation #2: catalog row 33 live-narrative ADR-026 D7 v1.10 pin bumped to v1.17 (current ADR-026 version per FB47 §Related ADRs row edit; D7 content unchanged since v1.16). |
| 1.23 | FB43 | 2026-05-16 | product-owner | F-LP54-HIGH-001 — v1.22 row narrative retroactively reframed under Fork B canonical rule (POL-30 established by FB42 D-662). The v1.22 closure correctly closed a legitimate FB37-introduced catalog-content-version sync gap (catalog row 33 was added at FB37 but bullet header (v1.20) was not bumped at that time; FB41 closed that legitimate gap by advancing bullet to (v1.21)). The v1.22 row's "syncing with frontmatter v1.21" framing was Fork-A-aligned; Fork B canonical rule (POL-30) instead recognizes that bullet-version-label tracks catalog-content-version INDEPENDENT of BC frontmatter. Under Fork B, FB41 fix remains correct (bullet (v1.21) reflects catalog state after FB37 row 33 addition), and post-FB41 state is internally consistent: bullet (v1.21) ↔ 8 cite-pin sites at (v1.21) ↔ frontmatter (v1.22) reflecting independent BC-narrative-cite-pin-update version-bump. The v1.22 row's "POL-30 codification candidate (BC frontmatter version bumps must include in-body canonical-anchor bullet label updates)" text is retired Fork-A phrasing; Fork B POL-30 instead canonicalizes independent versioning per SESSION-HANDOFF §POST-FB42-CLOSURE / SESSION-D644-TASKS §Fork B Canonical Rule. |
| 1.22 | FB41 | 2026-05-16 | product-owner | F-LP52-HIGH-001 §Postconditions Canonical Structured Event Catalog bullet header label advanced (v1.20)→(v1.21) syncing with frontmatter v1.21 + 8 PREREQ-E cite-pin sites. 9th manifestation of POL-23 within-FB sibling-sweep asymmetry closed; POL-30 codification candidate (BC frontmatter version bumps must include in-body canonical-anchor bullet label updates) for cycle-close. |
| 1.21 | FB37 | 2026-05-16 | product-owner | F-LP47-HIGH-001 row 33 trigger condition corrected from "after query engine init completes (step 8+)" to canonical "query-engine init starts (step 8+, ADR-026 D7); the AtomicBool query-phase flag is set at step 8 start (first act of step 8, before QueryEngine construction proceeds)"; semantic match with BC-2.16.012 EC-016-012-005 v1.16 + HS-PREREQ-E-003-05 v1.6. |
| 1.20 | prereq-e-fix-burst-14 | 2026-05-16 | product-owner | F-LP15-HIGH-001 — within-FB12 sibling-sweep asymmetry close (6th RECURRENCE of POL-23 class): §Postconditions Canonical Structured Event Catalog bullet header label advanced (v1.18) → (v1.19) matching frontmatter version (FB12 D-603 bumped frontmatter v1.18→v1.19 for row 33 source spec clarification but did not sync internal bullet label; downstream BC-2.16.012 + error-taxonomy cites of (v1.19) were phantom until this sync). Bump v1.19→v1.20 per POL-11 index-mutation-version-bump discipline + POL-23 sibling-sweep gate. |
| 1.19 | prereq-e-fix-burst-12 | 2026-05-16 | architect | F-LP13-HIGH-003 — Catalog row 33 source spec clarification: `plugin_name` field source = `entry.plugin_name` (`WriteToolInvalidationMap` struct field, set by `PluginRuntime` from plugin manifest `name` field per ADR-026 D7 v1.10); `tool_name` field source = `entry.tool_name` (`WriteToolInvalidationMap` struct field); `error` field source = literal string `"E-PLUGIN-020"`. Resolves spec-implementation coherence gap: row 33 mandated `plugin_name` field with no defined source; implementer could not satisfy the catalog contract without this provenance. Provenance anchored to ADR-026 D7 v1.10 Option A adjudication (struct extension). No catalog row count or event_type changes — source-spec clarification only. |
| 1.18 | prereq-e-fix-burst-11 | 2026-05-16 | product-owner | F-LP12-MED-001 — Add `write_tool_registration_after_boot` WARN event row to Canonical Structured Event Catalog (row 33). New emission site introduced by S-PLUGIN-PREREQ-E ADR-026 D7 + error-taxonomy E-PLUGIN-020 + HS-PREREQ-E-003-05. Source: `register_write_tool` in `crates/prism-query/src/invalidation.rs`. Fields: `plugin_name: String`, `tool_name: String`, `error: "E-PLUGIN-020"`. Audit role: forensic-trace. Recurrence: one emission per post-boot registration attempt; not retried. Catalog scope statement updated to include `prism-query` plugin write-tool registration (ADR-026 D7). Count 32→33; label v1.17→v1.18. Per PG-LP11-001 (codified PREREQ-B cascade) + CLAUDE.md Conventions §Structured event catalog discipline. Scope expansion to BC-2.16.002 per Canonical Principle Rule 4. |
| 1.17 | S-PLUGIN-PREREQ-D-fix-burst-impl-3 | 2026-05-14 | implementer | F-PASS3-HIGH-001 closure: add `plugin_log_level_unrecognized` catalog row (row 32). Emitted by `register_host_functions` `host::log` callback when plugin sends a WIT enum log-level name not recognized by the host (e.g., future log-level variant). Host safe-defaults to LogLevel::Info after emitting this event (forward-compat preservation). Fields: `plugin_id`, `received_name`. Audit role: operational observability. Catalog intro updated v1.16→v1.17; count 31→32. Closes F-PASS3-HIGH-001 (SOUL.md #4 observability; BC-2.16.002 PG-LP11-001 SOP). |
| 1.16 | S-PLUGIN-PREREQ-D-fix-burst-impl-2 | 2026-05-14 | implementer | F-PASS2-HIGH-001 closure: update Canonical Structured Event Catalog prose intro (v1.12→v1.16; count 25→31) per TD-VSDD-060 sibling-sweep gap. The intro line was last updated at v1.12 and missed 3 subsequent amendments (v1.13/v1.14/v1.15) that added 6 new catalog rows (25→31). BC version bumped to v1.16 to record this amendment. No catalog row content changes — metadata-only fix to the intro line. Closes F-PASS2-HIGH-001. |
| 1.15 | S-PLUGIN-PREREQ-D-fix-burst-impl-1 | 2026-05-14 | implementer | F-IMPL-LP1 fix-burst closures: (1) MED-001: add `message` field to `plugin_load_failed_compilation` row (emission site emits `message` but catalog row omitted it). (2) MED-002: rename `plugin_id` → `sensor_id` in `pipeline_max_requests_exceeded` catalog row and emission site (pipeline executor has sensor_id context, not plugin_id). (3) Add 3 new catalog rows: `plugin_load_failed_manifest_not_found` (E-PLUGIN-018, HIGH-005), `plugin_load_failed_manifest_parse_error` (E-PLUGIN-017, HIGH-003), `plugin_load_failed_format_version_missing` (E-PLUGIN-019, HIGH-006). Total catalog rows: 28 → 31. |
| 1.14 | S-PLUGIN-PREREQ-D-impl | 2026-05-14 | implementer | PG-LP11-001 closure: add 3 new Structured Event Catalog rows discovered during S-PLUGIN-PREREQ-D TDD implementation — `plugin_directory_not_found` (INFO, `PluginRuntime::load_all_plugins`, `plugin_dir`, EC-D-001 non-error path), `plugin_load_failed_read_error` (ERROR, same emitter, `plugin_path` + `error`, filesystem I/O failure on `.prx` read), `plugin_load_failed_compilation` (ERROR, same emitter, `plugin_path` + `error: E-PLUGIN-008`, WASM binary compilation failed). All 3 emit from `load_all_plugins` n-1 survivor loop. Total catalog rows: 25 → 28. Implements Standing Rule 3 §6: implementer amends BC-2.16.002 in same commit as emission site per PG-LP11-001 SOP codified in cycles/wave-4-operations/lessons.md Lesson 1. |
| 1.13 | fix-burst-37 | 2026-05-14 | state-manager | (D-541) F-LP40-MED-001 closure: frontmatter `modified` field updated from `null` to `2026-05-14` and `timestamp` updated from stale 2026-04-13T12:00:00 (original v1.0 cycle-1 authorship date) to 2026-05-14T00:00:00Z. Sibling-sweep gap from F-LP36-MED-001 / OBS-LP36-001 codification (fix-burst-34) — pattern was caught in BC-2.17.007 but not propagated to BC-2.16.002 despite 12 amendments through v1.12. Matches canonical pattern established in fix-burst-34 (BC-2.17.007 v1.2→v1.3 frontmatter sync). No body content changes — pure metadata sync per state-manager frontmatter-sync routing. |
| 1.12 | S-PLUGIN-PREREQ-D-fix-burst-17-stage-1A | 2026-05-13 | product-owner | F-LP18-MED-001 BC portion closure — add 2 new Structured Event Catalog rows for E-PLUGIN-015 and E-PLUGIN-016 manifest validation errors: `plugin_load_failed_manifest_name_missing` (ERROR, `PluginRuntime::load_plugin`, `plugin_path` + `error: E-PLUGIN-015`) and `plugin_load_failed_manifest_version_malformed` (ERROR, `PluginRuntime::load_plugin`, `plugin_path` + `version_value` + `error: E-PLUGIN-016`). AC-5 specifies 4 manifest error codes (E-PLUGIN-013/014/015/016); EC table covered all 4 after fix-burst-16; catalog was asymmetric with only E-PLUGIN-013/014 cataloged. This amendment restores symmetry. Total catalog rows: 23 → 25. Catalog label updated from "(v1.11)" to "(v1.12)". |
| 1.11 | S-PLUGIN-PREREQ-D-fix-burst-8-stage-1 | 2026-05-13 | product-owner | F-LP9-MEDIUM-001 closure — Path B adjudication: catalog-destination scope mismatch between BC-2.16.002 narrow scope ("PipelineExecutor and helpers / pipeline.rs only") and PG-LP11-001's universal-catalog architectural intent. 6 of 7 new story-listed event_type rows emit from outside pipeline.rs (PluginRuntime, host_functions, boot.rs step-7.5). Path B chosen: expand BC-2.16.002 scope to cover all prism-spec-engine emissions + prism-bin boot-step plugin-load emissions. Path A (new BC-2.17.008) rejected: unnecessary new ID per POL-1 append-only cost. Path C (scatter across 4 BCs) rejected: breaks PG-LP11-001 single-source-of-truth intent. Changes: (1) catalog header renamed from "Structured Event Catalog (v1.10)" to "Canonical Structured Event Catalog (v1.11)"; (2) scope statement rewritten to cover all prism-spec-engine + prism-bin plugin-load event_type sites; (3) 7 new rows added: plugin_load_unsigned, plugin_load_disabled_via_envvar, plugin_load_failed_manifest_no_allowed_urls, plugin_load_failed_format_version_exceeded, plugin_load_failed_wit_invalid, plugin_http_request_blocked, pipeline_max_requests_exceeded. Total catalog rows: 16 → 23. BC-2.22.001 delegation ("per BC-2.16.002") preserved unchanged — Path B makes BC-2.16.002 the correct authority; no material edit needed there. |
| 1.10 | S-PLUGIN-PREREQ-C-fix-burst-1 | 2026-05-12 | product-owner | Add 2 new Structured Event Catalog rows: jsonpath_extraction_failed (F-LP1-HIGH-001 — AC-2 bounds-check observability silence closure) and jsonpath_size_cap_exceeded (F-LP1-HIGH-007 — AC-2 nested-wildcard memory amplification protection). Total catalog rows: 14 → 16. Catalog label updated from "(v1.8)" to "(v1.10)". PG-LP11-001 SOP enforced. |
| 1.9 | S-PLUGIN-PREREQ-B-post-merge | 2026-05-12 | state-manager | Status draft→active per POL-14: anchor story S-PLUGIN-PREREQ-B merged via PR #143 at develop@ae7e26c8 (2026-05-12T06:58:48Z). 16 LOCAL adversary passes + 13 fix-bursts + PR-LEVEL 1/1 CLEAN + 34/34 CI + pr-reviewer APPROVE. No substantive content change — metadata-only promotion. |
| 1.8 | S-PLUGIN-PREREQ-B-fix-burst-11 | 2026-05-11 | product-owner | Add Structured Event Catalog enumerating all 14 event_type variants emitted by PipelineExecutor (auth_initial_* x 2 functions x 3 outcomes = 6; auth_refresh_* = 4; pipeline_truncated; pagination_cursor_unsupported_type; fanout_invalid_source_type; fanout_ambiguous_multi_array). Documents field-schema differences between execute() (no step_name on auth_initial_*) and execute_step() (includes step_name). Closes BC↔impl catalog drift surfaced by pass-11 (F-LP11-MED-001 + F-LP11-MED-002 + PG-LP11-001) and codifies the SOP that new event_type sites must be enumerated in BC. |
| 1.7 | S-PLUGIN-PREREQ-B-fix-burst-9 | 2026-05-11 | product-owner | Amend audit-signal postcondition row to enumerate THREE tracing events (was "one of two"): auth_initial_acquired (info, non-empty token), auth_initial_acquired_empty (debug, empty token), auth_initial_failed (error). Closes BC↔impl drift surfaced by pass-9 (F-LP9-MED-001). The third branch was added by fix-burst-7 (closing F-LP7-MED-001) and tested by fix-burst-8 (closing F-LP8-MED-001) but the BC text was never updated. |
| 1.6 | LOCAL-pass-7-fix | 2026-05-11 | product-owner | Clarify partial-record discard policy on mid-pipeline HTTP failure. Existing § Error Conditions row replaced with explicit "ALL accumulated records discarded" + new postcondition explaining all-or-nothing rationale. Closes F-LP7-MED-003 from LOCAL pass-7 adversary review at 8e9a92d0 (BC text ambiguity surfaced by partial-record test coverage gap). |
| 1.5 | LOCAL-pass-5-fix | 2026-05-11 | product-owner | Eager-token precondition lifecycle. Replace lazy-token-on-401 with eager-acquire-at-pipeline-start for non-Null AuthType. Closes F-LP5-LOW-003 from LOCAL pass-5 adversary review at d5a12e4a: prior lazy design polluted audit signal (auth_refresh_triggered fired on every legitimate execution) and doubled API quota per execution. Two new audit-log events (auth_initial_acquired/auth_initial_failed) augment the existing auth_refresh_* event family. request_count semantics now exclude AuthProvider transport. Status remains draft pending PREREQ-B merge — POL-14 promotes draft→active on merge. |
| 1.4 | LOCAL-pass-1-fix | 2026-05-11 | product-owner | Amend preconditions and postconditions to reflect AuthProvider abstraction introduced by S-PLUGIN-PREREQ-B. Lazy credential resolution replaces eager. New postconditions: AuthProvider trait dyn-safety; PipelineResult.truncated semantics; auth_refresh_triggered tracing event for VP-PLUGIN-005. Closes F-LP1-MED-001 from LOCAL pass-1 adversary review at b1b529fc. Status remains draft pending PREREQ-B merge — POL-14 promotes draft→active on merge. |
| 1.3 | pass-74-fix | 2026-04-20 | product-owner | Resolved (placeholder) row in ## Verification Properties per pass-74 VP-TBD decision matrix extension. |
| 1.2 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description; added ## Invariants; added ## Error Conditions (from inline Error Handling); converted ## Traces → ## Traceability table; added ## Canonical Test Vectors; added ## Verification Properties; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial draft (used ## Traces section) |
