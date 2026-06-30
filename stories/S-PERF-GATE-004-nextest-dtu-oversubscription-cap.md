---
document_type: story
story_id: S-PERF-GATE-004
title: "nextest dtu-cap test-group (max-threads=4) — eliminate prism-dtu-* oversubscription blowup (185 min → 4 min full workspace)"
epic_id: EPIC-MAINTENANCE
version: "1.0"
status: draft
producer: story-writer
phase: 3
wave: maintenance
priority: P2
points: 2
tdd_mode: "n/a"
# tdd_mode rationale: pure config story — no production Rust code added or modified.
# No function bodies, no Red Gate tests. The only change is a [test-groups] entry and
# two [[profile.*.overrides]] stanzas in .config/nextest.toml. Validated by running
# `just check` and grepping the config file. Mutation testing (facade-mode quality gate)
# does not apply to TOML config files.
target_module: "n/a — build tooling only (.config/nextest.toml)"
subsystems: []
depends_on: [S-PERF-GATE-003]
blocks: []
behavioral_contracts: [BC-5.39.001]
# BC status: BC-5.39.001 is the delivery-quality contract (3-CLEAN convergence protocol).
# This story has no product behavioral contracts — it is a build-tooling maintenance story.
# BC-5.39.001 is already ACTIVE. POL-14 will be a NO-OP at merge.
verification_properties: []
assumption_validations: []
risk_mitigations: []
red_gate_tests: 0
estimated_days: "0.25"
---

# S-PERF-GATE-004: nextest dtu-cap Test-Group — Eliminate prism-dtu-* Oversubscription Blowup

`dtu-cap = { max-threads = 4 }` with `package(/^prism-dtu-/)` override on prepush + ci profiles

## Narrative

As a Prism developer, I want all `prism-dtu-*` test binaries to run under a shared
4-thread cap in the pre-push gate and on CI, so that the 12 DTU HTTP-server + RocksDB
binaries do not oversubscribe the CPU (previously ~684 concurrent Tokio threads), which
was inflating the full workspace run from ~4 min to ~185 min and causing BC-2.06.019
scenario-progression tests to flake.

## Background

Source of truth: `.factory/research/test-suite-performance-diagnosis-2026-06-26.md`.

### Root cause

The full `just check` nextest run (`cargo nextest run --workspace --all-features --profile
prepush`) spawns test binaries from all 12 `prism-dtu-*` packages. Each DTU binary contains
HTTP-server tests (Axum + Tokio) and database tests (RocksDB). Under nextest's default
concurrency (all binaries in parallel), this creates approximately:

- **12 DTU packages × ~16 Tokio threads each ≈ 192+ concurrent Tokio threads** plus their
  Axum HTTP listeners and RocksDB instances, contending across the full test suite.
- Wall-clock inflation factor: ~300× on affected tests. A 0.4 s test takes 125 s under
  full oversubscription.
- Projected full-suite time: ~185 min (extrapolated from 37.9% completion in 71 min with
  uncapped DTU binaries).

This is a DIFFERENT root cause from the three prior PERF-GATE fixes:

| Prior fix | Binary | Root cause | Mechanism |
|-----------|--------|------------|-----------|
| S-PERF-GATE-001 | signal_handlers | RocksDB mmap SIGSEGV under parallel subprocess spawn | serial-subprocess max-threads=1 |
| S-PERF-GATE-002 | adv_p02 | DTU clone + DataFusion boot per test (per-process re-init) | adv-p02-serial max-threads=1 |
| S-PERF-GATE-003 | bc_2_01_013 | Concurrent wiremock::MockServer startup socket contention | bc-2-01-013-serial max-threads=1 |
| **S-PERF-GATE-004** | **prism-dtu-\* (all 12)** | **Aggregate HTTP-server + RocksDB CPU/thread oversubscription across 12 packages** | **dtu-cap max-threads=4** |

The prior fixes used `max-threads = 1` to fully serialize individual binaries. This story
uses `max-threads = 4` across the package group — intentionally allowing modest concurrency
(4 DTU threads in flight at once) while preventing the explosive 192+ thread pile-up.

### Measured result (proven fix)

Full workspace run on the developer's 16-core aarch64 machine, `--profile prepush`:

| Metric | Before cap | After cap=4 |
|--------|------------|------------|
| Total wall-clock | ~185 min (extrapolated) | **220.8 s (3.7 min)** |
| Tests | 4974 | 4974 |
| Failures | — | **0** |
| BC-2.06.019 stage tests (under forced 64-thread oversubscription) | 55.8 s | **~10 s** |

The `dtu-cap = { max-threads = 4 }` constraint caps the total number of concurrent nextest
threads executing prism-dtu-* tests. This does not serialize individual tests within a
binary; it bounds the AGGREGATE thread-slot usage across all 12 DTU packages simultaneously.

### CI 4-core runners — transparent

On CI (GitHub Actions 4-core runners), the default nextest concurrency is already ≤4 CPUs.
The `dtu-cap = 4` constraint is effectively a no-op for CI runners because the global thread
pool is already ≤4. No CI regression risk.

### BC-2.06.019 scenario-progression flakiness root cause

BC-2.06.019 stage-visibility scenario tests check that Armis devices transition between
stage-0 (first-seen < 60 s) and later stages. Under DTU oversubscription, the wall-clock
time for setup phases inflated beyond 60 s, causing tests to observe an incorrect stage
value. The `now+30` band-aid introduced in S-DEMO-FIDELITY-REMEDIATION-001 worked around
this by widening the time window. With the DTU cap, the setup phase completes in ~10 s even
under forced 64-thread oversubscription, making the `now+30` band-aid revertible to
`now-10` in a follow-up fix inside S-DEMO-FIDELITY-REMEDIATION-001. That revert is tracked
in that story's deferred-items, not here.

### No overlap with existing serial groups

The existing `[[profile.prepush.overrides]]` and `[[profile.ci.overrides]]` entries from
S-PERF-GATE-001/002/003 filter by `binary(...)` (specific test binary names). The new
entry filters by `package(/^prism-dtu-/)` (all packages matching the regex). These are
orthogonal filter dimensions in nextest — a binary can match both a `binary(...)` filter
and a `package(...)` filter, but in practice there is no overlap: the three existing
serial-group binaries (`signal_handlers`, `adv_p02_e2e_pushdown_pipeline_test`,
`bc_2_01_013_spec_driven_adapter`) are all in `prism-bin`, not in any `prism-dtu-*` package.
The `package(/^prism-dtu-/)` filter does NOT match `prism-bin` tests.

## Scope

Config only. Single file modified:

| File | Change | Rationale |
|------|--------|-----------|
| `.config/nextest.toml` | Add `dtu-cap = { max-threads = 4 }` to `[test-groups]`; add `[[profile.prepush.overrides]]` and `[[profile.ci.overrides]]` with `filter = 'package(/^prism-dtu-/)'` and `test-group = 'dtu-cap'` | Caps the 12 DTU package test binaries at 4 concurrent nextest threads total, eliminating CPU/thread oversubscription |

**NOT in scope:**

- Any production Rust code changes
- Justfile changes (no recipe needs updating — `--profile prepush` already set by S-PERF-GATE-001)
- Reverting the `now+30` band-aid in S-DEMO-FIDELITY-REMEDIATION-001 (separate follow-up in that story)
- S-PERF-GATE-001/002/003 serial groups (verified-only; no modification)

## Acceptance Criteria

### AC-001 — dtu-cap group definition present in [test-groups]

```
grep -c 'dtu-cap = { max-threads = 4 }' /Users/jmagady/Dev/prism/.config/nextest.toml
```

Expected output: `1` (exactly the inline group definition line in `[test-groups]`).

Source-verification: before change, `.config/nextest.toml` `[test-groups]` block contains
`serial-subprocess = { max-threads = 1 }`, `adv-p02-serial = { max-threads = 1 }`, and
`bc-2-01-013-serial = { max-threads = 1 }`; `dtu-cap` is absent. After change, exactly
one definition line is added. The grep is anchored to the full inline-table string
`dtu-cap = { max-threads = 4 }` so it cannot match comment lines or partial fragments.

Traces to: BC-5.39.001 postcondition — delivery quality; config change is present and
correctly formed. Without this definition line, the `[[profile.*.overrides]]` `test-group`
references would be invalid (nextest would reject the config at startup).

### AC-002 — package filter overrides reference `package(/^prism-dtu-/)` on BOTH prepush and ci profiles (count = 2)

```
grep -c "filter = 'package(/^prism-dtu-/)'" /Users/jmagady/Dev/prism/.config/nextest.toml
```

Expected output: `2` (one `[[profile.prepush.overrides]]` entry, one `[[profile.ci.overrides]]`
entry — both use the same `filter =` value).

Source-verification: the filter string `filter = 'package(/^prism-dtu-/)'` is absent in the
file before the change. Using the full `filter = 'package(...)'` form anchors the grep to
the nextest filter attribute lines only, not to any comment or test-group name line.

Traces to: BC-5.39.001 postcondition — both the local pre-push profile (used by `just check`)
and the CI profile apply the dtu-cap constraint. A count of 1 would indicate only one profile
was updated (incomplete fix — CI would still oversubscribe).

### AC-003 — dtu-cap test-group assignment present in BOTH profiles (count = 2)

```
grep -c "test-group = 'dtu-cap'" /Users/jmagady/Dev/prism/.config/nextest.toml
```

Expected output: `2` (one assignment in `[[profile.prepush.overrides]]`, one in
`[[profile.ci.overrides]]`).

Source-verification: `test-group = 'dtu-cap'` is absent before the change. Anchoring to
the full `test-group = 'dtu-cap'` string prevents a false match against the group definition
line (which uses `=` in a different syntactic position).

Traces to: BC-5.39.001 postcondition — both profiles reference the group; a count of 1
would indicate only one profile was updated (residual oversubscription on the other profile).

### AC-004 — max-threads = 4 for the dtu-cap group (redundant precision check)

```
grep -c 'dtu-cap = { max-threads = 4 }' /Users/jmagady/Dev/prism/.config/nextest.toml
```

Expected output: `1` (same as AC-001 — re-stated here to make the max-threads VALUE
explicit as a separate AC for review traceability).

Note: AC-001 and AC-004 use the same grep command. They exist as separate ACs because AC-001
asserts group PRESENCE and AC-004 asserts the group VALUE (`max-threads = 4` specifically).
A future edit changing `max-threads = 8` would pass AC-001 (bare presence) but fail AC-004
(exact value), making a scope-widening regression immediately visible.

Traces to: BC-5.39.001 postcondition — `max-threads = 4` is the specific bound proven by
measurement to prevent oversubscription. A higher value (e.g., 8 or 16) would not fully
constrain the concurrent DTU load on 4-core CI runners.

### AC-005 — existing S-PERF-GATE-001/002/003 serial groups unchanged (no overlap)

```
grep -c 'serial-subprocess = { max-threads = 1 }' /Users/jmagady/Dev/prism/.config/nextest.toml
grep -c 'adv-p02-serial = { max-threads = 1 }' /Users/jmagady/Dev/prism/.config/nextest.toml
grep -c 'bc-2-01-013-serial = { max-threads = 1 }' /Users/jmagady/Dev/prism/.config/nextest.toml
```

Expected output: `1` for each (all three groups present with their original `max-threads = 1`
definitions unchanged).

Source-verification: each of these lines is present in `.config/nextest.toml` after
S-PERF-GATE-003 merged (PR #207 develop@1f491590). The implementer must not modify or remove
these entries. The count of `1` each confirms the existing serial-group definitions are intact.

Traces to: BC-5.39.001 postcondition — no regression in existing SIGTERM flake protection
(S-PERF-GATE-001), adv_p02 filter-push-down correctness (S-PERF-GATE-002), or bc_2_01_013
wiremock startup stability (S-PERF-GATE-003).

### AC-006 — full workspace prepush run completes in < 10 min with all tests passing

```
time cargo nextest run --workspace --all-features --profile prepush
```

Expected: exits 0 with all 4974 tests passing. Wall-clock: < 10 min (target ~4 min;
measured 220.8 s on the developer's 16-core aarch64 machine with the cap applied).

This AC is measurement-based, not a strict hard gate. If the run exceeds 10 min on a warm
idle machine after all changes are applied, that is a P1 finding — re-examine whether the
`package(/^prism-dtu-/)` filter is matching correctly (run `cargo nextest list --workspace
--all-features --profile prepush` to verify the filter is being applied to DTU tests).

Traces to: BC-5.39.001 postcondition — the pre-push gate must not be a multi-hour
productivity bottleneck. Evidence: 220.8 s / 4974 pass measured 2026-06-29.

### AC-007 — BC-2.06.019 scenario-progression tests complete in ≤ 60 s under the cap

```
cargo nextest run -p prism-dtu-armis --profile prepush --no-fail-fast
```

Expected: exits 0 with all prism-dtu-armis tests passing in ≤ 60 s total wall-clock.

This verifies that the stage-progression timing tests (which check device stage transitions
relative to a 60 s stage-0 threshold) complete fast enough under the cap that wall-time
inflation does not cause them to observe an incorrect stage. Measured: ~10 s under forced
64-thread oversubscription with cap=4 applied (vs. 55.8 s without cap).

The `now+30` band-aid in S-DEMO-FIDELITY-REMEDIATION-001 is tracked as a follow-up revert
in that story's deferred-items table. It is NOT in scope here. This AC only asserts that
prism-dtu-armis tests pass reliably under the cap.

Traces to: BC-5.39.001 postcondition — the cap eliminates the wall-time inflation root
cause of BC-2.06.019 scenario flakiness; test reliability under the capped schedule is
the observable quality gate.

### AC-008 — `just check` exits 0 with all changes applied

```
just check
echo "Exit: $?"
```

Expected output: `Exit: 0`.

Traces to: BC-5.39.001 postcondition — the config change must not break the pre-push gate.
A non-zero exit indicates a TOML syntax error in `.config/nextest.toml` or a test regression.

Note: this AC requires AC-001 through AC-007 to be applied first. Run it once at the end
of the implementer's work, before committing.

## Red Gate

Zero Red Gate tests. Rationale: this story makes no changes to production Rust source code.
The only file modified is `.config/nextest.toml` (a TOML config file that nextest reads).
There is no `todo!()` stub to introduce, no failing test to write first, and no production
behavior to change. The validation mechanism is `just check` exit code (AC-008) and
structural grep assertions (AC-001 through AC-005) plus wall-clock measurement (AC-006,
AC-007).

This is consistent with S-PERF-GATE-001 (0 Red Gate tests for the config-only portions),
S-PERF-GATE-002 (0 Red Gate tests — config-only), and S-PERF-GATE-003 (0 Red Gate tests).

## Behavioral Contracts

| BC | Title | Role in this story |
|----|-------|--------------------|
| BC-5.39.001 | 3-CLEAN convergence protocol | Delivery-quality gate — this story's own PR must pass 3-CLEAN before merge |

This story has no product behavioral contracts. The nextest test-group configuration change
has no observable effect on test SEMANTICS — only on test scheduling POLICY. Tests that
pass under `max-threads = 4` pass under any threading configuration (and vice versa for
correct tests); the change affects wall-clock, not correctness.

## Tasks

1. **Read** `.config/nextest.toml` to confirm the current `[test-groups]` block ends with
   `bc-2-01-013-serial = { max-threads = 1 }` and contains no `dtu-cap` entry.

2. **Edit** `.config/nextest.toml`:
   - In the `[test-groups]` block, add `dtu-cap = { max-threads = 4 }` with a comment
     block explaining WHY (12 prism-dtu-* packages × ~16 Tokio threads = CPU
     oversubscription; measured 185 min → 4 min fix; S-PERF-GATE-004).
   - After the existing `[[profile.prepush.overrides]]` stanzas for signal_handlers,
     adv_p02, and bc_2_01_013, add a new `[[profile.prepush.overrides]]` stanza:
     ```toml
     filter = 'package(/^prism-dtu-/)'
     test-group = 'dtu-cap'
     ```
   - After the existing `[[profile.ci.overrides]]` stanzas for signal_handlers, adv_p02,
     and bc_2_01_013, add a new `[[profile.ci.overrides]]` stanza (same filter + test-group).

3. **Verify** AC-001 through AC-005 grep commands each return their expected values. Run
   each in turn before running `just check`.

4. **Run** `cargo nextest run -p prism-dtu-armis --profile prepush --no-fail-fast` to
   verify AC-007 (prism-dtu-armis tests pass in ≤ 60 s).

5. **Run** `time cargo nextest run --workspace --all-features --profile prepush` to verify
   AC-006 (full workspace < 10 min). Record the wall-clock for the changelog.

6. **Run** `just check` to verify AC-008 (exit 0, no syntax error).

7. **Confirm** the ONLY modified file is `.config/nextest.toml` (no production Rust code
   changes, no Justfile changes, no story-index changes — state-manager handles index
   registration).

## Token Budget Estimate

| Context component | Estimated tokens |
|-------------------|-----------------|
| This story spec (v1.0, ~280 lines) | ~3,500 |
| `.config/nextest.toml` (full file, ~170 lines — read + modify) | ~1,800 |
| AC verification grep outputs | ~200 |
| `cargo nextest run` output (workspace run) | ~1,000 |
| **Total** | **~6,500** |

Well within the implementer agent's context window. No context-splitting required.
Simpler than S-PERF-GATE-003 (no Justfile change; no verification-only ACs for sccache/dev-setup).

## Previous Story Intelligence

### From S-PERF-GATE-001 (delivered in PR #204)

- The `[test-groups]` inline table + `[[profile.prepush.overrides]]` / `[[profile.ci.overrides]]`
  pattern is the established template. Copy-adapt: change group name, change filter type
  from `binary(...)` to `package(...)`, change max-threads from 1 to 4.
- `red_gate_tests: 0` for config-only stories — applies here.
- **AC self-verification lesson**: use `grep -c` with fully-anchored strings. Do NOT use
  bare partial patterns that would also match comment lines.

### From S-PERF-GATE-002 (merged in PR #206)

- The `adv-p02-serial` group uses `max-threads = 1` to fully serialize ONE binary.
  The `dtu-cap` group uses `max-threads = 4` to cap TWELVE packages. Different scope,
  same nextest mechanism.
- The LazyLock approach was explored and abandoned for adv_p02 because nextest is
  process-per-test-binary (statics re-initialize per process). That decision does NOT
  apply here — S-PERF-GATE-004 does not use LazyLock; it uses a package-level thread cap.

### From S-PERF-GATE-003 (merged in PR #207, develop@1f491590)

- S-PERF-GATE-003 is MERGED. The `.worktrees/S-PERF-GATE-003/` worktree has been removed.
  The implementer must branch S-PERF-GATE-004 off develop after the S-PERF-GATE-003 merge
  (post-commit 1f491590).
- The `bc-2-01-013-serial` group is the most recent addition to `[test-groups]`. Add
  `dtu-cap` AFTER `bc-2-01-013-serial` in the block to maintain chronological order.
- **grep-count-drift lesson from S-PERF-GATE-003**: every AC grep was source-verified
  against the actual file before the story was promoted to `ready`. Do the same here.
  The filter string `'package(/^prism-dtu-/)'` contains characters (`/`, `^`, `-`) that
  have no grep special-character meaning in single-quoted strings — the grep command is
  safe as-is without additional escaping.

### nextest `package(...)` filter semantics (distinct from `binary(...)`)

- `binary(name)` matches tests by their compiled TEST BINARY NAME (the filename stem of
  the `.rs` file under `tests/`).
- `package(name)` or `package(/regex/)` matches tests by their CARGO PACKAGE NAME
  (the `name = "..."` field in `Cargo.toml`).
- The 12 `prism-dtu-*` packages each have `name = "prism-dtu-<sensor>"` in their
  `Cargo.toml`. The regex `/^prism-dtu-/` matches all 12: prism-dtu-armis,
  prism-dtu-claroty, prism-dtu-common, prism-dtu-crowdstrike, prism-dtu-cyberint,
  prism-dtu-demo-server, prism-dtu-harness, prism-dtu-jira, prism-dtu-nvd,
  prism-dtu-pagerduty, prism-dtu-slack, prism-dtu-threatintel.
- `package(/^prism-dtu-/)` does NOT match `prism-bin`, `prism-core`, `prism-query`, or
  any other non-DTU package. The filter is strictly scoped.

## Architecture Compliance Rules

Extracted from `architecture/` sections and ADRs relevant to this story:

1. **ADR-022 (Arc-DI wiring)**: Not applicable — no production Rust code modified.

2. **Single-workspace MSRV (rust-toolchain.toml)**: Not applicable — no Rust code.

3. **TD-VSDD-053 (single-commit-per-burst)**: The implementer must deliver this story's
   changes in a SINGLE commit. No multi-step "Stage 1 / Stage 2" commits.

4. **No `--no-verify` hook bypass**: The `just check` verification (AC-008) requires git
   hooks to pass normally. Do not bypass hooks to deliver this story.

5. **`.config/nextest.toml` syntax constraint**: nextest overrides use `[[profile.X.overrides]]`
   (double brackets, array of tables). A single-bracket `[profile.X.overrides]` is a syntax
   error. Always use `[[...]]` for override entries.

6. **package() filter regex**: The `package(/^prism-dtu-/)` filter uses nextest's regex
   filter syntax. The regex `^prism-dtu-` matches any package name starting with `prism-dtu-`.
   Do NOT use `package(prism-dtu-*)` (glob syntax, not valid in nextest filter expressions).
   Do NOT use `binary(/prism-dtu-/)` (would match binary names, not package names —
   prism-dtu-* test binaries have names like `integration_tests`, not `prism-dtu-*`).

7. **`.factory/` not modified by this story**: No spec, BC, or error-taxonomy changes are
   required. State-manager handles story file commit and STORY-INDEX registration.

8. **No AI attribution in commits** per project git conventions (CLAUDE.md).

9. **`just check` must exit 0 before the PR is opened.**

## Library and Framework Requirements

This story does not introduce new library dependencies. The only toolchain requirement is
`cargo-nextest` (already required by `just check`). No version pin needed.

The `package(/^prism-dtu-/)` regex filter syntax is supported in nextest's filtering DSL.
This filter form (`package(/regex/)`) has been available in nextest since v0.9.49
(2023-02-07). The workspace is already on a nextest version that supports this syntax
(confirmed: existing `[profile.ci]` and `[[profile.*.overrides]]` entries in `.config/nextest.toml`
use the current DSL without issue).

## File Structure Requirements

| File | Change type | Details |
|------|-------------|---------|
| `.config/nextest.toml` | Modify | Add `dtu-cap = { max-threads = 4 }` to `[test-groups]` + add 2 override stanzas (AC-001 through AC-005) |

**Files explicitly excluded from this story:**

- `Justfile` — no change needed (`--profile prepush` already added by S-PERF-GATE-001)
- `.cargo/config.toml` — no change needed
- `docs/dev-setup.md` — no change needed
- Any `crates/prism-dtu-*/` source files — no code changes
- Any `.factory/` file — state-manager handles index registration post-delivery

## Scheduling Note

**S-PERF-GATE-003 ALREADY MERGED (PR #207, develop@1f491590). Hard dependency satisfied.**

The implementer must branch `feature/S-PERF-GATE-004` off develop HEAD 1f491590 (or later).
No merge conflict risk on `.config/nextest.toml` — S-PERF-GATE-003 added the last group
entry (`bc-2-01-013-serial`) and this story adds the next group entry (`dtu-cap`) with no
overlap in the edited lines.

Correct branching order:
```
develop (after S-PERF-GATE-003 merge — 1f491590)
  └── feature/S-PERF-GATE-004   ← branch from here
        └── Edit .config/nextest.toml (dtu-cap group + 2 overrides)
```

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `package(/^prism-dtu-/)` regex typo (e.g., `prism_dtu_` with underscores instead of hyphens) | nextest silently applies no override (no match); DTU tests still run at full concurrency. AC-002 grep catches the typo because the anchored filter string `'package(/^prism-dtu-/)'` would not match an underscore variant. |
| EC-002 | `package(...)` vs `binary(...)` confusion — implementer uses `binary(/^prism-dtu-/)` | nextest would match by binary filename, not by package name. DTU integration test binaries typically have names like `integration_tests`, not `prism-dtu-*`. The filter would match zero tests. AC-002 grep catches this because the grep string is `filter = 'package(/^prism-dtu-/)'` — a `binary(...)` variant would not match. |
| EC-003 | New `prism-dtu-*` package added to the workspace AFTER this story ships | The `package(/^prism-dtu-/)` regex automatically includes any new `prism-dtu-*` package. No story amendment is needed for future DTU additions. This is intentional — the regex is forward-compatible. |
| EC-004 | `max-threads = 4` insufficient on a future developer machine with fewer than 4 effective CPUs (e.g., a 2-core CI runner running this locally) | nextest will use min(global_concurrency, 4). On a 2-core machine, the effective cap is already ≤2, so `max-threads = 4` is a no-op. No worse than current behavior. |
| EC-005 | TOML inline-table syntax error (e.g., `dtu-cap = {max-threads = 4}` without spaces around key) | TOML spec requires space around `=` in inline tables. `{max-threads = 4}` is valid; `{ max-threads = 4 }` (with surrounding spaces) is also valid and matches the style of existing entries. AC-001 grep is anchored to `dtu-cap = { max-threads = 4 }` (with spaces) — if the implementer omits spaces, AC-001 fails. |
| EC-006 | `[[profile.prepush.overrides]]` and `[[profile.ci.overrides]]` are in different array positions from where the implementer appends | nextest evaluates ALL `[[profile.X.overrides]]` entries regardless of order. Filter matching is independent of position. AC-002 and AC-003 use `grep -c` (count, not position) so order does not affect the assertion. |
| EC-007 | prism-dtu-harness or prism-dtu-common contains no tests and the package filter matches but has no effect | nextest silently ignores packages with no tests when applying overrides. The cap applies only to packages that have runnable tests. No issue. |

## §Evidence

Full workspace measurement that motivated this story, run on developer machine (16-core
aarch64, macOS, warm cache, single concurrent cargo job):

```
time cargo nextest run --workspace --all-features --profile prepush
  (with dtu-cap = { max-threads = 4 } applied)

     Running 4974 tests across 26 test binaries (354 skipped)
         PASS [  220.824s]

real    3m40.824s
user    19m28.351s
sys     2m15.124s
```

Before (extrapolated from uncapped DTU run at 37.9% completion in 71 min):
- 37.9% × 71 min / 0.379 ≈ 187 min (rounded to ~185 min in this story's narrative)

The `dtu-cap = 4` constraint reduces the user-time overhead (CPU seconds) from ~19 min to
3.7 min wall-clock — a ~50× wall-clock improvement on this machine.

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-06-29 | story-writer | Initial draft |
