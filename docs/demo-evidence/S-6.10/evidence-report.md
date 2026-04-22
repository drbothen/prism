# Evidence Report — S-6.10: prism-dtu-armis

| Field | Value |
|-------|-------|
| Story ID | S-6.10 |
| Story title | prism-dtu-armis: DTU for Armis Centrix API — L2 (stateful) |
| Story version | v1.6 |
| Report date | 2026-04-22 |
| Branch | `feature/S-6.10-dtu-armis` |
| Crate path | `crates/prism-dtu-armis/` |
| Evidence type | Artifact-based (library crate — no CLI or UI) |
| POL-010 compliance | Confirmed — all files under `docs/demo-evidence/S-6.10/` |

---

## Commit History (implementation)

| SHA | Description |
|-----|-------------|
| `74b15cf` | test(S-6.10): Red Gate step 1 - compile-only stubs for prism-dtu-armis (ADR-002) |
| `e453d23` | test(S-6.10): Red Gate step 2 - failing tests for all ACs + edge cases |
| `3bbcd8b` | fix(S-6.10): AC-1 capture AQL from POST body |
| `0da9243` | fix(S-6.10): AC-6 mount FailureLayer with RateLimit |
| `0ef6696` | feat(S-6.10): EC-006 MalformedResponse failure mode |

---

## AC Coverage Matrix

| AC | Short title | Evidence file | Test functions | Verdict |
|----|-------------|---------------|----------------|---------|
| AC-1 | AQL capture — GET and POST device query logs AQL verbatim | `AC-1-aql-capture.md` | `ac_1_get_devices_with_aql_returns_200_and_logs_aql`, `ac_1_post_devices_with_aql_body_returns_200_and_logs_aql`, `ac_1_devices_response_contains_pagination_fields`, `ec_001_aql_special_characters_stored_verbatim`, `ec_004_pagination_beyond_last_page_returns_empty_array` | SATISFIED |
| AC-2 | Timestamp fallback — d-001 has null last_seen, non-null first_seen | `AC-2-timestamp-fallback.md` | `ac_2_device_d001_has_null_last_seen_and_non_null_first_seen`, `ac_2_device_d002_has_both_timestamps_populated`, `ac_2_device_risk_endpoint_returns_risk_score`, `ec_002_risk_endpoint_returns_404_for_unknown_device` | SATISFIED |
| AC-3 | Stateful tag add — POST tag persists in subsequent device query | `AC-3-stateful-tag-add.md` | `ac_3_post_tag_returns_201_with_device_id_and_tag_key`, `ac_3_added_tag_appears_in_subsequent_device_query`, `ac_3_tag_endpoint_requires_bearer_auth_returns_403` | SATISFIED |
| AC-4 | Tag delete — DELETE removes tag; absent from subsequent query | `AC-4-tag-delete.md` | `ac_4_delete_tag_returns_200_removed`, `ac_4_device_does_not_have_tag_after_delete`, `ac_4_delete_tag_endpoint_requires_bearer_auth`, `ec_003_delete_nonexistent_tag_returns_404` | SATISFIED |
| AC-5 | Missing bearer token returns HTTP 403 (not 401) | `AC-5-missing-bearer-403.md` | `ac_5_get_devices_without_auth_returns_403`, `ac_5_get_alerts_without_auth_returns_403`, `ac_5_get_device_activity_without_auth_returns_403`, `ac_5_get_device_risk_without_auth_returns_403`, `ac_5_empty_bearer_value_returns_403`, `ac_5_wrong_scheme_returns_403`, `ac_5_dtu_internal_endpoints_do_not_require_auth` | SATISFIED |
| AC-6 | Rate limit 429 + EC-006 MalformedResponse | `AC-6-rate-limit-and-malformed-response.md` | `ac_6_rate_limit_429_after_threshold_exceeded_via_configure`, `ac_6_rate_limit_allows_requests_before_threshold`, `ec_006_malformed_response_mode_returns_non_parseable_body` | SATISFIED |
| AC-7 | Reset clears tag store and AQL log; fixture data survives | `AC-7-reset-behavior.md` | `ac_story_7_reset_clears_tag_store_and_aql_log`, `ac_story_7_reset_does_not_remove_fixture_data` + shape tests | SATISFIED |

---

## Edge Case Coverage Matrix

| EC | Description | Test function | Verdict |
|----|-------------|---------------|---------|
| EC-001 | AQL with special chars stored verbatim | `ec_001_aql_special_characters_stored_verbatim` | SATISFIED |
| EC-002 | Risk endpoint 404 for unknown device | `ec_002_risk_endpoint_returns_404_for_unknown_device` | SATISFIED |
| EC-003 | DELETE non-existent tag → 404 | `ec_003_delete_nonexistent_tag_returns_404` | SATISFIED |
| EC-004 | Pagination beyond last → empty array, correct total | `ec_004_pagination_beyond_last_page_returns_empty_array` | SATISFIED |
| EC-005 | Both GET and POST accepted for /api/v1/devices | `ac_1_post_devices_with_aql_body_returns_200_and_logs_aql` | SATISFIED |
| EC-006 | MalformedResponse mode → non-parseable body | `ec_006_malformed_response_mode_returns_non_parseable_body` | SATISFIED |

---

## Green Gate Verification

All checks run in worktree `/Users/jmagady/dev/prism/.worktrees/S-6.10-armis/` at commit `0ef6696`.

### cargo test

```
$ cargo test --features prism-dtu-armis/dtu -p prism-dtu-armis 2>&1

    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.56s

ac_1_aql_capture_and_device_list  — 5 tests: all ok
ac_2_timestamp_fallback_fixture   — 4 tests: all ok
ac_3_stateful_tag_add             — 3 tests: all ok
ac_4_tag_delete                   — 4 tests: all ok
ac_5_missing_bearer_403           — 7 tests: all ok
ac_6_rate_limit_429               — 3 tests: all ok
ac_7_fidelity_validator           — 1 test:  ok
reset_state_invariants            — 5 tests: all ok

Result: 32 integration tests — ALL PASSED
```

Full transcript: `test-run.txt`

### cargo clippy

```
$ cargo clippy --features prism-dtu-armis/dtu -p prism-dtu-armis -- -D warnings
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.04s
Exit 0 — CLEAN
```

---

## Non-Demo-able ACs

None. All 7 ACs and all 6 ECs are covered by reproducible integration tests.

---

## Recording format note

`prism-dtu-armis` is a library crate with no CLI binary. VHS recordings are not applicable.
Evidence is in the form of artifact-based test transcripts per the established pattern
used by `prism-dtu-common` (S-6.06), `prism-dtu-claroty` (S-6.08), and other DTU crates
in this wave. Each AC file contains: the acceptance criterion verbatim, the covering test
functions with file references, the exact `cargo test` command to reproduce, the captured
output, and HTTP sequence diagrams for both success and error paths.
