# PR-LEVEL Adversary Pass 2 — S-DEMO-MULTI-TENANT-DTU-001

**Date:** 2026-06-14
**Story:** S-DEMO-MULTI-TENANT-DTU-001 — Per-DTU-Instance Multi-Address Binding for Multi-Tenant Overlay Testing
**PR:** #187 (feature/S-DEMO-MULTI-TENANT-DTU-001 → develop)
**PR HEAD at pass time:** 846c21dc (on remote)
**develop HEAD:** f7400f83
**Story version entering this pass:** v1.11 (post-OBS-2 fix; F-PR2-MED-001 closes → v1.12)
**BC-2.06.017 version entering this pass:** v1.7 (F-PR2-MED-001 closes → v1.8)
**BC-5.39.001 streak entering this pass:** 0/3
**Adversary context:** PR-LEVEL (fresh context; second pass; information asymmetry)

---

## Part A — SEC-006 Redaction Verification (Pass 1 Finding Closure Check)

**Finding:** SEC-006 — CWE-209 error message disclosed raw input value in `validate_harness_key` rejection path.
**Fix commit:** 846c21dc
**Adversary verification:**
- `validate_harness_key` error return now emits truncated/redacted form: input longer than 32 characters is displayed as `[<first-16-chars>...<REDACTED>]`; input ≤ 32 characters that contains a violation character is emitted with the violating character replaced by `[REDACTED]`.
- `HarnessError::InvalidKey` Display impl confirmed redacting.
- Unit tests `test_validate_harness_key_long_input_redacted` and `test_validate_harness_key_violation_char_redacted` confirm both redaction paths with load-bearing assertions.
- AD-017 (credentials never transit AI context) satisfied: harness keys are structural identifiers, but defensive redaction closes CWE-209 regardless of input type.

**SEC-006 CLOSED — load-bearing. Pass 1 finding verified.**

---

## Part B — PR Diff Adversarial Review (Full Pass 2 Scope)

### Findings

#### F-PR2-MED-001 — MEDIUM — Changelog Ascending Violation (POL-32)

**Severity:** MEDIUM
**Finding ID:** F-PR2-MED-001
**POL violated:** POL-32 — changelogs are monotonic-descending (newest revision at top, oldest at bottom)
**Scope:** Story S-DEMO-MULTI-TENANT-DTU-001 changelog + BC-2.06.017 changelog
**Description:**

The story and BC changelogs both showed ASCENDING order — version 1.0 at the top, growing downward toward the current version. This is the opposite of POL-32 monotonic-descending requirement.

Story changelog (as found in v1.11):
```
| v1.0  | 2026-06-09 | Initial draft |
| v1.1  | 2026-06-09 | ... |
| ...
| v1.11 | 2026-06-14 | OBS-2 timeout wording fix |
```
Top row = v1.0 (oldest). This is ASCENDING — POL-32 violation.

BC-2.06.017 changelog (as found in v1.7):
```
| v1.0  | 2026-06-09 | Initial draft |
| v1.1  | 2026-06-09 | ... |
| ...
| v1.7  | 2026-06-13 | ... |
```
Same pattern — ASCENDING — POL-32 violation.

**Context note:** LOCAL adversary passes 1-11 (11 total) did NOT catch this. Pass 9 (D-1153) explicitly stated "story 1.10→1.0 descending" — this was a FALSE-PASS. The LOCAL adversary appears to have checked that the changelog version numbers were monotonic (i.e., they form an ordered sequence) without verifying that the TOP row holds the HIGHEST (newest) version, which is the POL-32 direction requirement. PR-LEVEL fresh context, reading the actual top-to-bottom text, caught the direction error.

**Root cause of LOCAL false-pass:** LOCAL adversary confused "monotonically ordered" with "monotonically descending." A changelog with rows 1.0, 1.1, ... 1.10 is monotonically ordered (each version is greater than the prior) but ASCENDING (older versions are at the top). POL-32 requires the newest at the top.

**Fix:** Reverse the changelog table order in both story and BC so that the current version appears first (top row) and v1.0 appears last (bottom row). All row content is unchanged; only the order of rows is reversed.

**Status:** CLOSED — story-writer reversed story changelog (v1.11→v1.12; newest at top); product-owner reversed BC changelog (v1.7→v1.8; newest at top). Lesson z22 codified (POL-32 ascending-changelog systemic miss).

---

## Verdicts

| Criterion | Result |
|-----------|--------|
| CLEAN (strict) — ZERO findings ANY severity | NO (F-PR2-MED-001 MEDIUM) |
| CLEAN (PR-merge) — ZERO findings CRIT+HIGH+MED | NO (F-PR2-MED-001 MEDIUM) |
| BC-5.39.001 streak advance | NO |

**CLEAN(strict):** NO
**CLEAN(PR-merge):** NO (MEDIUM finding)
**Streak after pass 2:** 0/3 (RESET — MEDIUM finding requires fix-burst before Pass 3)

---

## Fix-Burst Required Before Pass 3

- story-writer: reverse story changelog → v1.11→v1.12
- product-owner: reverse BC-2.06.017 changelog → v1.7→v1.8
- state-manager: bump STORY-INDEX v2.378→v2.379 (story v1.12); BC-INDEX v6.50→v6.51 (BC v1.8); STATE D-1154 decision row

Pass 3 scope: verify changelog reorder correct (top row = v1.12 / v1.8; bottom row = v1.0); reverify all Pass 1 + Pass 2 closures sound; full fresh-context adversarial pass on complete PR diff.
