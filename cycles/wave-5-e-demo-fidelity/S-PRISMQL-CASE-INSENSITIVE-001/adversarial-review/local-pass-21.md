---
document_type: adversarial-review
scope: LOCAL
passes: [21]
story: S-PRISMQL-CASE-INSENSITIVE-001
feature_head: 257074af
fix_burst_head: 2de85b18
date: 2026-07-07
clean_strict: false
clean_pr_merge: true
finding_counts: {OBS: 1}
streak_after: 0/3
---

# LOCAL Adversary Pass 21 — S-PRISMQL-CASE-INSENSITIVE-001

---

## Pass 21 (frozen 257074af; delta 43 files vs develop@ea714d14)

**Pass result:** CLEAN(strict)=no (1 OBS), CLEAN(PR-merge)=YES (zero CRIT/HIGH/MED — first PR-merge-clean pass of this cascade)
**Findings:** 1 OBS (F-P21-OBS-001; closed fix-in-scope per production-grade default)
**Code HEAD at review:** 257074af
**Fix-burst HEAD (new frozen candidate for pass-22):** 2de85b18
**Fix-burst commits on feature branch:** 2de85b18 (implementer: explain.rs guard arms `case_insensitive: true => vec![]` on Compare + In; RG-073 RED→GREEN; 1406/1406 prism-query)
**LOCAL 3-CLEAN(strict) streak after pass-21:** 0/3 (OBS blocks strict; new frozen HEAD 2de85b18; HEAD reset per DRIFT-ORCH-PRLEVEL-PUSH-001)
**Next:** LOCAL pass-22 on frozen 2de85b18 — if CLEAN(strict), passes 23/24 on SAME frozen HEAD complete 3-CLEAN

---

## Finding Inventory

### F-P21-OBS-001 (OBS) — explain.rs `predicate_to_exprs` dropped `case_insensitive` flag for EXPLAIN classification

**Finding:** In `crates/prism-query/src/explain.rs`, the function `predicates_from_ast::predicate_to_exprs`
converts `Predicate` AST nodes to `Expr` values for use by `classify_predicates`. When converting
`Predicate::Compare { case_insensitive: true, .. }` (IEQ/INE) and `Predicate::In { case_insensitive: true, .. }`
(IIN), the conversion discarded the `case_insensitive` flag — producing an `Expr` that looks identical
to a plain equality/IN predicate. The downstream `classify_predicates` function then classified these
as potentially push-downable, which would cause EXPLAIN output to misreport IEQ/IIN predicates as
pushdown candidates.

This is latent today: `classify_predicates` is called at `explain.rs:1101` with an empty `ColumnSpec`
(`ColumnSpec::default()`), which means zero columns are registered as pushdownable, so the
misclassification has no observable effect on current EXPLAIN output. However, once `ColumnSpec` is
wired with real column metadata (the natural next step for EXPLAIN fidelity), the function would report
IEQ/IIN predicates as pushdownable — contradicting BC-2.11.024 §Postconditions pushdown contract and
the spec's explicit "not pushed down" guarantee.

The runtime push-down path (`collect_equality_exprs` in `pushdown.rs:299`) was already correct: it
respects `case_insensitive: true` and correctly excludes IEQ/IIN from the pushdown set. Only the
EXPLAIN classification path was affected.

**Severity:** OBS (latent; no current regression; would become a correctness defect once ColumnSpec
is wired; BC-2.11.024 contract correctness at risk on EXPLAIN path).

**Closure:** CLOSED fix-in-scope per production-grade default. Implementer @2de85b18 added guard
arms in `predicate_to_exprs`:
- `Predicate::Compare { case_insensitive: true, .. } => vec![]` (IEQ/INE: return empty — not
  classifiable as pushdownable)
- `Predicate::In { case_insensitive: true, .. } => vec![]` (IIN: same)

Red Gate test RG-073 `test_BC_2_11_024_f_p21_obs001_explain_ieq_iin_not_classified_pushdownable`
added in `crates/prism-query/src/explain.rs` module `predicate_explain_classification_tests`. Test
exercises `predicate_to_exprs` with IEQ and IIN predicates and asserts the result is empty
(non-pushdownable). RED before commit, GREEN at 2de85b18. 1406/1406 prism-query tests GREEN.

**TD-VSDD-060 sibling sweep:** 8 `Predicate`→`Expr`/classification sites inspected across
`explain.rs` and `pushdown.rs`. Only `predicate_to_exprs` in `explain.rs` required the guard;
all other sites either already handled `case_insensitive` correctly or operated on the non-CI path.

---

## SAP Probe Results (Pass 21, verified against 257074af)

**SAP-1 (tracing emission catalog completeness):** PASS — only new `event_type` value in the
delta is `ocsf.enum_label_unrecognized` (dual emission sites: `build_column_array` in
`spec_driven_adapter.rs` + `normalize_with_mappers` in `normalizer.rs`) — both match BC-2.16.002
catalog row 91. No new `event_type` sites introduced in 257074af delta. Catalog count UNCHANGED 91.

**SAP-2 (DTU↔TOML schema parity):** N/A — delta does not touch `.prism/specs/sensors/*.toml`
or DTU clone types/routes. Adversary design note: `activity_name` and `disposition` columns are
no-op at runtime today since no current sensor TOML carries those literal column names;
spec-conformant, not a defect; latent-only behavior consistent with ADR-047 design.

**SID-1 (no-ignored-test rationalization prohibition):** PASS — RG-073 is a non-`#[ignore]`
unit test within `explain.rs` `#[cfg(test)] mod predicate_explain_classification_tests` block;
no external dependency required.

**POL-22 Phase A (ID/anchor integrity):** PASS — 5 BC anchors verified in story body
(BC-2.11.024, BC-2.02.013, BC-2.11.002, BC-2.11.004, BC-2.11.018) plus BC-2.10.012,
BC-2.02.002, BC-2.02.010. E-QUERY-002 byte-verbatim error code verified present in
error-taxonomy.md. All BC IDs resolve to files in `.factory/specs/behavioral-contracts/`.

**POL-22 Phase C (RGT inventory completeness):** PASS — 21+ domain entities and all 73 RGT
names (RG-001..RG-073) verified present in story v1.27 §Red Gate Tests table.

**prism-spec-engine delta re-verified:** Comment-only changes only (TD-VSDD-091 anti-volatile-pin
sweep from prior bursts). Zero production code logic changes in prism-spec-engine. Re-verified
against 257074af delta.

---

## Fix-Burst Commit Log (feature/S-PRISMQL-CASE-INSENSITIVE-001)

| Commit | Author | Change |
|--------|--------|--------|
| 2de85b18 | implementer | RG-073 `test_BC_2_11_024_f_p21_obs001_explain_ieq_iin_not_classified_pushdownable` (explain.rs `predicate_explain_classification_tests`); `predicate_to_exprs` guard arms `case_insensitive: true => vec![]` on Compare + In; RED before commit, GREEN at 2de85b18; 1406/1406 prism-query GREEN; F-P21-OBS-001 closed |

---

## Post-Fix-Burst State

- Feature HEAD: **2de85b18** (new frozen candidate for pass-22)
- 1406/1406 prism-query tests GREEN (full workspace re-verify recommended at pass-22 gate)
- non-exhaustive: 89/89 UNCHANGED
- RG-001..073 GREEN
- LOCAL 3-CLEAN(strict) streak: **0/3** (reset by fix-burst push 2de85b18 per DRIFT-ORCH-PRLEVEL-PUSH-001)
- NEXT ACTION: LOCAL adversary pass-22 on frozen 2de85b18; if CLEAN(strict), passes 23/24 on SAME frozen HEAD complete 3-CLEAN
