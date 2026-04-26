---
document_type: red-gate-log
level: ops
version: "1.0"
status: verified
producer: test-writer
timestamp: 2026-04-25T00:00:00Z
phase: 3
inputs:
  - .factory/stories/S-2.05-audit-events.md
  - .factory/specs/behavioral-contracts/BC-2.05.005-credential-access-audit.md
  - .factory/specs/behavioral-contracts/BC-2.05.007-vector-pipeline-compatibility.md
  - .factory/specs/behavioral-contracts/BC-2.05.009-feature-flag-evaluation-audit.md
  - .factory/specs/behavioral-contracts/BC-2.05.010-confirmation-token-audit.md
  - crates/prism-audit/src/credential_events.rs
  - crates/prism-audit/src/vector_compat.rs
  - crates/prism-audit/src/flag_events.rs
  - crates/prism-audit/src/token_events.rs
input-hash: "8e18961"
traces_to: "S-2.05"
stub_architect_agent: "stub-architect"
stub_compile_verified: true
test_writer_agent: "test-writer"
red_gate_verified: true
---

# Red Gate Log: S-2.05 — prism-audit: Specialized Audit Events

## Summary

| Story | Tests Written | RED | GREEN-BY-DESIGN | Total New | RED_RATIO | Gate |
|-------|--------------|-----|-----------------|-----------|-----------|------|
| S-2.05 | 35 | 19 | 16 | 35 | 19/35 = 54.3% | PASS (>= 50%) |

## Stubs Created (from stub commit dccc1ef3)

All 7 `todo!()` bodies are in production code; 4 trivial pure-data helpers are fully
implemented:

| Function | Location | Stub type |
|----------|----------|-----------|
| `emit_credential_event()` | `credential_events.rs:123` | `todo!()` |
| `to_vector_json()` | `vector_compat.rs:89` | `todo!()` |
| `resolve_host()` | `vector_compat.rs:104` | `todo!()` |
| `emit_flag_eval()` | `flag_events.rs:106` | `todo!()` |
| `emit_token_generated()` | `token_events.rs:110` | `todo!()` |
| `emit_token_consumed()` | `token_events.rs:144` | `todo!()` |
| `emit_token_expired()` | `token_events.rs:177` | `todo!()` |
| `outcome_to_log_level()` | `vector_compat.rs:121` | Fully implemented (GREEN-BY-DESIGN, stub-architect flagged) |
| `detail_to_json()` (x3) | `credential_events.rs`, `flag_events.rs`, `token_events.rs` | Fully implemented (GREEN-BY-DESIGN, pure serde delegates) |

## QueryContext Spec Gap Handling

The story spec references `QueryContext` from `prism-core`, which does not yet exist in the
workspace. The stub-architect created three interim local context types:

- `RequestingContext` in `credential_events.rs` — fields: `tool_name`, `client_id`, `trace_id`
- `FlagEvalContext` in `flag_events.rs` — fields: `tool_name`, `client_id`, `trace_id`
- `TokenEventContext` in `token_events.rs` — fields: `tool_name`, `client_id`, `sensor`

All tests construct these directly using the story-specified fields. If the implementer
later consolidates into `prism_core::QueryContext`, the call sites change but the field
names remain the same — no test logic needs to change.

## AC-to-Test Mapping

### BC-2.05.005 — Credential Access Events (AC-1)

| AC/BC Clause | Test Name | Status |
|---|---|---|
| AC-1: emit succeeds for valid Read on "crowdstrike_api_key" | `test_BC_2_05_005_credential_name_recorded_on_emit` | RED (todo!) |
| AC-1: serialized detail contains credential_name | `test_BC_2_05_005_serialized_detail_contains_credential_name` | GREEN-BY-DESIGN |
| BC-2.05.005 invariant DI-002: no value/secret/password/token fields | `test_BC_2_05_005_invariant_no_credential_value_fields_in_detail` | GREEN-BY-DESIGN |
| BC-2.05.005: requesting context fields present | `test_BC_2_05_005_requesting_context_fields_present_in_detail` | GREEN-BY-DESIGN |
| BC-2.05.005: access_type variants serialize correctly | `test_BC_2_05_005_access_type_variants_serialize_correctly` | GREEN-BY-DESIGN |
| BC-2.05.005: NotFound result emitted without panic | `test_BC_2_05_005_emit_not_found_result_succeeds` | RED (todo!) |

**BC-2.05.005 RED count: 2 / GREEN-BY-DESIGN count: 4**

### BC-2.05.007 — Vector Pipeline Compatibility (AC-2)

| AC/BC Clause | Test Name | Status |
|---|---|---|
| AC-2: @timestamp, host, service, log.level all present | `test_BC_2_05_007_vector_json_contains_required_fields` | RED (todo!) |
| AC-2: service == "prism" | `test_BC_2_05_007_service_field_is_prism` | RED (todo!) |
| AC-2: log.level == "info" for Success | `test_BC_2_05_007_log_level_info_for_success` | RED (todo!) |
| EC-005: log.level == "error" for Failure | `test_BC_2_05_007_log_level_error_for_failure` | RED (todo!) |
| BC-2.05.007: @timestamp is RFC 3339 | `test_BC_2_05_007_timestamp_is_rfc3339` | RED (todo!) |
| EC-002: host field never empty | `test_BC_2_05_007_host_field_never_empty` | RED (todo!) |
| S-2.05 arch compliance: to_vector_json read-only | `test_BC_2_05_007_to_vector_json_does_not_modify_entry` | RED (todo!) |
| BC-2.05.007: round-trip no data loss | `test_BC_2_05_007_round_trip_no_data_loss` | RED (todo!) |
| Dev Notes: parameters serialized as JSON string | `test_BC_2_05_007_parameters_serialized_as_string_not_nested_object` | RED (todo!) |
| BC-2.05.007 GREEN-BY-DESIGN: outcome_to_log_level Success | `test_BC_2_05_007_outcome_to_log_level_success_is_info` | GREEN-BY-DESIGN |
| BC-2.05.007 GREEN-BY-DESIGN: outcome_to_log_level Failure | `test_BC_2_05_007_outcome_to_log_level_failure_is_error` | GREEN-BY-DESIGN |
| EC-002: resolve_host never empty | `test_BC_2_05_007_resolve_host_never_empty` | RED (todo!) |
| EC-002: resolve_host uses PRISM_HOST_ID | `test_BC_2_05_007_resolve_host_uses_prism_host_id_env_var` | RED (todo!) |

**BC-2.05.007 RED count: 11 / GREEN-BY-DESIGN count: 2**

Note on `outcome_to_log_level` tests: The stub-architect explicitly flagged
`outcome_to_log_level()` as GREEN-BY-DESIGN because the function is implemented in the
stub (it's a trivial two-arm match). These two tests still provide traceability to AC-2
and confirm the mapping contract is met.

### BC-2.05.009 — Feature Flag Evaluation Audit Events (AC-3)

| AC/BC Clause | Test Name | Status |
|---|---|---|
| AC-3: emit_flag_eval succeeds with full resolution_trace | `test_BC_2_05_009_emit_flag_eval_records_resolution_trace` | RED (todo!) |
| AC-3: serialized FlagEvalDetail contains capability_path and trace array | `test_BC_2_05_009_serialized_flag_eval_detail_contains_capability_path_and_trace` | GREEN-BY-DESIGN |
| EC-004: empty resolution_trace does not panic | `test_BC_2_05_009_empty_resolution_trace_does_not_panic` | RED (todo!) |
| EC-004: empty trace serializes as [] | `test_BC_2_05_009_empty_resolution_trace_serializes_as_empty_array` | GREEN-BY-DESIGN |
| BC-2.05.009: FlagResolutionStep fields human-readable | `test_BC_2_05_009_resolution_step_fields_present_and_human_readable` | GREEN-BY-DESIGN |
| BC-2.05.009: canonical test vector direct path match | `test_BC_2_05_009_canonical_vector_direct_path_match_serializes` | GREEN-BY-DESIGN |

**BC-2.05.009 RED count: 2 / GREEN-BY-DESIGN count: 4**

### BC-2.05.010 — Confirmation Token Lifecycle Events (AC-4)

| AC/BC Clause | Test Name | Status |
|---|---|---|
| AC-4: emit_token_generated succeeds | `test_BC_2_05_010_emit_token_generated_succeeds` | RED (todo!) |
| AC-4: token detail contains action_summary and expiry_time | `test_BC_2_05_010_token_generated_detail_contains_action_summary_and_expiry` | GREEN-BY-DESIGN |
| BC-2.05.010: emit_token_consumed succeeds | `test_BC_2_05_010_emit_token_consumed_succeeds` | RED (todo!) |
| BC-2.05.010: TokenEvent::Consumed serializes to "consumed" | `test_BC_2_05_010_token_event_consumed_serializes_correctly` | GREEN-BY-DESIGN |
| BC-2.05.010: emit_token_expired succeeds | `test_BC_2_05_010_emit_token_expired_succeeds` | RED (todo!) |
| EC-003: Consumed and Expired event types are distinct | `test_BC_2_05_010_consumed_and_expired_event_types_are_distinct` | GREEN-BY-DESIGN |
| BC-2.05.010: all TokenEvent variants serialize correctly | `test_BC_2_05_010_all_token_event_variants_serialize_correctly` | GREEN-BY-DESIGN |
| BC-2.05.010: result_summary is "confirmation_token_issued" | `test_BC_2_05_010_token_generated_result_summary_is_confirmation_token_issued` | RED (todo!) |
| BC-2.05.010: token_id excluded from result_summary level | `test_BC_2_05_010_token_id_excluded_from_result_summary_level_detail` | GREEN-BY-DESIGN |
| BC-2.05.010: TokenEventContext carries required fields | `test_BC_2_05_010_token_event_context_carries_required_fields` | GREEN-BY-DESIGN |

**BC-2.05.010 RED count: 4 / GREEN-BY-DESIGN count: 6**

## Red Gate Verification

All 19 RED tests fail with `todo!()` panics — correct failure mode:

```
tests::specialized_event_tests::test_BC_2_05_005_credential_name_recorded_on_emit
  panicked at crates/prism-audit/src/credential_events.rs:130: not yet implemented: AC-1 / BC-2.05.005...

tests::specialized_event_tests::test_BC_2_05_005_emit_not_found_result_succeeds
  panicked at crates/prism-audit/src/credential_events.rs:130: not yet implemented...

tests::specialized_event_tests::test_BC_2_05_007_vector_json_contains_required_fields
  panicked at crates/prism-audit/src/vector_compat.rs:90: not yet implemented: AC-2 / BC-2.05.007...
  [8 more vector tests same location]

tests::specialized_event_tests::test_BC_2_05_009_emit_flag_eval_records_resolution_trace
  panicked at crates/prism-audit/src/flag_events.rs:110: not yet implemented: AC-3 / BC-2.05.009...

tests::specialized_event_tests::test_BC_2_05_009_empty_resolution_trace_does_not_panic
  panicked at crates/prism-audit/src/flag_events.rs:110: not yet implemented...

tests::specialized_event_tests::test_BC_2_05_010_emit_token_generated_succeeds
  panicked at crates/prism-audit/src/token_events.rs:116: not yet implemented: AC-4 / BC-2.05.010...

tests::specialized_event_tests::test_BC_2_05_010_emit_token_consumed_succeeds
  panicked at crates/prism-audit/src/token_events.rs:149: not yet implemented...

tests::specialized_event_tests::test_BC_2_05_010_emit_token_expired_succeeds
  panicked at crates/prism-audit/src/token_events.rs:183: not yet implemented...

tests::specialized_event_tests::test_BC_2_05_010_token_generated_result_summary_is_confirmation_token_issued
  panicked at crates/prism-audit/src/token_events.rs:116: not yet implemented...
```

No compile errors, no logic-error failures — all 19 RED tests fail for the correct reason
(unimplemented stub bodies).

## GREEN-BY-DESIGN Tests (16 total)

These tests pass because they exercise only fully-implemented pure-data structs and serde
roundtrips. They are not vacuously true — they confirm the struct shapes satisfy the spec
contracts. Each is annotated GREEN-BY-DESIGN in the test docstring.

| Test | Reason |
|------|--------|
| `test_BC_2_05_005_serialized_detail_contains_credential_name` | `detail_to_json` is fully implemented; asserts struct shape |
| `test_BC_2_05_005_invariant_no_credential_value_fields_in_detail` | Struct has no forbidden fields by definition |
| `test_BC_2_05_005_requesting_context_fields_present_in_detail` | `RequestingContext` serde roundtrip |
| `test_BC_2_05_005_access_type_variants_serialize_correctly` | `CredentialAccessType` serde |
| `test_BC_2_05_007_outcome_to_log_level_success_is_info` | `outcome_to_log_level` implemented in stub (stub-architect flagged) |
| `test_BC_2_05_007_outcome_to_log_level_failure_is_error` | Same |
| `test_BC_2_05_009_serialized_flag_eval_detail_contains_capability_path_and_trace` | `detail_to_json` for `FlagEvalDetail` is fully implemented |
| `test_BC_2_05_009_empty_resolution_trace_serializes_as_empty_array` | Pure serde |
| `test_BC_2_05_009_resolution_step_fields_present_and_human_readable` | `FlagResolutionStep` serde |
| `test_BC_2_05_009_canonical_vector_direct_path_match_serializes` | `FlagEvalDetail` struct shape |
| `test_BC_2_05_010_token_generated_detail_contains_action_summary_and_expiry` | `detail_to_json` for `TokenLifecycleDetail` is fully implemented |
| `test_BC_2_05_010_token_event_consumed_serializes_correctly` | `TokenEvent` serde |
| `test_BC_2_05_010_consumed_and_expired_event_types_are_distinct` | `TokenEvent` enum variants |
| `test_BC_2_05_010_all_token_event_variants_serialize_correctly` | `TokenEvent` serde completeness |
| `test_BC_2_05_010_token_id_excluded_from_result_summary_level_detail` | Caller contract assertion on struct fields |
| `test_BC_2_05_010_token_event_context_carries_required_fields` | `TokenEventContext` field access |

## Regression Check

| Crate / Suite | Pre-existing | New RED | New GREEN-BY-DESIGN | Total |
|---|---|---|---|---|
| prism-audit | 69 | 19 | 16 | 104 |
| All other crates | 1172 | 0 | 0 | 1172 |
| **Workspace total** | **1241** | **19** | **16** | **1276** |

All 1241 pre-existing tests continue to pass. No regressions.

## Quality Gates

| Gate | Result |
|------|--------|
| COMPILE | PASS |
| FMT (`cargo +nightly fmt --check`) | PASS |
| CLIPPY (`-D warnings`) | PASS |
| RED_RATIO (19/35 = 54.3%) | PASS (threshold: >= 50%) |
| RED failures are todo!() panics only | PASS |
| No vacuously-true tests that pass without implementation | PASS |

## Per-BC Notes

### BC-2.05.007 Vector Format (11 RED)

This BC accounts for the majority of RED tests (11) because `to_vector_json()` and
`resolve_host()` are both fully `todo!()`. The round-trip test
(`test_BC_2_05_007_round_trip_no_data_loss`) is particularly important — it verifies that
the implementer serializes `parameters` as a JSON string (per Dev Notes: "flat JSON object,
nested structs in `parameters` are serialized as a JSON string value") and that key fields
survive the JSON → string → parse cycle. The implementer should pay close attention to
this contract when implementing `to_vector_json()`.

### BC-2.05.009 Write-Only Scope

The dispatch note specifies that read-flag evals should NOT be logged. The current
`FlagEvalContext` struct has no `is_write: bool` discriminator — the caller
(`emit_flag_eval`) is always on the write path by convention (the function is only called
from the flag evaluation path in `prism-flags` for write operations). This gap is
acceptable for the Red Gate phase; the implementer should document the write-only contract
in the function body.

### BC-2.05.010 Token ID Exclusion

BC-2.05.010 states: "Token IDs are intentionally excluded from issuance audit entries."
The `token_id` field IS present in `TokenLifecycleDetail.token_id` — the exclusion applies
to the `result_summary` string and any higher-level "fast scan" fields. The implementer
must ensure `emit_token_generated()` does NOT embed `token_id` in `result_summary` when
constructing the `AuditEntry`. Test
`test_BC_2_05_010_token_id_excluded_from_result_summary_level_detail` enforces the caller
contract; the full audit entry check requires reading the persisted entry from storage
(which the emitter tests will verify once `AuditEmitter::emit()` is callable).

## Hand-Off to Implementer

Stories ready for implementation: S-2.05

Implementation guidance:
1. Implement `emit_credential_event()` in `credential_events.rs` — construct
   `CredentialAccessDetail`, embed as `parameters["credential_access_detail"]`, call
   `AuditEmitter::emit()`. The 2 RED credential tests will turn green.
2. Implement `to_vector_json()` and `resolve_host()` in `vector_compat.rs` — produce a
   flat `serde_json::Value::Object` with `@timestamp` (RFC 3339), `host` (PRISM_HOST_ID
   or gethostname fallback or "unknown-host"), `service: "prism"`, `log.level`
   ("info"/"error"), and `parameters` as a JSON **string** (not nested object). The 11 RED
   vector tests will turn green.
3. Implement `emit_flag_eval()` in `flag_events.rs` — embed `FlagEvalDetail` in
   `parameters["flag_eval_detail"]`, emit via `AuditEmitter::emit()`. Must not panic on
   empty `resolution_trace`. The 2 RED flag tests will turn green.
4. Implement `emit_token_generated()`, `emit_token_consumed()`, `emit_token_expired()` in
   `token_events.rs` — set the appropriate `result_summary` values per BC-2.05.010
   postconditions, embed `TokenLifecycleDetail` in `parameters["token_lifecycle_detail"]`.
   Do NOT embed `token_id` in `result_summary` for Generated entries. The 4 RED token
   tests will turn green.

Make each test pass, one at a time, with minimum code.
