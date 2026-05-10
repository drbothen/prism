---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-05-10T12:00:00
phase: 3
inputs:
  - "crates/prism-query/src/engine.rs"
  - "crates/prism-query/src/materialization.rs"
  - "crates/prism-query/src/internal_tables.rs"
  - "crates/prism-query/tests/execute_integration_tests.rs"
  - "crates/prism-storage/src/internal_tables.rs"
  - ".factory/specs/behavioral-contracts/BC-2.11.006-query-security-limits.md"
  - ".factory/specs/behavioral-contracts/BC-2.15.011-internal-table-registration.md"
input-hash: "[pending-recompute]"
traces_to: prd.md
pass: 59
previous_review: "pass-58.md"
review_class: PR-LEVEL
scope: PR #141 — S-3.02-FOLLOWUP-RUNTIME — feature/S-3.02-FOLLOWUP-RUNTIME vs origin/develop (Pass 2, post fix-burst 4874025b)
---

# Adversarial Review: PR #141 S-3.02-FOLLOWUP-RUNTIME — QueryEngine Execution Pipeline (Pass 59)

## Finding ID Convention

Finding IDs use the format: `ADV-W3MT-P59-<SEV>-<SEQ>`

- `ADV`: Fixed prefix identifying adversarial findings
- `W3MT`: wave-3-multi-tenant cycle
- `P59`: Pass 59
- `<SEV>`: Severity abbreviation (`CRIT`, `HIGH`, `MED`, `LOW`)
- `<SEQ>`: Three-digit sequence within the pass (e.g., `001`)

## Scope Note

This is PR-LEVEL Pass 2 — a fresh-context review of the full diff after fix-burst commit `4874025b` closed 13 findings from pass-1 (pass-58). The following findings from pass-58 are CLOSED per the fix-burst commit message and MUST NOT be re-raised: CRIT-001 (E-QUERY-007 assigned), CRIT-002 (50K constant), HIGH-001 (GreedyMemoryPool wired), HIGH-002 (timeout+depth tests added), HIGH-003 (7 tables registered), HIGH-004 (execute_scheduled capability gate added), HIGH-005 (sensors_queried populated), MED-001 (scan truncation warn), MED-002 (dead_code narrowed), MED-003 (AC-6 OrgRegistry), MED-004 (N/A), LOW-001 (bincode comment), LOW-002 (E-QUERY-006 for unknown tables).

This review independently read the current diff (`git diff origin/develop...feature/S-3.02-FOLLOWUP-RUNTIME`) and the authoritative source files on the feature branch. No findings from pass-58 were consulted before identifying issues.

## Part A — Fix Verification

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| ADV-W3MT-P58-CRIT-001 | CRITICAL | RESOLVED | `QueryLimitExceeded` now uses `E-QUERY-007` display string. `error.rs` confirmed: `QueryParseFailed` = `E-QUERY-001`, `QueryLimitExceeded` = `E-QUERY-007`. No collision. |
| ADV-W3MT-P58-CRIT-002 | CRITICAL | RESOLVED | `PRISM_MAX_INTERNAL_TABLE_SCAN = 50_000` in `internal_tables.rs`. |
| ADV-W3MT-P58-HIGH-001 | HIGH | RESOLVED | `build_session_context` in `memory.rs` correctly wires `GreedyMemoryPool` via `RuntimeEnvBuilder`. Both `execute_inner` and `execute_scheduled` call `crate::memory::build_session_context(self.config.memory_pool_bytes)`. |
| ADV-W3MT-P58-HIGH-002 | HIGH | RESOLVED | `test_AC_timeout_returns_query_timeout_error` and `test_AC_depth_limit_returns_parse_error` added to integration tests. Both exercise the required behavioral contracts. |
| ADV-W3MT-P58-HIGH-003 | HIGH | RESOLVED | `prism_rules` and `prism_aliases` added to `INTERNAL_TABLE_SPECS`. `parse_domain` now maps `"detection_rules"` and `"aliases"`. 7 tables registered. |
| ADV-W3MT-P58-HIGH-004 | HIGH | RESOLVED | `check_internal_table_capabilities(query_str, &[])` added at top of `execute_scheduled` body. |
| ADV-W3MT-P58-HIGH-005 | HIGH | RESOLVED | `MaterializationOutput.sensors_queried` populated in fan-out loop; `QueryResultContext.sensors_queried` assigned from `output.sensors_queried` in both `execute_inner` and `execute_scheduled`. |
| ADV-W3MT-P58-MED-001 | MEDIUM | PARTIALLY_RESOLVED | `tracing::warn!` added when scan is truncated. However, `_meta.scan_truncated: true` column is NOT injected into the result RecordBatch. BC-2.15.011 §Postconditions line 66 and canonical test vector (line 98) require `_meta.scan_truncated: true` in the returned data, not just a server-side log entry. See NEW finding ADV-W3MT-P59-HIGH-001. |
| ADV-W3MT-P58-MED-002 | MEDIUM | RESOLVED | Module-level `#![allow(dead_code)]` removed from `engine.rs`, `internal_tables.rs`, `materialization.rs`. Targeted `#[allow(dead_code)]` retained on specific fields with justification comments. |
| ADV-W3MT-P58-MED-003 | MEDIUM | RESOLVED | `test_AC_6_cross_client_query_all_scope_fans_out` now uses `QueryEngine::new_full` with a real `OrgRegistry` mapping `id_acme → "acme"` and `id_beta → "beta"`. Assertions are non-vacuous. |
| ADV-W3MT-P58-MED-004 | MEDIUM | RESOLVED (N/A) | Declared N/A by implementer: AC-3 test correctly checks `detail.contains("E-QUERY-003")` against `QueryExecutionFailed.to_string()`. Story spec body has a stale `PrismError::QueryError` name but this is a documentation-quality issue only (see NEW LOW finding). |
| ADV-W3MT-P58-LOW-001 | LOW | RESOLVED | Bincode config comment added: confirms `bincode::config::standard()` matches `prism-storage` encoding. `test_HIGH_2_audit_entry_bincode_deserialization` validates round-trip via `append_audit_entry` + `execute`. |
| ADV-W3MT-P58-LOW-002 | LOW | RESOLVED | `resolve_source_refs` now returns `Err(PrismError::QueryExecutionFailed { detail: "E-QUERY-006: unknown source table '...'..." })` for unrecognized non-`prism_*` table names. |

## Part B — New Findings

### CRITICAL

#### ADV-W3MT-P59-CRIT-001: All Seven Internal Table Schemas in prism-query Diverge from prism-storage's Authoritative Definitions

- **Severity:** CRITICAL
- **Category:** contradictions
- **Location:** `crates/prism-query/src/internal_tables.rs` — `alerts_schema()`, `cases_schema()`, `schedules_schema()`, `diff_results_schema()`, `audit_schema()`, `rules_schema()`, `aliases_schema()` (lines 608-695)
- **Description:** `prism-storage/src/internal_tables.rs` is the authoritative single source of truth for internal table schemas. It exports `all_descriptors()` which returns `InternalTableDescriptor` instances with the canonical column lists per table. The PR's `prism-query/src/internal_tables.rs` reimplemented all seven schemas independently with incorrect fields. Every schema is wrong:

  | Table | prism-query schema | prism-storage authoritative schema |
  |-------|-------------------|-------------------------------------|
  | `prism_alerts` | `alert_id, rule_id, severity(Int32), timestamp` | `alert_id, severity_id(UInt64), device_ip, device_hostname, client_id, created_at, rule_id` |
  | `prism_cases` | `case_id, status, severity(Int32), created_at` | `case_id, title, severity_id(UInt64), client_id, created_at, status` |
  | `prism_schedules` | `schedule_id, query, cron, next_run` | `schedule_id, name, client_id, query, interval_secs(UInt64), last_run_at` |
  | `prism_diff_results` | `diff_id, rule_id, timestamp, payload` | `query_hash, client_id, previous_results_hash, epoch(UInt64), counter(UInt64), last_diff_time` |
  | `prism_audit` | `timestamp, event_type, org_id, payload` | `trace_id, timestamp_ns(UInt64), operation, client_id, analyst_id, outcome, capability` |
  | `prism_rules` | `rule_id, name, query, severity(Int32)` | `rule_id, name, client_id, enabled(Bool), created_at` |
  | `prism_aliases` | `alias_id, name, expansion` | `alias_id, alias, expansion, client_id, created_at` |

  The consequences are severe:
  1. Any query referencing a field that exists only in the authoritative schema (e.g., `WHERE client_id='acme'`) will fail with "column not found" at DataFusion planning time. BC-2.15.011's canonical test vector `SELECT * FROM prism_alerts WHERE client_id='acme'` would fail immediately on the current implementation.
  2. The `prism_diff_results` schema is a complete fabrication — not a single column matches the authoritative definition.
  3. `prism_audit` uses `timestamp`(Utf8) instead of `timestamp_ns`(UInt64) — the `test_HIGH_2_audit_entry_bincode_deserialization` test currently passes because `AuditEntry.timestamp_ns.to_string()` is stored as a string, but the authoritative schema exposes `timestamp_ns` as UInt64. Downstream queries doing numeric comparisons (`WHERE timestamp_ns > 1_000_000_000`) would fail type-check.
  4. `prism_aliases` has `rocksdb_backed: false` in the prism-storage descriptor (backed by `AliasStore`/TOML, NOT RocksDB). The prism-query implementation scans the `"aliases"` RocksDB CF for `prism_aliases` — this is architecturally wrong. Queries against `prism_aliases` would return empty results (the CF is reserved/unused).

- **Evidence:** `prism-storage/src/internal_tables.rs` lines 38-130 define the canonical schemas via `alerts_columns()`, `cases_columns()`, `schedules_columns()`, `diff_results_columns()`, `audit_columns()`, `rules_columns()`, `aliases_columns()`. `prism-storage/src/internal_tables.rs` line 211: `rocksdb_backed: false` for `prism_aliases`. `prism-query/src/internal_tables.rs` lines 608-695: seven divergent schema functions ignoring the prism-storage SoT. `prism-storage/src/lib.rs` line 44: `pub mod internal_tables` (module is accessible to prism-query). BC-2.15.011 canonical test vector line 94: `SELECT * FROM prism_alerts WHERE client_id='acme'` — the `client_id` column does not exist in prism-query's `alerts_schema()`.
- **Proposed Fix:** Replace all seven `*_schema()` functions in `prism-query/src/internal_tables.rs` with a function that reads `prism_storage::internal_tables::all_descriptors()` and converts each descriptor's `columns` (`Vec<(String, InternalColumnType)>`) to an Arrow `SchemaRef`. `InternalColumnType` must be mapped to `DataType`: `Text → DataType::Utf8`, `UInt64 → DataType::UInt64`, `Bool → DataType::Boolean`, `Timestamp → DataType::Utf8` (or `DataType::Int64`/`Timestamp` as appropriate). For `prism_aliases` (rocksdb_backed=false), the `RocksDbTableProvider::scan()` must detect this and return an empty batch or route to the `AliasStore` provider instead of attempting a RocksDB scan. This eliminates schema drift permanently — any future schema change to the SoT is automatically reflected.

---

### HIGH

#### ADV-W3MT-P59-HIGH-001: MED-001 Partially Addressed — `_meta.scan_truncated` Column Not Injected in Scan Result

- **Severity:** HIGH
- **Category:** spec-fidelity
- **Location:** `crates/prism-query/src/internal_tables.rs` — `RocksDbTableProvider::scan()` (lines 283-298)
- **Description:** The fix-burst commit addressed MED-001 by adding a `tracing::warn!` when `scan_truncated` is true. However, BC-2.15.011 §Postconditions line 66 explicitly states: "partial results are returned with `_meta.scan_truncated: true`." The canonical test vector (line 98): "60K alerts in DB → First 50K returned; `_meta.scan_truncated: true`." This is an output column requirement, not a log requirement. The `inject_internal_virtual_fields` function adds `_sensor`, `_client`, `_source_table` but NOT `_meta_scan_truncated`. No column named `_meta.scan_truncated` or `_meta_scan_truncated` exists in the output batches.
- **Evidence:** `scan()` at line 283: `let scan_truncated = pairs_truncated.len() < total_pairs;` followed by `if scan_truncated { tracing::warn!(...) }` — no column injection. `inject_internal_virtual_fields()` (lines 343-420): adds only three columns (`_sensor`, `_client`, `_source_table`). No `_meta_scan_truncated` anywhere in the new code. BC-2.15.011 line 66: "partial results are returned with `_meta.scan_truncated: true`." BC-2.15.011 canonical test vector line 98: `_meta.scan_truncated: true` in expected output.
- **Proposed Fix:** Add a `_meta_scan_truncated` boolean column (Arrow `BooleanArray`) to the output schema and batches when `scan_truncated` is true. Modify `inject_internal_virtual_fields` to accept a `scan_truncated: bool` parameter and append the column. Update `InternalTableDescriptor::full_schema()` to include the `_meta_scan_truncated` field so DataFusion query planning includes it. Set the value to `true` when truncation occurred, `false` otherwise. The field name uses underscore (not dot) since dot-notation is not valid in Arrow field names or PrismQL identifiers.

---

#### ADV-W3MT-P59-HIGH-002: `execute_scheduled` Has No 30-Second Timeout Wrapper — BC-2.11.006 Violated

- **Severity:** HIGH
- **Category:** security-surface
- **Location:** `crates/prism-query/src/engine.rs` — `execute_scheduled` body (lines 535-607)
- **Description:** `execute()` wraps `execute_inner` with `tokio::time::timeout(Duration::from_secs(timeout_secs), ...)` per BC-2.11.006 postcondition 5: "Query timeout (30s): Enforced via `tokio::time::timeout` wrapping the **entire query execution lifecycle** (alias resolution through result serialization)." `execute_scheduled` performs an identical pipeline (parse → capability gate → session context → materialization → DataFusion execution) but has NO `tokio::time::timeout` wrapper. A detection-engine rule referencing a slow sensor adapter can hang `execute_scheduled` indefinitely. Detection rules are ultimately user-influenced (stored in RocksDB from MCP tool calls), making this a denial-of-service vector against the detection engine.
- **Evidence:** `execute()` at line 384: `let result = tokio::time::timeout(Duration::from_secs(timeout_secs), self.execute_inner(...)).await;`. `execute_scheduled()` at lines 535-607: no `tokio::time::timeout` call anywhere. `self.config.timeout_secs` is declared but never used inside `execute_scheduled`. BC-2.11.006 postcondition 5: "wrapping the entire query execution lifecycle." BC-2.11.006 EC-11-017: "Timeout fires during sensor API fan-out (before DataFusion execution) — Same timeout error; the timeout covers the entire lifecycle."
- **Proposed Fix:** Wrap the body of `execute_scheduled` in `tokio::time::timeout(Duration::from_secs(self.config.timeout_secs), async { ... }).await` and map the `Elapsed` error to `PrismError::QueryTimeout { elapsed_ms }`. This mirrors the pattern in `execute()`. Since `execute_scheduled` returns `Arc<SessionContext>`, the timeout wrapper must encompass the materialization pipeline execution only (not the SessionContext construction, which is already done outside for the `Arc` wrapper). Alternatively, split the function similarly to `execute` + `execute_scheduled_inner`. Add a corresponding integration test: `test_execute_scheduled_timeout_returns_query_timeout_error`.

---

### MEDIUM

#### ADV-W3MT-P59-MED-001: `prism_diff_results` Schema Is a Complete Fabrication Relative to BC-2.15.011

- **Severity:** MEDIUM
- **Category:** spec-fidelity
- **Location:** `crates/prism-query/src/internal_tables.rs` — `diff_results_schema()` (lines 654-661)
- **Description:** While covered under CRIT-001 (all schemas wrong), the `prism_diff_results` case deserves separate attention because it has ZERO overlap with the authoritative schema. BC-2.15.011 §Postconditions line 56 explicitly defines the columns: "DiffState metadata columns only: `query_hash`, `client_id`, `previous_results_hash`, `epoch`, `counter`, `last_diff_time`." The implementation has: `diff_id`, `rule_id`, `timestamp`, `payload`. None of these match. Any query `SELECT * FROM prism_diff_results` would return four columns (`diff_id`, `rule_id`, `timestamp`, `payload`) that don't exist in the actual stored data — all rows would be empty strings because `build_batch_from_kv` only handles `StorageDomain::AuditBuffer` specially; all other domains fall through to a raw bytes fallback that produces garbage.
- **Evidence:** `diff_results_schema()` at line 654: `{ diff_id: Utf8, rule_id: Utf8, timestamp: Utf8, payload: Utf8 }`. `prism-storage/src/internal_tables.rs` `diff_results_columns()` at line 96: `{ query_hash: Text, client_id: Text, previous_results_hash: Text, epoch: UInt64, counter: UInt64, last_diff_time: Timestamp }`. No column name matches. `build_batch_from_kv` match arm: only `AuditBuffer` is handled specifically; `DiffResults` falls to the wildcard arm producing raw-bytes or empty-string fallback — the `diff_id` column would contain raw bincode bytes, not a UUID.
- **Proposed Fix:** Addressed by CRIT-001 fix (read from `prism-storage::internal_tables::all_descriptors()`). If fixing incrementally: replace `diff_results_schema()` with the authoritative schema `{ query_hash: Utf8, client_id: Utf8, previous_results_hash: Utf8, epoch: UInt64, counter: UInt64, last_diff_time: Utf8 }` and add a `StorageDomain::DiffResults` match arm in `build_batch_from_kv` that deserializes `DiffState` via the appropriate storage API.

---

### LOW

#### ADV-W3MT-P59-LOW-001: Stale Doc Comment in `supports_filters_pushdown` References `10_000` — Now 50K

- **Severity:** LOW
- **Category:** code-quality
- **Location:** `crates/prism-query/src/internal_tables.rs` — `RocksDbTableProvider::supports_filters_pushdown` docstring (line 325)
- **Description:** The `supports_filters_pushdown` doc comment reads: "**Why deferred:** The `PRISM_MAX_INTERNAL_TABLE_SCAN=10_000` cap bounds the worst-case scan size, making full-scan acceptable for wave-4." The constant was changed from `10_000` to `50_000` to close CRIT-002. The doc comment still says `10_000`, creating reader confusion about whether the cap is 10K or 50K and potentially signaling that full-scan is more acceptable than it actually is (50K rows unfiltered is a larger overhead than 10K rows).
- **Evidence:** `internal_tables.rs` line 325: `/// **Why deferred:** The PRISM_MAX_INTERNAL_TABLE_SCAN=10_000 cap bounds`. Current constant at line 66: `pub const PRISM_MAX_INTERNAL_TABLE_SCAN: usize = 50_000;`.
- **Proposed Fix:** Update the doc comment to `PRISM_MAX_INTERNAL_TABLE_SCAN=50_000`. Optionally strengthen the deferral justification: at 50K rows with an arbitrary schema, the overhead is higher and the case for filter pushdown is stronger — the comment should note that wave-5 filter pushdown tracking is even more important now.

---

#### ADV-W3MT-P59-LOW-002: Story Spec AC-3 References Non-Existent `PrismError::QueryError` Variant

- **Severity:** LOW
- **Category:** spec-fidelity
- **Location:** `.factory/stories/S-3.02-FOLLOWUP-RUNTIME-query-engine.md` — §AC-3 (line 226)
- **Description:** Story AC-3 states: "Then `PrismError::QueryError(E-QUERY-003)` is returned." There is no `PrismError::QueryError` variant in `prism-core/src/error.rs`. The actual error returned when the materialization cap is exceeded is `PrismError::QueryExecutionFailed { detail: "E-QUERY-003: record cap exceeded..." }`. While the integration test correctly checks for `E-QUERY-003` in the display string, the AC is technically untraceable to a conforming implementation because it references a non-existent type. Downstream story authors referencing AC-3 would write incorrect code.
- **Evidence:** Story line 226: `PrismError::QueryError(E-QUERY-003)`. `prism-core/src/error.rs`: no `QueryError` variant in the `PrismError` enum. The record-cap implementation in `materialization.rs` (via `increment_record_count`) returns `PrismError::QueryExecutionFailed { detail: format!("E-QUERY-003: ...") }`.
- **Proposed Fix:** Update story AC-3 to: "Then `PrismError::QueryExecutionFailed { detail }` is returned where `detail.starts_with("E-QUERY-003")`. No DataFusion execution begins." Update the integration test to additionally assert `matches!(err, PrismError::QueryExecutionFailed { .. })` before the string check.

---

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 1 |
| HIGH | 2 |
| MEDIUM | 1 |
| LOW | 2 |

**Overall Assessment:** block

**Convergence:** FINDINGS_REMAIN — iterate

**Readiness:** Requires revision before merge. CRITICAL finding (all 7 internal table schemas wrong vs prism-storage SoT, plus prism_aliases incorrectly scans RocksDB) must be resolved — this is a fundamental architectural issue that would cause runtime failures for all internal table queries referencing canonical columns like `client_id`, `severity_id`, `timestamp_ns`, `query_hash`, etc. HIGH findings (scan_truncated column not injected; execute_scheduled missing timeout) must also be resolved. LOW findings should be addressed or tracked as TD stories.

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 59 (PR-level, fresh-context, Pass 2) |
| **New findings** | 6 (1 CRIT, 2 HIGH, 1 MED, 2 LOW) |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 6/6 = 1.0 |
| **Median severity** | MEDIUM-HIGH |
| **Trajectory** | PR-P01: 13 findings → PR-P02: 6 findings |
| **Verdict** | FINDINGS_REMAIN — critical schema drift discovered requires fix burst before pass 3 |
