---
document_type: red-gate-log
level: ops
version: "1.0"
status: draft
producer: test-writer
timestamp: 2026-04-25T00:00:00Z
phase: 3
inputs:
  - .factory/stories/S-2.07-per-sensor-auth.md
  - .factory/specs/behavioral-contracts/BC-2.01.004-offset-based-pagination-claroty.md
  - .factory/specs/behavioral-contracts/BC-2.01.005-crowdstrike-oauth2-two-step-fetch.md
  - .factory/specs/behavioral-contracts/BC-2.01.006-cyberint-cookie-auth.md
  - .factory/specs/behavioral-contracts/BC-2.01.007-claroty-bearer-polymorphic-ids.md
  - .factory/specs/behavioral-contracts/BC-2.01.008-armis-bearer-aql.md
input-hash: "21efb02"
traces_to: "S-2.07"
stub_architect_agent: "a4193c76"
stub_compile_verified: true
test_writer_agent: "claude-sonnet-4-6"
red_gate_verified: true
---

# Red Gate Log: S-2.07 — prism-sensors: Per-Sensor Auth and Pagination

## Summary

| Story | Tests Written | All Fail (Red)? | Gate |
|-------|--------------|-----------------|------|
| S-2.07 | 56 total (47 RED + 9 GREEN-BY-DESIGN) | YES — 47/47 RED tests fail with todo!() panics | PASS |

RED_RATIO: 47/56 = 0.839 (target >= 0.50 — PASS)

## Stubs Exercised by Tests

### auth/crowdstrike.rs
- `fn CrowdStrikeAdapter::new(_auth) -> Self` — todo!() panics in 7 tests
- `async fn CrowdStrikeAdapter::fetch(...)` — todo!() panics via new()
- `const CROWDSTRIKE_BATCH_SIZE: usize = 100` — GREEN (constant)

### auth/cyberint.rs
- `fn CyberintAdapter::new(_auth) -> Self` — todo!() panics in 5 tests
- `async fn CyberintAdapter::fetch(...)` — todo!() panics via new()

### auth/claroty.rs
- `fn ClarotyAdapter::new(_, _bearer_token) -> Self` — todo!() panics in 4 tests
- `async fn ClarotyAdapter::fetch(...)` — todo!() panics via new()
- `fn ClarotyId::deserialize(...)` — todo!() panics in 5 tests
- `ClarotyId::Display` — GREEN (implemented)
- `ClarotyId::Int` / `ClarotyId::Uuid` construction — GREEN (enum variants)

### auth/armis.rs
- `fn ArmisAdapter::new(_, _bearer_token) -> Self` — todo!() panics in 8 tests
- `async fn ArmisAdapter::fetch(...)` — todo!() panics via new()
- `const DEFAULT_AQL_TEMPLATE: &str` — GREEN (constant)

### pagination.rs
- `fn OffsetCursor::advance(...)` — todo!() panics in 5 tests
- `fn paginate_claroty(...)` — todo!() panics in 3 tests
- `fn OffsetCursor::new(...)` — GREEN (pure constructor)
- `fn OffsetCursor::is_exhausted(...)` — GREEN (pure predicate)

### timestamp.rs
- `fn parse_timestamp(_s)` — todo!() panics in 11 tests

### lib.rs
- `fn init_registry(...)` — todo!() panics in 1 test

## Stubs Created

### S-2.07: prism-sensors Per-Sensor Auth and Pagination

- `fn CrowdStrikeAdapter::new(_auth: &CrowdStrikeAuth) -> Self` — todo!(); builds base_url from cloud_region, constructs Client, initializes token_cache
- `async fn CrowdStrikeAdapter::acquire_token(&self, _auth: &CrowdStrikeAuth) -> Result<String, SensorError>` — todo!(); POST /oauth2/token → parse access_token + expires_in
- `async fn CrowdStrikeAdapter::query_resource_ids(...)` — todo!(); GET /queries/{resource_type} → Vec<String> IDs
- `async fn CrowdStrikeAdapter::fetch_entities(...)` — todo!(); chunked POST /entities/{resource_type}/GET at CROWDSTRIKE_BATCH_SIZE
- `async fn CrowdStrikeAdapter::fetch(...)` — todo!(); full two-step orchestration
- `fn CyberintAdapter::new(_auth: &CyberintAuth) -> Self` — todo!(); derive base_url, build Client with cookie_store(true)
- `async fn CyberintAdapter::login(&self, _auth: &CyberintAuth)` — todo!(); POST /login → Set-Cookie captured by reqwest
- `async fn CyberintAdapter::get_page(...)` — todo!(); GET with session cookie; 401 → re-login + retry once
- `async fn CyberintAdapter::fetch(...)` — todo!(); full cookie-auth orchestration
- `fn ClarotyAdapter::new(_auth: &ClarotyAuth, _bearer_token: String) -> Self` — todo!(); builds Client, stores bearer_token
- `async fn ClarotyAdapter::post_read(...)` — todo!(); POST with Authorization: Bearer header
- `async fn ClarotyAdapter::fetch(...)` — todo!(); dispatches to paginate_claroty() for audit_logs, post_read() otherwise
- `fn ClarotyId::deserialize<D>(...)` — todo!(); serde Visitor: visit_i64 → Int, visit_str → Uuid parse
- `fn ArmisAdapter::new(_auth: &ArmisAuth, _bearer_token: String) -> Self` — todo!(); builds Client, stores bearer_token
- `fn ArmisAdapter::build_aql(&self, _spec, _params) -> String` — todo!(); extract aql_query from sensor_config or substitute DEFAULT_AQL_TEMPLATE
- `fn ArmisAdapter::resolve_timestamp(&self, _record, _spec) -> DateTime<Utc>` — todo!(); firstSeen → lastSeen → Utc::now() + warn!
- `async fn ArmisAdapter::get_search(...)` — todo!(); GET /api/v1/search?aql=... with bearer header
- `async fn ArmisAdapter::fetch(...)` — todo!(); AQL-forwarding orchestration
- `fn OffsetCursor::advance(&mut self, _total_count, _page_timestamp)` — todo!(); increment offset by page_size, update total_count, assert no regression
- `fn paginate_claroty(_endpoint, _page_size, _client)` — todo!(); unfold-based Stream issuing GET ?offset=N&limit=page_size, parses total_count, halts on is_exhausted()
- `fn parse_timestamp(_s: &str) -> Result<DateTime<Utc>, SensorError>` — todo!(); RFC-3339 → Unix epoch i64 → custom "%Y-%m-%dT%H:%M:%S" fallback chain
- `fn try_rfc3339(_s: &str) -> Option<DateTime<Utc>>` — todo!(); parse_from_rfc3339 attempt
- `fn try_unix_epoch(_s: &str) -> Option<DateTime<Utc>>` — todo!(); parse as i64 then DateTime::from_timestamp
- `fn try_custom_format(_s: &str) -> Option<DateTime<Utc>>` — todo!(); NaiveDateTime::parse_from_str with "%Y-%m-%dT%H:%M:%S"
- `fn init_registry(_cs, _cy, _cl, _cl_tok, _ar, _ar_tok) -> AdapterRegistry` — todo!(); construct all 4 adapters, register each, return registry

---

## Red Gate Verification

### BC-2.01.005 — CrowdStrike OAuth2 + Two-Step Fetch

| Test | AC/TV | Status |
|------|-------|--------|
| test_BC_2_01_005_crowdstrike_batch_size_is_100 | const check | GREEN-BY-DESIGN |
| test_BC_2_01_005_oauth2_token_called_once_and_cached | AC-1, TV-001 | FAIL (todo! CrowdStrikeAdapter::new) |
| test_BC_2_01_005_cached_token_reused_on_second_fetch | AC-1 | FAIL (todo! CrowdStrikeAdapter::new) |
| test_BC_2_01_005_query_returns_zero_ids_yields_empty_result | TV-002 | FAIL (todo! CrowdStrikeAdapter::new) |
| test_BC_2_01_005_rejects_oauth2_401_with_authentication_error | TV-003 | FAIL (todo! CrowdStrikeAdapter::new) |
| test_BC_2_01_005_token_refresh_on_post_entities_401 | AC-2, TV-004 | FAIL (todo! CrowdStrikeAdapter::new) |
| test_BC_2_01_005_150_ids_batch_into_two_post_entities_calls | TV-005 adapted | FAIL (todo! CrowdStrikeAdapter::new) |

### BC-2.01.006 — Cyberint Cookie Auth + Timestamp

| Test | AC/TV | Status |
|------|-------|--------|
| test_BC_2_01_006_login_sets_cookie_used_for_data_request | TV-001 | FAIL (todo! CyberintAdapter::new) |
| test_BC_2_01_006_login_called_once_cookie_reused_on_second_fetch | TV-001 | FAIL (todo! CyberintAdapter::new) |
| test_BC_2_01_006_401_triggers_relogin_and_retry | cookie refresh | FAIL (todo! CyberintAdapter::new) |
| test_BC_2_01_006_rejects_login_401_with_authentication_error | TV-004 | FAIL (todo! CyberintAdapter::new) |
| test_BC_2_01_006_unix_epoch_timestamp_in_response_parsed_without_error | AC-3, EC-002 | FAIL (todo! CyberintAdapter::new) |

### BC-2.01.006 timestamp unit tests

| Test | AC/TV | Status |
|------|-------|--------|
| test_BC_2_01_006_parse_rfc3339_returns_correct_datetime | TV-001 | FAIL (todo! parse_timestamp) |
| test_BC_2_01_006_parse_rfc3339_with_offset_returns_utc | post-condition | FAIL (todo! parse_timestamp) |
| test_BC_2_01_006_parse_unix_epoch_1710500000_returns_correct_datetime | AC-3, EC-002 | FAIL (todo! parse_timestamp) |
| test_BC_2_01_006_parse_unix_epoch_zero_returns_epoch_origin | boundary | FAIL (todo! parse_timestamp) |
| test_BC_2_01_006_parse_negative_unix_epoch_parses_correctly | boundary | FAIL (todo! parse_timestamp) |
| test_BC_2_01_006_parse_custom_no_tz_format_returns_utc_datetime | TV-002 | FAIL (todo! parse_timestamp) |
| test_BC_2_01_006_rejects_unparseable_timestamp_with_error | EC-003, TV-003 | FAIL (todo! parse_timestamp) |
| test_BC_2_01_006_rejects_empty_string_with_unparseable_timestamp | EC-003 | FAIL (todo! parse_timestamp) |
| test_BC_2_01_006_rejects_date_only_string_with_unparseable_timestamp | EC-003 | FAIL (todo! parse_timestamp) |
| test_BC_2_01_006_rejects_float_string_with_unparseable_timestamp | EC-003 | FAIL (todo! parse_timestamp) |
| test_BC_2_01_006_rfc3339_takes_priority_over_unix_epoch | format order | FAIL (todo! parse_timestamp) |

### BC-2.01.007 — Claroty Bearer + Polymorphic IDs

| Test | AC/TV | Status |
|------|-------|--------|
| test_BC_2_01_007_claroty_id_int_display_formats_as_decimal_string | AC-4 | GREEN-BY-DESIGN |
| test_BC_2_01_007_claroty_id_uuid_display_formats_as_hyphenated_string | BC post | GREEN-BY-DESIGN |
| test_BC_2_01_007_claroty_id_int_equality | invariant | GREEN-BY-DESIGN |
| test_BC_2_01_007_deserialize_json_integer_as_claroty_id_int | AC-4, EC-004 | FAIL (todo! ClarotyId::deserialize) |
| test_BC_2_01_007_deserialize_uuid_string_as_claroty_id_uuid | TV-002 | FAIL (todo! ClarotyId::deserialize) |
| test_BC_2_01_007_integer_and_numeric_string_normalize_to_same_display | DEC-010 | FAIL (todo! ClarotyId::deserialize) |
| test_BC_2_01_007_rejects_non_uuid_non_integer_string | error | FAIL (todo! ClarotyId::deserialize) |
| test_BC_2_01_007_claroty_id_roundtrip_in_struct | round-trip | FAIL (todo! ClarotyId::deserialize) |
| test_BC_2_01_007_bearer_token_included_in_requests | post-condition | FAIL (todo! ClarotyAdapter::new) |
| test_BC_2_01_007_rejects_401_with_authentication_error | TV-003 | FAIL (todo! ClarotyAdapter::new) |
| test_BC_2_01_007_integer_ids_in_response_normalized_to_claroty_id | TV-001 | FAIL (todo! ClarotyAdapter::new) |
| test_BC_2_01_004_claroty_adapter_paginates_audit_logs_3_pages | AC-5 adapted | FAIL (todo! ClarotyAdapter::new) |

### BC-2.01.004 — Offset Pagination

| Test | AC/TV | Status |
|------|-------|--------|
| test_BC_2_01_004_offset_cursor_new_starts_at_zero | pre-condition | GREEN-BY-DESIGN |
| test_BC_2_01_004_offset_cursor_is_not_exhausted_when_fresh | post-condition | GREEN-BY-DESIGN |
| test_BC_2_01_004_offset_cursor_is_exhausted_when_offset_equals_total | post-condition | GREEN-BY-DESIGN |
| test_BC_2_01_004_offset_cursor_is_exhausted_when_offset_exceeds_total | DI-001 boundary | GREEN-BY-DESIGN |
| test_BC_2_01_004_offset_cursor_advance_increments_offset_by_page_size | BC post | FAIL (todo! advance) |
| test_BC_2_01_004_offset_cursor_advance_updates_total_count | BC post | FAIL (todo! advance) |
| test_BC_2_01_004_invariant_cursor_offset_never_regresses | DI-001 | FAIL (todo! advance) |
| test_BC_2_01_004_cursor_exhausted_after_5_advances_for_500_total | AC-5 | FAIL (todo! advance) |
| test_BC_2_01_004_offset_cursor_advance_stores_page_timestamp | BC post | FAIL (todo! advance) |
| test_BC_2_01_004_paginate_claroty_five_pages_for_500_total | AC-5, TV-001 | FAIL (todo! paginate_claroty) |
| test_BC_2_01_004_paginate_claroty_halts_when_offset_equals_total | TV-002, EC-01-005 | FAIL (todo! paginate_claroty) |
| test_BC_2_01_004_paginate_claroty_http_400_yields_sensor_error | TV-003 | FAIL (todo! paginate_claroty) |

### BC-2.01.008 — Armis AQL + Timestamp Fallback

| Test | AC/TV | Status |
|------|-------|--------|
| test_BC_2_01_008_default_aql_template_format | const check | GREEN-BY-DESIGN |
| test_BC_2_01_008_valid_api_key_returns_records_with_primary_timestamp | TV-001 | FAIL (todo! ArmisAdapter::new) |
| test_BC_2_01_008_first_seen_null_uses_last_seen_as_fallback | AC-6, TV-002 | FAIL (todo! ArmisAdapter::new) |
| test_BC_2_01_008_both_timestamps_null_uses_utc_now_without_error | AC-6, EC-005, TV-003 | FAIL (todo! ArmisAdapter::new) |
| test_BC_2_01_008_rejects_401_api_key_with_authentication_error | TV-004 | FAIL (todo! ArmisAdapter::new) |
| test_BC_2_01_008_rejects_400_aql_error_with_api_contract_error | TV-005 | FAIL (todo! ArmisAdapter::new) |
| test_BC_2_01_008_aql_query_forwarded_verbatim_without_modification | arch compliance | FAIL (todo! ArmisAdapter::new) |
| test_BC_2_01_008_absent_aql_query_uses_default_template_with_table | BC post | FAIL (todo! ArmisAdapter::new) |
| test_BC_2_01_008_init_registry_registers_armis_adapter | S-2.07 Task 5 | FAIL (todo! init_registry) |

## Regression Check

| Test Scope | Status |
|-----------|--------|
| 1276 pre-existing workspace tests (baseline) | All pass (0 failures in non-prism-sensors targets) |
| 51 pre-existing prism-sensors unit tests (S-2.06) | All pass |
| New integration tests (S-2.07, 56 tests) | 9 GREEN-BY-DESIGN pass; 47 RED fail as expected |
| Workspace total after S-2.07 tests added | 1294 PASS / 94 FAIL* |

*The 94 FAIL count includes duplicate runs from both `integration` binary and 6 individual `test_*.rs` binaries. True unique failing tests: 47 (confirmed via `cargo test --test integration`).

## Failure Modes

All 47 RED failures use the `todo!()` panic pattern with descriptive messages pointing to the BC and AC that needs implementation. None fail due to compile errors, type errors, or logic bugs in the test themselves.

Example failure message:
```
thread '...' panicked at crates/prism-sensors/src/timestamp.rs:43:5:
not yet implemented: AC-3 / BC-2.01.006: implement 3-format fallback chain: ...
```

## Dev Dependencies Added

- `wiremock = "0.6"` — HTTP mock server for integration tests
- `futures = "0.3"` — StreamExt for paginate_claroty stream consumption in tests

## Source Modification

- `crates/prism-sensors/src/auth/armis.rs`: `DEFAULT_AQL_TEMPLATE` visibility changed from `pub(crate)` to `pub` to allow integration test access

## Hand-Off to Implementer

Stories ready for implementation: **S-2.07**

Implementation guidance:
- Implement stubs in this order: `timestamp.rs` (parse_timestamp) → `pagination.rs` (advance, paginate_claroty) → `auth/crowdstrike.rs` → `auth/cyberint.rs` → `auth/claroty.rs` (ClarotyId::deserialize + adapter) → `auth/armis.rs` → `lib.rs` (init_registry)
- `timestamp.rs` is the shared dependency — implement it first since Cyberint and Armis adapters both call it
- Token cache for CrowdStrike uses `Arc<RwLock<Option<CachedToken>>>` — the `RwLock` is tokio's, not std
- `paginate_claroty()` must return `impl Stream` using `futures::stream::unfold` — do NOT collect into Vec
- `ClarotyId::deserialize` uses a custom `serde::de::Visitor` — see BC-2.01.007 §Deserialize impl docstring
- `AQL forwarding` MUST be verbatim — wiremock `query_param` matcher will catch any modification
- All test files have `#[allow(clippy::expect_used, clippy::unwrap_used)]` since test code is exempt from those lints
