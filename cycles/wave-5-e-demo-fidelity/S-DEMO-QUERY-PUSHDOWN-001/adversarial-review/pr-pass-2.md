---
document_type: adversarial-review-report
scope: PR-LEVEL
story_id: S-DEMO-QUERY-PUSHDOWN-001
pr_number: 173
pass_number: 2
cascade: PR-LEVEL (distinct from LOCAL; LOCAL converged at pass 11 @69aafcc7)
base_develop: "752e407a"
feature_head_at_review: "f290a43d"
feature_head_after_fix_burst: "eab62613"
clean_strict: false
clean_pr_merge: true
streak_after: "0/3"
produced: 2026-06-05
authority: BC-5.39.001 D-779
---

# PR-LEVEL Adversary Pass 2 — S-DEMO-QUERY-PUSHDOWN-001

**Story:** S-DEMO-QUERY-PUSHDOWN-001 — Push-Down Query Fidelity (Phase B Lane 2)
**PR:** #173 (base develop@752e407a, head f290a43d at review)
**Pass:** PR-LEVEL pass 2 (distinct from LOCAL cascade; LOCAL CONVERGED @69aafcc7 passes 9/10/11)
**Date:** 2026-06-05

## Pass-1 Closure Verification

All 4 pass-1 closures verified LOAD-BEARING at HEAD f290a43d:

| Closure | Verification |
|---------|-------------|
| OBS-1 (BC-2.16.002 version-agnostic cite) | BC-2.16.002 v1.68 — both cross-reference sites read `S-DEMO-QUERY-PUSHDOWN-001 EC-003` (no `v2.2` pin); version-agnostic form present and load-bearing |
| SEC-002 (doc note predicate_tree_to_filter_map) | Doc note present in `crates/prism-query/src/pushdown.rs` `predicate_tree_to_filter_map`; security invariant documented |
| SEC-004 (64-char length cap + unit tests) | `extract_fql_bound` (prism-dtu-crowdstrike state.rs) and `extract_aql_keyword_bound` (prism-dtu-armis search.rs) both have 64-char cap; 2 unit tests present confirming oversized input returns `None` |
| SEC-007 (hardening candidate — not blocking) | Correctly left open as architect-owned adjudication; DRIFT-D1016-SEC-007 recorded; does not block |

**Pass-1 closure class confirmed closed.** No regression on any closed finding.

## Adversary Pass 2 Findings

### ADV-PR-P02-LOW-001 — Demo Evidence Test-Count Drift

**Finding ID:** ADV-PR-P02-LOW-001
**Severity:** LOW
**Category:** Demo evidence accuracy (POLICY 32 evidence-count parity)

**Description:** The SEC-004 fix-burst (f290a43d) added 2 new unit tests to prism-dtu-crowdstrike (extract_fql_bound length cap) and 1 new unit test to prism-dtu-armis (extract_aql_keyword_bound length cap), bringing:
- CrowdStrike total tests cited in demo evidence-report.md: 7 → 8
- Armis total tests cited in demo evidence-report.md: 4 → 5

The `docs/demo-evidence/S-DEMO-QUERY-PUSHDOWN-001/evidence-report.md` test-count fields reflected the pre-fix-burst state (7/4). Evidence reports must stay current with the implementation they document.

**Note on SEC-004 classification:** SEC-004 unit tests are defense-in-depth hardening tests on length-cap guards in DTU parsers. These tests do NOT correspond to behavioral contract ACs in BC-2.01.013/BC-2.11.007/BC-2.11.005. They are security-hardening-only. SEC-004 is correctly classified as a SUGGESTION (non-AC defense-in-depth) — the label in the pass-1 report stands. This finding is solely about the evidence-report count fields being stale.

**Root cause:** Fix-burst added tests after evidence-report.md was produced. Evidence-report counts represent a snapshot; code changes after evidence generation cause count drift.

**Closure:** CLOSED by demo-recorder fix-burst.
- `docs/demo-evidence/S-DEMO-QUERY-PUSHDOWN-001/evidence-report.md` test counts refreshed to CrowdStrike=8, Armis=5.
- 2 DTU demo recordings re-recorded to reflect updated test suites (CrowdStrike DTU, Armis DTU sections).
- SEC-004 labeled defense-in-depth (not ACs) in evidence report notes.
- Feature HEAD f290a43d → eab62613 (docs/demo-evidence only; no production code change).

## Summary

**CLEAN(strict):** no (1 LOW ADV-PR-P02-LOW-001; not zero)
**CLEAN(PR-merge):** yes (0 CRIT + 0 HIGH + 0 MED; LOW is non-blocking for merge)
**Streak:** 0/3 (never advanced this PR-LEVEL cascade — pass 1 had OBS-1 LOW, pass 2 has ADV-PR-P02-LOW-001 LOW)
**Feature HEAD after fix-burst:** eab62613
**Next step:** PR-LEVEL pass 3

## Axes Checked

| Axis | Result | Notes |
|------|--------|-------|
| Correctness (BC-2.01.013, BC-2.11.007, BC-2.11.005) | PASS | All pass-1 closures verified load-bearing; no regression |
| Test strength (SAP-1 + SAP-2) | PASS | SAP-1: catalog 71 rows, no unregistered event_type. SAP-2: DTU↔TOML parity confirmed |
| SEC-004 classification | CONFIRMED — defense-in-depth (not ACs) | Hardening guards on length caps; non-AC tests; correctly non-blocking for BC convergence |
| Demo evidence accuracy | FAIL → FIXED | Test-count drift (7→8 CrowdStrike, 4→5 Armis) corrected by demo-recorder at eab62613 |
| Wiring (Arc-DI, ADR-022) | PASS | No regressions from fix-burst |
| Security (per pass-1 CLEAR-TO-MERGE) | PASS | No new security surface added by demo-evidence-only fix-burst |
