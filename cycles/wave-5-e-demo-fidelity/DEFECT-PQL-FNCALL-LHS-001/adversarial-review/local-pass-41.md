---
pass: 41
lane: DEFECT-PQL-FNCALL-LHS-001
frozen_head: 5e4c7ccb
date: 2026-07-14
authored_by: orchestrator-relay
clean_strict: true
clean_pr_merge: true
finding_count: 0
streak_before: 0/3
streak_after: 1/3
status: CLEAN
fix_burst: null
fix_burst_head_unchanged: null
fix_burst_spec_only: null
fix_burst_bc: null
---

# LOCAL Adversary Pass 41 — DEFECT-PQL-FNCALL-LHS-001

**Frozen HEAD: 5e4c7ccb** (fix/DEFECT-PQL-FNCALL-LHS-001; LOCAL-ONLY; unchanged from pass-40)
**CLEAN(strict): YES** (zero findings of any severity)
**CLEAN(PR-merge): YES** (zero CRIT + zero HIGH + zero MED)
**Streak: 1/3** (ADVANCES from 0/3 — first consecutive CLEAN(strict) pass; BC-5.39.001 streak-advance rule)

---

## Pass-40 Closure Re-Verification

Pass-40 had 1 LOW finding: F-PQLFN-P40-LOW-001 — BC-2.11.019 v1.18 terminal Implementation note scoped the walker to two positions while the same bullet enumerated three-position and seven-position coverage. **CLOSED in fix-burst-31 (BC-2.11.019 v1.18→v1.19 PO pre-edit; HEAD 5e4c7ccb UNCHANGED).**

Pass-41 closure re-verification confirms:
- BC-2.11.019 at v1.19: terminal Implementation note reworded to "ALL SEVEN predicate positions per ADR-048 §D.7.1"; incremental-coverage attribution present (positions 1–2 by original fix, positions 3–5 by OD-5, position 6 by OD-6 §D.7.5, position 7 by OD-7 §D.7.6). Internal consistency with §(a) three-position and §(b) seven-position enumerations confirmed.
- grep for "pipe | where and filter-mode" in Implementation note context: zero hits at v1.19.
- All prior closures from passes 37–39 remain stable at 5e4c7ccb: E-QUERY-042 arm-4 fn-call LHS gate at materialization boundary, push-down `collect_equality_exprs` non-Field-LHS rejection, span-shift explicit-enumeration compile-protection (all seven spans enumerated, any new span = compile error), OD-5/OD-6/OD-7 `FuncCall::Scalar` field presence confirmed, `Lit::Null` and `Lit::Timestamp` GROUP BY/ORDER BY arms (ADR-052 §D4 v1.11 arms 6+7), 1653/1653 prism-query GREEN state.

---

## Findings

**ZERO findings.** No CRIT, HIGH, MED, LOW, OBS, or PROCESS-GAP issues identified at frozen HEAD 5e4c7ccb.

---

## SAP-1 Result

**PASS.** `event_type =` emission sweep at frozen 5e4c7ccb: 232 total sites across `crates/`; 31 matches in `crates/prism-query/` (the primary scope of this defect lane). All `event_type` values map to catalogued rows in BC-2.16.002 §Postconditions Canonical Structured Event Catalog with full field schema, audit role, and recurrence policy. Zero orphans (emission site without catalog row). Fix-burst-31 (spec-only BC-2.11.019 doc correction) introduced no net-new `event_type =` emissions in `crates/`.

---

## Positive Verifications (Pass 41)

- **E-QUERY-042 arm-4 fn-call LHS orthogonal gate path verified (no overlap with fn-call gate):** The materialization-boundary E-QUERY-042 gate (arms 1–7 per ADR-052 §D4 v1.11) and the `collect_unknown_scalar_offsets_from_predicate` walk (E-QUERY-039 gate) operate on orthogonal code paths at 5e4c7ccb. E-QUERY-042 arm-4 rejects `Literal::FuncCall`-class expressions in GROUP BY / ORDER BY materialization position; the predicate walk rejects fn-call nodes in `| where` / filter-mode / aggregate-predicate positions. No shared state; no ordering dependency between the two gates. Both gates can fire independently on the same query if it contains multiple violations.

- **Push-down `collect_equality_exprs` non-Field-LHS rejection verified:** The DataFusion push-down path's `collect_equality_exprs` function at 5e4c7ccb rejects any comparison node whose LHS is not `Expr::Column` (the field-reference form). `Expr::ScalarFunction`-on-LHS expressions are silently passed to the runtime path rather than pushed down. This is the correct behavior (push-down is an optimization; runtime handles all expressions correctly). No E-QUERY-039 interaction; the predicate walk operates at the PrismQL AST level, not the DataFusion physical plan level.

- **Span-shift explicit-enumeration compile-protection confirmed (all seven spans present; FuncCall::Scalar span field):** The span-shift match in the `rewrite_predicate_spans` function at 5e4c7ccb enumerates all seven `Predicate` variants explicitly with no wildcard arm. `FuncCall::Scalar` carries a `span` field that participates in the span-shift rewrite; the field is present in the enum variant at 5e4c7ccb and matches the BC-2.11.019 §Postconditions span-shift coverage expectation. The `#[non_exhaustive]` attribute on `FuncCall` is NOT present on the enum itself (internal type; not part of the `prism-query` public API surface — `#[non_exhaustive]` discipline applies only to types in the public API boundary per CLAUDE.md); the explicit-match compile-protection is sufficient.

- **Semantic anchoring confirmed (SS-11, CAP-015, anchor_stories):** BC-2.11.019 §Preconditions and §Postconditions cite SS-11 (PrismQL Query Engine subsystem, ARCH-INDEX.md), CAP-015 (unknown-infusion-UDF detection capability, domain-spec-L2.md), and `anchor_stories: [S-PRISMQL-GRAMMAR-REMEDIATION-001, DEFECT-PQL-FNCALL-LHS-001]`. ARCH-INDEX.md SS-11 row confirmed present at current factory-artifacts HEAD; domain-spec-L2.md CAP-015 row confirmed present; both anchor stories confirmed in STORY-INDEX.md. Semantic citation chain coherent; no stale or missing anchors.

- **BC-INDEX / ARCH-INDEX row currency confirmed for this defect lane:** BC-INDEX.md BC-2.11.019 row at current factory-artifacts HEAD (v8.20 as of D-1759 burst): Status cell reads `draft — v1.19 (2026-07-14: ...)` after F-PQLFN-P40-LOW-001 closure. ARCH-INDEX.md SS-11 row for the PrismQL Query Engine subsystem references no stale BC versions that conflict with v1.19. No index drift introduced by fix-burst-31.

- **`#[non_exhaustive]` discipline — FuncCall::Scalar span field does not require `#[non_exhaustive]`:** `FuncCall` is an internal prism-query AST type with no public API exposure (not re-exported from `crates/prism-query/src/lib.rs`). CLAUDE.md §`#[non_exhaustive]` discipline applies to "public TOML-deserialized types and pub-API surface types." `FuncCall` does not meet either criterion. The compile-fail gate (`tests/external/non-exhaustive-violation/`) enforces the 91-type EXPECTED count for the public API boundary; `FuncCall` is not a member of that set. No `#[non_exhaustive]` addition required or appropriate.

- **Closed-item spot re-verification — F-PQLFN-P38-MED-001 + F-PQLFN-P40-LOW-001 confirmed stable:** F-PQLFN-P38-MED-001 (arm-4 E-QUERY-042 Literal::FuncCall LHS gate) closed in fix-burst-30; code confirmed live at 5e4c7ccb. F-PQLFN-P40-LOW-001 (BC-2.11.019 v1.18 terminal Implementation note stale scope) closed in fix-burst-31; BC v1.19 confirmed clean at factory-artifacts HEAD. Both closures stable; no regression.

---

## Status

```
CLEAN — pass 41 complete. Zero findings.

CASCADE TALLY: 41 passes / 31 fix-bursts (pass-42 in flight)

STREAK: 1/3 (ADVANCES from 0/3 — pass-41 CLEAN(strict); BC-5.39.001 streak-advance rule)
DRIFT-ORCH-PRLEVEL-PUSH-001: feature branch fix/DEFECT-PQL-FNCALL-LHS-001 is LOCAL-ONLY;
no push since 5e4c7ccb; frozen HEAD unchanged; streak gates on unchanged frozen HEAD.

FINDINGS BREAKDOWN:
  Total: 0

CLEAN(strict): YES (zero findings of any severity)
CLEAN(PR-merge): YES (zero CRIT+HIGH+MED)

NEXT ACTION: LOCAL adversary pass 42 on frozen 5e4c7ccb (streak 1/3; feature HEAD UNCHANGED — pass-42 in flight per D-1759 burst)
```
