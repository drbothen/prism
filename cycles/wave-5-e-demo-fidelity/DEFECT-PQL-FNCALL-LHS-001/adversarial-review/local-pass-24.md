---
document_type: adversarial-review
scope: LOCAL
fix_pr: DEFECT-PQL-FNCALL-LHS-001
passes: [24]
feature_head_at_review: 989588b7
date: 2026-07-14
clean_strict: false
clean_pr_merge: true
finding_counts:
  total: 3
  crit: 0
  high: 0
  med: 0
  low: 0
  obs: 3
  process_gap: 0
code_behavior_defects: 0
streak_after: 0/3
convergence: NOT_CONVERGED
authored_by: orchestrator-relay
---

# LOCAL Adversary Pass 24 — DEFECT-PQL-FNCALL-LHS-001

---

## Pass 24 (frozen 989588b7; fresh-context adversary; fn-call-LHS PrismQL grammar + ADR-048 §D.7 aggregate gate + SqlPipe span translation; streak candidate 1/3 — RESET to 0/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=YES

---

## Findings

### F-PQLFN-P24-OBS-001 [OBS][doc-truth]

**FuncCall::Scalar docstring elided two-step SqlPipe span normalization.**

The docstring on the `FuncCall::Scalar` AST variant describes the span field but does not capture the two-step normalization chain introduced by this defect fix:

1. The parser sets `Span::ZERO` for fn-calls outside WHERE/HAVING context.
2. `shift_scalar_spans_in_stages` post-processes spans for SqlPipe stages to yield truthful stage-relative offsets (the F-PQLFN-P22-MED-001 fix).

The pass-23 docstring correction (F-PQLFN-P23-OBS-001, fix-burst-18) improved the doc but did not name `shift_scalar_spans_in_stages` as the load-bearing post-parse normalization function. A reader tracing the correctness argument from the AST variant documentation would not discover the normalization chain without grepping.

**Fix:** Docstring updated to name `shift_scalar_spans_in_stages` explicitly and cite F-PQLFN-P22-MED-001 as the originating fix that made it load-bearing.

---

### F-PQLFN-P24-OBS-002 [OBS][wildcard-consistency]

**shift_scalar_spans_in_predicate kept wildcard while sibling expr walker was made fully explicit.**

After fix-burst-18 replaced the wildcard arm in `shift_scalar_spans_in_expr` with a fully explicit 13-variant enumeration (F-PQLFN-P23-OBS-002 closure), the sibling function `shift_scalar_spans_in_predicate` retained a `_ => ()` wildcard arm for its non-recursive variants.

This creates an asymmetric safety guarantee within the same module:
- New `Expr` variants added to the AST → compile error in the expr walker (desired; prevents silent no-op)
- New `Predicate` variants added to the AST → silently no-op in the predicate walker (undesired)

The fix-burst-18 sweep that made the expr walker explicit did not extend to the predicate walker (TD-VSDD-060 partial-scope miss).

**Fix:** `shift_scalar_spans_in_predicate` made fully explicit: `Compare`/`Logical`/`Not` arms recurse as before; 11 remaining arms enumerated explicitly with inline justification comments; wildcard removed; future variant addition forces compile error symmetrically with the expr walker.

---

### F-PQLFN-P24-OBS-003 [OBS][pre-existing-drift]

**Error-taxonomy E-QUERY-001 single-form template vs dual shipped Display forms.**

E-QUERY-001 in `error-taxonomy.md` documented a single canonical Display template. The shipped code emits two distinct forms:

- **Form A** (`PrismError::QueryParseFailed`): `"query parse error at offset {offset}"`
- **Form B** (`prism-query ParseError`): `"parse error at offset {offset}"`

Additionally, `"at position"` phrasing was observed in some Display outputs and was not acknowledged as a known variant or drift in the taxonomy. This is a pre-existing drift that predates the current defect branch (observable on develop@5f1b5771).

Neither form is incorrect; the drift is a documentation-completeness gap — the taxonomy claimed a single canonical form but two forms exist in production.

**Fix:** error-taxonomy v2.47→v2.48: E-QUERY-001 entry updated to acknowledge both Form A and Form B with their origin call sites; `"at position"` drift corrected (removed or attributed to a specific historical code path).

Companion: BC-2.11.017 v1.13→v1.14 — 3 stale normative claims that referenced the single-form template corrected.

---

## SAP-1 (Tracing Emission Catalog Completeness)

PASS — no new event_type emissions introduced in this branch. Existing emissions pre-catalogued in BC-2.16.002.

---

## Positive Verifications

- Pass-23 fix-burst-18 closures (OBS-001 doc, OBS-002 explicit enum, OBS-003 emitter arms) all verified structurally correct.
- shift_scalar_spans_in_stages named as load-bearing in pass-22 fix; F-PQLFN-P22-MED-001 offset-truthfulness verified.
- 14-variant predicate explicit enumeration (post fix-burst-18) structurally sound; no silent coverage gaps.

---

## Streak Status

**0/3** — fix-burst-19 commits pushed to branch; streak reset per DRIFT-ORCH-PRLEVEL-PUSH-001.

---

## Next Step

LOCAL pass 25 on frozen b55c7708 (new frozen HEAD post fix-burst-19).
