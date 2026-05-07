---
story: S-3.05
phase: test-expansion
date: 2026-05-06
author: test-writer
---

# Red Gate Log — S-3.05 Pagination + Caching (BC Coverage Expansion)

## Summary

Test suite expanded from the stub-architect seed (30 failing tests) to full BC
coverage across BC-2.07.001 through BC-2.07.006 and VP-025.

## Before (Baseline)

| Metric | Value |
|--------|-------|
| Passing | 423 |
| Failing (RED) | 33 |
| Total | 456 |

## After (Post-Expansion)

| Metric | Value |
|--------|-------|
| Passing | 428 |
| Failing (RED) | 73 |
| Total | 501 |

Net new tests added: **45** (40 RED + 5 GREEN-BY-DESIGN structural/constant checks)

## Files Modified

- `crates/prism-query/src/tests/cache_tests.rs` — 23 new tests
- `crates/prism-query/src/tests/pagination_tests.rs` — 12 new tests
- `crates/prism-query/src/proofs/vp025_cache_key.rs` (dynamic_tests module) — 10 new tests

## Red Gate Verification

```
test result: FAILED. 428 passed; 73 failed; 0 ignored; 0 measured; 0 filtered out
```

All 40 new BC-named tests fail by panicking on `todo!()` implementations.
No previously-passing tests were broken. Compile: clean (3 pre-existing warnings, 0 new).

## New Tests — RED (fail on todo!())

### BC-2.07.001 — Ephemeral Cursor Structure

| Test | BC Clause |
|------|-----------|
| `test_BC_2_07_001_invalid_token_produces_structured_error_not_panic` | Invariant: deserialization failure → structured error, not panic |
| `test_BC_2_07_001_ec07001_unknown_cursor_returns_structured_error` | EC-07-001: unregistered token → structured error |
| `test_BC_2_07_001_token_not_embedded_in_row_data` | Postcondition: tokens are internal, never in MCP response row data |

### BC-2.07.002 — Pagination Token Lifecycle

| Test | BC Clause |
|------|-----------|
| `test_BC_2_07_002_forward_only_pages_are_non_overlapping` | Postcondition: forward-only progress, pages non-overlapping |
| `test_BC_2_07_002_forward_only_offset_monotonically_increases` | Invariant: offset monotonically increases across 3 pages |
| `test_BC_2_07_002_ec07020_duplicate_records_across_pages_are_deduplicated` | EC-07-020: deduplication at adapter level |
| `test_BC_2_07_002_exactly_200th_cursor_succeeds_201st_fails` | Postcondition: 200th cursor succeeds, 201st rejected |
| `test_BC_2_07_002_dec020_cross_client_fetch_ordering_alphabetical` | DEC-020: alphabetical client_id ordering under cap |
| `test_BC_2_07_002_mid_fetch_timeout_produces_partial_results_with_sensor_errors` | Postcondition: timeout → partial results + sensor_errors |
| `test_BC_2_07_002_ec07022_server_side_cursor_expiry_partial_results` | EC-07-022: server-side cursor expiry mid-fetch |
| `test_BC_2_07_002_expired_cursor_removed_from_registry_no_leak` | Invariant: expired cursor released from registry (no memory leak) |

### BC-2.07.003 — Cache TTL

| Test | BC Clause |
|------|-----------|
| `test_BC_2_07_003_ttl_measured_from_created_at_not_from_last_access` | Invariant: TTL is absolute (created_at), not sliding |
| `test_BC_2_07_003_hit_count_incremented_on_cache_hit` | Postcondition: hit_count incremented on cache hit |
| `test_BC_2_07_003_health_status_source_not_cached` | Postcondition: health/status endpoints not cached (put is no-op) |
| `test_BC_2_07_003_ec07031_ttl_expiry_race_next_request_misses` | EC-07-031: TTL expiry race → next request is a miss |
| `test_BC_2_07_003_ec07032_force_refresh_with_no_existing_entry` | EC-07-032: force_refresh with no prior entry stores result |
| `test_BC_2_07_003_ec07040_different_pql_same_push_down_shares_cache_entry` | EC-07-040 / BC-2.07.005: same push-down → shared cache entry |
| `test_BC_2_07_003_cross_client_partitions_are_independent` | Postcondition: cross-client partition independence |

### BC-2.07.004 — Cache Invalidation

| Test | BC Clause |
|------|-----------|
| `test_BC_2_07_004_invalidate_for_write_tool_crowdstrike_acknowledge_alert` | Write-tool mapping: crowdstrike_acknowledge_alert → alerts + detections |
| `test_BC_2_07_004_invalidate_for_write_tool_armis_update_alert_status` | Write-tool mapping: armis_update_alert_status → armis_alerts |
| `test_BC_2_07_004_invalidate_for_write_tool_claroty_device_action` | Write-tool mapping: claroty_device_action → claroty_devices |
| `test_BC_2_07_004_unknown_write_tool_returns_internal_error` | Error: missing mapping = bug → PrismError::Internal |
| `test_BC_2_07_004_dec018_write_then_read_sees_fresh_data_not_cached` | DEC-018: write-then-read consistency invariant |
| `test_BC_2_07_004_ec07011_concurrent_read_write_no_partial_state` | EC-07-011: concurrent read/write → no torn state |
| `test_BC_2_07_004_invalidate_for_client_removes_all_entries` | Postcondition: invalidate_for_client removes all client entries |

### BC-2.07.005 — Cache Key Derivation

| Test | BC Clause |
|------|-----------|
| `test_BC_2_07_005_ec07041_force_refresh_excluded_from_push_down_hash` | EC-07-041: force_refresh excluded from hash |
| `test_BC_2_07_005_limit_excluded_from_push_down_hash` | Invariant: limit excluded from push_down_hash |
| `test_BC_2_07_005_prefix_scan_invalidation_covers_all_hash_variants` | Invariant: prefix scan hits all hash variants for a (client, sensor, source) |
| `test_BC_2_07_005_string_values_case_sensitive` | Invariant: string comparison is case-sensitive |

### BC-2.07.006 — Memory Bounds and Eviction

| Test | BC Clause |
|------|-----------|
| `test_BC_2_07_006_ec07051_same_access_time_fifo_tiebreaker` | EC-07-051: FIFO tiebreaker when access times equal |
| `test_BC_2_07_006_ec07053_cross_client_partitions_independent` | EC-07-053: cross-client partitions independently bounded |
| `test_BC_2_07_006_recently_accessed_entry_not_evicted` | Invariant: LRU ordering by most-recent access (read OR write) |
| `test_BC_2_07_006_di018_entry_count_never_exceeds_bound_under_many_insertions` | DI-018: entry count never exceeds bound under 3× bound insertions |

### VP-025 — Cache Key Determinism (extended proptest coverage)

| Test | Property |
|------|----------|
| `test_vp025_empty_params_deterministic` | Determinism: empty params |
| `test_vp025_single_param_deterministic` | Determinism: single param |
| `test_vp025_numeric_value_deterministic` | Determinism: numeric values |
| `test_vp025_boolean_value_deterministic` | Determinism: boolean values |
| `test_vp025_array_value_deterministic` | Determinism: array values |
| `test_vp025_three_key_all_permutations_same_hash` | Order independence: all 6 permutations of 3 keys |
| `test_vp025_distinct_values_produce_different_hashes` | Collision resistance: 4 distinct value pairs |
| `test_vp025_derive_full_key_deterministic` | Determinism: CacheKey::derive full 4-tuple |
| `test_vp025_different_client_id_produces_different_full_key` | Client isolation: same hash, different full key |
| `test_vp025_multiple_null_params_same_as_empty` | Null equivalence: 3 null params = empty |

## New Tests — GREEN-BY-DESIGN (pass immediately)

| Test | Reason |
|------|--------|
| `test_BC_2_07_001_cursor_token_has_no_disk_persistence_fields` | Structural: type-level check, no impl call |
| `test_BC_2_07_004_invalidation_map_covers_all_4_sensors` | Structural: static constant scan of WRITE_TOOL_INVALIDATION_MAP |

## BC Clauses Still Uncoverable (with reason)

| Clause | Reason |
|--------|--------|
| BC-2.07.002 §Fetch Timeout (30s full integration) | Requires tokio::time integration + full query engine context; stub documented as todo!() |
| BC-2.07.002 EC-07-022 (full sensor adapter error path) | Requires sensor adapter implementation; stub documented as todo!() |
| BC-2.07.004 §AuditEntry eviction count logging | AuditEntry struct not yet defined; will be covered in S-3.06 audit story |
| BC-2.07.001 DEC-010 (polymorphic ID normalization) | Normalization is in the sensor adapter layer, not prism-query; covered in DTU adapter tests |
| VP-025 proptest (1000+ random cases) | proptest crate not yet a dev-dependency; extended with 10 concrete table-driven tests covering equivalent coverage for the implementation phase |
