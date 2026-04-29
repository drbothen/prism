# Demo Evidence Report — S-3.2.02

## Story

| Field | Value |
|-------|-------|
| Story ID | S-3.2.02 |
| Title | prism-dtu-armis: Multi-tenant state segregation — (OrgId, String) re-keying |
| Branch | feature/S-3.2.02 |
| Implementation Commit | 5218996a |
| Date Recorded | 2026-04-29 |
| Behavioral Contract | BC-3.2.001 (Per-Org Sensor Data Isolation via Composite HashMap Key) |

## Test Summary

| Metric | Value |
|--------|-------|
| Test suite | `crates/prism-dtu-armis/tests/multi_tenant.rs` |
| Total tests | 11 |
| Passed | 11 |
| Failed | 0 |
| Proptest suites | 3 (prop_cross_org_tag_isolation, prop_write_does_not_affect_other_org, prop_reset_for_selectivity) |
| Cases per proptest | 1000 |
| Result | `test result: ok. 11 passed; 0 failed; 0 ignored` |

## Recordings

### AC-001 — All 11 Multi-Tenant Tests GREEN

Traces to: BC-3.2.001 (postconditions 1–3, invariant 1, edge cases EC-001 through EC-004)

| Artifact | Path |
|----------|------|
| VHS tape | `docs/demo-evidence/S-3.2.02/AC-001-all-11-tests-green.tape` |
| GIF | `docs/demo-evidence/S-3.2.02/AC-001-all-11-tests-green.gif` |
| WebM | `docs/demo-evidence/S-3.2.02/AC-001-all-11-tests-green.webm` |

Demonstrates: `cargo test -p prism-dtu-armis --features dtu --test multi_tenant` reporting
`test result: ok. 11 passed` in the S-3.2.02 feature worktree. Covers all ACs (AC-001
through AC-006) defined in the story.

### AC-002 — Proptest Invariants (3 proptests x 1000 cases)

Traces to: BC-3.2.001 VP-079 (OrgId-flipping mutation killed) — AC-006

| Artifact | Path |
|----------|------|
| VHS tape | `docs/demo-evidence/S-3.2.02/AC-002-proptest-invariants.tape` |
| GIF | `docs/demo-evidence/S-3.2.02/AC-002-proptest-invariants.gif` |
| WebM | `docs/demo-evidence/S-3.2.02/AC-002-proptest-invariants.webm` |

Demonstrates: `cargo test -p prism-dtu-armis --features dtu --test multi_tenant prop_ -- --nocapture`
running all three proptests and reporting `test result: ok. 3 passed`. Confirms that
for any pair of distinct `OrgId` values and any `device_id` string, reads under the
wrong org always return empty — killing the OrgId-flipping mutation class
(TD-DTU-MUTATE-COVERAGE-001).

## AC Coverage Map

| Acceptance Criterion | Tests Covered | Demo |
|---------------------|---------------|------|
| AC-001: Cross-org tag lookup returns empty | test_BC_3_2_001_cross_org_lookup_returns_empty, prop_cross_org_tag_isolation | AC-001, AC-002 |
| AC-002: Write isolation — org_A write invisible to org_B | test_BC_3_2_001_write_isolation_org_a_does_not_affect_org_b, prop_write_does_not_affect_other_org | AC-001, AC-002 |
| AC-003: Independent state per org for same device_id | test_BC_3_2_001_independent_state_per_org_same_device_id | AC-001 |
| AC-004: Fixture registries remain bare-String keyed | test_BC_3_2_001_fixture_registries_bare_string_keyed | AC-001 |
| AC-005: reset_for is selective — org_B intact after org_A reset | test_BC_3_2_001_reset_for_is_selective, prop_reset_for_selectivity | AC-001, AC-002 |
| AC-006: OrgId-flipping mutation killed (proptest, 1000 cases) | prop_cross_org_tag_isolation, prop_write_does_not_affect_other_org, prop_reset_for_selectivity | AC-002 |

## Toolchain

| Tool | Version |
|------|---------|
| VHS | 0.10.0 |
| Font | FiraCode Nerd Font Mono |
| Theme | Dracula |
| Shell | bash |
