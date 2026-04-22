# Evidence Report — S-6.08: prism-dtu-claroty

| Field | Value |
|-------|-------|
| Story ID | S-6.08 |
| Story title | prism-dtu-claroty: DTU for Claroty xDome API — L4 (adversarial) |
| Story version | v1.6 |
| Report date | 2026-04-22 |
| Branch | `feature/S-6.08-dtu-claroty` |
| Crate path | `crates/prism-dtu-claroty/` |
| Evidence type | VHS recordings (cargo test per-AC) |
| POL-010 compliance | Confirmed — all files under `docs/demo-evidence/S-6.08/` |
| VHS version | 0.10.0 |
| Font | FiraCode Nerd Font Mono |
| Theme | Dracula |

---

## Test Suite Summary

53 tests total: 0 failed, 0 skipped.

```
test result: ok. 4 passed  (ac_1_devices_list)
test result: ok. 3 passed  (ac_2_group_by)
test result: ok. 4 passed  (ac_3_tag_add_persists)
test result: ok. 3 passed  (ac_4_tag_remove)
test result: ok. 8 passed  (ac_5_auth)
test result: ok. 3 passed  (ac_6_rate_limit)
test result: ok. 3 passed  (ac_7_internal_error)
test result: ok. 4 passed  (ac_8_reset)
test result: ok. 16 passed (edge_cases)
test result: ok. 1 passed  (fidelity — FidelityValidator, all 9 routes)
```

---

## AC Coverage Matrix

| AC | Short title | Tape | GIF | WEBM | Test file | Verdict |
|----|-------------|------|-----|------|-----------|---------|
| AC-1 | Devices list — 20 objects with required fields | `AC-1-devices-list.tape` | `AC-1-devices-list.gif` | `AC-1-devices-list.webm` | `tests/ac_1_devices_list.rs` | SATISFIED |
| AC-2 | group_by returns grouped shape, not full devices | `AC-2-group-by.tape` | `AC-2-group-by.gif` | `AC-2-group-by.webm` | `tests/ac_2_group_by.rs` | SATISFIED |
| AC-3 | Tag add persists in subsequent device list | `AC-3-tag-add-persists.tape` | `AC-3-tag-add-persists.gif` | `AC-3-tag-add-persists.webm` | `tests/ac_3_tag_add_persists.rs` | SATISFIED |
| AC-4 | Tag remove absent from subsequent device list | `AC-4-tag-remove.tape` | `AC-4-tag-remove.gif` | `AC-4-tag-remove.webm` | `tests/ac_4_tag_remove.rs` | SATISFIED |
| AC-5 | Missing auth returns HTTP 401 + JSON body | `AC-5-auth.tape` | `AC-5-auth.gif` | `AC-5-auth.webm` | `tests/ac_5_auth.rs` | SATISFIED |
| AC-6 | RateLimit — 6th request returns 429 + Retry-After:30 | `AC-6-rate-limit.tape` | `AC-6-rate-limit.gif` | `AC-6-rate-limit.webm` | `tests/ac_6_rate_limit.rs` | SATISFIED |
| AC-7 | InternalError — first POST returns 500 (E-SENSOR-002) | `AC-7-internal-error.tape` | `AC-7-internal-error.gif` | `AC-7-internal-error.webm` | `tests/ac_7_internal_error.rs` | SATISFIED |
| AC-8 | reset() clears tag store; devices show empty tags | `AC-8-reset.tape` | `AC-8-reset.gif` | `AC-8-reset.webm` | `tests/ac_8_reset.rs` | SATISFIED |

---

## Edge Case Coverage Matrix

| EC | Description | Demo | Test function(s) | Verdict |
|----|-------------|------|------------------|---------|
| EC-001 | Unrecognized filter field ignored; response returned normally | `EC-all-edge-cases.tape` | `test_ec001_unrecognized_filter_field_ignored` | SATISFIED |
| EC-002 | DELETE tag never added returns 404 `{"error":"tag not found"}` | `EC-all-edge-cases.tape` | `test_ec002_delete_nonexistent_tag_returns_404`, `test_ec002_delete_nonexistent_tag_error_body`, `test_ec002_delete_tag_unknown_device_returns_404` | SATISFIED |
| EC-003 | group_by with unrecognized field returns valid JSON, no error | `EC-all-edge-cases.tape` | `test_ec003_group_by_unknown_field_no_error`, `test_ec003_group_by_unknown_field_returns_valid_json` | SATISFIED |
| EC-004 | Pagination beyond last page returns empty devices + unchanged total | `EC-all-edge-cases.tape` | `test_ec004_pagination_beyond_last_page_returns_empty`, `test_ec004_offset_beyond_fixture_returns_empty`, `test_ec004_total_unchanged_when_paging_beyond_last` | SATISFIED |
| EC-005 | 422 simulation via FailureLayer returns 422 (maps to E-SENSOR-004) | `EC-all-edge-cases.tape` | `test_ec005_422_failure_mode_returns_422` | SATISFIED |
| EC-006 | LatencyLayer configurable delay; no network I/O | `EC-all-edge-cases.tape` | `test_ec006_zero_latency_no_delay`, `test_ec006_latency_layer_delays_response` | SATISFIED |

---

## Fidelity Validator

| Demo | Description | Routes covered | Verdict |
|------|-------------|----------------|---------|
| `FIDELITY-full-suite.tape` | Full 53-test suite + FidelityValidator against all 9 routes | 7 API routes + 2 DTU control routes | ALL PASS |

The `claroty_dtu_fidelity` integration test in `tests/fidelity.rs` runs `FidelityValidator` against all 9 routes (7 API endpoints + `POST /dtu/configure` + `POST /dtu/reset`) and asserts `checks_failed == 0`.

---

## Reproducibility

All recordings are reproducible. The test suite uses pre-compiled binaries (zero incremental compilation needed after initial build). To reproduce any recording:

```bash
cd /path/to/prism/.worktrees/S-6.08-claroty
vhs docs/demo-evidence/S-6.08/<tape-file>.tape
```

To reproduce the full test run without VHS:

```bash
cargo test -p prism-dtu-claroty --features prism-dtu-claroty/dtu
```

---

## File Index

| File | Type | Purpose |
|------|------|---------|
| `AC-1-devices-list.tape` | VHS script | AC-1 recording source |
| `AC-1-devices-list.gif` | GIF recording | AC-1 visual evidence |
| `AC-1-devices-list.webm` | WebM recording | AC-1 archival |
| `AC-2-group-by.tape` | VHS script | AC-2 recording source |
| `AC-2-group-by.gif` | GIF recording | AC-2 visual evidence |
| `AC-2-group-by.webm` | WebM recording | AC-2 archival |
| `AC-3-tag-add-persists.tape` | VHS script | AC-3 recording source |
| `AC-3-tag-add-persists.gif` | GIF recording | AC-3 visual evidence |
| `AC-3-tag-add-persists.webm` | WebM recording | AC-3 archival |
| `AC-4-tag-remove.tape` | VHS script | AC-4 recording source |
| `AC-4-tag-remove.gif` | GIF recording | AC-4 visual evidence |
| `AC-4-tag-remove.webm` | WebM recording | AC-4 archival |
| `AC-5-auth.tape` | VHS script | AC-5 recording source |
| `AC-5-auth.gif` | GIF recording | AC-5 visual evidence |
| `AC-5-auth.webm` | WebM recording | AC-5 archival |
| `AC-6-rate-limit.tape` | VHS script | AC-6 recording source |
| `AC-6-rate-limit.gif` | GIF recording | AC-6 visual evidence |
| `AC-6-rate-limit.webm` | WebM recording | AC-6 archival |
| `AC-7-internal-error.tape` | VHS script | AC-7 recording source |
| `AC-7-internal-error.gif` | GIF recording | AC-7 visual evidence |
| `AC-7-internal-error.webm` | WebM recording | AC-7 archival |
| `AC-8-reset.tape` | VHS script | AC-8 recording source |
| `AC-8-reset.gif` | GIF recording | AC-8 visual evidence |
| `AC-8-reset.webm` | WebM recording | AC-8 archival |
| `EC-all-edge-cases.tape` | VHS script | EC-001..EC-006 recording source |
| `EC-all-edge-cases.gif` | GIF recording | Edge cases visual evidence |
| `EC-all-edge-cases.webm` | WebM recording | Edge cases archival |
| `FIDELITY-full-suite.tape` | VHS script | Full 53-test suite + fidelity validator |
| `FIDELITY-full-suite.gif` | GIF recording | Full suite visual evidence |
| `FIDELITY-full-suite.webm` | WebM recording | Full suite archival |
| `README.md` | This file | Evidence report + coverage matrix |
