---
pass: 34
lane: DEFECT-PQL-FNCALL-LHS-001
frozen_head: 0c534929
date: 2026-07-14
adversary: vsdd-factory:adversary
clean_strict: false
clean_pr_merge: false
finding_count: 2
streak_before: 0/3
streak_after: 0/3
status: CLOSED
fix_burst: 26
fix_burst_commits: [bdfaedf7, bd044a2e]
fix_burst_new_frozen_head: bd044a2e
---

# LOCAL Adversary Pass 34 — DEFECT-PQL-FNCALL-LHS-001

**Frozen HEAD:** 0c534929 (LOCAL-ONLY NOT pushed)
**CLEAN(strict):** NO — MED + OBS findings
**CLEAN(PR-merge):** NO — 1 MED finding
**Streak:** stays 0/3 on frozen 0c534929; RESET to 0/3 on new frozen bd044a2e after fix-burst-26 push
**SAP-1:** PASS (zero net-new event_type emissions)
**Status:** CLOSED — fix-burst-26 COMPLETE @bdfaedf7 + @bd044a2e (D-1752 2026-07-14)

---

## Findings

### F-PQLFN-P34-MED-001 [MED][doc-accuracy] — CLOSED (ADR-048 v1.13→v1.14 architect sweep)

**Severity:** MED
**Category:** doc-accuracy — ADR-048 §D.7.2 residual stale sentences
**Status:** CLOSED — fix-burst-26 @bdfaedf7 (architect ADR-048 v1.13→v1.14 prose sweep)

**Description:** ADR-048 (Fn-Call LHS Predicate Grammar Extension — gate scope and DML surface coverage) §D.7.2 prose contains two sentences that still read "all six predicate positions" after the OD-7 addition in fix-burst-24. These sentences are located at approximately lines 461 and 469 of the ADR body. Both sentences describe the aggregate gate's total coverage; the fix-burst-25 comment sweep targeted code comments in `crates/prism-query/` but did not sweep ADR-048 §D.7.2 prose.

**Affected lines (approximate):**
- Line 461: "The aggregate IEQ/IIN/INE predicate gate covers all six predicate positions defined in §D.7.1." — should read "seven" after OD-7.
- Line 469: "This ensures consistent E-QUERY-039 behaviour across all six gated surfaces." — should read "seven gated surfaces".

**Severity rationale:** MED because ADR-048 is an authoritative architecture decision record. An architect or security reviewer consulting §D.7.2 for gate coverage documentation would conclude the gate covers only six surfaces, missing the OD-7 (INSERT source_select WHERE) arm entirely. This is a spec accuracy defect at the architectural decision layer — it misrepresents the deployed gate coverage to a reader with no other context. TD-VSDD-091 behavioral-anchor discipline applies: ADR prose that enumerates gate surfaces is a behavioral anchor.

**Fix plan (fix-burst-26):** Grep sweep ADR-048 for all remaining "six predicate" / "6 gated" / "all six" occurrences. Advance each to "seven" with OD-7 anchor. TD-VSDD-060 requires confirming zero residual occurrences after the sweep. Estimated: 2–4 prose sites. No code change; ADR-only.

**Closure evidence:** ADR-048 v1.13→v1.14 architect @bdfaedf7. Three live-prose sites updated to "seven": (1) §D.3 line ~340 self-contradiction sentence "enumerates six predicate positions" → "enumerates seven predicate positions" (§D.7.1 now enumerates seven positions; Position 7 INSERT source_select WHERE added in v1.13); (2) §D.7.2 "All six predicate positions receive an identical, helpful message" → "All seven predicate positions"; (3) §D.7.2 "Two canonical forms apply to all six predicate positions" → "all seven predicate positions". Exempt sites confirmed: §Changelog rows for v1.6/v1.8/v1.13 quoting "six" are historical records (accurate at those versions; §D.6 "sixth gated position" line ~382 remains correct (Position 6 is still the sixth); "non-six-name aggregates" / "six named functions" (§D.7.3, §D.2, Related Architecture Nodes) refer to the six named aggregate functions in build_agg_call_parser — UNCHANGED; "Six callers" of build_predicate_parser UNCHANGED per v1.13 OD-7 note. BC-2.11.004 sibling sweep: live body already reads "seven" at v1.45. stories/ + error-taxonomy: zero stale "six" hits. TD-VSDD-060 sweep zero residual "all six predicate" / "six gated" occurrences in normative ADR-048 prose.

---

### F-PQLFN-P34-OBS-001 [OBS][test-coverage] — CLOSED @bdfaedf7 + @bd044a2e (position-7 lock + POL-29 position-6 sibling)

**Severity:** OBS
**Category:** test-coverage — position-7 (INSERT source_select WHERE) suite lacks E-QUERY-039 unknown-UDF sibling lock
**Status:** CLOSED — fix-burst-26 @bdfaedf7 + POL-29 within-burst sibling @bd044a2e

**Description:** The seventh gated surface (INSERT source_select WHERE, OD-7) has tests for: correct column-position E-QUERY-039 gate firing (added fix-burst-24), star-arg rejection, NULL-literal pass-through, AND-chain evaluation, and projection scope-boundary (added fix-burst-25). However, the suite does not include the "unknown-UDF sibling lock" that exists for the other six gated surfaces: a test that confirms when a known-UDF fn-call appears in INSERT source_select WHERE, the gate passes (E-QUERY-039 does NOT fire). The gate is designed to fire only for fn-calls resolving to column references, not for UDFs in a predicate position. The OD-7 surface needs explicit confirmation that a valid enrichment UDF (e.g., `enrich_lookup(col)`) in an INSERT source_select WHERE predicate is not incorrectly blocked.

**Severity rationale:** OBS because the positive path (valid fn-call passes) was tested in fix-burst-24 via a general column-gate test. The gap is that the test uses a contrived column-gate scenario, not an actual enrichment UDF call in a realistic INSERT source_select WHERE context. The behavioral boundary between "column-position fn-call → gate fires" and "enrichment UDF predicate → gate passes" is not explicitly locked for OD-7. Severity: OBS (behavioral boundary documented in ADR-048 §D.7.2; the gap is lock completeness, not a production defect).

**Fix plan (fix-burst-26):** Add `test_insert_source_select_where_enrich_udf_passes_gate` to the `insert_source_select_where_seventh_gated_position_tests` module. The test constructs an INSERT source_select WHERE with `enrich_lookup(ip_address) = 'US'` (or equivalent enrichment UDF form) and verifies it passes plan-time gate without E-QUERY-039. Estimated: 1 test, ~15 lines.

**Closure evidence:** `test_insert_source_select_where_enrich_udf_passes_gate` added to `insert_source_select_where_seventh_gated_position_tests` module @bdfaedf7 (known `enrich_lookup` UDF in INSERT source_select WHERE predicate passes the gate without E-QUERY-039; confirms behavioral boundary between column-position fn-call gate-fires and enrichment-UDF predicate gate-passes is explicitly locked for Position 7). 9/9 position-7 tests GREEN; 1647/1647 prism-query. POL-29 within-burst sibling sweep @bd044a2e: implementer found position-6 (DML WHERE, OD-6) had the same asymmetry — `test_dml_where_enrich_udf_passes_gate` added to `dml_where_sixth_gated_position_tests` module; confirms known enrichment UDF in DML WHERE passes gate without E-QUERY-039. 12/12 dml_where tests GREEN; 1648/1648 prism-query. Both lock tests confirm the gate correctly distinguishes column-position fn-calls (fire E-QUERY-039) from enrichment UDF predicates (pass gate) at all gated positions.

---

## SAP-1 Result

PASS — zero net-new `event_type =` emissions in crates/ on this HEAD (0c534929). No BC-2.16.002 catalog row required.

---

## Status

**CLOSED — fix-burst-26 COMPLETE (D-1752 2026-07-14).**

Both findings fully closed by fix-burst-26:
- F-PQLFN-P34-MED-001 CLOSED @bdfaedf7: ADR-048 v1.13→v1.14 architect prose sweep (three live-prose sites "all six" → "all seven"; no code change; TD-VSDD-060 residual grep zero; BC-2.11.004 already "seven"; sibling sweep clean).
- F-PQLFN-P34-OBS-001 CLOSED @bdfaedf7 + @bd044a2e: `test_insert_source_select_where_enrich_udf_passes_gate` added (1647/1647 prism-query) + POL-29 sibling `test_dml_where_enrich_udf_passes_gate` added for position-6 (12/12 dml_where; 1648/1648 prism-query).

**CASCADE TALLY:** 34 passes / 26 fix-bursts
**STREAK:** 0/3 on new frozen HEAD bd044a2e (DRIFT-ORCH-PRLEVEL-PUSH-001: streak resets when commits pushed)
**NEXT ACTION:** LOCAL pass-35 on frozen bd044a2e (streak 0/3)
