---
document_type: story
story_id: S-PERF-GATE-006
title: "Justfile RUSTFLAGS fingerprint alignment — add RUSTFLAGS=\"\" to check clippy step to eliminate ~157s test-target rebuild on every just check"
epic_id: EPIC-MAINTENANCE
version: "1.1"
status: draft
producer: story-writer
phase: 3
wave: maintenance
priority: P2
points: 1
tdd_mode: "n/a"
# tdd_mode rationale: pure config story — no production Rust code added or modified.
# No function bodies, no Red Gate tests. The only change is one RUSTFLAGS="" prefix on
# the cargo clippy line in the Justfile check recipe. Validated by running `just check`
# twice on a warm build and confirming the nextest build phase does not force a full
# test-binary recompile. Mutation testing (facade-mode quality gate) does not apply to
# Justfile recipes.
target_module: "n/a — build tooling only (Justfile)"
subsystems: []
depends_on: [S-PERF-GATE-005]
blocks: []
behavioral_contracts: [BC-5.39.001]
# BC status: BC-5.39.001 is the delivery-quality contract (3-CLEAN convergence protocol).
# This story has no product behavioral contracts — it is a build-tooling maintenance story.
# BC-5.39.001 is already ACTIVE. POL-14 will be a NO-OP at merge.
verification_properties: []
assumption_validations: []
risk_mitigations: []
red_gate_tests: 0
estimated_days: "0.1"
---

# S-PERF-GATE-006: Justfile RUSTFLAGS Fingerprint Alignment

One-line fix: add `RUSTFLAGS=""` to the `check` clippy step so clippy and nextest share the
same build-fingerprint cache, eliminating the ~157s cold-target rebuild on every warm `just
check` run after clippy

## Narrative

As a Prism developer, I want `cargo clippy` and `cargo nextest` in the `just check` recipe
to use the same RUSTFLAGS value (`""`), so that the test binary artifacts compiled by clippy
can be reused by nextest without a full rebuild, saving approximately 150 seconds of
unnecessary recompilation on every warm `just check` invocation.

## §Evidence

From profiling report `.factory/research/test-suite-perf-profile-2026-06-30.md`
(baseline: develop@8bc0404e, 16 logical CPUs, warm build):

| Stage | Command | Wall-clock (warm) |
|-------|---------|-------------------|
| fmt | `cargo fmt --check` | 1.49s |
| clippy | `cargo clippy --all-features -- -D warnings` | 43.85s |
| nextest build | compile test binaries (inside `cargo nextest run`) | **~157s** |
| nextest execution | 4976 tests, `--profile prepush` | 585.84s |
| doctest | `cargo test --workspace --all-features --doc` | ~8s |
| **TOTAL (just check)** | | **~798s (≈ 13.3 min)** |

The ~157s nextest build phase represents pure waste on a warm build: no code has changed
between clippy and nextest, yet all test binary targets are recompiled from scratch.

**Root cause:** `cargo clippy` in the `check` recipe uses the default RUSTFLAGS (whatever
the shell environment provides). `cargo nextest run` uses `RUSTFLAGS=""` explicitly. These
are different Cargo compiler fingerprints; artifacts compiled under one fingerprint are
treated as stale by the other, forcing a full test-binary rebuild every time.

**The fix intent already exists in the Justfile comment:**

```
# NOTE: RUSTFLAGS="" is set explicitly on both the nextest and doctest steps so they share
# the same fingerprint cache. Without alignment, a RUSTFLAGS drift (e.g. a shell export)
```

Nextest and doctest already carry `RUSTFLAGS=""`. Clippy was left unaligned. This story
closes the gap.

## Background

### Current `check` recipe (before fix)

```justfile
check:
    cargo fmt --check
    cargo clippy --all-features -- -D warnings                              # ← default RUSTFLAGS
    RUSTFLAGS="" PROPTEST_CASES=100 cargo nextest run --workspace --all-features --profile prepush
    RUSTFLAGS="" PROPTEST_CASES=100 cargo test --workspace --all-features --doc
    @scripts/check-crate-layout.sh
    @scripts/check-non-exhaustive.sh
```

Fingerprint sequence (warm build, no code changes):
1. clippy — compiles under fingerprint A (default/ambient RUSTFLAGS)
2. nextest — compiles under fingerprint B (RUSTFLAGS="") → **MISMATCH → ~157s full test-binary rebuild**
3. doctest — compiles under fingerprint B → reuses nextest cache → ~8s

### Fixed `check` recipe (after fix)

```justfile
check:
    cargo fmt --check
    RUSTFLAGS="" cargo clippy --all-features -- -D warnings                 # ← aligned
    RUSTFLAGS="" PROPTEST_CASES=100 cargo nextest run --workspace --all-features --profile prepush
    RUSTFLAGS="" PROPTEST_CASES=100 cargo test --workspace --all-features --doc
    @scripts/check-crate-layout.sh
    @scripts/check-non-exhaustive.sh
```

Fingerprint sequence (warm build, no code changes):
1. clippy — compiles under fingerprint B (RUSTFLAGS="")
2. nextest — compiles under fingerprint B → **MATCH → ~5-10s incremental only**
3. doctest — compiles under fingerprint B → reuses nextest cache → ~8s

**Estimated savings:** ~150s per warm `just check` invocation. For a developer running
`just check` 10 times per day, this recovers ~25 minutes of wall-clock time daily.

### Why `RUSTFLAGS=""` is the correct canonical value

- `RUSTFLAGS=""` explicitly overrides any ambient RUSTFLAGS to the empty string, making
  the build deterministic regardless of the developer's shell environment.
- `RUSTFLAGS=""` is already the established convention for nextest and doctest in `check`
  (per the existing Justfile comment).
- CI's `clippy` job sets no RUSTFLAGS (it runs `cargo clippy --workspace --all-features
  -- -D warnings` with no env override), so `RUSTFLAGS=""` matches CI's effective clippy
  fingerprint. This local-recipe change is CI-neutral because CI does not invoke
  `just check` or `just check-fast` at all — it runs individual cargo steps directly.
  CI's `test` job nextest and doctest steps DO set `RUSTFLAGS=-C link-arg=-fuse-ld=mold`
  on Linux (to use the mold linker), but this is unrelated to the local Justfile change
  and does not affect the fingerprint alignment this story addresses.
- `RUSTFLAGS=""` does NOT suppress `-D warnings`. The `-D warnings` lint flag is passed
  via cargo's `--` argument separator (`-- -D warnings`), NOT via RUSTFLAGS. Adding
  `RUSTFLAGS=""` before `cargo clippy` has zero effect on lint coverage.

### `check-ci` is not in scope

The `check-ci` recipe uses default RUSTFLAGS on ALL steps (clippy, nextest, doctest) — no
`RUSTFLAGS=""` anywhere. Within `check-ci`, the fingerprint is consistent (all default),
so there is no rebuild issue internal to `check-ci`. This story does NOT change `check-ci`.
Cross-recipe fingerprint drift (running `just check` then `just check-ci`) is a secondary
concern documented in EC-004 below.

### `check-fast` is in scope

The `check-fast` recipe runs `cargo clippy --all-features -- -D warnings` with default
RUSTFLAGS (no `RUSTFLAGS=""` prefix). A developer who runs `just check` (which now sets
`RUSTFLAGS=""` on clippy) and then runs `just check-fast` encounters a fingerprint change:
`check-fast` uses ambient RUSTFLAGS while `check` uses `RUSTFLAGS=""` — two different
fingerprints — causing a full clippy recompile even though no code changed. This story
therefore also prepends `RUSTFLAGS=""` to the `check-fast` clippy line to eliminate
the cross-recipe clippy divergence.

## Scope

One file modified, two lines changed:

| File | Change | Rationale |
|------|--------|-----------|
| `Justfile` | Prepend `RUSTFLAGS="" ` to `cargo clippy --all-features -- -D warnings` in the `check` recipe | Aligns clippy fingerprint with nextest/doctest; eliminates the ~157s rebuild |
| `Justfile` | Prepend `RUSTFLAGS="" ` to `cargo clippy --all-features -- -D warnings` in the `check-fast` recipe | Aligns check-fast clippy fingerprint with check; eliminates cross-recipe clippy divergence |

**NOT in scope:**

- `Justfile` `check-ci` recipe — no change needed (internally consistent; see EC-004)
- `.config/nextest.toml` — no change needed
- `.cargo/config.toml` — no change needed
- `.github/workflows/ci.yml` — no change needed
- Any production Rust code
- Any `.factory/` file — state-manager handles STORY-INDEX registration

## Acceptance Criteria

### AC-001 — `check` recipe: RUSTFLAGS="" prefix present on the clippy line

```
grep -c 'RUSTFLAGS="" cargo clippy --all-features -- -D warnings' Justfile
```

Expected output: `2` (one match in the `check` recipe, one in the `check-fast` recipe).

Source-verification: before the change, neither recipe carries the RUSTFLAGS prefix —
this grep returns 0. After the change, both `check` and `check-fast` carry the prefix and
it returns 2. The grep string is anchored to the full command including the
`-- -D warnings` suffix, preventing a false match against any future clippy variant that
lacks the warnings flag.

Traces to: BC-5.39.001 postcondition — delivery quality; the config change is present and
correctly formed. Without this line, the fingerprint mismatch persists.

### AC-002 — nextest RUSTFLAGS="" line unchanged (no regression to existing fingerprint)

```
grep -c 'RUSTFLAGS="" PROPTEST_CASES=100 cargo nextest run' Justfile
```

Expected output: `1`.

Source-verification: this line is present before and after the change. A count of 0
indicates accidental deletion. Anchoring to the full prefix prevents false matches.

Traces to: BC-5.39.001 postcondition — delivery quality; existing nextest fingerprint
alignment must not be disturbed.

### AC-003 — doctest RUSTFLAGS="" line unchanged (no regression to existing fingerprint)

```
grep -c 'RUSTFLAGS="" PROPTEST_CASES=100 cargo test.*--doc' Justfile
```

Expected output: `1`.

Source-verification: this line is present before and after the change. A count of 0
indicates accidental deletion.

Traces to: BC-5.39.001 postcondition — delivery quality; doctest fingerprint alignment
must not be disturbed.

### AC-004 — `-D warnings` lint flag preserved in the check clippy command

```
grep -c 'RUSTFLAGS="" cargo clippy.*-- -D warnings' Justfile
```

Expected output: `2` (one match per recipe: `check` and `check-fast`).

This AC is complementary to AC-001. It explicitly asserts that the `-D warnings` argument
is present in both the `check` and `check-fast` clippy commands (not accidentally dropped
when the RUSTFLAGS prefix was added). The `-D warnings` flag is in the cargo argument
vector after `--`; it is NOT in RUSTFLAGS. A count of 0 means the lint flag was
accidentally removed; a count of 1 means only one recipe was updated.

Traces to: BC-5.39.001 postcondition — delivery quality; lint coverage must be preserved.
No production-grade project ships with weakened clippy enforcement.

### AC-005 — warm `just check` does not trigger a full test-binary rebuild between clippy and nextest

Run `just check` once to warm the RUSTFLAGS="" fingerprint cache for all steps, then run it
again immediately with no code changes:

```
just check          # first run — warms the cache
just check          # second run — observe nextest build phase
```

On the second run, the nextest build phase must complete in < 30s (target: ~5-10s
incremental). No `Compiling` lines should appear during the nextest build phase that
correspond to test binary targets (only incremental symbol re-linking at most).

Before the fix, the second run shows ~157s of Compiling lines. After the fix, the
fingerprints align and the second run reuses the clippy cache.

This AC is measurement-based. Record the nextest build phase wall-clock for the PR
description changelog. If the second run still shows 100+ Compiling lines, the RUSTFLAGS
prefix may have a typo or trailing whitespace — re-verify AC-001.

Traces to: BC-5.39.001 postcondition — delivery quality; the fingerprint fix must produce
the measurable build-time improvement that motivates this story.

### AC-006 — `just check` exits 0 with all changes applied

```
just check
echo "Exit: $?"
```

Expected output: `Exit: 0`.

A non-zero exit indicates either: a Justfile syntax error (check that `RUSTFLAGS="" ` ends
with a space before `cargo`), or a clippy lint regression (unlikely — `RUSTFLAGS=""` has
no effect on `-D warnings` lint coverage). If exit is non-zero, run
`RUSTFLAGS="" cargo clippy --all-features -- -D warnings` directly to isolate the failure.

Note: this AC requires AC-001 through AC-005 to be applied and verified first. Run it once
at the end of the implementer's work, before committing.

Traces to: BC-5.39.001 postcondition — delivery quality; the config change must not break
the pre-push gate.

### AC-007 — `check-fast` recipe: RUSTFLAGS="" prefix present on the clippy line (cross-recipe alignment)

```
grep -A5 '^check-fast:' Justfile | grep -c 'RUSTFLAGS="" cargo clippy'
```

Expected output: `1`.

Source-verification: this grep uses context anchoring (`-A5`: 5 lines after the
`check-fast:` recipe header) to isolate the `check-fast` recipe and verify the
`RUSTFLAGS=""` prefix is present on its clippy line specifically. Before the change, the
output is `0` (the `check-fast` clippy line lacks the prefix). After the change, it is `1`.

This AC is distinct from AC-001 (which counts all global recipe matches) — it positively
confirms the `check-fast` recipe received the alignment, not just that two global matches
exist.

Traces to: BC-5.39.001 postcondition — delivery quality; the `check-fast` clippy
fingerprint must be aligned with `check` to prevent cross-recipe clippy recompilation
after running `just check`.

## Red Gate

Zero Red Gate tests. This story makes no changes to production Rust source code. The only
file modified is `Justfile`. There is no `todo!()` stub to introduce and no failing test to
write first. Validation is via `just check` exit code (AC-006), grep assertions (AC-001
through AC-004), and warm-cache timing observation (AC-005).

This is consistent with S-PERF-GATE-001 through S-PERF-GATE-004 (all zero Red Gate tests
for config-only portions) and S-PERF-GATE-003 (Justfile + nextest.toml, zero Red Gate).

## Behavioral Contracts

| BC | Title | Role in this story |
|----|-------|--------------------|
| BC-5.39.001 | 3-CLEAN convergence protocol | Delivery-quality gate — this story's own PR must pass 3-CLEAN before merge |

This story has no product behavioral contracts. The Justfile `check` recipe change has no
observable effect on test SEMANTICS — only on build fingerprinting POLICY. A test that
passes under `RUSTFLAGS=""` passes under any RUSTFLAGS value (and vice versa for a
correctly-written test). The change affects build time, not correctness.

## Tasks

1. **Read** `Justfile` lines 14-29 (the `check` recipe and its preceding comment block) to
   confirm the current `cargo clippy` line has no `RUSTFLAGS=""` prefix and that the
   existing nextest/doctest lines already carry `RUSTFLAGS=""`.

2. **Edit** `Justfile`: in BOTH the `check` recipe AND the `check-fast` recipe, change
   each occurrence of:
   ```
   cargo clippy --all-features -- -D warnings
   ```
   to:
   ```
   RUSTFLAGS="" cargo clippy --all-features -- -D warnings
   ```
   The change is a single word insertion (`RUSTFLAGS="" `) before `cargo`. Two lines are
   modified (one per recipe). Do NOT modify any other line in the file.

   Optionally update the preceding comment block in `check` to note that ALL three
   non-fmt steps now carry `RUSTFLAGS=""`, and that `check-fast` is now aligned as well.

3. **Verify** AC-001 through AC-004 and AC-007 grep commands each return their expected
   values. Run each grep before running `just check`.

4. **Run** `just check` once to warm the RUSTFLAGS="" cache for all steps (nextest build
   phase will be slow on this first run as it recompiles under the new fingerprint).

5. **Run** `just check` a second time immediately with no code changes. Observe the nextest
   build phase: confirm no (or very few) `Compiling` lines, and that the build phase
   completes in < 30s. Record the nextest build phase wall-clock for AC-005 confirmation.

6. **Run** `just check` a final time and verify AC-006 (`Exit: 0`).

7. **Confirm** the ONLY modified file is `Justfile` (no production Rust code changes, no
   `.config/nextest.toml` changes, no story-index changes — state-manager handles index
   registration).

## Token Budget Estimate

| Context component | Estimated tokens |
|-------------------|-----------------|
| This story spec (v1.1, ~330 lines) | ~3,900 |
| `Justfile` (full file, ~220 lines — read + modify) | ~2,000 |
| AC verification grep outputs (6 commands) | ~300 |
| `just check` output (two warm runs, abbreviated) | ~2,000 |
| **Total** | **~7,600** |

Well within the implementer agent's context window. Simpler than S-PERF-GATE-003 (one word
insertion in one file; no nextest.toml surgery, no shell script changes).

## Previous Story Intelligence

### From S-PERF-GATE-004 (PR #209, develop@e3148007)

- Config-only stories (tdd_mode: n/a) use `grep -c` with fully-anchored strings for
  self-verification. The full command string anchors prevent false positives against
  comment lines or adjacent recipe commands.
- `red_gate_tests: 0` for config-only stories — applies here.
- Single-commit-per-burst rule (TD-VSDD-053) applies — deliver in ONE commit.

### From S-PERF-GATE-003 (PR #207, develop@1f491590)

- grep-count-drift lesson: every AC grep should be source-verified against the actual
  file BEFORE the story is promoted to `ready`. Verify that
  `RUSTFLAGS="" cargo clippy` does NOT already appear in Justfile (count 0 before fix,
  1 after) to confirm the pre-condition.

### Justfile env-var prefix syntax

In `just` recipes (which run via `/bin/sh`), the POSIX form `VARNAME=value command args`
sets `VARNAME` in the environment of `command` for that single invocation. It does NOT
export the variable to the shell or to subsequent recipe lines. Each recipe line is an
independent shell invocation. Therefore `RUSTFLAGS="" cargo clippy` sets RUSTFLAGS="" only
for the duration of that one clippy command — it does not affect the nextest or doctest
lines that follow (which already carry their own `RUSTFLAGS=""` prefix). This is the correct
and safe behavior.

### Cargo fingerprint mechanics

Cargo uses RUSTFLAGS as part of the unit fingerprint for compiled artifacts. Two compilations
with different RUSTFLAGS values produce DIFFERENT fingerprints and CANNOT share artifacts —
even when source code, compiler version, and feature flags are identical. The `RUSTFLAGS=""`
form explicitly unsets any ambient RUSTFLAGS, making the build fingerprint environment-
independent and deterministic. This is not semantically equivalent to "whatever the shell
environment's RUSTFLAGS is" — it is a stable, portable, and reproducible value.

## Architecture Compliance Rules

Extracted from architecture sections and ADRs relevant to this story:

1. **ADR-022 (Arc-DI wiring)**: Not applicable — no production Rust code modified.

2. **Single-workspace MSRV (rust-toolchain.toml)**: Not applicable — no Rust code.

3. **TD-VSDD-053 (single-commit-per-burst)**: The implementer must deliver this story's
   changes in a SINGLE commit. No multi-step "Stage 1 / Stage 2" commits.

4. **No `--no-verify` hook bypass**: The `just check` verification (AC-006) requires git
   hooks to pass normally. Do not bypass hooks to deliver this story.

5. **RUSTFLAGS="" placement — before `cargo`, not inside `--`**: The correct form is
   `RUSTFLAGS="" cargo clippy ... -- -D warnings`. An incorrect form would be
   `cargo clippy ... -- -D warnings RUSTFLAGS=""` (invalid — this passes RUSTFLAGS="" as
   a compiler flag string) or `cargo clippy RUSTFLAGS="" ...` (RUSTFLAGS="" treated as
   a cargo argument, not an env var). Only the env-prefix form is correct.

6. **`-D warnings` is a rustc argument, NOT a RUSTFLAGS value**: The `-D warnings` flag
   is passed via `-- -D warnings` in the cargo argument vector. It travels to rustc via
   cargo's `--cfg` / lint mechanism, NOT via RUSTFLAGS. Do NOT add lint flags to RUSTFLAGS.
   Do NOT remove `-- -D warnings` from the command.

7. **No AI attribution in commits** per project git conventions (CLAUDE.md).

8. **`just check` must exit 0 before the PR is opened.**

9. **`.factory/` not modified by this story**: state-manager handles STORY-INDEX
   registration.

## Library and Framework Requirements

This story does not introduce or change any library dependency. The only toolchain
requirement is the `just` task runner and `cargo` already present in the developer
environment. No version pins change. `RUSTFLAGS=""` syntax is standard POSIX shell
env-var assignment and is fully supported by `just` (which uses sh for recipe execution).

## File Structure Requirements

| File | Change type | Details |
|------|-------------|---------|
| `Justfile` | Modify | Prepend `RUSTFLAGS="" ` to the `cargo clippy` line in the `check` recipe and the `check-fast` recipe |

**Files explicitly excluded from this story:**

- `Justfile` `check-ci` recipe — no change (internally consistent; see EC-004)
- `.config/nextest.toml` — no change needed
- `.cargo/config.toml` — no change needed
- `.github/workflows/ci.yml` — no change needed
- Any `crates/**/*.rs` files — no production code changes
- Any `.factory/` file — state-manager handles STORY-INDEX registration post-delivery

## Scheduling Note

**S-PERF-GATE-005 ALREADY MERGED (PR #210, develop@8bc0404e). Hard dependency satisfied.**

The implementer must branch `feature/S-PERF-GATE-006` off develop HEAD 8bc0404e (or later).
No merge conflict risk — S-PERF-GATE-005 only modified `crates/prism-dtu-*/src/clone.rs`
files and `crates/prism-dtu-common/src/`. `Justfile` was not touched by S-PERF-GATE-005.

S-PERF-GATE-006 and S-PERF-GATE-007 are INDEPENDENT of each other (different files):
- S-PERF-GATE-006 → `Justfile` only
- S-PERF-GATE-007 → `.config/nextest.toml` only

They may be developed in parallel on separate worktrees or sequentially; no ordering
constraint between them.

```
develop (after S-PERF-GATE-005 merge — 8bc0404e)
  └── feature/S-PERF-GATE-006   ← branch from here
        └── Edit Justfile (RUSTFLAGS="" on clippy line in check recipe)
```

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Developer has `RUSTFLAGS` set in their shell environment (e.g., `export RUSTFLAGS="-C instrument-coverage"`) | `RUSTFLAGS="" cargo clippy` explicitly overrides the ambient RUSTFLAGS with the empty string. The developer's shell RUSTFLAGS is ignored for this one clippy invocation — matching the existing behavior of the nextest and doctest steps. This is the intended behavior per the Justfile comment. |
| EC-002 | First `just check` run after the fix shows a full rebuild (~157s nextest build phase) | This is expected on the FIRST run after the change because the cache needs to be rebuilt under the new (aligned) fingerprint. All subsequent runs with no code changes will be fast (< 30s build phase). This is documented in the Tasks (Task 4 warms the cache). |
| EC-003 | `RUSTFLAGS=""` breaks a clippy lint that previously relied on an ambient RUSTFLAGS value | No currently-passing clippy lint in the workspace depends on RUSTFLAGS. `-D warnings` is in the cargo argument vector, not RUSTFLAGS. AC-006 (`just check` exit 0) is the safety net. If AC-006 fails, inspect `RUSTFLAGS="" cargo clippy --all-features -- -D warnings 2>&1` output directly. |
| EC-004 | Cross-recipe fingerprint drift: developer runs `just check` (RUSTFLAGS="") then `just check-ci` (default RUSTFLAGS on nextest) | `check-ci` uses default RUSTFLAGS for all steps (internally consistent). Running `just check` before `just check-ci` will cause `check-ci`'s nextest to rebuild (~157s) because its fingerprint differs from `check`'s RUSTFLAGS="" artifacts. This is NOT fixed by this story. A follow-up may align `check-ci` by adding `RUSTFLAGS=""` to ALL three non-fmt steps (clippy, nextest, doctest) simultaneously — changing only clippy in `check-ci` would CREATE a new mismatch there. |
| EC-005 | New step added to `check` recipe after this fix that compiles Rust code without `RUSTFLAGS=""` | The new step would create a fingerprint mismatch if it compiles under a different RUSTFLAGS. This is a process discipline issue for future stories. The Justfile comment explains the convention; implementers of future `check` steps must add `RUSTFLAGS=""` to maintain consistency. |
| EC-006 | RUSTFLAGS environment variable set to exactly `""` vs unset | `RUSTFLAGS=""` sets the environment variable to the empty string. This is distinct from unsetting the variable (`unset RUSTFLAGS`). In practice, cargo treats both as "no additional RUSTFLAGS" and produces the same fingerprint. The determinism benefit of `RUSTFLAGS=""` (vs unset) is that it is immune to a parent shell accidentally exporting an empty RUSTFLAGS, which would produce different behavior depending on whether the shell exported it or not. |

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-06-30 | story-writer | Initial draft (T-PERF-PROFILE initiative, D-1434) |
| 1.1 | 2026-06-30 | story-writer | MED-2: corrected false CI-neutral claim — CI clippy job sets no RUSTFLAGS (verified in ci.yml), CI does not invoke just check or just check-fast, but CI test nextest/doctest steps DO set RUSTFLAGS=-C link-arg=-fuse-ld=mold on Linux (mold linker); MED-1: expanded scope to include check-fast clippy alignment (cross-recipe divergence), added AC-007 (check-fast recipe grep), updated AC-001 and AC-004 expected counts from 1 to 2, updated Scope table, Task 2, File Structure Requirements |
