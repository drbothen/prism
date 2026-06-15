# Red Gate Log — S-5.02

**Story:** S-5.02 — prism-mcp: Tool Routing, Errors, and Client Scoping
**Branch:** feature/S-5.02
**Commit:** 27d3ffb5
**Date:** 2026-06-14
**Agent:** test-writer

## Red Gate Result: CONFIRMED

**155 tests run | 142 passed | 13 failed | 0 skipped**

All 13 new Red Gate tests FAIL before any implementation is committed. No previously-passing tests regressed.

## Files Created / Modified

| File | Action | Description |
|------|--------|-------------|
| `crates/prism-mcp/src/error_mapping.rs` | Modified | Added stub types `StructuredErrorFields`, stub functions `build_structured_error_response` (returns wrong `{"_stub": ...}` structuredContent) and `to_error_data_with_retry` (always returns `None` for retry_after_ms). No `todo!()`/`unimplemented!()` — POL-12 compliant. |
| `crates/prism-mcp/src/server.rs` | Modified | Added stub types `CapabilityStatus` (enum), `ResolutionStep`, `CapabilityEntry` (all `#[non_exhaustive]`). Added `ListCapabilitiesParams::for_client()` and `for_all_clients()` constructors to allow external crate construction of `#[non_exhaustive]` params. |
| `crates/prism-mcp/src/lib.rs` | Modified | Re-exported `CapabilityEntry`, `CapabilityStatus`, `ResolutionStep`, `ListCapabilitiesParams` from `prism_mcp` root. |
| `crates/prism-mcp/tests/tool_dispatch_tests.rs` | Modified | Added 13 Red Gate tests (block at end of file). |

## 13 Red Gate Tests — Failure Reasons

### BC-2.10.004 — validate_client_ids E-MCP-001 prefix

| Test Name | RED Failure Reason |
|-----------|-------------------|
| `test_BC_2_10_004_empty_client_id_returns_e_mcp_001_prefix` | Current `validate_client_ids` message is `"Invalid client_id '': must match [a-zA-Z0-9_-]{1,64} (BC-2.10.004)"` — does not start with `"E-MCP-001: invalid client_id format:"` |
| `test_BC_2_10_004_malformed_client_id_returns_e_mcp_001_prefix` | Same: wrong prefix for `"acme/../../etc"` |
| `test_BC_2_10_004_path_traversal_client_id_returns_e_mcp_001` | Same: wrong prefix for `"../passwd"` |
| `test_BC_2_10_004_well_formed_unknown_client_id_maps_to_e_cfg_100` | `build_structured_error_response` stub returns `{"_stub": ...}` — `structuredContent.error.original_params_valid` key absent → assertion fails |

### BC-2.10.007 — nested 9-field structured error shape

| Test Name | RED Failure Reason |
|-----------|-------------------|
| `test_BC_2_10_007_structured_error_has_nine_fields_and_meta_trust_level` | `build_structured_error_response` stub returns `{"_stub": "S-5.02 not implemented"}` — no `error` key, no `_meta` key → assertions on those keys fail |
| `test_BC_2_10_007_sensor_rate_limited_retry_after_seconds_ms_to_s_conversion` | `to_error_data_with_retry` stub always returns `None` for second tuple element → `Some(30)` assertion fails |
| `test_BC_2_10_007_no_retry_after_produces_null_not_absent` | `build_structured_error_response` stub returns wrong shape — `retry_after_seconds` key absent → null-not-absent assertion fails |
| `test_BC_2_10_007_upstream_message_isolation_from_prose_content` | `build_structured_error_response` stub returns wrong shape — `upstream_message` key absent → DI-006 isolation assertion fails |

### BC-2.10.011 — tri-state list_capabilities

| Test Name | RED Failure Reason |
|-----------|-------------------|
| `test_BC_2_10_011_enabled_capability_has_two_resolution_steps` | `list_capabilities` handler has no `WriteExecutor` wired in `PrismServer::new()` → returns `-32000 Internal error` → `expect()` assertion fails (documents that wired server is needed for the tri-state shape) |
| `test_BC_2_10_011_compile_time_disabled_has_one_deny_step` | Same: no WriteExecutor → `-32000` → `expect()` fails |
| `test_BC_2_10_011_runtime_disabled_has_two_steps_deny_at_runtime_tier` | Same: no WriteExecutor → `-32000` → `expect()` fails |
| `test_BC_2_10_011_cross_client_null_returns_summary_shape` | Same: no WriteExecutor → `-32000` → `expect()` fails |
| `test_BC_2_10_011_not_registered_tools_field_not_not_implemented` | Same: no WriteExecutor → `-32000` → `expect()` fails (also would assert `not_registered_tools` vs `not_implemented` rename) |

## Spec/Code Discrepancy — Surface to Orchestrator

**BC-2.10.007 v1.5 story spec** describes `PrismError::SensorRateLimited { retry_after_ms: Option<u64> }` (an optional ms value for cases where no Retry-After header was received).

**Actual code** (`crates/prism-core/src/error.rs`): `SensorRateLimited { sensor: String, retry_after_ms: u64 }` — the field is a required `u64` (not `Option<u64>`) and the field is named `sensor` (not `sensor_id`).

**Impact on test:** `test_BC_2_10_007_no_retry_after_produces_null_not_absent` was adjusted to use `PrismError::Internal` (a non-rate-limited error) to test the null-not-absent invariant for errors without a retry hint. The BC-2.10.007 `retry_after_seconds: null` contract still applies to non-rate-limited errors. The implementer must decide: (a) expand `SensorRateLimited` to carry `Option<u64>` per the BC spec, or (b) use `u64` and treat 0 as "no retry hint". This decision affects the `to_error_data_with_retry` signature and the structured error builder. Route to orchestrator for adjudication before implementation.

## No Regressions

Previously passing tests (142): all pass on this commit. The only test that changes failure mode was the POL-12 scan, which was correctly handled (no `todo!()`/`unimplemented!()` strings in `src/`).
