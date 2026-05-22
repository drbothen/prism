# AC-006 — Empty credential scenario is not an error

## Acceptance Criterion

When credentials for a sensor are absent or empty, `SpecCatalog::load_bundled()` does not return an error for that sensor. The spec loads successfully; the empty-credential state is surfaced via the credential layer, not the spec loader.

## Primary Test

| Test name | File |
|-----------|------|
| `test_BC_2_16_001_empty_credential_scenario_not_an_error` | `crates/prism-spec-engine/tests/bc_2_16_001_bundled_spec_load.rs:315` |

## Known Gap Reference

KG-006-001: Credential-absent runtime behavior (live sensor call with no credentials emitting E-SENSOR-401) is exercised via KG-006-001 and is deferred to the sensor integration tests in Wave 2. The spec-load path does not gate on credential presence — this is by design (ADR-028 §D2 separation of concerns).

## Command

```
cargo nextest run -p prism-spec-engine \
  -E 'test(test_BC_2_16_001_empty_credential_scenario_not_an_error)' --no-fail-fast
```

## Captured Output

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.28s
────────────
 Nextest run ID e91a3d45-df54-43c8-9721-9ad79a813c02 with nextest profile: default
    Starting 1 test across 32 binaries (425 tests skipped)
        PASS [   0.009s] (1/1) prism-spec-engine::bc_2_16_001_bundled_spec_load test_BC_2_16_001_empty_credential_scenario_not_an_error
────────────
     Summary [   0.010s] 1 test run: 1 passed, 425 skipped
EXIT:0
```

## Verdict

| AC | Status |
|----|--------|
| AC-006 (empty credential not error) | PASS |

## Metadata

| Field | Value |
|-------|-------|
| Captured at | 2026-05-22T08:05:20Z |
| Worktree HEAD SHA | 55b4f72daf3514599a87cd31866bc361e43fc1d6 |
| Branch | feature/PLUGIN-MIGRATION-001-D |
| Crate | prism-spec-engine |
| Exit code | 0 |
