---
document_type: story
level: L4
version: "1.0"
status: draft
producer: state-manager
timestamp: 2026-05-07T00:00:00
story_id: S-MAINT-001
title: "Widen lefthook pre-commit and pre-push to run cargo clippy --all-targets"
epic: maintenance
wave: maintenance
crate: workspace
bcs: []
vps: []
depends_on: []
triggered_by: TD-MAINT-001
---

# S-MAINT-001: Widen lefthook clippy to `--all-targets`

## Summary

TD-MAINT-001 R-1 (HIGH): Widen lefthook pre-commit and pre-push to run
`cargo clippy --all-features --all-targets -- -D warnings` so test-file
lint violations (e.g., `clippy::unwrap_used`, `clippy::expect_used`) are
caught before commit rather than discovered in adversarial review.

## Background

59 `clippy::unwrap_used`/`clippy::expect_used` violations accumulated in
`crates/prism-query/src/tests/integration_tests.rs` and
`crates/prism-query/src/tests/bc_gap_fill_tests.rs` between S-3.02 delivery
and adversarial pass-1 of `maintenance/clippy-unwrap-cleanup`. Root cause:
the pre-commit hook and pre-push hook both run `cargo clippy` **without**
`--all-targets`, which silently skips `#[cfg(test)]` modules and external
test files.

See TD-MAINT-001 (`.factory/tech-debt/TD-MAINT-001-clippy-enforcement-gap.md`)
for the full root-cause analysis.

## Scope

### R-1 — lefthook.yml pre-commit and pre-push hook widening

In `lefthook.yml`:

```yaml
# Before (pre-commit clippy step):
cargo clippy --all-features -- -D warnings

# After:
cargo clippy --all-features --all-targets -- -D warnings
```

Apply the same change to the pre-push hook if it also calls clippy directly.
If `just check` is called instead, update the `check` recipe in `Justfile`
to pass `--all-targets` to clippy.

### R-2 — Verify Justfile `just clippy` recipe

Update `just clippy` (workspace clippy recipe) to also include `--all-targets`
so manual `just clippy` runs are consistent with the hook.

### R-3 — Smoke-test

After updating hooks and Justfile:
1. Run `just clippy` — must pass (all test files have `#![allow]` per
   workspace convention established in `maintenance/clippy-unwrap-cleanup`).
2. Confirm `lefthook run pre-commit` exits 0 on the current HEAD.
3. Confirm `lefthook run pre-push` exits 0 on the current HEAD.

## Acceptance Criteria

| ID | Criterion |
|----|-----------|
| AC-1 | `lefthook.yml` pre-commit clippy invocation includes `--all-targets` |
| AC-2 | `lefthook.yml` pre-push clippy invocation (or `just check` recipe) includes `--all-targets` |
| AC-3 | `just clippy` includes `--all-targets` |
| AC-4 | `just clippy` exits 0 on current develop HEAD (no new failures introduced) |
| AC-5 | `lefthook run pre-commit` exits 0 on develop HEAD after the hook change |

## Out of Scope

- Adding `#![allow]` to any additional test files (covered by workspace
  convention; resolved in `maintenance/clippy-unwrap-cleanup`)
- Auditing non-prism-query crate lint configurations (TD-MAINT-001 R-3,
  separate story or manual sprint task)

## References

- TD-MAINT-001 R-1 (HIGH): `.factory/tech-debt/TD-MAINT-001-clippy-enforcement-gap.md`
- Adversary finding: `maintenance/clippy-unwrap-cleanup` pass-2 I-1 (process gap)
- Maintenance branch: `maintenance/clippy-unwrap-cleanup`
