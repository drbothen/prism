# AC-011 — Bundled specs declare correct auth types per ADR-028 §D2

## Acceptance Criterion

Each bundled spec's `auth` field matches the correct auth type for that sensor:
- CrowdStrike: `ClientCredentials` (OAuth2 client_id/secret flow)
- Cyberint: `ApiKey`
- Claroty: `BasicAuth`
- Armis: `ApiKey`

## Primary Test

| Test name | File |
|-----------|------|
| `test_BC_2_16_001_bundled_specs_declare_correct_auth_types` | `crates/prism-spec-engine/tests/bc_2_16_001_bundled_spec_load.rs:200` |

## Command

```
cargo nextest run -p prism-spec-engine \
  -E 'test(test_BC_2_16_001_bundled_specs_declare_correct_auth_types)' --no-fail-fast
```

## Captured Output

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.29s
────────────
 Nextest run ID 7a1769f6-3b01-4d58-9c35-59f729f7aed4 with nextest profile: default
    Starting 1 test across 32 binaries (425 tests skipped)
        PASS [   0.013s] (1/1) prism-spec-engine::bc_2_16_001_bundled_spec_load test_BC_2_16_001_bundled_specs_declare_correct_auth_types
────────────
     Summary [   0.013s] 1 test run: 1 passed, 425 skipped
EXIT:0
```

## Verdict

| AC | Status |
|----|--------|
| AC-011 (correct auth types per ADR-028 §D2) | PASS |

## Metadata

| Field | Value |
|-------|-------|
| Captured at | 2026-05-22T08:05:20Z |
| Worktree HEAD SHA | 55b4f72daf3514599a87cd31866bc361e43fc1d6 |
| Branch | feature/PLUGIN-MIGRATION-001-D |
| Crate | prism-spec-engine |
| Exit code | 0 |
