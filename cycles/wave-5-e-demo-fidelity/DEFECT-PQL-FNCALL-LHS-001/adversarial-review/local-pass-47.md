---
pass: 47
lane: DEFECT-PQL-FNCALL-LHS-001
frozen_head: 973aedcf
date: 2026-07-14
authored_by: orchestrator-relay
clean_strict: true
clean_pr_merge: true
finding_count: 0
streak_before: 0/3
streak_after: 1/3
status: CLOSED
---

# LOCAL Adversary Pass 47 — DEFECT-PQL-FNCALL-LHS-001

**Frozen HEAD: 973aedcf** (fix/DEFECT-PQL-FNCALL-LHS-001; LOCAL-ONLY; UNCHANGED since fix-burst-33; pass-47 is the third pass on this HEAD)
**CLEAN(strict): YES** (ZERO in-perimeter findings)
**CLEAN(PR-merge): YES** (ZERO findings of CRIT/HIGH/MED/LOW)
**Streak: 1/3** (ADVANCES 0/3→1/3; BC-5.39.001; DRIFT-ORCH-PRLEVEL-PUSH-001 clean — frozen HEAD 973aedcf UNCHANGED)

---

## Pass-46 Closure Verification

Pass-46 had 1 finding (F-PQLFN-P46-MED-001 BC-INDEX LOW-006 fabricated keyword list), CLOSED in fix-burst-35 (spec-only). Closure VERIFIED SUBSTANTIVE:

- **F-PQLFN-P46-MED-001 — SUBSTANTIVE:** BC-INDEX lines ~165/~414/~442 carry the verbatim RESERVED_KEYWORDS list byte-matching filter_parser.rs lines 1492-1496, with sanctioned markers; frontmatter v8.25; TD-VSDD-060 sweep = zero residual fabricated-list carriers (legit exclusions: S2-10 mockup, S-PRISMQL-CASE-INSENSITIVE-001 ILIKE prose, ADR-047/BC-2.11.024, ADR-009). Gate load-bearing (eq_ignore_ascii_case + E-QUERY-001 emit in fn_call_comparison .validate(), lines 1492-1508).

---

## Findings

**ZERO in-perimeter findings.**

---

## Deferred Findings (out-of-perimeter — BC-5.39.002 PC2; does NOT reset streak)

### D-PQLFN-P47-OBS-001 [OBS][cross-story, SR-006-scope] — DEFERRED (out-of-perimeter)

**Scope rationale:** S-3.09-query-profiling.md is a draft story (blocked on S-3.02) outside the DEFECT-PQL-FNCALL-LHS-001 fix perimeter. Per BC-5.39.002 PC2, out-of-perimeter observations do NOT reset the streak.

**Finding:** S-3.09-query-profiling.md §Edge Cases lines ~354-358 pre-allocates EC-11-080/081/082/083/084 at story level. EC-11-080 collides with BC-2.11.019 line ~165; EC-11-081 collides with BC-2.11.001 (D-1756); EC-11-082 collides with BC-2.11.004 v1.47. BC-2.11.004's pre-allocation grep claim was accurate under its qualified BC-corpus scope (grep targeted BC files only, not .factory/stories/).

**Root cause:** SR-006 EC-ID pre-allocation grep scope does not currently include .factory/stories/ — story-level draft EC reservations are invisible to the BC-corpus grep. This is the third EC-collision incident: EC-11-068 (D-1719), EC-11-013 pass-44, S-3.09 draft block.

**Routing:** Wave-gate adjudication at S-3.09 dispatch. Options: A) codify broadened SR-006 grep scope (include .factory/stories/), B) preemptive story-writer renumber, C) renumber at S-3.09 dispatch. Deferral anchor: S-3.09.

**Status:** DEFERRED — out-of-perimeter. Registered as D-PQLFN-P47-OBS-001 in STATE.md Drift Items (anchor: S-3.09 dispatch / wave-gate). Does NOT reset streak.

---

## SAP-1 Result

**PASS.** SAP-1 carried forward from pass-46 (no code change at frozen HEAD 973aedcf; fix-burst-35 spec-only; UNCHANGED). 55 raw prism-query occurrences of `event_type\s*=`; 12 distinct live production emission values per settled pass-45 methodology: write_tool_registration_after_boot, reload.overlay_rebuild_failed, reload.overlay_rebuilt, column_not_found.rejected, sql.sql_planning_error, filter.sql_lowering, filter.sql_planning_error, pipe.sql_lowering, pipe.sql_planning_error, infusion.coercion_failed, table_registry.rwlock_poisoned, push_down.inverted_time_range. All 12 catalogued in BC-2.16.002 §Postconditions Canonical Structured Event Catalog. No new or removed emission sites (fix-burst-35 spec-only; zero crate changes).

---

## Positive Verifications

- **Seven-position walker verified (lines 1930–2107, all 7 positions per ADR-048 §D.7.1):** All seven predicate positions gated by the fn_call_comparison walker confirmed at frozen HEAD 973aedcf. Position 7 (INSERT source_select WHERE — OD-7) present at engine.rs ~lines 1930–2107. ADR-048 v1.15 §D.7.1 attribution correct.

- **LOW-006 gate verified load-bearing (filter_parser.rs 1492–1508):** RESERVED_KEYWORDS const at lines 1492–1496 contains verbatim 20 PrismQL predicate-operator keywords: NOT, AND, OR, IN, IIN, IEQ, INE, IS, BETWEEN, LIKE, CIDR, MATCHES, HAS, MISSING, CONTAINS, ICONTAINS, STARTSWITH, ISTARTSWITH, ENDSWITH, IENDSWITH. Case-insensitive check via `.eq_ignore_ascii_case(kw)` at line 1499. E-QUERY-001 emitter at lines 1501–1506. Gate is load-bearing (TD-VSDD-059).

- **EC-11-082 renumber still substantive:** EC-11-082 in BC-2.11.004 v1.47 is the renumbered EC (was EC-11-013; renamed fix-burst-33 D-1764). Renumber structural — not a narrative-only correction.

- **Story v1.72 4 pins current:** S-PRISMQL-CASE-INSENSITIVE-001 v1.72 carries 4 live BC-2.11.004 v1.47 pin sites; zero stale v1.46 or earlier pins (post-fix-burst-34 D-1766 verified exhaustive).

- **ADR-048 v1.15 13 cites current:** All 13 ADR-048 pin sites in BC-2.11.004 carry v1.15; no stale v1.13 or earlier (post-fix-burst-32).

- **STORY-INDEX v2.686 / BC-INDEX v8.25 / ARCH-INDEX v2.191 current:** Index versions match STATE.md frontmatter.

- **Walker + LOW-006 gate + injection-safety all load-bearing with named tests.** Novelty LOW.

- **POL-22 Phase A+C PASS (7-position walker attribution matches engine.rs 1930-2106; sanitize_for_log at EnrichUdfNotFoundDetails::new matches BC-2.11.019 line 99 claim incl. U+2028/U+2029; aggregate plan-time gate sole enforcement, parser guard absent; ADR-048 §D.7.1 consistent):** BC-2.11.004 v1.47 body code-truth check: Phase A (BC prose → code match) and Phase C (code path → BC coverage) both PASS at frozen 973aedcf. No changes since pass-46 verification.

---

## Status

```
CLEAN(strict): YES — pass 47 complete. ZERO in-perimeter findings.
CLEAN(PR-merge): YES — ZERO findings of any severity within perimeter.

CASCADE TALLY: 47 passes / 35 fix-bursts

STREAK: 1/3 (ADVANCES 0/3→1/3; BC-5.39.001)
DRIFT-ORCH-PRLEVEL-PUSH-001: feature branch fix/DEFECT-PQL-FNCALL-LHS-001 is LOCAL-ONLY.
Frozen HEAD for pass-48: 973aedcf UNCHANGED.

DEFERRED (out-of-perimeter, does NOT reset streak per BC-5.39.002 PC2):
  D-PQLFN-P47-OBS-001: EC-11-080/081/082 collision in S-3.09 draft — anchor S-3.09 dispatch.

NEXT ACTION: LOCAL pass-48 on SAME frozen HEAD 973aedcf (streak 1/3; HEAD UNCHANGED)
```
