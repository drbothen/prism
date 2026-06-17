---
story: S-1.14-REDO
phase: Red Gate (Failing Tests)
date: 2026-06-17
agent: test-writer
commit_sha: 57de1b51
branch: feature/S-1.14-REDO
---

# Red Gate Log — S-1.14-REDO Full Infusion Engine

## Summary

**Result: RED GATE PASSED** — 27 new tests fail before implementation; 10 legitimately green tests covering already-implemented code pass as expected.

| Metric | Value |
|--------|-------|
| Test file | `crates/prism-spec-engine/tests/bc_2_19_redo_test.rs` |
| Tests added | 37 |
| Tests failing (RED) | 27 |
| Tests passing (legitimately GREEN) | 10 |
| Existing regression | 0 (infusion_tests.rs 41/41 still pass) |
| Commit SHA | `57de1b51` |
| Compile status | CLEAN (1 cosmetic fmt warning, fixed by `cargo fmt`) |

## Red Gate Verification Command

```
cargo nextest run -p prism-spec-engine --test bc_2_19_redo_test --no-fail-fast
# Summary: 37 tests run: 10 passed, 27 failed, 0 skipped
```

## AC / VP Coverage Matrix

| AC / VP | BC ID | Tests | Red Gate Failure Reason |
|---------|-------|-------|------------------------|
| AC-1 MMDB load (nonexistent file) | BC-2.19.001 | `test_BC_2_19_001_mmdb_source_load_nonexistent_file_returns_error` | `todo!()` in `MmdbSource::load` panics |
| AC-1 MMDB load (valid file) | BC-2.19.001 | `test_BC_2_19_001_mmdb_source_load_valid_file_succeeds` | `todo!()` in `MmdbSource::load` panics |
| AC-1 MMDB enrich_single (doc IP) | BC-2.19.001 | `test_BC_2_19_001_mmdb_source_enrich_single_documentation_ip_returns_none` | `todo!()` in `MmdbSource::load` panics |
| AC-1 MMDB enrich_single (invalid IP) | BC-2.19.001 | `test_BC_2_19_001_mmdb_source_enrich_single_invalid_ip_returns_none` | `todo!()` in `MmdbSource::load` panics |
| AC-1 MMDB enrich_batch | BC-2.19.001 | `test_BC_2_19_001_mmdb_source_enrich_batch_returns_parallel_results` | `todo!()` in `MmdbSource::load` panics |
| AC-1 Loader parses maxmind_mmdb type | BC-2.19.001 | `test_BC_2_19_001_infusion_loader_parses_maxmind_mmdb_type` | `Err(UnknownSourceType { "maxmind_mmdb" })` — S-1.14-REDO must implement |
| AC-1 CSV load (nonexistent file) | BC-2.19.001 | `test_BC_2_19_001_csv_source_load_nonexistent_file_returns_error` | `todo!()` in `CsvSource::load` panics |
| AC-1 CSV load (valid fixture) | BC-2.19.001 | `test_BC_2_19_001_csv_source_load_valid_fixture_succeeds` | `todo!()` in `CsvSource::load` panics |
| AC-1 CSV enrich_single (known key) | BC-2.19.001 | `test_BC_2_19_001_csv_source_enrich_single_known_key_returns_correct_value` | `todo!()` in `CsvSource::load` panics |
| AC-1 CSV enrich_single (unknown key) | BC-2.19.001 | `test_BC_2_19_001_csv_source_enrich_single_unknown_key_returns_none` | `todo!()` in `CsvSource::load` panics |
| AC-1 CSV enrich_batch | BC-2.19.001 | `test_BC_2_19_001_csv_source_enrich_batch_returns_parallel_results` | `todo!()` in `CsvSource::load` panics |
| AC-1 Loader parses csv type | BC-2.19.001 | `test_BC_2_19_001_infusion_loader_parses_csv_type` | `Err(UnknownSourceType { "csv" })` |
| AC-1 JSON load (nonexistent file) | BC-2.19.001 | `test_BC_2_19_001_json_lookup_source_load_nonexistent_file_returns_error` | `todo!()` in `JsonLookupSource::load` panics |
| AC-1 JSON load (valid file) | BC-2.19.001 | `test_BC_2_19_001_json_lookup_source_load_valid_json_succeeds` | `todo!()` in `JsonLookupSource::load` panics |
| AC-1 JSON enrich_single (known key) | BC-2.19.001 | `test_BC_2_19_001_json_lookup_source_enrich_single_known_key_returns_value` | `todo!()` in `JsonLookupSource::load` panics |
| AC-1 JSON enrich_single (unknown key) | BC-2.19.001 | `test_BC_2_19_001_json_lookup_source_enrich_single_unknown_key_returns_none` | `todo!()` in `JsonLookupSource::load` panics |
| AC-1 JSON malformed input | BC-2.19.001 | `test_BC_2_19_001_json_lookup_source_load_malformed_json_returns_error` | `todo!()` in `JsonLookupSource::load` panics |
| AC-1 Loader parses json_lookup type | BC-2.19.001 | `test_BC_2_19_001_infusion_loader_parses_json_lookup_type` | `Err(UnknownSourceType { "json_lookup" })` |
| AC-1 + AC-9 load_all MMDB+CSV specs | BC-2.19.001 | `test_BC_2_19_001_load_all_produces_specs_for_all_local_lookup_source_types` | 0 specs returned (2 `UnknownSourceType` errors) |
| AC-9 non-fatal failure | BC-2.19.001 | `test_BC_2_19_001_load_all_non_fatal_per_source_failure_continues_loading` | 0 specs (both CSV specs → `UnknownSourceType`) |
| AC-7 LRU hit within TTL | BC-2.19.002 | `test_BC_2_19_002_lru_cache_hit_within_ttl_returns_value` | `todo!()` in `InfusionLruCache::insert` |
| AC-7 LRU miss | BC-2.19.002 | `test_BC_2_19_002_lru_cache_miss_returns_none` | `todo!()` in `InfusionLruCache::get` |
| AC-7 TTL zero entry | BC-2.19.002 | `test_BC_2_19_002_lru_cache_ttl_zero_entry_is_expired_immediately` | `todo!()` in `InfusionLruCache::insert` |
| AC-7 LRU capacity eviction | BC-2.19.002 | `test_BC_2_19_002_lru_cache_capacity_evicts_lru_entry` | `todo!()` in `InfusionLruCache::insert` |
| AC-7 composite key isolation | BC-2.19.002 | `test_BC_2_19_002_lru_cache_composite_key_isolates_infusion_ids` | `todo!()` in `InfusionLruCache::insert` |
| AC-8 Tier 2 LRU hit | BC-2.19.002 | `test_BC_2_19_002_ac_8_tier2_lru_hit_returns_value_without_source_call` | `todo!()` in `InfusionLruCache::insert` |
| AC-8 Tier 3 bypassed | BC-2.19.002 | `test_BC_2_19_002_ac_8_tier3_bypassed_when_tier2_hits` | `todo!()` in `InfusionLruCache::insert` |

## Legitimately Green Tests (10)

These tests cover already-implemented code and correctly pass before S-1.14-REDO implementation:

| Test | Why Green |
|------|-----------|
| `test_BC_2_19_001_e_infuse_008_plugin_call_failed_message_format` | `InfusionError::PluginCallFailed` already defined in `prism-core/src/error.rs` |
| `test_BC_2_19_001_e_infuse_008_plugin_call_failed_carries_all_required_fields` | Same — error variant already implemented |
| `test_BC_2_19_001_e_infuse_008_plugin_call_failed_reason_no_credential_values` | Same — structural invariant test |
| `test_BC_2_19_001_e_infuse_008_plugin_call_failed_display_contains_all_fields` | Same — Display impl already in place |
| `test_BC_2_19_001_vp_048_mirror_one_field_produces_one_descriptor` | `InfusionRegistry::load_spec` fully implemented (S-DEMO-ENRICHMENT-PIVOT-001) |
| `test_BC_2_19_001_vp_048_mirror_sixteen_fields_produces_sixteen_descriptors` | Same |
| `test_BC_2_19_001_vp_048_mirror_duplicate_field_name_in_spec_errors` | Same |
| `test_BC_2_19_002_ac_8_tier1_hit_avoids_source_call` | `QueryScopedInfusionCache` fully implemented |
| `test_BC_2_19_002_vp_049_ac_8_five_hundred_events_thirty_unique` | Same |
| `test_BC_2_19_002_vp_049_ec_003_ten_thousand_events_two_hundred_unique` | Same |

## Notes for Implementer

1. **MMDB fixture required**: `crates/prism-spec-engine/fixtures/test.mmdb` must be created (valid MaxMind GeoLite2-City MMDB). The `test_BC_2_19_001_mmdb_source_load_valid_file_succeeds` test will fail with assertion `result.is_ok()` until the fixture exists alongside the implementation. Use the maxminddb-test-utils or a minimal synthetic MMDB file.

2. **`MmdbSource::load` API**: Uses `maxminddb 0.28` `Reader::open_readfile(path)`. Returns `LookupResult<'_, T>` from `lookup()`, NOT bare `T` — call `.deserialize::<T>()`. See stub comment in `mmdb.rs`.

3. **Loader TOML parsing**: `InfusionLoader::parse` must handle `source.type = "maxmind_mmdb"` / `"csv"` / `"json_lookup"` and return `InfusionType::LocalLookup` with a populated `InfusionSourceConfig`. Currently returns `Err(UnknownSourceType)`.

4. **LRU API**: `lru 0.17` uses `.put(key, value)` for insert and `.get(&key)` returns `Option<&V>`. The field is `self.inner` (not `_inner`) — it was renamed in the stub for S-1.14-REDO.

5. **load_all non-fatal**: AC-9 test expects that a bad spec (empty infusion_id) returns 1 error but still loads the valid CSV spec. The current implementation returns 0 specs because BOTH error on `UnknownSourceType`. After implementing CSV parsing, the valid spec will load and the invalid spec will error on empty infusion_id.

6. **VP-049 proptest** in `src/proofs/infusion_dedup.rs` is already implemented and covers `QueryScopedInfusionCache`. The concrete EC-003 and AC-8 proptest cases in this file complement it.

7. **VP-048 Kani proof** in `src/proofs/infusion_spec.rs` is `#[cfg(kani)]` gated and will not run in nextest. The mirror tests here provide the concrete unit-test layer.
