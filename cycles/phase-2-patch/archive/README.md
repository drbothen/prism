# Phase 2 Patch Cycle — Archive

Historical artifacts from the Phase 2 patch cycle (2026-04-16 through 2026-04-21, 99 adversarial passes, 20-burst remediation). Retained for provenance; not on the hot path for future agents or human readers.

## Subdirectory map

### `pre-patch-cycle-reviews/`
Adversarial review reports pass-24 through pass-58 (Phase 2 convergence pre-patch-cycle). Older naming convention `pass-NN.md` (newer convention is `adversary-pass-NN.md`). 35 files.

### `pre-convergence-remediation/`
Transient per-pass and per-wave remediation plans from p59 through p74, plus BC-wave and stories-wave remediation plans. One-off dispatches that produced commits — the committed work is authoritative; these plans are provenance. 37 files.

### `step5-drift-rebaseline/`
Step 5 DTU consistency sweep + input-hash drift rebaseline reports (2026-04-20 rebaseline under vsdd-factory v0.47.0), plus step5 track-a/b remediation plans. 5 files.

### `remediation-scripts/`
One-off bash scripts from p70/p73 (BC changelog normalization, changelog reordering, frontmatter version sync). Provenance only; not to be re-run.

## Hot-path artifacts (parent dir, not in archive)

- `adversary-pass-59.md` through `adversary-pass-99.md` (40 reports; p73 never existed)
- `INDEX.md` — cycle status + pass-by-pass summary
- `burst-log.md` — per-burst historical record
- `convergence-trajectory.md` — full finding/trajectory table
- `lessons.md` — cross-cycle learnings
- `blocking-issues-resolved.md` — closed blocker log
- `phase-steps-p59-p79.md` — pre-rebaseline phase steps snapshot
