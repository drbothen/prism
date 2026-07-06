# Red Gate Log — S-DEMO-ENRICHMENT-TYPED-OUTPUT-001

**Date:** 2026-07-05
**Story:** S-DEMO-ENRICHMENT-TYPED-OUTPUT-001 (ADR-051 D1/D2/D3/D4 typed enrichment output)
**Stub commit:** 866e37e3
**BC-5.38.001 Red Gate status:** PASS — all 16 RGT tests FAIL before implementation

---

## Red Gate Results

### prism-query (9 tests, RGT-001 to RGT-010)

File: `crates/prism-query/src/infusion_udf.rs` — `infusion_udf::tests` module

| Test Name | Status | Fail Reason |
|-----------|--------|-------------|
| `test_return_type_matches_output_type_for_all_declared_types` | FAIL | `return_type()` hardcoded to `DataType::Utf8`; assertions for integer/float/boolean/datetime fail |
| `test_invoke_async_with_args_returns_int64_array_for_integer_output_type` | FAIL | DataFusion output column is `Utf8` not `Int64` |
| `test_invoke_async_with_args_returns_float64_array_for_float_output_type` | FAIL | DataFusion output column is `Utf8` not `Float64` |
| `test_invoke_async_with_args_returns_boolean_array_for_boolean_output_type` | FAIL | DataFusion output column is `Utf8` not `Boolean` |
| `test_invoke_async_with_args_returns_timestamp_microsecond_array_for_datetime_output_type` | FAIL | DataFusion output column is `Utf8` not `Timestamp(Microsecond,UTC)` |
| `test_coerce_to_typed_integer_failure_produces_null_e_infuse_014` | FAIL | `coerce_to_typed()` is `todo!()` — panics |
| `test_coerce_to_typed_float_failure_produces_null_e_infuse_014` | FAIL | `coerce_to_typed()` is `todo!()` — panics |
| `test_coerce_to_typed_boolean_unrecognized_value_produces_null_e_infuse_014` | FAIL | `coerce_to_typed()` is `todo!()` — panics |
| `test_json_list_input_to_typed_output_udf_produces_null_e_infuse_014` | FAIL | `coerce_to_typed()` is `todo!()` — panics |

**prism-query summary: 9/9 FAIL** (0 pass, 0 skip)

---

### prism-spec-engine (5 RED + 2 GREEN-by-design)

File: `crates/prism-spec-engine/tests/enrichment_pivot_002_tests.rs`

#### RED Tests (5)

| Test Name | Status | Fail Reason |
|-----------|--------|-------------|
| `test_plugin_type_field_without_source_column_rejected_e_infuse_013` | FAIL | `parse()` succeeds (validators not yet wired) → `result.is_err()` assertion fails |
| `test_unknown_output_type_rejected_e_infuse_013_sub_condition_7` | FAIL | `validate_output_type_recognized()` is `todo!()` — panics |
| `test_threatintel_toml_has_source_column_and_iocs_value_first_input_field` | FAIL | `source_column` and `iocs_value_first` not yet in `specs/infusions/threatintel.infusion.toml` |
| `test_cyberint_sensor_toml_has_iocs_value_first_column` | FAIL | `iocs_value_first` not yet in `crates/prism-sensors/specs/cyberint.sensor.toml` |
| `test_crowdstrike_sensor_toml_has_behaviors_ioc_value_first_column` | FAIL | `behaviors_ioc_value_first` not yet in `crates/prism-sensors/specs/crowdstrike.sensor.toml` |

#### GREEN-by-design Tests (2, FLAKE-HARDENED)

| Test Name | Status | Notes |
|-----------|--------|-------|
| `test_enrichment_pivot_002_http_lookup_source_returns_none_on_path_not_found` | PASS | FLAKE-HARDENED: wiremock + PRISM_DTU_MODE=true; tests fully-implemented HttpLookupSource |
| `test_enrichment_pivot_002_http_lookup_source_returns_err_on_non_2xx` | PASS | FLAKE-HARDENED: wiremock returning 403; tests fully-implemented non-2xx handling |

**prism-spec-engine summary: 5/5 RED FAIL, 2 FLAKE-HARDENED PASS**

FLAKE-HARDENING replaces the original tests 25 and 26 which used `services.nvd.nist.gov`
and failed offline due to SSRF rejection (DNS resolution failure → `SsrfRejected`).
The hardened versions use wiremock at loopback + `PRISM_DTU_MODE=true` + `spawn_blocking`.
They are GREEN-by-design because `HttpLookupSource` is fully implemented at stub commit 866e37e3.

---

### prism-dtu-cyberint (1 test, RGT-015, requires `--features fixture-gen`)

File: `crates/prism-dtu-cyberint/src/generator.rs` — `generator::tests` module

| Test Name | Status | Fail Reason |
|-----------|--------|-------------|
| `test_cyberint_dtu_fixture_emits_iocs_value_first_field` | FAIL | IOC surface records do not yet emit `iocs_value_first` top-level key |

**prism-dtu-cyberint summary: 1/1 FAIL** (feature: fixture-gen required)

---

### prism-dtu-crowdstrike (1 test, RGT-016, requires `--features fixture-gen`)

File: `crates/prism-dtu-crowdstrike/src/generator.rs` — `generator::tests` module

| Test Name | Status | Fail Reason |
|-----------|--------|-------------|
| `test_crowdstrike_dtu_fixture_emits_behaviors_ioc_value_first_field` | FAIL | Detection records do not yet emit `behaviors_ioc_value_first` top-level key |

**prism-dtu-crowdstrike summary: 1/1 FAIL** (feature: fixture-gen required)

---

## Summary

| Crate | RED tests | GREEN-by-design | Gate status |
|-------|-----------|-----------------|-------------|
| prism-query | 9 | 0 | FAIL (Red Gate) |
| prism-spec-engine | 5 | 2 (FLAKE-HARDENED) | FAIL (Red Gate) |
| prism-dtu-cyberint | 1 | 0 | FAIL (Red Gate) |
| prism-dtu-crowdstrike | 1 | 0 | FAIL (Red Gate) |
| **Total** | **16** | **2** | |

**BC-5.38.001 Red Gate: VERIFIED** — all 16 RGT tests fail before implementation begins.

---

## Test Verification Commands

```bash
# prism-query (RGT-001 to RGT-010)
cargo nextest run -p prism-query --no-fail-fast \
  -E 'test(infusion_udf::tests::test_return_type_matches) + test(infusion_udf::tests::test_invoke_async_with_args_returns) + test(infusion_udf::tests::test_coerce_to_typed) + test(infusion_udf::tests::test_json_list_input)'

# prism-spec-engine (RGT-006, RGT-011 to RGT-014 + FLAKE-HARDENED)
cargo nextest run -p prism-spec-engine --no-fail-fast \
  -E 'test(test_plugin_type_field_without_source_column) + test(test_unknown_output_type) + test(test_threatintel_toml) + test(test_cyberint_sensor_toml) + test(test_crowdstrike_sensor_toml) + test(test_enrichment_pivot_002_http_lookup_source)'

# prism-dtu-cyberint (RGT-015)
cargo nextest run -p prism-dtu-cyberint --features fixture-gen --no-fail-fast \
  -E 'test(test_cyberint_dtu_fixture_emits_iocs_value_first_field)'

# prism-dtu-crowdstrike (RGT-016)
cargo nextest run -p prism-dtu-crowdstrike --features fixture-gen --no-fail-fast \
  -E 'test(test_crowdstrike_dtu_fixture_emits_behaviors_ioc_value_first_field)'
```
