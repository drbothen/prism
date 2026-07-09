---
document_type: adversarial-review
scope: PR-LEVEL
fix_pr: FIX-IEQ-ERRPATH-001
passes: [4]
feature_head_at_review: 8610ecd0
date: 2026-07-09
clean_strict: false
clean_pr_merge: false
finding_counts:
  total: 1
  crit: 0
  high: 0
  med: 1
  low: 0
  obs: 0
  process_gap: 0
code_behavior_defects: 0
streak_after: 0/3
convergence: IN_PROGRESS
authored_by: orchestrator-relay
---

# PR-LEVEL Adversary Pass 4 — FIX-IEQ-ERRPATH-001

---

## Pass 4 (frozen 8610ecd0; fresh-context adversary; PR-LEVEL cascade; streak candidate 1/3 — NOT ADVANCING — reset 0/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=NO

**Findings:** 1 total (0 CRIT / 0 HIGH / 1 MED / 0 LOW / 0 OBS / 0 PROCESS-GAP)

**SAP-1:** PASS — No new `event_type` values introduced; BC-2.16.002 v2.08 catalog complete.

**STREAK:** 0/3 — CLEAN(strict)=NO on frozen 8610ecd0 (1 MED finding). All findings CLOSED same-burst; no code change. PR HEAD 8610ecd0 UNCHANGED (spec/story artifacts only). Per BC-5.39.001 and DRIFT-ORCH-PRLEVEL-PUSH-001, streak stays 0/3 on UNCHANGED HEAD after spec-only closure. **Next: PR-LEVEL pass 5 on SAME frozen 8610ecd0 (streak candidate 1/3; NO push before pass 5).**

**Code HEAD at review:** 8610ecd0 (frozen; PR #219 OPEN base develop@f935edb6; just check 5397/5397 GREEN; non-exhaustive 89/89)

**CLEAN(strict):** NO — 1 MED finding

**CLEAN(PR-merge):** NO — 1 MED finding (MED counts toward PR-merge gate)

---

## Findings

### ADV-PR-P4-MED-001 — AC-M2 Function-Chain Attribution Error in S-DEMO-FIDELITY-REMEDIATION-001 v2.42 (MED, POL-4/POL-22, HIGH confidence)

**Finding:** The v2.42 rewrite of S-DEMO-FIDELITY-REMEDIATION-001 AC-M2 prose introduced a function-chain attribution error. The paragraph described `collect_predicate_columns` (the non-bareness variant, which walks Filter/Pipe positions 7–8 only) as the inner helper of `extract_predicate_columns_with_bareness`, while simultaneously attributing to it the recursion target semantics that belong to `collect_predicate_columns_with_bareness`. Specifically:

- The paragraph named `collect_predicate_columns` as the recursive helper invoked by `extract_predicate_columns_with_bareness`
- But `extract_predicate_columns_with_bareness` invokes `collect_predicate_columns_with_bareness` (the bareness-aware variant)
- `collect_predicate_columns` (non-bareness) is a separate helper restricted to Filter/Pipe positions 7–8 only and is NOT the recursion target of `extract_predicate_columns_with_bareness`

This made the ADR-048 historical blockquote in the same section incoherent: the blockquote references the non-bareness walkthrough, but the preceding paragraph had attributed bareness-variant semantics to it.

**POL-4/POL-22 classification:** This is a story AC prose attribution error (POL-4 story correctness) and a citation-accuracy defect (POL-22). The error was introduced during the D-1635 v2.42 rewrite that updated AC-M2 body to `_with_bareness` names — the paragraph correctly named `extract_predicate_columns_with_bareness` as the outer function but then incorrectly named the non-bareness variant as its inner helper.

**Status:** CLOSED

**Fix:** story-writer updated S-DEMO-FIDELITY-REMEDIATION-001 v2.42 → **v2.43** (2026-07-09 D-1636):
- AC-M2 paragraph corrected: `collect_predicate_columns_with_bareness` named as inner helper (recursion target) of `extract_predicate_columns_with_bareness`
- Clarifying sentence added: non-bareness variant `collect_predicate_columns` remains the positions-7–8 walker (Filter predicate tree and Pipe `| where` stage), making the ADR-048 historical blockquote coherent
- No code change; spec/story artifacts only

---

## Proactive Closures (Orchestrator-Directed, Same Burst)

During pass-4 review the adversary identified pre-existing defects in the BC-2.11.016 §Implementation location table — specifically AST type-name inaccuracies at positions 7, 8, and 10. These were adversary-observed-but-unreported defects in the same table that received the D-1635 pass-3 POL-22 correction. The orchestrator directed closure in-scope per Canonical Principle Rule 4 (AI-found defects are AI's responsibility to fix in-scope).

**BC-2.11.016 v1.23 → v1.24 (product-owner, 2026-07-09 D-1636):**

3 AST type-name corrections verified against `ast.rs`:

- **Position 7** ("And/Or/Not recursive walk") → "`Predicate::Logical`/`Predicate::Not` recursive walk" (corrected in the gate-positions table AND in §Preconditions.2 prose; `Predicate::Logical` is the And/Or enum variant; `Predicate::Not` is the negation variant — naming the actual enum variants is POL-22 citation accuracy)
- **Position 8** `PipeStage::Where(FilterExpr)` → `PipeStage::Where(Predicate)` (the inner type is `Predicate`, not `FilterExpr`; verified against `crates/prism-query/src/ast.rs` `PipeStage` definition)
- **Position 10** `SortEntry.field` → `SortExpr.field` (the type is `SortExpr`, not `SortEntry`; verified against `ast.rs` sort expression definition)
- Positions 9/11–14 re-scanned clean against `ast.rs` — no additional corrections needed

**Sibling BC pins (pin-only; no semantic change):**
- BC-2.11.017 v1.11 → **v1.12** (pin-only; BC-2.11.016 anchor→v1.24; ADV-PR-P4-MED-001 sibling sync)
- BC-2.11.020 v1.16 → **v1.17** (pin-only; BC-2.11.016 anchor→v1.24; ADV-PR-P4-MED-001 sibling sync)
- BC-2.11.004 v1.28 → **v1.29** (pin-only; BC-2.11.016 anchor→v1.24; ADV-PR-P4-MED-001 sibling sync)

**Story pin round (ONE version bump per story folding MED-001 closure + v1.24 pin-sync):**
- S-DEMO-FIDELITY-REMEDIATION-001 v2.42 → **v2.43** (AC-M2 chain fix + BC-2.11.016→v1.24 at 6 sites)
- S-DEMO-PRISMQL-ONBOARDING-001-B v2.18 → **v2.19** (BC-2.11.016→v1.24 + BC-2.11.017→v1.12)
- S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 v1.27 → **v1.28** (BC-2.11.020→v1.17 at 3 sites)
- S-PRISMQL-CASE-INSENSITIVE-001 v1.52 → **v1.53** (BC-2.11.004→v1.29 at 4 sites)

**L22 AST-type-name grep across stories:** zero hits — the AST type-name corrections (position 7/8/10) are in the BC §Implementation location table and §Preconditions.2 prose only; they do not propagate to story AC prose (stories cite the gate behavior, not the internal AST type names).

---

## Closure Summary

1 finding CLOSED same-burst (spec/story artifacts only; no code change):
- ADV-PR-P4-MED-001: story-writer S-DEMO-FIDELITY-REMEDIATION-001 v2.42→v2.43 (AC-M2 chain attribution corrected; non-bareness variant clarified)
- Proactive BC-2.11.016 v1.23→v1.24: 3 AST type-name corrections (positions 7/8/10 verified against ast.rs)
- Sibling pins: BC-2.11.017 v1.12 / BC-2.11.020 v1.17 / BC-2.11.004 v1.29
- Story pins: S-DEMO-FIDELITY-REMEDIATION-001 v2.43 / S-DEMO-PRISMQL-ONBOARDING-001-B v2.19 / S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 v1.28 / S-PRISMQL-CASE-INSENSITIVE-001 v1.53

PR HEAD **UNCHANGED** @ 8610ecd0. No code push this burst. PR-LEVEL streak stays 0/3.

---

## Convergence Assessment

**Trajectory:** LOCAL 19 passes on frozen 35117a38 (3-CLEAN D-1631) → PR-LEVEL pass 1 on frozen dacb60fa: 3 findings (0/0/2/0/1/0) [NOT CLEAN] → same-burst fix pushed @39c8b134 (streak reset) → **PR-LEVEL pass 2 on frozen 39c8b134: 0 findings (CLEAN; streak 1/3)** → **PR-LEVEL pass 3 on frozen 39c8b134: 3 findings (0/0/0/1/2/0) [NOT CLEAN; streak RESET 0/3]** → same-burst fix pushed @8610ecd0 → **PR-LEVEL pass 4 on frozen 8610ecd0: 1 finding (0/0/1/0/0/0) [NOT CLEAN; streak stays 0/3]** → same-burst spec-only closure (HEAD UNCHANGED)

**Novelty:** MEDIUM — ADV-PR-P4-MED-001 is an AC prose attribution error introduced in the D-1635 pass-3 burst when the AC-M2 paragraph was updated to `_with_bareness` names. The function-chain confusion (non-bareness vs bareness inner helper) is a new category of spec defect not previously seen in this cascade. The proactive BC-2.11.016 position 7/8/10 type-name corrections are POL-22 citation-accuracy defects in the same table that received pass-3 corrections — a tight-cluster recurrence pattern from the same D-1635 burst.

**Pattern:** 1 MED finding (story AC prose) + proactive BC type-name corrections (spec layer only). No code-behavior defects. CRIT/HIGH/MED injection-safety surfaces remain clean from passes 1–3.

**Streak status:** 0/3 — CLEAN(strict)=NO on frozen 8610ecd0. **PR HEAD 8610ecd0 UNCHANGED** (spec/story only closure; no code push). Per BC-5.39.001 and DRIFT-ORCH-PRLEVEL-PUSH-001, next pass re-gates on the same 8610ecd0. **NEXT: PR-LEVEL adversary pass 5 on SAME frozen HEAD 8610ecd0** (streak candidate 1/3; NO push before pass 5).

---

## Standing Probe Results

**SAP-1 (Structured Event Catalog — BC-2.16.002):** PASS — No new `event_type` values in pass-4 review scope; BC-2.16.002 v2.08 catalog complete and unchanged.

**SAP-2:** N/A — No sensor TOML spec modifications in this cascade.

**TD-VSDD-059 (paper-fix detection):** PASS — ADV-PR-P4-MED-001 fix rewrites AC-M2 prose with correct function names; the non-bareness variant clarification sentence is a substantive correction, not a rename. No paper-fix pattern.

**TD-VSDD-060 (sibling-site sweep):** PASS — BC-2.11.016 position 7/8/10 type-name corrections in the §Implementation location table; no function signature changes. L22 AST-type-name grep across all 4 carrier stories: zero hits. Sibling BC pins swept (BC-2.11.017/020/004 all pinned).

**POL-22 (citation accuracy):** FIXED — BC-2.11.016 v1.24 positions 7/8/10 AST type-name corrections applied; verified against ast.rs.

**POL-4 (story AC correctness):** FIXED — S-DEMO-FIDELITY-REMEDIATION-001 v2.43 AC-M2 function-chain attribution corrected.

**BC-5.39.001 (3-CLEAN streak):** 0/3 — pass-4 result CLEAN(strict)=NO on frozen 8610ecd0. Spec-only closure; no push; PR HEAD UNCHANGED. Next cascade gate on same frozen 8610ecd0.
