---
document_type: adversarial-review-cascade-summary
scope: LOCAL
defect: DEFECT-EQUERY042-GROUPBY-DEADARM-001
fix_branch: fix/equery042-groupby-deadarm
converged_at: "7db0b1ba"
date_converged: 2026-07-10
total_passes: 5
streak_passes: [3, 4, 5]
convergence: CONVERGED
authored_by: state-manager
---

# LOCAL Adversary Cascade Summary — DEFECT-EQUERY042-GROUPBY-DEADARM-001

**Defect:** `prism-query` `check_expr_temporal_pos` dead arm — `Literal::Timestamp` (RFC-3339
fast-path product of `check_temporal_literals`) in `GROUP BY`/`ORDER BY` position never
matched; only `Literal::RawTemporalLiteral` matched. Result: E-QUERY-042
`TemporalLiteralInvalidPosition` never fired for GroupBy/OrderBy with RFC-3339 date-time
strings — yielded "Internal error" instead of pedagogical structured error.

**Fix (code @f8bd5421):** `Literal::Timestamp` arm added to GroupBy+OrderBy branches in
`check_expr_temporal_pos` (ADR-052 §D4 arms 6/7). 4 RED gate tests in
`crates/prism-query/src/tests/temporal_typing_tests.rs` turned GREEN.

**BC-5.39.001 result:** LOCAL 3-CLEAN CONVERGED (passes 3/4/5 all CLEAN(strict) on frozen
HEAD 7db0b1ba). 5 passes total.

---

## 5-Pass Cascade Table

| Pass | Frozen HEAD | CLEAN(strict) | CLEAN(PR-merge) | Findings | Fix-burst HEAD | Streak |
|------|-------------|---------------|-----------------|----------|----------------|--------|
| 1 | f8bd5421 | NO | NO | 1 HIGH + 2 MED + 1 LOW (total 4) | f0fe1f9f | 0/3 |
| 2 | f0fe1f9f | NO | YES | 1 LOW + 1 OBS (total 2) | 7db0b1ba | 0/3 |
| 3 | 7db0b1ba | YES | YES | 0 | — | 1/3 |
| 4 | 7db0b1ba | YES | YES | 0 | — | 2/3 |
| 5 | 7db0b1ba | YES | YES | 0 | — | **3/3 CONVERGED** |

---

## Finding Summary

### Pass 1 (frozen f8bd5421)

| ID | Severity | Description | Resolution |
|----|----------|-------------|------------|
| F-EQ42-P1-001 | HIGH | ADR-052 §D4 arms 6/7 `Literal::Timestamp` co-trigger undocumented — spec did not describe that RFC-3339 fast-path `Literal::Timestamp` also triggers E-QUERY-042 in GroupBy/OrderBy position | ADR-052 v1.11 + error-taxonomy v2.37 + BC-2.11.021 v1.8 + BC-2.11.003 v1.12 + story v1.13 pin round (spec layer); 6 sibling call-site GREEN locks + 3 inject_now/grammar locks @f0fe1f9f (code layer) |
| F-EQ42-P1-002 | MED | (spec-propagation secondary finding) | Closed in spec layer above |
| F-EQ42-P1-003 | MED | (spec-propagation secondary finding) | Closed in spec layer above |
| F-EQ42-P1-004 | LOW | (doc-citation / pin finding) | Closed in fix-burst @f0fe1f9f |

### Pass 2 (frozen f0fe1f9f)

| ID | Severity | Description | Resolution |
|----|----------|-------------|------------|
| F-EQ42-P2-001 | LOW | Doc-citation sweep residual | Closed in fix-burst @7db0b1ba |
| F-EQ42-P2-002 | OBS | Minor prose clarity gap | Closed in fix-burst @7db0b1ba |

### Passes 3–5 (frozen 7db0b1ba)

Zero findings across all three passes. BC-5.39.001 3-CLEAN CONVERGED.

---

## Evidence at Convergence HEAD (7db0b1ba)

- prism-query: **1502/1502** tests GREEN
- Full workspace `just check`: **GREEN**
- Non-exhaustive gate: **89/89**
- Total lock tests (cascade): **15** (9 @f0fe1f9f + doc-citation sweep tests @7db0b1ba)

## Artifacts Modified

| Artifact | Version Bump | Change |
|----------|-------------|--------|
| ADR-052 | v1.10→v1.11 | Arms 6/7 `Literal::Timestamp` co-trigger documented + Behavior Reference Table + Red Gate rows |
| error-taxonomy | v2.36→v2.37 | GroupBy/OrderBy dual-trigger description; NonColumnLhsComparison RawTemporalLiteral-only clarification |
| BC-2.11.021 | v1.7→v1.8 | EC-11-021-019/020 + test vectors; `Literal::Timestamp` co-trigger arms |
| BC-2.11.003 | v1.11→v1.12 | POL-25 sweep catch; EC-11-003-008/009 + test vectors |
| story (DEFECT-EQUERY042-GROUPBY-DEADARM-001) | v1.12→v1.13 | Pin round: 34× ADR-052 §D4 v1.10→v1.11 + 4× error-taxonomy |

## Next Step

Push `fix/equery042-groupby-deadarm` @7db0b1ba → pr-manager 9-step fix-PR cycle →
PR-LEVEL cascade (BC-5.39.001 requires CLEAN(strict) ×3 at PR-LEVEL as well).
