---
document_type: adversarial-review
level: ops
version: "1.0"
status: complete
producer: adversary
timestamp: 2026-07-17T10:00:00Z
phase: 3
inputs: [S-MAINT-CI-DISK-EXHAUSTION-001-ci-disk-exhaustion-hardening.md]
input-hash: "8e9f2da"
traces_to: S-MAINT-CI-DISK-EXHAUSTION-001-ci-disk-exhaustion-hardening.md
pass: 13
previous_review: pr-level-pass-12.md
---

# Adversarial Review: S-MAINT-CI-DISK-EXHAUSTION-001 (Pass 13 — Reconstruction)

## Purpose

This file reconstructs the PR-LEVEL pass-13 adversarial review for
S-MAINT-CI-DISK-EXHAUSTION-001. The original pass-13 report was dispatched
during the D-1796 session (2026-07-17) on story v0.21 frozen @faf112fd, but was
lost at session wrap — only one-liner summaries (2L+1OBS) survived in
STATE.md/SESSION-HANDOFF.md. The full report detail, including severity
classifications and exact finding descriptions, was not persisted.

This reconstruction review re-derived all spec-side findings with full detail
using fresh-context adversarial analysis against story v0.21 @faf112fd.
The reconstruction identified 2 MED findings (vs the lost pass-13's apparent 2 LOW),
illustrating that lost detail also means lost severity fidelity.

**Important:** This is NOT a streak-numbered pass. The streak remains 0/3 on
frozen faf112fd. The next numbered pass is PR-LEVEL pass-14, to be dispatched
after the human reopens PR #224.

## Finding ID Convention

Finding IDs for this reconstruction use the format `F-CIDISK-RECON-<SEV>-<SEQ>`
(non-standard prefix to distinguish reconstructed findings from streak-counted
ADV-WAVE5-P13-* findings that would have appeared in the original report).

## Part A — Fix Verification

Pass-12 had one LOW finding (F-MAINT-P12-LOW-001); closed in story v0.21 before
this pass-13 review. All pass-12 closures verified correct.

| ID | Previous Severity | Status | Notes |
|----|-------------------|--------|-------|
| F-MAINT-P12-LOW-001 | LOW | RESOLVED | story v0.21 applied the fix; grep verified |

## Part B — New Findings (or all findings for pass 1)

### MEDIUM

#### F-CIDISK-RECON-MED-001: Test-matrix reclaimer YAML snippet missing `if: runner.os == 'Linux'` guard

- **Severity:** MED
- **Category:** spec-fidelity (spec more permissive than code; 3-site POL-29 gap)
- **Location:** story v0.21 §AC-002 YAML snippet + intro prose + §Tasks bullet
- **Description:** The §AC-002 test-matrix acceptance-criteria YAML snippet for
  the reclaimer step omits `if: runner.os == 'Linux'` that is present in the
  implemented ci.yml at approximately line 118. The test-matrix job is a
  mixed-OS matrix (Linux + Windows/macOS); `insightsengineering/disk-space-reclaimer`
  is Linux-only, so the guard is load-bearing. Additionally, §AC-002 intro prose
  claims the snippet "differs only in step name" from the test-no-default-features
  leg — this is false (two differences: step name AND `if:` guard). The
  test-no-default-features leg is an ubuntu-only job and is correctly unconditional.
  POL-29 gap: 3 sites — snippet body, intro prose, §Tasks reclaimer bullet.
  Code layer is CORRECT; spec under-documents code.
- **Evidence:** ci.yml ~line 118 contains `if: runner.os == 'Linux'` on the
  reclaimer step for the test-matrix job; story v0.21 §AC-002 snippet lacks this
  line; story intro prose uses "differs only in step name."
- **Proposed Fix:** (a) Add `if: runner.os == 'Linux'` to YAML snippet immediately
  after `- name: Reclaim disk space (Linux only)`; (b) rewrite intro prose to
  describe both differences; (c) update §Tasks bullet to require the guard for
  the test-matrix leg. POL-29 sweep: one other site (neutralization step note)
  already correctly describes the guard; no change needed there.

#### F-CIDISK-RECON-MED-002: STORY-INDEX row stale at v0.18 vs story v0.21

- **Severity:** MED
- **Category:** spec-fidelity (POL-13 index-truth drift)
- **Location:** .factory/stories/STORY-INDEX.md — S-MAINT-CI-DISK-EXHAUSTION-001 row
- **Description:** The STORY-INDEX.md row showed the newest version marker as
  `**ready v0.18**` (b54af749 2026-07-16), but the on-disk story at frozen HEAD
  faf112fd was v0.21. Versions v0.19, v0.20, and v0.21 were not registered. This
  hides three significant scope expansions from the index view: v0.19 (full
  fallback redesign), v0.20 (apt-spy2 neutralization + RG-8 + EC-016), v0.21
  (`|| true` guards + five-operations fix).
- **Evidence:** story frontmatter `version: "0.21"`; STORY-INDEX row newest
  marker `**ready v0.18**`.
- **Proposed Fix:** State-manager appends v0.19, v0.20, v0.21, v0.22 markers to
  the STORY-INDEX row in the D-1797 burst.

### LOW

#### F-CIDISK-RECON-LOW-001: §Token Budget Estimate v0.1-era stale (~11k vs actual ~82k)

- **Severity:** LOW
- **Category:** spec-fidelity (stale metadata)
- **Location:** story v0.21 §Token Budget Estimate section
- **Description:** §Token Budget Estimate retains v0.1-era values (~3 k spec /
  ~11 k total). Actual story at v0.21 is ~1,295 lines; ci.yml at faf112fd is
  ~2,055 lines. Realistic estimate is ~42 k spec / ~82 k total. The "Well within
  Claude's 200k context window" claim is misleadingly optimistic — the story is
  workable within the window with selective reading, not "well within" with full
  loading.
- **Evidence:** story v0.21 §Token Budget: "~3 k spec / ~11 k total"; actual
  line count substantially higher.
- **Proposed Fix:** Update §Token Budget Estimate to ~42 k spec / ~82 k total;
  change "Well within" to "Workable within" with selective-read guidance.

### OBS

#### F-CIDISK-RECON-OBS-001: AC-006 "five ordered operations" vs EC-010 six-step enumeration

- **Severity:** OBS
- **Category:** spec-fidelity (internal countable mismatch)
- **Location:** story v0.21 §AC-006 vs §EC-010
- **Description:** §AC-006 describes five ordered operations (Steps 1-5); §EC-010
  enumerates six items, listing dpkg-repair and retry as separate items 5 and 6.
  Not load-bearing (no RG assertion enforces step count), but creates reader
  confusion.
- **Proposed Fix:** Option A — keep AC-006 five-operation framing canonical;
  unify EC-010 by merging items (5) dpkg-repair + (6) retry into one step
  via " + retry" phrasing.

#### F-CIDISK-RECON-OBS-002: Non-monotonic echo-bump deltas at 3 sites

- **Severity:** OBS
- **Category:** spec-fidelity (documentation clarity)
- **Location:** story v0.21 §Tasks — 3 echo-bump instruction sites
- **Description:** Three echo-bump instruction sites carry numeric deltas
  (18→19, 16→17, 17→18) that are non-monotonic when read in document order.
  The deltas assume canonical ordered execution sequence in §Tasks (resolves to
  22 total), not document order. Applying document-order gives nonsensical
  sequence 19→16→17.
- **Proposed Fix:** Add one-line note at each of three sites clarifying that
  the delta assumes canonical §Tasks sequence (resolves to 22 total) and must
  not be applied in document order.

#### F-CIDISK-RECON-OBS-003: Volatile line-number pin in e2e.yml scope paragraph (TD-VSDD-091)

- **Severity:** OBS
- **Category:** spec-fidelity (TD-VSDD-091 volatile-pin)
- **Location:** story v0.21 §AC-006 e2e scope extension paragraph
- **Description:** The phrase "at line 104 of pre-rebase e2e.yml" is a volatile
  line-number pin per TD-VSDD-091. After subsequent rebases or e2e.yml edits the
  step is no longer at line 104. The step-name anchor is the correct non-volatile
  reference.
- **Proposed Fix:** Remove "at line 104 of pre-rebase e2e.yml"; retain step-name
  anchor.

## Summary

| Severity | Count |
|----------|-------|
| CRITICAL | 0 |
| HIGH | 0 |
| MEDIUM | 2 |
| LOW | 1 |
| OBS | 3 |

**Overall Assessment:** block (2 MED findings)
**Convergence:** FINDINGS_REMAIN — 2 MED + 1 LOW + 3 OBS; all closed in D-1797 burst (story v0.22)
**Readiness:** requires revision (spec layer only; code layer CLEAN)

## Verification Positives

- All 8 story grep counts reproduced against frozen worktree @faf112fd
  (12/12/2/2/2/2/1/1 for pre-existing reachability, AC-001, AC-002, AC-007,
  AC-003, RG-7, RG-8, AC-006 fallback respectively)
- Byte-verbatim YAML snippet spot-checks pass for all 13 fallback blocks
- POL-21 phantom-anchor: clean (no undefined ID anchors)
- SAP-1 tracing emission catalog: clean (no new event_type sites in spec)
- No code-layer findings — all 6 findings are spec-layer only

## Resolution Summary (D-1797 burst)

| Finding | Severity | Closed by | How |
|---------|----------|-----------|-----|
| F-CIDISK-RECON-MED-001 | MED | story-writer | story v0.22 — snippet + intro + §Tasks fix |
| F-CIDISK-RECON-MED-002 | MED | state-manager | STORY-INDEX v2.695→v2.696 — v0.19/v0.20/v0.21/v0.22 markers |
| F-CIDISK-RECON-LOW-001 | LOW | story-writer | story v0.22 — Token Budget updated |
| F-CIDISK-RECON-OBS-001 | OBS | story-writer | story v0.22 — EC-010 unified to 5 steps |
| F-CIDISK-RECON-OBS-002 | OBS | story-writer | story v0.22 — delta-ordering notes added |
| F-CIDISK-RECON-OBS-003 | OBS | story-writer | story v0.22 — TD-VSDD-091 volatile pin removed |

## Novelty Assessment

| Field | Value |
|-------|-------|
| **Pass** | 13 (reconstruction) |
| **New findings** | 6 |
| **Duplicate/variant findings** | 0 |
| **Novelty score** | 6/6 = 1.0 (reconstruction; not a streak pass) |
| **Median severity** | MED |
| **Trajectory** | →9→3→0→2 (prior streak passes); reconstruction not streak-counted |
| **Verdict** | FINDINGS_REMAIN (all closed in D-1797 burst; next numbered pass = pass-14 after PR #224 reopen) |
