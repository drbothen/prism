---
document_type: story
story_id: "S-AUDIT-REVIEW-AXES-001"
title: "Audit script standing adversary review axes — NYA per-stub error-code classification and sort-assertion divergence-window"
wave: maintenance
epic_id: maintenance
priority: P3
status: draft
version: "0.2"
spec_version: "v0.2"
level: ops
producer: story-writer
timestamp: "2026-07-12"
modified: "2026-07-12"
input-hash: ""
inputs:
  - scripts/t13-preflight-audit.py
traces_to: "F-AUD-P26-OBS-002, F-AUD-P28-OBS-005"
origin_finding: "F-AUD-P26-OBS-002 [process-gap] + F-AUD-P28-OBS-005 [process-gap]"
origin_cascade: "AUDIT-COVERAGE-001 B-hardening; D-1697 (passes 26–28); LOCAL 3-CLEAN converged D-1713 (2026-07-12)"
cycle: "v1.0.0-greenfield"
phase: 3
tdd_mode: strict
track: "Platform Engineering"
subsystems: []
crates_touched: []
target_module: "scripts/t13-preflight-audit.py"
behavioral_contracts: []
# BC status: pending PO authorship
# These two process-gap items govern standing adversary review axes for audit scripts.
# No pre-existing BC covers adversarial review discipline for audit-script NYA classification
# or ordering-assertion quality.
# PO must author a BC before this story can advance to status: ready (S-7.01 gate).
verification_properties: []
depends_on: [S-AUDIT-PROCESS-CONVENTIONS-001]
blocks: []
points: 2
estimated_days: 0.5
risk: LOW
acceptance_criteria_count: 2
red_gate_tests: 0
estimated_passes: "1"
holdout_scenarios: []
assumption_validations: []
risk_mitigations: []
---

# S-AUDIT-REVIEW-AXES-001: Audit script standing adversary review axes — NYA per-stub classification and sort-assertion divergence-window

## §Origin — [process-gap] F-AUD-P26-OBS-002 + F-AUD-P28-OBS-005

**Cascade:** AUDIT-COVERAGE-001 B-hardening; findings surfaced at passes 26 and 28
**Session record:** D-1697 (SESSION WRAP passes 26–28 + fix-bursts 26–28; queue items 7 and 8 added)
**Convergence:** LOCAL 3-CLEAN(strict) D-1713 (2026-07-12); S-7.02 codification gate now due

Two standing adversary review axes were identified during the AUDIT-COVERAGE-001 cascade that apply
whenever audit script checks are added or modified. These are intended for inclusion in the
`audit-conventions.md` document (created by S-AUDIT-PROCESS-CONVENTIONS-001) as named review axes
an adversary or code-reviewer must apply.

1. **F-AUD-P26-OBS-002 — A23 per-stub NYA-code classification review axis.** The A23 dynamic
   not-yet-available (NYA) tool sweep verifies that each NYA stub returns the expected fast-fail
   error code. Prior to pass 26, the adversary applied a set-level check (all stubs return some
   NYA-like code) rather than per-stub individual verification. The correct adversary posture is:
   for each tool name in the NYA set, verify that its specific returned error code (`-32003` or the
   registered not-yet-available pattern) is verified individually. Set-level checks can mask a
   single stub returning the wrong code while the rest are correct.

2. **F-AUD-P28-OBS-005 — Sort-assertion divergence-window review axis.** Pass 28 found that C7's
   ordering assertion used `LIMIT 5` in the data probe but the sort discrimination relied on
   detection severity levels where all 5 lowest-severity rows might have the same severity value —
   meaning the sort assertion was vacuously true within the probed window. The review axis: any
   ordering assertion must be checked for lex-vs-numeric divergence windows in the asserted data
   range. Specifically, an adversary must ask: "Is there any data range within the probed `LIMIT N`
   where lex sort and numeric sort produce the same order, making the assertion vacuous?" The fix
   was to use `LIMIT 12` to include the severity-level boundary where order actually diverges.

This story adds these two review axes to `scripts/audit-conventions.md` as a §Standing-Review-Axes
section.

## Narrative

As a Prism adversary or code-reviewer examining a PR that adds or modifies audit checks in
`scripts/t13-preflight-audit.py`, I want named standing review axes for (a) NYA per-stub
classification checks and (b) sort-assertion divergence-window analysis, so that I apply these
analyses systematically rather than discovering them by chance in individual adversary passes.

## Authority

No numbered ADR governs standing adversary review axes for audit scripts. The governing authorities for this story are:

**Origin findings:** F-AUD-P26-OBS-002 and F-AUD-P28-OBS-005 (AUDIT-COVERAGE-001 cascade, passes 26 and 28) are the two process-gap findings that triggered this story. Session record D-1697 (passes 26–28) contains the authoritative finding texts: set-level NYA error-code checks are insufficient for per-stub verification (SAR-1), and sort-assertion divergence-window analysis was not systematically applied (SAR-2 — the `LIMIT 5` vs `LIMIT 12` boundary where sort order actually diverges).

**CLAUDE.md §Standing Adversary Probes & Implementer Disciplines** (SAP-1, SAP-2, SAP-3) codifies the project-level standing probes for adversarial review. SAR-1 (NYA per-stub classification) and SAR-2 (sort-assertion divergence-window) in this story are audit-script-specific review axes analogous to SAP-2 (DTU↔TOML schema parity) — both operate on the principle of per-item individual verification rather than set-level aggregates.

**S-AUDIT-PROCESS-CONVENTIONS-001** (prerequisite story) creates `scripts/audit-conventions.md` — the file that AC-001 and AC-002 extend with the `§Standing-Review-Axes` section.

No product BCs govern standing adversary review axes. The `behavioral_contracts: []` status is intentional per the frontmatter note; PO authorship required before `status: ready` (S-7.01).

---

## Behavioral Contracts

No active BCs govern standing adversary review axes for audit scripts. PO authorship required
before status=ready.

## Acceptance Criteria

### AC-001 — NYA per-stub classification review axis codified in audit-conventions.md §Standing-Review-Axes
(pending BC trace — BC authorship required before status=ready)

`scripts/audit-conventions.md` §Standing-Review-Axes contains:

> **SAR-1 — NYA per-stub error-code classification (F-AUD-P26-OBS-002)**
>
> When reviewing any PR that modifies the not-yet-available (NYA) tool sweep (audit check A23 or
> equivalent), verify per-stub individually:
>
> For each tool name `t` in the NYA set:
> - Does the check extract `t`'s specific returned code/message?
> - Does the check assert that `t`'s code/message matches the expected pattern (`-32003` / not-yet-available message)?
>
> Set-level assertions ("all NYA stubs return SOME error code") are insufficient — they cannot
> detect one stub returning the wrong code while others are correct.
>
> Finding class: PROCESS-GAP (adversary); cite F-AUD-P26-OBS-002 if set-level check found in a PR.

### AC-002 — Sort-assertion divergence-window review axis codified in audit-conventions.md §Standing-Review-Axes
(pending BC trace — BC authorship required before status=ready)

`scripts/audit-conventions.md` §Standing-Review-Axes contains:

> **SAR-2 — Sort-assertion divergence-window check (F-AUD-P28-OBS-005)**
>
> When reviewing any PR that adds or modifies an ordering assertion (e.g., `assert rows[0][field] < rows[1][field]`
> or `assert rows == sorted(rows, key=...)`), verify:
>
> 1. What is the data range being probed (LIMIT N or full result set)?
> 2. Are there any rows within that range where lex sort and numeric sort produce the same ordering,
>    making the assertion vacuously true regardless of the actual sort implementation?
> 3. Does the probed window include at least one pair of adjacent rows whose order would DIFFER
>    between a correct and an incorrect sort implementation?
>
> If the window is vacuous (all probed rows have the same sort key value, or lex and numeric are
> identical in the range), the assertion must be expanded to a window that includes a divergence point.
>
> Example (C7): `LIMIT 5` on detection severity returned only `HIGH`-severity rows — lex and numeric
> sort identical. Fix: `LIMIT 12` includes the `HIGH/MEDIUM` boundary where order diverges.
>
> Finding class: LOW (vacuous assertion); cite F-AUD-P28-OBS-005 if divergence-window gap found.

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| audit-conventions.md §Standing-Review-Axes | `scripts/audit-conventions.md` | Pure data (convention document) |

Architecture section references: N/A — scripts-only, no Rust crates.

**Anchor justifications:**
- No subsystem anchor: `scripts/` tooling is not in the ARCH-INDEX Subsystem Registry.
- `depends_on: [S-AUDIT-PROCESS-CONVENTIONS-001]`: AC-001 and AC-002 add a §Standing-Review-Axes
  section to `scripts/audit-conventions.md`, which is created by S-AUDIT-PROCESS-CONVENTIONS-001.
  If that story has not run, the target file does not exist.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | The NYA tool set changes (tools added or removed) | SAR-1 is about the review axis structure, not the specific tool list; the axis applies to whatever NYA set exists at review time |
| EC-002 | A sort assertion uses a full result set (no LIMIT) | SAR-2 still applies: verify the full result set contains at least one divergence pair |
| EC-003 | The data seed for a sorted probe does not include any divergence pairs | The data seed itself is deficient; the reviewer must flag this as a seed design issue |

## Token Budget Estimate

| Item | Lines | Tokens (est.) |
|------|-------|--------------|
| Story spec (this file) | ~100 | ~1,400 |
| scripts/audit-conventions.md (§Standing-Review-Axes addition) | ~40 | ~560 |
| **Total estimate** | | **~1,960 tokens** |

Fits within a 100k-token agent context window (<2%). No split required.

## Tasks

- [ ] Read D-1697 SESSION WRAP entry to confirm the F-AUD-P26-OBS-002 and F-AUD-P28-OBS-005 finding descriptions are accurately reflected in this story's ACs.
- [ ] Confirm `scripts/audit-conventions.md` was created by S-AUDIT-PROCESS-CONVENTIONS-001 (this story's `depends_on` prerequisite).
- [ ] Add §Standing-Review-Axes section to `scripts/audit-conventions.md` with SAR-1 and SAR-2 entries (AC-001, AC-002).
- [ ] Verify `py_compile scripts/t13-preflight-audit.py` remains clean (no changes to the script itself in this story).

## Previous Story Intelligence

**Prerequisite story:** S-AUDIT-PROCESS-CONVENTIONS-001 creates `scripts/audit-conventions.md`.
This story extends it with a §Standing-Review-Axes section. Implement only after
S-AUDIT-PROCESS-CONVENTIONS-001 is complete.

Prior cascade context:
- C7 LIMIT-5 vacuous sort finding (F-AUD-P28-OBS-005): fix-burst 28 changed `LIMIT 5` → `LIMIT 12`
  to cross the HIGH/MEDIUM boundary; this is the exemplar for SAR-2.
- A23 per-stub NYA finding (F-AUD-P26-OBS-002): pass-26 adversary found set-level A23 check
  inadequate; subsequent fix-burst 26 added individual per-stub verification.

## Architecture Compliance Rules

- **TD-VSDD-091:** Cite function/check names (A23, C7), NOT file/line numbers.
- **py_compile gate:** No changes to `t13-preflight-audit.py` in this story; validate only that
  `scripts/audit-conventions.md` is valid Markdown with no broken structure.

## Library & Framework Requirements

| Library | Version | Notes |
|---------|---------|-------|
| Python | 3.x (workspace standard) | Used only for py_compile validation |

No new dependencies.

**Forbidden dependencies:** None applicable.

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `scripts/audit-conventions.md` | Modify | Add §Standing-Review-Axes section with SAR-1 + SAR-2 |

No Rust files, no Cargo.toml, no new scripts.

## Changelog

| Version | Burst | Date | Author | Changes |
|---------|-------|------|--------|---------|
| 0.2 | DRIFT-STORY-AUTHORITY-ABSENT-CORPUS-001-R6 | 2026-08-02 | story-writer | Add §Authority section (D-2084 Round 6 DRIFT-STORY-AUTHORITY-ABSENT-CORPUS-001). No numbered ADR governs; authority is origin findings F-AUD-P26-OBS-002/F-AUD-P28-OBS-005, CLAUDE.md §Standing Adversary Probes, and S-AUDIT-PROCESS-CONVENTIONS-001 prerequisite. |
| 0.1 | — | 2026-07-12 | story-writer | Initial story creation. |
