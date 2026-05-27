# Demo Evidence Report — PLUGIN-MIGRATION-001-F

**Story:** PLUGIN-MIGRATION-001-F — tests: Rewrite 12 Sensor-Named Test Files to TOML Fixture Loading + Compile-Fail Perimeter `no-hardcoded-sensors`
**Date:** 2026-05-27
**Wave:** 2
**Recorded by:** demo-recorder

---

## Coverage Summary

| AC | Title | Verdict | Evidence File |
|----|-------|---------|---------------|
| AC-001 | 4 parity test files rewritten — TOML fixture loading | PASS | AC-001-004-test-execution.txt |
| AC-002 | `bc_2_16_002_crowdstrike_two_step.rs` rewritten — TOML + DTU harness | PASS | AC-001-004-test-execution.txt |
| AC-003 | `crowdstrike_oauth2_plugin_tests.rs` rewritten — plugin dispatch via TOML | PASS | AC-001-004-test-execution.txt |
| AC-004 | `crowdstrike_session_isolation.rs` — SensorId string key | PASS | AC-001-004-test-execution.txt |
| AC-005 | DTU generator tests audited — exemption comments added | PASS | AC-005-AC-008-exemption-comments.txt |
| AC-006 | `tests/external/no-hardcoded-sensors/` compile-fail crate — E0432 on all 4 deleted symbols | PASS | AC-006-compile-fail-perimeter.txt |
| AC-007 | `ci.yml` updated — `no-hardcoded-sensors-compile-fail` job added | PASS | AC-007-ci-job.txt |
| AC-008 | DTU harness clone files audited — no stale imports, exemption comments added | PASS | AC-005-AC-008-exemption-comments.txt |

**Supplemental:** residual-sensor-name-scan.txt — verifies all remaining sensor-name matches are in comments, not live code.

---

## AC-001 through AC-004: Test Execution (7 red-gate tests, all PASS)

Crate: `prism-spec-engine`

```
Starting 6 tests across 34 binaries (475 tests skipped)
    PASS [   0.022s] prism-spec-engine::parity_claroty test_PLUGIN_MIGRATION_001_F_parity_claroty_toml_fixture_loading
    PASS [   0.023s] prism-spec-engine::parity_armis test_PLUGIN_MIGRATION_001_F_parity_armis_toml_fixture_loading
    PASS [   0.022s] prism-spec-engine::parity_cyberint test_PLUGIN_MIGRATION_001_F_parity_cyberint_toml_fixture_loading
    PASS [   0.024s] prism-spec-engine::parity_crowdstrike test_PLUGIN_MIGRATION_001_F_parity_crowdstrike_toml_fixture_loading
    PASS [   0.028s] prism-spec-engine::crowdstrike_oauth2_plugin_tests test_PLUGIN_MIGRATION_001_F_crowdstrike_oauth2_plugin_dispatch_via_toml
    PASS [   0.144s] prism-spec-engine::bc_2_16_002_crowdstrike_two_step test_PLUGIN_MIGRATION_001_F_bc_2_16_002_crowdstrike_two_step_toml_driven
Summary [   0.145s] 6 tests run: 6 passed, 475 skipped
```

Crate: `prism-query`

```
Starting 1 test across 12 binaries (911 tests skipped)
    PASS [   0.025s] prism-query::crowdstrike_session_isolation test_PLUGIN_MIGRATION_001_F_crowdstrike_session_isolation_sensor_id_key
Summary [   0.025s] 1 test run: 1 passed, 911 skipped
```

---

## AC-006: Compile-Fail Perimeter (4 E0432 errors confirmed)

```
cargo check --manifest-path tests/external/no-hardcoded-sensors/Cargo.toml --color=never
Exit code: 101
```

All 4 deleted symbols produce E0432 errors:
- `prism_sensors::auth::armis::ArmisAuth` — E0432: could not find `armis` in `auth`
- `prism_sensors::auth::claroty::ClarotyAuth` — E0432: could not find `claroty` in `auth`
- `prism_sensors::auth::crowdstrike::CrowdStrikeAuth` — E0432: could not find `crowdstrike` in `auth`
- `prism_sensors::auth::cyberint::CyberintAuth` — E0432: could not find `cyberint` in `auth`

---

## AC-005 + AC-008: Exemption Comments (8 files, all present)

```
rg 'ADR-023.*DTU-EXEMPT' crates/prism-dtu-*/tests/ crates/prism-dtu-harness/src/clones/
```

DTU generator tests (AC-005): 4/4 files carry exemption comments
DTU harness clones (AC-008): 4/4 files carry ADR-023 §DTU-EXEMPT header comments

---

## AC-007: CI Job Definition

Job `no-hardcoded-sensors-compile-fail` present in `.github/workflows/ci.yml`:
- Uses `--manifest-path tests/external/no-hardcoded-sensors/Cargo.toml`
- Uses `--color=never` (required for pattern matching on error codes)
- Contains per-symbol positive-coverage assertions for all 4 deleted auth symbols
- Contains self-referential guard to detect accidental job removal

---

## Residual Sensor-Name Scan

All 9 files matched by the grep contain only comment-line references:
- `prism-sensors/src/` files — comments documenting the deletion invariant
- Rewritten test files — comments explaining the absence of old patterns
- `crowdstrike-oauth2` plugin — legitimate sensor-named plugin code (DTU-adjacent)

No live code references to deleted sensor-named symbols remain outside DTU clone crates.

---

## Overall Verdict: ALL ACs PASS

The PLUGIN-MIGRATION-001-F implementation is evidence-complete:
- 7/7 red-gate tests pass (AC-001 through AC-004)
- compile-fail perimeter produces E0432 on all 4 deleted symbols (AC-006)
- all 8 exemption comments present (AC-005 + AC-008)
- CI job definition present with correct structure (AC-007)
- residual scan clean (no live sensor-name references outside DTU clones)
