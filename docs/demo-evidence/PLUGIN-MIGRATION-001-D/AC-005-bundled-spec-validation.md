# AC-005 — Validation of all 4 bundled specs

## Acceptance Criterion

`SpecCatalog::validate_all()` (or equivalent) validates all 4 bundled specs and produces no validation errors under nominal credential configuration.

## Primary Test

| Test name | File |
|-----------|------|
| `test_BC_2_16_009_validates_all_4_bundled_specs` | `crates/prism-spec-engine/tests/bc_2_16_009_bundled_spec_validation.rs:38` |

## Command

```
cargo nextest run -p prism-spec-engine \
  -E 'test(test_BC_2_16_009_validates_all_4_bundled_specs)' --no-fail-fast
```

## Captured Output

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.30s
────────────
 Nextest run ID 146d4a78-a8dd-4962-8c5f-ebe6fe8c1ef8 with nextest profile: default
    Starting 1 test across 32 binaries (425 tests skipped)
        PASS [   0.012s] (1/1) prism-spec-engine::bc_2_16_009_bundled_spec_validation test_BC_2_16_009_validates_all_4_bundled_specs
────────────
     Summary [   0.013s] 1 test run: 1 passed, 425 skipped
EXIT:0
```

## Verdict

| AC | Status |
|----|--------|
| AC-005 (validate all 4 bundled specs) | PASS |

## Metadata

| Field | Value |
|-------|-------|
| Captured at | 2026-05-22T08:05:20Z |
| Worktree HEAD SHA | 55b4f72daf3514599a87cd31866bc361e43fc1d6 |
| Branch | feature/PLUGIN-MIGRATION-001-D |
| Crate | prism-spec-engine |
| Exit code | 0 |
