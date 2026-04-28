# Demo Evidence Report — S-3.0.02

**Story:** prism-core: register DTU_DEFAULT_MODE registry (10-entry DtuRegistryEntry slice) per ADR-007 §2.3
**Impl commit:** e3ce4edb
**Branch:** feature/S-3.0.02
**Recorded:** 2026-04-28

---

## Coverage Map

| AC | Recording | Expected | Observed | Pass |
|----|-----------|----------|----------|------|
| AC-1 (DtuMode enum compiles) | AC-4-7-registry-tests-green | test result: ok | test result: ok. 17 passed | YES |
| AC-2 (DtuMode derives Copy/Clone/PartialEq/Eq/Debug) | AC-4-7-registry-tests-green | test result: ok | test result: ok. 17 passed | YES |
| AC-3 (DtuRegistryEntry fields) | AC-4-7-registry-tests-green | test result: ok | test result: ok. 17 passed | YES |
| AC-4 (DTU_DEFAULT_MODE len == 10) | AC-4-7-registry-tests-green | test result: ok | test result: ok. 17 passed | YES |
| AC-5 (5 MSSP Coordination = Shared, test_only=false) | AC-4-7-registry-tests-green | test result: ok | test result: ok. 17 passed | YES |
| AC-6 (4 Security Telemetry = Client, test_only=false) | AC-4-7-registry-tests-green | test result: ok | test result: ok. 17 passed | YES |
| AC-7 (demo-server = Client, test_only=true) | AC-4-7-registry-tests-green | test result: ok | test result: ok. 17 passed | YES |
| AC-8 (no per-crate exports) | AC-8-single-source-of-truth | exit_code=1 (no matches) + 2 lines in dtu.rs | exit_code=1 (no matches -- PASS); lines 4, 34 in dtu.rs | YES |

---

## Recordings

### AC-4-7-registry-tests-green

Demonstrates AC-1 through AC-7 by running the BC-3.2.005 integration test suite.
All 17 tests pass GREEN, confirming registry correctness at compile-time and runtime.

- Tape: `docs/demo-evidence/S-3.0.02/AC-4-7-registry-tests-green.tape`
- GIF: `docs/demo-evidence/S-3.0.02/AC-4-7-registry-tests-green.gif` (189K)
- WebM: `docs/demo-evidence/S-3.0.02/AC-4-7-registry-tests-green.webm` (487K)
- Command: `cargo test -p prism-core --test bc_3_2_005_dtu_registry`
- Exit code: 0
- Result: `test result: ok. 17 passed; 0 failed; 0 ignored`

Test names visible in recording:
- `test_bc_3_2_005_ac1_serde_client`
- `test_bc_3_2_005_ac1_serde_shared`
- `test_bc_3_2_005_ac1_serde_rejects_titlecase_client`
- `test_bc_3_2_005_ac1_serde_rejects_titlecase_shared`
- `test_bc_3_2_005_ac1_vp092_serde_rejects_unknown_mode`
- `test_bc_3_2_005_ac2_dtu_mode_clone`
- `test_bc_3_2_005_ac2_dtu_mode_debug`
- `test_bc_3_2_005_ac2_dtu_mode_equality`
- `test_bc_3_2_005_ac2_vp091_dtu_mode_is_copy`
- `test_bc_3_2_005_ac3_registry_entry_fields`
- `test_bc_3_2_005_ac4_registry_len_is_10`
- `test_bc_3_2_005_ac5_mssp_coordination_count_is_5`
- `test_bc_3_2_005_ac5_mssp_coordination_entries_are_shared`
- `test_bc_3_2_005_ac6_security_telemetry_count_is_4`
- `test_bc_3_2_005_ac6_security_telemetry_entries_are_client`
- `test_bc_3_2_005_ac7_demo_server_is_client_test_only`
- `test_bc_3_2_005_ac8_vp091_dtu_default_mode_not_in_dtu_crates`

---

### AC-8-single-source-of-truth

Demonstrates AC-8 by showing zero grep matches in all `prism-dtu-*` crates, then
confirming the centralized definition in `prism-core/src/dtu.rs`.

- Tape: `docs/demo-evidence/S-3.0.02/AC-8-single-source-of-truth.tape`
- GIF: `docs/demo-evidence/S-3.0.02/AC-8-single-source-of-truth.gif` (118K)
- WebM: `docs/demo-evidence/S-3.0.02/AC-8-single-source-of-truth.webm` (135K)
- Command 1: `grep -RIn 'DTU_DEFAULT_MODE|dtu_default_mode' crates/prism-dtu-*/`
- Exit code 1 (no matches): `exit_code=1 (no matches -- PASS)`
- Command 2: `grep -n 'DTU_DEFAULT_MODE' crates/prism-core/src/dtu.rs`
- Output: lines 4 (doc comment) and 34 (static definition)

---

## Summary

All 8 acceptance criteria satisfied. The registry is a single centralized static in
`prism-core/src/dtu.rs` with exactly 10 entries. No per-crate exports exist.
The 17-test integration suite covers all postconditions from BC-3.2.005.
