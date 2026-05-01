---
story_id: W3-FIX-LEFTHOOK-001
title: "Pre-push lefthook gate tuning — proptest case reduction, audit/deny CI-only, semver-checks pre-tag"
wave: 3
level: "L4"
target_module: workspace-tooling
subsystems: []
priority: P1
depends_on: []
blocks: []
estimated_days: 0.5
points: 3
status: merged
document_type: story
version: "1.0"
producer: story-writer
timestamp: "2026-04-30T18:00:00Z"
input-hash: "[live-state]"
inputs:
  - lefthook.yml
  - Justfile
  - .github/workflows/ci.yml
traces_to: []
cycle: "v1.0.0-greenfield"
epic_id: "E-3.5"
phase: 3
behavioral_contracts: []
# BC status: pending PO authorship — no formal BC governs dev toolchain speed.
# This story is tooling-only; no production behavior is affected.
verification_properties: []
anchor_bcs: []
anchor_capabilities: [CAP-DEV-SPEED]
anchor_subsystem: ["SS-00"]
tdd_mode: lenient
assumption_validations: []
risk_mitigations: []
---

# W3-FIX-LEFTHOOK-001: Pre-push lefthook gate tuning

## Narrative

As a Prism developer, I want the `pre-push` lefthook gate (`just check`) to complete
in under 10 minutes on a typical workstation, so that rapid Wave 3 story merges are
not blocked by a 30-45 minute gate running supply-chain checks and full proptest
suites on every push.

## Problem Statement

The current `just check` target runs the following commands serially in the
`pre-push` lefthook hook:

```
cargo fmt --check
cargo clippy --all-features -- -D warnings
cargo test --workspace --all-features          # 1000-case proptests
cargo deny check                               # supply-chain: license scan
cargo audit                                    # supply-chain: RustSec advisory
cargo semver-checks --workspace --baseline-rev origin/develop   # compiles wasmtime/cranelift baselines
@scripts/check-crate-layout.sh
```

Measured wall-clock time: 30-45 min per push on developer workstations.

**Root causes (by contribution):**
1. `cargo test --workspace --all-features` runs 1000 proptest cases per harness — the
   majority of elapsed time.
2. `cargo semver-checks` compiles wasmtime and cranelift baseline crates from scratch
   on every invocation; these are multi-MLOC C/C++ transitive deps.
3. `cargo audit` and `cargo deny check` are supply-chain checks that read Cargo.lock /
   Cargo metadata only (no compilation) but are slow in aggregate and irrelevant to
   whether the local diff is safe.

**Impact:** Wave 3 Batch 8/9/10 velocity blocked. Developer round-trip for a review
cycle (push → CI trigger) serially stalls at the local gate.

## Proposed Fix (4 sub-fixes)

### Sub-fix 1: Reduce proptest cases on pre-push

In `Justfile`, change the test invocation in `check` to set `PROPTEST_CASES=100`:

```makefile
# Local pre-push runs 100 proptest cases for fast feedback.
# CI runs the default 1000 cases for full coverage (see check-ci target).
check:
    cargo fmt --check
    cargo clippy --all-features -- -D warnings
    PROPTEST_CASES=100 cargo test --workspace --all-features
    @scripts/check-crate-layout.sh
```

### Sub-fix 2: Move cargo audit + cargo deny to CI-only

Remove `cargo audit` and `cargo deny check` from `just check`. Add dedicated targets:

```makefile
# Supply-chain checks — do not depend on local diff content.
# Run manually or via check-ci / CI pipeline.
audit:
    cargo audit

deny:
    cargo deny check

# Full gate identical to CI — includes audit, deny, semver-checks, full proptest cases.
check-ci:
    cargo fmt --check
    cargo clippy --all-features -- -D warnings
    cargo test --workspace --all-features
    cargo deny check
    cargo audit
    cargo semver-checks --workspace --baseline-rev origin/develop
    @scripts/check-crate-layout.sh
```

### Sub-fix 3: Move cargo semver-checks to pre-tag / manual

Remove `cargo semver-checks` from `just check`. Document it as a pre-release step.
In `lefthook.yml`, add an optional `pre-tag` hook block:

```yaml
pre-tag:
  commands:
    semver:
      run: just semver-checks
```

Add a `semver-checks` target to `Justfile`:

```makefile
# Semver compatibility — enforced at tag creation, not at every PR push.
# Run manually before tagging a release, or triggered by the pre-tag hook.
semver-checks:
    cargo semver-checks --workspace --baseline-rev origin/develop
```

### Sub-fix 4: Document CARGO_TARGET_DIR shared option

In `docs/CRATE-LAYOUT.md` (or a new `docs/dev-setup.md`), document the option to
share the Cargo build cache across worktrees:

```bash
# In ~/.zshrc / ~/.bashrc — optional, only if you have >=50GB free on target volume.
export CARGO_TARGET_DIR=$HOME/.cargo-target-shared/prism
```

**Caveats to document:**
- Cargo uses a file lock (`target/.cargo-lock`) per invocation; concurrent `cargo`
  processes from different worktrees on the same shared dir will serialize (not corrupt).
- Cold-build savings are large (~10-15 min first build); subsequent incremental builds
  benefit less.
- Not recommended if your home volume is an HDD or <50GB free.

## Acceptance Criteria

### AC-001: Pre-push completes under 10 minutes

`just check` on local (pre-push) completes in under 10 minutes on a typical
development workstation (Apple Silicon M-series or equivalent x86-64 with warm
Cargo build cache). The implementer MUST record `time just check` before and after
the change in the PR description.

### AC-002: cargo audit and cargo deny removed from just check

`cargo audit` and `cargo deny check` are absent from the `check` recipe in
`Justfile`. They are preserved as standalone `just audit` and `just deny` targets
for ad-hoc local use.

### AC-003: cargo semver-checks removed from just check

`cargo semver-checks` is absent from the `check` recipe in `Justfile`. A standalone
`just semver-checks` target is present. `lefthook.yml` includes a `pre-tag` hook
block that invokes `just semver-checks` (or is documented as such if lefthook does
not natively support tag hooks on the current version).

### AC-004: PROPTEST_CASES=100 set on pre-push; CI runs 1000

The `check` recipe in `Justfile` sets `PROPTEST_CASES=100` for `cargo test`. The
`check-ci` recipe does NOT set `PROPTEST_CASES` (allowing proptest's default of
1000). The CI workflow (`ci.yml`) continues to invoke the unmodified `cargo test
--workspace --all-features` step (i.e., it does NOT call `just check` — it invokes
individual cargo steps directly; no change needed to CI).

### AC-005: CARGO_TARGET_DIR sharing documented

The option to set `CARGO_TARGET_DIR=$HOME/.cargo-target-shared/prism` is documented
in at least one Markdown file under `docs/` (existing or new). The documentation
includes the caveat about lock-file contention and disk-space requirements.

### AC-006: CI continues full-strength validation

The CI workflow (`ci.yml`) is unchanged (audit, deny, semver-checks, and
full-proptest jobs remain present and unmodified). A new `check-ci` target in
`Justfile` provides an equivalent full-gate command for developers who want
CI-identical local validation.

### AC-007: Existing workspace tests pass under PROPTEST_CASES=100

`PROPTEST_CASES=100 cargo test --workspace --all-features` exits 0. No test is
gated on the specific value of `PROPTEST_CASES` in a way that causes a spurious
failure at 100 cases (i.e., no `assert_eq!(config.cases, 1000)` style assertions).

## Testing Strategy

This is a tooling-only story; there are no runtime behavior changes to test via unit
or integration tests. Validation is by direct measurement and inspection:

1. **Before measurement:** on the implementer's workstation, run `time just check`
   on the current `develop` branch and record the wall-clock time.
2. **Apply the four sub-fixes** to `Justfile`, `lefthook.yml`, and `docs/`.
3. **After measurement:** run `time just check` again on the same branch with the
   same warm cache and record the wall-clock time.
4. **Record both times in the PR description.** The PR must show a reduction of
   at least 50% in wall-clock time (target: <=10 min from >=20 min baseline).
5. **Inspect CI:** confirm that the `ci.yml` jobs for `audit`, `deny`, `semver`,
   and `test` are unmodified and still present in the workflow file.
6. **Regression check:** run `PROPTEST_CASES=100 cargo test --workspace --all-features`
   and confirm exit 0.

## Risk Mitigation

**Risk: 100-case proptest sweep misses a regression that 1000 cases catches.**

Mitigation: CI runs the full 1000-case sweep on every push. The local 100-case sweep
is a heuristic fast-feedback signal, not a correctness guarantee. False negative
tolerance is acceptable at the local gate: the invariant is that CI is the canonical
correctness signal. Developer workflow is: push to branch → CI catches full
regression → developer fixes locally. The 100-case local sweep catches the obvious
regressions that would fail immediately regardless of case count.

**Risk: semver-checks not running on routine PRs means an accidental API break
ships to develop.**

Mitigation: semver-checks remains in CI (`ci.yml` unchanged). The CI `semver-checks`
job runs on every push/PR to develop. The pre-tag hook adds a second enforcement
point at release time. The only gap is that semver-checks is no longer run on the
local developer machine — but it was always CI-enforced; the local run was redundant.

**Risk: supply-chain advisory slips through because audit/deny is CI-only.**

Mitigation: `cargo audit` and `cargo deny check` run on every CI push (both are fast
metadata-only steps in CI). Local developers can run `just audit` or `just deny` at
any time. Supply-chain checks do not depend on local diff content — a new advisory
in the RustSec DB is just as discoverable via CI as via local pre-push.

## Architecture Mapping

| Component | Module | Pure/Effectful | Notes |
|-----------|--------|---------------|-------|
| Justfile | workspace-tooling | Effectful (shell) | Add targets: audit, deny, semver-checks, check-ci |
| lefthook.yml | workspace-tooling | Effectful (shell) | Add pre-tag hook; pre-push unchanged except just check now runs faster |
| docs/ | workspace-tooling | Pure (documentation) | Add CARGO_TARGET_DIR guidance |

No production Rust crates are modified by this story.

## File Structure Requirements

| File | Action | Change |
|------|--------|--------|
| `Justfile` | Modify | Add `PROPTEST_CASES=100` to `check`; remove audit/deny/semver-checks; add `audit`, `deny`, `semver-checks`, `check-ci` targets |
| `lefthook.yml` | Modify | Add `pre-tag:` block with semver-checks command |
| `docs/CRATE-LAYOUT.md` OR `docs/dev-setup.md` | Modify or Create | Add CARGO_TARGET_DIR sharing section |
| `.github/workflows/ci.yml` | No change | CI unchanged — full-strength gates remain |

## Purity Classification

| Module | Classification | Justification |
|--------|---------------|---------------|
| Justfile | effectful-shell | Executes shell commands; invokes cargo subprocesses |
| lefthook.yml | effectful-shell | Triggers shell hook scripts on git events |
| docs/ (CARGO_TARGET_DIR section) | pure-core | Static documentation; no side effects |

## Token Budget Estimate

| Item | Estimated tokens |
|------|-----------------|
| This story file | ~2 000 |
| Justfile (current + new targets) | ~500 |
| lefthook.yml | ~200 |
| ci.yml (read-only reference) | ~1 500 |
| docs/ target file | ~500 |
| Implementation scratch | ~500 |
| **Total** | **~5 200** |

Well within a single agent context window. No splitting required.

## Tasks

- [ ] Read current `Justfile` and identify exact lines to modify
- [ ] Modify `check` recipe: add `PROPTEST_CASES=100`, remove `cargo deny check`, `cargo audit`, `cargo semver-checks`
- [ ] Add `audit` recipe (`cargo audit`)
- [ ] Add `deny` recipe (`cargo deny check`)
- [ ] Add `semver-checks` recipe (`cargo semver-checks --workspace --baseline-rev origin/develop`)
- [ ] Add `check-ci` recipe (full gate: fmt + clippy + test + deny + audit + semver-checks + layout)
- [ ] Add doc comment to `check` recipe explaining the PROPTEST_CASES and CI-vs-local split
- [ ] Modify `lefthook.yml`: add `pre-tag:` block invoking `just semver-checks`
- [ ] Add CARGO_TARGET_DIR documentation to `docs/CRATE-LAYOUT.md` or new `docs/dev-setup.md`
- [ ] Run `time just check` before and after (record both in PR)
- [ ] Run `PROPTEST_CASES=100 cargo test --workspace --all-features` to confirm exit 0
- [ ] Verify `ci.yml` is unmodified (no accidental edits)

## Previous Story Intelligence

W3-FIX-WIN-001 (cross-platform port-release test) is the only prior Wave 3 fix story.
It is unrelated to tooling; no lessons carry over. This is the first tooling-speed
story in Wave 3.

## Architecture Compliance Rules

- No production crate source files (`crates/**/src/**`) may be modified by this story.
- No changes to `Cargo.toml` or `Cargo.lock` are permitted.
- `ci.yml` must not be modified — CI must retain full-strength gates.
- The `check-ci` target must invoke all seven steps in the original `check` order:
  `fmt → clippy → test → deny → audit → semver-checks → check-layout`.

## Library & Framework Requirements

| Tool | Version constraint | Notes |
|------|--------------------|-------|
| just | >=1.14 | Workspace Justfile in use; no version pin change needed |
| lefthook | >=1.6 | `pre-tag` hook support confirmed in lefthook >= 1.6; verify with `lefthook --version` |
| cargo-semver-checks | current installed | No version change; just moving invocation to `just semver-checks` |
| cargo-audit | current installed | No version change |
| cargo-deny | current installed | No version change |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `lefthook` version < 1.6 does not support `pre-tag` hook | Document fallback: run `just semver-checks` manually before `git tag`; lefthook.yml comment explains the version requirement |
| EC-002 | Developer has `PROPTEST_CASES` already set in their shell environment | `just check` uses the shell env value; add a comment in the Justfile that `PROPTEST_CASES=100` in the recipe overrides any environment value for the `cargo test` invocation |
| EC-003 | `CARGO_TARGET_DIR` shared dir does not exist on first use | Cargo creates the directory automatically; document that no manual `mkdir` is needed |
| EC-004 | Two concurrent `cargo` invocations in different worktrees on same shared target dir | Cargo serializes via `.cargo-lock`; no corruption; document expected behavior (one build waits for the other) |
| EC-005 | CI picks up the new `just check` instead of direct cargo invocations | CI invokes individual cargo steps directly (not `just check`); verify this is still the case after the change so that CI does not accidentally inherit PROPTEST_CASES=100 |
