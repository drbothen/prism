---
document_type: story
story_id: "S-AUDIT-LINT-001"
title: "Python lint gate for scripts/ — ruff F821 undefined-name check in Justfile and CI"
wave: maintenance
epic_id: maintenance
priority: P2
status: draft
version: "0.1"
spec_version: "v0.1"
level: ops
producer: story-writer
timestamp: "2026-07-12"
modified: "2026-07-12"
input-hash: ""
inputs:
  - scripts/t13-preflight-audit.py
  - Justfile
  - .github/workflows/ci.yml
traces_to: "F-AUD-P24-OBS-001"
origin_finding: "F-AUD-P24-OBS-001 [process-gap]"
origin_cascade: "AUDIT-COVERAGE-001 B-hardening; D-1696 (passes 22–25); LOCAL 3-CLEAN converged D-1713 (2026-07-12)"
cycle: "v1.0.0-greenfield"
phase: 3
tdd_mode: strict
track: "Platform Engineering"
subsystems: []
crates_touched: []
target_module: "Justfile"
behavioral_contracts: []
# BC status: pending PO authorship
# F-AUD-P24-OBS-001 targets toolchain/CI quality, not a product behavioral contract.
# PO may author a BC covering CI quality gates, or this story may be implemented
# as a pure process-improvement without a BC (common for toolchain stories).
# Status must remain draft until either a BC is authored (S-7.01 gate) or
# the PO explicitly waives the BC requirement for toolchain stories.
verification_properties: []
depends_on: []
blocks: []
points: 3
estimated_days: 0.5
risk: LOW
acceptance_criteria_count: 4
red_gate_tests: 0
estimated_passes: "1-2"
holdout_scenarios: []
assumption_validations: []
risk_mitigations: []
---

# S-AUDIT-LINT-001: Python lint gate for scripts/ — ruff F821 undefined-name check in Justfile and CI

## §Origin — [process-gap] F-AUD-P24-OBS-001

**Cascade:** AUDIT-COVERAGE-001 B-hardening; finding surfaced at pass 24
**Session record:** D-1696 (SESSION WRAP passes 22–25; P24 CRIT TD-VSDD-060 NameError recurrence)
**Convergence:** LOCAL 3-CLEAN(strict) D-1713 (2026-07-12); S-7.02 codification gate now due

At pass 24 of the AUDIT-COVERAGE-001 cascade, an adversary found a **CRIT** finding:
a `NameError` in `scripts/t13-preflight-audit.py` was breaking approximately 82 audit checks
silently (most of the coverage matrix effectively skipped). This was a TD-VSDD-060 sibling-site
sweep gap: a rename operation at pass-23 missed a call site that referenced the old name. The
NameError was not caught until the adversary ran the full script.

The root cause (F-AUD-P24-OBS-001): `scripts/*.py` files have no static undefined-name checker
in the CI pipeline or in local developer tooling (`just` targets). `pyflakes` (already run
manually during fix-bursts) produces F821 "undefined name" warnings, but it is not gated in CI.
The `ruff` linter's `--select F821` is a minimal, fast check that would have caught this
`NameError` at commit time without requiring a full test run.

This story adds `ruff` (at minimum, the `--select F821` rule set) as a `just` target and
as a CI step for `scripts/*.py` files.

## Narrative

As a Prism developer committing changes to `scripts/*.py`, I want a fast `ruff check --select F821`
gate in `just check` and CI that catches undefined-name errors before the script is run, so that
a rename/refactor that misses a call site is detected at commit time rather than being found by
an adversary during a multi-hour cascade review.

## Behavioral Contracts

No active BCs govern CI linting quality. See frontmatter note.

## Acceptance Criteria

### AC-001 — ruff installed and version-pinned as a dev-tool dependency
(pending BC trace — BC authorship or PO waiver required before status=ready)

`ruff` is available in the developer environment. The workspace `dev-setup.sh` (or equivalent)
installs `ruff` via `pip install ruff` (or `cargo install ruff` if the workspace uses cargo-based
tools). The exact version pin is documented in `docs/dev-setup.md` and/or a
`scripts/requirements-dev.txt`. A minimum acceptable version is `ruff>=0.1.0` (F821 check is
available since 0.0.x). The implementer picks the current stable version and pins it.

### AC-002 — Justfile `lint-scripts` target added
(pending BC trace — BC authorship or PO waiver required before status=ready)

`Justfile` gains a `lint-scripts` recipe:

```makefile
# Lint scripts/ Python files for undefined names (F821) and other fast checks.
# Prevents NameError regressions from rename operations (F-AUD-P24-OBS-001).
lint-scripts:
    ruff check --select F821,F401,F811 scripts/
```

The rule set `F821,F401,F811` covers:
- `F821`: undefined name (the primary gate)
- `F401`: unused import (secondary; flags imports orphaned by refactors)
- `F811`: redefined-while-unused (secondary; catches accidental double-definitions)

The `lint-scripts` target is added to the existing `check` recipe as a step that runs after
`just fmt` and before or after `just clippy`. Exact ordering within `check` is at implementer
discretion; the constraint is that `just check` must fail if `lint-scripts` fails.

### AC-003 — CI workflow runs ruff check on scripts/
(pending BC trace — BC authorship or PO waiver required before status=ready)

`.github/workflows/ci.yml` gains a step in the appropriate job (the job that runs `just check`
or equivalent workspace validation) that executes `ruff check --select F821,F401,F811 scripts/`.
The step must run on both `ubuntu-latest` (primary CI runner) and any other runners that the
existing `check` job runs on. If ruff is not available in the CI runner image, it is installed
via `pip install ruff=={pinned-version}` in a `before` step.

The CI step must fail the job on any F821 violation. F401/F811 violations may be set to
`--exit-zero` if the implementer determines that the current codebase has pre-existing F401/F811
violations that would require a larger cleanup pass; in that case, F821 is the only blocking
check and the others are informational. This trade-off decision must be documented in an inline
comment in `ci.yml`.

### AC-004 — All existing scripts/*.py files pass ruff F821 with zero violations
(pending BC trace — BC authorship or PO waiver required before status=ready)

After adding the lint gate, all files currently in `scripts/` pass `ruff check --select F821 scripts/`
with zero F821 violations. If any violations exist, they must be fixed as part of this story
(not deferred). The pass-24 NameError has already been fixed by the cascade (fix-burst in D-1696);
this AC confirms no new violations were introduced and that the gate was retroactively clean at
the time of implementation.

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| ruff lint gate | `Justfile`, `.github/workflows/ci.yml` | Pure toolchain (no runtime effect) |
| Python lint output | `scripts/*.py` | Pure static analysis |

Architecture section references: N/A — no Rust crates, no subsystem involvement.

**Anchor justifications:**
- No subsystem anchor: CI toolchain is not assigned to a Subsystem in ARCH-INDEX.
- No `depends_on` dependencies: ruff is standalone; does not depend on any other AUDIT stories.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `ruff` is not available on the CI runner image | Install via `pip install ruff=={pinned}` in a prior CI step; pin the version to avoid surprise upgrades |
| EC-002 | A future `scripts/*.py` addition has F821 violations | `just check` and CI fail immediately; author must fix before merge |
| EC-003 | ruff's F821 check produces false positives for dynamically-computed names (e.g., `globals()[key]`) | Add per-line `# noqa: F821` with a comment explaining why the dynamic access is safe; do NOT disable ruff globally |
| EC-004 | The CI runner runs a different Python version than the developer environment | ruff is version-independent for F821; no Python version constraint beyond `>=3.8` |

## Token Budget Estimate

| Item | Lines | Tokens (est.) |
|------|-------|--------------|
| Story spec (this file) | ~120 | ~1,700 |
| Justfile (relevant sections) | ~30 | ~420 |
| .github/workflows/ci.yml (relevant job) | ~50 | ~700 |
| scripts/t13-preflight-audit.py (scan for F821 to confirm zero violations) | ~30 | ~420 |
| **Total estimate** | | **~3,240 tokens** |

Fits within a 100k-token agent context window (<4%). No split required.

## Tasks

- [ ] Check current `ruff` availability: `pip show ruff` or `ruff --version`; record the version.
- [ ] Determine whether any existing `scripts/*.py` file has F821 violations: `ruff check --select F821 scripts/`; fix any found.
- [ ] Add `lint-scripts` recipe to `Justfile` (AC-002).
- [ ] Add `ruff` installation and lint step to `.github/workflows/ci.yml` (AC-003).
- [ ] Document `ruff` version pin in `docs/dev-setup.md` and/or `scripts/requirements-dev.txt` (AC-001).
- [ ] Run `just check-fast` to confirm no Rust breakage from Justfile edit.

## Previous Story Intelligence

N/A — first story targeting the scripts/ Python lint gate. Prior context:
- Fix-burst at pass-24 fixed the specific NameError (`TD-VSDD-060` recurrence); that fix is
  already in HEAD `acf7ded0`.
- `pyflakes` has been run manually during fix-bursts throughout the cascade (mentioned in many
  D-NNN verification lines as "pyflakes 18 warnings (pre-existing, zero new)"); those 18 warnings
  are F401-class (f-string warnings), not F821. The F821 gate targets `NameError`-class bugs.

## Architecture Compliance Rules

- **TD-VSDD-091:** Cite tool names (`ruff`, `pyflakes`) and recipe names (`lint-scripts`), NOT file/line numbers.
- **reqwest TLS (ADR-050):** N/A — no Rust changes.
- **Justfile recipe ordering:** `lint-scripts` must be called from `check` or `check-ci`; do NOT create an isolated recipe that is only manually invoked.

## Library & Framework Requirements

| Library | Version | Notes |
|---------|---------|-------|
| `ruff` | `>=0.1.0` (pin exact version at implementation time) | F821/F401/F811 linting for `scripts/*.py` |
| Python | 3.x (workspace standard) | Used only by scripts runtime and ruff |

**Forbidden dependencies:** None applicable.

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `Justfile` | Modify | Add `lint-scripts` recipe; call from `check` recipe |
| `.github/workflows/ci.yml` | Modify | Add ruff install + `ruff check --select F821` step |
| `docs/dev-setup.md` | Modify | Document `ruff` install requirement |
| `scripts/requirements-dev.txt` | Create or modify | Pin ruff version |

No Rust files, no Cargo.toml changes.
