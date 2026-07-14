---
pass: 31
lane: DEFECT-PQL-FNCALL-LHS-001
frozen_head: 3e482e41
date: 2026-07-14
adversary: vsdd-factory:adversary
clean_strict: false
clean_pr_merge: true
finding_count: 1
streak_before: 2/3
streak_after: 0/3
---

# LOCAL Adversary Pass 31 — DEFECT-PQL-FNCALL-LHS-001

**Frozen HEAD:** 3e482e41 (LOCAL-ONLY NOT pushed)
**CLEAN(strict):** NO — 1 OBS finding
**CLEAN(PR-merge):** YES — zero CRIT/HIGH/MED findings
**Streak:** RESET 2/3 → 0/3 on frozen 3e482e41

---

## Findings

### F-PQLFN-P31-OBS-001 [OBS][scope-limit-doc-gap]

**Severity:** OBS
**Category:** scope-limit-doc-gap
**Novelty:** LOW

**Description:** `count(*) = 5` in a WHERE clause yields a generic Chumsky E-QUERY-001 parse error instead of a canonical aggregate-gate message. The `fn_call_arg` grammar production admits only `literal | field_path`; the star (`*`) token never parses as a valid argument, so the aggregate-function gate (`E-QUERY-042`) is never reached. BC-2.11.004 enumerated `count()` and `count(col)` as the supported forms in the Canonical Test Vector table but did not enumerate the `count(*)` star-arg form or document its scope-limit behavior.

**Evidence:** Grammar source confirms `fn_call_arg` accepts only `literal | field_path`. No parse path exists for `count(*)` to succeed at the fn_call_arg position. E-QUERY-042 aggregate-gate is never triggered; Chumsky returns E-QUERY-001 (generic parse error) at the star token.

**Impact:** OBS only — the behavior itself is a correct scope-limit (star-arg unsupported). The gap is documentation-only: the BC did not enumerate this form or its error category. No runtime regression.

**Fix:** BC-2.11.004 v1.43→v1.44: LOW-007 star-arg scope limit added at 3 sites (§Error Cases scope-limit note, §Canonical Test Vectors row, §Future Enhancements architect-adjudication note). "All five"→"All six" counter updated at affected sites. POL-23 sweep: S-PRISMQL-CASE-INSENSITIVE-001 v1.68→v1.69 (4 BC-2.11.004 pin sites updated).

**Disposition:** CLOSED by fix-burst-23 (spec-only; branch UNTOUCHED at 3e482e41; PO @7ea70220).

---

## SAP-1 Check (Tracing Emission Catalog)

`rg 'event_type\s*=' crates/ --type rust` — zero net-new `event_type` values introduced relative to frozen HEAD 3e482e41 (spec-only fix-burst-23; no code changes).
**SAP-1: CLEAN**

---

## Policy Rubric — Phase A + Phase C

Full Phase A + Phase C rubric from pass-30 carries forward unchanged (no code changes in fix-burst-23). Single new OBS finding is documentation-only scope-limit gap.

### New finding analysis

F-PQLFN-P31-OBS-001 is a scope-limit-doc-gap: grammar correctly rejects star-arg at parse time; the BC did not enumerate this form. No code fix required. BC amendment closes it.

### Streak impact

- CLEAN(strict): NO (1 OBS finding present)
- CLEAN(PR-merge): YES (zero CRIT/HIGH/MED)
- Streak RESETS 2/3 → 0/3 per BC-5.39.001
- Fix-burst-23 is spec-only; HEAD remains 3e482e41 (UNCHANGED)
- Frozen-HEAD rule (DRIFT-ORCH-PRLEVEL-PUSH-001): HEAD unchanged; streak reset re-gates on same HEAD for pass 32

---

## Workspace Test Count

PQL @3e482e41 = 5568 (unchanged — spec-only fix-burst-23; no code changes; all prism-query 1632/1632; just check 5568/5568 GREEN; non-exhaustive 91/91).

---

## Disposition

- CLEAN(strict): NO
- CLEAN(PR-merge): YES
- Findings: 1 OBS (F-PQLFN-P31-OBS-001 — scope-limit-doc-gap, LOW novelty)
- Streak: **RESET 2/3 → 0/3 on frozen 3e482e41**
- Fix-burst-23: spec-only closure (BC-2.11.004 v1.43→v1.44; PO @7ea70220); HEAD UNCHANGED at 3e482e41
- Frozen-HEAD rule (DRIFT-ORCH-PRLEVEL-PUSH-001): HEAD unchanged; pass 32 re-gates on 3e482e41 (streak 0/3)
- Next: LOCAL pass 32 on frozen 3e482e41 (streak 0/3)
