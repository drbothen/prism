---
document_type: adversarial-review
scope: LOCAL
fix_pr: DEFECT-PQL-FNCALL-LHS-001
passes: [21]
feature_head_at_review: 28d9600f
date: 2026-07-13
clean_strict: false
clean_pr_merge: true
finding_counts:
  total: 5
  crit: 0
  high: 0
  med: 0
  low: 1
  obs: 4
  process_gap: 0
code_behavior_defects: 0
streak_after: 0/3
convergence: NOT_CONVERGED
authored_by: orchestrator-relay
---

# LOCAL Adversary Pass 21 — DEFECT-PQL-FNCALL-LHS-001

---

## Pass 21 (frozen 28d9600f; fresh-context adversary; fn-call-LHS PrismQL grammar + ADR-048 §D.7 aggregate gate; streak candidate 2/3 — RESET to 0/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=YES

**Findings:** 5 total (0 CRIT / 0 HIGH / 0 MED / 1 LOW / 4 OBS / 0 PROCESS-GAP)

**STREAK RESET: 1/3 → 0/3** (F-PQLFN-P21-LOW-001 is non-OBS; BC-5.39.001 requires ZERO findings of any severity for streak advancement)

**Code HEAD at review:** 28d9600f (frozen; 21 commits over develop@5f1b5771; LOCAL-ONLY; prism-query 1616/1616; just check FULL WORKSPACE 5552/5552 GREEN; non-exhaustive 91/91)

**CLEAN(strict):** NO — 1 LOW + 4 OBS findings present; streak advancement criterion NOT satisfied

**CLEAN(PR-merge):** YES — ZERO CRIT + HIGH + MED findings; PR-merge gate satisfied

**SAP-1 (Structured Event Catalog — BC-2.16.002):** CLEAN — no new `event_type =` assignments at @28d9600f in the fn-call-LHS grammar/aggregate-gate surfaces

---

## Finding Register

### F-PQLFN-P21-LOW-001 [LOW] [security-defense-in-depth] Unknown-scalar fn-name emitted verbatim into SQL + misleading parity docstring + `_ => "func"` silent catch-all

**Severity:** LOW

**Classification:** security-defense-in-depth (not an exploitable injection path — grammar `fn_name` production rejects non-identifier inputs at parse time; defense-in-depth gap at the expr_to_sql emission boundary)

**Location:** `crates/prism-query/src/expr_to_sql.rs` — `ScalarFunc::Unknown(name)` arm in the function-name emitter

**Description:**
At @28d9600f the `ScalarFunc::Unknown(name)` arm emits `name` verbatim into the SQL string without any charset validation. While the grammar `fn_name` production (`[A-Za-z_][A-Za-z0-9_]*`) blocks hostile input at parse time, there is no charset re-validation at the emission boundary. A `ScalarFunc::Unknown` variant constructed programmatically (e.g., via a future API surface or test helper) could carry an unsafe name string that bypasses the grammar gate.

Additionally, the docstring for the `Unknown` arm contains a misleading "parity with normalize_func_call" claim that does not accurately reflect the emission behavior, creating a documentation-accuracy hazard (AD-017 / POL-24 class).

Finally, the `_ => "func"` wildcard catch-all in the fn-name match arm silently emits the string `"func"` for any unrecognized `ScalarFunc` variant, hiding new-variant introduction bugs at the emission layer rather than surfacing them as compile-time exhaustiveness errors.

**Fix required:**
1. Add charset validation `[A-Za-z_][A-Za-z0-9_]*` on `name` before SQL emission in the `ScalarFunc::Unknown` arm; return `Err(QueryExecutionFailed)` on unsafe input.
2. Replace the `_ => "func"` wildcard with an explicit 7-variant enumeration so rustc enforces exhaustiveness at compile time.
3. Correct the docstring to accurately describe the emission behavior.

---

### F-PQLFN-P21-OBS-001 [OBS] BC-2.11.004 LOW-002 documented only IEQ/IIN/INE non-composability; grammar rejects 14 predicate-operator families

**Severity:** OBS

**Classification:** spec-coverage gap (BC-2.11.004 §Error Cases LOW-002 is narrower than the grammar constraint)

**Description:**
BC-2.11.004 LOW-002 documents the non-composability of fn-call-LHS specifically for IEQ/IIN/INE. The grammar filter_parser `atom` choice rejects fn-call-LHS across all 14 predicate-operator families (EQ, NE, LT, LE, GT, GE, IN, NIN, IEQ, IIN, INE, IS, IS NOT, LIKE). Documenting only 3 of the 14 families creates a spec-coverage gap that could mislead future implementers into thinking fn-call-RHS is gated only for case-insensitive operators.

**Fix required:** Extend BC-2.11.004 LOW-002 to enumerate all 14 non-composable predicate-operator production families verified against filter_parser.rs `atom` choice.

---

### F-PQLFN-P21-OBS-002 [OBS] fn-call-RHS / two-sided fn-call scope limits unspecced

**Severity:** OBS

**Classification:** spec-coverage gap (grammar scope limit not documented)

**Description:**
BC-2.11.004 does not document that fn-call expressions are not admitted on the RHS of a predicate comparison (i.e., `severity = lower('high')` is not valid PrismQL) nor that two-sided fn-call expressions (`lower(a) = lower(b)`) are rejected by the grammar. These constraints exist in the grammar but lack spec anchors in BC-2.11.004, leaving the fn-call scope limits unspecified for reviewers and future implementers.

**Fix required:** Add LOW-005 to BC-2.11.004 §Error Cases documenting: fn-call expressions are NOT admitted on the RHS of predicate comparisons; two-sided fn-call predicate forms are rejected by the grammar at all 6 call surfaces (Pipe WHERE / Filter / SQL WHERE / SqlPipe head WHERE / SqlPipe where stage / DML WHERE).

---

### F-PQLFN-P21-OBS-003 [OBS] Aggregate gate hardcoded offset: 0 vs ADR-048 §D.7.2 truthful-offset template

**Severity:** OBS

**Classification:** spec-code alignment gap (ADR-048 §D.7.2 requires byte-offset truthfulness)

**Description:**
The aggregate gate (`E-QUERY-045` or equivalent) for fn-call-LHS uses a hardcoded `offset: 0` in the `QueryParseFailed` error payload. ADR-048 §D.7.2 truthful-offset template requires the byte-offset of the offending token to be propagated into error payloads. The `FuncCall::Scalar` AST variant at @28d9600f does not carry a `Span` field, preventing offset truthfulness.

**Fix required:** Add a `span: Span` field to the `FuncCall::Scalar` AST variant; capture the byte-offset via `map_with e.span()` in the parser; propagate the real offset into the aggregate gate error payload via `collect_unknown_scalar_offsets_from_{expr,predicate}` returning `Vec<(String, usize)>`.

---

### F-PQLFN-P21-OBS-004 [OBS] No two-sided fn-call negative coverage

**Severity:** OBS

**Classification:** test-coverage gap (negative locks for fn-call-RHS and two-sided forms absent)

**Description:**
The test suite at @28d9600f has no negative locks exercising fn-call-RHS rejection (`severity = lower('high')`) or two-sided fn-call rejection (`lower(a) = lower(b)`) across all 6 call surfaces. Without these locks, a future grammar regression that accidentally admits fn-call-RHS would not be caught by the test suite.

**Fix required:** Add 12 negative locks covering fn-call-RHS and two-sided fn-call forms across all 6 surfaces (Pipe WHERE / Filter / SQL WHERE / SqlPipe head WHERE / SqlPipe where stage / DML WHERE), with diagnostic-first assertion ordering and anchored to BC-2.11.004 LOW-005.

---

## Fix-Burst 16 Closure Audit (OBS-001 + OBS-002 + LOW-001 + OBS-003 + OBS-004)

Per the Single-Commit Burst Protocol (TD-VSDD-053), all 5 findings above were closed in fix-burst 16 across commits 28d9600f→9f510c1a (LOCAL-ONLY branch):

**OBS-001 + OBS-002 (BC-2.11.004 v1.40→v1.41) @36eadae2 (factory-artifacts, PO-committed):**
- LOW-002 §Error Cases extended to enumerate all 14 non-composable predicate-operator production families (verified against filter_parser.rs atom choice)
- New LOW-005 §Error Cases added: fn-call expressions not admitted on RHS; two-sided fn-call forms rejected at all 6 surfaces
- 2 test vectors added to BC-2.11.004
- POL-23 sweep: S-PRISMQL-CASE-INSENSITIVE-001 v1.65→v1.66 (4 live BC-2.11.004 v1.40→v1.41 pin sites updated)

**LOW-001 @40de2316:**
- `ScalarFunc::Unknown(name)`: charset validation added (`[A-Za-z_][A-Za-z0-9_]*`); `Err(QueryExecutionFailed)` returned on unsafe input; `_ => "func"` wildcard replaced with explicit 7-variant enumeration (compile-time exhaustiveness); docstring corrected
- 3 new tests including hostile-injection-name lock

**OBS-003 @40de2316:**
- `span: Span` field added to `FuncCall::Scalar` AST variant
- Parser captures byte-offset via `map_with e.span()`
- `collect_unknown_scalar_offsets_from_{expr,predicate}` return `Vec<(String, usize)>`
- Aggregate gate threads real offset into `QueryParseFailed` (was hardcoded 0)
- TD-VSDD-060 sweep: 20+ construction/match sites updated
- 2 offset-truthfulness tests added

**OBS-004 @9f510c1a:**
- 12 GREEN negative locks (fn-call-RHS + two-sided) across all 6 surfaces
- Diagnostic-first assertion ordering
- Anchored to BC-2.11.004 LOW-005

**Result after FB-16:** prism-query 1616/1616, FULL WORKSPACE just check 5552/5552 GREEN. NEW FROZEN HEAD 9f510c1a (LOCAL-ONLY). Streak RESET 0/3 per DRIFT-ORCH-PRLEVEL-PUSH-001 (FB-16 commits pushed to branch). NEXT: LOCAL pass 22 on frozen 9f510c1a.

---

## Standing Probe Results

**SAP-1 (Structured Event Catalog — BC-2.16.002):** CLEAN — grepped `event_type\s*=` across `crates/` workspace at frozen 28d9600f. Zero new `event_type` assignments in fn-call-LHS grammar or aggregate-gate surfaces at this HEAD. All emission sites verified against BC-2.16.002 §Postconditions catalog.

**SAP-2:** N/A — no sensor TOML spec modifications in this defect cascade.

**SID-1:** N/A — no `#[ignore]`'d tests driving spec-required behavior at @28d9600f.

---

## Convergence Assessment

**Trajectory (pass 21 on frozen 28d9600f):** streak candidate 2/3 — RESET 0/3 (1 LOW finding)

**Cascade tally at FB-16 close:** 21 passes / 16 fix-bursts.

**New frozen HEAD after FB-16:** 9f510c1a (LOCAL-ONLY).

**NEXT:** LOCAL pass 22 on frozen 9f510c1a (streak 0/3 on new frozen HEAD).
