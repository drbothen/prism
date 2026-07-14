---
document_type: adversarial-review
scope: LOCAL
fix_pr: DEFECT-PQL-FNCALL-LHS-001
passes: [23]
feature_head_at_review: 4e9d3f96
date: 2026-07-13
clean_strict: false
clean_pr_merge: true
finding_counts:
  total: 4
  crit: 0
  high: 0
  med: 0
  low: 1
  obs: 3
  process_gap: 0
code_behavior_defects: 0
streak_after: 0/3
convergence: NOT_CONVERGED
authored_by: orchestrator-relay
---

# LOCAL Adversary Pass 23 — DEFECT-PQL-FNCALL-LHS-001

---

## Pass 23 (frozen 4e9d3f96; fresh-context adversary; fn-call-LHS PrismQL grammar + ADR-048 §D.7 aggregate gate + SqlPipe span translation; streak candidate 1/3 — RESET to 0/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=YES

**Findings:** 4 total (0 CRIT / 0 HIGH / 0 MED / 1 LOW / 3 OBS / 0 PROCESS-GAP)

**STREAK RESET: 0/3** (F-PQLFN-P23-LOW-001 is LOW severity; BC-5.39.001 requires ZERO findings of any severity for streak advancement)

**Code HEAD at review:** 4e9d3f96 (frozen; LOCAL-ONLY NOT pushed; prism-query 1619/1619; just check FULL WORKSPACE 5555/5555 GREEN; non-exhaustive 91/91)

**CLEAN(strict):** NO — 1 LOW + 3 OBS findings present; streak advancement criterion NOT satisfied

**CLEAN(PR-merge):** YES — ZERO CRIT + HIGH + MED findings (LOW + OBS only; non-blocking for merge gate)

**SAP-1 (Structured Event Catalog — BC-2.16.002):** PASS — grepped `event_type\s*=` across `crates/` workspace at frozen 4e9d3f96. Zero new `event_type` assignments at this HEAD. All emission sites verified against BC-2.16.002 §Postconditions catalog.

**POL-22/24 verified clean:** 14-production enumeration byte-accurate (grammar constraint enumerates all productions FuncCall::Scalar may not appear as LHS); templates verbatim per BC-2.11.004 v1.41.

---

## Finding Register

### F-PQLFN-P23-LOW-001 [LOW] [test-coverage] BC-2.11.004 v1.41 canonical LIKE-with-fn-call vector had no test

**Severity:** LOW

**Classification:** test-coverage (TD-VSDD-059 risk — behavioral claim in BC v1.41 without corresponding RED gate test)

**Location:** BC-2.11.004 v1.41 §Error Cases — LOW-002 now enumerates all 14 non-composable predicate-operator families including `LIKE`. The fix-burst-16 (OBS-001) expanded the family list but the new families (beyond IEQ/IIN/INE) were not paired with corresponding RED gate tests.

**Description:**
BC-2.11.004 v1.41 §Error Cases LOW-002 now reads: "14 predicate-operator families (IEQ/IIN/INE/ILT/IGT/ILTE/IGTE/LIKE/ILIKE/IN/NOT IN/BETWEEN/IS NULL/IS NOT NULL) do not admit fn-call as LHS." The IEQ/IIN/INE family had RED gate tests from earlier bursts. However, LIKE (and the other 11 non-IEQ/IIN/INE families) have no corresponding test exercising fn-call-LHS rejection through the LIKE predicate path.

The grammar does reject LIKE-with-fn-call-LHS at parse time (the constraint is grammar-level, not operator-specific), but this is a TD-VSDD-059 risk: the BC now asserts a 14-family scope and that assertion has no test locking it. If the grammar is ever relaxed, the test coverage gap means the regression would not be caught.

**Fix required:**
Add `test_BC_2_11_004_low_002_like_with_fncall_lhs_rejected` — a RED gate test that executes a LIKE query with fn-call LHS (e.g., `SELECT * FROM hosts WHERE lower(hostname) LIKE '%web%'`) and asserts rejection with `E-QUERY-001` / `QueryParseFailed`. Per the representative-coverage principle, one LIKE test is sufficient: IEQ already covers the "parse-error at operator boundary" class; LIKE spans a different operator class (pattern-match vs equality). The remaining 12 families share the identical field_path-only mechanism and require no additional coverage.

---

### F-PQLFN-P23-OBS-001 [OBS] [doc-drift] ast.rs span doc overstated sql_parser coverage

**Severity:** OBS

**Classification:** doc-drift

**Location:** `crates/prism-query/src/ast.rs` — `FuncCall::Scalar` variant span field doc comment

**Description:**
The `span` doc comment on `FuncCall::Scalar` states that `sql_parser` (the Chumsky SQL parser) populates `span` for all FuncCall::Scalar instances. This is inaccurate. The `sql_parser` path populates `span` only for fn-call nodes encountered inside WHERE clauses and HAVING clauses (which are processed by `parse_predicate_expr` → `filter_parser`). Outside those positions, `FuncCall::Scalar` nodes constructed via the aggregate parsing path (SELECT clause fn-calls) carry `Span::ZERO` because the aggregate parser does not use `map_with e.span()`.

The doc comment should enumerate the truthful population paths:
- WHERE/HAVING fn-calls (via `filter_parser.rs` / `parse_predicate_expr`) → truthful byte-offset
- SELECT clause fn-calls (aggregate parser path) → `Span::ZERO` (not populated)

**Fix required:**
Correct the `span` field doc comment to enumerate truthful-span paths (WHERE/HAVING) vs Span::ZERO paths (SELECT/aggregate).

---

### F-PQLFN-P23-OBS-002 [OBS] [defensive-programming] shift_scalar_spans_in_expr wildcard arm silently skipped TimestampArithmetic.base and InSubquery

**Severity:** OBS

**Classification:** defensive-programming

**Location:** `crates/prism-query/src/filter_parser.rs` — `shift_scalar_spans_in_expr`

**Description:**
`shift_scalar_spans_in_expr` introduced in fix-burst-17 handles all `Expr` variants via `_ => {}` wildcard after the `Expr::FuncCall(FuncCall::Scalar { .. })` arm. The wildcard correctly no-ops for variants that have no nested `FuncCall::Scalar` (e.g., `Expr::Literal`, `Expr::FieldPath`). However, two variants that CAN contain nested expressions are silently no-oped by the wildcard:

1. `Expr::TimestampArithmetic { base, .. }` — `base` is an `Box<Expr>` that could itself be a `FuncCall::Scalar`; the wildcard skips recursion into `base`.
2. `Expr::InSubquery { expr, .. }` — `expr` is a `Box<Expr>` that could contain a nested `FuncCall::Scalar`; wildcard skips recursion.

In practice, neither `TimestampArithmetic.base` nor `InSubquery.expr` would ever be a `FuncCall::Scalar` in the current grammar (the parser rejects fn-call as LHS for all predicates), so the silence is currently harmless. However, the `#[non_exhaustive]` discipline requires that wildcard arms be justified inline, and a wildcard over variants with nested expressions is an anti-pattern that prevents compile-time enforcement when new variants are added.

**Fix required:**
Replace the `_ => {}` wildcard with:
- Explicit recursive arms for `Expr::TimestampArithmetic` and `Expr::InSubquery` (recursion even though unreachable in practice — correctness-by-construction)
- No-op arms (with justification comment) for all remaining variants that have no nested `Expr` fields
- This removes the wildcard entirely so future new `Expr` variants force a compile error rather than silent skip

---

### F-PQLFN-P23-OBS-003 [OBS] [doc-drift] emitter catch-all hardcoded 'Aggregate/Window' message

**Severity:** OBS

**Classification:** doc-drift (misleading emit message in a production code path)

**Location:** `crates/prism-query/src/engine.rs` — `FuncCall` match in the expression-to-SQL emitter

**Description:**
The `FuncCall` arm in `expr_to_sql` (or equivalent emitter function) has a catch-all branch that emits a hardcoded error message referencing "Aggregate/Window function" even for `FuncCall::Scalar` variants. Since `FuncCall::Scalar` is now a valid variant (added by fix-burst-16 to support fn-call rejection), the catch-all message is no longer truthful — it would misidentify a `Scalar` fn-call failure as an aggregate/window problem, which is misleading for any operator or LLM-agent consuming the error.

The `FuncCall` enum is `#[non_exhaustive]`, so the emitter's match must have a catch-all arm. However, the arm should:
1. Have explicit match arms for `FuncCall::Aggregate` and `FuncCall::Window` with their truthful messages
2. Have a variant-agnostic catch-all that does NOT hardcode "Aggregate/Window"

**Fix required:**
Refactor the `FuncCall` match in the emitter to explicit `Aggregate` + `Window` arms with truthful messages, plus a variant-agnostic catch-all (e.g., "function call not supported in SQL emission context") respecting `#[non_exhaustive]`.

---

## Fix-Burst 18 Closure Audit

Fix-burst 18 addressed all 4 findings via branch 4e9d3f96 → 989588b7 (LOCAL-ONLY, NOT pushed):

**F-PQLFN-P23-LOW-001 (test-coverage):**
- `test_BC_2_11_004_low_002_like_with_fncall_lhs_rejected` added — asserts `SELECT * FROM hosts WHERE lower(hostname) LIKE '%web%'` returns `QueryParseFailed` with `E-QUERY-001`. GREEN lock on commit.
- Representative-coverage rationale documented in test comment: IEQ covers "equality operator class" + LIKE covers "pattern-match operator class"; remaining 12 productions share identical field_path-only mechanism, no additional tests needed.

**F-PQLFN-P23-OBS-001 (doc-drift):**
- `FuncCall::Scalar` `span` field doc comment corrected to enumerate:
  - Truthful-span paths: WHERE/HAVING fn-calls via `filter_parser.rs` `parse_predicate_expr` using `map_with e.span()`
  - Span::ZERO paths: SELECT/aggregate fn-calls (aggregate parser does not populate span)

**F-PQLFN-P23-OBS-002 (defensive-programming):**
- Wildcard `_ => {}` removed from `shift_scalar_spans_in_expr`
- Full 13-variant explicit `Expr` enumeration:
  - `Expr::FuncCall(FuncCall::Scalar { .. })` — shifts span (existing arm, unchanged)
  - `Expr::TimestampArithmetic { base, .. }` — recursion arm: `shift_scalar_spans_in_expr(base, delta)`
  - `Expr::InSubquery { expr, .. }` — recursion arm: `shift_scalar_spans_in_expr(expr, delta)`
  - Remaining 10 variants (Literal, FieldPath, Null, FuncCall::Aggregate, FuncCall::Window, etc.) — no-op arms with inline justification ("no nested Expr fields that could carry FuncCall::Scalar span")
- Future `Expr` variants now force compile error instead of silent skip

**F-PQLFN-P23-OBS-003 (doc-drift):**
- Emitter `FuncCall` match refactored:
  - Explicit `FuncCall::Aggregate { .. }` arm with truthful message
  - Explicit `FuncCall::Window { .. }` arm with truthful message
  - Variant-agnostic catch-all (respects `#[non_exhaustive]`): does NOT reference "Aggregate/Window"

**Test results at 989588b7:**
- `just iter prism-query`: 1620/1620 GREEN (1 new test LOW-001)
- `just check` FULL WORKSPACE: 5556/5556 GREEN (60 skipped); non-exhaustive 91/91; develop UNCHANGED @5f1b5771
- LOCAL-ONLY NOT pushed

---

## Standing Probe Results

**SAP-1 (Structured Event Catalog — BC-2.16.002):** PASS — grepped `event_type\s*=` across `crates/` workspace at frozen 4e9d3f96. Zero new `event_type` assignments in fn-call-LHS grammar or aggregate-gate surfaces at this HEAD. All emission sites verified against BC-2.16.002 §Postconditions catalog.

**SAP-2:** N/A — no sensor TOML spec modifications in this defect cascade.

**SID-1:** N/A — no `#[ignore]`'d tests driving spec-required behavior at @4e9d3f96.

**POL-22/24 verified clean:** 14-production enumeration in BC-2.11.004 v1.41 §Error Cases LOW-002 is byte-accurate. Grammar constraint correctly rejects all 14 families. Templates verbatim.

---

## BC/Index Sync Verification

No BC/index changes in fix-burst 18 (code+tests only). Verified:
- BC-2.11.004 unchanged (no new version bump needed — test adds coverage for existing LOW-002 text)
- BC-INDEX count unchanged
- STORY-INDEX count unchanged
- ARCH-INDEX unchanged
- error-taxonomy unchanged

No propagation sweep needed.

---

## Convergence Assessment

**Trajectory (pass 23 on frozen 4e9d3f96):** 4 findings (1 LOW + 3 OBS) — streak RESET 0/3

**Cascade tally at FB-18 close:** 23 passes / 18 fix-bursts.

**New frozen HEAD after FB-18:** 989588b7 (LOCAL-ONLY NOT pushed).

**NEXT:** LOCAL pass 24 on frozen 989588b7 (streak 0/3 on new frozen HEAD).
