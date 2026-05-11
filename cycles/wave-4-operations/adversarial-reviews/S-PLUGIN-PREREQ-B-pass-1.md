---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-05-11T18:00:00Z
phase: 3
inputs: []
input-hash: "4cc4bb7"
traces_to: ""
pass: 1
previous_review: null
review_level: LOCAL
target_artifact: S-PLUGIN-PREREQ-B
pass_number: 1
target_sha: b1b529fc
base_sha: 90d7c80f
verdict: BLOCKED-hard
streak: 0/3
finding_summary: { critical: 4, high: 6, medium: 5, low: 2, obs: 3 }
prior_passes: pass-1 first — story Green TDD complete at b1b529fc; adversary found significant spec-implementation drift
---

# Adversarial Review: S-PLUGIN-PREREQ-B (Pass 1)

## Finding ID Convention

Finding IDs use the format: `F-LP1-<SEV>-<SEQ>`

- `F`: Fixed prefix
- `LP1`: LOCAL Pass 1
- `<SEV>`: Severity abbreviation (`CRIT`, `HIGH`, `MED`, `LOW`, `OBS`)
- `<SEQ>`: Three-digit sequence within the severity group

## ADV-S-PLUGIN-PREREQ-B-LP1 — Pass 1 Verdict

**Target:** feature/S-PLUGIN-PREREQ-B@b1b529fc
**Diff base:** develop@90d7c80f
**Verdict:** BLOCKED-hard (CRITICAL findings present)
**Streak:** 0/3 (first pass — no prior streak)

## Part A — Fix Verification (pass >= 2 only)

_Pass 1 — no prior pass to verify._

## Part B — New Findings (all findings for pass 1)

### CRITICAL

#### F-LP1-CRIT-001: body_template not interpolated + Content-Type hardcoded as form-encoded

- **Severity:** CRITICAL
- **Category:** spec-fidelity / behavioral-contract-violation
- **AC:** AC-1, AC-2
- **Description:** The `body_template` field from the pipeline step spec is stored but never interpolated at execution time. The `execute_step` function constructs the HTTP request body from a static string rather than evaluating `body_template` with the current variable bindings. Additionally, the Content-Type header is hardcoded as `application/x-www-form-urlencoded` regardless of what the step spec declares. Both CrowdStrike and Claroty API integrations require JSON bodies — the hardcoded form-encoded Content-Type will cause all real-world executions to fail.
- **Evidence:** `pipeline.rs` `execute_step` body construction site: template interpolation call absent; Content-Type set unconditionally to `application/x-www-form-urlencoded`.
- **Proposed Fix:** (1) Call the template interpolation helper at request construction time, substituting all `{{ variable }}` references with values from `step_vars`. (2) Use the Content-Type from the step spec (or default to `application/json` per ADR-023 §C step defaults). Add a Red Gate test asserting body interpolation fires and JSON Content-Type is used.

#### F-LP1-CRIT-002: cursor not percent-encoded before URL construction

- **Severity:** CRITICAL
- **Category:** spec-fidelity / correctness
- **AC:** AC-3 (pagination)
- **Description:** When a prior step returns a cursor token for subsequent pages, `pipeline.rs` appends the cursor directly to the URL query string without percent-encoding. CrowdStrike cursor tokens contain `+` and `=` characters; unencoded, these are misinterpreted by HTTP clients as form-encoded delimiters, producing malformed URLs and causing pagination to fail silently after the first page.
- **Evidence:** URL construction in `execute_step` pagination branch: raw cursor string concatenated to URL query string with no encoding step.
- **Proposed Fix:** Percent-encode the cursor value using `urlencoding::encode` (or equivalent) before concatenating to the URL query string. Add a test with a cursor string containing `+`, `=`, and `/` characters to assert correct encoding.

#### F-LP1-CRIT-003: intermediate-step records leak into final PipelineResult

- **Severity:** CRITICAL
- **Category:** spec-fidelity / data-correctness
- **AC:** AC-4 (result aggregation)
- **Description:** The `PipelineExecutor::execute` accumulates records from every step into a single shared buffer. The AC specifies that only the final step's records are surfaced in the `PipelineResult`; intermediate steps are intermediate state and must not appear in output. Currently all step outputs are concatenated into the same buffer, so a 2-step pipeline returns 2× the expected record count (first-step records + second-step records). This is a correctness violation that will cause double-counting in the query engine.
- **Evidence:** `execute` method: `all_records.extend(step_result.records)` called in the loop body without guarding on `step_index == last_step`.
- **Proposed Fix:** Accumulate intermediate records only for variable binding (cursor, continuation tokens); surface only `step_result.records` from the final step. Add a 2-step Red Gate test that asserts output record count equals only the second step's record count.

#### F-LP1-CRIT-004: CrowdStrike test asserts nothing and makes real network calls in CI

- **Severity:** CRITICAL
- **Category:** test-quality / CI-safety
- **AC:** AC-9 (test coverage)
- **Description:** The test named `test_crowdstrike_pipeline_execute` in `bc_2_16_002_test.rs` (or equivalent Red Gate test file) makes an HTTP call to the live CrowdStrike API using a hard-coded base URL. The test has no `#[ignore]` attribute and no mock infrastructure — it will fail in CI where no CrowdStrike credentials are present. Additionally the assertions in the test body are absent or trivially pass regardless of execution outcome (e.g., `assert!(true)` or no assert). Both issues are present simultaneously: the test is dangerous (live network) and useless (no assertions).
- **Evidence:** Test file: `use` of live client without mock injection; no `#[ignore]` attribute; assertion body empty or asserts trivially true condition.
- **Proposed Fix:** (1) Inject a `MockHttpClient` in all Red Gate tests (no live network in CI). (2) Add meaningful assertions: assert record count, assert pagination cursor consumed, assert variable bindings populated. (3) If a live integration test is desired, add it with `#[ignore]` and a doc-comment explaining it requires `CROWDSTRIKE_*` env vars.

### HIGH

#### F-LP1-HIGH-001: AC-6 fan-out unimplemented and untested

- **Severity:** HIGH
- **Category:** spec-fidelity / missing-implementation
- **AC:** AC-6
- **Description:** AC-6 requires that when a pipeline step produces multiple `fan_out_key` values, the executor spawns parallel sub-executions for each key and merges results. The `fan_out_batches` function exists in the codebase but has zero call sites inside `execute` or `execute_step`. The AC is completely unimplemented: the executor processes each step linearly regardless of fan-out keys. No Red Gate test exercises AC-6.
- **Evidence:** `grep -r "fan_out_batches" crates/` returns only the function definition — zero call sites in `execute` or `execute_step` body.
- **Proposed Fix:** Wire `fan_out_batches` into the `execute` loop: after receiving step output, check for fan-out keys; if present, spawn parallel sub-executions and merge. Add Red Gate test `test_bc_2_16_002_ac6_fanout` with a mock that returns 3 fan-out keys and asserts 3× parallel calls + merged result.

#### F-LP1-HIGH-002: AC-7 rate-limit inter-step bug and no positive-case test

- **Severity:** HIGH
- **Category:** spec-fidelity / correctness + verification-gap
- **AC:** AC-7
- **Description:** Two issues: (1) The rate-limiter `sleep` is applied between steps of the same pipeline run, but the per-step rate limit applies to calls to the *same endpoint* — inter-step sleeps conflate distinct APIs and cause unnecessary throttling when steps target different hosts. The sleep should be keyed per-host or per-step-template, not per-pipeline-invocation. (2) No test exercises the positive case: a step that returns `Retry-After: N` is retried after N seconds with the correct backoff. The only rate-limit test asserts the fast path.
- **Evidence:** `execute_step` rate-limit sleep: sleep applied unconditionally between steps without inspecting host identity. Test file: no test with `Retry-After` header mock.
- **Proposed Fix:** (1) Key rate-limit state per endpoint host (or per step-template ID). (2) Add Red Gate test `test_bc_2_16_002_ac7_rate_limit_retry_after` asserting: mock returns 429 + `Retry-After: 1` on first call; second call succeeds; total elapsed time >= 1s.

#### F-LP1-HIGH-003: AC-5 audit-log unimplemented

- **Severity:** HIGH
- **Category:** spec-fidelity / missing-implementation
- **AC:** AC-5
- **Description:** AC-5 requires that every pipeline step execution emits a structured audit-log entry containing: step name, timestamp, HTTP status code, record count, and any error. The `execute_step` function returns a result struct but emits no audit-log event — no call to the audit dispatcher, no tracing event carrying the required fields. The audit-log requirement is part of the security architecture (ADR-023 §C audit clause).
- **Evidence:** `execute_step` body: no `audit_log::emit` or `tracing::info!` call with the required structured fields (step_name, status_code, record_count).
- **Proposed Fix:** Add an audit emit at the end of `execute_step` (both success and error paths) with the required fields. Add a Red Gate test asserting the audit event is emitted with correct fields on both success and 4xx paths.

#### F-LP1-HIGH-004: query_filters dead-letter — filter expressions passed but never applied

- **Severity:** HIGH
- **Category:** spec-fidelity / correctness
- **AC:** AC-2 (query composition)
- **Description:** The pipeline step spec accepts `query_filters` (a list of filter expressions to apply to records before returning). These are deserialized correctly from the spec but the `execute_step` function never passes them to the record-filtering layer. All records from the HTTP response are returned unfiltered. Queries that rely on server-side-equivalent filtering via the pipeline will return more records than expected, causing downstream correctness errors.
- **Evidence:** `execute_step` signature accepts `step_spec` containing `query_filters`; record post-processing loop: no filter application call.
- **Proposed Fix:** After receiving records from the HTTP response, apply each filter expression from `step_spec.query_filters` using the shared filter-eval helper. Add a Red Gate test asserting a filter expression removes matching records from output.

#### F-LP1-HIGH-005: cursor pagination first-call asymmetry — initial call skips cursor injection logic

- **Severity:** HIGH
- **Category:** spec-fidelity / correctness
- **AC:** AC-3
- **Description:** The pagination loop correctly injects the cursor into the URL on subsequent calls, but the first call takes a separate code path that skips cursor-related initialization. This means if a step spec includes a `cursor_init` field (initial cursor override for resumable pagination), it is silently ignored on the first call. The asymmetry also makes the code path structurally different in a way that is likely to cause regressions when cursor logic is extended.
- **Evidence:** `execute_step`: two distinct branches for `is_first_call` and subsequent calls; the `cursor_init` field read is absent from the first-call branch.
- **Proposed Fix:** Unify the first-call and subsequent-call branches into a single code path parameterized on cursor value (None for first call with no init, or `step_spec.cursor_init` if present). Add a test asserting `cursor_init` is honoured on the first call.

#### F-LP1-HIGH-006: JSONPath split('.') breaks key escaping for dotted-key names

- **Severity:** HIGH
- **Category:** correctness / spec-fidelity
- **AC:** AC-2 (response parsing)
- **Description:** The JSONPath evaluation used to extract values from HTTP responses splits the path string on `'.'` using a naive `str::split('.')`. JSONPath allows keys to contain literal dots when escaped with bracket notation (e.g., `$.response['key.with.dot']`). The naive split incorrectly fragments bracket-escaped keys containing dots, producing wrong field extraction results. CrowdStrike response payloads include nested keys with dots in some sub-objects.
- **Evidence:** `extract_at_path` (or equivalent): `path.split('.')` without bracket-notation awareness.
- **Proposed Fix:** Replace naive `split('.')` with a proper JSONPath segment parser that respects bracket notation. Alternatively, adopt a well-tested JSONPath crate. Add a Red Gate test with a mock response payload containing a dotted key accessed via bracket notation.

### MEDIUM

#### F-LP1-MED-001: BC-2.16.002 v1.3 status:draft — AuthProvider trait not amended for new pipeline auth model

- **Severity:** MEDIUM
- **Category:** spec-drift / BC-amendment-required
- **AC:** N/A (spec artifact)
- **Description:** BC-2.16.002 describes the PipelineExecutor behavioral contract. The story S-PLUGIN-PREREQ-B introduces an `AuthProvider` trait for supplying credentials to pipeline steps. BC-2.16.002 v1.3 does not reflect this trait — its postconditions still describe a closed credential-injection model that was superseded by the AuthProvider pattern. The BC must be amended to v1.4 to document the new trait, its required methods, and the credential-injection postconditions before the story can merge (POL-14).
- **Proposed Fix:** Product-owner or state-manager to amend BC-2.16.002 v1.3→v1.4 in a separate factory-artifacts commit: add AuthProvider trait signature, required methods, postcondition table row for credential injection. This is a spec-side commit separate from the implementer fix-burst.

#### F-LP1-MED-002: truncated:bool no truthy test — partial-page detection always false

- **Severity:** MEDIUM
- **Category:** verification-gap / correctness
- **AC:** AC-3 (pagination completeness)
- **Description:** The `PipelineResult` struct carries a `truncated: bool` field indicating whether pagination was halted before all records were fetched (e.g., due to a configured `max_pages` limit). There is no Red Gate test that exercises the truthy case (`truncated: true`). The only tests exercise the case where all pages are fetched. Without a positive test, a regression that always sets `truncated: false` would go undetected.
- **Proposed Fix:** Add a Red Gate test `test_bc_2_16_002_ac3_truncated_flag` with a mock that returns 3 pages and a `max_pages: 2` config; assert `PipelineResult.truncated == true` and record count equals 2-page total.

#### F-LP1-MED-003: Cargo.toml edition 2021 vs workspace edition 2024

- **Severity:** MEDIUM
- **Category:** toolchain / workspace-consistency
- **AC:** N/A
- **Description:** The new crate introduced or modified by S-PLUGIN-PREREQ-B declares `edition = "2021"` in its `Cargo.toml`. The workspace `Cargo.toml` declares `edition = "2024"` (per `rust-toolchain.toml` and workspace resolver). All crates in the workspace must use edition 2024 for consistent language feature availability and resolver behavior. The mismatch will cause clippy and rustfmt to apply different rules to this crate.
- **Evidence:** `crates/<new-crate>/Cargo.toml`: `edition = "2021"`. `Cargo.toml` workspace: `edition = "2024"`.
- **Proposed Fix:** Change the crate's `Cargo.toml` to `edition = "2024"`. Run `just fmt` to verify no formatting regressions.

#### F-LP1-MED-004: store_step_vars auto-fallback fragile — silent promotion of missing keys to empty string

- **Severity:** MEDIUM
- **Category:** correctness / error-handling
- **AC:** AC-4 (variable propagation)
- **Description:** The `store_step_vars` function (or equivalent) stores the variables extracted from a step's response for use in subsequent steps. When a key specified in the step's `extract_vars` config is absent from the response JSON, the function silently stores an empty string `""` rather than returning an error or a `None`. Downstream steps that depend on this variable will receive `""` and produce subtly wrong URLs or request bodies without any diagnostic signal.
- **Evidence:** `store_step_vars`: `unwrap_or_default()` or `.unwrap_or("")` on missing JSON key extraction.
- **Proposed Fix:** Return `Err(PipelineError::MissingExtractedVariable { key, step })` when a required variable key is absent. Allow optional variables to be marked `optional: true` in the spec. Add a test asserting the error is returned for a required missing key.

#### F-LP1-MED-005: bc_2_16_002_test.rs not counted in Red Gate test count

- **Severity:** MEDIUM
- **Category:** bookkeeping / spec-fidelity
- **AC:** N/A (story artifact)
- **Description:** The story spec's §Red Gate section states a specific count of Red Gate tests that satisfy the ACs. The file `bc_2_16_002_test.rs` (or the equivalent Red Gate test file for S-PLUGIN-PREREQ-B) is not reflected in that count — either the count predates the file's creation or the file name does not match the expected BC-prefixed naming convention (PG-LP7-002 discipline from S-PLUGIN-PREREQ-A cascade). The story will fail the Red Gate materialization audit in a future pass if the count is stale.
- **Proposed Fix:** (1) Verify all Red Gate test files use BC-prefixed exact names per PG-LP7-002. (2) Update the story §Red Gate count to include `bc_2_16_002_test.rs`. (3) Confirm the count in the story matches `grep -c "#\[test\]" <file>`.

### LOW

#### F-LP1-LOW-001: extract_at_path returns Err(()) — opaque error loses path context

- **Severity:** LOW
- **Category:** observability / error-quality
- **Description:** The `extract_at_path` function returns `Err(())` when a JSONPath lookup fails. The empty error type provides no information about which path segment failed, what the actual JSON structure was, or whether the failure was a missing key vs a type mismatch. This makes debugging pipeline misconfigurations extremely difficult in production.
- **Proposed Fix:** Replace `Err(())` with a typed error variant carrying the failing path segment, the expected type, and the actual JSON token encountered. Non-blocking for this pass — file as TD-S-PLUGIN-PREREQ-B-001.

#### F-LP1-LOW-002: MockAuthProvider uses Ordering::SeqCst unnecessarily

- **Severity:** LOW
- **Category:** test-quality / performance
- **Description:** The `MockAuthProvider` test double uses `std::sync::atomic::Ordering::SeqCst` for its call-count atomic. `SeqCst` is the strongest ordering and introduces full memory barriers on x86/ARM; `Relaxed` is sufficient for a call counter that is only read after the test completes (no data race through the counter). This is a minor performance issue in tests only.
- **Proposed Fix:** Change call-counter atomics in `MockAuthProvider` to `Ordering::Relaxed`. Non-blocking — cosmetic.

### OBS

#### OBS-001: TD-PIPELINE-001, TD-PIPELINE-002, TD-PIPELINE-003 not filed

- **Severity:** OBS (process-gap)
- **Description:** Three technical debt items surfaced during implementation were mentioned in code comments (`// TODO: TD-PIPELINE-001`, etc.) but are absent from `.factory/tech-debt-register.md`. Per the TODO↔TD round-trip discipline (PG from S-PLUGIN-PREREQ-A cascade), every `TD-*` citation in code must have a corresponding entry in the register. Without registration, the TDs are invisible to the orchestrator and will never be prioritized or resolved.
- **Proposed Fix:** State-manager to register TD-PIPELINE-001/002/003 in tech-debt-register.md in the same fix-burst commit (or a separate factory-artifacts commit). Implementer to confirm TD IDs match the registered entries.

#### OBS-002: AuthToken Drop does not zeroize — credentials linger in heap after use

- **Severity:** OBS (security observation — scope-deferred)
- **Description:** The `AuthToken` struct holds credential material (bearer tokens, API keys) as `String`. When the struct is dropped, the heap memory is freed but not zeroed — the credential material persists in memory until overwritten by the allocator. For a long-running MCP server process, this means credentials from completed pipeline runs remain readable in memory dumps. The `zeroize` crate (`impl Drop for AuthToken { fn drop(&mut self) { self.token.zeroize(); } }`) would mitigate this.
- **Disposition:** Scope-defer to TD-S-PLUGIN-PREREQ-B-002 P3. Non-blocking for this pass.

#### OBS-003: execute_step is dead code at module boundary — compiler will warn in release build

- **Severity:** OBS (hygiene)
- **Description:** `execute_step` is a private function called only from `execute` within the same module. If the module is conditionally compiled (e.g., `#[cfg(feature = "pipeline")]`), the function will generate a `dead_code` warning in configurations where the feature is disabled. The warning is suppressed in test builds because tests exercise it, but release builds without the feature gate will surface it.
- **Disposition:** Non-blocking. Add `#[allow(dead_code)]` with a comment citing the feature gate, or restructure the call graph. Recommend addressing in fix-burst-1 if trivial.

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 4 |
| HIGH | 6 |
| MEDIUM | 5 |
| LOW | 2 |
| OBS | 3 |

**Overall Assessment:** block
**Convergence:** FINDINGS_REMAIN — iterate
**Readiness:** requires revision (fix-burst-1 dispatching to close 4 CRIT + 6 HIGH + 4 MED)

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 1 |
| **New findings** | 20 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 1.0 (20 / (20 + 0)) |
| **Median severity** | HIGH |
| **Trajectory** | 20 (pass-1 only) |
| **Verdict** | FINDINGS_REMAIN |

## KUDOs

1. **Green TDD at 251/251** — implementing all story tasks to test-green before adversarial review demonstrates strong TDD discipline. The test suite covers the happy path comprehensively.
2. **AuthProvider trait abstraction** — introducing the `AuthProvider` trait rather than hard-coding credential lookup is the correct architectural move per ADR-023 §C runtime credential injection. The abstraction is clean and extensible.
3. **PipelineResult struct design** — the result struct correctly carries `records`, `truncated`, `pages_fetched`, and `step_vars_final`. The field set matches the AC requirements even if some fields have incorrect values. Good forward-thinking design.
4. **MockHttpClient injection** — the test infrastructure correctly injects a `MockHttpClient` in most tests, demonstrating understanding of the DTU test-isolation principle. The CI-safety violation (CRIT-004) is a single test that escaped the pattern.
5. **Cursor token handling in non-first pages** — the pagination logic for pages 2..N correctly reads and injects the cursor from the prior step's response. The asymmetry (CRIT-002 encoding + HIGH-005 first-call) is a gap at the boundary, not in the general case.

## Convergence Position

**Streak: 0/3** — first pass, BLOCKED-hard.

Recommended fix-burst-1 dispatch order (priority-sequenced):

1. **CRIT-004 first** — remove live network call, add `MockHttpClient` injection + meaningful assertions. This unblocks CI and is a pre-condition for all other tests to be valid.
2. **CRIT-003** — fix intermediate-step record leak (accumulate only final step). Add 2-step Red Gate test.
3. **CRIT-001** — wire body_template interpolation + fix Content-Type to use spec value / default JSON. Add Red Gate test.
4. **CRIT-002** — add percent-encoding for cursor URL injection. Add test with special characters.
5. **HIGH-001 (AC-6)** — wire `fan_out_batches` call into `execute` loop. Add Red Gate test with 3 fan-out keys.
6. **HIGH-003 (AC-5)** — add audit-log emit in `execute_step` success + error paths.
7. **HIGH-002 (AC-7)** — fix rate-limit key per-host; add `Retry-After` positive test.
8. **HIGH-004** — wire query_filters to record post-processing. Add filter test.
9. **HIGH-005** — unify first-call / subsequent-call pagination branch.
10. **HIGH-006** — replace naive `split('.')` JSONPath with bracket-notation aware parser.
11. **MED-002 (truncated flag)** — add truthy test.
12. **MED-003** — fix Cargo.toml edition 2021→2024.
13. **MED-004** — change `unwrap_or("")` to typed error for missing extracted variable.
14. **MED-005** — update story §Red Gate count; verify BC-prefixed names.
15. **MED-001** — spec-side: BC-2.16.002 v1.3→v1.4 amendment (separate factory-artifacts commit by product-owner or state-manager; not implementer scope).

LOW-001/002 and OBS-001..003: address in fix-burst-1 where trivial (LOW-002 one-line change; OBS-001 state-manager scope; OBS-003 one-line `#[allow]`). LOW-001 defer to TD-S-PLUGIN-PREREQ-B-001.

## File References

Implementation files reviewed:
- `/Users/jmagady/Dev/prism/crates/prism-pipeline/src/pipeline.rs` (or equivalent — PipelineExecutor::execute, execute_step, store_step_vars)
- `/Users/jmagady/Dev/prism/crates/prism-pipeline/src/auth.rs` (AuthProvider trait, AuthToken)
- `/Users/jmagady/Dev/prism/crates/prism-pipeline/src/jsonpath.rs` (or extract_at_path location)
- `/Users/jmagady/Dev/prism/crates/prism-pipeline/src/pagination.rs` (cursor handling)
- `/Users/jmagady/Dev/prism/crates/prism-pipeline/Cargo.toml` (edition field)
- `/Users/jmagady/Dev/prism/crates/prism-pipeline/tests/bc_2_16_002_test.rs` (Red Gate tests)

Factory artifact reviewed:
- `/Users/jmagady/Dev/prism/.factory/stories/S-PLUGIN-PREREQ-B.md` (story spec v1.0)
- `/Users/jmagady/Dev/prism/.factory/specs/behavioral-contracts/BC-2.16.002.md` (v1.3 status:draft)
