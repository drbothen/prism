---
pass: 32
lane: DEFECT-PQL-FNCALL-LHS-001
frozen_head: 3e482e41
date: 2026-07-14
adversary: vsdd-factory:adversary
clean_strict: false
clean_pr_merge: false
finding_count: 4
streak_before: 0/3
streak_after: 0/3
---

# LOCAL Adversary Pass 32 — DEFECT-PQL-FNCALL-LHS-001

**Frozen HEAD:** 3e482e41 (LOCAL-ONLY NOT pushed)
**CLEAN(strict):** NO — HIGH + MED + 2 OBS findings
**CLEAN(PR-merge):** NO — 1 HIGH + 1 MED finding
**Streak:** stays 0/3 on frozen 3e482e41
**SAP-1:** PASS (zero net-new event_type emissions)

---

## Findings

### F-PQLFN-P32-HIGH-001 [HIGH][pin-sync] — CLOSED (bookkeeping race, D-1749)

**Severity:** HIGH
**Category:** pin-sync
**Status:** CLOSED — bookkeeping race; not a substantive gap

**Description:** BC-INDEX row for BC-2.11.004 showed `active — v1.43` while the actual BC file was at v1.44 (D-1749 fix-burst-23 spec-only closure). The v1.44 change (LOW-007 star-arg scope limit + "All five"→"All six" counters + count(*) Canonical Test Vector row) had been committed to BC-2.11.004 but BC-INDEX had not yet been updated.

**Adjudication:** BOOKKEEPING RACE — the pass ran while the D-1749 burst was in flight. D-1749 synced BC-INDEX v8.12→v8.13 and advanced the BC-2.11.004 row to v1.44. CLOSED by D-1749. No further action required from this pass.

---

### F-PQLFN-P32-MED-001 [MED][semantic-anchoring] — CLOSED (fix-burst-24)

**Severity:** MED
**Category:** semantic-anchoring
**Status:** CLOSED by fix-burst-24 code @e00e5d03 (implementer part 1)

**Description:** 6 code/test citation sites across the codebase referenced the phantom anchor "BC-2.11.023 AC-025". The behavioral contract BC-2.11.023 uses zero AC- identifiers (its acceptance criteria are §Postconditions items, not AC-NNN numbered). The real anchor for the predicate fn-call gate closure evidence is §Postconditions D2 of BC-2.11.023. The phantom AC-022/AC-025 citations are structurally false: they assert a numbered acceptance criterion that does not exist in the spec, violating TD-VSDD-091 behavioral-anchor discipline.

**Sites confirmed (6):** TD-VSDD-060 confirmed no 7th site via grep across all crates.

**Fix:** All 6 phantom AC-022/AC-025 citations rewritten to real §Postconditions D2 anchor. just check 5574/5574 GREEN after part 1.

---

### F-PQLFN-P32-OBS-001 [OBS][scope-boundary] — CLOSED (fix-burst-24)

**Severity:** OBS
**Category:** scope-boundary
**Status:** CLOSED by fix-burst-24: architect ADR-048 v1.12→v1.13 @73102f72 + implementer code @94ef044a (part 2) + spec BC-2.11.004 v1.44→v1.45 @87b65928 (PO)

**Description:** ADR-048 §D.7.5 prose stated "DELETE/UPDATE source_select WHERE" but the seventh gated position per the scope is INSERT source_select WHERE — §D.7.5 prose was inaccurate. The aggregate gate in the code did not cover the INSERT source_select WHERE predicate position, leaving it unguarded. ADR-048 §D.7.1 "ALL six positions" counter was also stale (should be "ALL seven").

**Adjudicated gaps (report-only, S-3.07 anchored):**
- Gap-1: E-QUERY-038 DML fail-open — INSERT source_select WHERE fires E-QUERY-039 (not E-QUERY-038) for DML predicate-column gate miss; blast radius bounded by DML materialization no-op; deferred to S-3.07.
- Gap-2: source_select projections/JOIN/HAVING E-QUERY-039 un-gated — not covered by OD-7; deferred to S-3.07.

**Fix:** ADR-048 v1.13 — OD-7 LOCKED as seventh gated position (§D.7.6 new section; §D.7.5 prose corrected to DELETE/UPDATE scope; §D.7.1 "ALL seven"); source_select HAVING exempt; sibling gap-1/gap-2/gap-3 noted as S-3.07 anchored deferrals. Code @94ef044a: source_select.where_ walk into predicate_fncall_names; RED "Got Ok"→GREEN; HAVING-exempt GREEN lock; E-QUERY-039 fold coverage verified at engine.rs 2101; 3 tests in insert_source_select_where_seventh_gated_position_tests module. just iter 1641/1641; just check GREEN.

---

### F-PQLFN-P32-OBS-002 [OBS][TD-VSDD-059] — CLOSED (fix-burst-24)

**Severity:** OBS
**Category:** TD-VSDD-059 (paper-fix detection)
**Status:** CLOSED by fix-burst-24 code @e00e5d03 (implementer part 1)

**Description:** The LOW-007 closure (star-arg scope limit, BC-2.11.004 v1.44) was spec-only (doc comment + canonical test vector) without a load-bearing test that actually executes against the gate and verifies it rejects star-arg. TD-VSDD-059 requires every claimed closure to have a load-bearing test or assertion, not just doc-comment or rename.

**Fix:** 6 star-arg GREEN lock tests added across all 6 predicate-fn-call gate surfaces (Pipe WHERE, Filter, SQL WHERE, SqlPipe head WHERE, SqlPipe where stage, DML WHERE). Each test confirms no surface accepts star as a fn-call argument — all gates reject it with the appropriate error at parse time. just check 5574/5574 GREEN.

---

## Out-of-Scope Notes (no fix required)

- **OOS-1 [scope-boundary]:** LOW-006 `list` expression excludes NULL/TRUE/FALSE literals (pre-existing grammar constraint; low blast radius; not a fn-call gate defect; out of DEFECT-PQL-FNCALL-LHS-001 scope perimeter).
- **OOS-2 [scope-boundary]:** SelectItem::Star paths — pre-existing behavior; not introduced by this defect fix branch.

---

## SAP-1 Result

PASS — zero net-new `event_type =` emissions in crates/ on this HEAD (3e482e41). No BC-2.16.002 catalog row required.

---

## Fix-Burst-24 Summary

All 4 findings CLOSED. Fix-burst-24 comprised:

1. **Code @e00e5d03** (implementer part 1): F-PQLFN-P32-MED-001 (6 phantom AC citations → §Postconditions D2) + F-PQLFN-P32-OBS-002 (6 star-arg GREEN locks). just check 5574/5574.
2. **Architect @73102f72**: ADR-048 v1.12→v1.13 (F-PQLFN-P32-OBS-001 OD-7 LOCKED; §D.7.5 corrected; §D.7.6 new section; §D.7.1 "ALL seven"; 3 sibling gaps anchored to S-3.07).
3. **Code @94ef044a** (implementer part 2): OD-7 gate arm (source_select.where_); RED→GREEN; HAVING-exempt GREEN lock; E-QUERY-039 fold coverage; 3 tests. just iter 1641/1641; just check GREEN.
4. **Spec @87b65928** (PO): BC-2.11.004 v1.44→v1.45 (SHARED-PARSER SCOPE "ALL six" → "all seven predicate positions"; 10 ADR-048 v1.12→v1.13 pin advances) + error-taxonomy v2.49→v2.50 (E-QUERY-039 coverage-split truth) + S-PRISMQL-CASE-INSENSITIVE-001 v1.69→v1.70 (4 pins).

**NEW FROZEN HEAD:** 94ef044a (LOCAL-ONLY NOT pushed)
**CASCADE TALLY:** 32 passes / 24 fix-bursts
**STREAK:** 0/3 on new frozen HEAD 94ef044a (DRIFT-ORCH-PRLEVEL-PUSH-001: NO commits/pushes until 3/3)
