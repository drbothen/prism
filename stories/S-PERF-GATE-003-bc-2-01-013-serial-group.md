---
document_type: story
story_id: S-PERF-GATE-003
title: "Remaining test/build-gate performance levers — bc_2_01_013 serialization + check-ci --profile ci + sccache opt-in + RUSTFLAGS dedup"
epic_id: EPIC-MAINTENANCE
version: "1.2"
status: merged
producer: story-writer
phase: 3
wave: maintenance
priority: P3
points: 2
tdd_mode: "n/a"
# tdd_mode rationale: pure config/build-tooling story — no production Rust code added or
# modified. No function bodies, no Red Gate tests. The only changes are a [test-groups]
# entry in .config/nextest.toml and a --profile ci flag in the Justfile check-ci recipe.
# Changes are verified by running `just check` and grepping the config files. Mutation
# testing (facade-mode quality gate) does not apply to TOML config or Justfile recipes.
target_module: "n/a — build tooling only (.config/nextest.toml, Justfile, .cargo/config.toml, docs/dev-setup.md)"
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

bc_2_01_013 serialization + check-ci --profile ci fix + sccache opt-in + RUSTFLAGS dedup

## Narrative

As a Prism developer, I want the `bc_2_01_013_spec_driven_adapter` integration test binary
to run serially in the pre-push gate and on CI, so that wiremock server startup contention
under parallel execution does not inflate its wall-clock cost from ~15s to 300+ seconds on
a loaded machine. I also want the sccache opt-in stanza and RUSTFLAGS dedup to be documented
and in place so that future contributors can enable these levers without re-researching them.

## Background

Source of truth: `.factory/research/test-suite-performance-diagnosis-2026-06-26.md`.

### Items this story delivers

| Item | Diagnosis / source | Status at story authoring |
|------|--------------------|-----------------------------|
| bc_2_01_013 serialization | §4.1 Binary 1; §6 rank #4 analog | NOT YET DONE — no nextest serial group for this binary |
| check-ci --profile ci fix | OBS-1 / EC-004 routed from S-PERF-GATE-002 cascade | NOT YET DONE — Justfile check-ci recipe line 60 uses `--no-fail-fast` with no `--profile ci`; serial-group overrides are silently skipped when check-ci is run |
| RUSTFLAGS dedup | §3, §7c, rank #3 | DONE — Justfile lines 26-27 already have `RUSTFLAGS=""` on both steps + commentary |
| sccache opt-in stanza | §7d, rank #5 | DONE — `.cargo/config.toml` lines 104-127 already have the commented stanza |
| sccache dev-setup.md note | §7d | DONE — `docs/dev-setup.md` line 161 already documents enable procedure |

The story's primary implementation work is the **bc_2_01_013 serialization** item and the
**check-ci --profile ci fix**. The RUSTFLAGS and sccache items are already shipped; their
ACs are verification-only (grep-based) to guard against future regression.

### Why check-ci needs --profile ci

The `just check-ci` recipe is documented as "identical to CI behavior." However, its nextest
invocation (line 60 of the Justfile) currently reads:

```
cargo nextest run --workspace --all-features --no-fail-fast
```

This runs under the **default** nextest profile, which does NOT include the `[[profile.ci.overrides]]`
serial-group entries for `signal_handlers` (S-PERF-GATE-001) or `adv_p02` (S-PERF-GATE-002), or
the `bc_2_01_013_spec_driven_adapter` entry added by this story. The CI comment is therefore
inaccurate, and a developer running `just check-ci` would see different parallelism (and potentially
different wall-clock behavior) than the actual CI run that uses `--profile ci`.

The fix is minimal: add `--profile ci` to the nextest invocation in the check-ci recipe:

```
cargo nextest run --workspace --all-features --no-fail-fast --profile ci
```

This makes `just check-ci` truly identical to CI nextest behavior.

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
| `Justfile` | Add `--profile ci` to the nextest invocation in the `check-ci` recipe (line 60) | Makes `just check-ci` truly identical to CI — without `--profile ci` the serial-group overrides are silently skipped |

Files verified (no modification needed — already shipped):

| File | Verification | Source story |
|------|-------------|--------------|
| `Justfile` | `RUSTFLAGS=""` present on both nextest and doctest steps of the `check` recipe | S-PERF-GATE-001 + S-PERF-GATE-002 |
| `.cargo/config.toml` | Commented `[build] rustc-wrapper = "sccache"` stanza present | S-PERF-GATE-001/002 |
| `docs/dev-setup.md` | sccache note present | S-PERF-GATE-001/002 |

**NOT in scope:**

- `adv_p02` shared fixtures (delivered by S-PERF-GATE-002)
- StageMask projection in prism-dtu-armis (separate feature story, BC-2.06.019 AC-007)
- `build_http_client_with_timeout` refactor (diagnosis rank #2; separate story if needed)
- CI architecture changes (separate story)

## Acceptance Criteria

### AC-001 — bc-2-01-013-serial group definition present in nextest.toml

```
grep -c 'bc-2-01-013-serial = { max-threads = 1 }' /Users/jmagady/Dev/prism/.config/nextest.toml
```

Expected output: `1` (exactly the inline group definition line in `[test-groups]`).

Source-verification: before change, `.config/nextest.toml` contains only
`serial-subprocess = { max-threads = 1 }` in `[test-groups]`; `bc-2-01-013-serial` is
absent. After change, exactly one definition line is added. The grep is anchored to the
full inline-table string `bc-2-01-013-serial = { max-threads = 1 }` so it cannot match
comment lines or partial fragments.

Traces to: BC-5.39.001 postcondition — delivery quality; config change is present and
correctly formed. This is the mechanism that enforces serial execution; without
`max-threads = 1` the contention would not be eliminated.

### AC-002 — prepush override references bc_2_01_013_spec_driven_adapter via binary() filter

```
grep -c "filter = 'binary(bc_2_01_013_spec_driven_adapter)'" /Users/jmagady/Dev/prism/.config/nextest.toml
```

Expected output: `2` (one `[[profile.prepush.overrides]]` entry, one `[[profile.ci.overrides]]`
entry — both use the same `filter =` value).

Source-verification: the filter string `filter = 'binary(bc_2_01_013_spec_driven_adapter)'`
is unique to this story's additions and appears zero times before the change. Using the
full `filter = 'binary(...)'` form anchors the grep to the nextest filter attribute lines only,
not to any comment or test-group name line.

Traces to: BC-5.39.001 postcondition — the prepush profile (used by `just check`) and the
ci profile both apply the serial group to this binary. Count of 2 confirms both overrides
are present.

### AC-003 — ci override assigns bc-2-01-013-serial test group

```
grep -c "test-group = 'bc-2-01-013-serial'" /Users/jmagady/Dev/prism/.config/nextest.toml
```

Expected output: `2` (one assignment in `[[profile.prepush.overrides]]`, one in
`[[profile.ci.overrides]]`).

Source-verification: `test-group = 'bc-2-01-013-serial'` is absent before the change.
Anchoring to the full `test-group = 'bc-2-01-013-serial'` string prevents a false match
against the group definition line (which uses `=` in a different syntactic position).

Traces to: BC-5.39.001 postcondition — both profiles reference the group; a count of 1
would indicate only one profile was updated (incomplete change).

### AC-004 — max-threads = 1 for the bc-2-01-013-serial group (redundant precision check)

```
grep -c 'bc-2-01-013-serial = { max-threads = 1 }' /Users/jmagady/Dev/prism/.config/nextest.toml
```

Expected output: `1` (same as AC-001 — re-stated here to make the max-threads requirement
explicit as a separate AC for review traceability).

Note: AC-001 and AC-004 use the same grep command. They exist as separate ACs because AC-001
asserts group PRESENCE and AC-004 asserts the group VALUE (`max-threads = 1` specifically).
A future edit changing `max-threads = 2` would pass AC-001 (bare presence) but fail AC-004
(exact value), making the regression immediately visible.

Traces to: BC-5.39.001 postcondition — `max-threads = 1` is the specific mechanism that
serializes execution; a higher value would leave residual contention.

### AC-005 — check-ci recipe uses --profile ci for nextest (OBS-1 / EC-004 fix)

```
grep -c "cargo nextest run.*--profile ci" /Users/jmagady/Dev/prism/Justfile
```

Expected output: `1` (the check-ci recipe's nextest invocation now includes `--profile ci`).

Source-verification: before change, the `check-ci` recipe line 60 is
`cargo nextest run --workspace --all-features --no-fail-fast` (no `--profile`). After the
fix it becomes `cargo nextest run --workspace --all-features --no-fail-fast --profile ci`.
The grep `cargo nextest run.*--profile ci` uses `.*` to match any flags between the command
start and `--profile ci`; it cannot match lines that lack `--profile ci` entirely. Before
the change, this grep returns 0; after, it returns 1.

This is the OBS-1 finding from S-PERF-GATE-002's adversarial cascade (EC-004 in that
story's edge-cases table), now routed and owned by S-PERF-GATE-003. Without this fix,
`just check-ci` runs nextest under the default profile, silently skipping all
`[[profile.ci.overrides]]` serial-group assignments (signal_handlers, adv_p02,
bc_2_01_013_spec_driven_adapter). The comment "identical to CI behavior" was inaccurate.

Traces to: BC-5.39.001 postcondition — `just check-ci` must faithfully replicate CI
nextest behavior, including all `[profile.ci]` serial-group overrides.

### AC-006 — RUSTFLAGS dedup already present in Justfile check recipe (verification-only)

```
grep -c 'RUSTFLAGS="" PROPTEST_CASES' /Users/jmagady/Dev/prism/Justfile
```

Expected output: `2` (one on the nextest step at line 26, one on the doctest step at
line 27, both in the `check` recipe).

Source-verification: Justfile lines 26-27 confirmed:
- `    RUSTFLAGS="" PROPTEST_CASES=100 cargo nextest run --workspace --all-features --profile prepush`
- `    RUSTFLAGS="" PROPTEST_CASES=100 cargo test --workspace --all-features --doc`

The anchor `RUSTFLAGS="" PROPTEST_CASES` matches ONLY the two functional recipe lines.
Justfile line 20 contains a comment `# NOTE: RUSTFLAGS="" is set explicitly...` which
matches bare `RUSTFLAGS=""` but does NOT match `RUSTFLAGS="" PROPTEST_CASES` (the comment
continues with " is set explicitly", not "PROPTEST_CASES"). Using the bare `RUSTFLAGS=""`
pattern returns 3 (comment + 2 recipe lines); the anchored pattern returns exactly 2.
This is the same grep-count-drift class as S-PERF-GATE-002 F-SPG2-RG-MED-001 — a
comment line matching the unanchored pattern inflated the count.

Traces to: BC-5.39.001 postcondition — guards against future regression of the RUSTFLAGS
alignment that prevents fingerprint-cache invalidation between nextest and doctest steps
(diagnosis §3, §7c; ci.yml lines 127-134 pattern).

### AC-007 — sccache stanza present and commented in .cargo/config.toml (verification-only)

```
grep -c 'rustc-wrapper = "sccache"' /Users/jmagady/Dev/prism/.cargo/config.toml
```

Expected output: `1` (the opt-in stanza is present but commented out — the grep matches
the line content regardless of the `#` comment prefix).

Source-verification: `.cargo/config.toml` line 126 confirmed:
`# rustc-wrapper = "sccache"` (inside a commented `[build]` block, lines 125-126).
The grep pattern `rustc-wrapper = "sccache"` matches this line because the `#` and leading
whitespace are not part of the pattern. No other line in .cargo/config.toml contains this
string. Count of 1 is exact.

Traces to: BC-5.39.001 postcondition — sccache opt-in is documented and ready for
developers to enable after `cargo install sccache`, without breaking CI (diagnosis §7d).

### AC-008 — sccache note present in docs/dev-setup.md (verification-only)

```
grep -c 'sccache' /Users/jmagady/Dev/prism/docs/dev-setup.md
```

Expected output: at least `2` (confirmed: line 161 contains the section heading
"### sccache (optional compilation cache)" and line 163 contains multiple occurrences of
"sccache" in the description paragraph — `grep -c` counts lines containing the pattern,
not occurrences per line, so both lines match).

Traces to: BC-5.39.001 postcondition — developer discoverability of the sccache opt-in
path is documented in the canonical dev-setup reference.

### AC-009 — `just check` exits 0 with all changes applied

```
just check
echo "Exit: $?"
```

Expected output: `Exit: 0` (or the just check output ends with a clean exit).

Traces to: BC-5.39.001 postcondition — the config changes must not break the pre-push gate.
A non-zero exit indicates a syntax error in `.config/nextest.toml`, a Justfile parse error,
or a test regression introduced by the changes.

Note: this AC requires all implementation changes to be applied first (AC-001 through AC-005
done). It is a final integration gate, not a per-change prerequisite grep. Run it once at
the end of the implementer's work, before committing.

## Red Gate

Zero Red Gate tests. Rationale: this story makes no changes to production Rust source code.
The files modified are `.config/nextest.toml` (a TOML config file that nextest reads) and
`Justfile` (a Just recipe file). There is no `todo!()` stub to introduce, no failing test
to write first, and no production behavior to change. The validation mechanism is `just check`
exit code (AC-009) and structural grep assertions (AC-001 through AC-008).

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
   - In the `[test-groups]` block (currently after the S-PERF-GATE-001 comment block,
     line 91), add `bc-2-01-013-serial = { max-threads = 1 }` with a comment explaining
     WHY (wiremock startup contention, diagnosis §4.1).
   - After the existing `[[profile.prepush.overrides]]` stanza for `signal_handlers`, add
     a new `[[profile.prepush.overrides]]` stanza:
     ```toml
     filter = 'binary(bc_2_01_013_spec_driven_adapter)'
     test-group = 'bc-2-01-013-serial'
     ```
   - After the existing `[[profile.ci.overrides]]` stanza for `signal_handlers`, add a
     new `[[profile.ci.overrides]]` stanza (same filter + test-group).

3. **Read** `Justfile` to confirm line 60 (check-ci recipe nextest invocation) currently
   reads `cargo nextest run --workspace --all-features --no-fail-fast` with no `--profile`.

4. **Edit** `Justfile`:
   - In the `check-ci` recipe, change the nextest invocation from:
     ```
     cargo nextest run --workspace --all-features --no-fail-fast
     ```
     to:
     ```
     cargo nextest run --workspace --all-features --no-fail-fast --profile ci
     ```
   - Update the recipe comment (line 56) to remove or qualify the "identical to CI
     behavior" claim — it is now correct after the fix.

5. **Verify** AC-001 through AC-008 grep commands return expected values (run each in turn
   before running `just check`).

6. **Run** `just check` to verify AC-009 (exit 0, no syntax error in the edited TOML or
   Justfile).

7. **Confirm** the changes are the only modifications (no production Rust code changes, no
   story-index changes — state-manager handles index registration).

## Token Budget Estimate

| Context component | Estimated tokens |
|-------------------|-----------------|
| This story spec (v1.1, ~350 lines) | ~4,500 |
| `.config/nextest.toml` (full file, ~135 lines — read + modify) | ~1,500 |
| `Justfile` (full file, ~315 lines — read + modify for check-ci fix) | ~3,500 |
| `.cargo/config.toml` (verification only, ~134 lines) | ~1,500 |
| `docs/dev-setup.md` (verification only, scan for sccache) | ~500 |
| Diagnosis research sidecar (reference for rationale) | ~4,000 |
| **Total** | **~15,500** |

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

### From S-PERF-GATE-002 (MERGED in PR #206 — develop HEAD 4417d691 at story finalization)

- S-PERF-GATE-002 merged in PR #206. The worktree for S-PERF-GATE-003 is branched off
  develop after this merge, so there is NO `[[profile.prepush.overrides]]` conflict.
- S-PERF-GATE-002 handled `adv_p02_e2e_pushdown_pipeline_test` via a DIFFERENT mechanism:
  `LazyLock` shared fixtures + internal reset endpoints. DO NOT conflate the two mechanisms.
- For `bc_2_01_013`, the root cause is per-test wiremock startup contention, not shared
  DTU state — serialization alone is sufficient (no reset endpoint needed).
- S-PERF-GATE-002 adds its own `[[profile.prepush.overrides]]` entry for `adv_p02_e2e_pushdown_pipeline_test`.
  This story adds a SEPARATE entry for `bc_2_01_013_spec_driven_adapter`. They coexist in
  the same `[[profile.prepush.overrides]]` array without conflict.
- The `[test-groups]` block in S-PERF-GATE-002 adds a group for `adv_p02`. This story
  adds a SEPARATE group for `bc_2_01_013_spec_driven_adapter`. They coexist in the same
  `[test-groups]` block without conflict.
- **OBS-1 / EC-004 (check-ci --profile ci) routed from S-PERF-GATE-002 cascade**: the
  adversarial pass on S-PERF-GATE-002 identified that `just check-ci` lacked `--profile ci`.
  That finding is now owned and closed by S-PERF-GATE-003 (AC-005 in this story). It was
  deliberately deferred to S-PERF-GATE-003 rather than fixed in S-PERF-GATE-002 because
  the `--profile ci` fix only becomes meaningful once the `[profile.ci.overrides]` entries
  from both stories are present; fixing it mid-S-PERF-GATE-002 would have made the check-ci
  change depend on an incomplete override set.
- **Nextest process-per-test lesson**: nextest spawns one OS process per test binary
  (not per test). The `max-threads = 1` config limits the thread count WITHIN a binary's
  test runner process. This means all tests in the binary run serially, which is exactly
  what we want here (no concurrent wiremock servers within the binary).
- **AC grep-command drift lesson from S-PERF-GATE-002 cascade**: every AC grep was
  source-verified against the actual files before this story was promoted to `ready`. See
  the source-verification notes inline in each AC above.

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
| `.config/nextest.toml` | Modify | Add `bc-2-01-013-serial` group + 2 override stanzas (AC-001 through AC-004) |
| `Justfile` | Modify | Add `--profile ci` to check-ci nextest invocation (AC-005) |
| `.cargo/config.toml` | No change | Verify sccache stanza present (AC-007, read-only assertion) |
| `docs/dev-setup.md` | No change | Verify sccache note present (AC-008, read-only assertion) |

**Files explicitly excluded from this story:**

- `crates/prism-bin/tests/bc_2_01_013_spec_driven_adapter.rs` — no code changes
- `crates/prism-bin/tests/adv_p02_e2e_pushdown_pipeline_test.rs` — S-PERF-GATE-002 scope
- `crates/prism-bin/src/spec_driven_adapter.rs` — no code changes
- Any `.factory/` file — state-manager handles index registration post-delivery

## Scheduling Note

**S-PERF-GATE-002 ALREADY MERGED (PR #206). Hard dependency satisfied.**

S-PERF-GATE-002 merged in PR #206. The `.worktrees/S-PERF-GATE-003/` worktree was created
from develop HEAD 4417d691 (post-merge). No merge conflict risk on `.config/nextest.toml`.

Correct branching order was:

```
develop (after S-PERF-GATE-002 merge — 4417d691)
  └── feature/S-PERF-GATE-003   ← already branched correctly
        ├── Edit .config/nextest.toml (bc-2-01-013-serial group)
        └── Edit Justfile (check-ci --profile ci)
```

The implementer can proceed immediately without waiting for any upstream story.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `.config/nextest.toml` syntax error (e.g., `[test-groups.bc-2-01-013-serial]` instead of `[test-groups]` inline value) | `cargo nextest run` exits non-zero with TOML parse error; AC-009 catches this |
| EC-002 | Binary name typo (e.g., `bc-2-01-013-spec-driven-adapter` with hyphens instead of underscores) | nextest silently applies no override (no match); the test binary still runs under default parallelism. AC-002 grep catches the typo because the `filter = 'binary(bc_2_01_013_spec_driven_adapter)'` pattern would not match a hyphenated variant. |
| EC-003 | S-PERF-GATE-002 NOT yet merged when implementer branches | Merge conflict on `[[profile.prepush.overrides]]` when PR is created. Scheduling Note in this story prevents this; orchestrator must enforce the dependency. S-PERF-GATE-002 merged in PR #206. |
| EC-004 | Justfile `check-ci` recipe flag order (--profile ci placed before vs after --no-fail-fast) | nextest accepts flags in any order; flag placement does not affect behavior. Either `--no-fail-fast --profile ci` or `--profile ci --no-fail-fast` is correct. AC-005 grep uses `.*` between command and `--profile ci` so it matches either order. |
| EC-005 | Developer uncomments `rustc-wrapper = "sccache"` without sccache installed | Cargo invocations fail with "error: failed to run custom build command ... sccache: No such file or directory". Not a concern for this story (sccache stays commented); documented in `.cargo/config.toml` inline comment. |

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.2 (merged) | 2026-06-29 | state-manager | PR #207 squash-merged to develop 1f491590 (D-1420). Status ready→merged. POL-14 NO-OP: BC-5.39.001 already ACTIVE. Worktree + branch removed. |
| 1.2 | 2026-06-28 | story-writer | AC-006 grep command anchored to `RUSTFLAGS="" PROPTEST_CASES` to exclude Justfile line-20 comment (bare `RUSTFLAGS=""` returns 3, not 2; same grep-count-drift class as S-PERF-GATE-002 F-SPG2-RG-MED-001); all ACs re-verified against worktree HEAD 718252f4 — no other drift found |
| 1.1 | 2026-06-28 | story-writer | Promoted draft→ready; added check-ci --profile ci fix as explicit AC-005 (OBS-1/EC-004 routed from S-PERF-GATE-002); tightened all AC grep commands with source-verification against actual files; renumbered ACs (old AC-005→AC-006, AC-006→AC-007, AC-007→AC-008, AC-008→AC-009); updated title, Tasks, File Structure Requirements, Edge Cases, Token Budget, Previous Story Intelligence (noted S-PERF-GATE-002 MERGED PR #206), and Scheduling Note (dependency satisfied) |
| 1.0 | 2026-06-28 | story-writer | Initial draft |
