---
pass: 49
lane: DEFECT-PQL-FNCALL-LHS-001
frozen_head: 973aedcf
date: 2026-07-14
authored_by: orchestrator-relay
clean_strict: true
clean_pr_merge: true
finding_count: 0
streak_before: 2/3
streak_after: 3/3
convergence: CONVERGED
status: CLOSED
---

# LOCAL Adversary Pass 49 — DEFECT-PQL-FNCALL-LHS-001

**Frozen HEAD: 973aedcf** (fix/DEFECT-PQL-FNCALL-LHS-001; LOCAL-ONLY; UNCHANGED since fix-burst-33; pass-49 is the fifth pass on this HEAD)
**CLEAN(strict): YES** (ZERO in-perimeter findings)
**CLEAN(PR-merge): YES** (ZERO findings of CRIT/HIGH/MED/LOW)
**Streak: 3/3** (ADVANCES 2/3→3/3; BC-5.39.001; DRIFT-ORCH-PRLEVEL-PUSH-001 clean — frozen HEAD 973aedcf UNCHANGED; passes 47/48/49 consecutive CLEAN(strict) on unchanged frozen HEAD — LOCAL CONVERGENCE ACHIEVED)

---

## Pass-48 Closure Verification

Pass-48 had ZERO in-perimeter findings (CLEAN(strict)=YES; CLEAN(PR-merge)=YES; streak 1/3→2/3; D-1771). No fix-burst between passes 48 and 49. HEAD UNCHANGED at 973aedcf. Closure verified: no action required.

---

## Findings

**ZERO in-perimeter findings.**

---

## Deferred Findings (out-of-perimeter — BC-5.39.002 PC2; does NOT reset streak)

### D-PQLFN-P47-OBS-001 [OBS][cross-story, SR-006-scope] — UNCHANGED (out-of-perimeter)

Status UNCHANGED from passes 47 and 48 (D-1770, D-1771). Deferred per BC-5.39.002 PC2. Does NOT reset streak.

**Finding (verbatim from pass-47):** S-3.09-query-profiling.md (draft, blocked on S-3.02) §Edge Cases lines ~354-358 pre-allocates EC-11-080/081/082/083/084 at story level; EC-11-080 collides with BC-2.11.019 line ~165; EC-11-081 collides BC-2.11.001 (D-1756); EC-11-082 collides BC-2.11.004 v1.47. SR-006 EC-ID pre-allocation grep scope does not currently include .factory/stories/ — 3rd EC-collision incident.

**Routing:** Wave-gate adjudication at S-3.09 dispatch. Options: A) codify broadened SR-006 grep scope (include .factory/stories/), B) preemptive story-writer renumber, C) renumber at S-3.09 dispatch. Deferral anchor: S-3.09.

**Status:** DEFERRED — out-of-perimeter. Registered in STATE.md Drift Items. Does NOT reset streak.

---

## SAP-1 Result

**PASS.** SAP-1 carried forward from pass-48 (no code change at frozen HEAD 973aedcf; fix-burst-35 spec-only; UNCHANGED). 55 raw prism-query occurrences of `event_type\s*=`; 12 distinct live production emission values per settled pass-45 methodology: write_tool_registration_after_boot, reload.overlay_rebuild_failed, reload.overlay_rebuilt, column_not_found.rejected, sql.sql_planning_error, filter.sql_lowering, filter.sql_planning_error, pipe.sql_lowering, pipe.sql_planning_error, infusion.coercion_failed, table_registry.rwlock_poisoned, push_down.inverted_time_range. All 12 catalogued in BC-2.16.002 §Postconditions Canonical Structured Event Catalog. No new or removed emission sites (HEAD UNCHANGED; no code changes since fix-burst-33).

---

## Positive Verifications

All positive verifications re-derived fresh at frozen HEAD 973aedcf (UNCHANGED):

- **Seven-position walker load-bearing (engine.rs 1930–2107, positions 1–7 enumerated with line anchors; DML match arm wildcard removed; HAVING intentionally exempt):** All seven predicate positions gated by the fn_call_comparison walker confirmed. Position 7 (INSERT source_select WHERE — OD-7) present at engine.rs ~lines 1930–2107. ADR-048 v1.15 §D.7.1 attribution correct.

- **DATAFUSION_BUILTIN_AGGREGATE_NAMES gate load-bearing (engine.rs 2109–2134, runs unconditionally, EC-11-082-compliant message).**

- **LOW-006 gate load-bearing (filter_parser.rs 1492–1508, byte-exact E-QUERY-001 template):** RESERVED_KEYWORDS const at lines 1492–1496 contains verbatim 20 PrismQL predicate-operator keywords: NOT, AND, OR, IN, IIN, IEQ, INE, IS, BETWEEN, LIKE, CIDR, MATCHES, HAS, MISSING, CONTAINS, ICONTAINS, STARTSWITH, ISTARTSWITH, ENDSWITH, IENDSWITH. Case-insensitive check via `.eq_ignore_ascii_case(kw)` at line 1499. E-QUERY-001 emitter at lines 1501–1506. Gate is load-bearing (TD-VSDD-059).

- **BC-INDEX v8.25 verbatim RESERVED_KEYWORDS at 3 sites grep-clean of fabricated residue:** Frontmatter v8.25; TD-VSDD-060 sweep zero residual fabricated-list carriers (sanctioned exclusions unchanged from passes 47–48).

- **Injection-safety load-bearing (error.rs 83–93 + test at test_enrich_udf_not_found_display.rs:162 asserting U+0085/U+2028 stripped).**

- **EC-11-082 renumber structural (3 live BC sites; BC-2.11.005 KEEPER sole live EC-11-013):** EC-11-082 in BC-2.11.004 v1.47 is the renumbered EC (was EC-11-013; renamed fix-burst-33 D-1764). Renumber structural — not a narrative-only correction.

- **Story v1.72 4 pins current:** S-PRISMQL-CASE-INSENSITIVE-001 v1.72 carries 4 live BC-2.11.004 v1.47 pin sites; zero stale v1.46 or earlier pins.

- **ADR-048 v1.15 13 live pins current:** All 13 ADR-048 pin sites in BC-2.11.004 carry v1.15; no stale v1.13 or earlier.

- **pipe_sql_emitter defense-in-depth backstop (943–948):** Backstop present as final defense layer behind the seven-position walker.

- **SAP-1 PASS (55 raw / 12 distinct live values, settled methodology).**

- **POL-22 Phase A+C PASS.**

- **Novelty LOW** — "Spec has converged for this defect scope."

---

## CONVERGENCE

**BC-5.39.001 3-CLEAN LOCAL CONVERGENCE ACHIEVED.**

Passes 47/48/49 — three consecutive CLEAN(strict) on unchanged frozen HEAD 973aedcf. DRIFT-ORCH-PRLEVEL-PUSH-001 frozen-HEAD rule satisfied: zero commits pushed to fix/DEFECT-PQL-FNCALL-LHS-001 since fix-burst-33 (HEAD 973aedcf). Cascade tally: 49 passes / 35 fix-bursts.

**Handoff items (carry verbatim through push/PR/merge stages):**

1. **DRIFT-PQLFN-OD7 human ratification at PR merge gate:** Gap-1 (E-QUERY-038 DML fail-open → S-3.07) + Gap-2 (dml.source_select projections/join_on/having un-gated by E-QUERY-039 → S-3.07; where_ IS gated by OD-7 position 7; BC-2.11.019 §OBS-003 documents scope).

2. **BC-2.11.019 cross-branch enforcement note:** §Injection-safety rides THIS branch, not PR #222; both merge gates must confirm the division.

3. **D-PQLFN-P47-OBS-001 deferred** (S-3.09 EC collisions; anchor S-3.09 dispatch; does not block this PR).

4. **LOCAL-ONLY status ends at push;** pr-manager 9-step cycle opens PR-LEVEL cascade with fresh 0/3 on the pushed HEAD.

5. **MCP and PQL convergences are independent;** PQL push+PR can proceed without waiting on MCP merge.

---

## Status

```
CLEAN(strict): YES — pass 49 complete. ZERO in-perimeter findings.
CLEAN(PR-merge): YES — ZERO findings of any severity within perimeter.

CASCADE TALLY: 49 passes / 35 fix-bursts

STREAK: 3/3 (ADVANCES 2/3→3/3; BC-5.39.001; LOCAL CONVERGENCE ACHIEVED)
DRIFT-ORCH-PRLEVEL-PUSH-001: satisfied — frozen HEAD 973aedcf UNCHANGED across passes 47/48/49.

DEFERRED (out-of-perimeter, does NOT reset streak per BC-5.39.002 PC2):
  D-PQLFN-P47-OBS-001: EC-11-080/081/082 collision in S-3.09 draft — anchor S-3.09 dispatch.

LOCAL CONVERGENCE: ACHIEVED (3/3 consecutive CLEAN(strict) on frozen HEAD 973aedcf).
NEXT ACTION: push fix/DEFECT-PQL-FNCALL-LHS-001 → pr-manager PR creation → PR-LEVEL cascade (fresh 0/3 on pushed HEAD).
```
