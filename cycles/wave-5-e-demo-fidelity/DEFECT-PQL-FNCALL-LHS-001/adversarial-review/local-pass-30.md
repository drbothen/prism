---
pass: 30
lane: DEFECT-PQL-FNCALL-LHS-001
frozen_head: 3e482e41
date: 2026-07-14
adversary: vsdd-factory:adversary
clean_strict: true
clean_pr_merge: true
finding_count: 0
streak_before: 1/3
streak_after: 2/3
---

# LOCAL Adversary Pass 30 — DEFECT-PQL-FNCALL-LHS-001

**Frozen HEAD:** 3e482e41 (LOCAL-ONLY NOT pushed)
**CLEAN(strict):** YES — ZERO findings
**CLEAN(PR-merge):** YES — ZERO findings
**Streak:** ADVANCES 1/3 → 2/3 on frozen 3e482e41

---

## Findings

**None.** Zero findings of any severity (CRIT, HIGH, MED, LOW, OBS, PROCESS-GAP).

---

## SAP-1 Check (Tracing Emission Catalog)

`rg 'event_type\s*=' crates/ --type rust` — zero net-new `event_type` values introduced relative to frozen HEAD 3e482e41.
**SAP-1: CLEAN**

---

## Policy Rubric — Full Phase A + Phase C Verification

### Phase A: Grammar alternatives (compare_op) — byte-matched

All 7 compare_op alternatives byte-matched against source:
- `<` (Lt), `>` (Gt), `<=` (Le), `>=` (Ge), `!=` / `<>` (Ne), `=` / `==` (Eq alias)
- fn_call_comparison gate correctly covers all 7 alternatives
- ADR-048 §D.7 aggregate gate correct and source-verified

### Phase C: Reserved keyword gate — byte-matched

20 RESERVED_KEYWORDS case-insensitive list byte-matched against source:
NOT/AND/OR/IN/IIN/IEQ/INE/IS/BETWEEN/LIKE/CIDR/MATCHES/HAS/MISSING/CONTAINS/ICONTAINS/STARTSWITH/ISTARTSWITH/ENDSWITH/IENDSWITH [narrative-only correction per F-PQLFN-P46-MED-001; original list was not derived from grep]

### Phase C: Non-compose productions — Expr walker enumeration

14 non-compose Expr production families verified exhaustive via Expr walker enumeration — no grammar surface admits function-call LHS outside the gated paths.

### Phase C: 3-Rewriter parity (OOS-001 closure verification)

Fix-burst-22 added `rewrite_temporal_literal_in_pipe_key_position` to the SqlPipe stage-error path (BEFORE the offset shift, stage-relative). All 3 rewriters present in SqlPipe stage-error path. **Parity: PASS.**

### E-QUERY-042 templates — byte-verbatim including em-dash codepoint (U+2014)

E-QUERY-042 error message templates verified byte-verbatim including the em-dash (U+2014) codepoint — matches spec canonical form exactly.

### DML WHERE + 6-position gate coverage

All 6 gate positions verified:
1. SqlWhere (SQL WHERE clause)
2. SqlHaving (SQL HAVING clause)
3. SqlJoin ON condition
4. SqlPipe stage-error path (stage-relative, BEFORE offset shift)
5. DML WHERE (build_delete_parser / build_update_parser)
6. Filter-mode SELECT

**All 6 positions: PASS.**

---

## Workspace Test Count

PQL @3e482e41 = 5568 (all prism-query 1632/1632; just check 5568/5568 GREEN; non-exhaustive 91/91). Unchanged from pass-29.

---

## Disposition

- CLEAN(strict): YES
- CLEAN(PR-merge): YES
- Streak: **ADVANCES 1/3 → 2/3 on frozen 3e482e41**
- Frozen-HEAD rule (DRIFT-ORCH-PRLEVEL-PUSH-001): NO commits or pushes to this branch until streak reaches 3/3
- Novelty: LOW
- Next: LOCAL pass 31 on frozen 3e482e41
