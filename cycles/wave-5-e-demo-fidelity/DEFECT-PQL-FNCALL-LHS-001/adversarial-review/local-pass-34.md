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
status: OPEN
fix_burst_pending: 26
---

# LOCAL Adversary Pass 34 — DEFECT-PQL-FNCALL-LHS-001

**Frozen HEAD:** 0c534929 (LOCAL-ONLY NOT pushed)
**CLEAN(strict):** NO — MED + OBS findings
**CLEAN(PR-merge):** NO — 1 MED finding
**Streak:** stays 0/3 on frozen 0c534929
**SAP-1:** PASS (zero net-new event_type emissions)
**Status:** OPEN — fix-burst-26 pending (session wrap freeze 2026-07-14 D-1751)

---

## Findings

### F-PQLFN-P34-MED-001 [MED][doc-accuracy] — OPEN (fix-burst-26 pending)

**Severity:** MED
**Category:** doc-accuracy — ADR-048 §D.7.2 residual stale sentences
**Status:** OPEN — fix-burst-26 NOT dispatched (session wrap freeze)

**Description:** ADR-048 (Fn-Call LHS Predicate Grammar Extension — gate scope and DML surface coverage) §D.7.2 prose contains two sentences that still read "all six predicate positions" after the OD-7 addition in fix-burst-24. These sentences are located at approximately lines 461 and 469 of the ADR body. Both sentences describe the aggregate gate's total coverage; the fix-burst-25 comment sweep targeted code comments in `crates/prism-query/` but did not sweep ADR-048 §D.7.2 prose.

**Affected lines (approximate):**
- Line 461: "The aggregate IEQ/IIN/INE predicate gate covers all six predicate positions defined in §D.7.1." — should read "seven" after OD-7.
- Line 469: "This ensures consistent E-QUERY-039 behaviour across all six gated surfaces." — should read "seven gated surfaces".

**Severity rationale:** MED because ADR-048 is an authoritative architecture decision record. An architect or security reviewer consulting §D.7.2 for gate coverage documentation would conclude the gate covers only six surfaces, missing the OD-7 (INSERT source_select WHERE) arm entirely. This is a spec accuracy defect at the architectural decision layer — it misrepresents the deployed gate coverage to a reader with no other context. TD-VSDD-091 behavioral-anchor discipline applies: ADR prose that enumerates gate surfaces is a behavioral anchor.

**Fix plan (fix-burst-26):** Grep sweep ADR-048 for all remaining "six predicate" / "6 gated" / "all six" occurrences. Advance each to "seven" with OD-7 anchor. TD-VSDD-060 requires confirming zero residual occurrences after the sweep. Estimated: 2–4 prose sites. No code change; ADR-only.

---

### F-PQLFN-P34-OBS-001 [OBS][test-coverage] — OPEN (fix-burst-26 pending)

**Severity:** OBS
**Category:** test-coverage — position-7 (INSERT source_select WHERE) suite lacks E-QUERY-039 unknown-UDF sibling lock
**Status:** OPEN — fix-burst-26 NOT dispatched (session wrap freeze)

**Description:** The seventh gated surface (INSERT source_select WHERE, OD-7) has tests for: correct column-position E-QUERY-039 gate firing (added fix-burst-24), star-arg rejection, NULL-literal pass-through, AND-chain evaluation, and projection scope-boundary (added fix-burst-25). However, the suite does not include the "unknown-UDF sibling lock" that exists for the other six gated surfaces: a test that confirms when a known-UDF fn-call appears in INSERT source_select WHERE, the gate passes (E-QUERY-039 does NOT fire). The gate is designed to fire only for fn-calls resolving to column references, not for UDFs in a predicate position. The OD-7 surface needs explicit confirmation that a valid enrichment UDF (e.g., `enrich_lookup(col)`) in an INSERT source_select WHERE predicate is not incorrectly blocked.

**Severity rationale:** OBS because the positive path (valid fn-call passes) was tested in fix-burst-24 via a general column-gate test. The gap is that the test uses a contrived column-gate scenario, not an actual enrichment UDF call in a realistic INSERT source_select WHERE context. The behavioral boundary between "column-position fn-call → gate fires" and "enrichment UDF predicate → gate passes" is not explicitly locked for OD-7. Severity: OBS (behavioral boundary documented in ADR-048 §D.7.2; the gap is lock completeness, not a production defect).

**Fix plan (fix-burst-26):** Add `test_insert_source_select_where_enrich_udf_passes_gate` to the `insert_source_select_where_seventh_gated_position_tests` module. The test constructs an INSERT source_select WHERE with `enrich_lookup(ip_address) = 'US'` (or equivalent enrichment UDF form) and verifies it passes plan-time gate without E-QUERY-039. Estimated: 1 test, ~15 lines.

---

## SAP-1 Result

PASS — zero net-new `event_type =` emissions in crates/ on this HEAD (0c534929). No BC-2.16.002 catalog row required.

---

## Status

**OPEN — fix-burst-26 NOT dispatched.**

Session wrap freeze 2026-07-14 (D-1751). Both findings are scoped and bounded: MED-001 is an ADR-048 prose sweep (no code change) and OBS-001 is a single lock test. fix-burst-26 is the immediate next action when the PQL lane resumes.

**CASCADE TALLY:** 34 passes / 25 fix-bursts
**STREAK:** 0/3 on frozen HEAD 0c534929
**NEXT ACTION:** fix-burst-26 — ADR-048 §D.7.2 seven-surface sweep (F-PQLFN-P34-MED-001) + OD-7 unknown-UDF sibling lock test (F-PQLFN-P34-OBS-001) → local-pass-35 gate
