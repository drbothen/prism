---
pass: 44
lane: DEFECT-PQL-FNCALL-LHS-001
frozen_head: 5e4c7ccb
date: 2026-07-14
authored_by: orchestrator-relay
clean_strict: false
clean_pr_merge: false
finding_count: 2
streak_before: 0/3
streak_after: 0/3
status: OPEN
fix_burst: fix-burst-33
fix_burst_pending: true
fix_burst_spec_only: false
fix_burst_bc: [BC-2.11.004]
---

# LOCAL Adversary Pass 44 — DEFECT-PQL-FNCALL-LHS-001

**Frozen HEAD: 5e4c7ccb** (fix/DEFECT-PQL-FNCALL-LHS-001; LOCAL-ONLY; unchanged from pass-43)
**CLEAN(strict): NO** (2 findings: 1 MED + 1 LOW)
**CLEAN(PR-merge): NO** (1 MED finding is a namespace-collision structural defect in spec; severity justifies blocking merge-gate as well as strict-streak)
**Streak: 0/3** (RESET — pass-44 NOT CLEAN(strict); BC-5.39.001 streak-reset rule)
**Fix-Burst-33:** PENDING (wrap-freeze; dispatched next session)

---

## Pass-43 Closure Re-Verification

Pass-43 had 2 findings (F-PQLFN-P43-MED-001 + F-PQLFN-P43-LOW-001), both CLOSED in fix-burst-32 (spec-only). Streak was RESET 0/3.

Pass-44 opens on 0/3 streak on unchanged frozen HEAD 5e4c7ccb. Two new findings identified below.

---

## Findings

### F-PQLFN-P44-MED-001 [MED][SR-006 EC-namespace collision] — OPEN (fix-burst-33 pending)

**Affected artifact:** BC-2.11.005 §Edge Cases line ~79 and BC-2.11.004 §Edge Cases line ~118 (plus ~51 and ~145).

**Finding:** EC-11-013 is doubly allocated:

1. **BC-2.11.005 line ~79** (aggregate-pushdown cache-hit path): allocated in the original cycle-1 BC-2.11.005 authorship. Per append-only precedence (same rule applied at EC-11-068 renumber, D-1719), BC-2.11.005's allocation is the **KEEPER** — it is the senior allocator.

2. **BC-2.11.004 line ~118** (aggregate-in-pipe-where position gate, v1.33 fix-burst-2 collider): also allocated EC-11-013. This is the **COLLIDER** — it must be renumbered. The same EC-11-013 identifier also appears at BC-2.11.004 lines ~51 and ~145 (two additional in-body citations), making 3 live body occurrences in BC-2.11.004 total.

**Survival history:** This collision has survived 42 passes. It is a structural namespace defect (SR-006: EC-ID uniqueness across the BC corpus), not a documentation-drift issue.

**Fix plan (fix-burst-33, product-owner):**

1. **MANDATORY pre-allocation grep:** Before assigning any replacement EC-ID, grep ALL BC files for every EC-11-NNN in the corpus to identify the next free slot. The adversary suggested EC-11-082 as a candidate, but **EC-11-081 was allocated TODAY** to BC-2.11.001 (non-finite float boundary; D-1756 burst). The grep must confirm whether EC-11-082 is free or whether another gap exists. Do NOT assume the next free slot without the corpus sweep.

2. Renumber: BC-2.11.004 — replace all 3 live body citations of EC-11-013 (lines ~51, ~118, ~145) with the next confirmed free EC-11-NNN. Bump BC-2.11.004 v1.46→v1.47.

3. POL-23 story sweep: grep all story files for BC-2.11.004 EC-11-013 citations (particularly S-PRISMQL-CASE-INSENSITIVE-001 which tracks BC-2.11.004 pins closely); update any story cites to the new ID.

4. BC-INDEX sync: update BC-2.11.004 row to v1.47.

**Precedents:** EC-11-068 three-way collision renumber (D-1719; BC-2.11.001 v1.20 EC-11-079 + BC-2.11.019 v1.11 EC-11-080). Same SR-006 discipline applies.

**Status:** OPEN — fix-burst-33 pending (product-owner; spec-only; HEAD 5e4c7ccb UNCHANGED for spec fix; NOTE: LOW-001 fix below IS a code edit, creating a new HEAD for pass-45).

---

### F-PQLFN-P44-LOW-001 [LOW][stale imperative doc-comment] — OPEN (fix-burst-33 pending)

**Affected artifact:** `crates/prism-query/src/engine.rs` ~line 1587.

**Finding:** The comment reads: "Architect must update ADR-048 v1.3 to remove this claim" (or equivalent imperative-future language). ADR-048 has since been advanced to v1.4 (nomenclature correction) and v1.15 (full §D.7.6 OD-7 section, §D.7.5 corrected, §D.7.1 "ALL seven"). The retraction referenced by the comment was completed as part of the v1.4 update. The comment is now stale — the cited action is done.

**Fix plan (fix-burst-33, implementer):** Reword the comment to past-tense, citing the completed v1.4 retraction. Example: "ADR-048 v1.4 retracted this claim (nomenclature correction; see §D.7.1 table)." or similar factual past-tense form.

**CRITICAL NOTE — new HEAD:** This is a CODE edit (engine.rs). Committing this fix advances the `fix/DEFECT-PQL-FNCALL-LHS-001` branch to a new HEAD. Per DRIFT-ORCH-PRLEVEL-PUSH-001 (frozen-HEAD rule), **pass-45 gates on the new frozen HEAD after the fix-burst-33 commit** — not on 5e4c7ccb. The MED-001 spec fix (BC-2.11.004 renumber) does NOT produce a new code HEAD (spec-only .factory/ commit), so the frozen HEAD for pass-45 is the post-LOW-001-fix commit.

**Status:** OPEN — fix-burst-33 pending (implementer; code edit; new HEAD produced; pass-45 gates on new frozen HEAD).

---

## SAP-1 Result

**PASS.** No `event_type =` emission changes at frozen HEAD 5e4c7ccb. Sweep count unchanged at 232 total / 31 in `crates/prism-query/`. No new or removed emission sites in this pass window.

---

## Positive Verifications

- **ADR-048 changelog chain v1.0–v1.15 verified monotonic with no gaps:** Each version from v1.0 through v1.15 has a corresponding changelog row; no skipped versions; no out-of-order entries. Chain is internally consistent.

- **All spec test-name citations resolve:** BC-2.11.004, BC-2.11.005, BC-2.11.019 test-name citations in §Red Gate Tests and §Edge Cases sections all correspond to extant test functions in `crates/prism-query/src/`.

- **BC-2.11.001 v1.22 attribution verified:** v1.20 (EC-11-079) and v1.22 (EC-11-081) changelog rows reference distinct, non-colliding EC IDs. BC-2.11.001 does not allocate EC-11-013 or interfere with the BC-2.11.004/BC-2.11.005 collision.

---

## Status

```
NOT CLEAN(strict) — pass 44 complete. 2 findings (1 MED + 1 LOW).

CASCADE TALLY: 44 passes / 32 fix-bursts

STREAK: 0/3 (RESET — pass-44 NOT CLEAN(strict); BC-5.39.001 streak-reset rule)
DRIFT-ORCH-PRLEVEL-PUSH-001: feature branch fix/DEFECT-PQL-FNCALL-LHS-001 is LOCAL-ONLY;
feature HEAD 5e4c7ccb UNCHANGED at wrap-freeze. Fix-burst-33 LOW-001 IS a code edit →
new frozen HEAD for pass-45.

FIX-BURST-33 STATUS (pending next session):
  F-PQLFN-P44-MED-001: PENDING — BC-2.11.004 EC-11-013 renumber (PO; grep for next free
                                  EC-11-NNN FIRST — EC-11-081 IS TAKEN); v1.46→v1.47;
                                  3 body cites + POL-23 story sweep + BC-INDEX sync.
                                  Spec-only; HEAD unchanged during this step.
  F-PQLFN-P44-LOW-001: PENDING — engine.rs ~1587 stale imperative comment reword
                                  (implementer; code edit → NEW frozen HEAD).
  After fix-burst-33: state-manager burst → LOCAL pass-45 on new frozen HEAD.

FINDINGS BREAKDOWN:
  MED: 1 (F-PQLFN-P44-MED-001 SR-006 EC-11-013 namespace collision; KEEPER=BC-2.11.005)
  LOW: 1 (F-PQLFN-P44-LOW-001 stale imperative comment engine.rs ~1587)
  Total: 2

CLEAN(strict): NO (any finding resets streak per BC-5.39.001)
CLEAN(PR-merge): NO (MED-001 is structural namespace defect; blocks merge-gate)

NEXT ACTION: fix-burst-33 (PO + implementer) → state-manager burst → LOCAL pass-45 on new frozen HEAD
```
