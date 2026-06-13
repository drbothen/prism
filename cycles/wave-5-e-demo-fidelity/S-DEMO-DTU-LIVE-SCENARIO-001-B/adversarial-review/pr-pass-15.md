---
document_type: adversarial-review-pass
pass: 15
level: PR-LEVEL
story: S-DEMO-DTU-LIVE-SCENARIO-001-B
pr: 185
head: 7ddc0a51
timestamp: 2026-06-13T02:00:00Z
streak_before: 0/3
streak_after: 0/3
clean_strict: false
clean_pr_merge: true
findings_count: 1
finding_ids: [BPRL-P15-01]
closure_burst: D-1121
---

# PR-LEVEL Pass 15 — S-DEMO-DTU-LIVE-SCENARIO-001-B

**Pass:** 15 | **PR:** #185 | **HEAD:** 7ddc0a51 (CODE UNCHANGED — no code commits since D-1118)
**Streak before:** 0/3 | **Streak after:** 0/3 (BPRL-P15-01 resets)
**CLEAN(strict):** no | **CLEAN(PR-merge):** yes

---

## Finding: BPRL-P15-01 MED (SPEC-ONLY)

**ID:** BPRL-P15-01
**Severity:** MED
**Category:** SPEC-ONLY (story prose — Phase-6 gate instruction stale RGT count)
**File:** `.factory/stories/S-DEMO-DTU-LIVE-SCENARIO-001-B-scenario-progression-enrichment.md`
**Location:** §Phase 6 gate check instruction, line ~581 (exact line per story v2.12)

**Finding:** The story's Phase-6 gate instruction contained the phrase "all 19 Red Gate tests pass" — a stale count from the pre-D-1117 story revision (v2.9 and earlier had 19 RGTs). The canonical count is 23 (established D-1117: AC-019 added VP-020-I..VP-020-L + 4 new RGTs; red_gate_tests frontmatter = 23; Phase-6 gate table in story = 23 rows). A literal verifier following the gate instruction could declare the gate passed after running only 19 of the 23 RGTs, silently skipping the 4 cyberint-correlation tests (VP-020-I, VP-020-J, VP-020-K, VP-020-L).

**Axes verified clean:**
- BPRL-P14-01 closure verified fully propagated: BC-2.06.020 v1.4 PC-9 directive reads `0..10000`; story B AC-019 literal reads `0..10000`; `^CVE-9999-\d{4}$` invariant, TV-020-011, and code all consistent. DO NOT REFLAG.
- base_score consistency: NvdClone fixture base_score values >= 7.0 matches assertion in VP-020-K test (demo-server). PASS.
- H1/title sync: story H1 and STORY-INDEX title cell consistent. PASS.
- SAP-1: `grep -r 'event_type\s*=' crates/ --type rust` — zero new unregistered event_type values in PR diff. PASS.
- SAP-2: no sensor TOML changes in diff. N/A.
- Changelog descending: story B §Changelog entries descend monotonically from v2.12 down to v1.0. PASS.
- BC-INDEX rows 119/120 anchor story pin: both carry `ready v2.12 (D-1120 2026-06-13)`. PASS (stale pin was the subject of this pass; see closure below).

---

## Closure: BPRL-P15-01 (SPEC-ONLY — D-1121)

**Scope:** Story prose only. No code change. No CI trigger.
**Agent:** story-writer (story B §Phase-6 gate instruction prose)

**Fix:** Story B v2.12→v2.13 — line ~581 gate instruction "all 19 Red Gate tests pass" corrected to "all 23 Red Gate tests pass".

**Sweep evidence (exhaustive \b19\b/\b18\b classification):**

All occurrences of `\b19\b` and `\b18\b` in story B classified before and after fix:

| Location | Value | Classification | Action |
|----------|-------|----------------|--------|
| `acceptance_criteria_count: 19` (frontmatter) | 19 | AC count — correct, unchanged | Leave |
| `red_gate_tests: 23` (frontmatter) | 23 | RGT count — already correct | Leave |
| Phase-6 gate instruction "all 19 Red Gate tests pass" | 19 | STALE RGT gate count | FIXED → 23 |
| RGT table header "23 Red Gate Tests" | 23 | Correct count | Leave |
| RGT table rows RGT-1 through RGT-23 | 1–23 | Individual test row numbers | Leave |
| AC index labels "AC-001" through "AC-019" | 1–19 | AC identifier labels | Leave |
| "19 ACs" in §Token Budget | 19 | AC count annotation — correct | Leave |
| Changelog entries referencing v2.11/v2.12 "19 ACs / 23 RGTs" | 19, 23 | Historical changelog prose — immutable | Leave |

**Confirmed:** Line ~581 was the SOLE stale gate-count prose. All other `19` occurrences in the story file refer to the AC count (19 correct and unchanged) or are RGT row-index labels. Zero false-positive changes.

**BC-INDEX sweep (POL-29 / POL-z8 class):** BC-INDEX rows 119 and 120 carry `anchor story S-DEMO-DTU-LIVE-SCENARIO-001-B ready v2.12 (D-1120 2026-06-13)`. Story B version advances to v2.13 → both rows swept to `ready v2.13 (D-1121 2026-06-13)`. BC versions UNCHANGED (BC-2.06.019 stays v1.7; BC-2.06.020 stays v1.4). BC-INDEX v6.39→v6.40.

---

## Do-Not-Reflag Additions (D-1121)

- **BPRL-P15-01 CLOSED:** Story B Phase-6 gate instruction now reads "all 23 Red Gate tests pass". DO NOT re-raise "gate instruction says 19" or "Phase-6 gate skips cyberint-correlation RGTs."

## Pass Status

CLEAN(strict): no (BPRL-P15-01 MED resets streak)
CLEAN(PR-merge): yes (zero CRIT/HIGH/MED findings remaining after closure — BPRL-P15-01 was the sole finding and is SPEC-ONLY / story prose)
Streak: 0/3 → 0/3
NEXT: PR-LEVEL pass 16 at 7ddc0a51 (diff UNCHANGED — CODE UNCHANGED; story B v2.13 spec corrected)
