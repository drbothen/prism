# Demo Evidence Report — S-3.1.01

## Story

| Field | Value |
|-------|-------|
| Story ID | S-3.1.01 |
| Title | prism-core: declare OrgId(Uuid v7) newtype via uuid_v7_newtype! macro |
| Branch | feature/S-3.1.01 |
| Implementation Commit | a6b6e958 |
| Date Recorded | 2026-04-29 |
| Behavioral Contract | BC-3.1.001 (OrgRegistry Bijective Slug/UUID Resolution) |

## Test Summary

| Metric | Value |
|--------|-------|
| Test suite | `tests/bc_3_1_001_org_id.rs` |
| Total tests | 11 |
| Passed | 11 |
| Failed | 0 |
| Result | `test result: ok. 11 passed; 0 failed; 0 ignored` |

## Recordings

### AC-001 — All 11 OrgId Tests GREEN

Traces to: BC-3.1.001 (preconditions 1–3, invariants 1 & 3, EC-001 through EC-003)

| Artifact | Path |
|----------|------|
| VHS tape | `docs/demo-evidence/S-3.1.01/AC-001-all-11-tests-green.tape` |
| GIF | `docs/demo-evidence/S-3.1.01/AC-001-all-11-tests-green.gif` |
| WebM | `docs/demo-evidence/S-3.1.01/AC-001-all-11-tests-green.webm` |

Demonstrates: `cargo test --test bc_3_1_001_org_id` reporting `test result: ok. 11 passed`
in the S-3.1.01 feature worktree. Covers all ACs (AC-1 through AC-4) and edge cases
(EC-001 through EC-003) defined in S-3.1.01.

### AC-002 — Display Format (AC-4 hyphenated lowercase UUID)

Traces to: BC-3.1.001 invariant 3 — `OrgId::from_uuid_v7(known_uuid).to_string()` equals
hyphenated lowercase UUID string.

| Artifact | Path |
|----------|------|
| VHS tape | `docs/demo-evidence/S-3.1.01/AC-002-display-format.tape` |
| GIF | `docs/demo-evidence/S-3.1.01/AC-002-display-format.gif` |
| WebM | `docs/demo-evidence/S-3.1.01/AC-002-display-format.webm` |

Demonstrates: `cargo test --test bc_3_1_001_org_id test_bc_3_1_001_ac_4_display_hyphenated_lowercase -- --nocapture`
running and reporting `test result: ok. 1 passed`. Confirms `impl std::fmt::Display for OrgId`
produces the bare hyphenated lowercase UUID string (AC-4 / BC-3.1.001 invariant 3).

## AC Coverage Map

| Acceptance Criterion | Tests Covered | Demo |
|---------------------|---------------|------|
| AC-1: OrgId::new() generates v7; from_uuid_v7() panics on non-v7 | test_bc_3_1_001_ac_1_new_generates_v7_uuid, test_bc_3_1_001_ac_1_from_uuid_panics_on_v4 | AC-001 |
| AC-2: Re-exported from prism_core | test_bc_3_1_001_ac_2_re_export_compiles | AC-001 |
| AC-3: Debug/Clone/Copy/PartialEq/Eq/Hash/Serialize/Deserialize derived | test_bc_3_1_001_ac_3_hashmap_key_compiles, test_bc_3_1_001_ac_3_derives_equality, test_bc_3_1_001_ac_3_derives_clone_copy, test_bc_3_1_001_ac_3_serde_round_trip_json | AC-001 |
| AC-4: Display produces hyphenated lowercase UUID | test_bc_3_1_001_ac_4_display_hyphenated_lowercase | AC-001, AC-002 |
| EC-001: from_uuid_v7(v4) panics | test_bc_3_1_001_ec_001_from_uuid_v4_panics | AC-001 |
| EC-002: Two new() calls both valid v7 | test_bc_3_1_001_ec_002_two_new_both_valid_v7 | AC-001 |
| EC-003: HashMap key works | test_bc_3_1_001_ec_003_hashmap_key_stores_values | AC-001 |

## Toolchain

| Tool | Version |
|------|---------|
| VHS | 0.10.0 |
| Font | FiraCode Nerd Font Mono |
| Theme | Dracula |
| Shell | bash |
