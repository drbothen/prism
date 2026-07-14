---
document_type: adversarial-review
scope: LOCAL
fix_pr: DEFECT-PQL-FNCALL-LHS-001
passes: [25]
feature_head_at_review: b55c7708
date: 2026-07-14
clean_strict: true
clean_pr_merge: true
finding_counts:
  total: 0
  crit: 0
  high: 0
  med: 0
  low: 0
  obs: 0
  process_gap: 0
code_behavior_defects: 0
streak_after: 1/3
convergence: IN_PROGRESS
authored_by: orchestrator-relay
---

# LOCAL Adversary Pass 25 — DEFECT-PQL-FNCALL-LHS-001

---

## Pass 25 (frozen b55c7708; fresh-context adversary; fn-call-LHS PrismQL grammar + ADR-048 §D.7 aggregate gate + expr_to_sql FuncCall arm + SqlPipe stage span offset translation; LOCAL cascade; streak ADVANCES 0/3 → 1/3)

**Pass result:** CLEAN(strict)=YES, CLEAN(PR-merge)=YES

**Findings:** ZERO

---

## SAP-1 Verification

SAP-1 (tracing emission catalog completeness) PASS:
- `rg 'event_type\s*=' crates/ --type rust` executed: approximately 230 `event_type =` sites found across workspace
- All emissions pre-existing and catalogued in BC-2.16.002 §Postconditions Canonical Structured Event Catalog
- Zero new `event_type =` emissions introduced in this branch (fix/DEFECT-PQL-FNCALL-LHS-001 HEAD b55c7708)

---

## Phase A+C Verification Summary

Full policy rubric PASS. All verifications passed across both Phase A (grammar correctness) and Phase C (implementation correctness):

- **Grammar productions (fn-call-LHS rejection):** PrismQL grammar correctly rejects `fn_call_expr` in LHS positions across all 6 call surfaces (Pipe WHERE / Filter / SQL WHERE / SqlPipe head WHERE / SqlPipe where stage / DML WHERE). Grammar productions verified against `prism-query/src/filter_parser.rs` `atom` choice: fn-call not admitted as predicate LHS operand.

- **Both span walkers exhaustive:** `shift_scalar_spans_in_expr` (14-variant explicit enum; Compare/Logical/Not/BinaryOp/Unary/FuncCall/etc.; no wildcard) and `shift_scalar_spans_in_predicate` (14-variant explicit enum; Compare/Logical/Not recurse; 11 no-op arms with inline justification; wildcard removed by fix-burst-19 — F-PQLFN-P24-OBS-002 closure) are both fully explicit. New variant addition forces compile error symmetrically in both walkers.

- **Dual E-QUERY-001 forms byte-verified:** error-taxonomy v2.48 E-QUERY-001 entry documents both Form A (`PrismError::QueryParseFailed` Display `"query parse error at offset {offset}"`) and Form B (`prism-query ParseError` Display `"parse error at offset {offset}"`). Both forms byte-verified against `prism-core/src/error.rs` and `prism-query` parser error emission sites. F-PQLFN-P24-OBS-003 closure confirmed complete.

- **All named entities located:** FuncCall AST variant, shift_scalar_spans_in_stages, shift_scalar_spans_in_predicate, shift_scalar_spans_in_expr, aggregate gate (ADR-048 §D.7), expr_to_sql FuncCall arm — all present and correct in branch HEAD b55c7708.

- **FuncCall::Scalar docstring two-step chain:** docstring on `FuncCall::Scalar` AST variant correctly names `shift_scalar_spans_in_stages` as the load-bearing post-parse normalization function (F-PQLFN-P24-OBS-001 closure confirmed); F-PQLFN-P22-MED-001 origin cited.

- **ADR-048 §D.7 aggregate gate:** verified correct; FuncCall in aggregate argument position produces E-QUERY-001 parse error at plan time; no regression from span fixes.

- **expr_to_sql FuncCall arm:** verified present; maps `FuncCall::Scalar { name, args, .. }` to `Expr::Function`; no stale code paths.

- **No new high-severity patterns:** no new `unwrap()`/`expect()` in non-test paths, no new `println!`, no `native-tls` features, no retired enum variants, no production placeholder constructors.

---

## Frozen-HEAD Rule Confirmation

Per BC-5.39.001 (3-CLEAN convergence protocol) and DRIFT-ORCH-PRLEVEL-PUSH-001 (frozen-HEAD streak rule):

- Streak ADVANCES to **1/3** on frozen HEAD b55c7708
- NO commits or pushes to the branch are permitted until 3/3 achieved
- Next: LOCAL pass 26 dispatched on frozen b55c7708

---

## Streak Status

**1/3** on frozen HEAD b55c7708.

Pass-24 was CLEAN(PR-merge)=YES but CLEAN(strict)=NO (3 OBS). Fix-burst-19 closed all 3 OBS; HEAD advanced to b55c7708. This pass (25) is the first CLEAN(strict)=YES pass on the new frozen HEAD b55c7708 — streak advances from 0/3 to 1/3.

---

## Cascade Status

- Total passes: 25 (LOCAL cascade only; pre-push)
- Fix-bursts: 19 completed
- LOCAL 3-CLEAN streak: 1/3 on frozen b55c7708
- HEAD: b55c7708 (LOCAL-ONLY, NOT pushed)
- Next: LOCAL pass 26 on frozen b55c7708 (dispatched)
