---
pass: 40
lane: DEFECT-PQL-FNCALL-LHS-001
frozen_head: 5e4c7ccb
date: 2026-07-14
authored_by: orchestrator-relay
clean_strict: false
clean_pr_merge: true
finding_count: 1
streak_before: 1/3
streak_after: 0/3
status: CLOSED
fix_burst: 31
fix_burst_head_unchanged: true
fix_burst_spec_only: true
fix_burst_bc: BC-2.11.019-v1.19
---

# LOCAL Adversary Pass 40 — DEFECT-PQL-FNCALL-LHS-001

**Frozen HEAD: 5e4c7ccb** (fix/DEFECT-PQL-FNCALL-LHS-001; LOCAL-ONLY; unchanged from pass-39)
**CLEAN(strict): NO** (1 finding: 1 LOW)
**CLEAN(PR-merge): YES** (0 CRIT + 0 HIGH + 0 MED open; 1 LOW closed within fix-burst-31)
**Streak: 0/3** (RESET from 1/3 — pass-40 NOT CLEAN(strict); any non-CLEAN(strict) pass resets the streak)

---

## Findings

### F-PQLFN-P40-LOW-001 [LOW][documentation-drift] — CLOSED fix-burst-31 (BC-2.11.019 v1.18→v1.19 PO pre-edit; HEAD 5e4c7ccb UNCHANGED)

**Severity:** LOW
**Classification:** documentation-drift — BC-2.11.019 §Postconditions predicate fn-call positions bullet (the long §Postconditions paragraph beginning "Post-DEFECT-PQL-FNCALL-LHS-001 predicate fn-call positions") contained a terminal Implementation note that restricted the `collect_unknown_scalar_offsets_from_predicate` walker descent scope to "pipe | where and filter-mode" — the initial-fix two-position scope — while the same bullet immediately above that sentence enumerated three-position (§(a)) and seven-position (§(b)) coverage. This internal inconsistency within the same bullet was a documentation-drift artifact: the terminal sentence was written at the time of the original fix (positions 1–2 only) and was not updated as OD-5, OD-6 (§D.7.5), and OD-7 (§D.7.6) incrementally extended coverage to all seven positions.
**Status:** CLOSED — PO pre-edited BC-2.11.019 v1.18→v1.19 (uncommitted at time of finding; file at `.factory/specs/behavioral-contracts/BC-2.11.019-e-query-039-enrich-udf-not-found.md`). Terminal Implementation note sentence reworded to ALL SEVEN positions with incremental-coverage attribution. Feature HEAD 5e4c7ccb UNCHANGED (spec-only closure; no code change; no feature-branch commit).

**Finding detail:** At frozen 5e4c7ccb, the §Postconditions "Post-DEFECT-PQL-FNCALL-LHS-001 predicate fn-call positions" bullet ends with an Implementation note that reads (approximate v1.18 text):

> "**Implementation note:** the `collect_unknown_scalar_offsets_from_predicate` ... walk function must descend into `Predicate::Compare { lhs: Expr::FuncCall(FuncCall::Scalar { func: ScalarFunc::Unknown(name), .. }), .. }` in pipe `| where` stage predicates and filter-mode root predicates..."

This terminal sentence scoped the walker to exactly two positions (pipe `| where` and filter-mode) — the scope at the time of the original grammar-extension fix (DEFECT-PQL-FNCALL-LHS-001 initial scope). However:

1. The same bullet's preceding text (§(a)) enumerates positions (i)–(iii) (pipe `| where`, filter-mode, SqlPipe `| where`) = THREE positions — OD-5 extension.
2. The same bullet's §(b) enumeration lists seven positions — OD-5 + OD-6 (§D.7.5) + OD-7 (§D.7.6) coverage complete.
3. Pass-39's verification walk confirmed ALL SEVEN positions are live at 5e4c7ccb.

The terminal sentence was therefore internally inconsistent with the enumerations immediately above it in the same bullet, and inconsistent with the actual implementation.

**Sibling grep (TD-VSDD-060):** grep for "pipe | where and filter-mode" in walker-scope context at 5e4c7ccb: one hit — the terminal Implementation note (the finding site). Also swept: "pipe-mode only", "filter-mode only", "two positions" in `check_enrich_udf_availability` context — zero hits. The five-position Error Cases §E-QUERY-039 Condition column enumeration (approximately at line 148 of the BC file) was inspected: it enumerates (i)–(v) positions correctly (added incrementally by v1.12–v1.17); no stale two-position text present. Sibling grep CLEAN.

**Zero story pins:** grep for `BC-2.11.019` in `.factory/stories/` at current factory state: existing story citations were verified at v1.18 before this finding was authored. No story file contains the stale "pipe | where and filter-mode" Implementation note text. Story pins for BC-2.11.019 cite the BC identifier only (not the two-position sentence); no story update required for v1.18→v1.19.

**Severity rationale:** LOW because: (1) no production code path is wrong — the implementation correctly serves all seven positions at 5e4c7ccb; (2) the fix is a documentation-only sentence correction within the same bullet that already contained the correct enumeration; (3) the inconsistency is within a single bullet (within-bullet self-contradiction class), not a cross-document drift; (4) the seven-position enumeration (§(a) three-position + §(b) seven-position) provides the correct specification for any implementer reading the BC — the terminal sentence was the only inconsistent site.

**Closure evidence (fix-burst-31 — PO pre-edit BC-2.11.019 v1.18→v1.19; HEAD 5e4c7ccb UNCHANGED):**

1. **BC-2.11.019 frontmatter updated:** `version: "1.19"`, `modified: 2026-07-14`. File at `.factory/specs/behavioral-contracts/BC-2.11.019-e-query-039-enrich-udf-not-found.md` carries the v1.19 edits (uncommitted in factory-artifacts; to be committed in this D-1758 burst).

2. **Terminal Implementation note reworded:** Sentence now reads (v1.19 text): "the `collect_unknown_scalar_offsets_from_predicate` ... walk function must descend into `Predicate::Compare { lhs: Expr::FuncCall(FuncCall::Scalar { func: ScalarFunc::Unknown(name), .. }), .. }` in ALL SEVEN predicate positions per ADR-048 §D.7.1 (the walker is context-agnostic; each gate arm passes it the position's predicate). This walk coverage was added incrementally: positions 1–2 by the original grammar-extension fix, positions 3–5 by OD-5, position 6 by OD-6 (§D.7.5), position 7 by OD-7 (§D.7.6); prior to the original fix, none of these inputs reached the walker." Internally consistent with the same bullet's §(a) three-position and §(b) seven-position enumerations.

3. **Sibling grep at v1.19:** grep for "pipe | where and filter-mode" in Implementation note context: zero hits. Internal consistency within the bullet: §(a) three-position + §(b) seven-position + terminal Implementation note "ALL SEVEN" — consistent throughout. Five-position Error Cases enumeration (line ~148) was re-scanned: enumerates (i)–(v) positions correctly; not-stale.

4. **Feature HEAD UNCHANGED:** BC-2.11.019 is a `.factory/` spec artifact. No code change to `fix/DEFECT-PQL-FNCALL-LHS-001` feature branch. `5e4c7ccb` remains the frozen head for all subsequent passes.

5. **No test change required:** finding is documentation-only. The implementation correctness was verified at pass-39 (all seven positions confirmed live). 1653/1653 prism-query GREEN state from 5e4c7ccb is unchanged.

---

## SAP-1 Result

**PASS.** `crates/` `event_type =` emission sweep at frozen 5e4c7ccb: 174 unique `event_type` values found (same count as pass-39); all 174 catalogued in BC-2.16.002 §Postconditions. Fix-burst-31 (spec-only BC-2.11.019 doc correction) introduces zero net-new `event_type =` emissions. No catalog update required.

---

## Positive Verifications (Pass 40)

- **`fn_call_comparison` identifier/keyword gates verified vs BC LOW-004/005/006 verbatim:** The `fn_call_comparison` Chumsky parser combinator at 5e4c7ccb rejects PrismQL reserved identifiers, aggregate-class names (via DataFusion built-in registry check superseding the retired `AGGREGATE_FUNC_NAMES` blocklist), and temporal-class keywords at the fn-call-name position. BC-2.11.019 §LOW-004, §LOW-005, §LOW-006 error-condition text verified consistent with implementation.

- **Gate ordering 037→038→039 + aggregate-before-temporal verified:** Plan-time gate sequence at 5e4c7ccb: E-QUERY-037 (table) → E-QUERY-038 (column) → E-QUERY-039 (infusion); confirmed by `execute_inner` gate call order. DataFusion aggregate built-in check runs before temporal check in `check_enrich_udf_availability`'s DataFusion exclusion branch; correct ordering preserved.

- **`did_you_mean` determinism verified (sort+dedup, tuple tie-break, 128-byte Levenshtein cap SEC-002):** The `did_you_mean` computation at 5e4c7ccb sorts candidate infusion names lexicographically, deduplicates, uses tuple tie-breaking for equal-distance candidates, and caps the Levenshtein computation at 128 bytes per query string per SEC-002 bound. Deterministic under fixed-registry inputs.

- **Span-shift logic verified (head byte-0-anchored, stage-relative rewrites):** Error span offsets in E-QUERY-039 payloads are computed relative to the stage that produced the fn-call node; head query spans are byte-0-anchored; stage-relative rewrites applied correctly for pipe-mode stages. Confirmed at 5e4c7ccb.

- **`Expr::InSubquery` fail-open documented §OBS-001 accepted:** BC-2.11.019 §Gate Scope Boundaries §OBS-001 documents the subquery fail-open behavior; confirmed present at v1.18 (carry-forward into v1.19 without modification). No production code path produces E-INT-001 from an InSubquery-nested unknown UDF in the current DataFusion path. ACCEPTED-NO-ACTION status unchanged.

- **Six-callers claim code-anchored:** The claim that `check_enrich_udf_availability` has six call sites in the production code path was verified code-anchored at 5e4c7ccb. grep for `check_enrich_udf_availability` in `crates/prism-spec-engine/src/`: confirmed call sites enumerated.

---

## Status

```
CLOSED — pass 40 complete. Fix-burst-31 COMPLETE (spec-only; HEAD UNCHANGED).

CASCADE TALLY: 40 passes / 31 fix-bursts

STREAK: 0/3 (RESET — pass-40 NOT CLEAN(strict); any non-clean pass resets streak regardless of fix-burst type)

FINDINGS BREAKDOWN:
  LOW:  1 (F-PQLFN-P40-LOW-001 — CLOSED fix-burst-31 BC-2.11.019 v1.19 PO pre-edit; HEAD UNCHANGED)

CLEAN(strict): NO (1 LOW — not zero-finding; streak resets)
CLEAN(PR-merge): YES (0 CRIT+HIGH+MED open at pass close; LOW-001 closed)

NEXT ACTION: LOCAL adversary pass 41 on frozen 5e4c7ccb (streak 0/3; feature HEAD UNCHANGED — fix-burst-31 was spec-only)
```
