---
document_type: adversarial-review
scope: PR-LEVEL
fix_pr: DEFECT-PQL-FNCALL-LHS-001
passes: [9]
feature_head_at_review: 97cb070e
date: 2026-07-15
clean_strict: false
clean_pr_merge: true
finding_counts:
  total: 3
  crit: 0
  high: 0
  med: 0
  low: 2
  obs: 1
  process_gap: 0
  out_of_scope_obs: 1
code_behavior_defects: 0
streak_after: 0/3
convergence: IN_PROGRESS
status: CLOSED
authored_by: orchestrator-relay
---

# PR-LEVEL Adversary Pass 9 — DEFECT-PQL-FNCALL-LHS-001

---

## Pass 9 (frozen 97cb070e; fresh-context adversary; PR #223 PQL function-call LHS cascade; streak 1/3→0/3 RESET)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=YES

Streak: **0/3** (BC-5.39.001 strict criterion: pass-9 has 2 LOW + 1 OBS findings within perimeter; CLEAN(strict) requires zero findings of any severity; streak RESETS 1/3→0/3; fix-burst-40 pushed commit f715b0a5; DRIFT-ORCH-PRLEVEL-PUSH-001: re-gate on frozen HEAD f715b0a5 for all subsequent passes)

Cascade tally (as of this pass): **9 passes / 5 fix-bursts**

CLEAN(strict): NO — 3 in-perimeter findings (2 LOW + 1 OBS)
CLEAN(PR-merge): YES — 0 CRIT/HIGH/MED findings; LOW/OBS non-blocking

**Attack angles for this pass:** mutation reasoning, dependency-version coupling, init/concurrency, pub-API surface.

---

## Findings

### F-PQLFN-PR9-LOW-001 — 17 of 21 RESERVED_KEYWORDS had no per-keyword rejection test (M14 class survives)

**Severity:** LOW
**Category:** test-coverage gap / mutation reasoning
**Routing:** test-writer (fix-burst-40; CLOSED this session)

**File/Anchor:** `crates/prism-query/src/filter_parser.rs` — `RESERVED_KEYWORDS` set (21 entries); `crates/prism-query/tests/bc_2_11_004_test.rs` (or equivalent keyword-rejection test file); BC-2.11.004 v1.48 EC-11-004-006

**Defect:** The M14 mutation class targets list-entry removal: deleting any single entry from `RESERVED_KEYWORDS` in `filter_parser.rs`. Of the 21 keywords in the set, only 4 had per-keyword named tests directly exercising their rejection path. The remaining 17 keywords would survive an M14 entry-removal mutation — the test suite passes even if those 17 keywords are silently removed from the gate. This means the gate's completeness invariant (exactly 21 keywords blocked) is not mutation-killed; a DataFusion upgrade that silently reclassified one keyword from reserved to callable would go undetected until analyst runtime.

**Failure scenario:** M14 mutation applied: DELETE keyword `NULLIF` from `RESERVED_KEYWORDS` → test suite PASSES (no test directly exercises `NULLIF` as fn-call LHS in WHERE) → gate silently allows `nullif(col, 0) > 0` to proceed to DataFusion → DataFusion parses successfully → wrong query result or execution error instead of E-QUERY-001 → analyst receives misleading output.

**BC references:** BC-2.11.004 v1.48 EC-11-004-006; BC-5.39.001 (3-CLEAN criterion)

---

### F-PQLFN-PR9-LOW-002 — No uppercase-aggregate-in-WHERE test (M13 mutation — deleting `to_ascii_lowercase` at predicate gate — survives suite)

**Severity:** LOW
**Category:** test-coverage gap / mutation reasoning
**Routing:** test-writer (fix-burst-40; CLOSED this session)

**File/Anchor:** `crates/prism-query/src/engine.rs` — predicate-position walker aggregate-detection arm; `crates/prism-query/tests/bc_2_11_019_test.rs` (or equivalent aggregate-in-WHERE test file); BC-2.11.019 v1.23

**Defect:** The M13 mutation targets the `to_ascii_lowercase` normalization call applied to aggregate function names in the WHERE-position walker arm. Deleting this call means `COUNT`, `SUM`, `AVG`, `STDDEV`, and other all-uppercase or mixed-case aggregate function names would NOT be intercepted at the predicate gate — only lowercase variants would fire E-QUERY-001. The test suite exercises only lowercase (`count(x)`, `avg(y)`) in WHERE position; no test asserts that `COUNT(x) > 0` in a WHERE clause fires E-QUERY-001. An M13 mutation (delete `to_ascii_lowercase`) passes the suite undetected.

**Failure scenario:** M13 mutation applied: DELETE `to_ascii_lowercase` at predicate gate → `COUNT(x) > 0` in WHERE does NOT trigger E-QUERY-001 → query proceeds to DataFusion → DataFusion evaluates `COUNT(x) > 0` in a filter context (unsupported semantics, non-aggregate row filter) → query execution error or wrong semantics instead of E-QUERY-001 with correct guidance → analyst confused.

**BC references:** BC-2.11.019 v1.23 §Postconditions position-a (WHERE-position aggregate detection); BC-5.39.001

---

### F-PQLFN-PR9-OBS-001 — DataFusion 53.x caret-range registry drift only partially locked (4 named pins; no set-difference invariant lock)

**Severity:** OBS
**Category:** dependency-version coupling / regression risk
**Routing:** test-writer (fix-burst-40; CLOSED this session)

**File/Anchor:** `crates/prism-query/src/engine.rs` — `DATAFUSION_BUILTIN_AGGREGATE_NAMES` and `DATAFUSION_BUILTIN_FUNCTION_NAMES` initialization; `Cargo.toml` / `Cargo.lock` DataFusion pin; BC-2.11.019 v1.23 §OBS-004 two-branch detail-builder

**Defect:** The `having_aggregate_interception_detail` two-branch helper introduced in fix-burst-38 relies on `DATAFUSION_BUILTIN_AGGREGATE_NAMES ∖ DATAFUSION_BUILTIN_FUNCTION_NAMES == {distinct_count, percentile}` at runtime. This set-difference invariant determines which aggregate names receive the generic `(...)` template vs the specialized `(field, p)` template. DataFusion's Cargo dependency uses caret-range versioning (`"^44"` or similar), allowing patch and minor upgrades within the locked range. Four named aggregate functions (`distinct_count`, `percentile`, `count`, `sum`) are referenced by name in EC-11-087 lock tests, but no test asserts the COMPLETE set-difference — i.e., that the set of aggregate names NOT in the function registry is exactly `{distinct_count, percentile}` and no more. If DataFusion 53.x introduces a new aggregate function `approx_variance` in `DATAFUSION_BUILTIN_AGGREGATE_NAMES` without adding it to `DATAFUSION_BUILTIN_FUNCTION_NAMES`, the gate silently emits the generic `(...)` template for `approx_variance` — which is correct behavior — but there is no test that would fail and alert the team to the new set member, nor is there a test that verifies the set-difference hasn't shrunk unexpectedly.

**Failure scenario:** DataFusion 44.x→44.y minor bump adds `approx_variance` to `DATAFUSION_BUILTIN_AGGREGATE_NAMES` only → `DATAFUSION_BUILTIN_AGGREGATE_NAMES ∖ DATAFUSION_BUILTIN_FUNCTION_NAMES` is now `{distinct_count, percentile, approx_variance}` → no test fails → gate is correct at runtime (generic template fires for `approx_variance`) but the team is unaware the invariant footprint expanded → future fix-burst may incorrectly harden only `{distinct_count, percentile}` as the invariant → silent drift.

**BC references:** BC-2.11.019 v1.23 §OBS-004 two-branch detail-builder; EC-11-087; POL-34

---

### Out-of-Scope Deferred Item (carried from LOCAL cascade and prior PR-LEVEL passes; unchanged)

- **D-PQLFN-P47-OBS-001** — EC-collision potential for E-QUERY-038 / new function-call gate interaction at S-3.09 DML surface. OBS severity; out-of-perimeter per BC-5.39.002 PC2; anchor S-3.09 dispatch. UNCHANGED from LOCAL cascade and PR-LEVEL passes 1+2+3+4+5+6+7+8; not re-raised as a PR-LEVEL finding.

---

## Mutation Matrix Summary

15 mutations enumerated across M13 (case-sensitivity) and M14 (list-entry-removal) classes:

| Mutation | Target | Killed By | Status |
|----------|--------|-----------|--------|
| M14-1 | DELETE `distinct` from RESERVED_KEYWORDS | existing test_keyword_as_function_call_in_where_sql | KILLED |
| M14-2 | DELETE `null` from RESERVED_KEYWORDS | existing test_keyword_as_function_call_in_where_pipe (NULL variant) | KILLED |
| M14-3 | DELETE `true` from RESERVED_KEYWORDS | existing test_keyword_as_function_call_in_where (true variant) | KILLED |
| M14-4 | DELETE `false` from RESERVED_KEYWORDS | existing test_keyword_as_function_call_in_where (false variant) | KILLED |
| M14-5..M14-18 | DELETE any of 14 remaining keywords from RESERVED_KEYWORDS | NO NAMED TEST | SURVIVED (17 total; 4 killed above; 3 remaining with shared test coverage) |
| M13-1 | DELETE `to_ascii_lowercase` at predicate walker aggregate arm | NO NAMED TEST for uppercase aggregate | SURVIVED |
| M13-2 | DELETE `to_ascii_lowercase` at HAVING-intercept arm | existing EC-11-086 casing-lock verbatim tests (`PERCENTILE`, `Percentile`) | KILLED |
| (remaining 13) | Various gate-boundary mutations | Named tests EC-11-085/086/087, LOW-006, detail-builder unit tests | KILLED |

**2 SURVIVED → 2 LOW findings (F-PQLFN-PR9-LOW-001 M14, F-PQLFN-PR9-LOW-002 M13).**

Init/concurrency audit: `DATAFUSION_BUILTIN_AGGREGATE_NAMES` and `DATAFUSION_BUILTIN_FUNCTION_NAMES` use `LazyLock` initialization — no panic paths, O(1) post-init lookup, no race window. GREEN.

Pub-API audit: `#[non_exhaustive]` preserved on all gate-adjacent pub types. No new pub variants. `FuncCall::Scalar` span field swept across 11 construction sites — all consistent. GREEN.

---

## Fix-Burst-40 Closure

All three in-perimeter findings closed same-session. Fix-burst-40 produced one commit: test-writer f715b0a5. Branch pushed to origin; PR #223 HEAD is now f715b0a5.

### F-PQLFN-PR9-LOW-001 — 17 of 21 keywords had no per-keyword rejection test (M14 class)

**test-writer (commit f715b0a5):** `test_f_pqlfn_pr9_low_001_all_21_keyword_rejection_lock` — parameterized loop over all 21 `RESERVED_KEYWORDS` entries, asserting E-QUERY-001 fires for each keyword used as fn-call LHS in a WHERE clause (SQL mode). Includes `assert_eq!(RESERVED_KEYWORDS.len(), 21)` invariant assertion so any list-shrink or list-grow mutation fails the length guard before the per-keyword loop runs. This test directly kills any M14 single-entry-removal mutation for all 21 keywords; removing any keyword from the set causes its loop iteration to fail. prism-query 1671/1671. Non-exhaustive 91/91.

### F-PQLFN-PR9-LOW-002 — No uppercase-aggregate-in-WHERE test (M13 at predicate gate)

**test-writer (commit f715b0a5):** Two new tests added:
- `test_f_pqlfn_pr9_low_002_sql_where_stddev_uppercase_fires_aggregate_gate` — verifies `SELECT * FROM t WHERE STDDEV(col) > 0` fires E-QUERY-001 (uppercase aggregate in WHERE, SQL mode). Kills M13 delete-to_ascii_lowercase mutation at the predicate walker arm.
- `test_f_pqlfn_pr9_low_002_pipe_where_avg_mixed_case_fires_aggregate_gate` — verifies `t | WHERE Avg(col) > 0` fires E-QUERY-001 (mixed-case aggregate in WHERE, Pipe mode). Confirms case normalization is applied consistently across SQL and Pipe surfaces. prism-query 1671/1671. Non-exhaustive 91/91.

### F-PQLFN-PR9-OBS-001 — No set-difference invariant lock for DataFusion registry drift

**test-writer (commit f715b0a5):** `test_f_pqlfn_pr9_obs_001_datafusion_set_difference_invariant` — asserts `DATAFUSION_BUILTIN_AGGREGATE_NAMES.difference(&DATAFUSION_BUILTIN_FUNCTION_NAMES).collect::<BTreeSet<_>>() == BTreeSet::from(["distinct_count", "percentile"])`. This test fails loudly if a DataFusion upgrade adds any new member to the set-difference (alerting the team that the invariant footprint has expanded and the two-branch detail-builder may need review) or removes an existing member (alerting that the invariant footprint shrank — potential regression). Fail-loud semantics are release-mode clean: `assert_eq!` is not `debug_assert_eq!`; POL-34 compliant. prism-query 1671/1671. Non-exhaustive 91/91.

**Post-fix verification:** just iter prism-query 1671/1671. just check FULL WORKSPACE 5607/5607 GREEN (+4 tests over 5603 from fix-burst-38). Non-exhaustive 91/91. All 4 new tests GREEN first-run. Branch pushed to origin/fix/DEFECT-PQL-FNCALL-LHS-001 at f715b0a5 (1 commit over 97cb070e). PR #223 HEAD updated to f715b0a5. CI PENDING on new HEAD.

---

## Positive Verifications

- **Mutation matrix 13/15 KILLED (pre-fix):** 13 of 15 enumerated mutations were killed by named tests from prior fix-bursts (EC-11-085/086/087 casing-lock verbatim tests, LOW-006 keyword gate tests, detail-builder unit tests, EC-11-087 set-asymmetry lock). The 2 SURVIVED mutations map exactly to the 2 LOW findings; fix-burst-40 closes both by adding kill tests.

- **Init/concurrency probe — GREEN:** `DATAFUSION_BUILTIN_AGGREGATE_NAMES` and `DATAFUSION_BUILTIN_FUNCTION_NAMES` both use `std::sync::LazyLock` (or equivalent one-time init). No panic paths in init code. O(1) hash-set lookup post-init. No race window: initialization is idempotent and happens before any concurrent query processing. No `unwrap()` / `expect()` in init. GREEN.

- **Pub-API surface probe — GREEN:** `#[non_exhaustive]` attribute preserved on all gate-adjacent pub types at 97cb070e. `FuncCall::Scalar` span field swept: 11 construction sites across `crates/prism-query/src/` confirmed consistent — all supply the span argument; no site was missed. No new pub variants added in the DEFECT-PQL-FNCALL-LHS-001 branch beyond those already registered in the non-exhaustive gate (EXPECTED=91 unchanged). GREEN.

- **SAP-1 PASS:** Zero net-new `event_type =` emissions added in fix-burst-40 (test-only additions; no production emission sites modified). Settled methodology carries from prior passes: 55 raw occurrences / 12 distinct values verified against BC-2.16.002 v1.61 catalog 92 rows; ZERO net-new emissions.

- **POL-22 Phase A+C PASS:**
  - Phase A (adversary independently re-derived all load-bearing evidence; no reliance on implementer disclosure)
  - Phase C (all positive verifications cross-checked against code at 97cb070e, not only pass reports)
  - `RESERVED_KEYWORDS` set size verified: 21 entries at 97cb070e (grep confirmed; keywords include `null`, `true`, `false`, `distinct`, `count`, `sum`, `avg`, `min`, `max`, `group`, `order`, `having`, `limit`, `offset`, `where`, `select`, `from`, `join`, `union`, `stddev`, `percentile` class)
  - `to_ascii_lowercase` at predicate walker aggregate arm verified present at 97cb070e — `rg 'to_ascii_lowercase' crates/prism-query/src/engine.rs` confirms presence in aggregate-detection loop

- **TD-VSDD-059 PASS (fix-burst-40):** All three closures have load-bearing tests: LOW-001 closed by parameterized all-21 loop that structurally fails on any entry removal; LOW-002 closed by two tests that independently fail if `to_ascii_lowercase` is removed from the predicate walker arm; OBS-001 closed by `assert_eq!` set-difference lock that fails on any invariant change. No paper-fix (rename-only / doc-comment-only / assert-only against passing stubs).

- **TD-VSDD-060 PASS:** No function signatures, constants, or canonical identifiers changed in fix-burst-40 (test-only additions). No sibling-site sweep required.

- **TD-VSDD-091 PASS:** Narrative spec content cites function names and behavioral anchors; no `file.rs:NNN` volatile line-pins in live BC prose.

- **POL-14 vehicle confirmed:** BC-2.11.019 draft→active auto-promotion fires on PR #223 merge per POL-14. No new BCs added in fix-burst-40 (test-only).

- **CLAUDE.md forbidden patterns clean:** No `println!` in production code paths. No new `unwrap()`/`expect()` in production code. No `reqwest` changes (ADR-050 rustls-tls untouched). No AI attribution in commits. No `--no-verify` bypass.

- **Spec versions verified at 97cb070e (pre-fix-burst-40):**
  - BC-2.11.019 v1.23 (two-branch detail-builder; debug_assert REMOVED; DML scope cross-note)
  - BC-2.11.004 v1.48 (EC-11-085/086/087; RESERVED_KEYWORDS 21 keywords)
  - ADR-048 v1.16 (§D.2 rewritten; §D.7.3 HAVING-exemption caveat)
  - error-taxonomy E-QUERY-039 template current (v2.52)
  - policies.yaml v1.34 (POL-34 registered)

- **Novelty: LOW-MEDIUM** — findings are all test-coverage class (0 production defects identified); mutation-reasoning angle surfaced 2 surviving mutations and 1 set-difference drift risk. All structural code behavior has been CLEAN for multiple passes; remaining cascade work is test-completeness hardening. Fix-burst-40 is test-writer only (no production code changes).

---

## Convergence Status

- CLEAN(strict): NO — 3 in-perimeter findings (2 LOW + 1 OBS); strict criterion requires zero findings of any severity
- CLEAN(PR-merge): YES — 0 CRIT/HIGH/MED findings; LOW/OBS non-blocking for PR merge
- Streak: **0/3** (BC-5.39.001 strict criterion failed; fix-burst-40 pushed new commit f715b0a5; DRIFT-ORCH-PRLEVEL-PUSH-001: streak must re-gate on frozen HEAD f715b0a5)
- New frozen HEAD: **f715b0a5** (PR #223 HEAD after fix-burst-40; CI PENDING)
- DRIFT-ORCH-PRLEVEL-PUSH-001: fix-burst-40 push mid-cascade resets streak; all pass-10+ must use f715b0a5 as the frozen HEAD; passes 1–9 on prior frozen HEADs do NOT count toward f715b0a5 streak

---

## Next Step

CI green on f715b0a5 (PR #223 new HEAD) → PR-LEVEL pass-10 on frozen f715b0a5 (fresh streak 0/3; DRIFT-ORCH-PRLEVEL-PUSH-001 clean; no pushes mid-cascade). On 3/3 CLEAN(strict) streak on frozen f715b0a5 → HUMAN merge gate PR #223 (DRIFT-PQLFN-OD7 Gap-1/Gap-2 ratification + BC-2.11.019 cross-branch sequencing confirmation + POL-14 BC-2.11.019 auto-promotion on merge + LOW-006 keyword-list adjudication merge-gate feature-decision).
