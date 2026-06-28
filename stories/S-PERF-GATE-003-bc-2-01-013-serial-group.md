---
document_type: story
story_id: S-PERF-GATE-003
title: "Remaining test/build-gate performance levers — bc_2_01_013 serialization + sccache opt-in + RUSTFLAGS dedup"
epic_id: EPIC-MAINTENANCE
version: "1.0"
status: draft
producer: story-writer
phase: 3
wave: maintenance
priority: P3
points: 2
tdd_mode: "n/a"
# tdd_mode rationale: pure config/build-tooling story — no production Rust code added or
# modified. No function bodies, no Red Gate tests. The only change is a [test-groups] entry
# in .config/nextest.toml. Changes are verified by running `just check` and grepping the
# config files. Mutation testing (facade-mode quality gate) does not apply to TOML config.
target_module: "n/a — build tooling only (.config/nextest.toml, .cargo/config.toml, Justfile, docs/dev-setup.md)"
subsystems: []
depends_on: [S-PERF-GATE-002]
blocks: []
behavioral_contracts: [BC-5.39.001]
# BC status: BC-5.39.001 is the delivery-quality contract (3-CLEAN convergence protocol).
# This story has no product behavioral contracts — it is a build-tooling maintenance story.
verification_properties: []
assumption_validations: []
risk_mitigations: []
red_gate_tests: 0
estimated_days: "0.5"
---

# S-PERF-GATE-003: Remaining Test/Build-Gate Performance Levers

bc_2_01_013 serialization + sccache opt-in + RUSTFLAGS dedup

## Narrative

As a Prism developer, I want the `bc_2_01_013_spec_driven_adapter` integration test binary
to run serially in the pre-push gate and on CI, so that wiremock server startup contention
under parallel execution does not inflate its wall-clock cost from ~15s to 300+ seconds on
a loaded machine. I also want the sccache opt-in stanza and RUSTFLAGS dedup to be documented
and in place so that future contributors can enable these levers without re-researching them.

## Background

Source of truth: `.factory/research/test-suite-performance-diagnosis-2026-06-26.md`.

### Items this story delivers

| Item | Diagnosis reference | Status at story authoring |
|------|--------------------|-----------------------------|
| bc_2_01_013 serialization | §4.1 Binary 1; §6 rank #4 analog | NOT YET DONE — no nextest serial group for this binary |
| RUSTFLAGS dedup | §3, §7c, rank #3 | DONE — Justfile lines 26-27 already have `RUSTFLAGS=""` on both steps + commentary |
| sccache opt-in stanza | §7d, rank #5 | DONE — `.cargo/config.toml` lines 104-127 already have the commented stanza |
| sccache dev-setup.md note | §7d | DONE — `docs/dev-setup.md` line 161 already documents enable procedure |

The story's primary implementation work is the **bc_2_01_013 serialization** item. The
RUSTFLAGS and sccache items are already shipped; their ACs are verification-only (grep-based)
to guard against future regression.

### Why bc_2_01_013 oversubscribes

The test binary `bc_2_01_013_spec_driven_adapter` contains tests that each:

1. Start a `wiremock::MockServer` on an ephemeral port (Axum HTTP listener + Tokio runtime)
2. Construct a `reqwest::Client` and issue real HTTP requests to the mock server
3. Construct a `SpecDrivenSensorAdapter` and call `fetch()` through the full pipeline

Under nextest's default per-binary parallelism, multiple tests in this binary start their
own `MockServer` instances simultaneously. On a loaded machine (parallel cargo jobs competing
for 16 cores), each wiremock startup can take 60-300 seconds due to OS-level socket setup
contention. The diagnosis observed 6 out of the 15 tests exceeding 60s in heavy-load runs
(diagnosis §4.1 table rows `bearer_static`, `plugin`, `static_cookie`, `ocsf_conformance_*`).

This is the SAME mechanism fixed for `signal_handlers` by S-PERF-GATE-001 (diagnosis
§4.1, §8 row 1): a `serial-subprocess` test group with `max-threads = 1` was added to
`.config/nextest.toml` for the `signal_handlers` binary. This story applies the same
pattern to `bc_2_01_013_spec_driven_adapter`.

**Key distinction from `adv_p02_e2e_pushdown_pipeline_test`:** The `adv_p02` binary has
a DIFFERENT root cause (full DTU clone + DataFusion boot per test; addressed by shared
`LazyLock` fixtures in S-PERF-GATE-002). The `bc_2_01_013` binary uses wiremock at the
HTTP boundary and does NOT start a full DTU clone — serialization alone is sufficient
because each test creates its own isolated `MockServer` that would conflict under parallel
startup, but is fast when run serially (each test completes in 2-8s when uncontended).

Serialization does NOT help if there is a shared-state hazard (then reset is needed).
Verification: the bc_2_01_013 tests use per-test `MockServer::start().await` and do NOT
share any `LazyLock` or `OnceLock` state. There is no global mock state to reset. Pure
serialization is the correct fix.

## Scope

Config/build-tooling only. Files modified:

| File | Change | Rationale |
|------|--------|-----------|
| `.config/nextest.toml` | Add `bc-2-01-013-serial` test group; add `[[profile.prepush.overrides]]` and `[[profile.ci.overrides]]` entries scoped to `binary(bc_2_01_013_spec_driven_adapter)` | Prevents wiremock startup contention under parallel nextest execution |

Files verified (no modification needed — already done):

| File | Verification | Source story |
|------|-------------|--------------|
| `Justfile` | `RUSTFLAGS=""` present on nextest and doctest steps | S-PERF-GATE-001 + S-PERF-GATE-002 |
| `.cargo/config.toml` | Commented `[build] rustc-wrapper = "sccache"` stanza present | S-PERF-GATE-001/002 |
| `docs/dev-setup.md` | sccache note present | S-PERF-GATE-001/002 |

**NOT in scope:**

- `adv_p02` shared fixtures (delivered by S-PERF-GATE-002)
- StageMask projection in prism-dtu-armis (separate feature story, BC-2.06.019 AC-007)
- `build_http_client_with_timeout` refactor (diagnosis rank #2; separate story if needed)
- CI architecture changes (separate story)

## Acceptance Criteria

### AC-001 — bc-2-01-013-serial group present in nextest.toml `[test-groups]`

```
grep -c 'bc-2-01-013-serial' /Users/jmagady/Dev/prism/.config/nextest.toml
```

Expected output: a positive integer (at least 1 match — the group definition itself).

Traces to: BC-5.39.001 postcondition — delivery quality; config change is present and
correctly formed. This prevents wiremock startup contention in the `bc_2_01_013_spec_driven_adapter`
binary under parallel nextest execution.

### AC-002 — prepush override scoped to bc_2_01_013_spec_driven_adapter binary

```
grep -A2 "bc_2_01_013_spec_driven_adapter" /Users/jmagady/Dev/prism/.config/nextest.toml
```

Expected output includes both `filter = 'binary(bc_2_01_013_spec_driven_adapter)'` and
`test-group = 'bc-2-01-013-serial'` lines under a `[[profile.prepush.overrides]]` stanza.

Traces to: BC-5.39.001 postcondition — the prepush profile (used by `just check`) applies
the serial group to this binary.

### AC-003 — ci override scoped to bc_2_01_013_spec_driven_adapter binary

```
grep -c "bc_2_01_013_spec_driven_adapter" /Users/jmagady/Dev/prism/.config/nextest.toml
```

Expected output: 2 (one in `[[profile.prepush.overrides]]`, one in `[[profile.ci.overrides]]`).

Traces to: BC-5.39.001 postcondition — CI also applies the serial group so the gate
wall-clock is consistent between local pre-push and CI runs.

### AC-004 — max-threads = 1 for the bc-2-01-013-serial group

```
grep -A1 'bc-2-01-013-serial' /Users/jmagady/Dev/prism/.config/nextest.toml | grep 'max-threads'
```

Expected output: a line containing `max-threads = 1`.

Traces to: BC-5.39.001 postcondition — `max-threads = 1` is the mechanism that enforces
serial execution; a higher value would not eliminate the contention.

### AC-005 — RUSTFLAGS dedup already present in Justfile (verification-only)

```
grep -c 'RUSTFLAGS=""' /Users/jmagady/Dev/prism/Justfile
```

Expected output: `2` (one on the nextest step, one on the doctest step in the `check` recipe).

Traces to: BC-5.39.001 postcondition — guards against future regression of the RUSTFLAGS
alignment that prevents fingerprint-cache invalidation between nextest and doctest steps
(diagnosis §3, §7c; ci.yml lines 127-134 pattern).

### AC-006 — sccache stanza present and commented in .cargo/config.toml (verification-only)

```
grep -c 'rustc-wrapper = "sccache"' /Users/jmagady/Dev/prism/.cargo/config.toml
```

Expected output: `1` (the opt-in stanza is present but commented out — the `grep` matches
the line regardless of comment character).

Traces to: BC-5.39.001 postcondition — sccache opt-in is documented and ready for
developers to enable after `cargo install sccache`, without breaking CI (diagnosis §7d).

### AC-007 — sccache note present in docs/dev-setup.md (verification-only)

```
grep -c 'sccache' /Users/jmagady/Dev/prism/docs/dev-setup.md
```

Expected output: at least `1`.

Traces to: BC-5.39.001 postcondition — developer discoverability of the sccache opt-in
path is documented in the canonical dev-setup reference.

### AC-008 — `just check` exits 0 with all changes applied

```
just check
echo "Exit: $?"
```

Expected output: `Exit: 0` (or the just check output ends with a clean exit).

Traces to: BC-5.39.001 postcondition — the config change must not break the pre-push gate.
A non-zero exit would indicate a syntax error in `.config/nextest.toml` or a test failure.

Note: this AC requires the config changes to be applied first (depends on AC-001 through
AC-004 being done). It is a final gate verification, not a prerequisite grep.

## Red Gate

Zero Red Gate tests. Rationale: this story makes no changes to production Rust source code.
The only file modified is `.config/nextest.toml` (a TOML config file that nextest reads).
There is no `todo!()` stub to introduce, no failing test to write first, and no production
behavior to change. The validation mechanism is `just check` exit code (AC-008) and
structural grep assertions (AC-001 through AC-007).

This is consistent with S-PERF-GATE-001 and S-PERF-GATE-002, which also had 0 Red Gate
tests for the config-only portions of their scope.

## Behavioral Contracts

| BC | Title | Role in this story |
|----|-------|--------------------|
| BC-5.39.001 | 3-CLEAN convergence protocol | Delivery-quality gate — this story's own PR must pass 3-CLEAN before merge |

This story has no product behavioral contracts. The serialization config change has no
observable effect on test SEMANTICS — only on test scheduling POLICY. Tests that pass
serially pass in parallel too (and vice versa); the change affects wall-clock, not
correctness.

## Tasks

1. **Read** `.config/nextest.toml` to confirm the current `[test-groups]` block contains
   only `serial-subprocess = { max-threads = 1 }` and no `bc-2-01-013-serial` entry.

2. **Edit** `.config/nextest.toml`:
   - Add `bc-2-01-013-serial = { max-threads = 1 }` to the `[test-groups]` block, with
     a comment explaining WHY (wiremock startup contention, diagnosis §4.1).
   - Add `[[profile.prepush.overrides]]` stanza:
     ```toml
     filter = 'binary(bc_2_01_013_spec_driven_adapter)'
     test-group = 'bc-2-01-013-serial'
     ```
   - Add `[[profile.ci.overrides]]` stanza (same filter + test-group).

3. **Verify** AC-001 through AC-007 grep commands return expected values.

4. **Run** `just check` to verify AC-008 (exit 0, no syntax error in the edited TOML).

5. **Confirm** the changes are the only modifications (no code changes, no story-index
   changes — state-manager handles index registration).

## Token Budget Estimate

| Context component | Estimated tokens |
|-------------------|-----------------|
| This story spec | ~3,000 |
| `.config/nextest.toml` (full file, ~135 lines) | ~1,500 |
| `Justfile` (verification only, ~310 lines) | ~3,500 |
| `.cargo/config.toml` (verification only, ~134 lines) | ~1,500 |
| `docs/dev-setup.md` (verification only, scan for sccache) | ~500 |
| Diagnosis research sidecar (reference for rationale) | ~4,000 |
| **Total** | **~14,000** |

Well within the implementer agent's context window. No context-splitting required.

## Previous Story Intelligence

### From S-PERF-GATE-001 (delivered in PR #204)

- The `signal_handlers` serial group pattern (`serial-subprocess = { max-threads = 1 }`,
  filter by `binary(signal_handlers)`) is the exact template for this story's change.
  Copy-adapt the structure, change group name and binary name.
- The story had `red_gate_tests: 0` for the config-only portions; same applies here.
- **AC self-verification lesson**: S-PERF-GATE-001's ACs used grep commands with count
  assertions (`grep -c` returning `2`) that were stable across file edits. Use the same
  pattern here. DO NOT use line-number references or regex that breaks with context changes.

### From S-PERF-GATE-002 (delivered in PR #205 / ready-to-merge at story authoring)

- S-PERF-GATE-002 handled `adv_p02_e2e_pushdown_pipeline_test` via a DIFFERENT mechanism:
  `LazyLock` shared fixtures + internal reset endpoints. DO NOT conflate the two mechanisms.
- For `bc_2_01_013`, the root cause is per-test wiremock startup contention, not shared
  DTU state — serialization alone is sufficient (no reset endpoint needed).
- S-PERF-GATE-002 adds its own `[[profile.prepush.overrides]]` entry for `adv_p02_e2e_pushdown_pipeline_test`.
  **Hard dependency**: this story MUST branch off develop AFTER S-PERF-GATE-002 merges to
  avoid a `[[profile.prepush.overrides]]` conflict in `.config/nextest.toml`.
- The `[test-groups]` block in S-PERF-GATE-002 adds a group for `adv_p02`. This story
  adds a SEPARATE group for `bc_2_01_013_spec_driven_adapter`. They coexist in the same
  `[test-groups]` block without conflict.
- **Nextest process-per-test lesson**: nextest spawns one OS process per test binary
  (not per test). The `max-threads = 1` config limits the thread count WITHIN a binary's
  test runner process. This means all tests in the binary run serially, which is exactly
  what we want here (no concurrent wiremock servers within the binary).

## Architecture Compliance Rules

Extracted from `architecture/` sections and ADRs relevant to this story:

1. **ADR-022 (Arc-DI wiring)**: Not applicable — no production Rust code modified.

2. **Single-workspace MSRV (rust-toolchain.toml)**: Not applicable — no Rust code.

3. **TD-VSDD-053 (single-commit-per-burst)**: The implementer must deliver this story's
   changes in a SINGLE commit to `.factory/`-adjacent files. No multi-step "Stage 1 / Stage 2"
   commits. The nextest.toml change is a single-file edit committed atomically.

4. **No `--no-verify` hook bypass**: The `just check` verification (AC-008) requires the
   git hooks to pass normally. Do not bypass hooks to deliver this story.

5. **`.config/nextest.toml` syntax constraint**: nextest TOML uses `[[profile.X.overrides]]`
   (double brackets, array of tables). A single-bracket `[profile.X.overrides]` is a syntax
   error that silently produces wrong behavior. Always use `[[...]]` for override entries.

6. **Filter syntax**: nextest filter uses `binary(name)` where `name` matches the TEST BINARY
   NAME, not the Rust crate name. The binary `bc_2_01_013_spec_driven_adapter` is compiled
   from `crates/prism-bin/tests/bc_2_01_013_spec_driven_adapter.rs`. The filter string must
   be `binary(bc_2_01_013_spec_driven_adapter)` (exact filename stem, no extension).

## Library and Framework Requirements

This story does not introduce new library dependencies. The only toolchain requirement is
`cargo-nextest` (already required by `just check`). No version pin needed.

**sccache (opt-in, pre-shipped)**: The sccache stanza in `.cargo/config.toml` references
`rustc-wrapper = "sccache"` in a commented block. When a developer uncomments this, sccache
must be installed separately (`cargo install sccache --locked` or `brew install sccache`).
No hard version pin — the sccache stanza is an opt-in comment. The story does not activate
it, so no sccache version is required in `Cargo.toml`.

## File Structure Requirements

| File | Change type | Details |
|------|-------------|---------|
| `.config/nextest.toml` | Modify | Add `bc-2-01-013-serial` group + 2 override stanzas |
| `Justfile` | No change | Verify `RUSTFLAGS=""` present on 2 lines (AC-005) |
| `.cargo/config.toml` | No change | Verify sccache stanza present (AC-006) |
| `docs/dev-setup.md` | No change | Verify sccache note present (AC-007) |

**Files explicitly excluded from this story:**

- `crates/prism-bin/tests/bc_2_01_013_spec_driven_adapter.rs` — no code changes
- `crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs` — S-PERF-GATE-002 scope
- `crates/prism-bin/src/spec_driven_adapter.rs` — no code changes
- Any `.factory/` file — state-manager handles index registration post-delivery

## Scheduling Note

**Hard dependency on S-PERF-GATE-002.**

Both stories modify `.config/nextest.toml` — specifically the `[test-groups]` block and
`[[profile.prepush.overrides]]` / `[[profile.ci.overrides]]` arrays. If this story branches
off a commit BEFORE S-PERF-GATE-002 merges, the implementer will produce a merge conflict
when the PR is rebased onto develop.

Correct branching order:

```
develop (after S-PERF-GATE-002 merge)
  └── feature/S-PERF-GATE-003
        └── Edit .config/nextest.toml (bc-2-01-013-serial group)
```

The state-manager must confirm S-PERF-GATE-002's PR has merged to develop before dispatching
this story's implementer.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `.config/nextest.toml` syntax error (e.g., `[test-groups.bc-2-01-013-serial]` instead of `[test-groups]` inline value) | `cargo nextest run` exits non-zero with TOML parse error; AC-008 catches this |
| EC-002 | Binary name typo (e.g., `bc-2-01-013-spec-driven-adapter` with hyphens instead of underscores) | nextest silently applies no override (no match); the test binary still runs under default parallelism. AC-002 grep catches the typo because the filter string will not match the expected binary name pattern. |
| EC-003 | S-PERF-GATE-002 NOT yet merged when implementer branches | Merge conflict on `[[profile.prepush.overrides]]` when PR is created. Scheduling Note in this story prevents this; orchestrator must enforce the dependency. |
| EC-004 | Developer uncomments `rustc-wrapper = "sccache"` without sccache installed | Cargo invocations fail with "error: failed to run custom build command ... sccache: No such file or directory". Not a concern for this story (sccache stays commented); documented in `.cargo/config.toml` inline comment. |

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-06-28 | story-writer | Initial draft |
