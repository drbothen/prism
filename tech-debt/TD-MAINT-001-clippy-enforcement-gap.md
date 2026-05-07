# TD-MAINT-001 — Clippy Enforcement Gap: `unwrap_used`/`expect_used` in Test Files

**Status:** resolved  
**Severity:** medium  
**Filed:** 2026-05-07  
**Resolved:** 2026-05-07 (maintenance/clippy-unwrap-cleanup)

---

## Summary

59 accumulated `clippy::unwrap_used` and `clippy::expect_used` violations existed
in `crates/prism-query/src/tests/integration_tests.rs` and
`crates/prism-query/src/tests/bc_gap_fill_tests.rs` at the time of the first
adversarial pass against those files.

---

## Root Cause Analysis

### Timeline

| Date | Event |
|------|-------|
| 2026-04-26 | `clippy::unwrap_used = "deny"` and `clippy::expect_used = "deny"` added to `crates/prism-query/Cargo.toml` via commit `0be11cd6` (feat(S-2.08)) |
| 2026-05-06 | S-3.02 stories delivered, writing `integration_tests.rs` and `bc_gap_fill_tests.rs` with `unwrap()`/`expect()` throughout |
| 2026-05-06 | S-3.01/S-3.06 tests (`parser_tests.rs`, `write_parser_unit_tests.rs`) had `#![allow(clippy::unwrap_used, clippy::expect_used)]` at file level — correct |
| 2026-05-07 | Adversary pass-1 flagged 59 violations in the two new test files |

### Why Violations Accumulated

The `[lints.clippy]` deny rules in `Cargo.toml` apply to `--all-targets`, which
includes test code. However:

1. The pre-commit hook runs `cargo clippy` **without** `--all-targets`. Violations
   in `#[cfg(test)]` modules and test files are silently skipped at commit time.
2. The pre-push hook runs `just check`, which calls `cargo clippy` also without
   `--all-targets` in the fast path.
3. CI was not verifying `--all-targets` for clippy.

This means test-file lints were never enforced locally or in CI. The lint was
`deny` in theory but `allow` in practice for test code.

### Why Other Test Files Were Correct

`parser_tests.rs`, `regression_tests.rs`, `write_parser_unit_tests.rs`, and 40+
workspace test files had `#![allow(clippy::unwrap_used, clippy::expect_used)]`
at the file level — applied proactively by the test-writer or implementer as
workspace convention. The S-3.02 files missed this step.

---

## Workspace Survey (as of 2026-05-07, HEAD 159e922b)

Survey run from worktree `maintenance/clippy-unwrap-cleanup` against two test
path patterns (some crates use `src/tests/`, others use `tests/` at the crate root):

```
rg "\.unwrap\(\)" crates/*/src/tests/ -c | sort -t: -k2 -nr
rg "\.unwrap\(\)" crates/*/tests/ -c | sort -t: -k2 -nr
```

### Embedded test files (crates/*/src/tests/)

Total: 139 `.unwrap()` calls across 16 files.

| File | unwrap() count |
|------|---------------|
| prism-query/src/tests/regression_tests.rs | 27 |
| prism-query/src/tests/write_parser_unit_tests.rs | 21 |
| prism-credentials/src/tests/store_tests.rs | 15 |
| prism-query/src/tests/integration_tests.rs | 14 (lint-clean via #![allow]) |
| prism-ocsf/src/tests/bc_2_02_012_class_selector.rs | 13 |
| prism-audit/src/tests/bc_2_05_008.rs | 12 |
| prism-audit/src/tests/bc_2_05_004.rs | 10 |
| prism-audit/src/tests/bc_2_05_002.rs | 10 |
| prism-core/src/tests/test_cursor_registry.rs | 5 |
| prism-audit/src/tests/bc_3_1_001_org_fields.rs | 5 |
| prism-core/src/tests/test_credential_name.rs | 2 |
| prism-sensors/src/tests/bc_2_01_014.rs | 1 |
| prism-query/src/tests/parser_tests.rs | 1 |
| prism-query/src/tests/bc_gap_fill_tests.rs | 1 (lint-clean via #![allow]) |
| prism-ocsf/src/tests/bc_2_02_001_pool.rs | 1 |
| prism-audit/src/tests/bc_2_05_003.rs | 1 |

### External test files (crates/*/tests/)

Top offenders from external test directories (separate `tests/` crate-root path):

| File | unwrap() count |
|------|---------------|
| prism-customer-config/tests/validation_tests.rs | 70 |
| prism-credentials/tests/bc_3_2_002_org_id_namespace.rs | 60 |
| prism-spec-engine/tests/hot_reload_tests.rs | 53 |
| prism-customer-config/tests/startup_boot_test.rs | 31 |
| prism-spec-engine/tests/plugin_tests.rs | 17 |
| prism-dtu-armis/tests/bc_3_4_armis_generator.rs | 17 |
| prism-credentials/tests/bc_2_03_009_resolve_secret.rs | 14 |
| prism-credentials/tests/bc_3_2_002_trait_impl.rs | 12 |
| prism-customer-config/tests/cr003_slug_pattern.rs | 11 |
| prism-security/tests/bc_2_04_010_test.rs | 10 |
| (+ 55 more files with 1–9 calls each) | — |

Note: The stale survey in the v1 draft cited `prism-credentials/store_tests.rs` —
that file lives at `crates/prism-credentials/src/tests/store_tests.rs` (embedded
test path). No file named `store_tests.rs` exists under `crates/*/tests/`.

All crates other than `prism-query` should be audited against their `[lints.clippy]`
configuration to verify test files are covered by `#![allow]`.

---

## Resolution (this maintenance branch)

- Added `#![allow(clippy::unwrap_used, clippy::expect_used)]` to
  `integration_tests.rs` and `bc_gap_fill_tests.rs` — matching the workspace
  convention used in 40+ other test files.
- Reverted the adversary's `?`-conversion approach (wrong axis: converting test
  panics to propagated errors changes semantics and diverges from workspace pattern).

---

## Recommendations

### R-1 (HIGH): Widen pre-commit/pre-push clippy to `--all-targets`

In `lefthook.yml` and `Justfile`, change:

```
cargo clippy --all-features -- -D warnings
```

to:

```
cargo clippy --all-features --all-targets -- -D warnings
```

This ensures test-file lint violations are caught at commit time, not in
adversarial review.

### R-2 (MEDIUM): Establish workspace-wide `#![allow]` convention at story scoping

When a story creates new test files in a crate with `unwrap_used = "deny"` in
`Cargo.toml`, the story spec MUST include an explicit task to add the file-level
`#![allow(clippy::unwrap_used, clippy::expect_used)]` at the top of each test file.
This should be a checklist item in the story template.

### R-3 (LOW): Audit non-prism-query test files

Run `cargo clippy --all-features --all-targets -- -D warnings` across all crates
to verify that the 138 `unwrap()` calls in other test files are all covered by
`#![allow]` attributes. Crates to prioritize: `prism-credentials`, `prism-ocsf`,
`prism-audit`.

---

## References

- Maintenance branch: `maintenance/clippy-unwrap-cleanup`
- Adversary finding: F-7 (process gap — enforcement gap allowing 59 violations)
- Adversary finding: F-1 (HIGH) / F-4 (MEDIUM) — `build_dml_parser` dead code (resolved separately)
- Commit: `0be11cd6` — where `clippy::unwrap_used = "deny"` was first added (2026-04-26)
- Story files violated: `integration_tests.rs` (S-3.02), `bc_gap_fill_tests.rs` (S-3.02)
