---
document_type: story
story_id: S-PLUGIN-PREREQ-B
title: "Real PipelineExecutor — HTTP Client, JSONPath, Fan-out, Paginate, 401-Retry (Plugin Migration Keystone)"
wave: 0
epic_id: PLUGIN-MIGRATION-001
priority: P0
status: draft
# BC status: behavioral_contracts populated from existing BCs only — no new BCs required
#            BC-2.16.002 is the primary anchor (pipeline execution semantics).
#            BC-2.01.013 is the secondary anchor (spec-driven adapter pattern; AuthProvider
#            trait is the TOML-driven runtime auth surface replacing compile-time adapter dispatch).
behavioral_contracts:
  - BC-2.16.002  # Multi-Step Fetch Pipeline Execution — primary authority
  - BC-2.01.013  # DataSource Trait: Spec-Driven Adapter Pattern — AuthProvider replaces hardcoded auth
depends_on: [S-PLUGIN-PREREQ-F, S-PLUGIN-PREREQ-A]
blocks:
  - PLUGIN-MIGRATION-001-D  # Author 4 Production TOML Sensor Specs — requires real executor
  - PLUGIN-MIGRATION-001-A  # Delete 4 Named Auth Modules — gated on VP-PLUGIN-003 (parity)
points: 13
estimated_days: 5
risk: HIGH
tdd_mode: strict
crates_touched: [prism-spec-engine]
target_module: prism-spec-engine
# Subsystem anchor justifications (PG-PR1-002):
#   SS-16 owns this story's scope because prism-spec-engine is the sole module that contains
#   PipelineExecutor and the TOML spec-driven execution engine per ARCH-INDEX Subsystem Registry.
#   SS-01 is included because the AuthProvider trait introduced here is the spec-driven
#   replacement for the hardcoded sensor auth modules in the Sensor Adapters subsystem (SS-01),
#   and the new auth interface is consumed at query time through the same dispatch path that
#   SS-01 currently owns.
subsystems: [SS-16, SS-01]
version: "1.11"
level: "L4"
producer: state-manager
timestamp: "2026-05-11T05:00:00Z"
input-hash: "6954524"
traces_to: []
cycle: "v1.0.0-greenfield"
phase: 3
verification_properties:
  - VP-PLUGIN-002  # VP-147: PipelineExecutor::execute returns non-empty records against wiremock DTU clone
  - VP-PLUGIN-005  # VP-150: OAuth2 refresh-on-401 via declarative TOML retry policy (pipeline_oauth_retry.rs test target)
anchor_vps: [VP-PLUGIN-002, VP-PLUGIN-005]
anchor_bcs: [BC-2.16.002, BC-2.01.013]
anchor_capabilities: [CAP-029]
anchor_subsystem: [SS-16, SS-01]
assumption_validations: []
risk_mitigations: []
acceptance_criteria_count: 9
red_gate_tests: 47
inputs:
  - ".factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md"
  - ".factory/specs/behavioral-contracts/BC-2.16.002-multi-step-fetch-pipeline.md"
  - ".factory/specs/behavioral-contracts/BC-2.01.013-datasource-trait-adapter-pattern.md"
  - ".factory/stories/S-PLUGIN-PREREQ-A-sensorid-newtype.md"
---

# S-PLUGIN-PREREQ-B — Real PipelineExecutor — HTTP Client, JSONPath, Fan-out, Paginate, 401-Retry

## Narrative

As the Prism platform, I want `PipelineExecutor::execute` in
`crates/prism-spec-engine/src/pipeline.rs` replaced with a real HTTP-capable implementation,
so that sensor queries drive actual API calls declared in TOML spec files instead of returning
an empty stub, unblocking the full plugin-only sensor architecture where TOML specs define
sensor behavior without compiled-in Rust adapters.

---

## Summary

`PipelineExecutor::execute` currently returns `Ok(Vec::new())` unconditionally — TOML-driven
sensor execution is entirely non-functional at runtime (noted in ADR-023 §C2 as "architectural
fraud"). This story replaces the stub with a real implementation that:

1. Resolves `SensorSpec` from the spec-catalog for the given table
2. Executes HTTP fetch steps sequentially with `${step.field}` variable interpolation
3. Applies JSONPath extraction (`response_path`) on each HTTP response
4. Handles offset/cursor pagination (iterating pages until exhausted or `null` cursor)
5. Implements fan-out batching (existing `fan_out_batches` pure function is reused)
6. Handles 401-Unauthorized via auth-driver re-acquisition using a new `AuthProvider` trait
7. Applies rate-limit hints between API calls
8. Injects an HTTP client (`reqwest::Client`) via dependency injection (not a global)
9. Produces non-empty `PipelineResult` records when run against a wiremock mock server

VP-PLUGIN-002 (PipelineExecutor returns non-empty records against at least one wiremock mock)
must pass. This story unblocks PLUGIN-MIGRATION-001-D (authoring production TOML sensor specs)
which is in turn the gate for VP-PLUGIN-003 (DTU parity) and PLUGIN-MIGRATION-001-A (deletion
of the four hardcoded Rust auth modules).

**What PREREQ-B does NOT do:**
- Does not author the four production sensor TOML specs (that is PLUGIN-MIGRATION-001-D)
- Does not integrate `PipelineExecutor` into the `prism-query` materialization path (the
  existing `prism_sensors::fan_out()` path in `materialization.rs` continues to route queries
  through legacy adapters; the cutover to `PipelineExecutor` happens in Wave 1)
- Does not implement the PREREQ-C TOML grammar extensions (`[fetch_step.retry]`,
  `[fetch_step.batch]`, `virtual_field_aliases`, `cache_ttl_secs`)
- Does not wire `PluginRuntime` into boot (that is PREREQ-D)

---

## Behavioral Contracts

| BC ID | Title | Subsystem | Role in This Story |
|-------|-------|-----------|-------------------|
| BC-2.16.002 | Multi-Step Fetch Pipeline Execution — Sequential Steps with Variable Interpolation | SS-16 | Primary authority — defines sequential step execution, variable interpolation semantics, fan-out batching, pagination, rate-limit application, and the 10K materialization limit (DI-019). Every PREREQ-B AC must satisfy these postconditions. |
| BC-2.01.013 | DataSource Trait: Spec-Driven Adapter Pattern | SS-01 | Secondary authority — `AuthProvider` trait introduced in this story is the TOML-driven auth surface that replaces compile-time adapter dispatch. Auth resolution at query time (via `acquire_token`) is the mechanism by which spec-driven adapters authenticate without hardcoded Rust code. |

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec | ~5,500 |
| `prism-spec-engine/src/pipeline.rs` (stub → real, full rewrite) | ~3,000 |
| `prism-spec-engine/src/auth_provider.rs` (new `AuthProvider` trait) | ~1,500 |
| `prism-spec-engine/src/lib.rs` (re-export `AuthProvider`) | ~300 |
| `prism-spec-engine/Cargo.toml` (add `wiremock` to dev-deps) | ~200 |
| `prism-spec-engine/tests/pipeline_http_integration.rs` (new — wiremock test) | ~2,500 |
| `prism-spec-engine/tests/pipeline_oauth_retry.rs` (new — VP-PLUGIN-005 test) | ~2,000 |
| `prism-spec-engine/tests/bc_2_16_002_test.rs` (update for HttpClient injection) | ~1,500 |
| BC files (2 BCs: BC-2.16.002, BC-2.01.013) | ~4,000 |
| ADR-023 §C2 reference | ~1,000 |
| Total | ~21,500 |

Within the 30% context window budget (~40k tokens for a 128k-context agent).

---

## Tasks

1. **Define `AuthProvider` trait in `prism-spec-engine/src/auth_provider.rs`** (new file):
   ```rust
   /// Trait for resolving auth tokens for a sensor at query time.
   /// PREREQ-B introduces this as the spec-driven replacement for hardcoded
   /// SensorAuth impls. AuthProvider is injected into PipelineExecutor.
   #[async_trait]
   pub trait AuthProvider: Send + Sync {
       /// Acquire an auth token for the given sensor spec and client.
       /// Called on first request and on 401-Unauthorized retry.
       async fn acquire_token(
           &self,
           spec: &SensorSpec,
           client_id: &OrgSlug,
       ) -> Result<AuthToken, SpecEngineError>;
   }

   /// An opaque bearer token string.
   pub struct AuthToken(pub String);
   ```
   This trait is the auth interface referenced in ADR-023 §C2 and VP-PLUGIN-005. It
   abstracts credential resolution from `PipelineExecutor` — test code injects a mock;
   production code injects a `CredentialStoreAuthProvider` (a `NullAuthProvider` is acceptable
   as the production placeholder until PLUGIN-MIGRATION-001-D wires real credentials).

2. **Rewrite `PipelineExecutor::execute` in `pipeline.rs`** to:
   - Accept `http_client: &reqwest::Client` and `auth_provider: &dyn AuthProvider`
   as additional parameters (or wrap in a `PipelineExecutorConfig` struct for extensibility)
   - Build the base URL from `spec.base_url`
   - For each `FetchStep` in `table.steps`:
     a. Resolve variable interpolations via the existing `Interpolator` (already implemented)
     b. Issue the HTTP request (`GET` or `POST`) with `Authorization: Bearer <token>`
     c. Parse the JSON response and extract records via `response_path` (JSONPath)
     d. If HTTP 401: call `auth_provider.acquire_token()`, update token, retry ONCE
     e. If `pagination` is set: iterate pages (cursor/offset) until empty page or null cursor
     f. If a variable resolves to an array: call existing `fan_out_batches()` and batch-execute
     g. Apply rate-limit hints: `1 / requests_per_second` sleep between calls
     h. Collect final step records into `PipelineResult.records`
   - The `execute_step` method signature similarly gains `http_client` + `auth_provider`

3. **Implement JSONPath extraction helper** (simple `$.field` / `$.a.b` paths) in `pipeline.rs`
   or a new `jsonpath.rs` module. The existing `FetchStep.response_path` (e.g., `$.resources`,
   `$.data.items`) requires extracting a JSON array from an arbitrary path. Use `serde_json`
   pointer-style navigation or a minimal recursive descent — do NOT add a heavy JSONPath crate.
   The grammar is limited to dot-notation paths and wildcard array extraction as declared in
   the spec types; `$[*]` glob is NOT required.

4. **Update `prism-spec-engine/Cargo.toml`** dev-dependencies:
   ```toml
   [dev-dependencies]
   wiremock = "0.6"
   tokio = { version = "1", features = ["rt-multi-thread", "macros", "time"] }
   ```
   Production dependencies: `reqwest = "0.12"` is already present. No new prod deps needed.
   `async-trait = "0.1"` must be added if `AuthProvider` uses `#[async_trait]`; alternatively,
   use `impl Future` return or `async fn` in trait (Rust 1.75+ RPIT). Check `rust-toolchain.toml`
   for the exact edition — if edition 2024 is in use, `async fn in trait` is stable and preferred.

5. **Write integration test `tests/pipeline_http_integration.rs`** (VP-PLUGIN-002):
   - Start a `wiremock::MockServer`
   - Register a `GET /api/detections` mock that returns a JSON body with 2 records
   - Build a `SensorSpec` pointing at `mock_server.uri()` with a single `FetchStep`
   - Inject a `MockAuthProvider` that returns a fixed bearer token
   - Call `PipelineExecutor::execute(&spec, &table, &context, &http_client, &auth_provider).await`
   - Assert `result.records.len() == 2` (non-empty, VP-PLUGIN-002 fulfilled)

6. **Write integration test `tests/pipeline_oauth_retry.rs`** (VP-PLUGIN-005 Red Gate):
   - Start a `wiremock::MockServer` with a sequential responder:
     - First request to `/api/data` → HTTP 401
     - Second request to `/api/data` → HTTP 200 with fixture data
   - Register a second mock for `/oauth2/token` → HTTP 200 with fresh token
   - Assert: (a) auth_provider.acquire_token called a second time on 401 receipt,
     (b) final result is non-empty, (c) request count reflects the retry

7. **Update existing test `tests/bc_2_16_002_test.rs`**: the `test_BC_2_16_002_two_step_pipeline_step2_uses_step1_token` test currently calls `PipelineExecutor::execute` with the stub signature and `drop(result)`. After this story, the signature changes — update the call site to inject a `MockAuthProvider` and a `reqwest::Client` (or test-only no-op wiring). The test must either pass with a wiremock server OR be marked `#[ignore]` pending full wiring (do NOT leave a compilation failure).

8. **Export `AuthProvider` from `lib.rs`**: add `pub use auth_provider::{AuthProvider, AuthToken};` to `prism-spec-engine/src/lib.rs` alongside the existing `PipelineExecutor` re-export.

9. **Run `just iter prism-spec-engine`** — all existing tests must remain green; new tests must be present as Red Gate stubs on the feature branch before implementation begins.

---

## Acceptance Criteria

**AC-1 (HTTP execution):** `PipelineExecutor::execute` issues at least one real HTTP request per
`FetchStep` in the table spec. Given a `SensorSpec` with one `FetchStep` and a wiremock server
registered to respond at `spec.base_url + step.path_template`, `execute` returns a
`PipelineResult` whose `records` field is non-empty and matches the mock's JSON response body
extracted at `step.response_path`.
(traces to BC-2.16.002 postcondition — each step produces an HTTP request using method, path_template, and body_template as declared; response is parsed according to response_path)

**AC-2 (Variable interpolation survives HTTP boundary):** A two-step pipeline where step 2's
`path_template` contains `${step1.access_token}` correctly resolves the token value from step 1's
HTTP response and injects it into step 2's request URL. The existing `Interpolator` is reused for
string substitution; the HTTP layer provides the actual response JSON for step 1.
(traces to BC-2.16.002 postcondition — path_template is interpolated against variables from prior steps; step_name.field resolves to the field path in the step's parsed response)

**AC-3 (Cursor pagination):** When a `FetchStep` has `pagination: Some(PaginationConfig::CursorToken { cursor_response_path })`, `execute` iterates pages: on each response, extract the cursor at `cursor_response_path`; if cursor is present and non-null, issue a subsequent request with the cursor value substituted; stop when cursor is null or the page is empty. All page records are concatenated into `PipelineResult.records`.
(traces to BC-2.16.002 postcondition — pagination within a step follows the sensor spec's declared pagination config, iterating until the API returns an empty page or the cursor is null)

**AC-4 (Offset pagination):** When a `FetchStep` has `pagination: Some(PaginationConfig::OffsetLimit { page_size })`, `execute` iterates pages using an incrementing offset: `offset = 0, page_size, 2*page_size, ...`. Stops when a page returns fewer records than `page_size`. All records concatenated.
(traces to BC-2.16.002 postcondition — pagination within a step follows the sensor spec's declared pagination config)

**AC-5 (401 retry via AuthProvider):** When any HTTP step returns 401-Unauthorized, `execute` calls `auth_provider.acquire_token(spec, client_id)` exactly once per 401 occurrence, replaces the `Authorization: Bearer` header with the fresh token, and retries the failed step ONCE. If the retry also returns 401, the pipeline aborts with a structured `SpecEngineError`. The `AuthProvider` trait is defined in `prism-spec-engine/src/auth_provider.rs` and re-exported via `lib.rs`.
(traces to BC-2.16.002 precondition — credentials for the sensor's auth_type have been resolved for the target client_id; and BC-2.01.013 postcondition — adapter implementations are produced from TOML SensorSpec declarations at runtime; auth re-acquisition is spec-driven not hardcoded)

**AC-6 (Fan-out reuse):** When a variable from step N resolves to an array, `execute` calls the existing `PipelineExecutor::fan_out_batches()` to split the array into batches of `step.fan_out_batch_size.unwrap_or(100)`, and executes the downstream step once per batch. All batch results are concatenated into a single result set for that step. The `fan_out_batches` pure function is NOT duplicated — it is called directly.
(traces to BC-2.16.002 Fan-Out Behavior — when a variable interpolation resolves to an array, the step is executed in batches; fan-out results are concatenated into a single result set)

**AC-7 (Rate-limit hints):** When `spec.rate_limit_hints` is `Some(RateLimitHints { requests_per_second: Some(r), .. })`, `execute` inserts an inter-request delay of `1.0 / r` seconds (tokio::time::sleep) between consecutive HTTP calls. Delay applies between all calls including pagination iterations. When `requests_per_second` is None, no delay is inserted.
(traces to BC-2.16.002 postcondition — rate limit hints from the SensorSpec are applied between API calls: inter-request delay = 1 / requests_per_second)

**AC-8 (DI-019 limit respected):** The 10K materialization limit applies to the final collected records across all steps. When `PipelineResult.records.len()` would exceed 10,000, `execute` truncates and sets a `truncated: bool` flag in `PipelineResult` (or returns an error — implementer chooses; must match DI-019 behavior defined in BC-2.11.006). The existing in-query record cap in `materialization.rs` does NOT double-apply; the `PipelineExecutor` cap is the inner guard.
(traces to BC-2.16.002 invariant — the 10K materialization limit (DI-019) applies to the final collected records, not to intermediate step results)

**AC-9 (VP-PLUGIN-002 integration test passes):** The test `test_BC_PLUGIN_002_pipeline_executor_returns_nonempty_records_against_wiremock` in `tests/pipeline_http_integration.rs` passes: a `wiremock::MockServer` returns a two-record JSON fixture; `PipelineExecutor::execute` returns `PipelineResult { records: [r1, r2], request_count: 1, .. }`. This is the acceptance criterion for VP-PLUGIN-002 in this story's scope.
(traces to BC-2.16.002 postcondition — the final step's response records are collected; and to VP-PLUGIN-002 — PipelineExecutor::execute returns non-empty records against at least one wiremock mock, replacing the Ok(Vec::new()) stub)

---

## Red Gate Test Set (failing tests that MUST exist BEFORE implementation)

The test-writer MUST produce these failing tests on the feature branch before the implementer
writes any production code. All tests must fail RED at that point because `PipelineExecutor::execute`
still has the stub signature (no `http_client`, no `auth_provider` parameter).

1. **`test_BC_2_16_002_execute_issues_http_request_and_returns_nonempty_records`**
   (`tests/pipeline_http_integration.rs`) — calls `PipelineExecutor::execute` with a wiremock
   server as the HTTP backend. Fails RED because the current signature has no `http_client`
   parameter and the stub returns empty records. This is the primary VP-PLUGIN-002 Red Gate.

2. **`test_BC_2_16_002_execute_interpolates_step1_var_into_step2_url`**
   (`tests/pipeline_http_integration.rs`) — two-step wiremock test; step 2 URL must contain
   the token extracted from step 1's response. Fails RED because stub returns empty records,
   never touching step 2.

3. **`test_BC_2_16_002_execute_iterates_cursor_pagination_until_null`**
   (`tests/pipeline_http_integration.rs`) — wiremock with sequential cursor responses
   (cursor present → cursor null). Asserts `records.len() == page1_count + page2_count`.
   Fails RED because stub returns empty.

4. **`test_BC_2_16_002_execute_iterates_offset_pagination_until_short_page`**
   (`tests/pipeline_http_integration.rs`) — wiremock with two offset pages (full, then partial).
   Asserts all records concatenated. Fails RED.

5. **`test_BC_2_16_002_execute_calls_auth_provider_acquire_token_on_401`**
   (`tests/pipeline_oauth_retry.rs`) — wiremock returns 401 then 200 for the data endpoint;
   second wiremock returns 200 for the token endpoint. Asserts that (a) `acquire_token` was called,
   (b) retry succeeded, (c) result is non-empty. Fails RED because stub returns empty.

6. **`test_BC_2_16_002_execute_aborts_on_double_401`**
   (`tests/pipeline_oauth_retry.rs`) — wiremock always returns 401. Asserts `execute` returns
   `Err(...)` after the retry also fails. Fails RED.

7. **`test_BC_PLUGIN_002_pipeline_executor_returns_nonempty_records_against_wiremock`**
   (`tests/pipeline_http_integration.rs`) — the canonical VP-PLUGIN-002 acceptance test.
   A single-step spec against wiremock with a known JSON fixture. Asserts non-empty records.
   Fails RED because stub returns `Ok(Vec::new())`.

8. **`test_BC_2_16_002_auth_provider_trait_object_is_object_safe`**
   (`prism-spec-engine/src/auth_provider.rs`, `#[cfg(test)]`) — verifies `AuthProvider` can be
   used as a `dyn AuthProvider`. Fails RED because `AuthProvider` trait does not yet exist in
   the file. Compile error is acceptable as a Red Gate failure.

---

## Architecture Mapping

| Component | Module | Pure/Effectful | ADR Reference |
|-----------|--------|----------------|---------------|
| `PipelineExecutor::execute` (real impl) | `prism-spec-engine/src/pipeline.rs` | Effectful (async HTTP, tokio sleep) | ADR-023 §C2; BC-2.16.002 |
| `PipelineExecutor::execute_step` (real impl) | `prism-spec-engine/src/pipeline.rs` | Effectful (HTTP) | BC-2.16.002 postcondition |
| `PipelineExecutor::fan_out_batches` (unchanged) | `prism-spec-engine/src/pipeline.rs` | Pure | BC-2.16.002 Fan-Out Behavior |
| `AuthProvider` trait + `AuthToken` | `prism-spec-engine/src/auth_provider.rs` | Pure (trait def); async impl | ADR-023 §C2; BC-2.01.013 |
| JSONPath helper (simple dot-notation) | `prism-spec-engine/src/pipeline.rs` or `jsonpath.rs` | Pure | BC-2.16.002 response_path extraction |
| `pipeline_http_integration.rs` test | `prism-spec-engine/tests/` | Effectful (wiremock server) | VP-PLUGIN-002 |
| `pipeline_oauth_retry.rs` test | `prism-spec-engine/tests/` | Effectful (wiremock server) | VP-PLUGIN-005 |

Architecture layer: `prism-spec-engine` is Layer 1.5 (sits above `prism-core` Layer 0 and
alongside `prism-sensors` Layer 1, consuming sensor specs). `PipelineExecutor` MUST NOT depend
on DataFusion or Arrow (existing AD-015 / `prism-spec-engine` compliance rule). The HTTP client
(`reqwest`) is already a production dependency. The auth interface (`AuthProvider`) lives in
`prism-spec-engine` only — it is NOT imported by `prism-sensors` or `prism-query`.


---

## Purity Classification

| Module | Classification | Justification |
|--------|----------------|---------------|
| `pipeline.rs` — `fan_out_batches` | Pure | No I/O; pure batch splitting; unchanged from stub |
| `pipeline.rs` — `execute` / `execute_step` | Effectful | Issues HTTP requests; calls tokio::time::sleep |
| `auth_provider.rs` — trait definition | Pure | No I/O in definition; impls may be effectful |
| `jsonpath.rs` helper | Pure | String parsing only; no I/O |

---

## Previous Story Intelligence

**S-PLUGIN-PREREQ-A (merged PR #142 at 90d7c80f):**
- `SensorId(Arc<str>)` newtype is now the workspace-wide sensor identity type in `prism-core`.
- `AdapterRegistry` keys by `(OrgId, SensorId)` — registry lookup uses `&SensorId`.
- `SensorSpec.sensor_id` is a `String` in the current TOML parser types — PREREQ-B does NOT
  need to migrate this to `SensorId`; that is a PREREQ-C or later concern. The string-based
  `sensor_id` in `spec_parser.rs` is acceptable for PREREQ-B scope.
- All 7 match-site dispatch groups on `SensorType` are replaced with `SensorId` open dispatch.
  PREREQ-B does not touch any of these sites.

**Key PREREQ-A lesson (LP-PR1-001):** File Structure Requirements must be derived from
workspace-wide grep for ALL definition patterns, not just call-site enumeration. Applied to
PREREQ-B: run `grep -rn "PipelineExecutor\|execute_step\|fn execute" crates/prism-spec-engine/src/`
to locate the full set of signatures that will change when the stub becomes real.

**S-3.02-FOLLOWUP-RUNTIME (merged earlier):** Filled 9 `todo!()` sites in `prism-query/src/`.
The `run_materialization_pipeline` in `materialization.rs` now calls `prism_sensors::fan_out()`
through the legacy adapter registry — this is the EXISTING path that remains in place through
Wave 0. PREREQ-B does NOT reroute materialization to use `PipelineExecutor`; the cutover is
Wave 1 scope (PLUGIN-MIGRATION-001-A/B). PREREQ-B only proves the executor works standalone.

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| `prism-spec-engine` MUST NOT depend on DataFusion or Arrow | AD-015; existing `Cargo.toml` comment | `cargo deny` / build lint; `Cargo.toml` comment is a living constraint |
| `AuthProvider` trait MUST live in `prism-spec-engine`, NOT in `prism-sensors` or `prism-query` | PREREQ-B scope boundary; SS-16 owns spec-driven auth interface | Code review; circular dep check |
| `PipelineExecutor` HTTP client MUST be injected (not a global singleton or `Lazy<reqwest::Client>`) | Testability; wiremock test requires directing traffic to mock port | Test-writer Red Gate enforces this: mock URL must be injectable |
| `fan_out_batches` MUST NOT be duplicated — the existing pure function is the canonical impl | BC-2.16.002 Fan-Out Behavior; DRY | Code review; grep for duplicated chunk logic |
| Rate-limit delays MUST use `tokio::time::sleep` (not `std::thread::sleep`) | Async executor compatibility | clippy / code review |
| The 10K limit (DI-019) MUST be applied in `execute()`, not only at the materialization layer | BC-2.16.002 invariant; defense-in-depth | AC-8 test |
| Atomic commit: all `pipeline.rs` + `auth_provider.rs` + test files land in ONE squash commit | ADR-023 §C2; AC-9 integrity | CI; PR merge strategy |

**Forbidden Dependencies:** After this story merges, `prism-spec-engine` MUST NOT gain a
dependency on `prism-sensors`, `prism-query`, or DataFusion/Arrow. The `AuthProvider` trait
in `prism-spec-engine` is a NEW interface — it does NOT wrap `SensorAuth` from `prism-sensors`.
If any implementation crate attempts `use prism_sensors::...` in `prism-spec-engine`, the
workspace dependency graph check MUST fail CI.

---

## Library & Framework Requirements

| Library | Version | Purpose |
|---------|---------|---------|
| `reqwest` | `0.12` (already in Cargo.toml production deps) | HTTP client — injected into `PipelineExecutor::execute` |
| `serde_json` | `1` (already present) | JSON response parsing, JSONPath extraction |
| `tokio` | `1` (already present) | Async runtime; `tokio::time::sleep` for rate-limit delays |
| `wiremock` | `0.6` (ADD to dev-deps — already in `prism-sensors/Cargo.toml`) | Mock HTTP server for VP-PLUGIN-002 and VP-PLUGIN-005 integration tests |
| `percent-encoding` | `2` (already present) | URL encoding in path templates (reuse existing Interpolator) |
| `tracing` | `0.1` (already present) | Structured logging of pipeline steps, retry events, auth_refresh_triggered |
| Rust stable | per `rust-toolchain.toml` (1.85+) | edition 2024; `async fn in trait` is stable — prefer over `async_trait` crate |

Do NOT add a heavy JSONPath crate (e.g., `jsonpath-rust`, `serde_json_path`). The spec grammar
uses simple dot-notation paths (`$.field`, `$.a.b`, `$.resources[*]`) only. Implement a
minimal traversal using `serde_json::Value::pointer` or recursive descent over the path string.
This keeps the dependency footprint minimal and avoids RUSTSEC advisory surface.

---

## File Structure Requirements

Derived per LP-PR1-001 from workspace-wide grep for all definition patterns. Implementer
MUST re-run these greps before the first commit to catch any additional sites.

**Grep commands for pre-implementation discovery:**
```
grep -rn "PipelineExecutor\|fn execute\|fn execute_step\|FetchContext\|PipelineResult" \
  crates/prism-spec-engine/src/ | grep -v "target/"
grep -rn "PipelineExecutor" crates/ | grep -v "target/" | grep "src/"
grep -rn "pub use pipeline" crates/prism-spec-engine/src/lib.rs
```

| File | Action | Purpose |
|------|--------|---------|
| `crates/prism-spec-engine/src/pipeline.rs` | Rewrite | Replace stub `execute` and `execute_step` with real HTTP implementations; add JSONPath helper; add rate-limit logic; add 401-retry logic using `&dyn AuthProvider`; retain unchanged `fan_out_batches` |
| `crates/prism-spec-engine/src/auth_provider.rs` | Create | `AuthProvider` trait with `acquire_token`; `AuthToken` newtype; `NullAuthProvider` (no-op for tests that don't need auth); `MockAuthProvider` helper for tests |
| `crates/prism-spec-engine/src/lib.rs` | Modify | Add `mod auth_provider;` and `pub use auth_provider::{AuthProvider, AuthToken};` alongside existing `pipeline` re-exports |
| `crates/prism-spec-engine/Cargo.toml` | Modify | Add `wiremock = "0.6"` to `[dev-dependencies]`; no new production deps (reqwest already present) |
| `crates/prism-spec-engine/tests/pipeline_http_integration.rs` | Create | wiremock-based integration tests; VP-PLUGIN-002 canonical test; interpolation, pagination, fan-out tests against mock server |
| `crates/prism-spec-engine/tests/pipeline_oauth_retry.rs` | Create | wiremock-based 401-retry tests; VP-PLUGIN-005 Red Gate |
| `crates/prism-spec-engine/tests/bc_2_16_002_test.rs` | Modify | Update `test_BC_2_16_002_two_step_pipeline_step2_uses_step1_token` call site to new signature (inject `reqwest::Client` + `&dyn AuthProvider`); add wiremock wiring or mark `#[ignore]` with justification comment |

**Files verified NOT to need changes (from grep):**
- `prism-query/src/materialization.rs` — the legacy `prism_sensors::fan_out()` path is unchanged; PREREQ-B does NOT reroute materialization
- `prism-sensors/src/` — no changes; `AuthProvider` is a NEW trait in `prism-spec-engine`, not a migration of `SensorAuth`
- `prism-core/src/` — no changes; `SensorId` was delivered by PREREQ-A
- `prism-bin/src/` — no changes; boot sequence wiring is PREREQ-D scope

---

## Match-Site / Stub Replacement Inventory

These are the exact stub bodies that this story replaces (verified by reading pipeline.rs):

| # | Location | Line | Current Stub | Replacement |
|---|----------|------|-------------|-------------|
| 1 | `pipeline.rs:PipelineExecutor::execute` | 54–65 | `Ok(PipelineResult { records: Vec::new(), table_name: table.table_name.clone(), request_count: 0 })` | Real HTTP multi-step execution per BC-2.16.002 |
| 2 | `pipeline.rs:PipelineExecutor::execute_step` | 72–79 | `Ok(serde_json::Value::Null)` | Real HTTP request → parse JSON → extract at `response_path` |

**Signature changes** (implementer must update all call sites):

```rust
// Before:
pub async fn execute(
    _spec: &SensorSpec,
    table: &TableSpec,
    _context: &FetchContext,
) -> Result<PipelineResult, PrismError>

// After (one acceptable form — implementer may choose struct wrapping):
pub async fn execute(
    spec: &SensorSpec,
    table: &TableSpec,
    context: &FetchContext,
    http_client: &reqwest::Client,
    auth_provider: &dyn AuthProvider,
) -> Result<PipelineResult, PrismError>
```

Call site in `tests/bc_2_16_002_test.rs:241` must be updated to match the new signature.

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `FetchStep.pagination = None` (no pagination) | Execute exactly one request; return its records; no loop |
| EC-002 | Cursor-paginated step returns `cursor = null` on first page | Stop after first page; records from page 1 only |
| EC-003 | Offset-paginated step returns 0 records on first page | `PipelineResult.records` is empty; no error |
| EC-004 | Step 1 returns empty array; step 2 depends on step 1 output | Step 2 receives empty variable; fan-out produces zero batches; zero records; no error (BC-2.16.002 edge case) |
| EC-005 | HTTP 401 on first request; `acquire_token` returns fresh token; retry returns 200 | Single retry succeeds; records returned; `auth_refresh_triggered` logged |
| EC-006 | HTTP 401 on first request; `acquire_token` returns fresh token; retry also returns 401 | `Err(SpecEngineError::AuthRefreshFailed)` or equivalent; no infinite loop |
| EC-007 | HTTP 500 on a non-final step | Pipeline aborts; `sensor_error` returned (consistent with BC-2.01.010 partial failure; abort is correct for non-transient server error) |
| EC-008 | `response_path` does not match response structure | `E-SPEC-010` — step name and path included in error (BC-2.16.002 error table) |
| EC-009 | `fan_out_batch_size = 0` (invalid) | Implementer may treat as `batch_size = 1` OR return a structured error; must not panic |
| EC-010 | `rate_limit_hints.requests_per_second = Some(0.0)` (divide by zero) | Treat as "no rate limit" (skip delay); must not panic with divide-by-zero |
| EC-011 | `base_url` has trailing slash; `path_template` has leading slash | Normalize to single slash at join boundary; avoid double-slash URLs |
| EC-012 | Final `PipelineResult.records.len()` approaches 10K limit (DI-019) | Truncate at 10,000; set `truncated: true` flag (or return partial error); never return > 10K records |
| EC-013 | Cyberint DTU clone `incidents` endpoint pagination gap (noted in ADR-023 §C2) | AC-9 test uses a generic wiremock fixture — the Cyberint DTU gap is a Wave-1/D concern, not PREREQ-B scope. PREREQ-B passes if wiremock fixture returns non-empty records. |

---

## Implementation Notes

**HttpClient injection pattern:** The existing `reqwest::Client` in `prism-spec-engine` is used
in the WASM plugin host (`plugin/mod.rs`). Do NOT reuse that client for `PipelineExecutor` —
the plugin runtime has its own lifecycle. Inject a separate `&reqwest::Client` into `execute`.
Tests create a `reqwest::Client::new()` per test (cheap; no connection pool reuse needed in tests).

**JSONPath implementation scope:** The spec defines paths like `$.resources`, `$.data.items`,
`$.access_token`, `$.pagination.cursor`. The `$` prefix is conventional but the actual traversal
is simple object key descent. Use `serde_json::Value::pointer("/resources")` (RFC 6901 JSON
Pointer format, replacing `$.` prefix) or a manual recursive match. Do NOT implement `$[*]`
wildcard selectors — they are not required by the TOML grammar defined in `spec_parser.rs`.

**`async fn in trait` (Rust 1.85+):** The workspace targets Rust stable with edition 2024. Use
`async fn` directly in the `AuthProvider` trait — no `#[async_trait]` macro needed. The RPITIT
(return-position impl trait in trait) is stable as of Rust 1.75. Verify against `rust-toolchain.toml`.

**`NullAuthProvider` for non-auth tests:** For tests that only verify pagination or fan-out (not
auth), implement a `NullAuthProvider` that returns an empty bearer token. Tests that exercise auth
must use a `MockAuthProvider` that records calls and returns configurable tokens.

**Do NOT wire `PipelineExecutor` into `materialization.rs`:** The `run_materialization_pipeline`
in `prism-query` continues to call `prism_sensors::fan_out()` through Wave 0. The cutover is
Wave 1 (PLUGIN-MIGRATION-001-A/B). PREREQ-B only proves standalone executor correctness.

**`PipelineResult` struct extension:** Consider adding a `truncated: bool` field to
`PipelineResult` for DI-019 enforcement (AC-8). The struct is currently defined in `pipeline.rs`
and re-exported via `lib.rs`. Adding a field is a non-breaking internal change (no downstream
callers outside the crate in production code).

**Audit log for auth refresh:** When `acquire_token` is called on 401 retry, emit a tracing
event at WARN level: `event_type: auth_refresh_triggered, sensor_id, client_id`. This satisfies
the VP-PLUGIN-005 assertion (d) in ADR-023 §E: "the auth re-acquisition is recorded in the
audit log as `event_type: auth_refresh_triggered`".

---

## Out of Scope

- Authoring production TOML sensor specs for CrowdStrike/Cyberint/Claroty/Armis (PLUGIN-MIGRATION-001-D)
- TOML grammar extensions (`[fetch_step.retry]`, `[fetch_step.batch]`, `virtual_field_aliases`, `cache_ttl_secs`) — those are PREREQ-C
- Wiring `PluginRuntime` into boot sequence (PREREQ-D)
- Un-sealing `SensorAuth` or removing `CustomAdapter` (PREREQ-E)
- Rerouting `prism-query` materialization from `prism_sensors::fan_out()` to `PipelineExecutor` (Wave 1)
- DTU parity testing against real Cyberint/CrowdStrike/Claroty/Armis responses (Wave 1)
- The Cyberint DTU clone `incidents` endpoint pagination gap identified in ADR-023 §C2 (Wave 1/D verification)
- `PluginRuntime` WASM plugin dispatch hook points within the executor (PREREQ-D prerequisite not yet delivered)
- Performance profiling or benchmarking of the executor under load

---

## Risks and Tech-Debt Surfaces

| ID | Risk | Likelihood | Impact | Mitigation |
|----|------|-----------|--------|-----------|
| R-PB-001 | `PipelineExecutor` signature change breaks call sites in `tests/bc_2_16_002_test.rs` | HIGH (known) | LOW (one test file, easy fix) | Task 7 explicitly covers the update; Red Gate test-writer must update before implementation |
| R-PB-002 | Cyberint DTU clone pagination gap (ADR-023 §C2) may block VP-PLUGIN-003 later | MEDIUM | MEDIUM | EC-013 explicitly defers to Wave-1/D; PREREQ-B uses wiremock, not DTU clone |
| R-PB-003 | `reqwest::Client` connection pool contention if tests share a global client | LOW | LOW | Each wiremock test creates its own `reqwest::Client::new()` (stateless; cheap) |
| R-PB-004 | JSONPath simple-descent may not cover future spec grammar evolution | LOW | MEDIUM | Explicitly out of scope for this story; TD filed if gap emerges in PREREQ-C |
| R-PB-005 | `PipelineResult` struct change (add `truncated` field) breaks serialization in callers | LOW | LOW | No external serialized consumers yet; internal change is safe |
| R-PB-006 | wiremock 0.6 dev-dep not yet in prism-spec-engine Cargo.toml | HIGH (known gap) | LOW (one-line fix) | Task 4 explicitly adds it to dev-deps |

---

## Green Gate Definition of Done

This story is shipped when ALL of the following are true:

1. `cargo build -p prism-spec-engine` is clean (zero errors, zero warnings with `-D warnings`)
2. `just iter prism-spec-engine` passes — all existing fan-out/interpolation unit tests still green
3. `test_BC_PLUGIN_002_pipeline_executor_returns_nonempty_records_against_wiremock` passes (VP-PLUGIN-002 fulfilled)
4. `test_BC_2_16_002_execute_calls_auth_provider_acquire_token_on_401` passes (VP-PLUGIN-005 Red Gate)
5. `grep -rn "Ok(Vec::new())" crates/prism-spec-engine/src/pipeline.rs` returns zero hits
6. `grep -rn "Stub: full HTTP execution in S-1.12" crates/prism-spec-engine/src/pipeline.rs` returns zero hits
7. `AuthProvider` trait is exported from `prism-spec-engine` (visible via `cargo doc -p prism-spec-engine`)
8. AD-015 compliance: `cargo tree -p prism-spec-engine | grep -E "datafusion|arrow"` returns zero hits
9. PR is squash-merged into `develop` as exactly ONE commit
10. STORY-INDEX row transitions to `status: merged` with PR# recorded

---

## Changelog

| Version | Burst | Date | Author | Changes |
|---------|-------|------|--------|---------|
| 1.11 | prereq-b-fix-burst-9 | 2026-05-11 | state-manager | D-415 fix-burst-9 CLOSED 2 MED + 1 LOW + 1 OBS-bundled. red_gate_tests 45→47 (+2). F-LP9-MED-001: BC-2.16.002 v1.6→v1.7 audit-signal enumeration (3 distinct events: auth_initial_acquired info, auth_initial_acquired_empty debug, auth_initial_failed error). F-LP9-MED-002: find_fan_out_array runtime warn fanout_ambiguous_multi_array when >=2 array-valued variables referenced. F-LP9-LOW-001: dead Mock 1 (query_param cursor="") deleted. OBS-LP9-003: cursor_preview UTF-8 boundary fix (char_indices nth(100) vs byte-slice). 280/280 tests pass. just check-fast clean. Worktree HEAD f5746553. BC-INDEX v4.57→v4.58. STORY-INDEX v2.48→v2.49. STATE+HANDOFF v7.148→v7.149. |
| 1.10 | prereq-b-pass-9 | 2026-05-11 | state-manager | pass-9 record: 2 MED + 1 LOW + 4 OBS (incl 1 [process-gap]) filed; streak 0/3 unchanged; fix-burst-9 dispatched. F-LP9-MED-001: BC-2.16.002 v1.6 audit-signal row says "two events" but impl emits three (auth_initial_acquired_empty added in fix-burst-7 not enumerated in BC). F-LP9-MED-002: validator multi-array heuristic misses non-paginated whole-array response_path (sibling-sweep gap on F-LP8-LOW-001 — silent data corruption regression). F-LP9-LOW-001: dead Mock 1 in rewritten partial-discard test. OBS-LP9-003: cursor_preview UTF-8 boundary panic — genuine bug, bundled into fix-burst-9. Trajectory 20→10→4→7→10→9→8→4→4 (pass-9 adds 4 findings; novelty score 3/3=1.0). Worktree HEAD 411f4cbf unchanged. Report: .factory/code-delivery/S-PLUGIN-PREREQ-B/adversarial-review/local-pass-9.md. |
| 1.9 | prereq-b-fix-burst-8 | 2026-05-11 | state-manager | D-413 fix-burst-8 CLOSED 3 MED + 1 LOW. red_gate_tests 41→45 (+4). F-LP8-MED-001: test_BC_2_16_002_auth_initial_acquired_emits_distinct_events_per_token_state added — sub-case (b) asserts auth_initial_acquired_empty appears AND auth_initial_acquired absent (load-bearing per TD-VSDD-059). F-LP8-MED-002: partial-discard test rewritten with CursorToken paginated pipeline + real record accumulation (page-1 2 records, page-2 500) — assert!(result.is_err()) now regression-load-bearing. F-LP8-MED-003: extract_cursor warn enriched with event_type="pagination_cursor_unsupported_type" structured field + test captures tracing buffer. F-LP8-LOW-001: validation.rs Category 2b multi-array fan-out check added — validator rejects specs referencing >1 array-valued variable; test asserts Err(ValidationError). just check-fast clean. 278/278 tests pass. Worktree HEAD 411f4cbf. Factory commit: this burst. |
| 1.8 | prereq-b-pass-8 | 2026-05-11 | state-manager | D-412 pass-8 BLOCKED-soft record: 3 MED + 1 LOW + 4 OBS filed; streak 0/3 unchanged; fix-burst-8 dispatched. F-LP8-MED-001: empty-token branch (auth_initial_acquired_empty) has zero test assertions — paper-fix (TD-VSDD-059). F-LP8-MED-002: partial-discard test uses scalar response_path so all_records always empty before failure — paper-fix (TD-VSDD-059). F-LP8-MED-003: extract_cursor non-string termination emits bare warn without event_type structured field. F-LP8-LOW-001: multi-array fan-out templates silently use first array only. 4 OBS non-blocking (request_count semantics; POL-15 deferred PREREQ-D; execute_step zero call sites TD-012; BC postcondition amendment SOP process-gap). Trajectory 20→10→4→7→10→9→8→4. Worktree HEAD ebd9a3ec (no code change). Pass-8 report: .factory/code-delivery/S-PLUGIN-PREREQ-B/adversarial-review/local-pass-8.md. |
| 1.7 | prereq-b-fix-burst-7 | 2026-05-11 | state-manager | D-411 fix-burst-7 CLOSED 3 MED + 1 LOW (deferred). red_gate_tests 39→41 (2 new: test_BC_2_16_002_eager_auth_initial_failed_aborts_pipeline_immediately + test_BC_2_16_002_execute_discards_partial_records_on_mid_pipeline_500). F-LP7-MED-001: empty-token branch in execute() + execute_step() emits auth_initial_acquired_empty (debug) vs auth_initial_acquired (info) — audit signal integrity restored. F-LP7-MED-002: FailingAuthProvider added under cfg(feature="test-helpers"); abort test verifies zero HTTP requests on auth failure. F-LP7-MED-003: partial-record discard test + BC-2.16.002 v1.5→v1.6 postcondition amendment (all-or-nothing semantics). F-LP7-LOW-001: DEFERRED as TD-S-PLUGIN-PREREQ-B-012 P3 (execute_step PREREQ-D test vehicle; doc comment added at pipeline.rs:424). 3 OBS acknowledged. 275/275 prism-spec-engine tests pass + 1 skipped. Worktree HEAD ebd9a3ec. Factory commit d11dbf0d. |
| 1.6 | prereq-b-fix-burst-6 | 2026-05-11 | product-owner | SPEC-SIDE pass-6 fixes. F-LP6-HIGH-001 CLOSED: VP-PLUGIN-002 anchor corrected from PLUGIN-MIGRATION-001-D to S-PLUGIN-PREREQ-B in VP-INDEX (VP-147 numbered row + VP-PLUGIN-002 named-alias row). VP-147 description updated from stale "Unknown sensor registers without code change" to "PipelineExecutor::execute returns non-empty records against wiremock DTU clone". F-LP6-HIGH-002 CLOSED: VP-PLUGIN-005 (VP-150) description in VP-INDEX named-alias row (line 187) corrected from "PluginRuntime::load_all_plugins boot-time wired; allowed_urls host-only enforcement after PREREQ-D" to "OAuth2 refresh-on-401 via declarative TOML retry policy"; anchor updated PLUGIN-MIGRATION-001-D → S-PLUGIN-PREREQ-B (both VP-150 numbered row and VP-PLUGIN-005 named-alias row). F-LP6-MED-002 CLOSED: verification_properties frontmatter + anchor_vps extended to include VP-PLUGIN-005 (pipeline_oauth_retry.rs test target). VP-INDEX v1.30 → v1.31. |
| 1.5 | prereq-b-fix-burst-5 | 2026-05-11 | state-manager | LOCAL fix-burst-5 CLOSED F-LP5-LOW-003 (D-408). red_gate_tests 37→39 (2 new: test_BC_2_16_002_execute_acquires_token_eagerly_before_first_request + test_BC_2_16_002_no_auth_refresh_triggered_on_legitimate_execution). Eager-token implementation: pipeline.rs acquires token at pipeline start — unconditional for all AuthType variants (no Null variant exists). 2 existing tests adjusted for acquire_token call-count semantics (1→2 for refresh scenarios). BC-2.16.002 v1.4→v1.5 amendment: precondition lifecycle lazy→eager; request_count counts HTTP requests only (excludes acquire_token transport); auth_initial_acquired/auth_initial_failed audit family added; auth_refresh_* family fully enumerated. TD-S-PLUGIN-PREREQ-B-010 CLOSED. 273/273 tests pass. Worktree HEAD 2fe7068c. Factory commit 82fd868c. |
| 1.4 | prereq-b-fix-burst-4 | 2026-05-11 | state-manager | LOCAL pass-5 fix-burst-4 CLOSED 3 actionable findings (D-407). red_gate_tests 33→37 (4 new: MED-001 gzip-decode wiremock+flate2; MED-002(c) pipeline_truncated tracing-subscriber; LOW-001 extract_at_path dollar-dot negative + validator). F-LP5-MED-001 reqwest features=[gzip,deflate,brotli] added to Cargo.toml. F-LP5-MED-002 audit-log symmetry: auth_refresh_succeeded/failed on acquire_token Ok/Err; auth_refresh_double_401 before abort; pipeline_truncated before 10K break. F-LP5-LOW-001 dollar-dot double defense: runtime guard + validator rejection. 4 TDs filed: TD-S-PLUGIN-PREREQ-B-006 P2 (proptest); -007 P3 (status_code overload); -008 P3 (template escape); -009 P3 (dead scalar arm). F-LP5-LOW-003 lazy-token SURFACED as TD-010 P2 — ORCHESTRATOR-DECISION-PENDING. 2 OBS acknowledged. 271/271 tests pass. Worktree HEAD e19372f4. |
| 1.3 | prereq-b-fix-burst-3 | 2026-05-11 | state-manager | LOCAL pass-4 fix-burst-3 CLOSED (D-405). 7 actionable findings closed (1 HIGH + 2 MED + 4 LOW). red_gate_tests 29→33 (4 new: F-LP4-HIGH-001 paper-fix-proof rejection test + accept-1 + accept-None; F-LP4-MED-002 MAX_PAGES_PER_STEP regression). F-LP4-HIGH-001 double defense: validation.rs:247 validator reject + pipeline.rs:451 `.max(1)` runtime clamp. F-LP4-MED-001 reqwest timeout: 18 test fixture sites updated to builder pattern; TD-S-PLUGIN-PREREQ-B-005 P2 filed for PREREQ-D production wiring. F-LP4-MED-002 MAX_PAGES_PER_STEP regression test at pipeline_http_integration.rs:1538. 4 LOW: execute_step docstring rewritten; PipelineResult/FetchContext `#[non_exhaustive]` + FetchContext::new() constructor; Duration `.min(3600.0)` clamp; AuthToken private field + as_str(). 267/267 tests pass. Worktree HEAD d5a12e4a. 1 TD filed: TD-S-PLUGIN-PREREQ-B-005 P2 (production reqwest::Client.timeout; PREREQ-D scope). |
| 1.2 | prereq-b-fix-burst-2 | 2026-05-11 | state-manager | LOCAL pass-2 fix-burst-2 CLOSED (D-402). 8 actionable findings closed (2 HIGH + 3 MED + 3 LOW). red_gate_tests corrected 16→29 (F-LP2-MED-001; canonical-name grep + 1 error.rs unit = 30 total). 5 new Red Gate tests added (paper-fix-proof fan-out + cursor abort + array CT + numeric cursor + variant construction). 2 OBS acknowledged (OBS-LP2-001 partial-cover via MAX_PAGES guard + TD-S-PLUGIN-PREREQ-B-004; OBS-LP2-002 links PG-LP7-002). 2 TDs filed: TD-S-PLUGIN-PREREQ-B-003 P3 (JSON Pointer bracket/wildcard PREREQ-C scope), TD-S-PLUGIN-PREREQ-B-004 P3 (MAX_REQUESTS_PER_PIPELINE full bound; PREREQ-D scope). Worktree HEAD a6895d7a. |
| 1.1 | prereq-b-fix-burst-1 | 2026-05-11 | state-manager | LOCAL pass-1 fix-burst-1 CLOSED (D-400). 12 actionable findings closed (4 CRIT + 5 HIGH + 3 MED). 8 net new Red Gate tests (7 new + 1 upgraded crowdstrike→wiremock); red_gate_tests 8→16. 2 TDs filed: TD-S-PLUGIN-PREREQ-B-001 P2 (cursor page_size first-call; PREREQ-C scope), TD-S-PLUGIN-PREREQ-B-002 P3 (AuthToken zeroize; PREREQ-D scope). Worktree HEAD 7511e749; BC-2.16.002 v1.4 amendment at factory-artifacts c2e7b376. |
| 1.0 | prereq-b-materialization | 2026-05-11 | story-writer | Initial story creation from ADR-023 §C2 + workspace grep of PipelineExecutor stub + BC-2.16.002 postcondition tracing. 9 ACs, 8 Red Gate tests. AuthProvider trait scoped, JSONPath implementation notes added. Stub replacement inventory from pipeline.rs read. Subsystem anchors justified per PG-PR1-002. Forbidden dependency rules explicit. LP-PR1-001 file-structure derivation applied. |
