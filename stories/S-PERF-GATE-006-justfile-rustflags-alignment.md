---
document_type: story
story_id: S-PERF-GATE-006
title: "Justfile RUSTFLAGS fingerprint alignment — align check, check-fast, and iter clippy/nextest fingerprints with RUSTFLAGS=\"\" so all dev-loop recipes share the test-artifact cache"
epic_id: EPIC-MAINTENANCE
version: "2.0"
status: draft
producer: story-writer
phase: 3
wave: maintenance
priority: P2
points: 1
tdd_mode: "n/a"
# tdd_mode rationale: pure config story — no production Rust code added or modified.
# No function bodies, no Red Gate tests. The only changes are RUSTFLAGS="" prefixes on
# cargo clippy lines in the Justfile check and check-fast recipes, and on the cargo nextest
# run -p line in the iter recipe, plus comment-block updates on all three recipes (rewrite
# on check, addition on check-fast, addition on iter). Validated by running just check-fast
# followed immediately by just check and confirming the nextest build phase does not force a
# full test-binary recompile. Mutation testing (facade-mode quality gate) does not apply to
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

Three `RUSTFLAGS=""` insertions plus comment-block updates on three recipes: prepend `RUSTFLAGS=""` to the
`cargo clippy` lines in both the `check` and `check-fast` recipes and to the
`cargo nextest run -p` line in the `iter` recipe (the project's primary TDD inner loop per CLAUDE.md),
plus comment-block rewrites (rewrite on `check`, addition on `check-fast`, addition on `iter`).
With all three dev-loop recipes aligned to `RUSTFLAGS=""`, nextest's test-artifact cache
stays warm across all dev-loop transitions (iter → check, check-fast → check, iter → check-fast),
eliminating the measured ~157s rebuild and the ~44s residual clippy re-check. Three separable effects:
(1) aligning `check`'s clippy to `RUSTFLAGS=""` eliminates the ~157s nextest rebuild
(Evidence table nextest build cost) by resolving the clippy(ambient)↔nextest(RUSTFLAGS="")
fingerprint mismatch internal to the `check` recipe;
(2) aligning `check-fast`'s clippy additionally eliminates the ~44s residual clippy re-check
(Evidence table clippy cost: 43.85s) that would otherwise remain in the check-fast → check
dev loop — `cargo clippy` without `--all-targets` does not compile test binaries and cannot
incur the ~157s test-binary codegen cost, so the clippy-side residual is ~44s, not ~157s;
(3) aligning `iter`'s nextest to `RUSTFLAGS=""` eliminates the ~157s rebuild on the primary
iter → check dev-loop transition (iter previously used ambient RUSTFLAGS for nextest, creating
the same fingerprint mismatch as check's unaligned clippy).

## Narrative

As a Prism developer, I want `cargo clippy` in the `just check` and `just check-fast` recipes
and `cargo nextest run -p` in the `just iter` recipe to use `RUSTFLAGS=""`, so that all
dev-loop recipes share the same RUSTFLAGS value — keeping nextest's `RUSTFLAGS=""` test-artifact
cache warm across dev-loop transitions (iter → check, check-fast → check, iter → check-fast).
This eliminates the measured ~157s nextest rebuild (Evidence table nextest build cost) caused
by the fingerprint mismatch between ambient-RUSTFLAGS steps and `RUSTFLAGS=""` steps, and
the ~44s residual clippy re-check (Evidence table clippy cost: 43.85s) in the check-fast → check
dev loop (edit → `just iter` [primary TDD inner loop] or `just check-fast` [refactor sweep] →
fix → `just check` [pre-push gate] → push).

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

The ~157s nextest build phase represents pure waste on a `just check` run that follows any
ambient-RUSTFLAGS step (e.g., `just check-fast` clippy, `just iter` nextest before this fix,
or IDE clippy-on-save): no code changed, yet all test binary targets are recompiled from scratch
because the prior step populated the ambient-RUSTFLAGS bucket while nextest requires the
`RUSTFLAGS=""` bucket. The research profile infers that a second consecutive `just check`
(no intervening ambient-RUSTFLAGS step) would be fast — a plausible inference from the cargo
fingerprinting model, but not a directly measured figure and not load-bearing for this story's
value proposition. What is measured and unfalsifiable: with all dev-loop recipes aligned to
`RUSTFLAGS=""`, the nextest build phase after `just check-fast` drops from ~157s to ~1.25s
(implementation verification run, recorded in PR evidence bundle per AC-005).

**Root cause:** `cargo clippy` in the `check` and `check-fast` recipes uses the default
RUSTFLAGS (whatever the shell environment provides), as did `cargo nextest run -p` in the
`iter` recipe before this fix. `cargo nextest run` in `check` uses `RUSTFLAGS=""` explicitly.
These are different Cargo compiler fingerprints; artifacts compiled under one fingerprint are
treated as stale by the other, forcing a full test-binary rebuild each time nextest follows
any ambient-RUSTFLAGS step (clippy-only run or iter nextest before fix) that warmed the
ambient-RUSTFLAGS bucket while leaving the `RUSTFLAGS=""` bucket cold.

**The fix intent already exists in the Justfile comment:**

```
# NOTE: RUSTFLAGS="" is set explicitly on both the nextest and doctest steps so they share
# the same fingerprint cache. Without alignment, a RUSTFLAGS drift (e.g. a shell export)
```

The `check` recipe's nextest and doctest already carry `RUSTFLAGS=""`. Clippy in `check` and
`check-fast` was left unaligned; `iter`'s nextest was also left unaligned. This story closes
all three gaps.

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

Fingerprint sequence (just check following any ambient-RUSTFLAGS step — the common dev loop penalty):
1. clippy — reuses fingerprint A artifacts (default/ambient RUSTFLAGS) populated by prior ambient-RUSTFLAGS step (e.g., `just check-fast` clippy or `just iter` nextest before this fix)
2. nextest — compiles under fingerprint B (RUSTFLAGS="") → **MISMATCH → ~157s full test-binary rebuild**
3. doctest — compiles under fingerprint B → reuses nextest cache → ~8s

Note: the research profile infers a second consecutive `just check` (no intervening ambient-RUSTFLAGS
step) would be fast — a plausible cargo-model inference, not a directly measured figure.
The story's value is independent of this inference: aligning all dev-loop recipes to `RUSTFLAGS=""`
eliminates the fingerprint mismatch that triggers the ~157s rebuild on any ambient-RUSTFLAGS →
`just check` transition.

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

Fingerprint sequence (just check following just check-fast or just iter — with all three fixes applied):
1. clippy — reuses fingerprint B (RUSTFLAGS="") artifacts populated by prior `just check-fast` (now aligned)
2. nextest — reuses fingerprint B → **MATCH → implementation verification run measured ~1.25s (to be recorded in PR evidence bundle per AC-005); also matches artifacts from prior `just iter` (now aligned)**
3. doctest — reuses fingerprint B → reuses nextest cache → ~8s

**Estimated savings — three separable effects:**

- **Effect 1 (~157s saving from aligning `check`'s clippy, Evidence table nextest build
  cost):** The mismatch is internal to the `check` recipe: clippy runs under ambient
  RUSTFLAGS, nextest uses `RUSTFLAGS=""`. These produce different fingerprints; any
  ambient-RUSTFLAGS step preceding `just check` (e.g., `just check-fast`, `just iter`
  before this fix, or IDE clippy-on-save) leaves nextest's `RUSTFLAGS=""` artifact cache
  cold → ~157s full rebuild. Aligning `check`'s clippy to `RUSTFLAGS=""` keeps nextest's
  artifact cache warm across dev-loop transitions, and the rebuild disappears.

- **Effect 2 (~44s additional saving from aligning `check-fast`'s clippy, Evidence table
  clippy cost: 43.85s):** Once Effect 1 is applied, `check-fast`'s clippy (ambient RUSTFLAGS)
  populates the ambient artifact bucket; `check`'s now-aligned clippy needs the `RUSTFLAGS=""`
  bucket → ~44s clippy re-check. `cargo clippy --all-features` (no `--all-targets`) does NOT
  compile test binaries; the clippy re-check costs the Evidence-table clippy figure (~44s),
  not the ~157s nextest test-binary codegen cost. Aligning `check-fast`'s clippy eliminates
  this residual ~44s penalty, making the check-fast → check dev loop fully penalty-free.

- **Effect 3 (~157s saving from aligning `iter`'s nextest, Evidence table nextest build
  cost):** Before the fix, `iter`'s `cargo nextest run -p` used ambient RUSTFLAGS, creating
  the same fingerprint mismatch as `check`'s unaligned clippy. Any `just iter` invocation
  left nextest's `RUSTFLAGS=""` artifact cache cold; the next `just check` must rebuild
  (~157s). Aligning `iter`'s nextest to `RUSTFLAGS=""` eliminates this rebuild on the
  primary iter → check TDD dev-loop transition (the most common transition for a developer
  using `just iter` as the inner loop and `just check` as the pre-push gate).

### Why `RUSTFLAGS=""` is the correct canonical value

- `RUSTFLAGS=""` explicitly overrides any ambient RUSTFLAGS to the empty string, making
  the build deterministic regardless of the developer's shell environment.
- `RUSTFLAGS=""` is already the established convention for nextest and doctest in `check`
  (per the existing Justfile comment).
- This local-recipe change is CI-neutral because CI does not invoke `just check` or
  `just check-fast` at all — it runs individual cargo steps directly. CI's `clippy` job
  runs `cargo clippy --workspace --all-features -- -D warnings` with no RUSTFLAGS env
  override; this produces a fingerprint DISTINCT from `RUSTFLAGS=""` (per EC-006), but
  since CI never executes `just check` or `just check-fast`, CI's artifact cache is
  entirely separate and is unaffected by this Justfile change. CI's `test` job nextest
  and doctest steps DO set `RUSTFLAGS=-C link-arg=-fuse-ld=mold` on Linux (to use the
  mold linker), but this is unrelated to the local Justfile change and does not affect
  the fingerprint alignment this story addresses.
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
RUSTFLAGS (no `RUSTFLAGS=""` prefix). The fix involves two separable effects — do not
conflate the ~157s nextest rebuild (Evidence table nextest build cost) with the ~44s clippy
re-check (Evidence table clippy cost: 43.85s):

**Effect 1 — ~157s nextest rebuild (fixed by aligning `check`'s clippy, not `check-fast`):**
The mismatch is internal to the `check` recipe: `check`'s clippy runs under ambient RUSTFLAGS
while `check`'s nextest uses `RUSTFLAGS=""`. These produce different fingerprints; any
ambient-RUSTFLAGS step preceding `just check` (e.g., `just check-fast`, `just iter` before
this fix, or IDE clippy-on-save) leaves nextest's `RUSTFLAGS=""` cache cold — so the next
`just check`'s nextest step must rebuild (~157s, Evidence table nextest build cost). Aligning
`check`'s own clippy to `RUSTFLAGS=""` keeps the `RUSTFLAGS=""` cache warm across dev-loop
transitions, eliminating the ~157s rebuild. (The research profile infers a second consecutive
`just check` is already fast — a plausible cargo-model inference, but not a directly measured
figure and not load-bearing for this story.)

**Effect 2 — ~44s residual clippy re-check (fixed by aligning `check-fast`'s clippy):**
Once `check`'s clippy is aligned to `RUSTFLAGS=""`, the check-fast → check dev loop has a
residual penalty: `check-fast`'s clippy (ambient RUSTFLAGS) populates the ambient artifact
bucket; `check`'s clippy (now `RUSTFLAGS=""`) must recompile its artifact bucket → ~44s
clippy re-check (Evidence table clippy cost: 43.85s). `cargo clippy --all-features`
(no `--all-targets`) does NOT compile test binaries; the clippy re-check cannot incur the
~157s test-binary codegen cost. Aligning `check-fast`'s clippy to `RUSTFLAGS=""` eliminates
this residual ~44s penalty.

**Cross-recipe divergence (check → check-fast direction):** after fixing `check`'s clippy
to `RUSTFLAGS=""`, a developer who runs `just check` and then `just check-fast` also
encounters a fingerprint change: check-fast uses ambient RUSTFLAGS while check uses
`RUSTFLAGS=""` — causing a ~44s clippy re-check in check-fast (Evidence table clippy cost:
43.85s). This story prepends `RUSTFLAGS=""` to the `check-fast` clippy line to eliminate
both residual clippy re-check directions.

### `iter` is in scope

The `iter` recipe is the project's primary TDD inner loop (per CLAUDE.md:
`just iter <crate> [test_filter]` — single crate, fast iteration ~10-30 sec warm).
Before this fix, `iter`'s `cargo nextest run -p` line used ambient RUSTFLAGS (no `RUSTFLAGS=""`
prefix). This creates the same fingerprint mismatch as `check`'s unaligned clippy: any
`just iter` invocation (ambient RUSTFLAGS nextest) leaves the `RUSTFLAGS=""` artifact bucket
cold — so the next `just check`'s nextest step must rebuild all test binaries (~157s,
Evidence table nextest build cost).

**Effect 3 — ~157s nextest rebuild on iter → check transition (fixed by aligning `iter`'s nextest):**
Adding `RUSTFLAGS=""` to `iter`'s `cargo nextest run -p` line aligns the primary TDD inner
loop with the pre-push gate fingerprint. After this fix, a developer who runs `just iter <crate>`
then `just check` does not incur the ~157s full test-binary rebuild — the `RUSTFLAGS=""` bucket
populated by `just iter` is reused directly by `just check`'s nextest step. This is the same
mechanism as Effect 1 (aligning `check`'s clippy), applied to the iter → check transition.

Note: `just iter` only runs `cargo nextest run -p <crate>` — it does NOT run `cargo clippy`.
There is no clippy alignment needed for `iter`. The fix is a single `RUSTFLAGS=""` prefix on
the `cargo nextest run -p` line in the `iter` recipe.

## Scope

One file modified; three `RUSTFLAGS=""` insertions (required) plus comment-block updates on all three dev-loop recipes (`check`, `check-fast`, `iter`):

| File | Change | Rationale |
|------|--------|-----------|
| `Justfile` | Prepend `RUSTFLAGS="" ` to `cargo clippy --all-features -- -D warnings` in the `check` recipe | Aligns clippy fingerprint with nextest/doctest; eliminates the ~157s rebuild |
| `Justfile` | Prepend `RUSTFLAGS="" ` to `cargo clippy --all-features -- -D warnings` in the `check-fast` recipe | Aligns check-fast clippy fingerprint with check; eliminates cross-recipe clippy divergence |
| `Justfile` | Prepend `RUSTFLAGS="" ` to `cargo nextest run -p` in the `iter` recipe | Aligns iter nextest fingerprint with check/check-fast nextest; eliminates ~157s rebuild on iter → check transition (primary TDD inner loop per CLAUDE.md) |
| `Justfile` | Rewrite preceding comment block on the `check` recipe to document the full `RUSTFLAGS=""` convention (all dev-loop recipes now aligned) | Behavioral-anchor rewrite, not cosmetic; records the full convention for future contributors (OBS-006-001 in-scope treatment) |
| `Justfile` | Add preceding comment block to the `check-fast` recipe documenting that its clippy fingerprint is aligned with `check` | Behavioral-anchor addition, not cosmetic; mirrors the comment-block treatment of `check` (F-006-LOW-001 in-scope) |
| `Justfile` | Add preceding comment block to the `iter` recipe documenting that its nextest fingerprint is aligned with `check` | Behavioral-anchor addition; documents the fingerprint convention for the primary TDD inner loop (F-006-P-MED-002 in-scope treatment) |

**NOT in scope:**

- `Justfile` `check-ci` recipe — no change needed (internally consistent; see EC-004)
- `.config/nextest.toml` — no change needed
- `.cargo/config.toml` — no change needed
- `.github/workflows/ci.yml` — no change needed
- Any production Rust code
- Any `.factory/` file — state-manager handles STORY-INDEX registration

## Acceptance Criteria

### AC-001 — `check` and `check-fast` recipes: RUSTFLAGS="" prefix present on clippy lines

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

### AC-005 — `just check` following `just check-fast` does not trigger a full test-binary rebuild

Run `just check-fast` to warm the ambient-RUSTFLAGS fingerprint bucket, then run `just check`
immediately with no code changes:

```
just check-fast     # warms the clippy-only (ambient-RUSTFLAGS) bucket
just check          # observe the nextest build phase
```

The nextest build phase must complete in < 30s. Research estimate: ~5-10s incremental.
An implementation verification run measured ~1.25s (within/better than the estimate) —
do not conflate this with the §2a per-test average of 1.25s in the research profile; these
are distinct quantities. This figure is the build-phase wall-clock from the S-PERF-GATE-006
implementation verification run and is not yet a persisted artifact. It becomes traceable at
merge time when recorded in the PR evidence bundle. AC-005 explicitly requires the
build-phase wall-clock to be captured in the PR evidence bundle, so the measurement is
grounded to that artifact at the point of merge, not to a pre-existing persisted source.
No `Compiling` lines should appear during the nextest build
phase that correspond to test binary targets.

Before the fix, this sequence shows ~157s of Compiling lines in the nextest build phase:
`just check-fast` warmed the ambient-RUSTFLAGS bucket and nextest requires the RUSTFLAGS=""
bucket — full rebuild. After the fix, `just check-fast` populates the RUSTFLAGS="" bucket
(via AC-007), so nextest reuses it without a rebuild.

This AC specifically measures the check-fast → check scenario (the common refactor-sweep →
pre-push-gate transition). The iter → check scenario (primary TDD loop) is additionally
covered by AC-008's grep verification; the same ~157s penalty is eliminated there via the
iter nextest alignment.

This AC is measurement-based. Record the nextest build phase wall-clock for the PR
description changelog. If the sequence still shows 100+ Compiling lines, the RUSTFLAGS
prefix may have a typo or trailing whitespace — re-verify AC-001, AC-007, and AC-008.

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

### AC-008 — `iter` recipe: RUSTFLAGS="" prefix present on the nextest run line (primary TDD inner loop alignment)

```
grep -A10 '^iter ' Justfile | grep -c 'RUSTFLAGS="" .*cargo nextest run -p'
```

Expected output: `1`.

Source-verification: this grep uses context anchoring (`-A10`: 10 lines after the `iter `
recipe header) to isolate the `iter` recipe and verify the `RUSTFLAGS=""` prefix is present
on its `cargo nextest run -p` line. Before the change, the output is `0` (the `iter` nextest
line lacks the prefix). After the change, it is `1`.

This AC covers the project's primary TDD inner loop (per CLAUDE.md: `just iter <crate>
[test_filter]` — single crate, fast iteration ~10-30 sec warm). Without this alignment, a
developer using `just iter` (ambient RUSTFLAGS nextest) followed by `just check`
(RUSTFLAGS="" nextest) incurs the same ~157s full test-binary rebuild as the check-fast →
check transition fixed by AC-005.

Traces to: BC-5.39.001 postcondition — delivery quality; the `iter` nextest fingerprint
must be aligned with `check` to prevent cache thrash on the primary TDD dev loop
(F-006-P-MED-002 in-scope treatment).

## Red Gate

Zero Red Gate tests. This story makes no changes to production Rust source code. The only
file modified is `Justfile`. There is no `todo!()` stub to introduce and no failing test to
write first. Validation is via `just check` exit code (AC-006), grep assertions (AC-001
through AC-004, AC-007, AC-008), and warm-cache timing observation (AC-005).

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

1. **Read** the `check`, `check-fast`, and `iter` recipes in `Justfile` to confirm
   the current `cargo clippy` lines have no `RUSTFLAGS=""` prefix (in `check` and
   `check-fast`) and that the `iter` `cargo nextest run -p` line has no `RUSTFLAGS=""`
   prefix. Confirm the existing nextest/doctest lines in `check` already carry `RUSTFLAGS=""`.

2. **Edit** `Justfile`:
   - In BOTH the `check` recipe AND the `check-fast` recipe, change each occurrence of:
     ```
     cargo clippy --all-features -- -D warnings
     ```
     to:
     ```
     RUSTFLAGS="" cargo clippy --all-features -- -D warnings
     ```
   - In the `iter` recipe, prepend `RUSTFLAGS=""` to the `cargo nextest run -p` line:
     ```
     RUSTFLAGS="" cargo nextest run -p ...
     ```
   Three lines are modified (one per recipe). Do NOT modify any other line in the file.

   Update the preceding comment block in `check` to note that ALL dev-loop recipes now
   carry `RUSTFLAGS=""` (including `iter`), and that `check-fast` and `iter` are aligned
   (OBS-006-001 — behavioral-anchor rewrite, not cosmetic). Add a preceding comment block
   to the `check-fast` recipe documenting clippy fingerprint alignment with `check`
   (F-006-LOW-001 in-scope treatment). Add a preceding comment block to the `iter` recipe
   documenting nextest fingerprint alignment with `check` (F-006-P-MED-002 in-scope
   treatment). Three comment changes total: one rewrite on `check`, one addition on
   `check-fast`, one addition on `iter`.

3. **Verify** AC-001 through AC-004, AC-007, and AC-008 grep commands each return their
   expected values. Run each grep before running `just check`.

4. **Run** `just check` once (with no preceding `just check-fast`) to pre-populate the
   RUSTFLAGS="" bucket for all steps. The nextest build phase on this first run will be
   slow (~157s) if the RUSTFLAGS="" bucket was not already warm — this is expected and
   one-time.

5. **Run** `just check-fast` followed immediately by `just check` with no code changes.
   This is the dev-loop scenario AC-005 validates. Observe the nextest build phase: confirm
   no (or very few) `Compiling` lines, and that the build phase completes in < 30s
   (research estimate: ~5-10s; implementation verification run measured ~1.25s — the actual
   wall-clock from this run must be recorded in the PR evidence bundle per AC-005 to be
   traceable). Record the nextest build phase wall-clock for AC-005 confirmation.

6. **Run** `just check` a final time and verify AC-006 (`Exit: 0`).

7. **Confirm** the ONLY modified file is `Justfile` (no production Rust code changes, no
   `.config/nextest.toml` changes, no story-index changes — state-manager handles index
   registration).

## Token Budget Estimate

| Context component | Estimated tokens |
|-------------------|-----------------|
| This story spec (v2.0, ~680 lines) | ~8,200 |
| `Justfile` (full file, ~220 lines — read + modify) | ~2,000 |
| AC verification grep outputs (7 commands) | ~350 |
| `just check` output (two warm runs, abbreviated) | ~2,000 |
| **Total** | **~12,550** |

Well within the implementer agent's context window. Simpler than S-PERF-GATE-003 (one-word
or one-line insertion per recipe in one file; no nextest.toml surgery, no shell script changes).

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
| `Justfile` | Modify | Prepend `RUSTFLAGS="" ` to the `cargo clippy` line in both the `check` and `check-fast` recipes and to the `cargo nextest run -p` line in the `iter` recipe; rewrite the preceding comment block on `check` to document the full `RUSTFLAGS=""` convention across all dev-loop recipes; add a preceding comment block to `check-fast` documenting clippy fingerprint alignment with `check`; add a preceding comment block to `iter` documenting nextest fingerprint alignment with `check` (all comment changes are behavioral-anchor updates per F-006-LOW-001 and F-006-P-MED-002 in-scope treatment) |

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
        └── Edit Justfile (RUSTFLAGS="" on clippy lines in check and check-fast, nextest line in iter)
```

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Developer has `RUSTFLAGS` set in their shell environment (e.g., `export RUSTFLAGS="-C instrument-coverage"`) | `RUSTFLAGS="" cargo clippy` explicitly overrides the ambient RUSTFLAGS with the empty string. The developer's shell RUSTFLAGS is ignored for this one clippy invocation — matching the existing behavior of the nextest and doctest steps. This is the intended behavior per the Justfile comment. |
| EC-002 | First `just check` run after the fix shows a full rebuild (~157s nextest build phase) | This is expected on the FIRST run after the change because the cache needs to be rebuilt under the new (aligned) fingerprint. All subsequent runs with no code changes will be fast (< 30s build phase). This is documented in the Tasks (Task 4 warms the cache). |
| EC-003 | `RUSTFLAGS=""` breaks a clippy lint that previously relied on an ambient RUSTFLAGS value | No currently-passing clippy lint in the workspace depends on RUSTFLAGS. `-D warnings` is in the cargo argument vector, not RUSTFLAGS. AC-006 (`just check` exit 0) is the safety net. If AC-006 fails, inspect `RUSTFLAGS="" cargo clippy --all-features -- -D warnings 2>&1` output directly. |
| EC-004 | Cross-recipe fingerprint drift: developer runs `just check` (RUSTFLAGS="" on all non-fmt steps) then `just check-ci` (default RUSTFLAGS on all steps) | `check-ci` uses default RUSTFLAGS for ALL steps — clippy, nextest, and doctest — making it internally consistent. This story introduces a NEW `check` ↔ `check-ci` CLIPPY fingerprint divergence (in addition to the pre-existing nextest divergence): `check` clippy now runs under `RUSTFLAGS=""` while `check-ci` clippy still uses ambient RUSTFLAGS. Before this story, only nextest caused cross-recipe cache misses when switching between `check` and `check-ci`; after this story, both clippy AND nextest produce cache misses. The net effect is marginal — `check` ↔ `check-ci` already forced a full nextest/doctest rebuild pre-story, so the added clippy recompile is a small increment to an already-full rebuild, not a new category of waste. **Deliberate design decision (all-three-or-none):** `check-ci` is intentionally left on ambient RUSTFLAGS for all steps. Aligning only `check-ci`'s clippy to `RUSTFLAGS=""` while leaving nextest and doctest on ambient would CREATE a new check-ci-internal fingerprint mismatch — the same class of problem this story fixes in `check`. `check-ci` must align ALL three non-fmt steps simultaneously or none. That full three-step alignment of `check-ci` is intentionally out of this story's scope — it is a permanent scope boundary, not a deferred fix. |
| EC-005 | New step added to `check` recipe after this fix that compiles Rust code without `RUSTFLAGS=""` | The new step would create a fingerprint mismatch if it compiles under a different RUSTFLAGS. This is a process discipline issue for future stories. The Justfile comment explains the convention; implementers of future `check` steps must add `RUSTFLAGS=""` to maintain consistency. |
| EC-006 | Ambient RUSTFLAGS vs explicit `RUSTFLAGS=""` — fingerprint distinctness | In this workspace's `just check` context, the clippy step's ambient RUSTFLAGS and an explicit `RUSTFLAGS=""` produce DISTINCT cargo build fingerprints — empirically verified: aligning the clippy step to `RUSTFLAGS=""` eliminated the ~157s test-target rebuild entirely. Cargo incorporates the RUSTFLAGS value into its build fingerprint; therefore all non-CI clippy steps must use the same explicit `RUSTFLAGS=""` value to share the artifact cache with the nextest and doctest steps (which already carry `RUSTFLAGS=""`). The explicit `RUSTFLAGS=""` form also makes the fingerprint environment-independent and deterministic: a developer with an ambient RUSTFLAGS export (e.g., `export RUSTFLAGS="-C instrument-coverage"`) sees a different fingerprint than a developer with no RUSTFLAGS set, causing non-reproducible cache behavior across workstations. The observed rebuild-elimination is the authoritative evidence: the two states demonstrably produced different fingerprints in this workspace. Do NOT assume "empty string and unset produce identical fingerprints" — that reasoning is unprovable for all cargo versions and contradicts the measured evidence this story rests on. |

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 2.0 | 2026-07-01 | story-writer | F-006-P-MED-002 (iter scope): expanded scope to include the `just iter` recipe's nextest line (`RUSTFLAGS=""` on `cargo nextest run -p`), aligning the project's primary TDD inner loop (per CLAUDE.md) with `check`/`check-fast`. Updated: frontmatter title + version; tdd_mode rationale comment (three recipes); intro paragraph (three effects, all dev-loop transitions); narrative (iter added); §Background saved estimates (two → three effects; Effect 3 added); post-fix fingerprint sequence (now mentions iter); new `### iter is in scope` subsection; §Scope prose + table (iter row added); Task 1 (read iter recipe); Task 2 (three-recipe edit, three comment changes); Task 3 (AC-008 added); AC-008 (new iter nextest grep); Token Budget (v2.0, ~680 lines, ~12,550 tokens); §File Structure Requirements (Justfile row updated). F-006-P-MED-001 (causal-narrative de-inference): removed "second consecutive just check is fast" as a load-bearing trigger for Effect 1. All four occurrences (§Evidence note, §Background pre-fix sequence note, §Background Effect 1, §check-fast scope Effect 1) are now explicitly hedged as "the research profile infers … a plausible cargo-model inference, not a directly measured figure." Value proposition simplified to the measured and unfalsifiable claim: aligning all dev-loop recipes to `RUSTFLAGS=""` keeps nextest's test-artifact cache warm across transitions, eliminating the measured ~157s→~1.25s rebuild. No load-bearing claim now requires an unmeasured cargo-cache model. AC-005 note updated (removes second-consecutive-check framing; cites iter scenario covered by AC-008). |
| 1.9 | 2026-07-01 | story-writer | MED-1 causal-narrative simplification: removed all "nextest reuses clippy's library artifacts" claims (false — `cargo clippy` without `--all-targets` produces no codegen artifacts that nextest links against) and removed "regardless of whether check-fast ran first" clause (contradicts the measured second-consecutive-check-is-fast behavior in the Evidence section). Rewrote three sections to state only observed/measured behavior: (1) Narrative: replaced artifact-reuse framing with "clippy and nextest steps share the same RUSTFLAGS value — keeping nextest's RUSTFLAGS="" build cache warm"; (2) §Background Effect 1 savings estimate: replaced "nextest cannot reuse clippy's library artifacts" with "clippy-only invocation leaves nextest's RUSTFLAGS="" artifact cache cold"; (3) §check-fast is in scope Effect 1: replaced both false claims with observation-grounded text naming the trigger (clippy-only invocation under ambient RUSTFLAGS leaves nextest's cache cold) and explicitly preserving the second-consecutive-check-is-fast observation already present in the Evidence section. Narrative is now internally consistent with §Evidence. No ACs, Evidence figures, or behavioral scope changed. Token budget updated: v1.9, ~605 lines, ~7,300 tokens; total ~11,600. |
| 1.8 | 2026-07-01 | story-writer | F-006-LOW-001: spec under-described the delivered diff — `check-fast` comment-block addition was unlisted in §Scope, §Scope table, and §File Structure Requirements. Updated intro paragraph (added check-fast comment-block addition alongside check rewrite); §Scope prose (now names both recipes); §Scope table (two new rows: check comment-block rewrite + check-fast comment-block addition); Task 2 (explicit instruction to add check-fast comment block, mirrors check treatment); §File Structure Requirements (Details now lists both comment changes); frontmatter tdd_mode rationale comment (explicit "rewrite on check, addition on check-fast"). Delivery diff is now fully described: two RUSTFLAGS="" clippy-line prefixes AND two comment-block changes (one rewrite on `check`, one addition on `check-fast`). Token budget updated to v1.8, ~645 lines, ~7,800 tokens; total ~12,100. |
| 1.7 | 2026-07-01 | story-writer | OBS-2 grounding: the ~1.25s nextest build-phase figure was presented without a cited source and numerically coincides with the §2a per-test average in the research profile (conflation risk). Re-framed in all three occurrences (post-fix fingerprint sequence, AC-005, Task 5): labeled as "measured in the implementation verification run" and made explicit that this figure is not yet a persisted artifact — it becomes traceable at merge time when captured in the PR evidence bundle, which AC-005 already requires. Kept the existing disclaimer distinguishing it from the §2a per-test average. No other figures changed: all other numbers (157s, 43.85s, 1.49s, 585.84s, ~8s, ~798s, ~44s) are pinned to the Evidence table in `.factory/research/test-suite-perf-profile-2026-06-30.md` and remain correctly grounded. Token budget updated to v1.7, ~615 lines, ~7,400 tokens; total ~11,700. |
| 1.6 | 2026-07-01 | story-writer | F-006-P1-MED-002 + grounding discipline sweep: corrected the "fixing only `check` would move the ~157s rebuild from nextest to clippy" claim — `cargo clippy --all-features` (no `--all-targets`) does NOT compile test binaries and cannot incur the ~157s test-binary codegen cost; the Evidence table pins clippy at 43.85s (~44s). Separated the two fix effects with explicit Evidence table traces: (1) aligning `check`'s clippy eliminates the ~157s nextest rebuild by resolving the clippy(ambient)↔nextest(RUSTFLAGS="") fingerprint mismatch internal to the `check` recipe (Evidence table nextest build cost); (2) aligning `check-fast`'s clippy eliminates the ~44s residual clippy re-check in the check-fast → check dev loop (Evidence table clippy cost: 43.85s). Fixed incorrect narrative claim "test binary artifacts compiled by clippy" — clippy without `--all-targets` compiles library/binary targets, not test binaries; changed to "library artifacts compiled by clippy are reusable by nextest." Changed "approximately 150 seconds" (narrative) to trace explicitly to Evidence table (~157s nextest, ~44s clippy). Rewrote intro, narrative, Background savings estimate, and check-fast scope section to keep the two effects clearly separate with Evidence table source citations throughout. Updated Token Budget: v1.6, ~600 lines, ~7,200 tokens; total ~11,500. |
| 1.5 | 2026-07-01 | story-writer | F-006-MED-001 causal-model reconciliation: corrected the penalty trigger throughout — the ~157s rebuild occurs on `just check` following a clippy-only run (e.g., `just check-fast`, IDE clippy-on-save), NOT on every consecutive warm `just check`. A second consecutive `just check` is already fast before the fix; the research profile §1 note is now the authoritative framing. Updated: frontmatter title, intro paragraph, narrative, evidence table note, Background pre-fix fingerprint sequence (header + added clarifying note), Background post-fix fingerprint sequence (header renamed; added ~1.25s measured figure), savings estimate (removed ~25 min/day inflation; added load-bearing framing for AC-007), check-fast scope section (rewrote to lead with primary check-fast→check penalty; explained why check-fast alignment is load-bearing, not supplementary), Tasks 4-5 (updated verification scenario to check-fast → check). OBS-1: updated AC-001 header to name both `check` and `check-fast` recipes (matching expected count of 2). OBS-2: AC-005 corrected pre-fix scenario (check-fast → check triggers 157s, not second consecutive check); added measured ~1.25s build-phase figure alongside ~5-10s research estimate with non-conflation note. Token Budget updated: v1.5, ~555 lines, ~6,700 tokens; total ~11,000. |
| 1.4 | 2026-07-01 | story-writer | LOW-1 (TD-VSDD-091): replaced volatile line pin "lines 14-29" in Task 1 with behavioral anchor "the `check` recipe and its preceding comment block". OBS-1: reworded intro from "Two-line fix:" to "Two clippy-line insertions plus comment-block rewrites:" to agree exactly with §Scope (acknowledges comment-block rewrites are part of the delivered change while noting the functional fix is the two clippy-line prefixes). OBS-2: removed deferral language from EC-004 ("deferred until CI caching semantics justify it") — reworded to permanent scope-boundary framing per the all-three-or-none design decision (out of this story's scope, not a deferred fix). Token Budget updated: v1.4, ~515 lines, ~6,200 tokens; total ~10,500. |
| 1.3 | 2026-07-01 | story-writer | F-006-MED-001: removed false fingerprint-equivalence claim ("RUSTFLAGS="" matches CI's effective clippy fingerprint") from canonical-value section — CI clippy fingerprint is DISTINCT from RUSTFLAGS="" per EC-006, but CI never invokes `just check`/`just check-fast` so it is irrelevant; reworded to lead with CI-neutrality argument only. F-006-MED-002: changelog reordered DESCENDING per POL-32 (newest first). OBS-006-001: scope statement corrected — now documents two clippy-line insertions PLUS comment-block rewrites; Task 2 "Optionally" removed (comment-block update is in scope). Consistency sweep: fixed "one-line fix" intro claim (two-line fix + comment blocks); updated narrative and frontmatter title to mention both check and check-fast; updated Token Budget version reference to v1.3. |
| 1.2 | 2026-06-30 | story-writer | LOCAL re-gate F1 [MED] EC-006 rewritten: old text claimed RUSTFLAGS="" and ambient produce the same cargo fingerprint — directly contradicts root-cause and measured evidence; new text anchors the claim in the observed rebuild-elimination (empirically verified: two states produced DISTINCT fingerprints). F2 [LOW] EC-004 amended: documents new check↔check-ci CLIPPY fingerprint divergence this story introduces; net effect is marginal (already forced full rebuild pre-story); floating "follow-up may" reframed as documented deliberate design decision (all-three-or-none rule; check-ci alignment deferred until CI caching semantics justify full three-step alignment simultaneously). |
| 1.1 | 2026-06-30 | story-writer | MED-2: corrected false CI-neutral claim — CI clippy job sets no RUSTFLAGS (verified in ci.yml), CI does not invoke just check or just check-fast, but CI test nextest/doctest steps DO set RUSTFLAGS=-C link-arg=-fuse-ld=mold on Linux (mold linker); MED-1: expanded scope to include check-fast clippy alignment (cross-recipe divergence), added AC-007 (check-fast recipe grep), updated AC-001 and AC-004 expected counts from 1 to 2, updated Scope table, Task 2, File Structure Requirements |
| 1.0 | 2026-06-30 | story-writer | Initial draft (T-PERF-PROFILE initiative, D-1434) |
