# Demo Evidence Report — S-3.3.06

**Story:** reload_config DTU mode-change rejection
**Impl SHA:** 735d56dd
**Recorded:** 2026-04-29
**Tool:** VHS 0.10.0

## Coverage Summary

| AC | Description | Success Path | Error Path | Status |
|----|-------------|-------------|------------|--------|
| AC-001 | All 15 mode-change tests GREEN | AC-001-all-15-tests-green.{gif,webm} | N/A — full suite green | COVERED |
| AC-002 | Mode change detected + warned | AC-002-mode-change-detection.{gif,webm} | N/A — single targeted test | COVERED |

## Recordings

### AC-001 — All 15 mode-change tests GREEN

**Command:** `cargo test -p prism-spec-engine --test bc_3_2_005_reload_mode_rejection 2>&1 | tail -25`

**Result:** `test result: ok. 15 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s`

- [AC-001-all-15-tests-green.gif](AC-001-all-15-tests-green.gif)
- [AC-001-all-15-tests-green.webm](AC-001-all-15-tests-green.webm)
- [AC-001-all-15-tests-green.tape](AC-001-all-15-tests-green.tape)

Tests covered:
- `test_BC_3_2_005_dtu_only_in_old_snapshot_not_compared` — ok
- `test_BC_3_2_005_dtu_only_in_new_snapshot_not_compared` — ok
- `test_BC_3_2_005_mode_change_correct_org_slug_and_dtu_type` — ok
- `test_BC_3_2_005_mode_change_detected_and_returned` — ok
- `test_BC_3_2_005_mode_change_shared_to_client_detected` — ok
- `test_BC_3_2_005_invariant_mode_change_count_matches_changed_dtus` — ok
- `test_BC_3_2_005_mode_change_warns_old_mode_preserved` — ok
- `test_BC_3_2_005_multi_dtu_only_changed_ones_appear` — ok
- `test_BC_3_2_005_no_change_produces_empty_warnings` — ok
- `test_BC_3_2_005_multi_dtu_all_changed_all_appear` — ok
- `test_BC_3_2_005_reload_dry_run_mode_change_no_side_effects` — ok
- `test_BC_3_2_005_reload_integration_mode_change_in_result` — ok
- `test_BC_3_2_005_reload_integration_no_mode_change_no_warning` — ok
- `test_BC_3_2_005_tv_01_reload_claroty_client_to_shared_warned` — ok
- `test_BC_3_2_005_tv_02_reload_slack_shared_to_client_warned` — ok

### AC-002 — Mode change detected + warned

**Command:** `cargo test -p prism-spec-engine --test bc_3_2_005_reload_mode_rejection test_BC_3_2_005_mode_change_detected_and_returned -- --nocapture 2>&1`

**Result:** `test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 14 filtered out; finished in 0.00s`

- [AC-002-mode-change-detection.gif](AC-002-mode-change-detection.gif)
- [AC-002-mode-change-detection.webm](AC-002-mode-change-detection.webm)
- [AC-002-mode-change-detection.tape](AC-002-mode-change-detection.tape)

## Notes

- Error paths for these ACs are the test cases themselves: the suite contains dedicated tests for every rejection invariant (mode-change detection, DTU-only snapshots not compared, dry-run isolation, cross-org multi-DTU scenarios). All 15 pass.
- VHS `Wait+Line` was replaced with `Sleep 30s` due to piped-command output buffering in the VHS PTY environment; the tests complete in under 1s so the recording captures the full output within the sleep window.
