# Demo Evidence Report — S-2.07 Per-Sensor Auth and Pagination

**Story:** S-2.07 — prism-sensors: Per-Sensor Auth and Pagination
**Branch:** feature/S-2.07-per-sensor-auth
**Recorded:** 2026-04-26
**Tool:** VHS 0.10.0

---

## TDD Health Note

S-2.07 followed proper Red Gate discipline: 28 `todo!()` in production drove **47 RED tests**; 9 GREEN-BY-DESIGN tests covered constants and pure-data Display impls. RED ratio 83.9% well above 50% threshold (Layer 2 Red Gate density check satisfied — second story since prevention layers conceptualized). 7 BC implementations across 7 micro-commits. BC-2.01.005 batch size discrepancy resolved as non-conflict (1000 = API ceiling, 100 = conservative default per story Dev Note). 5 minor test bug fixes (wiremock mock ordering + timestamp epoch values) documented as corrections, not implementation shortcuts.

---

## Coverage Map

| AC | Story Criterion | Recording | BC | Test Binary | Tests Shown |
|----|----------------|-----------|-----|-------------|-------------|
| AC-1 | CrowdStrike OAuth2 token acquisition, cache, two-step fetch | [ac-1-crowdstrike-oauth2.gif](ac-1-crowdstrike-oauth2.gif) | BC-2.01.005 | test_crowdstrike | 7/7 PASS |
| AC-2 | CrowdStrike 401 retry re-authenticates with new token | [ac-1-crowdstrike-oauth2.gif](ac-1-crowdstrike-oauth2.gif) | BC-2.01.005 | test_crowdstrike | included in AC-1 recording |
| AC-3 | Cyberint Unix epoch `"1710500000"` parses to correct DateTime (cookie auth) | [ac-2-cyberint-cookie.gif](ac-2-cyberint-cookie.gif) | BC-2.01.006 | test_cyberint | 5/5 PASS |
| AC-3 (timestamp) | Multi-format timestamp parsing: RFC 3339, Unix epoch, custom no-TZ | [ac-6-timestamp-multi-format.gif](ac-6-timestamp-multi-format.gif) | BC-2.01.006 | test_timestamp | 11/11 PASS |
| AC-4 | Claroty `"id": 12345` (integer) deserializes as `ClarotyId::Int(12345)` | [ac-3-claroty-bearer-polymorphic-ids.gif](ac-3-claroty-bearer-polymorphic-ids.gif) | BC-2.01.007 | test_claroty | 12/12 PASS |
| AC-5 | Claroty pagination: total_count=500 / page_size=100 → exactly 5 HTTP requests | [ac-5-pagination-offset-cursor.gif](ac-5-pagination-offset-cursor.gif) | BC-2.01.004 | test_pagination | 12/12 PASS |
| AC-6 | Armis firstSeen=null → lastSeen fallback; both null → DateTime::now() + warn! | [ac-4-armis-aql-forwarding.gif](ac-4-armis-aql-forwarding.gif) | BC-2.01.008 | test_armis | 9/9 PASS |

---

## Recording Details

### AC-1 + AC-2: CrowdStrike OAuth2 (BC-2.01.005)

**File:** [ac-1-crowdstrike-oauth2.gif](ac-1-crowdstrike-oauth2.gif) | [ac-1-crowdstrike-oauth2.tape](ac-1-crowdstrike-oauth2.tape)

Demonstrates:
- OAuth2 token called once, cached for reuse on second fetch
- 401 response triggers re-authentication and retry with new token
- 150 IDs batched into two POST /entities calls (batch size = 100)
- Zero-ID query yields empty result without error
- Overall: 7/7 tests PASS

```
test test_BC_2_01_005_crowdstrike_batch_size_is_100 ... ok
test test_BC_2_01_005_rejects_oauth2_401_with_authentication_error ... ok
test test_BC_2_01_005_query_returns_zero_ids_yields_empty_result ... ok
test test_BC_2_01_005_cached_token_reused_on_second_fetch ... ok
test test_BC_2_01_005_oauth2_token_called_once_and_cached ... ok
test test_BC_2_01_005_token_refresh_on_post_entities_401 ... ok
test test_BC_2_01_005_150_ids_batch_into_two_post_entities_calls ... ok
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured
```

---

### AC-3 (Cyberint): Cookie Auth + Unix Epoch Timestamp (BC-2.01.006)

**File:** [ac-2-cyberint-cookie.gif](ac-2-cyberint-cookie.gif) | [ac-2-cyberint-cookie.tape](ac-2-cyberint-cookie.tape)

Demonstrates:
- POST /login sets cookie, reused on subsequent data requests
- Unix epoch `"1710500000"` parsed to correct DateTime<Utc>
- 401 triggers re-login and retry once
- Overall: 5/5 tests PASS

```
test test_BC_2_01_006_rejects_login_401_with_authentication_error ... ok
test test_BC_2_01_006_login_sets_cookie_used_for_data_request ... ok
test test_BC_2_01_006_unix_epoch_timestamp_in_response_parsed_without_error ... ok
test test_BC_2_01_006_login_called_once_cookie_reused_on_second_fetch ... ok
test test_BC_2_01_006_401_triggers_relogin_and_retry ... ok
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured
```

---

### AC-4: Claroty Bearer Token + Polymorphic IDs (BC-2.01.007)

**File:** [ac-3-claroty-bearer-polymorphic-ids.gif](ac-3-claroty-bearer-polymorphic-ids.gif) | [ac-3-claroty-bearer-polymorphic-ids.tape](ac-3-claroty-bearer-polymorphic-ids.tape)

Demonstrates:
- Integer ID `12345` deserializes as `ClarotyId::Int(12345)` without error
- UUID string deserializes as `ClarotyId::Uuid(...)` without error
- Non-UUID non-integer strings rejected with error
- Bearer token included in all requests; 401 → AuthenticationError
- Claroty 3-page audit log pagination (BC-2.01.004 via test_claroty)
- Overall: 12/12 tests PASS

```
test test_BC_2_01_007_claroty_id_int_equality ... ok
test test_BC_2_01_007_deserialize_json_integer_as_claroty_id_int ... ok
test test_BC_2_01_007_claroty_id_int_display_formats_as_decimal_string ... ok
test test_BC_2_01_007_claroty_id_uuid_display_formats_as_hyphenated_string ... ok
test test_BC_2_01_007_integer_and_numeric_string_normalize_to_same_display ... ok
test test_BC_2_01_007_deserialize_uuid_string_as_claroty_id_uuid ... ok
test test_BC_2_01_007_rejects_non_uuid_non_integer_string ... ok
test test_BC_2_01_007_claroty_id_roundtrip_in_struct ... ok
test test_BC_2_01_007_integer_ids_in_response_normalized_to_claroty_id ... ok
test test_BC_2_01_007_rejects_401_with_authentication_error ... ok
test test_BC_2_01_007_bearer_token_included_in_requests ... ok
test test_BC_2_01_004_claroty_adapter_paginates_audit_logs_3_pages ... ok
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured
```

---

### AC-6: Armis AQL Forwarding + Timestamp Fallback Chain (BC-2.01.008)

**File:** [ac-4-armis-aql-forwarding.gif](ac-4-armis-aql-forwarding.gif) | [ac-4-armis-aql-forwarding.tape](ac-4-armis-aql-forwarding.tape)

Demonstrates:
- AQL query forwarded verbatim without modification
- Absent AQL uses default template with table name
- `firstSeen: null` falls back to `lastSeen`
- Both timestamps null → `DateTime::now()` used without error
- 401 → AuthenticationError; 400 → ApiContractError
- init_registry registers Armis adapter
- Overall: 9/9 tests PASS

```
test test_BC_2_01_008_default_aql_template_format ... ok
test test_BC_2_01_008_init_registry_registers_armis_adapter ... ok
test test_BC_2_01_008_both_timestamps_null_uses_utc_now_without_error ... ok
test test_BC_2_01_008_valid_api_key_returns_records_with_primary_timestamp ... ok
test test_BC_2_01_008_aql_query_forwarded_verbatim_without_modification ... ok
test test_BC_2_01_008_absent_aql_query_uses_default_template_with_table ... ok
test test_BC_2_01_008_rejects_400_aql_error_with_api_contract_error ... ok
test test_BC_2_01_008_first_seen_null_uses_last_seen_as_fallback ... ok
test test_BC_2_01_008_rejects_401_api_key_with_authentication_error ... ok
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured
```

---

### AC-5: Offset-Based Hybrid Pagination — OffsetCursor (BC-2.01.004)

**File:** [ac-5-pagination-offset-cursor.gif](ac-5-pagination-offset-cursor.gif) | [ac-5-pagination-offset-cursor.tape](ac-5-pagination-offset-cursor.tape)

Demonstrates:
- `OffsetCursor::new()` starts at offset 0
- `advance()` increments offset by page_size, stores timestamp
- Cursor exhausted when offset >= total_count
- `paginate_claroty()` makes exactly 5 HTTP requests for total_count=500 / page_size=100
- Stream halts when offset equals total_count
- HTTP 400 yields SensorError
- Invariant: offset never regresses
- Overall: 12/12 tests PASS

```
test test_BC_2_01_004_cursor_exhausted_after_5_advances_for_500_total ... ok
test test_BC_2_01_004_invariant_cursor_offset_never_regresses ... ok
test test_BC_2_01_004_offset_cursor_advance_increments_offset_by_page_size ... ok
test test_BC_2_01_004_offset_cursor_advance_updates_total_count ... ok
test test_BC_2_01_004_offset_cursor_is_exhausted_when_offset_equals_total ... ok
test test_BC_2_01_004_offset_cursor_advance_stores_page_timestamp ... ok
test test_BC_2_01_004_offset_cursor_is_exhausted_when_offset_exceeds_total ... ok
test test_BC_2_01_004_offset_cursor_is_not_exhausted_when_fresh ... ok
test test_BC_2_01_004_offset_cursor_new_starts_at_zero ... ok
test test_BC_2_01_004_paginate_claroty_halts_when_offset_equals_total ... ok
test test_BC_2_01_004_paginate_claroty_http_400_yields_sensor_error ... ok
test test_BC_2_01_004_paginate_claroty_five_pages_for_500_total ... ok
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured
```

---

### AC-3 (Timestamp): Multi-Format Timestamp Parsing (BC-2.01.006)

**File:** [ac-6-timestamp-multi-format.gif](ac-6-timestamp-multi-format.gif) | [ac-6-timestamp-multi-format.tape](ac-6-timestamp-multi-format.tape)

Demonstrates:
- RFC 3339 with offset parsed and normalized to UTC
- Unix epoch `1710500000` parses to correct DateTime<Utc>
- Negative Unix epoch parses correctly
- Unix epoch `0` returns epoch origin
- Custom no-TZ format `"%Y-%m-%dT%H:%M:%S"` returns UTC DateTime
- RFC 3339 takes priority over Unix epoch interpretation
- Unparseable strings return `Err(SensorError::UnparseableTimestamp)` — no panic
- Date-only string, empty string, float string all rejected
- Overall: 11/11 tests PASS

```
test test_BC_2_01_006_parse_custom_no_tz_format_returns_utc_datetime ... ok
test test_BC_2_01_006_parse_negative_unix_epoch_parses_correctly ... ok
test test_BC_2_01_006_parse_unix_epoch_1710500000_returns_correct_datetime ... ok
test test_BC_2_01_006_parse_rfc3339_returns_correct_datetime ... ok
test test_BC_2_01_006_rejects_date_only_string_with_unparseable_timestamp ... ok
test test_BC_2_01_006_parse_unix_epoch_zero_returns_epoch_origin ... ok
test test_BC_2_01_006_parse_rfc3339_with_offset_returns_utc ... ok
test test_BC_2_01_006_rejects_empty_string_with_unparseable_timestamp ... ok
test test_BC_2_01_006_rejects_float_string_with_unparseable_timestamp ... ok
test test_BC_2_01_006_rejects_unparseable_timestamp_with_error ... ok
test test_BC_2_01_006_rfc3339_takes_priority_over_unix_epoch ... ok
test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured
```

---

## Summary

| Metric | Value |
|--------|-------|
| ACs covered | 6 of 6 |
| Recordings produced | 6 GIF + 6 tape |
| Total tests demonstrated | 56 (47 RED→green + 9 GREEN-by-design) |
| Test binaries | 6 (test_crowdstrike, test_cyberint, test_claroty, test_armis, test_pagination, test_timestamp) |
| RED ratio | 83.9% (47/56) — above 50% threshold |
| VHS version | 0.10.0 |
| Total GIF size | ~812 KB |
| Error paths covered | Yes — 401/400 error paths included in test_crowdstrike, test_cyberint, test_claroty, test_armis; UnparseableTimestamp in test_timestamp |
