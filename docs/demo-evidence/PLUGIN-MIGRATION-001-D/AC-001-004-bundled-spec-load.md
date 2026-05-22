# AC-001..004 — Load all 4 bundled specs at boot / per-sensor table counts

## Acceptance Criteria

- **AC-001:** `SpecCatalog::load_bundled()` loads the CrowdStrike TOML spec.
- **AC-002:** `SpecCatalog::load_bundled()` loads the Cyberint TOML spec.
- **AC-003:** `SpecCatalog::load_bundled()` loads the Claroty TOML spec.
- **AC-004:** `SpecCatalog::load_bundled()` loads the Armis TOML spec.
- **AC-002 (table count):** CrowdStrike spec exposes the correct number of tables (3: detections, devices, alerts).
- Canonical table namespaces across all 4 sensors verified via sibling test.

## Primary Test

| Test name | File |
|-----------|------|
| `test_BC_2_16_001_loads_4_bundled_specs_at_boot` | `crates/prism-spec-engine/tests/bc_2_16_001_bundled_spec_load.rs:64` |
| `test_BC_2_16_001_bundled_specs_produce_canonical_table_namespaces` | `crates/prism-spec-engine/tests/bc_2_16_001_bundled_spec_load.rs:158` |
| `test_BC_2_16_009_crowdstrike_spec_has_3_tables` | `crates/prism-spec-engine/tests/bc_2_16_009_bundled_spec_validation.rs:180` |

## Command

```
cargo nextest run -p prism-spec-engine \
  -E 'test(test_BC_2_16_001_loads_4_bundled_specs_at_boot)' --no-fail-fast
```

## Captured Output

```
   Compiling prism-core v0.2.0 (crates/prism-core)
   Compiling prism-dtu-common v0.1.0 (crates/prism-dtu-common)
   Compiling prism-spec-engine v0.9.0 (crates/prism-spec-engine)
   Compiling prism-dtu-cyberint v0.1.0 (crates/prism-dtu-cyberint)
   Compiling prism-dtu-armis v0.1.0 (crates/prism-dtu-armis)
   Compiling prism-dtu-crowdstrike v0.1.0 (crates/prism-dtu-crowdstrike)
   Compiling prism-dtu-claroty v0.1.0 (crates/prism-dtu-claroty)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 29.73s
────────────
 Nextest run ID 910b37b7-9baa-45fb-867a-459da8ee9a81 with nextest profile: default
    Starting 1 test across 32 binaries (425 tests skipped)
        PASS [   0.014s] (1/1) prism-spec-engine::bc_2_16_001_bundled_spec_load test_BC_2_16_001_loads_4_bundled_specs_at_boot
────────────
     Summary [   0.017s] 1 test run: 1 passed, 425 skipped
EXIT:0
```

## Sibling test — canonical table namespaces (all 4 sensors)

```
cargo nextest run -p prism-spec-engine \
  -E 'test(test_BC_2_16_001_bundled_specs_produce_canonical_table_namespaces)' --no-fail-fast
```

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.30s
────────────
 Nextest run ID f331d38f-100a-433c-bd6d-6f2b20e82565 with nextest profile: default
    Starting 1 test across 32 binaries (425 tests skipped)
        PASS [   0.012s] (1/1) prism-spec-engine::bc_2_16_001_bundled_spec_load test_BC_2_16_001_bundled_specs_produce_canonical_table_namespaces
────────────
     Summary [   0.013s] 1 test run: 1 passed, 425 skipped
EXIT:0
```

## Per-sensor table count — CrowdStrike 3 tables

```
cargo nextest run -p prism-spec-engine \
  -E 'test(test_BC_2_16_009_crowdstrike_spec_has_3_tables)' --no-fail-fast
```

```
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.29s
────────────
 Nextest run ID 2f69e97f-7c63-4ae9-89eb-a5242549f977 with nextest profile: default
    Starting 1 test across 32 binaries (425 tests skipped)
        PASS [   0.010s] (1/1) prism-spec-engine::bc_2_16_009_bundled_spec_validation test_BC_2_16_009_crowdstrike_spec_has_3_tables
────────────
     Summary [   0.010s] 1 test run: 1 passed, 425 skipped
EXIT:0
```

## Verdict

| AC | Status |
|----|--------|
| AC-001 (crowdstrike load) | PASS |
| AC-002 (cyberint load) | PASS |
| AC-003 (claroty load) | PASS |
| AC-004 (armis load) | PASS |
| AC-001..004 canonical namespaces | PASS |
| AC-002 crowdstrike 3 tables | PASS |

## Metadata

| Field | Value |
|-------|-------|
| Captured at | 2026-05-22T08:05:20Z |
| Worktree HEAD SHA | 55b4f72daf3514599a87cd31866bc361e43fc1d6 |
| Branch | feature/PLUGIN-MIGRATION-001-D |
| Crate | prism-spec-engine |
| Exit code | 0 |
