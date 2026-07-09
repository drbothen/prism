---
document_type: adversarial-review
scope: LOCAL
fix_pr: FIX-IEQ-ERRPATH-001
passes: [12]
feature_head_at_review: e5170899
fix_burst_head: 09ea9979
date: 2026-07-09
clean_strict: false
clean_pr_merge: true
finding_counts:
  total: 2
  low: 2
  obs: 0
code_behavior_defects: 1
streak_after: 0/3
convergence: IN_PROGRESS
authored_by: orchestrator-relay
---

# LOCAL Adversary Pass 12 — FIX-IEQ-ERRPATH-001

---

## Pass 12 (frozen e5170899; fresh-context adversary; diversified angles; fix-PR IEQ non-existent column error path; streak candidate 2/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=YES

**Findings:** 2 total (0 CRIT / 0 HIGH / 0 MED / 2 LOW — zero severity ≥ MED; CLEAN(PR-merge)=YES per BC-5.39.001 disambiguation 2026-05-22)

**STREAK RESET: 1/3 → 0/3** (BC-5.39.001). Both findings CLOSED same-burst via fix-burst @09ea9979.

**Code HEAD at review:** e5170899 (frozen; D-1621 last-segment fallback seeding with DERIVED provenance in branches (b) and (c) of compute_sqlpipe_head_binding; head-seeding only; 5369/5369 GREEN; non-exhaustive 89/89)

**Fix-burst HEAD:** 09ea9979 (D-1623 fix-burst: doc-count fix ADV-FIX-P12-OBS-001 + STAR-WITH-JOIN SUSPENSION RULE ADV-FIX-P12-OBS-002; 5371/5371 GREEN; non-exhaustive 89/89; fix-branch LOCAL-ONLY)

**LOCAL 3-CLEAN(strict) streak after pass-12:** 0/3 (RESET — findings closed same-burst; next streak starts at pass 13 on frozen 09ea9979)

---

## Finding ADV-FIX-P12-OBS-001 (LOW — doc-comment inaccuracy)

**Severity:** LOW  
**Confidence:** HIGH  
**Category:** Documentation accuracy (TD-VSDD-091 / POL-4)

**Description:** The `columns_for_table` function in `crates/prism-query/src/engine.rs` (or equivalent column-resolution module) carried a doc-comment stating "two distinct cases" while the implementation contained three distinct branches: (1) registered table with columns, (2) registered table without columns (fail-open), (3) unregistered table (fail-open via is_registered() path). The doc-comment "two distinct cases" was stale from before the D-1620 pass-9 fix-burst that added the `is_registered()` disambiguation gate (EC-11-041). A literal reader following the doc-comment would believe only two branches existed, missing the is_registered() path entirely.

**Routing:** implementer (code-side comment fix; no spec amendment required)

**Closure:** Implementer fixed "two distinct cases" → "three distinct cases" in the doc-comment at commit @09ea9979. Comment-only change; 5369/5369 tests unaffected; non-exhaustive 89/89 UNCHANGED.

**Status:** CLOSED @09ea9979

---

## Finding ADV-FIX-P12-OBS-002 (LOW/MED-confidence — FP-001 false E-QUERY-038 on star+join shape)

**Severity:** LOW (adversary initial classification) — confirmed behavioral defect; orchestrator evaluated and fixed in-scope per production-grade Rule 3 (no human direction required for deferral)  
**Confidence:** MEDIUM (adversary: "could be a design choice"; orchestrator: confirmed FP-001 violation via static grammar analysis)  
**Category:** Correctness — false E-QUERY-038 (FP-001 violation)

**Description:** When a SqlPipe head SELECT contains a Star (`SELECT *`) or TableStar (`SELECT j.*`) item AND the query has a non-empty JOIN list, the `compute_sqlpipe_head_binding` branches (a) and (c) at e5170899 extracted star items to `None` binding entries (anonymous non-Field expressions → suspended fail-open per FP-001) but did NOT set `suspended:=true` in the returned binding context. As a result, pipe stages downstream of the star item fired the full column-not-found gate against the FROM-only schema columns, producing false E-QUERY-038 on join-source-only column references in two shapes:

- Shape 1 (`SELECT j.* FROM t JOIN u j ON ... | where u_col`): `u_col` is from the JOIN source `u`; the star seeds only FROM-schema `t` columns; `u_col` absent → false E-QUERY-038 with `available_columns = [t columns only]`.
- Shape 2 (`SELECT * FROM t JOIN ... | where u_col`): same failure mode via bare SELECT *.

Both shapes violated FP-001: "the E-QUERY-038 gate MUST NOT fire a false ColumnNotFound for column references that are legitimately resolvable at runtime." A multi-source union would fully fix this; the adversary proposed deferring to a follow-up story. The orchestrator REJECTED deferral per production-grade Rule 3 (no human direction, no concrete future dependency, no specific story anchor) and fixed in-scope via the contract's sanctioned fail-open mechanism (FP-001 allows suspension as the fail-open path when full union is not yet implemented).

**Fix approach (orchestrator-directed, in-scope):** Product-owner authored BC-2.11.016 v1.17→v1.18 (STAR-WITH-JOIN SUSPENSION RULE): branches (a) and (c) of `compute_sqlpipe_head_binding` set `suspended:=true` when the JOIN list is non-empty and the head contains ≥1 Star/TableStar item. Branch (b) unchanged (non-star items continue to seed DERIVED via LAST-SEGMENT OUTPUT-NAME RULE per EC-11-069). Precise multi-source union documented in BC as permissible future strengthening — this is NOT a tracked deferral; the FP-001 fail-open obligation is fully met by the suspension path.

**Test evidence:**
- RED @bd62d12e: test-writer authored two new tests covering both shapes; both fired ColumnNotFound with FROM-only available columns, confirming the FP-001 violation.
- GREEN @09ea9979: implementer — branch (a) returns `Some((vec![], HashMap::new(), true))` when the JOIN list is non-empty; branch (c) sets `suspended:=true` additively alongside existing DERIVED-seeding logic; EC-11-069 lock verified GREEN; 2 new tests pass; no regression.
- Module test count: 50/50 GREEN pre-burst; 52/52 GREEN post-burst (2 new tests added).
- Full workspace: just check 5369/5369 → 5371/5371 GREEN after @09ea9979.

**Closure:** CLOSED via BC-2.11.016 v1.18 (product-owner) + RED @bd62d12e (test-writer) + GREEN @09ea9979 (implementer). Siblings pinned: BC-2.11.017 v1.5→v1.6, BC-2.11.020 v1.10→v1.11, BC-2.11.004 v1.22→v1.23, error-taxonomy v2.30→v2.31, 4 carrier story rounds (v1.47/v2.36/v2.13/v1.22). BC-INDEX v7.69→v7.70. STORY-INDEX v2.641→v2.642.

**Status:** CLOSED @09ea9979

---

## Coverage Summary — Gate Positions + FP-001 Probes

The adversary performed a fresh-context trace with diversified angles at frozen e5170899, focusing on gap areas not exhaustively covered in passes 1–11.

**Star+JOIN interaction (the gap found):**

| Shape | Head | JOIN present | Pipe stage | Expected | Actual at e5170899 | Result |
|-------|------|-------------|------------|----------|--------------------|--------|
| A | `SELECT j.* FROM t JOIN u j ON t.id = j.id` | YES | `\| where u_col` | fail-open (suspension) | false E-QUERY-038 (available=[t cols only]) | FAIL → ADV-FIX-P12-OBS-002 |
| B | `SELECT * FROM t JOIN u j ON t.id = j.id` | YES | `\| where u_col` | fail-open (suspension) | false E-QUERY-038 (available=[t cols only]) | FAIL → ADV-FIX-P12-OBS-002 |
| C | `SELECT * FROM t` (no JOIN) | NO | `\| where t_col` | E-QUERY-038 on missing col | E-QUERY-038 per EC-11-041 | PASS |
| D | `SELECT j.col FROM t JOIN other j ON ...` (EC-11-069) | YES | `\| where col` | resolve via DERIVED seeding | DERIVED seeded @e5170899 | PASS |

Shapes A and B were the gap. Shapes C and D confirmed prior behavior intact.

**FP-001 extended probe results after @09ea9979 fix:**

| Probe | Description | Result |
|-------|-------------|--------|
| FP-001-A | Star+JOIN alias qualifier → fail-open on join-source-only column | PASS (suspended:=true; fail-open) |
| FP-001-B | Bare SELECT * with JOIN → fail-open on join-source-only column | PASS (suspended:=true; fail-open) |
| FP-001-C | Star WITHOUT JOIN → E-QUERY-038 on truly absent column | PASS (suspension NOT triggered; EC-11-041 gate active) |
| FP-001-D | EC-11-069 LAST-SEGMENT still seeds DERIVED when star NOT present | PASS (branch (b) unchanged; DERIVED seeded) |
| FP-001-E | MIXED-STAR (SELECT *, expr AS alias) WITH JOIN → fail-open | PASS (branch (a) suspended:=true; branch (c) additive suspended:=true) |
| FP-001-F | Shadow alias (SELECT count(*) AS severity ... \| where severity > 5) | PASS (SIBLING-GATE CONSISTENCY per-name RAW/DERIVED provenance; DERIVED alias resolves; no false E-QUERY-002) |

---

## Standing Probe Results

**SAP-1 (Structured Event Catalog — BC-2.16.002):** PASS — adversary grepped `event_type\s*=` across the entire `crates/` workspace. Five emission sites found at @09ea9979 (3 column_not_found.rejected sites established D-1618 pass-7; 2 reload.* sites from existing module). All five have corresponding catalog rows in BC-2.16.002 §Postconditions. No new emission sites added at @09ea9979 (fix-burst touches only engine.rs branch logic and one doc-comment). No new catalog rows required.

**POL-24 (byte-verbatim EC-body):** PASS — EC-11-070 and EC-11-071 added to BC-2.11.016 v1.18 carry full field schema, audit role, and recurrence policy matching the EC-11-039..069 body format. Byte-parity with canonical EC template confirmed.

**TD-VSDD-060 (sibling-site sweep):** PASS — fix-burst @09ea9979 touches compute_sqlpipe_head_binding branches (a) and (c) only. Function signature unchanged; callers swept (2 call sites in engine.rs, both passing joins parameter correctly; no other callers). No new constant or identifier changes requiring sibling sweep.

**TD-VSDD-091 (no volatile line-pin citations):** PASS — no new `file.rs:NNN` line-number citations introduced. All behavioral anchors cite function names and EC-NN-NNN identifiers.

---

## Production Discipline Checks

**`unwrap()` / `expect()` in non-test code:** PASS — fix @09ea9979 adds no new `unwrap()` or `expect()` calls in production code. The existing `NonZeroUsize::new(N).unwrap()` const-eval exception unchanged.

**`println!` in production code:** PASS — no `println!` calls in production code at @09ea9979.

**`#[non_exhaustive]` preserved:** PASS — no new public types introduced. Non-exhaustive gate EXPECTED=89 unchanged. All 89 registered types intact.

**Story pins current after fix-burst:** PASS — 4-story pin round complete at @09ea9979 (state-manager D-1623): S-PRISMQL-CASE-INSENSITIVE-001 v1.47, S-DEMO-FIDELITY-REMEDIATION-001 v2.36, S-DEMO-PRISMQL-ONBOARDING-001-B v2.13, S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 v1.22. BC-2.11.016 v1.18 propagated to all 4 carriers.

---

## Test Load-Bearing Verification (TD-VSDD-059)

ADV-FIX-P12-OBS-002 closure claims two new tests at @09ea9979. Both tests:
1. Assert shape A (`SELECT j.* FROM t JOIN u j ON ... | where u_col`) does NOT fire E-QUERY-038 after the fix (fail-open expected).
2. Assert shape B (`SELECT * FROM t JOIN ... | where u_col`) does NOT fire E-QUERY-038 after the fix (fail-open expected).

Both tests fail at e5170899 (RED confirmed @bd62d12e) and pass at @09ea9979 (GREEN confirmed). These are load-bearing behavioral tests, not tautologies. No paper-fix.

---

## Convergence Assessment

**Trajectory:** 6 → 3 → 3 → 2 → 1 → 1 → 0 → 2(low)

**Pattern:** Pass 12 finds 2 LOW findings after pass 11 found 0. This is a mild regression from CLEAN(strict) back to 2 LOW. Regression analysis:

- ADV-FIX-P12-OBS-001 (doc-count): Pure documentation defect. Originated from the D-1620 is_registered() addition (EC-11-041) not updating the doc-comment count. Unrelated to the convergence of the behavioral gate. CLOSED commit-level.
- ADV-FIX-P12-OBS-002 (star+join suspension): Genuinely novel angle — the star+JOIN interaction was not covered by any of EC-11-039..069. The adversary applied diversified angles (new shape permutations not in prior probes) and found a real gap. This is the class of finding the multi-pass cascade is designed to surface. CLOSED same-burst via fail-open suspension.

The trajectory 0→2(low) is a regression by count but both findings are LOW severity and CLOSED same-burst with load-bearing tests. This is consistent with genuine convergence: the behavioral surface is stable; the adversary had to go to diversified novel angles to find any gap at all. After the fix-burst, the gate is stronger (suspension covers star+join) and the regression is structurally resolved.

**Novelty assessment:** MEDIUM — star+JOIN interaction was a novel angle not covered by EC-11-039..069. The doc-count defect is LOW novelty (documentation drift pattern). Combined novelty MEDIUM for this pass.

**Streak status:** 0/3 (RESET). NEXT: LOCAL adversary pass 13 on FROZEN HEAD 09ea9979 (fresh context, strict; no commits between passes per DRIFT-ORCH-PRLEVEL-PUSH-001). If CLEAN(strict), passes 14 and 15 complete BC-5.39.001 3-CLEAN → push branch → open fix-PR via pr-manager.
