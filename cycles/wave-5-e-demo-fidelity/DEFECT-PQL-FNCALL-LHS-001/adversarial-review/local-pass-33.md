---
pass: 33
lane: DEFECT-PQL-FNCALL-LHS-001
frozen_head: 94ef044a
date: 2026-07-14
adversary: vsdd-factory:adversary
clean_strict: false
clean_pr_merge: false
finding_count: 3
streak_before: 0/3
streak_after: 0/3
---

# LOCAL Adversary Pass 33 — DEFECT-PQL-FNCALL-LHS-001

**Frozen HEAD:** 94ef044a (LOCAL-ONLY NOT pushed)
**CLEAN(strict):** NO — HIGH + MED + LOW findings
**CLEAN(PR-merge):** NO — 1 HIGH + 1 MED finding
**Streak:** stays 0/3 on frozen 94ef044a
**SAP-1:** PASS (zero net-new event_type emissions)

---

## Findings

### F-PQLFN-P33-HIGH-001 [HIGH][doc-accuracy] — CLOSED (fix-burst-25 @0c534929)

**Severity:** HIGH
**Category:** doc-accuracy — stale position-count comments
**Status:** CLOSED by fix-burst-25 @0c534929 (12-site sweep)

**Description:** Following the OD-7 gate arm added in fix-burst-24 part 2 (@94ef044a), the predicate fn-call gate now covers seven predicate positions: Pipe WHERE, Filter, SQL WHERE, SqlPipe head WHERE, SqlPipe where stage, DML WHERE, and INSERT source_select WHERE. However, 12 comment/docstring sites across the `prism-query` crate retained the stale count "six predicate positions" or "6 gated surfaces" without updating to "seven". Sites confirmed via TD-VSDD-060 grep sweep:

- `crates/prism-query/src/planner/predicate_fn_call.rs` — 4 comment sites describing the gate surface count
- `crates/prism-query/src/planner/mod.rs` — 2 module-level doc-comment sites summarizing coverage
- `crates/prism-query/tests/fn_call_predicate_gate_tests.rs` — 4 test doc-string sites in module-level commentary
- `crates/prism-query/src/planner/engine.rs` — 2 inline comment sites at the E-QUERY-039 dispatch point

**Severity rationale:** HIGH because doc-comments at the gate implementation site that miscount the number of covered positions are operationally misleading: a future maintainer or security reviewer relying on these comments to assess fn-call gate coverage completeness would believe the gate is incomplete (covering 6, not 7). This is a material accuracy defect in security-relevant commentary, not cosmetic drift. TD-VSDD-091 behavioral-anchor discipline applies: gate implementation comments are behavioral anchors, not stylistic prose.

**Fix (fix-burst-25 @0c534929):** 12-site sweep — all "six predicate positions" / "6 gated surfaces" occurrences advanced to "seven predicate positions" / "7 gated surfaces" with the OD-7 (INSERT source_select WHERE) anchor added. Zero assertion values altered; zero runtime code changed. just check 5582/5582 GREEN.

---

### F-PQLFN-P33-MED-001 [MED][TD-VSDD-059] — CLOSED (fix-burst-25 @0c534929)

**Severity:** MED
**Category:** TD-VSDD-059 (paper-fix detection) — non-load-bearing HAVING lock
**Status:** CLOSED by fix-burst-25 @0c534929 (load-bearing stddev HAVING lock)

**Description:** The HAVING exemption added in fix-burst-24 (per ADR-048 §D.7.2 HAVING-exempt rationale) was documented in ADR-048 and a test module but the test exercising the HAVING-exempt path used a trivial `COUNT(*)` aggregate. `COUNT(*)` over a literal does not exercise the fn-call predicate gate's decision to exempt HAVING predicates because `COUNT(*)` contains no fn-call in a column-position predicate — it trivially passes the gate for an unrelated reason (no column argument to evaluate). The TD-VSDD-059 probe requires load-bearing tests that actually exercise the gate decision being asserted.

**Specific gap:** No test exercised a HAVING clause containing a genuine fn-call-position predicate such as `STDDEV(col) > 0.5` or `AVG(col) BETWEEN 1 AND 10` where the aggregate function is the LHS of a predicate comparison. Without such a test, the HAVING exemption logic in the gate could be silently deleted and `COUNT(*)` tests would still pass.

**Severity rationale:** MED because a regression in HAVING exemption handling would cause `HAVING STDDEV(col) > threshold` queries to incorrectly fail with E-QUERY-039 at plan time — a query that should be legal would be rejected. The lock gap was created in the same fix-burst that introduced the exemption, making it a same-commit TD-VSDD-059 violation.

**Fix (fix-burst-25 @0c534929):** Added `test_having_fn_call_stddev_exemption` and `test_having_fn_call_avg_between_exemption` to the HAVING-exempt test module. Both tests construct queries with genuine fn-call aggregate predicates in HAVING clauses (STDDEV, AVG) and verify they pass the plan-time gate without E-QUERY-039. just check 5582/5582 GREEN; prism-query 1641/1641 GREEN.

---

### F-PQLFN-P33-LOW-001 [LOW][test-coverage] — CLOSED (fix-burst-25 @0c534929)

**Severity:** LOW
**Category:** test-coverage — missing seventh-surface scope locks
**Status:** CLOSED by fix-burst-25 @0c534929 (4 scope locks: LOW-005×2, LOW-006, LOW-007)

**Description:** The OD-7 gate arm added in fix-burst-24 (INSERT source_select WHERE) introduced a new seventh gated surface. The existing test suite for the OD-7 arm verified the positive-column gate (E-QUERY-039 fires for unknown fn-call in source_select WHERE) and the negative path (valid fn-call passes). However, four scope-boundary lock tests were missing:

- **LOW-005a:** INSERT source_select WHERE with star-arg `fn(*)` — should reject with parse-time error (star-arg not accepted in any predicate fn-call position per BC-2.11.004 v1.45)
- **LOW-005b:** INSERT source_select WHERE with NULL literal — should pass gate (NULL is not a column reference; gate applies to column-position fn-call predicates only)
- **LOW-006:** INSERT source_select WHERE with multiple fn-call predicates chained via AND — verifies that each predicate in the conjunction is independently evaluated by the gate, not short-circuited
- **LOW-007:** INSERT source_select WHERE with a fn-call in the SELECT projection, not the WHERE clause — gate must NOT fire for projection fn-calls (scope boundary: gate applies to predicate positions only)

**Impact:** The missing scope locks mean that regressions in the OD-7 gate's scope boundary (e.g., accidentally gating projection fn-calls, or silently accepting star-arg) would not be caught by the test suite. Severity: LOW (scope-boundary tests; production behavior is correct per code review; coverage gap only).

**Fix (fix-burst-25 @0c534929):** Added LOW-005a, LOW-005b, LOW-006, and LOW-007 scope lock tests in the `insert_source_select_where_seventh_gated_position_tests` module. All 4 pass. just check 5582/5582 GREEN.

---

## SAP-1 Result

PASS — zero net-new `event_type =` emissions in crates/ on this HEAD (94ef044a). No BC-2.16.002 catalog row required.

---

## Fix-Burst-25 Summary

All 3 findings CLOSED. Fix-burst-25 comprised:

1. **12-site position-count comment sweep** (HIGH-001): All "six predicate positions" comments advanced to "seven" with OD-7 anchor. Zero runtime code changed.
2. **Load-bearing HAVING lock** (MED-001): `test_having_fn_call_stddev_exemption` + `test_having_fn_call_avg_between_exemption` added. Genuine aggregate fn-call predicates exercise the HAVING-exempt gate decision.
3. **Seventh-surface scope locks** (LOW-001): 4 scope-boundary tests (LOW-005a/b, LOW-006, LOW-007) for INSERT source_select WHERE surface.

Combined commit @0c534929. just check 5582/5582 GREEN; prism-query 1641/1641 GREEN; non-exhaustive 91/91. develop UNCHANGED @5f1b5771.

**NEW FROZEN HEAD:** 0c534929 (LOCAL-ONLY NOT pushed)
**CASCADE TALLY:** 33 passes / 25 fix-bursts
**STREAK:** 0/3 on new frozen HEAD 0c534929 (DRIFT-ORCH-PRLEVEL-PUSH-001: NO commits/pushes until 3/3)
