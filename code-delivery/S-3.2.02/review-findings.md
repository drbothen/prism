# Review Findings — S-3.2.02

## Convergence Tracking

| Cycle | Findings | Blocking | Fixed | Remaining |
|-------|----------|----------|-------|-----------|
| 1 (pr-reviewer) | 0 | 0 | 0 | 0 → APPROVE |
| 1 (CI gate) | 1 | 1 | 1 | 0 |

**Total cycles to APPROVE:** 1
**Total blocking findings resolved:** 1 (CI failure)

## Security Review (Step 4)

| Severity | Count | Notes |
|----------|-------|-------|
| Critical | 0 | — |
| High | 0 | — |
| Medium | 1 | Empty tag_key silently accepted in DTU route (test-only crate, no production exposure) |
| Low | 0 | — |

No implementer dispatch required (0 critical/high).

## PR Review Cycle 1

**Reviewer verdict:** APPROVE

**Checklist results:**
- [x] Composite key (OrgId, String) correctly enforces org isolation
- [x] All 6 ACs covered by tests (multi_tenant.rs traceability table complete)
- [x] DEFAULT_ORG_ID #[cfg(test)] gate verified
- [x] Fixture registries remain bare-String keyed
- [x] BehavioralClone::reset() delegates to reset_all() (clone.rs:231 → state.reset() → reset_all())
- [x] Proptest 1000 cases (explicit cases:1000 config block)
- [x] tags_for HashSet→Vec dedup: O(n²) but acceptable for DTU harness

**Non-blocking note:** `tags_for` deduplication changed from `HashSet`-based (automatic, sorted output) to `Vec`-based with `contains` check (O(n²), fixture-tags-first order). Acceptable for test DTU with small tag counts.

## CI Fix (Cycle 1)

**Finding:** `Test (no-default-features)` failed with `error[E0432]: unresolved import crate::state::DTU_ROUTE_ORG_ID`

**Root cause:** `DTU_ROUTE_ORG_ID` declared with `#[cfg(feature = "dtu")]` gate, but imported unconditionally in `routes/tags.rs` and `routes/devices.rs`. CI job runs `cargo test --workspace --no-default-features`, which omits the `dtu` feature.

**Fix:** Removed `#[cfg(feature = "dtu")]` from `DTU_ROUTE_ORG_ID` in `state.rs`. Updated doc comment to explain why the constant is not feature-gated.

**Fix commit:** `ba867409` — pushed to `feature/S-3.2.02` 2026-04-29.

## Final Status

- Review verdict: APPROVE (0 blocking findings)
- CI: 14/14 PASS after fix commit ba867409
- Dependencies: S-6.10 (#12) MERGED 2026-04-22 ✓
- PR #88 MERGED: SHA 65cb3269e58644b9df2893b62fb87246abd8e9d4 at 2026-04-29T17:58:10Z
- Remote branch feature/S-3.2.02 deleted (auto-deleted by GitHub on merge)
