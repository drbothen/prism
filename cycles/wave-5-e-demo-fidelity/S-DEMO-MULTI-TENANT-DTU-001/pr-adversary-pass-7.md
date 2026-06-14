---
document_type: pr-adversary-pass-report
pass: 7
story: S-DEMO-MULTI-TENANT-DTU-001
pr: 187
pr_head_at_review: 2746f878
feature_branch: feature/S-DEMO-MULTI-TENANT-DTU-001
base_branch: develop
develop_head_at_review: f7400f83
clean_strict: "NO"
clean_pr_merge: "YES"
streak_before: "0/3"
streak_after: "0/3 (streak RESET — 1 LOW OBS-PR7-1 found; CLOSED 2746f878)"
date: 2026-06-14
producer: adversary (D-1157)
---

# PR-LEVEL Adversary Pass 7 — S-DEMO-MULTI-TENANT-DTU-001 PR #187

## Summary

**CLEAN(strict): NO** (1 LOW OBS-PR7-1 found; CLOSED in-scope; streak RESET to 0/3)
**CLEAN(PR-merge): YES** (LOW only; non-blocking for merge gate)

Pass 7 is a full fresh-context adversarial review of the complete PR diff at HEAD
eb77316f (post OBS-PR6-1 gate-count comment fix). One LOW finding identified: 2 stale
present-tense "EXPECTED=52" comments remaining in the `struct_violations.rs` `v51` block
predating this story. A prior pass (Pass 6) corrected the *new* comment block for arms
54–61; it did not sweep *all* extant count references in the file. CLOSED at commit
2746f878 via exhaustive grep-driven sweep.

---

## Full Fresh-Context Adversarial Review

### OBS-PR7-1 [LOW] Stale Present-Tense `EXPECTED=52` Comments in `struct_violations.rs` v51 Block

**Severity:** LOW
**Status:** CLOSED (commit 2746f878; post-sweep zero stale present-tense gate-count refs confirmed)

**Finding:**

After the Pass-6 OBS-PR6-1 fix corrected the *new* comment block for arms 54–61 to
"52→60", a fresh-context read of `struct_violations.rs` reveals 2 residual present-tense
comments in the existing `v51` block (arms added in a prior story, before this PR) that
still read:

```rust
// EXPECTED=52 (current total)
```

These comments are stale: the current total after this story is EXPECTED=60, not 52.
They are present-tense claims ("current total") which are now factually false. Historical
"EXPECTED was 52 before this story" prose is permissible; "EXPECTED=52 (current total)"
in a file where the current total is 60 is a documentation accuracy gap.

The root cause is that Pass-6 performed a targeted fix of the *new* story comment block
but did not execute a file-wide sweep of all present-tense gate-count references per
TD-VSDD-060 sibling-site sweep discipline.

**Resolution:**

Implementer ran an exhaustive grep of `struct_violations.rs` and `main.rs` for all
present-tense gate-count prose:

```bash
grep -n "EXPECTED=52\|current total\|52 types" \
  tests/external/perimeter-violation/src/struct_violations.rs \
  tests/external/perimeter-violation/src/main.rs
```

Both occurrences in the `v51` block updated to "EXPECTED=60 (current total)" with a
parenthetical note "(52 before S-DEMO-MULTI-TENANT-DTU-001)". Post-sweep grep-clean:
zero remaining "EXPECTED=52" + "current total" combinations. `just check` GREEN.

Commit: 2746f878.

---

## Checklist Sweep

- [x] **SAP-1**: no new `event_type=` emissions. CLEAN.
- [x] **INV-PERIMETER-001**: no change to gate arms or perimeter logic. CLEAN.
- [x] **Gate EXPECTED=60**: unchanged in ci.yml; comment-only correction. CLEAN.
- [x] **POL-32**: changelogs descending. CLEAN.
- [x] **SID-1**: no new ignored tests. CLEAN.
- [x] **TD-VSDD-091**: no volatile SHA in body prose. CLEAN.
- [x] **TD-VSDD-060** (sibling-site sweep): exhaustive grep confirmed — zero residual
  stale present-tense gate-count refs after fix. CLEAN.
- [x] **unwrap/expect in production paths**: none. CLEAN.

---

## Streak Status

PR-LEVEL streak RESETS: **0/3 → 0/3** (LOW OBS-PR7-1 present — per BC-5.39.001
CLEAN(strict) requires ZERO findings of ANY severity for streak advancement).

New HEAD after fix: **2746f878**.

**NEXT:** PR-LEVEL adversary Pass 8. Full fresh-context pass on complete PR diff at HEAD
2746f878. Gate-comment class confirmed exhausted (44+16=60 arithmetic exact; sweep
complete). Need 3 consecutive CLEAN(strict) passes for BC-5.39.001 PR-LEVEL convergence.
