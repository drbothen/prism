---
document_type: adversarial-review
scope: PR-LEVEL
fix_pr: FIX-IEQ-ERRPATH-001
passes: [5]
feature_head_at_review: 8610ecd0
date: 2026-07-09
clean_strict: false
clean_pr_merge: false
finding_counts:
  total: 3
  crit: 0
  high: 0
  med: 3
  low: 0
  obs: 0
  process_gap: 0
code_behavior_defects: 0
streak_after: 0/3
convergence: IN_PROGRESS
authored_by: orchestrator-relay
---

# PR-LEVEL Adversary Pass 5 — FIX-IEQ-ERRPATH-001

---

## Pass 5 (frozen 8610ecd0; fresh-context adversary; PR-LEVEL cascade; streak candidate 1/3 — NOT ADVANCING — stays 0/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=NO

**Findings:** 3 total (0 CRIT / 0 HIGH / 3 MED / 0 LOW / 0 OBS / 0 PROCESS-GAP)

**SAP-1:** PASS — No new `event_type` values introduced; BC-2.16.002 v2.08 catalog complete and unchanged.

**STREAK:** 0/3 — CLEAN(strict)=NO on frozen 8610ecd0 (3 MED findings). All 3 findings CLOSED same-burst plus class-closure sweep; no code change. PR HEAD 8610ecd0 UNCHANGED (spec/story artifacts only). Per BC-5.39.001 and DRIFT-ORCH-PRLEVEL-PUSH-001, streak stays 0/3 on UNCHANGED HEAD after spec-only closure. **Next: PR-LEVEL pass 6 on SAME frozen 8610ecd0 (streak candidate 1/3; NO push before pass 6).**

**Code HEAD at review:** 8610ecd0 (frozen; PR #219 OPEN base develop@f935edb6; just check 5397/5397 GREEN; non-exhaustive 89/89)

**CLEAN(strict):** NO — 3 MED findings

**CLEAN(PR-merge):** NO — 3 MED findings (MED counts toward PR-merge gate)

---

## Findings

### ADV-PR-P5-MED-001 — BC-2.11.016 §Preconditions.2 Position-8 Prose Residual "FilterExpr tree" (MED, POL-22, HIGH confidence)

**Finding:** BC-2.11.016 v1.24 §Preconditions.2 body prose at position-8 retained the stale phrase "FilterExpr tree" when describing the `PipeStage::Where` variant. The D-1636 pass-4 same-burst closure corrected position-8 in the **§Implementation location table** (`PipeStage::Where(FilterExpr)` → `PipeStage::Where(Predicate)`) but left the **§Preconditions.2 prose** with the old "embedded FilterExpr tree" phrasing — a dual-surface miss.

Specifically, the §Preconditions.2 prose for position-8 described the walk as "recursing into the embedded FilterExpr tree" when the correct description per the implemented AST is "recursing into the embedded `Predicate` tree" (`PipeStage::Where(Predicate)` — verified against `crates/prism-query/src/ast.rs`).

**POL-22 classification:** Citation-accuracy defect (POL-22). The prose and table are parallel narrative surfaces for the same content; fixing the table only left the prose stale. This is the dual-surface gap pattern codified in L23.

**Status:** CLOSED

**Fix:** product-owner corrected BC-2.11.016 v1.24 → **v1.25** (2026-07-09 D-1637):
- §Preconditions.2 position-8 prose: "embedded FilterExpr tree" → "embedded `Predicate` tree"
- Verified against `crates/prism-query/src/ast.rs` `PipeStage::Where(Predicate)` definition
- No code change; spec artifacts only

---

### ADV-PR-P5-MED-002 — BC-2.11.016 §Preconditions.2 Position-11 `PipeStage::Stats { by: Vec<Expr> }` Wrong on 3 Dimensions (MED, POL-22, HIGH confidence)

**Finding:** BC-2.11.016 v1.24 §Preconditions.2 position-11 prose described the Stats stage as `` `PipeStage::Stats { by: Vec<Expr> }` `` — incorrect on three independent dimensions:

1. **Variant form:** `PipeStage::Stats { by: ... }` is wrong struct-syntax; the AST defines `PipeStage::Stats(StatsStage)` — a tuple variant wrapping a `StatsStage` struct, not an inline struct variant.
2. **Field name:** The field is `by_fields`, not `by`. Verified against `crates/prism-query/src/ast.rs` `StatsStage` definition.
3. **Element type:** The element type is `FieldPath`, not `Expr`. `| stats ... by` grouping references are `FieldPath` values (column name paths), not arbitrary `Expr` nodes.

Correct description: `` `PipeStage::Stats(StatsStage)` — `StatsStage.by_fields: Vec<FieldPath>` ``

**POL-22 classification:** Three-dimensional citation-accuracy defect (POL-22) in §Preconditions.2 prose. The position-11 table row in the §Implementation location table was corrected at v1.23 (positions 10–14 update, D-1635), but the §Preconditions.2 prose for position-11 retained the pre-v1.23 `Stats { by: Vec<Expr> }` form.

**Status:** CLOSED

**Fix:** product-owner corrected BC-2.11.016 v1.24 → **v1.25** (2026-07-09 D-1637, folded into same bump as MED-001):
- §Preconditions.2 position-11 prose: `` `PipeStage::Stats { by: Vec<Expr> }` `` → `` `PipeStage::Stats(StatsStage)` — `StatsStage.by_fields: Vec<FieldPath>` ``
- Verified against `crates/prism-query/src/ast.rs` `PipeStage::Stats(StatsStage)` tuple-variant definition and `StatsStage` struct with `by_fields: Vec<FieldPath>`
- No code change; spec artifacts only

---

### ADV-PR-P5-MED-003 — Story AC-M2 "in both cases recursing via extract_field_paths_with_bareness" — Bare-Field Arm Uses Different Function (MED, POL-4/POL-22, HIGH confidence)

**Finding:** S-DEMO-FIDELITY-REMEDIATION-001 v2.43 AC-M2 described the bareness-extraction logic with the phrase "in both cases recursing via `extract_field_paths_with_bareness`" — implying that BOTH the full-path case and the bare-Field case call `extract_field_paths_with_bareness`. This is incorrect:

- The **full-path case** (multi-segment `FieldPath`) does recurse via `extract_field_paths_with_bareness`.
- The **bare-Field arm** (single-segment bare field name) calls `extract_column_name_from_field_path` **directly** — it does NOT recurse via `extract_field_paths_with_bareness`. It extracts the column name via the single-segment path accessor, then classifies it as bare with `is_bare: true`.

The phrase "in both cases" was introduced during the D-1635 v2.42 rewrite that fixed the `_with_bareness` naming, but the rewrite accidentally collapsed the two distinct code paths into a single "both cases" description — obscuring the architectural distinction between the recursive path and the direct extraction path.

**POL-4/POL-22 classification:** Story AC prose accuracy defect (POL-4) and function-citation-accuracy defect (POL-22). A literal re-implementer following "in both cases recursing via `extract_field_paths_with_bareness`" would implement the bare-Field arm as a recursive call instead of a direct `extract_column_name_from_field_path` call, producing incorrect bareness classification behavior.

**Status:** CLOSED

**Fix:** story-writer updated S-DEMO-FIDELITY-REMEDIATION-001 v2.43 → **v2.44** (2026-07-09 D-1637):
- AC-M2 split the "in both cases" description into two explicit sub-arms:
  - Multi-segment `FieldPath`: "recurses via `extract_field_paths_with_bareness`" (path-based recursive case)
  - Single-segment bare `Field`: "calls `extract_column_name_from_field_path` directly, classifying with `is_bare: true`" (direct extraction case)
- The two-arm distinction makes the architectural boundary explicit and eliminates the false "both cases recurse" claim
- No code change; story artifacts only

---

## Class-Closure Sweep (Orchestrator-Directed, Same Burst)

The adversary's dual-surface analysis of the §Preconditions.2 prose vs §Implementation location table revealed additional same-class defects across the 14-position BC-2.11.016 probe and in sibling BC-2.11.004 §Error Cases. The orchestrator directed an exhaustive class-closure sweep per Canonical Principle Rule 4.

### BC-2.11.016 §Preconditions.2 — Position-4 Type-Name Error (Additional Closure, Folded into v1.25)

During the 14-position × dual-surface sweep of §Preconditions.2 prose against `ast.rs`, the adversary found a third table error at position-4:

- **Position 4** (`| sort`): Table cited `OrderBy::expr` as the field being walked for sort key column names. `OrderBy` does not exist as a type in `crates/prism-query/src/ast.rs`. The correct type is `OrderExpr::expr` (the `OrderExpr` struct, with field `expr: Expr`). Verified against `ast.rs` `OrderExpr` definition.

This was folded into BC-2.11.016 v1.25 (same product-owner fix-burst as MED-001/MED-002).

### BC-2.11.004 §Error Cases — 4 Same-Class AST Citation Errors (Folded into v1.30)

Extending the sweep to the sibling BC-2.11.004 §Error Cases (which mirrors BC-2.11.016 gate positions for the pipe-mode parsing contract), the adversary found 4 same-class AST citation errors:

1. **`Where(Predicate)` tree description:** BC-2.11.004 §Error Cases described the position-8 walk as "the embedded FilterExpr tree" — same error as ADV-PR-P5-MED-001 above. FIXED: corrected to "embedded `Predicate` tree" (`PipeStage::Where(Predicate)`).

2. **`Stats(StatsStage)` / `by_fields` / `Vec<FieldPath>` description:** BC-2.11.004 §Error Cases used the `Stats { by: Vec<Expr> }` inline-struct form — same three-dimensional error as ADV-PR-P5-MED-002. FIXED: corrected to `PipeStage::Stats(StatsStage)` with `StatsStage.by_fields: Vec<FieldPath>`.

3. **`Enrich(EnrichStage).field` vs `.input_col`:** BC-2.11.004 §Error Cases described the `| enrich` position-13 walk as accessing `EnrichStage.field` to extract the input column name. The actual field on `EnrichStage` is `input_col` (verified against `ast.rs`). FIXED: corrected `EnrichStage.field` → `EnrichStage.input_col`.

4. **`Dedup(Vec<FieldPath>)` shape:** BC-2.11.004 §Error Cases described the `| dedup` position-14 walk using an incorrect structural form. FIXED: corrected to `PipeStage::Dedup(Vec<FieldPath>)` matching the `ast.rs` tuple-variant wrapping `Vec<FieldPath>` directly.

All 4 fixes folded into BC-2.11.004 v1.29 → **v1.30** (same product-owner fix-burst, 2026-07-09 D-1637).

### BC-2.11.017 and BC-2.11.020 Bodies Swept Clean

BC-2.11.017 (E-QUERY Pedagogical Enrichments) and BC-2.11.020 (SQL→Pipe Composition) bodies were swept against `ast.rs` for the same class of AST citation errors — no additional corrections needed. Both received pin-only bumps for BC-2.11.016 anchor sync (v1.24→v1.25) and BC-2.11.004 anchor sync (v1.29→v1.30).

---

## Version Summary

**BC version progression this burst:**
- BC-2.11.016 v1.24 → **v1.25** (MED-001 + MED-002 §Preconditions.2 prose fixes + position-4 `OrderExpr::expr` class-closure fix; 2026-07-09 D-1637)
- BC-2.11.017 v1.12 → **v1.13** (pin-only; BC-2.11.016 anchor→v1.25; ADV-PR-P5 sibling sync)
- BC-2.11.020 v1.17 → **v1.18** (pin-only; BC-2.11.016 anchor→v1.25; ADV-PR-P5 sibling sync)
- BC-2.11.004 v1.29 → **v1.30** (pin-sync + 4 same-class §Error Cases POL-22 fixes folded; BC-2.11.016 anchor→v1.25)

**Story pin round:**
- S-DEMO-FIDELITY-REMEDIATION-001 v2.43 → **v2.44** (AC-M2 sub-arm split fix + BC-2.11.016→v1.25 at 6 sites + BC-2.11.004→v1.30; ADV-PR-P5-MED-003 closure)
- S-DEMO-PRISMQL-ONBOARDING-001-B v2.19 → **v2.20** (BC-2.11.016→v1.25 + BC-2.11.017→v1.13)
- S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 v1.28 → **v1.29** (BC-2.11.020→v1.18 at 3 sites)
- S-PRISMQL-CASE-INSENSITIVE-001 v1.53 → **v1.54** (BC-2.11.004→v1.30 at 4 sites)

**Unchanged:** error-taxonomy v2.35, BC-2.16.002 v2.08

---

## Closure Summary

3 findings CLOSED same-burst (spec/story artifacts only; no code change):
- ADV-PR-P5-MED-001: product-owner BC-2.11.016 v1.24→v1.25 (§Preconditions.2 position-8 "FilterExpr tree"→"Predicate tree")
- ADV-PR-P5-MED-002: product-owner BC-2.11.016 v1.24→v1.25 (§Preconditions.2 position-11 `Stats { by: Vec<Expr> }`→`Stats(StatsStage)` with `by_fields: Vec<FieldPath>`; folded same bump)
- ADV-PR-P5-MED-003: story-writer S-DEMO-FIDELITY-REMEDIATION-001 v2.43→v2.44 (AC-M2 "in both cases recursing" split into two explicit sub-arms)

Class-closure sweep (orchestrator-directed, in-scope):
- BC-2.11.016 v1.25: position-4 `OrderBy::expr`→`OrderExpr::expr` (OrderBy type doesn't exist; folded into same v1.25 bump)
- BC-2.11.004 v1.30: 4 §Error Cases POL-22 fixes (Where(Predicate) tree; Stats(StatsStage) by_fields; Enrich input_col; Dedup(Vec<FieldPath>))
- BC-2.11.017 v1.13: pin-only (BC-2.11.016→v1.25; bodies swept clean)
- BC-2.11.020 v1.18: pin-only (BC-2.11.016→v1.25; bodies swept clean)
- Story pins: S-DEMO-FIDELITY-REMEDIATION-001 v2.44 / S-DEMO-PRISMQL-ONBOARDING-001-B v2.20 / S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 v1.29 / S-PRISMQL-CASE-INSENSITIVE-001 v1.54

PR HEAD **UNCHANGED** @ 8610ecd0. No code push this burst. PR-LEVEL streak stays 0/3.

---

## Convergence Assessment

**Trajectory:** LOCAL 19 passes on frozen 35117a38 (3-CLEAN D-1631) → PR-LEVEL pass 1 on frozen dacb60fa: 3 findings (0/0/2/0/1/0) [NOT CLEAN] → same-burst fix pushed @39c8b134 (streak reset) → **PR-LEVEL pass 2 on frozen 39c8b134: 0 findings (CLEAN; streak 1/3)** → **PR-LEVEL pass 3 on frozen 39c8b134: 3 findings (0/0/0/1/2/0) [NOT CLEAN; streak RESET 0/3]** → same-burst fix pushed @8610ecd0 → **PR-LEVEL pass 4 on frozen 8610ecd0: 1 finding (0/0/1/0/0/0) [NOT CLEAN; streak stays 0/3]** → same-burst spec-only closure (HEAD UNCHANGED) → **PR-LEVEL pass 5 on frozen 8610ecd0: 3 findings (0/0/3/0/0/0) [NOT CLEAN; streak stays 0/3]** → same-burst spec-only closure (HEAD UNCHANGED)

**Novelty:** MEDIUM — ADV-PR-P5-MED-001 and MED-002 are dual-surface POL-22 recurrences of the same class as ADV-PR-P3-LOW-001 and ADV-PR-P4-MED-001: each prior pass corrected one surface (table OR prose) but left the parallel surface stale. This is the root-cause pattern codified in L23 (POL-22 Phase-C citation audits must sweep ALL parallel narrative surfaces as a JOINT pair). ADV-PR-P5-MED-003 is a story AC prose accuracy defect in the "both cases" description introduced during the D-1635 v2.42 burst rewrite — a second instance of the same-burst-introduced attribution error pattern seen in ADV-PR-P4-MED-001.

**Pattern:** 2 MED BC §Preconditions.2 prose citation errors (same-class as prior passes; dual-surface miss) + 1 MED story AC prose accuracy error (introduced by D-1635 rewrite). Zero code-behavior defects. Zero CRIT/HIGH findings. Injection-safety and correctness surfaces remain clean.

**Streak status:** 0/3 — CLEAN(strict)=NO on frozen 8610ecd0. **PR HEAD 8610ecd0 UNCHANGED** (spec/story only closure; no code push). Per BC-5.39.001 and DRIFT-ORCH-PRLEVEL-PUSH-001, next pass re-gates on the same 8610ecd0. **NEXT: PR-LEVEL adversary pass 6 on SAME frozen HEAD 8610ecd0** (streak candidate 1/3; NO push before pass 6).

---

## Standing Probe Results

**SAP-1 (Structured Event Catalog — BC-2.16.002):** PASS — No new `event_type` values in pass-5 review scope; BC-2.16.002 v2.08 catalog complete and unchanged.

**SAP-2:** N/A — No sensor TOML spec modifications in this cascade.

**TD-VSDD-059 (paper-fix detection):** PASS — ADV-PR-P5-MED-001/MED-002 fixes rewrite §Preconditions.2 prose with verified AST type names (not renames or doc-comments); ADV-PR-P5-MED-003 fix splits the AC-M2 description into two structurally distinct sub-arms reflecting the actual code paths. No paper-fix pattern.

**TD-VSDD-060 (sibling-site sweep):** PASS — BC-2.11.016 §Preconditions.2 prose corrections are narrative-only (no function signature changes). BC-2.11.004 §Error Cases fixes are citation-only corrections of existing prose. Class-closure sweep covered BC-2.11.017/020 bodies (clean) and all 4 carrier stories (pin round complete). No additional propagation sites identified.

**POL-22 (citation accuracy):** FIXED — BC-2.11.016 v1.25 §Preconditions.2 positions 8/11/4 corrected; BC-2.11.004 v1.30 §Error Cases 4 citations corrected; all verified against `crates/prism-query/src/ast.rs`.

**POL-4 (story AC correctness):** FIXED — S-DEMO-FIDELITY-REMEDIATION-001 v2.44 AC-M2 two-sub-arm description reflects actual code path distinction (recursive vs direct extraction).

**BC-5.39.001 (3-CLEAN streak):** 0/3 — pass-5 result CLEAN(strict)=NO on frozen 8610ecd0. Spec-only closure; no push; PR HEAD UNCHANGED. Next cascade gate on same frozen 8610ecd0.
