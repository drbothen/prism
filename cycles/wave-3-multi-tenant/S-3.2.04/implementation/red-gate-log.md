# Red Gate Log — S-3.2.04 prism-dtu-cyberint Multi-tenant State Segregation

**Step:** 2 (failing tests written)
**Date:** 2026-04-29
**Author:** test-writer agent
**Command:** `cargo test -p prism-dtu-cyberint --features dtu --test multi_tenant`
**Worktree:** `.worktrees/S-3.2.04` @ branch `feature/S-3.2.04`
**Prior commit:** `5e84fb0c` (stub: stores re-keyed, reset_for + extract_org_id stubs)

## Summary

| Metric | Count |
|--------|-------|
| New test file | `tests/multi_tenant.rs` |
| Tests added (multi_tenant suite) | 15 |
| Passed (already-implemented isolation methods) | 9 |
| Failed (unimplemented stubs) | 6 |
| Build errors | 0 |
| `--no-run` result | CLEAN (warnings only, no errors) |

Red Gate status: **PARTIAL — 6 of 15 new tests FAIL for the correct reason.**

The 9 passing tests confirm that the core isolation invariants (`register_session`,
`is_valid_session`, `alert_store` composite key construction via `build_alert_store`)
are already correctly implemented in the stub phase. These tests document BC coverage
for implemented behavior.

The 6 failing tests gate on unimplemented stubs and constitute the Red Gate proper.

## Failure Summary

| Test | Failure Cause | Root Stub |
|------|--------------|-----------|
| `test_BC_3_2_001_reset_for_removes_org_a_alert_entries_preserves_org_b` | `todo!()` at `state.rs:248` | `reset_for` |
| `test_BC_3_2_003_reset_for_removes_org_a_session_tokens_preserves_org_b` | `todo!()` at `state.rs:248` | `reset_for` |
| `test_BC_3_2_001_reset_for_clears_both_stores_atomically_for_org_a` | `todo!()` at `state.rs:248` | `reset_for` |
| `test_BC_3_2_001_invariant_reset_for_selectivity` (proptest) | `todo!()` at `state.rs:248` | `reset_for` |
| `test_BC_3_2_003_http_session_token_registered_for_org_a_rejected_by_org_b` | server panic → connection error | `extract_org_id` |
| `test_BC_3_2_001_http_reset_for_invalidates_org_a_preserves_org_b` | server panic → connection error | `extract_org_id` |

All failures are for the right reason (stub panics), not test code errors.

## Test Coverage Map

| Test | AC / BC Covered | Status |
|------|----------------|--------|
| `test_BC_3_2_001_alert_cross_org_isolation_write_a_read_b_returns_none` | AC-001 / BC-3.2.001 post-1 / TV-3.2.001-02 | PASS |
| `test_BC_3_2_001_alert_independent_per_org_state_same_key` | AC-001 / BC-3.2.001 post-3 / TV-3.2.001-03 | PASS |
| `test_BC_3_2_003_session_cross_org_isolation_register_a_validate_b_returns_false` | AC-002 / BC-3.2.003 post-2 / TV-3.2.003-02 | PASS |
| `test_BC_3_2_003_identical_token_string_independent_per_org_contexts` | AC-003 / BC-3.2.003 EC-001 / TV-3.2.003-04 | PASS |
| `test_BC_3_2_003_token_refresh_preserves_org_binding` | AC-004 / BC-3.2.003 post-3 / TV-3.2.003-03 | PASS |
| `test_BC_3_2_001_build_alert_store_keys_are_org_composite` | AC-005 / BC-3.2.001 inv-1 | PASS |
| `test_BC_3_2_001_reset_for_removes_org_a_alert_entries_preserves_org_b` | AC-006 / BC-3.2.001 EC-004 / TV-3.2.001-05 | **FAIL** |
| `test_BC_3_2_003_reset_for_removes_org_a_session_tokens_preserves_org_b` | AC-006 / BC-3.2.003 EC-004 | **FAIL** |
| `test_BC_3_2_001_reset_for_clears_both_stores_atomically_for_org_a` | AC-006 (atomic) / BC-3.2.001 EC-004 + BC-3.2.003 EC-004 | **FAIL** |
| `test_BC_3_2_003_invariant_cross_org_session_validation_always_false` (proptest) | AC-007 / VP-3.2.003-01 | PASS |
| `test_BC_3_2_001_invariant_cross_org_alert_lookup_always_none` (proptest) | AC-007 / VP-3.2.001-01 | PASS |
| `test_BC_3_2_001_invariant_org_id_flip_kills_mutation` (proptest) | AC-007 / VP-3.2.001-03 / TD-DTU-MUTATE-COVERAGE-001 | PASS |
| `test_BC_3_2_001_invariant_reset_for_selectivity` (proptest) | AC-007 / VP-3.2.001-04 + VP-3.2.003-03 | **FAIL** |
| `test_BC_3_2_003_http_session_token_registered_for_org_a_rejected_by_org_b` | AC-002 (HTTP layer) | **FAIL** |
| `test_BC_3_2_001_http_reset_for_invalidates_org_a_preserves_org_b` | AC-006 (HTTP layer) | **FAIL** |

## Passing Tests Rationale

The 9 passing tests are not false positives — they correctly verify behavior that
was already implemented in the prior stub commit (5e84fb0c):

- `alert_store` keyed as `HashMap<(OrgId, String), AlertStatus>` ✓
- `session_store` keyed as `HashSet<(OrgId, String)>` ✓
- `register_session(org_id, token)` inserts `(org_id, token)` ✓
- `is_valid_session(org_id, token)` checks `(org_id, token)` ✓
- `build_alert_store(org_id, ...)` keys all entries under `(org_id, alert_id)` ✓

These are regression guards. They must continue to pass through the implementation phase.

## Implementation Order (Handoff to Implementer)

To make each failing test pass in order:

1. **`reset_for(org_id: OrgId)`** — implement in `src/state.rs`:
   - Retain all `(k_org, k_id)` entries where `k_org != org_id` in `alert_store`
   - Retain all `(k_org, k_token)` entries where `k_org != org_id` in `session_store`
   - Both stores cleared atomically (sequential Mutex acquire/release is acceptable)
   - This passes AC-006 unit tests (3 tests) and the `reset_for_selectivity` proptest.

2. **`extract_org_id(headers: &HeaderMap) -> OrgId`** — implement in `src/routes/alerts.rs`:
   - Read `X-Prism-Org-Id` header; parse as UUID; wrap as `OrgId::from_uuid`
   - On missing/malformed header: return a fallback (define behavior per ADR-008 §2.1)
   - This unblocks the HTTP-layer isolation tests and restores the previously-passing
     HTTP test suites (ac_1 through ac_8) which are currently ALL failing because
     `extract_org_id` panics on every request.

Note: fixing `extract_org_id` is also needed to restore the 13 pre-existing HTTP tests
that are currently failing as a side-effect of the stub. Those are NOT new failures from
this Red Gate — they failed before this commit.

## Notes

- `proptest-regressions` file committed alongside tests to preserve the shrunk failing
  case for `test_BC_3_2_001_invariant_reset_for_selectivity`.
- The 4 `unused_doc_comments` warnings on `proptest!` blocks are cosmetic; proptest
  macros don't process doc comments. These are suppressed at file level in the
  project pattern.
