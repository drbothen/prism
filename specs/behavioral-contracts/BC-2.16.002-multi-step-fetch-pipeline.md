---
document_type: behavioral-contract
level: L3
version: "1.10"
status: active
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "SS-16"
capability: "CAP-029"
lifecycle_status: active
introduced: cycle-1
modified: null
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
- **Structured Event Catalog (v1.10)** — `PipelineExecutor` and its private helper functions emit the following 16 structured `tracing` events. Each event sets `event_type` as a structured field. All events are emitted via `tracing` macros; no event includes the token value. New `event_type` sites added to `pipeline.rs` or its helpers MUST be enumerated here as a BC amendment before merge.

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
