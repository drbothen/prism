---
document_type: adversarial-review
scope: PR-LEVEL
fix_pr: DEFECT-PQL-FNCALL-LHS-001
passes: [12]
feature_head_at_review: 9c99e54f
date: 2026-07-15
clean_strict: false
clean_pr_merge: true
finding_counts:
  total: 5
  crit: 0
  high: 0
  med: 0
  low: 2
  obs: 3
  process_gap: 1
  out_of_scope_obs: 1
code_behavior_defects: 1
streak_after: 0/3
convergence: IN_PROGRESS
status: CLOSED
authored_by: orchestrator-relay
---

# PR-LEVEL Adversary Pass 12 — DEFECT-PQL-FNCALL-LHS-001

---

## Pass 12 (frozen 9c99e54f; fresh-context adversary; PR #223 PQL function-call LHS cascade; streak 0/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=YES

Streak: **0/3** (BC-5.39.001 strict criterion: pass-12 has 2 LOW + 3 OBS; LOW/OBS are non-blocking per CLEAN(PR-merge) criterion; CLEAN(strict) requires zero findings of ANY severity — 2 LOW + 3 OBS prevent strict CLEAN; fix-burst-43 pushed commit 168b2e96; DRIFT-ORCH-PRLEVEL-PUSH-001: re-gate on frozen HEAD 168b2e96 for all subsequent passes)

Cascade tally (as of this pass): **12 passes / 8 fix-bursts** (PR-LEVEL); **59 passes / 43 fix-bursts** (total)

CLEAN(strict): NO — 2 LOW + 3 OBS in-perimeter findings (0 CRIT / 0 HIGH / 0 MED)
CLEAN(PR-merge): YES — zero CRIT/HIGH/MED findings; LOW/OBS non-blocking under PR-merge criterion

**Priority fix-burst-42 surface verified CLEAN:** All fix-burst-42 closures (regex_match try_map span; EC-004 grammar widened; materialization detail path; semantic-flag normalization) positive-verified correct in this pass.

**Novelty: LOW** — findings are documentation/placement class with one residual code-behavior OBS (pre-existing non-semantic E-QUERY-001 prefix class).

---

## Findings

### F-PQLFN-PR12-LOW-001 — python gate section placement: ParseError gating under E0004 header but emits E0639; header counts stale (closed by fix-burst-43)

**Severity:** LOW
**Category:** doc-accuracy / test-infrastructure placement
**Routing:** implementer (fix-burst-43)

**Defect:** The `tests/external/non-exhaustive-violation/` Python gate script placed the `ParseError` `#[non_exhaustive]` check under the `E0004` subsection header (mismatched — E0004 is the `non-exhaustive patterns` error; the compile-fail gate for `ParseError` emits E0639 `cannot create non-exhaustive struct using struct expression`). Additionally, the section-level counts in the gate header comment were stale after the 91→92 advancement (header cited 91; actual expected is 92).

**Resolution:** gate script placement corrected (ParseError under E0639 section); header counts updated 91→92. fix-burst-43.

---

### F-PQLFN-PR12-LOW-002 — doc referenced non-existent ParseError::semantic constructor (closed by fix-burst-43)

**Severity:** LOW
**Category:** doc-accuracy / nonexistent API reference
**Routing:** implementer (fix-burst-43)

**Defect:** A doc comment in `error_recovery.rs` referenced `ParseError::semantic()` as a constructor — but fix-burst-42 added the `semantic: bool` field to the struct; it did NOT add a `ParseError::semantic()` free-standing constructor function. The comment misleads readers into expecting a constructor that does not exist. Unit tests exercised the struct initialization path directly but did not document the absence of a named constructor.

**Resolution:** `ParseError::semantic()` constructor added to `error_recovery.rs` with 3 unit tests. Constructor provides an ergonomic wrapper setting `semantic: true`. fix-burst-43.

---

### F-PQLFN-PR12-OBS-001 — residual SAME-code E-QUERY-001 doubling on ~20 pre-existing non-semantic manual-prefix sites (code-behavior; pre-existing class; cycle-close deferral per BC-2.11.006 v1.20 Option B) (closed by fix-burst-43)

**Severity:** OBS (code-behavior class; non-blocking per BC-5.39.001 strict criterion — OBS; out-of-scope per BC-2.11.006 v1.20 Option B)
**Category:** code-behavior / pre-existing prefix class
**Routing:** implementer (fix-burst-43; strip unified to both branches; OBS-001 RED lock count==2 proven → GREEN count==1)

**Defect:** Approximately 20 pre-existing sites in `filter_parser.rs` and `sql_parser.rs` used manual `format!("E-QUERY-001: {}", msg)` prefix construction (non-semantic errors that predate the semantic-flag cascade). These sites did not use `ParseError.semantic` and therefore passed through the detail builder's structural path — which calls `ParseError::Display`. Since these errors already contained the `E-QUERY-001:` prefix in their message, Display added it again, producing doubled `E-QUERY-001: E-QUERY-001:` output in approximately 20 code paths including the empty-query case.

This is a residual instance of the same-code doubling class that fix-burst-41 addressed for semantic errors (LOW-006 de-prefixing). BC-2.11.006 v1.20 ratified Option B (two-layer Display form for security-limit wrapped errors only); this class falls outside the Option B rationale.

**Resolution:** strip unified to both branches in the detail builder — semantic branch uses `e.message` (already correct after fix-burst-42); structural branch strips `E-QUERY-001: ` prefix before calling Display, preventing double-prefix for non-semantic pre-existing sites. OBS-001 RED lock (count==2 proven) → GREEN (count==1) after strip. fix-burst-43.

---

### F-PQLFN-PR12-OBS-002 — Display-lock simulator drift from production branch (closed by fix-burst-43)

**Severity:** OBS (test-infrastructure class; non-blocking)
**Category:** test drift / regression-lock fidelity
**Routing:** implementer (fix-burst-43)

**Defect:** Two regression lock tests (from earlier fix-bursts) still exercised Display output via a lightweight in-test simulator path that constructed `ParseError` fields manually rather than routing through `engine.execute()`. After fix-burst-42's semantic-flag changes, the simulator path and the production engine path diverged: the simulator did not exercise the full materialization chain, so the locks were nominally green but did not reflect the production code path.

**Resolution:** both regression locks converted to full `engine.execute()` path — same mechanism as the live code. fix-burst-43.

---

### F-PQLFN-PR12-OBS-003 — [process-gap] multi-error offset single-source: QueryParseFailed.offset = first error only; joined detail loses per-error offsets (deferred; cycle-close rule)

**Severity:** OBS (process-gap / deferred)
**Category:** process-gap / multi-error semantics
**Status:** DEFERRED (cycle-close rule — legacy pattern predating this cascade; out-of-scope per BC-2.11.006 v1.20 Option B code-layering only)

**Defect:** When `parse_query` returns multiple parse errors, `QueryParseFailed.offset` is set to the offset of the first error only. The joined detail string concatenates all error messages but loses the per-error offset values. A consumer debugging a complex syntax error cannot recover the position of the 2nd or Nth error. This is a multi-error offset semantics gap — not introduced by this cascade (the single-offset behavior predates fix-burst-1).

**Deferral rationale:** BC-2.11.006 v1.20 Option B scoped this cascade to code-layering only (the semantic-flag and Display normalization work). The multi-error offset design is a follow-up product decision (requires PO adjudication on whether multi-offset arrays belong in `QueryParseFailed`, or whether a new error structure is needed). Deferred per cycle-close rule.

**Drift item registered:** DRIFT-PQLFN-MULTIERR-OFFSET-001 (see STATE.md Drift Items). Target: follow-up story to be authored at wave-5-e cycle close (candidate S-PQL-MULTIERROR-OFFSET-001); owner PO.

---

## Fix-Burst-43 Summary

**Fix-burst-43 (implementer @168b2e96, TDD):**

1. **LOW-001 closure:** Gate script placement corrected (ParseError under E0639 section); header counts updated 91→92. Both gate scripts (check-non-exhaustive.sh + check-non-exhaustive-per-symbol.py) pass at 92/92 after correction.

2. **LOW-002 closure:** `ParseError::semantic()` constructor added to `error_recovery.rs` with 3 unit tests. Constructor sets `semantic: true`, `message: msg.to_string()`, `span: None`, `label: None`.

3. **OBS-001 closure:** Strip unified to both branches. Structural branch strips `E-QUERY-001: ` prefix before Display call. OBS-001 RED lock (count==2 proven) flipped to GREEN (count==1). All ~20 pre-existing non-semantic sites now emit single-prefixed output.

4. **OBS-002 closure:** Both regression locks converted to full `engine.execute()` path.

**Test counts:** 5623/5623 (+4 over 5619: 1 `ParseError::semantic()` constructor unit test + 3 tests from OBS-001 strip RED→GREEN locks and OBS-002 regression-lock promotions); non-exhaustive gate 92/92; `just check` GREEN.

**PUSHED:** 9c99e54f → 168b2e96 to origin/fix/DEFECT-PQL-FNCALL-LHS-001. PR #223 HEAD now 168b2e96; CI pending; streak 0/3 re-gates on frozen 168b2e96; pass-13 NEXT.

---

## Positive Verifications

- **Fix-burst-42 surface fully verified CLEAN:** regex_match try_map span placement correct; EC-004 try_map reachable via widened grammar; materialization detail uses `e.message` for semantic errors with correct strip_prefix; `ParseError.semantic` flag set/consumed correctly at all 4 sites (construction, propagation, materialization, has_semantic_error). No additional consumption sites found.

- **SAP-1 PASS:** Zero net-new `event_type =` emissions in fix-burst-43. No production emission sites added or modified.

- **TD-VSDD-059 PASS (fix-burst-43):** All 4 closures have load-bearing artifacts: LOW-001 gate script is structural (section placement + count); LOW-002 constructor is callable and 3 tests enforce signature; OBS-001 strip is structural (both branches unified; count==1 lock); OBS-002 locks exercise production path (not simulator).

- **TD-VSDD-060 PASS (fix-burst-43):** `ParseError::semantic()` constructor — new function, single definition site; no sibling callsites (constructor is additive, not a signature change). Strip-prefix change in structural branch — single-site in materialization.rs; no callsites outside the detail builder.

- **BC-5.39.001 CLEAN(PR-merge) criterion:** ZERO CRIT/HIGH/MED findings. PR-merge gate criterion satisfied. Does NOT advance CLEAN(strict) streak — 2 LOW + 3 OBS (including OBS-003 which is deferred) prevent strict CLEAN. Streak remains 0/3 on new frozen 168b2e96.

- **DRIFT-ORCH-PRLEVEL-PUSH-001:** fix-burst-43 push mid-cascade resets strict streak. All pass-13+ must use 168b2e96 as frozen HEAD. Passes 1–12 on prior frozen HEADs do NOT count toward 168b2e96 streak.

- **OBS-003 deferral compliant:** DRIFT-PQLFN-MULTIERR-OFFSET-001 registered with concrete future-story target (S-PQL-MULTIERROR-OFFSET-001 candidate; wave-5-e cycle close; PO owner). Canonical Principle Rule 3 satisfied: explicit deferral, concrete dependency (BC-2.11.006 v1.20 Option B scope), specific future story anchor.

---

## Convergence Status

- CLEAN(strict): NO — 2 LOW + 3 OBS in-perimeter findings; strict criterion requires zero findings of any severity
- CLEAN(PR-merge): YES — zero CRIT/HIGH/MED findings; LOW/OBS non-blocking
- Streak: **0/3** (CLEAN(strict)=NO; fix-burst-43 pushed 168b2e96; DRIFT-ORCH-PRLEVEL-PUSH-001: streak re-gates on frozen HEAD 168b2e96)
- New frozen HEAD: **168b2e96** (PR #223 HEAD after fix-burst-43; CI pending on 168b2e96)
- DRIFT-ORCH-PRLEVEL-PUSH-001: fix-burst-43 push mid-cascade resets streak; all pass-13+ must use 168b2e96 as frozen HEAD

---

## Next Step

CI green on 168b2e96 (PR #223 new HEAD) → PR-LEVEL pass-13 on frozen 168b2e96 (fresh streak 0/3; DRIFT-ORCH-PRLEVEL-PUSH-001 clean; no pushes mid-cascade). On 3/3 CLEAN(strict) streak on frozen 168b2e96 → HUMAN merge gate PR #223 (DRIFT-PQLFN-OD7 Gap-1/Gap-2 ratification + BC-2.11.019 cross-branch sequencing confirmation + POL-14 BC-2.11.019 auto-promotion on merge + LOW-006 keyword-list adjudication merge-gate feature-decision).
