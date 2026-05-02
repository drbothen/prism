# Review Findings — W3-FIX-SEC-003

## Review Metadata

| Field | Value |
|-------|-------|
| PR | #114 |
| Story | W3-FIX-SEC-003 |
| Branch | feature/W3-FIX-SEC-003 → develop |
| Reviewer | pr-manager fresh-context diff review (HEAD SHA 54f88a634f2921cc4f94a6d71548d96ff63eae5f) |
| Cycle | 2 (re-review on current SHA post demo-evidence commit) |
| Date | 2026-05-01 |

## Convergence Table

| Cycle | Findings | Blocking | Fixed | Remaining |
|-------|----------|----------|-------|-----------|
| 1     | 0        | 0        | 0     | 0 → APPROVE (prior review) |
| 2     | 1        | 0        | 0     | 1 suggestion (non-blocking) → **APPROVE** |

## Verdict: APPROVE

All 7 acceptance criteria are verified. No blocking findings. The implementation
is correct, well-structured, and the test suite provides strong regression coverage.

---

## Positive Observations

### Implementation Correctness

1. **Pre-join checks fire unconditionally for existing targets.** `validate_spec_path`
   correctly runs `is_absolute()` and `Component::ParentDir` inspection with zero
   filesystem I/O before any join occurs. Rejection is pure and deterministic.

2. **Symlink escape protection is correct.** The post-join path uses `canonicalize()`
   on both the candidate and the parent, then `starts_with()`. This correctly handles
   multi-hop symlink chains (the OS resolves all links before the prefix check).

3. **E-CFG-018 is in the multi-error collector.** `errors.push(e)` at line 562 does
   not short-circuit the outer validation pass — `validate_dtu_block` is called per-DTU
   inside a `for` loop, and `return` exits only the current DTU's validation (not the
   file or pass). AC-005 is satisfied.

4. **E-CFG-017 sequence is preserved.** E-CFG-018 is inserted between E-CFG-017 and
   E-CFG-020 with no collision. E-CFG-019 is intentionally absent (not reserved).

5. **No new runtime dependencies.** Only `std::path` is used. `tempfile` is already
   a dev-dependency and only appears in the test target.

6. **Public API surface is minimal and correct.** `validate_spec_path` is `pub` to
   enable direct testing via the integration test crate. The function signature
   (`config_path: &Path, spec_path: &str`) is clean and consistent with the rest of the
   validator module.

7. **Test quality is high.** Tests exercise real temp-dir structures (not mocks). The
   AC-004 symlink test creates an actual filesystem symlink and verifies the
   `canonicalize()` boundary check fires. All 7 tests pass against the real implementation.

8. **Error Display is safe.** `E-CFG-018` Display includes `spec_path` (user-supplied
   string) and `message` (static string or OS error message). It uses `file.display()`
   for the config file path. No secret values or internal paths beyond what the operator
   supplied are exposed.

### AC Coverage

| AC | Title | Status |
|----|-------|--------|
| AC-001 | `..` traversal rejected | PASS — 2 tests |
| AC-002 | Absolute path rejected | PASS — 2 tests |
| AC-003 | Relative within-tree passes | PASS — 2 tests |
| AC-004 | Symlink escape rejected | PASS — 1 test (unix-only, gated `#[cfg(unix)]`) |
| AC-005 | Multi-error collection | PASS — structural (return exits DTU block only) |
| AC-006 | Regression tests in path_traversal.rs | PASS — 7/7 |
| AC-007 | Process exits code 1 on traversal | PASS — covered by AC-001 (startup rejection) |

---

## Acknowledged Non-Blocking Finding (from security-findings.md)

**SEC-003-R1 (MEDIUM):** `resolved.exists()` gate at `validator.rs:554` bypasses
the pre-join `..` check for non-existent traversal targets (e.g.,
`../../../../etc/nonexistent`). For such inputs, `E-CFG-015` is emitted instead of
`E-CFG-018`. The primary CWE-22 HIGH vector (reading an existing file) is fully
mitigated. This finding is explicitly acknowledged as non-blocking tech-debt in
`.factory/code-delivery/W3-FIX-SEC-003/security-findings.md`. Not re-raised here.

---

## Test Evidence

```
running 7 tests
test test_BC_3_3_004_AC_002_absolute_path_root_slash_rejected ... ok
test test_BC_3_3_004_AC_001_single_dotdot_always_rejected ... ok
test test_BC_3_3_004_AC_003_dot_prefix_relative_within_tree_passes ... ok
test test_BC_3_3_004_AC_001_relative_path_traversal_rejected_with_e_cfg_018 ... ok
test test_BC_3_3_004_AC_003_relative_within_tree_passes ... ok
test test_BC_3_3_004_AC_002_absolute_path_rejected ... ok
test test_BC_3_3_004_AC_004_symlink_escape_rejected ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

running 46 tests
[... all 46 crate tests pass ...]
test result: ok. 46 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

---

## APPROVE

PR #114 is approved for merge. All acceptance criteria verified, 0 blocking findings,
7/7 path-traversal tests pass, 46/46 crate tests pass, SEC-003 HIGH gate finding resolved.
