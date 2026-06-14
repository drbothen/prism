---
document_type: pr-adversary-pass-report
pass: 6
story: S-DEMO-MULTI-TENANT-DTU-001
pr: 187
pr_head_at_review: eb77316f
feature_branch: feature/S-DEMO-MULTI-TENANT-DTU-001
base_branch: develop
develop_head_at_review: f7400f83
clean_strict: "NO"
clean_pr_merge: "YES"
streak_before: "1/3"
streak_after: "0/3 (streak RESET — 1 LOW OBS-PR6-1 found; CLOSED eb77316f)"
date: 2026-06-14
producer: adversary (D-1157)
---

# PR-LEVEL Adversary Pass 6 — S-DEMO-MULTI-TENANT-DTU-001 PR #187

## Summary

**CLEAN(strict): NO** (1 LOW OBS-PR6-1 found; CLOSED in-scope; streak RESET to 0/3)
**CLEAN(PR-merge): YES** (LOW only; non-blocking for merge gate)

Pass 6 is a full fresh-context adversarial review of the complete PR diff at HEAD
41d093fe (BC v1.10 / story v1.14). One LOW finding identified in the non-exhaustive gate
crate comment block: the gate-count arithmetic comment in `struct_violations.rs` / `main.rs`
misreported the EXPECTED count transition. CLOSED at commit eb77316f.

---

## Full Fresh-Context Adversarial Review

### OBS-PR6-1 [LOW] Gate-Count Comment Arithmetic Incorrect in `struct_violations.rs` and `main.rs`

**Severity:** LOW
**Status:** CLOSED (commit eb77316f; grep-clean confirmed)

**Finding:**

The non-exhaustive compile-fail gate (`tests/external/perimeter-violation/`) contains a
comment block describing the gate-count transition introduced by this story's
`MultiInstanceServers` addition. The comment read:

```
// 54-59: 6 E0639 + 1 E0004 → 7 total
```

The actual count in the `struct_violations.rs` file after D-1145 added `MultiInstanceServers`
is: arms 54–61, comprising 7 E0639 arms + 1 E0004 arm = 8 new test arms in this story's
addition. The baseline before this story was EXPECTED=52. After this story EXPECTED=60
(52 + 8 = 60). The comment "6 E0639 + 1 E0004 → 7 total" was arithmetically wrong on both
the E0639 count (6 vs actual 7) and the total (7 vs actual 8).

Correspondingly, `main.rs` diagnostic message "All 49 types"/"All 52 types" (legacy from
earlier passes) had not been updated to reflect the current 60-type baseline.

This is a comment/documentation accuracy finding (gate behavior is correct; EXPECTED=60 is
correct in `ci.yml`; only the inline comment block was wrong).

**Resolution:**

Implementer corrected `struct_violations.rs` comment block to:

```
// 54-61: 7 E0639 + 1 E0004 → 8 new; cumulative 52→60
```

`main.rs` diagnostic updated from "49 types" to "60 types". Grep-clean confirmed. No
change to actual gate arms or EXPECTED value.

Commit: eb77316f. `just check` GREEN.

---

## Checklist Sweep

- [x] **SAP-1**: no new `event_type=` emissions. CLEAN.
- [x] **INV-PERIMETER-001**: no change to perimeter gate logic. CLEAN.
- [x] **Gate EXPECTED=60**: unchanged (ci.yml authoritative; only comment corrected). CLEAN.
- [x] **POL-32**: changelogs descending. CLEAN.
- [x] **SID-1**: no new ignored tests. CLEAN.
- [x] **TD-VSDD-091**: no volatile SHA in body prose. CLEAN.
- [x] **unwrap/expect in production paths**: none. CLEAN.

---

## Streak Status

PR-LEVEL streak RESETS: **1/3 → 0/3** (LOW OBS-PR6-1 present — per BC-5.39.001
CLEAN(strict) requires ZERO findings of ANY severity for streak advancement).

New HEAD after fix: **eb77316f**.

**NEXT:** PR-LEVEL adversary Pass 7. Full fresh-context pass on complete PR diff at HEAD
eb77316f. Need 3 consecutive CLEAN(strict) passes for BC-5.39.001 PR-LEVEL convergence.
