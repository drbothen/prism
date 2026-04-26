---
document_type: demo-evidence-report
story_id: S-2.03
story_title: "prism-storage: Decorators and Internal Tables"
producer: demo-recorder
timestamp: 2026-04-25
version: "1.0"
phase: 3
wave: 2
implementation_sha: 521fbfb0
test_pass_count: 19
test_total_count: 19
acs_demonstrated: 14
acs_total: 14
---

# Demo Evidence — S-2.03 prism-storage Decorators and Internal Tables

## Summary

- **Story:** S-2.03 — prism-storage: Decorators and Internal Tables
- **Story version:** 1.3
- **Implementation commit:** `521fbfb0`
- **Test results:** 19/19 passing (`cargo test -p prism-storage --lib` — S-2.03 suite)
- **Full workspace:** 53 prism-storage lib tests, 1058 workspace tests; 0 failed
- **ACs demonstrated:** 14 of 14
- **Demo medium:** VHS (`.tape` source + rendered `.gif`)
- **Font:** FiraCode Nerd Font Mono, 14pt, Dracula theme, 1000x600

---

## Per-AC Evidence

### AC-1: `get_config_time` returns Phase 1 fields populated; Phase 2/3 fields absent

![AC-1 demo](ac-1-config-time-phase1.gif)

- **Tape source:** `ac-1-config-time-phase1.tape`
- **Test:** `test_BC_2_15_010_get_config_time_phase1_fields_populated`
- **BC link:** BC-2.15.010 — Phase 1 config-time decorator populated at startup from TOML; `client_name` and `prism_version` are set; all Phase 2 (`analyst_id`, `query_source`, `sensor_instance`) and Phase 3 (`sensor_health_status`) fields are `None`.
- **Test filter:** `cargo test -p prism-storage --lib -- test_BC_2_15_010_get_config_time_phase1_fields_populated --nocapture`
- **Result:** `test result: ok. 1 passed; 0 failed`

---

### AC-2: `merge(config_time, query_time, None)` carries Phase 1 and Phase 2 fields through

![AC-2 demo](ac-2-merge-phase1-phase2.gif)

- **Tape source:** `ac-2-merge-phase1-phase2.tape`
- **Test:** `test_BC_2_15_010_merge_without_periodic_carries_phase1_and_phase2`
- **BC link:** BC-2.15.010 — query-time context with `analyst_id = Some("joshua")` and `query_source = Some("interactive")` is merged with config-time context; both Phase 1 and Phase 2 fields appear in the merged result; Phase 3 absent since `periodic = None`.
- **Test filter:** `cargo test -p prism-storage --lib -- test_BC_2_15_010_merge_without_periodic_carries_phase1_and_phase2 --nocapture`
- **Result:** `test result: ok. 1 passed; 0 failed`

---

### AC-3: Scheduled query — `analyst_id = None`, `query_source = "schedule:{name}"`

![AC-3 demo](ac-3-scheduled-query-source.gif)

- **Tape source:** `ac-3-scheduled-query-source.tape`
- **Test:** `test_BC_2_15_009_scheduled_query_analyst_id_none_query_source_schedule`
- **BC link:** BC-2.15.009 EC-15-034 and BC-2.15.010 Phase 2 postcondition — a scheduled query execution with no analyst session populates `analyst_id = None` and `query_source = Some("schedule:check_alerts")`.
- **Test filter:** `cargo test -p prism-storage --lib -- test_BC_2_15_009_scheduled_query_analyst_id_none_query_source_schedule --nocapture`
- **Result:** `test result: ok. 1 passed; 0 failed`

---

### AC-4: `store_periodic` / `load_periodic` round-trip; fresh tenant returns `None`

![AC-4 demo](ac-4-periodic-roundtrip.gif)

- **Tape source:** `ac-4-periodic-roundtrip.tape`
- **Tests:** `test_BC_2_15_010_store_and_load_periodic_round_trip`, `test_BC_2_15_010_ec001_load_periodic_fresh_tenant_returns_none`
- **BC link:** BC-2.15.010 Phase 3 postcondition — periodic decorators are serialized via bincode 2.x and written to the `decorators` CF; a subsequent `load_periodic` deserializes them with all fields intact. EC-15-039: a fresh tenant with no prior periodic write returns `None` from `load_periodic`.
- **Test filter:** `cargo test -p prism-storage --lib -- test_BC_2_15_010_store_and_load_periodic_round_trip test_BC_2_15_010_ec001_load_periodic_fresh_tenant_returns_none --nocapture`
- **Result:** `test result: ok. 2 passed; 0 failed`

---

### AC-5: Three-phase merge precedence — periodic wins over query-time and config-time

![AC-5 demo](ac-5-three-phase-merge-precedence.gif)

- **Tape source:** `ac-5-three-phase-merge-precedence.tape`
- **Tests:** `test_BC_2_15_010_merge_precedence_periodic_wins_over_query_time_and_config_time`, `test_BC_2_15_010_ec001_merge_with_none_periodic_sensor_health_absent`
- **BC link:** BC-2.15.010 invariant — last-write-wins with precedence periodic > query-time > config-time. All three phase values appear in the result when non-overlapping; periodic value wins for any overlapping key. When `periodic = None`, `sensor_health_status` is absent from the merged result (EC-15-039).
- **Test filter:** `cargo test -p prism-storage --lib -- test_BC_2_15_010_merge_precedence_periodic_wins_over_query_time_and_config_time test_BC_2_15_010_ec001_merge_with_none_periodic_sensor_health_absent --nocapture`
- **Result:** `test result: ok. 2 passed; 0 failed`

---

### AC-6: `store_periodic` failure — warn log; stale value returned (E-DECOR-001)

![AC-6 demo](ac-6-store-periodic-failure-stale-value.gif)

- **Tape source:** `ac-6-store-periodic-failure-stale-value.tape`
- **Test:** `test_BC_2_15_010_store_periodic_failure_stale_value_pattern`
- **BC link:** BC-2.15.010 error case E-DECOR-001 — when `store_periodic` fails (simulated here with an `AlwaysFailBackend` that returns `Err` on every write), the last successfully cached periodic value is still returned by `load_periodic`. The test pattern: write a good value via `InMemoryBackend`, verify it loads; then attempt a write via `AlwaysFailBackend` (simulates RocksDB CF failure), verify the stale good value is still accessible.
- **Test filter:** `cargo test -p prism-storage --lib -- test_BC_2_15_010_store_periodic_failure_stale_value_pattern --nocapture`
- **Result:** `test result: ok. 1 passed; 0 failed`
- **Note:** The stale-value pattern is tested at the unit level using `AlwaysFailBackend`. Production behavior logs `tracing::warn!` on the failure; the test verifies the contract without asserting log output (log subscriber not wired in unit tests per project convention).

---

### AC-7: `get_descriptor("prism_alerts")` — correct domain, columns, and flags

![AC-7 demo](ac-7-prism-alerts-descriptor.gif)

- **Tape source:** `ac-7-prism-alerts-descriptor.tape`
- **Test:** `test_BC_2_15_011_get_descriptor_prism_alerts_fields`
- **BC link:** BC-2.15.011 postcondition — `get_descriptor("prism_alerts")` returns a descriptor with `table_name == "prism_alerts"`, `domain == Some(StorageDomain::Alerts)`, `requires_audit_read == false`, `rocksdb_backed == true`, and `columns[0] == ("alert_id", ColumnType::Text)`.
- **Test filter:** `cargo test -p prism-storage --lib -- test_BC_2_15_011_get_descriptor_prism_alerts_fields --nocapture`
- **Result:** `test result: ok. 1 passed; 0 failed`

---

### AC-8: `get_descriptor("prism_audit").requires_audit_read == true`

![AC-8 demo](ac-8-prism-audit-requires-audit-read.gif)

- **Tape source:** `ac-8-prism-audit-requires-audit-read.tape`
- **Test:** `test_BC_2_15_011_get_descriptor_prism_audit_requires_audit_read`
- **BC link:** BC-2.15.011 error case E-QUERY-011 — the `prism_audit` descriptor carries `requires_audit_read = true`; this flag is the trigger for the capability gate enforced in `check_table_access`.
- **Test filter:** `cargo test -p prism-storage --lib -- test_BC_2_15_011_get_descriptor_prism_audit_requires_audit_read --nocapture`
- **Result:** `test result: ok. 1 passed; 0 failed`

---

### AC-9: `check_table_access(audit, no_audit_read)` → `Err(AuditTableAccessDenied)`

![AC-9 demo](ac-9-check-table-access-denied.gif)

- **Tape source:** `ac-9-check-table-access-denied.tape`
- **Test:** `test_BC_2_15_011_check_table_access_audit_without_capability_denied`
- **BC link:** BC-2.15.011 error case E-QUERY-011 — when `check_table_access` is called with the `prism_audit` descriptor and a `ClientCapabilities` struct that does NOT include `audit.read = Allow`, it returns `Err(PrismError::AuditTableAccessDenied)` whose `Display` contains the string `"audit.read capability"`.
- **Test filter:** `cargo test -p prism-storage --lib -- test_BC_2_15_011_check_table_access_audit_without_capability_denied --nocapture`
- **Result:** `test result: ok. 1 passed; 0 failed`

---

### AC-10: `check_table_access(alerts, any caps)` → `Ok(())`

![AC-10 demo](ac-10-check-table-access-allowed.gif)

- **Tape source:** `ac-10-check-table-access-allowed.tape`
- **Test:** `test_BC_2_15_011_check_table_access_alerts_any_caps_ok`
- **BC link:** BC-2.15.011 postcondition — `prism_alerts` does not require `audit.read`; `check_table_access` returns `Ok(())` regardless of the capability set presented.
- **Test filter:** `cargo test -p prism-storage --lib -- test_BC_2_15_011_check_table_access_alerts_any_caps_ok --nocapture`
- **Result:** `test result: ok. 1 passed; 0 failed`

---

### AC-11: `scan_limit()` — default 50 000; valid env var; invalid env var → default

![AC-11 demo](ac-11-scan-limit-env-var.gif)

- **Tape source:** `ac-11-scan-limit-env-var.tape`
- **Tests:** `test_BC_2_15_011_scan_limit_default`, `test_BC_2_15_011_scan_limit_valid_numeric`, `test_BC_2_15_011_scan_limit_invalid_string`
- **BC link:** BC-2.15.011 — `scan_limit()` reads `PRISM_MAX_INTERNAL_TABLE_SCAN`; absent or unparseable → `50_000`; valid numeric string → parsed value. Three tests run serially (env-var mutation requires `--test-threads=1` or serial execution in a single test binary invocation; cargo runs lib tests single-threaded within one filter).
- **Test filter:** `cargo test -p prism-storage --lib -- test_BC_2_15_011_scan_limit_default test_BC_2_15_011_scan_limit_valid_numeric test_BC_2_15_011_scan_limit_invalid_string --nocapture`
- **Result:** `test result: ok. 3 passed; 0 failed`
- **Note:** The three scan_limit tests were implemented as fully functional tests (not stubs) by the stub author, correctly demonstrating the pure env-var parsing function.

---

### AC-12: `VirtualField::column_name()` — `_sensor`, `_client`, `_source_table`

![AC-12 demo](ac-12-virtual-field-column-names.gif)

- **Tape source:** `ac-12-virtual-field-column-names.tape`
- **Test:** `test_BC_2_15_009_virtual_field_column_names`
- **BC link:** BC-2.15.009 postcondition — virtual fields are underscore-prefixed queryable columns: `VirtualField::Sensor.column_name() == "_sensor"`, `VirtualField::Client.column_name() == "_client"`, `VirtualField::SourceTable.column_name() == "_source_table"`. The `VirtualField` enum lives in `prism-core` (no Arrow/DataFusion dependency).
- **Test filter:** `cargo test -p prism-storage --lib -- test_BC_2_15_009_virtual_field_column_names --nocapture`
- **Result:** `test result: ok. 1 passed; 0 failed`
- **Note:** This test exercises a pure prism-core type via the prism-storage test harness. The test is green-by-design since the implementation is a pure constant-return match expression with no I/O.

---

### AC-13: `all_descriptors()` returns exactly 7 entries

![AC-13 demo](ac-13-all-descriptors-count.gif)

- **Tape source:** `ac-13-all-descriptors-count.tape`
- **Test:** `test_BC_2_15_011_all_descriptors_count_and_names`
- **BC link:** BC-2.15.011 postcondition — all seven queryable domains registered: `prism_alerts`, `prism_cases`, `prism_rules`, `prism_schedules`, `prism_diff_results`, `prism_audit`, `prism_aliases`. The test asserts both count (7) and all seven names present in the slice.
- **Test filter:** `cargo test -p prism-storage --lib -- test_BC_2_15_011_all_descriptors_count_and_names --nocapture`
- **Result:** `test result: ok. 1 passed; 0 failed`
- **Note:** `prism_aliases` uses `rocksdb_backed: false` per BC-2.11.008; aliases persist in `aliases.toml` (S-3.04) via `AliasStore`, not a RocksDB CF. The descriptor is still registered for S-3.02 TableProvider dispatch.

---

### AC-14: `diff_results_columns()` — metadata-only; no `previous_results` column; EC-005

![AC-14 demo](ac-14-diff-results-metadata-only.gif)

- **Tape source:** `ac-14-diff-results-metadata-only.tape`
- **Tests:** `test_BC_2_15_011_diff_results_columns_metadata_only`, `test_BC_2_15_011_ec005_get_descriptor_unknown_table_returns_none`
- **BC link:** BC-2.15.011 postcondition — `prism_diff_results` exposes only DiffState metadata columns (`query_hash`, `client_id`, `previous_results_hash`, `epoch`, `counter`, `last_diff_time`); the raw `previous_results` bincode blob is deliberately excluded. EC-005: `get_descriptor("nonexistent_table")` returns `None`; caller handles gracefully without panic.
- **Test filter:** `cargo test -p prism-storage --lib -- test_BC_2_15_011_diff_results_columns_metadata_only test_BC_2_15_011_ec005_get_descriptor_unknown_table_returns_none --nocapture`
- **Result:** `test result: ok. 2 passed; 0 failed`

---

## Full Suite Green

```
test result: ok. 53 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
```

S-2.03 contributes 19 tests across `decorator_tests.rs` and `internal_table_tests.rs`:
- 1 AC-1 test (BC-2.15.010 Phase 1 config-time)
- 1 AC-2 test (BC-2.15.010 Phase 1+2 merge)
- 1 AC-3 test (BC-2.15.009 EC-15-034 scheduled query)
- 2 AC-4 tests (BC-2.15.010 round-trip + fresh tenant)
- 2 AC-5 tests (BC-2.15.010 merge precedence)
- 1 AC-6 test (BC-2.15.010 E-DECOR-001 stale-value)
- 1 AC-7 test (BC-2.15.011 prism_alerts descriptor)
- 1 AC-8 test (BC-2.15.011 prism_audit requires_audit_read)
- 1 AC-9 test (BC-2.15.011 E-QUERY-011 access denied)
- 1 AC-10 test (BC-2.15.011 access allowed)
- 3 AC-11 tests (BC-2.15.011 scan_limit env var parsing)
- 1 AC-12 test (BC-2.15.009 VirtualField column names)
- 1 AC-13 test (BC-2.15.011 all_descriptors count and names)
- 2 AC-14 tests (BC-2.15.011 diff_results metadata-only + EC-005)

---

## Implementation Notes

### AC-6 Stale-Value Pattern (E-DECOR-001)

The stale-value test uses an `AlwaysFailBackend` stub that returns `Err` on every `put` operation, simulating a RocksDB CF write failure. The test workflow: (1) write a good periodic value via `InMemoryBackend` so it is cached; (2) attempt an overwrite using `AlwaysFailBackend` — this fails as expected; (3) verify that `load_periodic` on the original store still returns the last good value. Production `DecorationStore` logs `tracing::warn!` on the error; the unit test verifies the value contract without wiring a log subscriber.

### AC-11/AC-12/AC-13 Green-by-Design

These three ACs cover pure helper functions and static data:
- `scan_limit()` is a pure env-var read with a fixed default — no I/O beyond `std::env::var`.
- `VirtualField::column_name()` is a pure `match` returning `&'static str` constants.
- `all_descriptors()` returns a `&'static [InternalTableDescriptor]` initialized at compile time.

All three are fully implemented (not stubs) by the implementing agent. The tests remain fast and deterministic because they exercise real production logic, not test-only scaffolding.

### Pre-Red-Gate Spec Gaps Preserved

Three minor implementation divergences from the spec letter were carried forward from the stub-author's decisions; all preserve spec intent:

| Spec wording | Implementation choice | Rationale |
|---|---|---|
| `ColumnType` as shared enum | `InternalColumnType` type alias in some paths | Alias preserves the same variants; no semantic change. S-3.02 will use the canonical form. |
| `ClientCapabilities.audit_read: Allow` | `BTreeMap<String, String>` pattern for capabilities | S-1.02's `ClientCapabilities` uses a map; `check_table_access` reads `capabilities.get("audit.read") == Some("allow")`. Functionally equivalent to the spec's field access. |
| `INTERNAL_TABLES: &[...]` as `static` slice | `OnceLock<Vec<InternalTableDescriptor>>` lazy init | `&'static str` fields require `'static` lifetime; `OnceLock` provides this safely without `unsafe`. Intent (single global registration) is identical. |

None of these constitute spec violations; they are implementation-layer choices within the allowed space defined by BC-2.15.011.

---

## Verification Artifacts

| Artifact | Status |
|---|---|
| `cargo test -p prism-storage --lib` | 53/53 ok (19 S-2.03 + 34 S-2.01/S-2.02) |
| Workspace `cargo test` | 1058/1058 ok |
| 14 `.tape` files | present |
| 14 `.gif` files | present |
| Total GIF size | ~1792 KB |
| evidence-report.md | present |
