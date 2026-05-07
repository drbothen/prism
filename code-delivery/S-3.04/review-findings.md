# PR #133 Review Findings — S-3.04 Alias System

**Story:** S-3.04 — prism-query: Alias System (P1)
**PR:** #133
**Head commit (original):** `90699a5b1d32b528ebf351fefe0bee168df235dd`
**Head commit (after fix-pass):** `ebdc3c336937f7adbf5be17c139b7c00693aa58e`

---

## Convergence Summary

| Cycle | Findings | Blocking | Fixed | Remaining |
|-------|----------|----------|-------|-----------|
| 1 (CI + security + adversarial + code) | 3 CI + 1 medium | 2 | 3 CI | 1 medium (non-blocking) |
| 2 (re-review post fix-pass) | 0 | 0 | 0 | 0 → APPROVE |

**Verdict:** APPROVE after fix-pass (1 cycle)

---

## Cycle 1 Findings

### CI Failures (BLOCKING — fixed in ebdc3c33)

**CI-001** — `Test (aarch64-apple-darwin)`: `test_BC_2_11_014_cascade_delete_respects_scope`
- **Severity:** HIGH / Blocking
- **Root cause:** Non-unique temp file name in `alias_store.rs::write_entries_to_file`. Parallel
  nextest threads computing the same `aliases.toml.tmp.{nanos}` rename target; one thread's rename
  fails with "No such file or directory" because the source temp file was already renamed by another
  thread (or the prior run's file was in a different state).
- **Fix:** Added path-hash discriminant to temp file name: `aliases.toml.tmp.{path_hash:016x}.{nanos}`
- **File:** `crates/prism-query/src/alias_store.rs` line ~363
- **Status:** FIXED

**CI-002** — `Test (x86_64-pc-windows-msvc)`: 12+ test failures
- **Severity:** HIGH / Blocking
- **Root cause:** Hardcoded `/tmp/` paths in `alias_tests.rs` and `vp013_cycle_detection.rs`.
  Windows has no `/tmp/` directory; `File::create("/tmp/...")` fails with "The system cannot
  find the path specified. (os error 3)".
- **Fix:** Added `temp_path()` helper using `std::env::temp_dir()`. Replaced all 50+ `/tmp/`
  literals in both files. Added `use super::temp_path` in `vp037_proptest` nested module.
- **Files:** `crates/prism-query/src/tests/alias_tests.rs`,
  `crates/prism-query/src/proofs/vp013_cycle_detection.rs`
- **Status:** FIXED

**CI-003** — `prop_vp013_transitive_cycle_via_3node_graph`: "Too many global rejects"
- **Severity:** MEDIUM / Blocking (cascades from CI-002)
- **Root cause:** All `prop_assume!` calls in proptest rejected because file writes to `/tmp/`
  fail on Windows, causing proptest to abort with "Too many global rejects".
- **Fix:** Cascades from CI-002 fix (temp_path helper).
- **Status:** FIXED

### Security Review Findings (0 blocking)

**S-001** — `AliasStore::delete` takes `_token: ConfirmationToken` as dead parameter
- **Severity:** LOW / Non-blocking
- **Detail:** Token is validated in the tool layer (`token_store.consume()`) before `store.delete()`
  is called. The `_token` parameter is accepted as proof of consumption but not re-validated inside
  `delete()`. This is acceptable because `ConfirmationToken` can only be constructed via
  `ConfirmationTokenStore::consume()`.
- **Recommendation:** Consider renaming to `_validated_token` for clarity, or making `delete`
  `pub(crate)` if external callers aren't needed.
- **Status:** NOT FIXED (non-blocking, filed as suggestion)

### Adversarial Review Findings (0 blocking)

**A-001** — `AliasStore::delete` is `pub` but internal to alias management pattern
- **Severity:** MEDIUM / Non-blocking
- **Detail:** External callers could call `store.delete()` with any `ConfirmationToken` (obtained
  from any confirmed action, not necessarily delete-specific). However, `ConfirmationToken`
  construction requires `consume()` which validates action parameters — exploitability is low.
- **Recommendation:** Consider making `pub(crate)` if no external callers are needed.
- **Status:** NOT FIXED (non-blocking, technical debt)

---

## Cycle 2 (post fix-pass re-review)

All blocking findings resolved. CI monitoring on `ebdc3c33` in progress.

**Verdict: APPROVE pending CI green on `ebdc3c33`**

---

## LOCAL Adversarial Cascade (pre-PR)

18 LOCAL adversarial passes with 3/3 CLEAN streak (passes 16/17/18):
- CRIT-001: AC-13 vacuous — ConfirmationTokenStore not wired (FIXED)
- CRIT-002: VP-037 fuzz target decode_fuzz_input stub (FIXED)
- HIGH-001..HIGH-005+: ungated symbols, SEC-009 cascade scope, per-client cycle detection (FIXED)
- All SEC/CR findings from passes 1-15: FIXED
