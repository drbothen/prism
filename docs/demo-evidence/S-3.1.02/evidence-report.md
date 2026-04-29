# Demo Evidence Report — S-3.1.02

**Story:** S-3.1.02 — workspace: rename TenantId → OrgSlug across all crates  
**Behavioral Contract:** BC-3.1.001 (OrgRegistry Bijective Slug/UUID Resolution)  
**Implementation commit:** 8d676f60  
**Recorded:** 2026-04-29  
**Tool:** VHS 0.10.0 (CLI product)

---

## Coverage Map

| Recording | AC | Path | Files |
|-----------|-----|------|-------|
| AC-001-workspace-tests-green | AC-4 (BC-3.1.001 invariant 1) | Success | [.tape](AC-001-workspace-tests-green.tape) · [.gif](AC-001-workspace-tests-green.gif) · [.webm](AC-001-workspace-tests-green.webm) |
| AC-002-rename-verification | AC-1, AC-3 (BC-3.1.001 precondition 2 / invariant 2) | Success | [.tape](AC-002-rename-verification.tape) · [.gif](AC-002-rename-verification.gif) · [.webm](AC-002-rename-verification.webm) |

---

## AC-001 — Workspace Tests All GREEN Post-Rename

**Acceptance Criterion:** AC-4 — `cargo test --workspace` passes with zero compilation
errors and zero test failures after the rename sweep.

**Command demonstrated:**

```
cargo test --workspace --all-features 2>&1 | tail -20
```

**Result:** All test suites report `test result: ok`. No compilation errors. Deprecation
warnings from the `TenantId` alias are not treated as errors (migration wave behaviour).

**Evidence files:**
- `AC-001-workspace-tests-green.tape` — VHS script source
- `AC-001-workspace-tests-green.gif` — animated recording (199 KB)
- `AC-001-workspace-tests-green.webm` — archival recording (731 KB)

---

## AC-002 — Symbol Rename Verification

**Acceptance Criteria:** AC-1, AC-3

- **AC-1:** `OrgSlug` declared in `crates/prism-core/src/tenant.rs` with constant
  `ORG_SLUG_PATTERN = "^[a-zA-Z0-9_-]{1,64}$"` (renamed from `TENANT_ID_PATTERN`).
- **AC-3:** Deprecation alias `#[deprecated(since = "3.0.0", note = "use OrgSlug")] pub type TenantId = OrgSlug;`
  present; all production call sites updated to `OrgSlug` directly.

**Commands demonstrated:**

```bash
# Show OrgSlug struct, ORG_SLUG_PATTERN constant, and deprecation alias in tenant.rs
grep -n 'OrgSlug\|ORG_SLUG_PATTERN\|deprecated.*TenantId' crates/prism-core/src/tenant.rs

# Confirm no non-deprecated TenantId references in production source
grep -rn 'TenantId' crates/ --include='*.rs' \
  | grep -v 'deprecated\|#\[allow\|test\|//!' | grep -v 'Binary' | head -20

# Count total OrgSlug symbol occurrences across workspace (175 usages confirmed)
grep -rn 'OrgSlug' crates/ --include='*.rs' | grep -v 'Binary' | wc -l
```

**Result:**
- `ORG_SLUG_PATTERN` and `OrgSlug` struct declared in `tenant.rs`
- Deprecation alias for `TenantId` present (one-wave migration alias)
- 175 `OrgSlug` symbol usages across workspace
- No non-deprecated `TenantId` references in non-test production code

**Evidence files:**
- `AC-002-rename-verification.tape` — VHS script source
- `AC-002-rename-verification.gif` — animated recording (1.4 MB)
- `AC-002-rename-verification.webm` — archival recording (512 KB)

---

## Error Paths

This story is a mechanical rename (pure refactor). The relevant error paths are:

| Edge Case | Coverage |
|-----------|----------|
| `TenantId` used via deprecation alias compiles with warning, not error | Covered by AC-001 (workspace test run includes any call sites using the alias — zero errors observed) |
| `OrgSlug::new` rejects invalid slugs | Covered by existing unit tests in `prism-core/tests/` (ac_1_tenant_id_rejects_empty.rs, ac_3_tenant_id_rejects_path_traversal.rs) which execute during the AC-001 cargo test run |

---

## Acceptance Criteria Status

| AC | Description | Status |
|----|-------------|--------|
| AC-1 | `OrgSlug` + `ORG_SLUG_PATTERN` declared in `tenant.rs` | PASS — AC-002 |
| AC-2 | `OrgSlug::try_new` validation unit tests pass | PASS — AC-001 (all prism-core tests green) |
| AC-3 | Deprecation alias present; call sites updated to `OrgSlug` | PASS — AC-002 |
| AC-4 | `cargo test --workspace` zero errors, zero failures | PASS — AC-001 |
