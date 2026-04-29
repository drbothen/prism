---
story: S-3.5.01
phase: red-gate
status: VERIFIED
timestamp: 2026-04-29
agent: test-writer
---

# Red Gate Log — S-3.5.01 Workspace src/ Convention Sweep

## Result

**ALL TESTS FAIL — Red Gate verified.**

### Rust integration tests (12 tests, all FAIL)

```
test result: FAILED. 0 passed; 12 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.09s
```

### Shell conformance tests (11 assertions fail, 13 pass vacuously on stub side-effects)

```
# RED GATE ACTIVE: 11 test(s) failing — check-crate-layout.sh stub not yet implemented.
# Total: 24  Passed: 13  Failed: 11  Skipped: 0
```

## Cargo Check

```
cargo test --workspace --all-features --no-run
Finished `test` profile [unoptimized + debuginfo] target(s)
```

Clean compile. No errors. No warnings.

## Test Inventory

### Rust Integration Tests — `crates/prism-core/tests/bc_3_7_001_check_crate_layout_test.rs`

All 12 tests fail because `scripts/check-crate-layout.sh` is a Red Gate stub that
unconditionally exits 1 with the message "NOT YET IMPLEMENTED".

| Test Name | BC Clause | VP | Failure Reason |
|-----------|-----------|-----|----------------|
| `test_bc_3_7_001_vp134_all_existing_crates_pass` | Postcondition 1 | VP-134 | Stub exits 1; expects 0 |
| `test_bc_3_7_001_vp134_workspace_crate_count` | Postcondition 1 | VP-134 | Stub exits 1 for 22-crate workspace |
| `test_bc_3_7_001_vp135_tv2_lib_rs_at_root_fails` | Postcondition 2+3 | VP-135 | Stub exits non-zero but wrong output format — crate name absent |
| `test_bc_3_7_001_vp135_tv3_tests_fixtures_triggers_violation` | Postcondition 2 | VP-135 | Stub exits non-zero but crate name absent from output |
| `test_bc_3_7_001_vp135_tv7_loose_rs_at_root_fails` | EC-007 | VP-135 | Stub exits non-zero but loose file name absent from output |
| `test_bc_3_7_001_tv4_prism_ocsf_no_tests_passes` | Postcondition 5 | — | Stub exits 1 for no-tests crate; expects 0 |
| `test_bc_3_7_001_tv5_build_rs_at_root_permitted` | Postcondition 7 | — | Stub exits 1 for build.rs crate; expects 0 |
| `test_bc_3_7_001_vp136_script_is_readonly` | Invariant 2 | VP-136 | Stub exits 1; VP-136 requires exit 0 |
| `test_bc_3_7_001_ac003_violation_output_format` | Postcondition 3 | — | No "crates/<name>: <rule>" line in stub output |
| `test_bc_3_7_001_tv1_conformant_synthetic_crate_passes` | Postcondition 1 | — | Stub exits 1 for conformant synthetic crate |
| `test_bc_3_7_001_ac006_prism_spec_engine_fixture_migration` | Postcondition 6 | — | `tests/fixtures/` still exists; `fixtures/` absent |
| `test_bc_3_7_001_invariant2_new_crate_checked` | Invariant 1 | — | Crate name absent from stub output |

### Shell Conformance Tests — `tests/crate-layout-gate/`

| Test File | Failing Assertions | Passing Assertions |
|-----------|-------------------|-------------------|
| `test_BC-3.7.001_AC-001_all-existing-crates-pass.sh` | 1 (exit code) | 2 (no violations in stdout, 22 crates exist) |
| `test_BC-3.7.001_AC-002_synthetic-bad-crate-fails.sh` | 3 (crate name absent, rule absent, format absent) | 1 (exit non-zero) |
| `test_BC-3.7.001_AC-005_prism-ocsf-no-tests-passes.sh` | 1 (synthetic fixture exit code) | 3 |
| `test_BC-3.7.001_AC-006_spec-engine-fixture-migration.sh` | 2 (old path exists, new path absent) + 1 (TV-3 message) | 1 (TV-3 exit code) |
| `test_BC-3.7.001_AC-007_build-rs-permitted.sh` | 1 (synthetic fixture exit code) | 2 |
| `test_BC-3.7.001_AC-008_script-is-readonly.sh` | 1 (exit code precondition) | 2 (git status, no temp files) |
| `test_BC-3.7.001_EC-007_loose-rs-not-buildrs.sh` | 1 (crate name absent from output) | 2 |

## AC → Test Coverage Map

| AC | Rust Test | Shell Test |
|----|-----------|------------|
| AC-001 | `test_bc_3_7_001_vp134_all_existing_crates_pass`, `test_bc_3_7_001_vp134_workspace_crate_count` | `test_BC-3.7.001_AC-001_all-existing-crates-pass.sh` |
| AC-002 | `test_bc_3_7_001_vp135_tv2_lib_rs_at_root_fails`, `test_bc_3_7_001_ac003_violation_output_format` | `test_BC-3.7.001_AC-002_synthetic-bad-crate-fails.sh` |
| AC-003 | `test_bc_3_7_001_ac003_violation_output_format` | `test_BC-3.7.001_AC-002_synthetic-bad-crate-fails.sh` |
| AC-005 | `test_bc_3_7_001_tv4_prism_ocsf_no_tests_passes` | `test_BC-3.7.001_AC-005_prism-ocsf-no-tests-passes.sh` |
| AC-006 | `test_bc_3_7_001_ac006_prism_spec_engine_fixture_migration` | `test_BC-3.7.001_AC-006_spec-engine-fixture-migration.sh` |
| AC-007 | `test_bc_3_7_001_tv5_build_rs_at_root_permitted` | `test_BC-3.7.001_AC-007_build-rs-permitted.sh` |
| AC-008 | `test_bc_3_7_001_vp136_script_is_readonly` | `test_BC-3.7.001_AC-008_script-is-readonly.sh` |

## VP → Test Coverage Map

| VP | Test |
|----|------|
| VP-134 | `test_bc_3_7_001_vp134_all_existing_crates_pass`, `test_bc_3_7_001_vp134_workspace_crate_count`, `AC-001` shell test |
| VP-135 | `test_bc_3_7_001_vp135_tv2_lib_rs_at_root_fails`, `test_bc_3_7_001_vp135_tv3_tests_fixtures_triggers_violation`, `test_bc_3_7_001_vp135_tv7_loose_rs_at_root_fails`, `AC-002` shell test |
| VP-136 | `test_bc_3_7_001_vp136_script_is_readonly`, `AC-008` shell test |

## Files Written

| File | Purpose |
|------|---------|
| `crates/prism-core/tests/bc_3_7_001_check_crate_layout_test.rs` | Rust integration tests (12 tests) |
| `crates/prism-core/Cargo.toml` | Added `tempfile = "3"` dev-dep + `[[test]]` registration |
| `tests/crate-layout-gate/run.sh` | Shell test aggregator |
| `tests/crate-layout-gate/tap_lib.sh` | TAP helper library |
| `tests/crate-layout-gate/test_BC-3.7.001_AC-001_all-existing-crates-pass.sh` | VP-134 — all 22 crates pass |
| `tests/crate-layout-gate/test_BC-3.7.001_AC-002_synthetic-bad-crate-fails.sh` | VP-135 / TV-2 — lib.rs at root fails |
| `tests/crate-layout-gate/test_BC-3.7.001_AC-005_prism-ocsf-no-tests-passes.sh` | TV-4 / EC-003 — no tests/ is conformant |
| `tests/crate-layout-gate/test_BC-3.7.001_AC-006_spec-engine-fixture-migration.sh` | AC-006 / TV-3 — fixture migration + tests/fixtures/ violation |
| `tests/crate-layout-gate/test_BC-3.7.001_AC-007_build-rs-permitted.sh` | TV-5 / EC-004 — build.rs permitted |
| `tests/crate-layout-gate/test_BC-3.7.001_AC-008_script-is-readonly.sh` | VP-136 — read-only invariant |
| `tests/crate-layout-gate/test_BC-3.7.001_EC-007_loose-rs-not-buildrs.sh` | EC-007 — loose .rs (not build.rs) fails |

## Spec Notes

1. The `WORKSPACE_ROOT` env-var protocol: shell tests pass a synthetic workspace path
   via `WORKSPACE_ROOT="${TMP_WORKSPACE}"` to isolate fixture crates from the real workspace.
   The implementer MUST honour this override in `check-crate-layout.sh` — the real script
   should default to the directory two levels up from `$0` but respect `WORKSPACE_ROOT` when
   set.  This is standard for test isolation and matches the pattern of other workspace tools.

2. `test_bc_3_7_001_vp134_workspace_crate_count` passes only when the workspace has at
   least 22 crates AND the script exits 0.  If the workspace grows (e.g., new Wave 3 crates),
   the count assertion auto-scales — the test requires `>= 22`, not exactly 22.

3. `test_bc_3_7_001_ac006_prism_spec_engine_fixture_migration` fails for two reasons:
   (a) `tests/fixtures/` still exists (migration Task 1 not yet done), and
   (b) `fixtures/` does not yet exist.  The implementer must perform the `mv` and update
   test path references before this test can pass.
