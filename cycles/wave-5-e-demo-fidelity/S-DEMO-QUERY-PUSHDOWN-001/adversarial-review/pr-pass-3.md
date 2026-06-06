---
document_type: adversarial-review-report
scope: PR-LEVEL
story_id: S-DEMO-QUERY-PUSHDOWN-001
pr_number: 173
pass_number: 3
cascade: PR-LEVEL (distinct from LOCAL; LOCAL converged at pass 11 @69aafcc7)
base_develop: "752e407a"
feature_head_at_review: "eab62613"
feature_head_after_fix_burst: "eab62613"
clean_strict: false
clean_pr_merge: false
streak_after: "0/3"
produced: 2026-06-05
authority: BC-5.39.001 D-779
---

# PR-LEVEL Adversary Pass 3 — S-DEMO-QUERY-PUSHDOWN-001

**Story:** S-DEMO-QUERY-PUSHDOWN-001 — Push-Down Query Fidelity (Phase B Lane 2)
**PR:** #173 (base develop@752e407a, head eab62613 at review)
**Pass:** PR-LEVEL pass 3 (distinct from LOCAL cascade; LOCAL CONVERGED @69aafcc7 passes 9/10/11)
**Date:** 2026-06-05

## Pass-2 Closure Verification

ADV-PR-P02-LOW-001 (demo evidence test-count drift) verified LOAD-BEARING at HEAD eab62613:
- `docs/demo-evidence/S-DEMO-QUERY-PUSHDOWN-001/evidence-report.md` now shows CrowdStrike=8, Armis=5.
- 2 DTU demo sections re-recorded reflecting updated test suites.
- SEC-004 defense-in-depth label present.
- No production code changes in f290a43d→eab62613 delta.

**Pass-2 closure confirmed load-bearing.**

## Adversary Pass 3 Findings

### ADV-P173-P03-MED-001 — BC-2.16.002 §Changelog Non-Monotonic + Duplicate 1.64 + Missing 1.63

**Finding ID:** ADV-P173-P03-MED-001
**Severity:** MEDIUM
**Category:** BC changelog structural integrity (POLICY 32 monotonic-descending-no-gaps requirement)

**Description:** BC-2.16.002 §Changelog, as of PR #173's last fix-burst prepending entries for v1.66/v1.67/v1.68, contained the following structural violations when audited against POLICY 32:

1. **Duplicate version 1.64**: Two distinct changelog rows were both labeled `1.64` — one from a prior story's fix-burst, one from this story's SAP-1 burst. POLICY 32 requires unique version numbers in descending order with no gaps.

2. **Missing version 1.63**: The sequence jumped from 1.62 directly to 1.64 (first instance), leaving 1.63 absent. POLICY 32 requires contiguous descending sequence with no gaps.

3. **1.41/1.42 inversion**: Rows labeled 1.41 and 1.42 appeared in reversed order (1.41 was listed before 1.42 in the descending sequence, violating monotone descent).

**Pre-existing vs hygiene-on-touch classification:** These violations are pre-existing in BC-2.16.002 (introduced prior to PR #173; the duplicate 1.64 and missing 1.63 originated in the S-DEMO-002 cascade; the 1.41/1.42 inversion is older). However, PR #173 prepended new changelog rows (v1.66/v1.67/v1.68) to BC-2.16.002. POLICY 32's hygiene-on-touch principle applies: when a PR touches a BC changelog, the full changelog must be clean upon departure. Deferring known structural violations past a touching PR violates POLICY 32.

**Root cause:** BC-2.16.002 changelog accumulated without a full monotonic sweep. S-DEMO-002 cascade introduced the duplicate 1.64 collision; the missing 1.63 was a labeling error in the same burst. The 1.41/1.42 inversion is an older artifact.

**Closure:** CLOSED by product-owner. Full-column POLICY 32 sweep of all 70+ rows:
- Duplicate 1.64 resolved: the mislabeled second 1.64 renumbered to 1.63 (the logically correct version it should have been).
- Missing 1.63 gap filled by the renaming above.
- 1.41/1.42 inversion corrected: rows reordered to strict descending order.
- Complete changelog now strict-descending, no duplicates, no gaps across all 70 rows.
- BC-2.16.002 version bumped v1.68 → v1.69 (changelog sweep is a substantive correction per POL-11).
- BC-INDEX frontmatter v5.88 → v5.89 (BC version bump per POL-11).
- Feature HEAD remains eab62613 (product-owner fixed `.factory/` specs only; no feature branch code change needed).

**Note on streak:** This finding is MEDIUM severity, which means CLEAN(PR-merge)=no and CLEAN(strict)=no for this pass. The streak remains 0/3 — it never advanced in this PR-LEVEL cascade (pass 1 had LOW, pass 2 had LOW, pass 3 had MEDIUM). However, the finding was entirely in BC changelog structure (docs/demo-evidence + BC files); feature code at eab62613 is clean.

## Summary

**CLEAN(strict):** no (1 MEDIUM ADV-P173-P03-MED-001)
**CLEAN(PR-merge):** no (MEDIUM finding present; blocks this pass from counting as PR-merge-clean)
**Streak:** 0/3 (streak never advanced; MEDIUM finding resets any prospective count)
**Feature HEAD:** eab62613 (unchanged — product-owner fixed .factory specs only)
**BC-2.16.002:** v1.69 (full POLICY 32 sweep complete; 70 rows strict-descending, no duplicates, no gaps)
**BC-INDEX:** v5.89
**Next step:** PR-LEVEL pass 4 on frozen eab62613 (fresh streak attempt; code is clean; remaining findings were evidence/BC-hygiene)

## Axes Checked

| Axis | Result | Notes |
|------|--------|-------|
| Correctness (BC-2.01.013, BC-2.11.007, BC-2.11.005) | PASS | All prior closures verified load-bearing; pass-2 closure confirmed |
| Test strength (SAP-1 + SAP-2) | PASS | SAP-1: catalog 71 rows, no unregistered event_type. SAP-2: DTU↔TOML parity confirmed |
| BC changelog structural integrity (POLICY 32) | FAIL → FIXED | Full 70-row sweep; duplicate 1.64→1.63 renaming; 1.41/1.42 inversion corrected; BC v1.68→v1.69; BC-INDEX v5.88→v5.89 |
| POLICY 27 (ISO dates in changelog) | PASS | All changelog dates confirmed ISO8601 format |
| POLICY 1 (append-only changelog) | PASS | No historical rows deleted; only renumbering of mislabeled rows and reordering of inverted rows |
| Demo evidence accuracy | PASS | Pass-2 evidence-count fix confirmed still accurate at eab62613 |
| Wiring (Arc-DI, ADR-022) | PASS | No regressions |
| Security | PASS (still CLEAR-TO-MERGE per pass-1 verdict) | No new security surface in BC changelog fixes |
