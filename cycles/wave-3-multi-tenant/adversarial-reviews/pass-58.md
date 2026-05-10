---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-05-10T00:00:00
phase: 3
inputs:
  - "crates/prism-query/src/engine.rs"
  - "crates/prism-query/src/materialization.rs"
  - "crates/prism-query/src/internal_tables.rs"
  - "crates/prism-query/tests/execute_integration_tests.rs"
  - ".factory/stories/S-3.02-FOLLOWUP-RUNTIME-query-engine.md"
input-hash: "ada3cc5"
traces_to: prd.md
pass: 58
previous_review: "pass-57.md"
review_class: PR-LEVEL
scope: PR #141 — S-3.02-FOLLOWUP-RUNTIME — feature/S-3.02-FOLLOWUP-RUNTIME vs origin/develop
---

# Adversarial Review: PR #141 S-3.02-FOLLOWUP-RUNTIME — QueryEngine Execution Pipeline (Pass 58)

## Finding ID Convention

Finding IDs use the format: `ADV-<CYCLE>-P<PASS>-<SEV>-<SEQ>`

- `ADV`: Fixed prefix identifying adversarial findings
- `<CYCLE>`: `W3MT` (wave-3-multi-tenant cycle)
- `<PASS>`: Two-digit pass number — `P58`
- `<SEV>`: Severity abbreviation (`CRIT`, `HIGH`, `MED`, `LOW`)
- `<SEQ>`: Three-digit sequence within the pass (e.g., `001`)

Examples: `ADV-W3MT-P58-CRIT-001`, `ADV-W3MT-P58-HIGH-002`, `ADV-W3MT-P58-MED-001`

## Scope Note

This is a PR-LEVEL fresh-context review. Target: `git diff origin/develop...feature/S-3.02-FOLLOWUP-RUNTIME` across four files:
- `crates/prism-query/src/engine.rs`
- `crates/prism-query/src/materialization.rs`
- `crates/prism-query/src/internal_tables.rs`
- `crates/prism-query/tests/execute_integration_tests.rs`

Plus supporting changes in `crates/prism-core/src/error.rs` and `crates/prism-query/Cargo.toml`.
No prior pass findings were consulted. Independent fresh-context review.

## Part A — Fix Verification

N/A — Pass 1 at PR level (not a re-review pass over a prior adversarial cycle at this scope).

## Part B — New Findings

### CRITICAL

#### ADV-W3MT-P58-CRIT-001: E-QUERY-001 Code Collision — Two Distinct Errors Share the Same Error Code

- **Severity:** CRITICAL
- **Category:** contradictions
- **Location:** `crates/prism-core/src/error.rs` lines 396-399 and 577-589
- **Description:** The PR introduces `PrismError::QueryLimitExceeded` with the `#[error]` display string `"E-QUERY-001: limit {requested} exceeds maximum of {max} (BC-2.11.001)"`. However, `PrismError::QueryParseFailed` already carries `"E-QUERY-001: query parse error at offset {offset}: {detail}"`. This is a direct error-code collision: two semantically distinct errors — parse failure and limit violation — share the same `E-QUERY-001` code. Per BC-2.11.001, a limit validation error is a pre-execution parameter rejection, not a parse error. BC-2.11.001 §Error Cases assigns `E-QUERY-001` to "PrismQL query string cannot be parsed" and `E-QUERY-006` is cited for record-cap exceeded. There is no canonical code assigned for limit > 1000 in the BC text, but reusing E-QUERY-001 conflicts with its existing parse-error semantics.
- **Evidence:** `error.rs` line 397: `#[error("E-QUERY-001: query parse error at offset {offset}: {detail}")]` on `QueryParseFailed`. `error.rs` line 582: `#[error("E-QUERY-001: limit {requested} exceeds maximum of {max} (BC-2.11.001)")]` on `QueryLimitExceeded`. Both variant display strings begin with `E-QUERY-001:`. Any tooling or client that pattern-matches on `E-QUERY-001` will conflate parse failures and limit-exceeded rejections. The integration test `test_LP2_LOW_1_limit_exceeded_returns_query_limit_exceeded_variant` correctly asserts the variant type but also asserts `display.contains("E-QUERY-001")` — validating the collision rather than flagging it.
- **Proposed Fix:** Assign a new, unambiguous error code for the limit-exceeded case. The canonical candidate from BC-2.11.001 §Error Cases does not map `limit > 1000` to any specific E-QUERY-NNN. One approach: assign `E-QUERY-008` (or the next available code in the E-QUERY series) to `QueryLimitExceeded`. Update the `#[error]` attribute, the integration test `assert!(display.contains("E-QUERY-001")...)`  assertion, and any BC text that needs to reference it. An alternative is to document the collision as intentional in the BC with a disambiguation note, but that is strongly discouraged given downstream tooling risk.

---

#### ADV-W3MT-P58-CRIT-002: `PRISM_MAX_INTERNAL_TABLE_SCAN` Defaults to 10K — Contradicts BC-2.15.011 50K Soft Limit

- **Severity:** CRITICAL
- **Category:** spec-fidelity
- **Location:** `crates/prism-query/src/internal_tables.rs` line 64
- **Description:** `PRISM_MAX_INTERNAL_TABLE_SCAN` is set to `10_000` (10K rows). BC-2.15.011 §Postconditions explicitly states: "Internal table scans are bounded by a configurable limit (default 50K rows, configurable via `PRISM_MAX_INTERNAL_TABLE_SCAN`)." The module docstring in `internal_tables.rs` at line 31 also cites "Scans are truncated at `PRISM_MAX_INTERNAL_TABLE_SCAN` entries." — but with the value set to 10K, the BC's 50K default is not met. BC-2.15.011 also distinguishes this from the external 10K sensor cap: "Internal table scans use a separate 50K-row soft limit (not the external 10K hard limit) and return partial results with `_meta.scan_truncated: true` when the limit is hit."
- **Evidence:** `internal_tables.rs` line 64: `pub const PRISM_MAX_INTERNAL_TABLE_SCAN: usize = 10_000;`. BC-2.15.011 §Postconditions: "Internal table scans are bounded by a configurable limit (default 50K rows, configurable via `PRISM_MAX_INTERNAL_TABLE_SCAN`)". BC-2.15.011 §Error Cases: "Scan truncation: 60K alerts in DB → First 50K returned; `_meta.scan_truncated: true`." These are directly contradicted.
- **Proposed Fix:** Change `PRISM_MAX_INTERNAL_TABLE_SCAN` to `50_000`. Additionally, the `_meta.scan_truncated` field is not injected anywhere in the current `scan()` implementation — the scan silently truncates. The BC requires partial results to be returned with `_meta.scan_truncated: true`. This flag injection is also missing and should be added (see separate MEDIUM finding).

---

### HIGH

#### ADV-W3MT-P58-HIGH-001: `GreedyMemoryPool` Referenced in BC Comment and Config — Not Wired Into SessionContext

- **Severity:** HIGH
- **Category:** spec-fidelity
- **Location:** `crates/prism-query/src/engine.rs` (execute_inner, execute_scheduled)
- **Description:** BC-2.11.006 postcondition 6 states "200MB GreedyMemoryPool: enforced via DataFusion `RuntimeEnv` configured with a `GreedyMemoryPool(200MB)` on the `SessionContext`." The `QueryEngineConfig` struct carries `memory_pool_bytes: 200 * 1024 * 1024` and the module docstring references "200MB GreedyMemoryPool." However, neither `execute_inner` nor `execute_scheduled` construct a `RuntimeEnv` with a `GreedyMemoryPool` — both use `SessionContext::new()` (default, unbounded memory). The `memory_pool_bytes` config field is stored but never consumed anywhere in the new code. A query that materializes 200MB of Arrow data will not be bounded.
- **Evidence:** `execute_inner`: `let session_ctx = datafusion::execution::context::SessionContext::new();` — no `RuntimeEnv` or memory pool configuration. `execute_scheduled`: same. `QueryEngineConfig` at engine.rs line 78: `pub memory_pool_bytes: usize,` with default `200 * 1024 * 1024` — not referenced in either execution path. BC-2.11.006 §Postconditions: "Memory pool: 200MB GreedyMemoryPool enforced via DataFusion RuntimeEnv."
- **Proposed Fix:** In both `execute_inner` and `execute_scheduled`, construct a `RuntimeEnv` with `GreedyMemoryPool::new(self.config.memory_pool_bytes)` and pass it to `SessionContext::new_with_config_rt(SessionConfig::default(), Arc::new(runtime_env))`. This wires the 200MB cap into DataFusion's execution engine so large result sets return `ResourcesExhausted` rather than OOM-killing the process.

---

#### ADV-W3MT-P58-HIGH-002: Story AC Tests Miss Two Required Integration Scenarios (Timeout and Depth Limit)

- **Severity:** HIGH
- **Category:** coverage-gap
- **Location:** `crates/prism-query/tests/execute_integration_tests.rs`
- **Description:** The story spec §Tasks item 8 lists seven required integration tests. Two of them are absent from the PR: (a) "Test: query hitting timeout → `E-QUERY-004` (timeout)" and (b) "Test: query that exceeds depth limit → `E-QUERY-005` (parse error)." The test file has no function whose name or body exercises a timeout scenario or a depth-limit parse rejection. The story itself (S-3.02-FOLLOWUP-RUNTIME §Tasks) makes these mandatory. AC-8 ("No `todo!()` may remain") is tested but the two mandatory behavioral tests are not.
- **Evidence:** Story §Tasks item 8 bullet 5: "Test: query hitting timeout → `E-QUERY-004` (timeout)." Bullet 6: "Test: query that exceeds depth limit → `E-QUERY-005` (parse error)." Searching `execute_integration_tests.rs` for `E-QUERY-004`, `E-QUERY-005`, `timeout`, `depth.*limit`, `nesting` returns zero matches. These scenarios are never exercised.
- **Proposed Fix:** Add two integration tests: (1) `test_timeout_returns_e_query_004` — shorten `QueryEngineConfig.timeout_secs` to 0 (or use a spy adapter with a long sleep), call `execute`, verify `Err(PrismError::QueryTimeout { .. })`. (2) `test_depth_limit_parse_error_returns_e_query_005` — construct a deeply nested PrismQL query (64+ levels of parenthesized predicates), call `execute`, verify the result is `Err` containing a parse error. Both are straightforward unit-style scenarios that do not require live adapters.

---

#### ADV-W3MT-P58-HIGH-003: BC-2.15.011 Requires `prism_rules` and `prism_aliases` Tables — Both Absent

- **Severity:** HIGH
- **Category:** spec-fidelity
- **Location:** `crates/prism-query/src/internal_tables.rs` — `INTERNAL_TABLE_SPECS` constant (lines 649-654)
- **Description:** BC-2.15.011 §Postconditions requires seven RocksDB domains to be registered as DataFusion tables: `prism_alerts`, `prism_cases`, `prism_rules` (StorageDomain::DetectionRules), `prism_schedules`, `prism_diff_results`, `prism_audit`, and `prism_aliases` (StorageDomain::Aliases). The `INTERNAL_TABLE_SPECS` constant in the PR registers only five: `prism_audit`, `prism_alerts`, `prism_cases`, `prism_schedules`, `prism_diff_results`. `prism_rules` (mapping to `StorageDomain::DetectionRules`) and `prism_aliases` (mapping to `StorageDomain::Aliases`) are missing entirely. An analyst query `SELECT * FROM prism_rules` or `SELECT * FROM prism_aliases` would return `E-QUERY-006` (table not found) rather than the expected BC-2.15.011 behavior.
- **Evidence:** BC-2.15.011 §Postconditions lists all seven tables including "prism_rules — Detection rule definitions (StorageDomain::DetectionRules)" and "prism_aliases — Alias definitions and metadata (StorageDomain::Aliases)". `INTERNAL_TABLE_SPECS` (lines 649-654): five entries only — `prism_audit`, `prism_alerts`, `prism_cases`, `prism_schedules`, `prism_diff_results`. `parse_domain()` (line 568) does list `"detection_rules"` as parseable, confirming the domain exists, but the table is not registered. `StorageDomain::Aliases` is not in `parse_domain()` at all.
- **Proposed Fix:** Add `prism_rules` and `prism_aliases` to `INTERNAL_TABLE_SPECS`. Define `rules_schema()` (e.g. `{ rule_id: Utf8, rule_name: Utf8, query: Utf8, severity: Int32 }`) and `aliases_schema()` (e.g. `{ alias_name: Utf8, target_query: Utf8, created_at: Utf8 }`). Add `"aliases"` to `parse_domain()`. Add both schemas to the zip in `register_internal_tables_with_capabilities`. Add an integration test covering `SELECT * FROM prism_rules` and `SELECT * FROM prism_aliases` after registration.

---

#### ADV-W3MT-P58-HIGH-004: `execute_scheduled` Has No Pre-Execution Capability Gate for `prism_audit`

- **Severity:** HIGH
- **Category:** security-surface
- **Location:** `crates/prism-query/src/engine.rs` — `execute_scheduled` body (lines 520-589)
- **Description:** `execute_inner` (called by `execute`) correctly calls `check_internal_table_capabilities(query_str, &options.capabilities)` (Layer 1 gate) before registering internal tables. `execute_scheduled`, however, does NOT call `check_internal_table_capabilities`. It registers internal tables with no capabilities (`&[]`) and then calls `run_materialization_pipeline` with `..QueryOptions::default()` — which also has an empty `capabilities` vec. If a detection-engine rule query references `prism_audit`, it bypasses the Layer 1 pre-execution gate entirely. Layer 2 (scan-time) would still block it because the provider was registered with `&[]` capabilities — but the defense-in-depth model requires Layer 1 to also fire for all code paths.
- **Evidence:** `execute_scheduled` at line ~520: no call to `check_internal_table_capabilities`. `execute_inner` at line 413: `check_internal_table_capabilities(query_str, &options.capabilities)?`. `execute_scheduled` uses `..QueryOptions::default()` which sets `capabilities: Vec::new()` by default. The comment at line 537 states "Scheduled queries run with no caller capabilities (system context)" — but this is an assertion about policy, not about whether the Layer 1 gate should fire. If Layer 1 is the authoritative pre-check, it should run for all execution paths that eventually call `register_internal_tables_with_capabilities`.
- **Proposed Fix:** Add `check_internal_table_capabilities(query_str, &[])` call at the top of `execute_scheduled` body (after resolving clients, before creating SessionContext). Since scheduled queries run with no capabilities, this is a no-op for all tables except `prism_audit` — and for `prism_audit` it will return `Err(AuditTableAccessDenied)`, which is the correct behavior: scheduled queries should not query the audit log unless explicitly granted the capability. Alternatively, document in the BC that `execute_scheduled` intentionally bypasses Layer 1 (system context grants no audit access) and rely on Layer 2.

---

#### ADV-W3MT-P58-HIGH-005: `sensors_queried` in `QueryResultContext` Is Always `Vec::new()` — BC-2.11.001 Postcondition Violated

- **Severity:** HIGH
- **Category:** spec-fidelity
- **Location:** `crates/prism-query/src/engine.rs` lines 473, 572
- **Description:** BC-2.11.001 §Postconditions states the response must include `query_context.sensors_queried`. Both `execute_inner` and `execute_scheduled` build `QueryResultContext` with `sensors_queried: Vec::new()` — always empty. The materialization pipeline collects per-sensor fan-out results but never populates `sensors_queried` in the returned output. The field exists in `QueryResultContext` (line 150) but is never populated.
- **Evidence:** `execute_inner` at line 473: `sensors_queried: Vec::new()`. `execute_scheduled` at line 572: `sensors_queried: Vec::new()`. BC-2.11.001 §Postconditions: "Response includes `query_context` with: `original_query`, `expanded_query`, `clients_queried`, `sensors_queried`, `time_range_applied`, `total_available`, `returned_results`, `is_truncated`, `execution_time_ms`." An always-empty `sensors_queried` means downstream tooling (MCP response, observability, partial-failure correlation) cannot determine which sensors were actually queried.
- **Proposed Fix:** Populate `sensors_queried` from the `FanOutTarget` sensor types resolved during `run_materialization_pipeline`. Add `sensors_queried: Vec<String>` to `MaterializationOutput` and collect `target.sensor_type.as_str()` for each target processed in the fan-out loop. In `execute_inner`, assign `sensors_queried: output.sensors_queried` in the `QueryResultContext` constructor. Use a deduplicated list (multiple targets of the same sensor type should appear once).

---

### MEDIUM

#### ADV-W3MT-P58-MED-001: `_meta.scan_truncated` Not Injected on Truncated Internal Table Scans

- **Severity:** MEDIUM
- **Category:** spec-fidelity
- **Location:** `crates/prism-query/src/internal_tables.rs` — `RocksDbTableProvider::scan()`
- **Description:** BC-2.15.011 §Postconditions: "When the limit is hit, partial results are returned with `_meta.scan_truncated: true`." The canonical test vector in the BC: "Scan truncation: 60K alerts in DB → First 50K returned; `_meta.scan_truncated: true`." The `scan()` implementation truncates via `pairs_truncated.into_iter().take(cap)` but never sets any `_meta.scan_truncated` field in the result. There is no `_meta` column added to the schema or injected via `inject_internal_virtual_fields`. The truncation happens silently.
- **Evidence:** `scan()` at internal_tables.rs line 261: `let pairs_truncated: Vec<_> = kv_pairs.into_iter().take(cap).collect();` — truncates but does not signal. `inject_internal_virtual_fields()` adds `_sensor`, `_client`, `_source_table` but not `_meta.scan_truncated`. BC-2.15.011 scan truncation canonical vector.
- **Proposed Fix:** Either (a) add a `_meta_scan_truncated` boolean column (Arrow `BooleanArray`) set to `true` when `pairs.len() == cap` (indicating the cap was hit), or (b) return the truncation status via a side channel in the execution plan (e.g., custom `ExecutionPlan` metadata). Option (a) is simpler and matches the BC's "virtual field" pattern. The field name `_meta_scan_truncated` (using underscore separator since dots are not valid in PrismQL identifiers as noted in BC-2.15.011 §Postconditions).

---

#### ADV-W3MT-P58-MED-002: `#![allow(dead_code)]` Remains on Implementation Module Without Expiry Condition

- **Severity:** MEDIUM
- **Category:** code-quality
- **Location:** `crates/prism-query/src/engine.rs` line 29
- **Description:** The module-level `#![allow(dead_code)]` was appropriate during the stub phase when most functions had empty bodies. Now that all stub sites are filled, dead code warnings are legitimate signals: if a function or field is genuinely unreachable, it should be removed, not suppressed. The comment "Dead code suppression retained during the transition phase" does not define what constitutes the end of the transition phase. Per the CLAUDE.md project conventions, suppressing dead-code warnings globally on an implementation module masks real cleanup debt.
- **Evidence:** Line 28: `// Implementation module: all stub sites are now filled per S-3.02-FOLLOWUP-RUNTIME.` Line 29: `#![allow(dead_code)]`. The story AC-8 prohibits `todo!()` / `unimplemented!()` but does not prohibit or address this suppression. The same pattern appears in `internal_tables.rs` line 38: `#![allow(dead_code)]` with no expiry.
- **Proposed Fix:** Remove `#![allow(dead_code)]` from both `engine.rs` and `internal_tables.rs`. Address any resulting dead-code warnings individually: either remove unused functions/fields, or add a targeted `#[allow(dead_code)]` attribute with a comment explaining why the item is kept (e.g., "retained for S-4.01 consumer" or "exposed for integration testing via `pub(crate)`"). This transitions from a blanket suppression to an explicit, auditable allowlist.

---

#### ADV-W3MT-P58-MED-003: AC-6 Test — `_client` Values Asserted From Synthetic Slug Fallback, Not OrgRegistry

- **Severity:** MEDIUM
- **Category:** coverage-gap
- **Location:** `crates/prism-query/tests/execute_integration_tests.rs` — `test_AC_6_cross_client_query_all_scope_fans_out`
- **Description:** AC-6 (BC-2.11.011) requires cross-client queries with ALL scope to produce `_client` values for each registered org. The test registers two `StubAdapter` instances with distinct `OrgId::new()` values and asserts `_client` contains `"acme"` and `"beta"`. However, the engine is constructed with `make_engine(registry, vec![org_acme.clone(), org_beta.clone()])` — which uses `QueryEngine::new()` not `QueryEngine::new_full()`. Without an `OrgRegistry`, `resolve_source_refs` falls through to the synthetic-slug fallback (`org-<first-8-hex-of-uuid>`) rather than the correct org slug. The test clients list is `[acme, beta]` but these are used only as `ClientRegistry` entries; the fan-out with `clients: None` (ALL scope) uses `get_all_for_sensor_type` which returns `OrgId`-keyed entries. Without `OrgRegistry.slug_for(org_id)`, the `_client` value cannot be `"acme"` or `"beta"` — it will be `"org-<hex>"`. The test may fail at assertion or assert vacuously if the `_client` column contains the synthetic slug.
- **Evidence:** `test_AC_6_cross_client_query_all_scope_fans_out` lines 608-671: uses `helpers::make_engine(registry, vec![org_acme.clone(), org_beta.clone()])`. `make_engine` calls `QueryEngine::new(...)` with no `OrgRegistry`. In `resolve_source_refs` ALL-scope branch: `let Some(client_slug) = org_registry.as_ref().and_then(|reg| reg.slug_for(&org_id)) else { ... synthetic_slug ... }` — when `org_registry` is `None`, `slug_for` is unreachable and the else branch runs, producing `org-<uuid-prefix>`. The assertion `client_values.contains("acme")` will fail or the test was passing with the wrong values.
- **Proposed Fix:** Use `QueryEngine::new_full()` in the AC-6 test with an `OrgRegistry` that maps the registered `OrgId` values to `"acme"` and `"beta"` slugs. This requires tracking the `OrgId::new()` values returned during `registry.register(...)` and pre-populating `OrgRegistry::insert(org_id, org_slug)` before constructing the engine. Alternatively, test with explicit `clients: Some(vec![org_acme, org_beta])` (known client scope) rather than `clients: None` (ALL scope), which takes the explicit-client-list code path that uses client slugs directly.

---

#### ADV-W3MT-P58-MED-004: AC-3 Test Uses `make_mat_ctx_with_stub` But Does Not Match Story Spec E-QUERY-003 Behavior

- **Severity:** MEDIUM
- **Category:** spec-fidelity
- **Location:** `crates/prism-query/tests/execute_integration_tests.rs` — `test_AC_3_size_limit_returns_e_query_003`
- **Description:** The story spec §Tasks item 8 bullet 4 states: "Test: query hitting BC-2.11.006 size limit → `E-QUERY-003` (materialization records limit)." The test `test_AC_3_size_limit_returns_e_query_003` sets `max_records=1`, stub returns 5 rows, and asserts the error contains `"E-QUERY-003"`. This is correct in structure but the error message check relies on the string content of `QueryExecutionFailed.detail` (which embeds `"E-QUERY-003:"` as a prefix). If the detail format changes, the test still passes structurally. More importantly, the story spec says the error is `PrismError::QueryExecutionFailed { detail }` where detail begins with `E-QUERY-003`. This matches the implementation in `increment_record_count`. However, AC-3 in the story spec says "Then `PrismError::QueryError(E-QUERY-003)` is returned" — there is no `PrismError::QueryError` variant; the actual variant is `PrismError::QueryExecutionFailed`. This is a spec/implementation mismatch (story AC-3 references a non-existent variant name `QueryError`).
- **Evidence:** Story §AC-3: "Then `PrismError::QueryError(E-QUERY-003)` is returned." `PrismError` in `error.rs` has no `QueryError` variant — the record cap violation returns `QueryExecutionFailed { detail: "E-QUERY-003: record cap exceeded..." }`. This makes AC-3 technically untraceable to a conforming behavior: the AC references a variant that does not exist, and the test uses `err.to_string().contains("E-QUERY-003")` as a proxy. This is a story spec authoring error.
- **Proposed Fix:** Update story AC-3 to reference `PrismError::QueryExecutionFailed { detail }` where `detail.starts_with("E-QUERY-003")`. Update the integration test to use `matches!(err, PrismError::QueryExecutionFailed { .. })` as the variant assertion before the string check. This makes the test's structural assertion explicit.

---

### LOW

#### ADV-W3MT-P58-LOW-001: `bincode = { version = "2", features = ["serde"] }` Added to Production Dependencies — Should Be Dev-Only

- **Severity:** LOW
- **Category:** code-quality
- **Location:** `crates/prism-query/Cargo.toml` lines 56-59 (production `[dependencies]` section)
- **Description:** `bincode` is added to production `[dependencies]` for use in `RocksDbTableProvider::scan()` to deserialize `AuditEntry` values from RocksDB. Since `prism-storage` already has `bincode` as a production dependency (per the comment citing AD-012), `prism-query` adding it again creates a second crate in the dependency tree that must stay version-synchronized. More concerning: the story spec §Library & Framework Requirements lists `bincode` as a production dependency — but the audit deserialization in `build_batch_from_kv` is the only use site. If the `scan()` path is considered a production code path (which it is), `bincode` must be in `[dependencies]` — this is consistent. The LOW severity is that the comment says "matching prism-storage AD-012" but there is no verification that the `bincode::config::standard()` call matches the config used by `prism-storage` to serialize. A version or config mismatch would cause silent empty-string fallback (graceful degradation) rather than surfacing the error.
- **Evidence:** `build_batch_from_kv`: `bincode::serde::decode_from_slice::<AuditEntry, _>(value_bytes, bincode::config::standard())`. If `prism-storage` serializes with a different bincode config (e.g., `bincode::config::legacy()`), every deserialization falls back to empty strings silently. The graceful degradation path does emit a `tracing::debug!` but no test exercises the cross-crate config alignment.
- **Proposed Fix:** Add a compile-time assertion or `#[cfg(test)]` round-trip test that serializes an `AuditEntry` using `prism-storage`'s serialization path and deserializes it using `prism-query`'s deserialization path, asserting the round-trip succeeds. This catches config mismatches before they silently corrupt audit queries in production. Alternatively, expose a `bincode_config()` function from `prism-storage` that returns the canonical config, and import it in `prism-query`.

---

#### ADV-W3MT-P58-LOW-002: `resolve_source_refs` Silently Skips Unknown Table Names Instead of Returning E-QUERY-006

- **Severity:** LOW
- **Category:** spec-fidelity
- **Location:** `crates/prism-query/src/materialization.rs` — `resolve_source_refs()` lines ~610-620
- **Description:** The story spec §Edge Cases EC-001 states: "resolve_source_refs encounters unknown table name → `E-QUERY-006` (unknown source); no fan-out attempted." The implementation silently skips unknown source names via `tracing::debug!(source_name, "resolve_source_refs: unknown sensor prefix; skipping")` and `continue`. This matches the silent-skip behavior of `prism_*` internal tables (which is correct), but extends the same skip to unrecognized external table names. An analyst who mistypes a sensor table name (e.g., `crowdstrike_detectionz`) receives an empty result set rather than a diagnostic `E-QUERY-006` error.
- **Evidence:** `resolve_source_refs()` in materialization.rs: `let Some(sensor_type) = sensor_type_from_table_name(source_name) else { tracing::debug!(...); continue; };` — the `None` arm (unknown prefix) silently continues. Story §EC-001: "EC-001: `resolve_source_refs` encounters unknown table name → `E-QUERY-006` (unknown source); no fan-out attempted." The word "attempted" suggests the function should return an error, not skip silently.
- **Proposed Fix:** For non-`prism_*` source names that do not map to a known sensor type, return `Err(PrismError::QueryExecutionFailed { detail: "E-QUERY-006: unknown source '{source_name}'" })` rather than `continue`. Reserve the silent-skip behavior only for `prism_*` prefixed names (internal tables handled separately). This implements EC-001 as specified. If silent-skip is intentional for MVP (to allow partial materialization when one of several sources is unknown), update the story EC-001 text to reflect the actual behavior.

---

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 2 |
| HIGH | 5 |
| MEDIUM | 4 |
| LOW | 2 |

**Overall Assessment:** block

**Convergence:** FINDINGS_REMAIN — iterate

**Readiness:** Requires revision before merge. CRITICAL findings (E-QUERY-001 code collision; PRISM_MAX_INTERNAL_TABLE_SCAN at 10K vs BC-mandated 50K) and HIGH findings (GreedyMemoryPool not wired; missing timeout/depth-limit tests; prism_rules/prism_aliases absent; execute_scheduled lacks Layer 1 capability gate; sensors_queried always empty) must be resolved prior to merge. MEDIUM and LOW findings should be addressed within the same PR or tracked as explicit TD stories.

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 58 (PR-level, fresh-context) |
| **New findings** | 13 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 13/13 = 1.0 |
| **Median severity** | HIGH |
| **Trajectory** | PR-P01: 13 findings (2 CRIT, 5 HIGH, 4 MED, 2 LOW) |
| **Verdict** | FINDINGS_REMAIN |
