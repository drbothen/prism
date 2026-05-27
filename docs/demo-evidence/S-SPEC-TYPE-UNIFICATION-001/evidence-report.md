# Demo Evidence — S-SPEC-TYPE-UNIFICATION-001

**Story:** Retire `types::SensorSpec` — Unify on `spec_parser::SensorSpec` as Canonical
**Recorded:** 2026-05-27
**Branch:** feature/S-SPEC-TYPE-UNIFICATION-001
**Method:** Terminal command capture (manual evidence per VSDD demo-recorder protocol)

---

## AC-001: Zero remaining `types::SensorSpec` usages in code

**Command:**
```
rg "types::SensorSpec" crates/ --type rust
```

**Output (all matches are comments/doc strings — zero live code usages):**
```
crates/prism-bin/src/boot.rs:    // types::SensorSpec (auth_type: String) and spec_parser::SensorSpec (AuthType enum)
crates/prism-bin/src/boot.rs:    // to prism-spec-engine::types::SensorSpec and spec_parser::SensorSpec). The TOML
crates/prism-spec-engine/src/spec_parser.rs:/// `ConfigSnapshot::sensor_specs` stores this type directly; `types::SensorSpec` is retired.
crates/prism-spec-engine/src/types.rs:    /// sensor spec type. `types::SensorSpec` is retired; this field now carries the richer
crates/prism-spec-engine/tests/hot_reload_tests.rs:/// ADR-030 Approach D: constructs `spec_parser::SensorSpec` directly (no `types::SensorSpec`).
crates/prism-spec-engine/tests/hot_reload_tests.rs://   AC-001: Zero remaining types::SensorSpec usages (compile-time + grep verification)
crates/prism-spec-engine/tests/hot_reload_tests.rs:/// of the double-parse that existed while types::SensorSpec and spec_parser::SensorSpec
```

**Result:** PASS — All 7 matches are in `//` comments or `///` doc comments. Zero struct usage, zero import usage, zero field type usage. The `types::SensorSpec` struct has been deleted from `types.rs`; compilation succeeds, confirming no live references remain.

---

## AC-002: `build_type_spec_map_for_overlay` deleted

**Command:**
```
rg "build_type_spec_map_for_overlay" crates/ --type rust
```

**Output (all matches are comments — zero live function definition or call sites):**
```
crates/prism-bin/src/boot.rs:    // This eliminates the build_type_spec_map_for_overlay double-parse that existed while
crates/prism-bin/src/boot.rs:        // This replaces the hard-abort that was in build_type_spec_map_for_overlay — the
crates/prism-bin/src/boot.rs:    /// Verifies the pass-1 fix that changed `build_type_spec_map_for_overlay` from
crates/prism-spec-engine/tests/hot_reload_tests.rs://   AC-002: build_type_spec_map_for_overlay deleted (structural verification)
crates/prism-spec-engine/tests/hot_reload_tests.rs:/// Verifies the single-parse contract: after retiring build_type_spec_map_for_overlay,
crates/prism-spec-engine/tests/hot_reload_tests.rs:         (AC-003: single-parse contract; build_type_spec_map_for_overlay deleted)"
```

**Result:** PASS — All 6 matches are in `//` or `///` comments. The function definition (formerly lines 890-938 in boot.rs post-S-CONFIG merge) is deleted. No call sites remain. Compilation confirms deletion.

---

## AC-003, AC-004, AC-006: New AC test execution

**Command:**
```
cargo nextest run -p prism-spec-engine -E 'test(S_SPEC_TYPE_UNIFICATION_001)'
```

**Output:**
```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.29s
────────────
 Nextest run ID 0b35d825-b77b-4259-9521-9cc692d63967 with nextest profile: default
    Starting 3 tests across 34 binaries (481 tests skipped)
        PASS [   0.010s] (1/3) prism-spec-engine::hot_reload_tests test_S_SPEC_TYPE_UNIFICATION_001_006_list_sensor_specs_response_unchanged
        PASS [   0.010s] (2/3) prism-spec-engine::hot_reload_tests test_S_SPEC_TYPE_UNIFICATION_001_004_auth_type_is_enum_not_string
        PASS [   0.010s] (3/3) prism-spec-engine::hot_reload_tests test_S_SPEC_TYPE_UNIFICATION_001_003_spec_loader_parse_called_n_not_2n_times
────────────
     Summary [   0.011s] 3 tests run: 3 passed, 481 skipped
```

**Result:** PASS — All 3 new acceptance criterion tests pass:
- `test_S_SPEC_TYPE_UNIFICATION_001_003`: verifies `SpecLoader::parse` is called N times (not 2N) during boot with N sensor specs (AC-003)
- `test_S_SPEC_TYPE_UNIFICATION_001_004`: verifies `ConfigSnapshot::sensor_specs[id].auth_type` returns structured `AuthType` enum variant (AC-004)
- `test_S_SPEC_TYPE_UNIFICATION_001_006`: verifies `list_sensor_specs` MCP response is unchanged after unification (AC-006)

---

## AC-005: Non-exhaustive gate EXPECTED=35

**Evidence from `.github/workflows/ci.yml`:**
```yaml
EXPECTED=35
if [ "${TOTAL_COUNT}" -lt "${EXPECTED}" ]; then
  echo "::error::Expected at least ${EXPECTED} E0639/E0004 errors..."
```

**Gate comment confirms types list (excerpt):**
> `...S-SPEC-TYPE-UNIFICATION-001). Note: types::SensorSpec removed (ADR-030 Approach D).`

**Result:** PASS — EXPECTED decremented from 36 to 35. The ci.yml comment explicitly names `types::SensorSpec` as removed and cites ADR-030 Approach D. The compile-fail gate at `tests/external/non_exhaustive_violation/` will enforce this count in CI.

---

## Full Suite Baseline: prism-spec-engine

**Command:**
```
cargo nextest run -p prism-spec-engine --no-fail-fast
```

**Result:**
```
Summary [6.313s] 474 tests run: 474 passed, 10 skipped
```

**Result:** PASS — 474/474 tests pass. 0 regressions. 10 skipped (all `#[ignore]`'d DTU/external-service integration tests — pre-existing, not introduced by this story).

---

## AC-007: Adversarial Convergence Summary

| Pass | Findings | CLEAN (strict) | CLEAN (PR-merge) |
|------|----------|----------------|------------------|
| 1 | 4 (MED-001, LOW-001, LOW-002, OBS-001) | no | no |
| 2 | 0 | yes | yes |
| 3 | 0 | yes | yes |
| 4 | 0 | yes | yes |

**3-CLEAN streak:** passes 2/3/4 — CONVERGED per BC-5.39.001

Trajectory: 4 → 0 → 0 → 0
