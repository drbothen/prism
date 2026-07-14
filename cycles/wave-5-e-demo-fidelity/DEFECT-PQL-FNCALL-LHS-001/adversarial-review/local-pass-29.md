---
pass: 29
lane: DEFECT-PQL-FNCALL-LHS-001
frozen_head: 3e482e41
date: 2026-07-14
adversary: vsdd-factory:adversary
clean_strict: true
clean_pr_merge: true
finding_count: 0
streak_before: 0/3
streak_after: 1/3
---

# LOCAL Adversary Pass 29 — DEFECT-PQL-FNCALL-LHS-001

**Frozen HEAD:** 3e482e41 (LOCAL-ONLY NOT pushed)
**CLEAN(strict):** YES — ZERO findings
**CLEAN(PR-merge):** YES — ZERO findings
**Streak:** ADVANCES 0/3 → 1/3 on frozen 3e482e41

---

## Findings

**None.** Zero findings of any severity (CRIT, HIGH, MED, LOW, OBS, PROCESS-GAP).

---

## SAP-1 Check (Tracing Emission Catalog)

`rg 'event_type\s*=' crates/ --type rust` — zero net-new `event_type` values introduced relative to frozen HEAD 3e482e41.
**SAP-1: CLEAN**

---

## Policy Rubric — Full Phase A + Phase C Verification

### Phase A: Grammar alternatives (compare_op)

All 7 compare_op alternatives verified source-correct:
- `<` (Lt), `>` (Gt), `<=` (Le), `>=` (Ge), `!=` / `<>` (Ne), `=` / `==` (Eq alias)
- fn_call_comparison gate correctly covers all 7 alternatives
- ADR-048 §D.7 aggregate gate correct and source-verified

### Phase C: Reserved keyword gate + non-compose productions

- 20 RESERVED_KEYWORDS correctly enumerated in the keyword gate
- 14 non-compose productions verified exhaustive (no grammar surface admits function-call LHS outside the gated paths)
- 6 gate-position coverage: all 6 positions verified (SqlWhere, SqlHaving, SqlJoin ON, SqlPipe stage-error, DML WHERE, filter-mode SELECT)

### 3-Rewriter parity (OOS-001 closure verification)

Fix-burst-22 added `rewrite_temporal_literal_in_pipe_key_position` to the SqlPipe stage-error path (BEFORE the offset shift, stage-relative). Ordering documented inline. RED parity evidence: generic error → actionable ADR-052 §D4 message. Test `test_f_pqlfn_p28_oos_001_sqlpipe_sort_literal_parity` GREEN. All 3 rewriters now present in SqlPipe stage-error path. **Parity: PASS.**

### ADR-048 citation accuracy (LOW-001 closure verification)

Guard comment rewritten with honest citation (no pseudo-quoted phrases). ADR-048 language now correctly paraphrased, not fabricated. **Citation truth: PASS.**

### InSubquery rationale accuracy (LOW-002 closure verification)

False "SqlQuery is not an Expr" rationale replaced with grammar-unreachability rationale: InSubquery subqueries cannot contain a function-call LHS because the grammar does not currently admit that form. FUTURE-EXTENSION NOTE added: if grammar extends to allow this, no-op arms must be revisited. **Comment truth: PASS.**

### Workspace test count

PQL @3e482e41 = 5568 (all prism-query 1632/1632; just check 5568/5568 GREEN; non-exhaustive 91/91).

---

## Disposition

- CLEAN(strict): YES
- CLEAN(PR-merge): YES
- Streak: **ADVANCES 0/3 → 1/3 on frozen 3e482e41**
- Frozen-HEAD rule (DRIFT-ORCH-PRLEVEL-PUSH-001): NO commits or pushes to this branch until streak reaches 3/3
- Novelty: LOW
- Next: LOCAL pass 30 on frozen 3e482e41
