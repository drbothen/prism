# AC-6 Evidence — Cross-Newtype `*::new_unchecked` Audit + OrgSlug Validated Constructor

**Story:** S-PLUGIN-PREREQ-C v1.3
**Status: SATISFIED**
**Resolves:** TD-S-PLUGIN-PREREQ-A-006 (P3)
**BC anchor:** BC-2.01.013 postcondition — the spec-driven adapter surface enforces
validation-on-construct for sensor identifiers and org identifiers; `new_unchecked` bypasses
that enforcement and must be explicitly inventoried and justified.

---

## AC Summary (quoted from story v1.3)

> All pub-API validation-bypass constructors in `crates/prism-core/` are audited. The known
> example is `OrgSlug::new_unchecked` in `crates/prism-core/src/tenant.rs`.
>
> For each identified `*::new_unchecked`:
> - (a) Add/update the doc-comment to clearly state the precondition the caller must assert.
> - (b) Optionally restrict visibility to `cfg(any(test, feature = "test-helpers"))` if
>   test-only (implementer's judgment).
>
> Note (fix-burst-1 closure): the production caller in `prism-query/src/materialization.rs`
> has been migrated from `OrgSlug::new_unchecked()` to validated `OrgSlug::new()` with
> explicit error handling. The `new_unchecked` constructor remains in the allowlist for test
> fixtures only.

---

## Red Gate Test

**File:** `crates/prism-core/tests/new_unchecked_audit.rs`
**Test:** `test_BC_2_01_013_new_unchecked_inventory_baseline`

This test walks every `.rs` file under `crates/prism-core/src/` looking for `fn new_unchecked`
declarations. For each found site, it verifies the site is present in the
`GATED_OR_ALLOWLISTED_UNCHECKED` symbol-keyed allowlist. Any ungated site causes the test to
fail with an explicit error message naming the file and type, and instructing the developer
to either gate the constructor or add a justified allowlist entry.

The allowlist uses `(file_suffix, type_name)` tuples as keys, ensuring that a new
`OtherType::new_unchecked` added to `tenant.rs` is NOT automatically allowlisted just because
`OrgSlug::new_unchecked` in `tenant.rs` is already present.

---

## Real Test Output

```
$ cd /Users/jmagady/Dev/prism/.worktrees/S-PLUGIN-PREREQ-C && \
  cargo nextest run -p prism-core -E 'test(test_BC_2_01_013_new_unchecked_inventory_baseline)' \
  --no-fail-fast

   Compiling prism-core v0.1.0 (crates/prism-core)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 5.88s
────────────
 Nextest run ID 1d692cf7-eb72-4f1f-adb8-786c85f74e8b with nextest profile: default
    Starting 1 test across 19 binaries (240 tests skipped)
        PASS [   0.018s] (1/1) prism-core::new_unchecked_audit test_BC_2_01_013_new_unchecked_inventory_baseline
────────────
     Summary [   0.019s] 1 test run: 1 passed, 240 skipped
```

---

## Allowlist (GATED_OR_ALLOWLISTED_UNCHECKED)

From `crates/prism-core/tests/new_unchecked_audit.rs`:

```
const GATED_OR_ALLOWLISTED_UNCHECKED: &[(&str, &str)] = &[
    // AC-6 audit result (S-PLUGIN-PREREQ-C): OrgSlug::new_unchecked in tenant.rs.
    // Symbol-keyed: only OrgSlug in tenant.rs is allowlisted; a future OtherType::new_unchecked
    // in tenant.rs would fail this test even though OrgSlug is accepted.
    ("tenant.rs", "OrgSlug"),
];
```

One entry: `OrgSlug` in `tenant.rs`. All other `prism-core` source files contain no
`fn new_unchecked` declarations as of the AC-6 audit sweep.

---

## Production Code Reference

**File:** `crates/prism-core/src/tenant.rs`

`OrgSlug::new_unchecked` retains `pub` visibility because it serves test fixtures that need
to construct `OrgSlug` values for arbitrary string inputs without the validated constructor's
charset constraint. The doc-comment on `new_unchecked` was updated during fix-burst-1 (commit
ed661aea, F-LP2-MED-001) to document:
- The precondition the caller must assert (the string must already satisfy `OrgSlug` charset
  rules: `[a-zA-Z0-9_-]{1,64}`)
- That this constructor bypasses validation and must not be called with untrusted input
- That production code should use `OrgSlug::new()` instead

**File:** `crates/prism-query/src/materialization.rs`

The production call site previously using `OrgSlug::new_unchecked` was migrated to
`OrgSlug::new()` (validated constructor) during fix-burst-1. The migration site uses
`OrgSlug::new(&synthetic_candidate)` with explicit error handling that falls back to
`OrgSlug::new("synthetic-unmapped")` when the candidate string does not satisfy the charset
constraint (HIGH-006 closure). The comment at the call site (commit 8908bf27 cluster) documents
the rationale: UUID-derived synthetic slugs are valid for `OrgSlug`, but the validated path
is used as a defense-in-depth measure.

---

## Cross-References

- HIGH-005 closure: audit sweep confirming `OrgSlug::new_unchecked` was the only ungated site
- HIGH-006 closure: `prism-query/src/materialization.rs` production caller migrated to
  `OrgSlug::new()` (validated constructor)
- TD-S-PLUGIN-PREREQ-A-006: original finding from PREREQ-A adversary pass-7 (F-LP7-LOW-002)
- BC-2.01.013 v1.6: validation-on-construct postcondition for sensor identifiers and org identifiers
