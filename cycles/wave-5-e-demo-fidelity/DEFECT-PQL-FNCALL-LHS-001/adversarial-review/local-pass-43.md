---
pass: 43
lane: DEFECT-PQL-FNCALL-LHS-001
frozen_head: 5e4c7ccb
date: 2026-07-14
authored_by: orchestrator-relay
clean_strict: false
clean_pr_merge: true
finding_count: 2
streak_before: 2/3
streak_after: 0/3
status: CLOSED
fix_burst: fix-burst-32
fix_burst_head_unchanged: true
fix_burst_spec_only: true
fix_burst_bc: [BC-2.11.019, BC-2.11.004]
---

# LOCAL Adversary Pass 43 — DEFECT-PQL-FNCALL-LHS-001

**Frozen HEAD: 5e4c7ccb** (fix/DEFECT-PQL-FNCALL-LHS-001; LOCAL-ONLY; unchanged from pass-42)
**CLEAN(strict): NO** (2 findings: 1 MED + 1 LOW)
**CLEAN(PR-merge): YES** (zero CRIT + zero HIGH + zero MED that block merge — MED-001 is a spec-only attribution error, not a code defect; however per BC-5.39.001 strict criterion ANY finding resets the streak)
**Streak: 0/3** (RESET from 2/3 — pass-43 NOT CLEAN(strict); BC-5.39.001 streak-reset rule)
**Fix-Burst-32:** ALL FINDINGS CLOSED (spec-only; feature HEAD 5e4c7ccb UNCHANGED)

---

## Pass-42 Closure Re-Verification

Pass-42 had zero findings. CLEAN(strict)=YES confirmed; streak advanced to 2/3.

Pass-43 opens with the 2/3 streak position from pass-42. Two new findings identified below reset it to 0/3.

---

## Findings

### F-PQLFN-P43-MED-001 [MED][fact-error] — BC-2.11.019 v1.19 incremental-coverage attribution: "positions 3–5 by OD-5" is wrong

**Affected artifact:** BC-2.11.019 §Postconditions Implementation note (final sentence), v1.19 body, line ~56.

**Finding:** The v1.19 Implementation note stated incremental walker coverage was added "positions 1–2 by the original grammar-extension fix, positions 3–5 by OD-5 (v1.2), position 6 by OD-6 (§D.7.5), position 7 by OD-7 (§D.7.6)." The attribution "positions 3–5 by OD-5" is factually incorrect for position 3.

**Ground-truth sources verified (all three):**

1. **ADR-048 §D.7.1 Pre-v1.2 column** — Position 3 (SqlPipe pipe-stage `| where`) has "YES" in the pre-v1.2 coverage column, meaning it was covered by the ORIGINAL fix before OD-5 introduced v1.2. The original fix was the grammar-extension (the core DEFECT-PQL-FNCALL-LHS-001 fix that added `fn_call_comparison` to `build_predicate_parser`). OD-5 added positions 4 and 5 (SQL WHERE and SqlPipe-head WHERE) only.

2. **BC-2.11.019 v1.7 changelog row** — The v1.7 entry (DEFECT-PQL-FNCALL-LHS-001-adversary-pass-1) explicitly reads: "pipe `| where`, filter-mode root predicate, and SqlPipe pipe-stage `| where`" as the three NEW positions from the original grammar-extension fix. SqlPipe pipe-stage `| where` is position 3 — unambiguously in the original fix, not OD-5.

3. **BC-2.11.019 engine.rs docstring (at 5e4c7ccb)** — The walk function docstring cites positions 1–3 as the original grammar-extension set. OD-5 is identified as adding positions 4–5 only.

**Attribution error introduced:** The v1.19 attribution was supplied by orchestrator-composed fix-burst-31 dispatch text. The PO faithfully applied the text; passes 41 and 42 did not catch the factual error because the attribution appeared internally plausible (three pre-v1.2 positions grouped as 1-2-3, OD-5 adding more).

**Correct attribution:** "positions 1–3 by the original DEFECT-PQL-FNCALL-LHS-001 grammar-extension fix (ADR-048 pre-v1.2), positions 4–5 by OD-5 (v1.2), position 6 by OD-6 (§D.7.5), position 7 by OD-7 (§D.7.6)."

**Status:** CLOSED — fix-burst-32. PO BC-2.11.019 v1.19→v1.20: attribution corrected to "1–3 original fix (pre-v1.2), 4–5 OD-5 (v1.2), 6 OD-6, 7 OD-7." v1.19 changelog row preserved intact per TD-VSDD-091 append-only; v1.20 row added noting the attribution correction. TD-VSDD-060 residual grep: zero other "positions 3–5 by OD-5" or "3-5 by OD-5" occurrences found in live prose (only the corrected line-56 site and the exempt v1.19 changelog row).

**PROCESS NOTE [process-gap candidate for S-7.02]:** Orchestrator-composed fix-burst prompts that carry factual attributions should cite ground-truth sources for the specialist to verify rather than assert facts as given. The PO verified the three ground-truth sources for this correction and confirmed. Future orchestrator prompts supplying attribution claims should follow the same discipline: cite source + version, not just the attribution text.

---

### F-PQLFN-P43-LOW-001 [LOW][POL-25 pin-currency] — BC-2.11.004 v1.45 carries 13 live "ADR-048 v1.13" citations; ADR-048 is at v1.15

**Affected artifact:** BC-2.11.004 §Postconditions (multiple prose sites), v1.45 body.

**Finding:** BC-2.11.004 v1.45 contains 13 live citations of "ADR-048 v1.13" across 7 body lines. ADR-048 was advanced to v1.14 (ADR-048 v1.14 nomenclature correction — function name `collect_unknown_scalar_offsets_from_predicate`) and v1.15 (ADR-048 v1.15 INSERT source_select WHERE OD-7 full §D.7.6 section; §D.7.5 corrected; §D.7.1 "ALL seven"). The v1.38 precedent for this BC class codifies: pin-currency sweeps of this magnitude are POL-25 LOW findings — not process-gaps — because they gate future adversary passes.

**Scope:** 13 live citations at "ADR-048 v1.13" spanning: SHARED-PARSER SCOPE paragraph (3 sites), LOW-001/002/004/005/006/007 scope-limit bullets (4 sites), aggregate-gate enforcement mechanism note (3 sites), E-QUERY-042 arm description (2 sites), plus 1 in the §Error Cases table-cell embedded row text. One occurrence in the v1.45 changelog row is a historical citation — exempt per TD-VSDD-091 append-only audit trail.

**Status:** CLOSED — fix-burst-32.
- PO BC-2.11.004 v1.45→v1.46: 13 live "ADR-048 v1.13" citations advanced to "ADR-048 v1.15"; 1 changelog-internal occurrence in the v1.45 entry correctly reverted to v1.13 (historical exempt).
- Story-writer S-PRISMQL-CASE-INSENSITIVE-001 v1.70→v1.71: 4 live BC-2.11.004 pin sites in the story body updated from v1.45 to v1.46, including the bare-cell table form at line ~171 (TD-VSDD-060 discipline; the line-171 cell form has been a recurrent miss-site per F-PQLFN-P9-HIGH-001 and subsequent passes).
- Residual greps: zero live "BC-2.11.004 v1.45" occurrences after story sweep (frontmatter comment form + prose form + table-cell form all swept).

---

## SAP-1 Result

**PASS.** No `event_type =` emission changes in fix-burst-32 (spec-only; BC attribution text and ADR pin-currency corrections only). Zero net-new emission sites in `crates/`. Sweep count unchanged at 232 total / 31 in `crates/prism-query/`.

---

## Status

```
NOT CLEAN(strict) — pass 43 complete. 2 findings (1 MED + 1 LOW).

CASCADE TALLY: 43 passes / 32 fix-bursts

STREAK: 0/3 (RESET from 2/3 — pass-43 NOT CLEAN(strict); BC-5.39.001 streak-reset rule)
DRIFT-ORCH-PRLEVEL-PUSH-001: feature branch fix/DEFECT-PQL-FNCALL-LHS-001 is LOCAL-ONLY;
feature HEAD 5e4c7ccb UNCHANGED (fix-burst-32 was spec-only; no code commits); streak
gates on unchanged frozen HEAD.

FIX-BURST-32 STATUS:
  F-PQLFN-P43-MED-001: CLOSED — BC-2.11.019 v1.19→v1.20 attribution corrected
  F-PQLFN-P43-LOW-001: CLOSED — BC-2.11.004 v1.45→v1.46 (13 pins v1.13→v1.15) +
                                  S-PRISMQL-CASE-INSENSITIVE-001 v1.70→v1.71 (4 pins v1.45→v1.46)
  Feature HEAD: 5e4c7ccb UNCHANGED (spec-only burst)
  Code HEAD: UNCHANGED

FINDINGS BREAKDOWN:
  MED: 1 (F-PQLFN-P43-MED-001 fact-error attribution)
  LOW: 1 (F-PQLFN-P43-LOW-001 POL-25 pin-currency ADR-048 v1.13→v1.15)
  Total: 2

CLEAN(strict): NO (any finding resets streak per BC-5.39.001)
CLEAN(PR-merge): YES (zero CRIT+HIGH+MED blocking code defects — both findings are spec-only)

NEXT ACTION: LOCAL adversary pass 44 on frozen 5e4c7ccb (streak 0/3; feature HEAD UNCHANGED)
```
