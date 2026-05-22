# AC-012 — Plugin dispatch uses spec catalog, not hardcoded sensor names

## Acceptance Criterion

The plugin dispatch path resolves sensor specs from `SpecCatalog` at runtime. No hardcoded sensor identifiers (e.g., `"crowdstrike"`, `"cyberint"`, `"claroty"`, `"armis"`) appear as match arms or string literals in the dispatch logic. New sensors added via TOML spec are discovered automatically.

## Primary Test

| Test name | File |
|-----------|------|
| `test_BC_2_16_012_001_spec_parser_no_hardcoded_sensor_dispatch` | `crates/prism-spec-engine/tests/bc_2_16_012_test.rs:146` |

## Command

```
cargo nextest run -p prism-spec-engine \
  -E 'test(test_BC_2_16_012_001_spec_parser_no_hardcoded_sensor_dispatch)' --no-fail-fast
```

## Captured Output

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.29s
────────────
 Nextest run ID a96ddcb0-172a-4acf-ad76-1b9fe1b008bf with nextest profile: default
    Starting 1 test across 32 binaries (425 tests skipped)
        PASS [   0.012s] (1/1) prism-spec-engine::bc_2_16_012_test test_BC_2_16_012_001_spec_parser_no_hardcoded_sensor_dispatch
────────────
     Summary [   0.012s] 1 test run: 1 passed, 425 skipped
EXIT:0
```

## Verdict

| AC | Status |
|----|--------|
| AC-012 (spec-catalog dispatch, no hardcoded names) | PASS |

## Metadata

| Field | Value |
|-------|-------|
| Captured at | 2026-05-22T08:05:20Z |
| Worktree HEAD SHA | 55b4f72daf3514599a87cd31866bc361e43fc1d6 |
| Branch | feature/PLUGIN-MIGRATION-001-D |
| Crate | prism-spec-engine |
| Exit code | 0 |
