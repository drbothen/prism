---
document_type: story
story_id: S-PERF-GATE-007
title: "nextest cap groups for uncapped WASMtime + HTTP binaries — spec-engine-wasm-cap and spec-engine-http-cap (max-threads=4) to eliminate Cranelift JIT + wiremock oversubscription and close bc_2_11_007_pushdown_test DTU-cap gap (~150-200s savings)"
epic_id: EPIC-MAINTENANCE
version: "1.4"
status: ready
producer: story-writer
phase: 3
wave: maintenance
priority: P2
points: 2
tdd_mode: "n/a"
# tdd_mode rationale: pure config story — no production Rust code added or modified.
# No function bodies, no Red Gate tests. The only changes are two new [test-groups] entries
# and four new [[profile.*.overrides]] stanzas in .config/nextest.toml. Validated by
# running `just check` and observing elimination of WASMtime Engine-init and wiremock
# socket oversubscription. Mutation testing (facade-mode quality gate) does not apply
# to TOML config files.
target_module: "n/a — build tooling only (.config/nextest.toml)"
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
estimated_days: "0.25"
---

# S-PERF-GATE-007: nextest Cap Groups for Uncapped WASMtime + HTTP Binaries

Add `spec-engine-wasm-cap` (max-threads=4) and `spec-engine-http-cap` (max-threads=4) to
`.config/nextest.toml` prepush + ci profiles, covering 7 WASMtime-heavy binaries and 4
HTTP/wiremock-heavy binaries; also close the `bc_2_11_007_pushdown_test` DTU-cap filter gap

## Narrative

As a Prism developer, I want all WASMtime-heavy (Cranelift JIT) and HTTP/wiremock-heavy
test binaries to run under a shared 4-thread cap in the pre-push gate and on CI, so that
the ~70-90 concurrent `Engine::new()` Cranelift JIT initializations and concurrent wiremock
socket startups do not oversubscribe the CPU — reducing per-test WASMtime cost from ~8-9s
(under load) toward ~3-4s (under the cap), and also closing the filter gap where
`bc_2_11_007_pushdown_test` starts in-process DTU clones but is in `prism-spec-engine`
(not `prism-dtu-*`), so it currently escapes the `dtu-cap` constraint entirely.

## §Evidence

From profiling report `.factory/research/test-suite-perf-profile-2026-06-30.md`
(baseline: develop@8bc0404e, dtu-cap=4 already active post-S-PERF-GATE-004/005,
16 logical CPUs, warm build):

### WASMtime Engine Init Cost Quantification

| Condition | PluginRuntime::new() cost |
|-----------|--------------------------|
| Isolated (single binary) | ~1-2s (estimated Cranelift JIT init) |
| Under full workspace concurrency | ~8-9s (measured: `test_BC_2_17_004_ac3_infinite_loop` = 14.6s = ~9s init + 5s timeout) |
| Oversubscription inflation factor | ~5-9x |

**Root cause:** wasmtime v44 with `wasm_component_model(true)` + `epoch_interruption(true)`
triggers Cranelift JIT compiler initialization. Under CPU contention (16 logical CPUs, ~10-16
concurrent test binaries each spawning multi-threaded tokio runtimes), Cranelift initialization
serializes on global compiler state, causing each `Engine::new()` to wait for others.

### Top Uncapped Binaries by Serial Time

| Binary | Load Type | Avg/Test | Serial Sum | Recommended Cap |
|--------|-----------|---------|-----------|-----------------|
| `prism-spec-engine::plugin_integration_tests` | WASMtime (Cranelift JIT) | 8.16s | 277.4s | `spec-engine-wasm-cap` |
| `prism-spec-engine::pipeline_http_integration` | wiremock HTTP | 9.15s | 247.0s | `spec-engine-http-cap` |
| `prism-spec-engine::plugin_tests` | WASMtime (Cranelift JIT) | 8.19s | 204.7s | `spec-engine-wasm-cap` |
| `prism-bin::plugin_boot_tests` | WASMtime (Cranelift JIT) | 6.94s | 159.5s | `spec-engine-wasm-cap` |
| `prism-spec-engine::enrichment_pivot_002_tests` | WASMtime | 2.75s | 112.9s | `spec-engine-wasm-cap` |
| `prism-spec-engine::crowdstrike_oauth2_plugin_tests` | WASMtime | 5.46s | 103.8s | `spec-engine-wasm-cap` |
| `prism-ocsf::spec_driven_mapper_fixtures` | WASMtime | 7.27s | 94.6s | `spec-engine-wasm-cap` |
| `prism-spec-engine::bc_2_11_007_pushdown_test` | DTU clones (uncapped!) | 7.80s | 85.8s | `spec-engine-http-cap` |
| `prism-spec-engine::pipeline_oauth_retry` | wiremock HTTP | 9.73s | 58.4s | `spec-engine-http-cap` |
| `prism-bin::infusion_boot_integration` | WASMtime | 9.68s | 48.4s | `spec-engine-wasm-cap` |

**Total WASMtime serial time (8 uncapped WASMtime binaries per report §3b — includes
`infusion_tests`, which this story intentionally does NOT cap per REC-1; does NOT include
`bc_2_16_002_crowdstrike_two_step`, which is an HTTP/wiremock binary assigned to
`spec-engine-http-cap`):** approximately 1022.7s. All run under default nextest concurrency
(= 16 on dev machine), competing with the dtu-cap=4 DTU binaries and each other.

### DTU-cap Gap — bc_2_11_007_pushdown_test

`prism-spec-engine::bc_2_11_007_pushdown_test` (11 tests, avg 7.80s = 85.8s serial) starts
in-process CrowdStrike and Armis DTU Axum HTTP servers + tokio runtimes. These are
structurally equivalent to the DTU binaries capped by `dtu-cap`. However, the `dtu-cap`
filter is `package(/^prism-dtu-/)` — it matches by Cargo package name. This test binary
lives in the `prism-spec-engine` package, NOT in `prism-dtu-*`. It runs UNCAPPED, adding
DTU server load on top of the existing `dtu-cap=4` budget.

**Estimated savings for this story (REC-1 + REC-4 from profiling report):**
With max 4 concurrent WASMtime inits (vs uncontrolled ~10-16), per-init time drops from
~8-9s to ~3-4s. For `plugin_tests` (25 tests × avg −5s reduction): −125s serial.
For `plugin_integration_tests` (34 tests × avg −4s): −136s serial. Wall-clock improvement
depends on critical path, but ~150-200s wall-clock is achievable. Closing the
`bc_2_11_007_pushdown_test` gap adds ~40-60s more.

**REC-4 implementation variant — deviation from report code sample:** The profiling
report's REC-4 code sample prescribes folding `bc_2_11_007_pushdown_test` INTO the
existing `dtu-cap` group (sharing dtu-cap's 4-slot budget alongside the ~194
`package(/^prism-dtu-/)` binaries). The delivered implementation instead assigns
`bc_2_11_007_pushdown_test` to the new dedicated `spec-engine-http-cap` pool (runs
concurrently with `dtu-cap`, not inside it). This is an intentional authorial choice:
a dedicated pool preserves `bc_2_11_007`'s own throughput rather than contending with
the ~194 DTU binaries already filling the `dtu-cap` budget. Additionally, the wasm-cap
and http-cap groups simultaneously remove 11 previously-uncapped heavy binaries from
free-running concurrency, so total contention pressure is materially lower than baseline
regardless of which pool captures `bc_2_11_007`. Empirically validated: zero TMT tests,
GREEN `just check`. (Source-of-truth precedence: story spec supersedes the research
report code sample per CLAUDE.md §Source-of-Truth Precedence — story overrides research
for implementation-scope decisions.)

### PR Evidence Framing Note

The measured performance headline (wall-clock improvement vs baseline, reported as
approximately 5.4x / 585.84s → 108.4s in the PR evidence bundle) reflects BOTH the
scheduling caps introduced by this story AND the elimination of timing-out (TMT) tests
that previously consumed large wall-clock before termination. These two effects compound
in the measured wall-clock time.

**Measurement provenance (required for PR evidence bundle):**

(a) **585.84s baseline is a GREEN run — no TMT.** The profiling report
(`.factory/research/test-suite-perf-profile-2026-06-30.md`, baseline develop@8bc0404e) records
this run as GREEN: 4976 passed, 60 skipped, zero timing-out tests. The 585.84s figure does
NOT include any 180s timeout contributions.

(b) **"28 TMT → 0" comes from a separate, heavier-contention run.** The 28 timing-out
tests were observed during S-PERF-GATE-007 delivery (the PR #208 resume sequence), in a
full-workspace `just check` run taken BEFORE the cap fix was applied. Under that
heavier-contention condition (more concurrent processes competing for the same 16 CPUs),
the uncapped WASMtime binaries caused 28 tests to hit the 180s nextest hard timeout. The
PR evidence bundle must record the specific run (timestamp, machine, exact command) where
the 28-TMT baseline was observed so the headline is substantiated rather than asserted.

(c) **108.4s post-cap is measured on the same machine after the cap fix.** The before/after
pair (`time cargo nextest run --workspace --all-features --profile prepush`) must be taken
on the same machine under comparable load conditions; the PR evidence bundle should note
both runs side-by-side.

(d) **The 5.4x headline compounds two distinct effects.** The profiling report's REC-1 +
REC-4 predicted ~190-260s of scheduling savings from the WASMtime/HTTP cap groups. The
~477s of observed wall-clock improvement (585.84s − 108.4s) substantially exceeds that
prediction. The gap (~220-290s) is the TMT-elimination contribution: 28 tests × up to
180s timeout each, partially reclaimed by the cap fix preventing the oversubscription
that caused them. A future re-baseline that starts from a cap-already-applied state (no
TMT tests at baseline) would show only the scheduling-improvement component — the
standalone value of the cap groups for future tuning decisions is the ~190-260s figure,
not the full 5.4x headline.

The headline should NOT be cited as a pure-scheduling effect without this provenance context.

## Background

### Relation to Existing PERF-GATE Cap Groups

| Story | Group | Filter type | Max-threads | Rationale |
|-------|-------|-------------|-------------|-----------|
| S-PERF-GATE-001 | `serial-subprocess` | `binary(signal_handlers)` | 1 | RocksDB mmap SIGSEGV under parallel subprocess spawn |
| S-PERF-GATE-002 | `adv-p02-serial` | `binary(adv_p02_e2e_pushdown_pipeline_test)` | 1 | DTU + DataFusion per-process re-init |
| S-PERF-GATE-003 | `bc-2-01-013-serial` | `binary(bc_2_01_013_spec_driven_adapter)` | 1 | Concurrent wiremock startup socket contention |
| S-PERF-GATE-004 | `dtu-cap` | `package(/^prism-dtu-/)` | 4 | 12 DTU packages (194 test binaries) oversubscription |
| **S-PERF-GATE-007** | **`spec-engine-wasm-cap`** | `binary(...)` (7 WASMtime binaries) | **4** | Cranelift JIT init contention |
| **S-PERF-GATE-007** | **`spec-engine-http-cap`** | `binary(...)` (4 HTTP binaries, incl. bc_2_11_007) | **4** | Wiremock socket contention + DTU-cap gap |

### No overlap with existing groups

The existing override entries from S-PERF-GATE-001/002/003 filter by `binary(signal_handlers)`,
`binary(adv_p02_e2e_pushdown_pipeline_test)`, and `binary(bc_2_01_013_spec_driven_adapter)`.
These are all in `prism-bin`. The S-PERF-GATE-004 `dtu-cap` filter uses
`package(/^prism-dtu-/)`. The new wasm-cap and http-cap filters use `binary(...)` expressions
targeting specific test binary NAMES in `prism-spec-engine`, `prism-ocsf`, and `prism-bin`.

**nextest last-match semantics:** When multiple overrides match the same test, the LAST
matching override wins (nextest scalar `test-group` assignment). In practice, no existing
filter overlaps with the new wasm-cap or http-cap binary names — `signal_handlers`,
`adv_p02_e2e_pushdown_pipeline_test`, and `bc_2_01_013_spec_driven_adapter` are distinct
binary names from all the WASMtime and HTTP filter entries. The `package(/^prism-dtu-/)`
filter for `dtu-cap` uses package-level matching and does NOT match any binary in
`prism-spec-engine`, `prism-ocsf`, or `prism-bin` — confirmed: `bc_2_11_007_pushdown_test`
is in `prism-spec-engine`, which does not match the `^prism-dtu-` prefix.

### Why max-threads = 4

- `max-threads = 4` is the same value chosen for `dtu-cap` in S-PERF-GATE-004 and proven
  effective by the S-PERF-GATE-004 measurement (DTU cap=4: 86.4s nextest; cap=8: 91.5s +
  1 flake; cap=16: 97.5s — S-PERF-GATE-005 §Evidence).
- On a 16-core dev machine: 4 concurrent WASMtime binaries × ~16 tokio threads each = 64
  threads / 16 cores = 4x oversubscription — manageable; Cranelift JIT contention drops
  dramatically vs 10-16 concurrent binaries.
- On CI (ubuntu-latest, 2-4 vCPU): 4 concurrent WASMtime binaries × 2-4 tokio threads =
  8-16 threads / 2-4 cores = 4x — acceptable; previously all uncapped binaries ran at
  whatever concurrency the runner allowed anyway.
- The WASMtime and HTTP cap groups are SEPARATE from `dtu-cap` — they do not share a pool.
  Total concurrent test slots under all caps = 4 (dtu-cap) + 4 (wasm-cap) + 4 (http-cap)
  = 12 constrained slots out of the global thread budget. Unconstrained binaries continue
  to run in parallel as before.

## Scope

Config only. Single file modified:

| File | Change | Rationale |
|------|--------|-----------|
| `.config/nextest.toml` | Add `spec-engine-wasm-cap = { max-threads = 4 }` and `spec-engine-http-cap = { max-threads = 4 }` to `[test-groups]`; add two `[[profile.prepush.overrides]]` stanzas; add two `[[profile.ci.overrides]]` stanzas | Caps 7 WASMtime binaries and 4 HTTP/wiremock binaries at 4 concurrent nextest threads each; closes bc_2_11_007_pushdown_test DTU-cap gap |

**NOT in scope:**

- Any production Rust code changes
- Justfile changes (no recipe needs updating)
- Changes to existing S-PERF-GATE-001/002/003/004 serial groups
- `prism-spec-engine::infusion_tests` — NOT in the uncapped-binaries list (most of its
  54 tests are non-WASMtime; only 3 active Engine inits; not in profiling report REC-1)
- Any `.factory/` file — state-manager handles STORY-INDEX registration post-delivery

## Acceptance Criteria

### AC-001 — `spec-engine-wasm-cap` group definition present in [test-groups]

```
grep -c 'spec-engine-wasm-cap = { max-threads = 4 }' /Users/jmagady/Dev/prism/.config/nextest.toml
```

Expected output: `1` (exactly the inline group definition line).

Source-verification: before this change, `[test-groups]` ends with
`dtu-cap = { max-threads = 4 }` (S-PERF-GATE-004); `spec-engine-wasm-cap` is absent.
After the change, exactly one definition line is added. The grep is anchored to the full
inline-table string to prevent false matches against comment lines.

Traces to: BC-5.39.001 postcondition — delivery quality; without this definition, the
`[[profile.*.overrides]]` references to `'spec-engine-wasm-cap'` would be invalid (nextest
rejects unknown test-group names at startup).

### AC-002 — `spec-engine-http-cap` group definition present in [test-groups]

```
grep -c 'spec-engine-http-cap = { max-threads = 4 }' /Users/jmagady/Dev/prism/.config/nextest.toml
```

Expected output: `1`.

Source-verification: same as AC-001 but for the http-cap group. A count of 0 means the
second group definition was accidentally omitted (wiremock + DTU-gap binaries would remain
uncapped).

Traces to: BC-5.39.001 postcondition — delivery quality; the HTTP/wiremock oversubscription
fix requires this group to exist.

### AC-003 — wasm-cap test-group assigned in BOTH prepush and ci profiles (count = 2)

```
grep -c "test-group = 'spec-engine-wasm-cap'" /Users/jmagady/Dev/prism/.config/nextest.toml
```

Expected output: `2` (one `[[profile.prepush.overrides]]` entry, one
`[[profile.ci.overrides]]` entry).

Source-verification: `test-group = 'spec-engine-wasm-cap'` is absent before the change.
A count of 1 would indicate only one profile was updated — the other profile (local or CI)
would still have uncapped WASMtime binaries.

Traces to: BC-5.39.001 postcondition — delivery quality; both the pre-push profile (used
by `just check`) and the CI profile must apply the wasm-cap constraint.

### AC-004 — http-cap test-group assigned in BOTH prepush and ci profiles (count = 2)

```
grep -c "test-group = 'spec-engine-http-cap'" /Users/jmagady/Dev/prism/.config/nextest.toml
```

Expected output: `2`.

Source-verification: same rationale as AC-003. A count of 1 indicates incomplete fix.

Traces to: BC-5.39.001 postcondition — delivery quality; wiremock + DTU-gap binaries must
be capped on both profiles.

### AC-005 — `bc_2_11_007_pushdown_test` included in http-cap filter on BOTH profiles (DTU-cap gap closed)

```
grep -c 'bc_2_11_007_pushdown_test' /Users/jmagady/Dev/prism/.config/nextest.toml
```

Expected output: `2` (one occurrence in the prepush http-cap filter, one in the ci
http-cap filter).

Source-verification: before this change, `bc_2_11_007_pushdown_test` does not appear in
`.config/nextest.toml`. A count of 1 means only one profile includes the gap fix. A count
of 0 means the critical gap was not closed.

This AC specifically validates the DTU-cap gap closure: `bc_2_11_007_pushdown_test` starts
in-process CrowdStrike and Armis DTU servers but lives in `prism-spec-engine` (not
`prism-dtu-*`), so it escaped the `dtu-cap = 'package(/^prism-dtu-/)'` filter. Assigning
it to `spec-engine-http-cap` closes the gap.

Traces to: BC-5.39.001 postcondition — delivery quality; the gap closure is load-bearing.
Without it, 11 tests adding DTU server load remain uncapped, undermining the dtu-cap=4
concurrency budget.

### AC-006 — `plugin_integration_tests` in wasm-cap filter (representative WASMtime coverage check)

```
grep -c 'plugin_integration_tests' /Users/jmagady/Dev/prism/.config/nextest.toml
```

Expected output: `2` (one in prepush wasm-cap override, one in ci wasm-cap override).

Source-verification: `plugin_integration_tests` is the highest-serial-time WASMtime binary
(277.4s serial, avg 8.16s/test). Its presence in the filter confirms the wasm-cap filter
expression was correctly written. A count of 0 indicates the filter was empty or mis-typed.

Traces to: BC-5.39.001 postcondition — delivery quality; the highest-impact binary must
be in the wasm-cap group.

### AC-007 — existing S-PERF-GATE-001/002/003/004 groups unchanged

```
grep -c 'serial-subprocess = { max-threads = 1 }' /Users/jmagady/Dev/prism/.config/nextest.toml
grep -c 'adv-p02-serial = { max-threads = 1 }' /Users/jmagady/Dev/prism/.config/nextest.toml
grep -c 'bc-2-01-013-serial = { max-threads = 1 }' /Users/jmagady/Dev/prism/.config/nextest.toml
grep -c 'dtu-cap = { max-threads = 4 }' /Users/jmagady/Dev/prism/.config/nextest.toml
```

Expected output: `1` for each (all four groups present with their original `max-threads`
definitions unchanged).

Source-verification: each of these lines is present after S-PERF-GATE-004 merged
(PR #209 develop@e3148007). The implementer must not modify or remove these entries. The
count of `1` each confirms the definitions are intact. A count of 0 for any group indicates
accidental removal.

Traces to: BC-5.39.001 postcondition — no regression in existing SIGTERM flake protection
(S-PERF-GATE-001), adv_p02 filter-push-down correctness (S-PERF-GATE-002), bc_2_01_013
wiremock startup stability (S-PERF-GATE-003), or DTU HTTP-server cap (S-PERF-GATE-004).

### AC-008 — `just check` exits 0 with all changes applied

```
just check
echo "Exit: $?"
```

Expected output: `Exit: 0`.

A non-zero exit indicates either a TOML syntax error in `.config/nextest.toml` (nextest
rejects the config at startup with a clear error message) or a test regression. If the
exit is non-zero, run `cargo nextest show-config --workspace --profile prepush` to validate
the TOML before re-running `just check`.

Note: this AC requires AC-001 through AC-007 to be applied first. Run it once at the end
of the implementer's work, before committing.

Traces to: BC-5.39.001 postcondition — the config change must not break the pre-push gate.

### AC-009 — Binary-name resolution: all 11 filtered binaries appear in their groups with non-empty test lists (mistyped-filter detection)

```
cargo nextest show-config test-groups --profile prepush
```

Expected output (verified on implementation run): the command resolves each test-group's
filter against the compiled test binary inventory. All 11 capped binaries must appear under
their assigned group with non-empty test lists:

**`spec-engine-wasm-cap` (7 binaries):**
- `plugin_integration_tests` (prism-spec-engine)
- `plugin_tests` (prism-spec-engine)
- `crowdstrike_oauth2_plugin_tests` (prism-spec-engine)
- `enrichment_pivot_002_tests` (prism-spec-engine)
- `spec_driven_mapper_fixtures` (prism-ocsf)
- `plugin_boot_tests` (prism-bin)
- `infusion_boot_integration` (prism-bin)

**`spec-engine-http-cap` (4 binaries):**
- `pipeline_http_integration` (prism-spec-engine)
- `pipeline_oauth_retry` (prism-spec-engine)
- `bc_2_11_007_pushdown_test` (prism-spec-engine)
- `bc_2_16_002_crowdstrike_two_step` (prism-spec-engine)

A binary name that appears in a `filter = 'binary(...)'` expression but does NOT resolve
in `show-config` output (or resolves to an empty test list) indicates a mistyped binary
name. nextest silently no-ops a zero-match `binary()` filter, so a mistyped name leaves
`just check` GREEN and all grep-count ACs (AC-001 through AC-007) passing while the cap
constraint binds to nothing. This AC detects that class of false-green.

Output of this command must be captured in the PR evidence bundle alongside the wall-clock
timing improvement.

Traces to: BC-5.39.001 postcondition — delivery quality; the cap groups are only effective
if their binary-name filters resolve to actual compiled binaries with tests to constrain.

## Red Gate

Zero Red Gate tests. This story makes no changes to production Rust source code. The only
file modified is `.config/nextest.toml` (a TOML config file that nextest reads). There is
no `todo!()` stub to introduce and no failing test to write first. Validation is via
`just check` exit code (AC-008) and structural grep assertions (AC-001 through AC-007).

This is consistent with S-PERF-GATE-001 through S-PERF-GATE-004 (all zero Red Gate tests
for config-only portions).

## Behavioral Contracts

| BC | Title | Role in this story |
|----|-------|--------------------|
| BC-5.39.001 | 3-CLEAN convergence protocol | Delivery-quality gate — this story's own PR must pass 3-CLEAN before merge |

This story has no product behavioral contracts. The nextest test-group configuration changes
have no observable effect on test SEMANTICS — only on test scheduling POLICY. Tests that
pass under `max-threads = 4` pass under any threading configuration (and vice versa for
correct tests). The change affects wall-clock and CPU contention, not test correctness.

## Tasks

1. **Read** `.config/nextest.toml` to confirm the current `[test-groups]` block ends with
   `dtu-cap = { max-threads = 4 }` and contains no `spec-engine-wasm-cap` or
   `spec-engine-http-cap` entries. Confirm that the last `[[profile.prepush.overrides]]`
   entry is for `dtu-cap` and the last `[[profile.ci.overrides]]` entry is likewise.

2. **Edit** `.config/nextest.toml` — make the following additions:

   a. In the `[test-groups]` block, after `dtu-cap = { max-threads = 4 }`, add:
   ```toml
   # S-PERF-GATE-007: WASMtime-heavy spec-engine + ocsf + prism-bin binaries.
   # wasmtime v44 + wasm_component_model + epoch_interruption triggers Cranelift JIT
   # on every PluginRuntime::new() call. Under default concurrency (~10-16 concurrent
   # binaries), Cranelift serializes on global compiler state → per-init cost inflates
   # from ~1-2s (isolated) to ~8-9s (under load), ~5-9x oversubscription.
   # max-threads = 4 limits concurrent WASMtime binaries to 4, reducing Cranelift
   # contention back toward isolation cost.
   spec-engine-wasm-cap = { max-threads = 4 }
   # S-PERF-GATE-007: HTTP/wiremock-heavy spec-engine binaries + bc_2_11_007 gap.
   # pipeline_http_integration, pipeline_oauth_retry, bc_2_16_002_crowdstrike_two_step
   # spin up wiremock::MockServer per test (OS socket contention at high concurrency).
   # The bc_2_11_007 pushdown test starts in-process CrowdStrike + Armis DTU clones but
   # lives in prism-spec-engine — NOT matched by dtu-cap package(/^prism-dtu-/) filter.
   # This group closes that gap and caps the wiremock binaries simultaneously.
   spec-engine-http-cap = { max-threads = 4 }
   ```

   **AC-005 guard:** The comments above use "the bc_2_11_007 pushdown test" (with spaces),
   matching the delivered nextest.toml. Do NOT reword any comment in this block to use the
   underscore-form binary name as a bare token — AC-005 counts exactly 2 occurrences of that
   token in nextest.toml (one per profile in the filter-expression lines in step 2b/2c).
   A third occurrence in a comment would cause AC-005 to return 3 and FAIL.

   b. After the existing `[[profile.prepush.overrides]]` stanzas, add:
   ```toml
   [[profile.prepush.overrides]]
   # S-PERF-GATE-007: cap WASMtime-heavy binaries at 4 concurrent threads.
   filter = 'binary(plugin_integration_tests) | binary(plugin_tests) | binary(crowdstrike_oauth2_plugin_tests) | binary(enrichment_pivot_002_tests) | binary(spec_driven_mapper_fixtures) | binary(plugin_boot_tests) | binary(infusion_boot_integration)'
   test-group = 'spec-engine-wasm-cap'

   [[profile.prepush.overrides]]
   # S-PERF-GATE-007: cap HTTP/wiremock-heavy binaries + close bc_2_11_007 DTU-cap gap.
   filter = 'binary(pipeline_http_integration) | binary(pipeline_oauth_retry) | binary(bc_2_11_007_pushdown_test) | binary(bc_2_16_002_crowdstrike_two_step)'
   test-group = 'spec-engine-http-cap'
   ```

   c. After the existing `[[profile.ci.overrides]]` stanzas, add the SAME two overrides
      with `[[profile.ci.overrides]]` headers (identical filter and test-group values,
      only the header differs from `prepush` → `ci`).

3. **Verify** AC-001 through AC-007 grep commands each return their expected values. Run
   each in turn before running `just check`.

4. **Run** `cargo nextest show-config --workspace --profile prepush 2>&1 | head -20` to
   confirm nextest accepts the new TOML without a config-parse error.

5. **Run** `time cargo nextest run --workspace --all-features --profile prepush` to verify
   the full workspace run completes GREEN (exit 0) and record the wall-clock for the PR
   description (expected: < current 585.84s baseline).

6. **Run** `just check` to verify AC-008 (exit 0, no syntax error).

7. **Confirm** the ONLY modified file is `.config/nextest.toml` (no production Rust code
   changes, no Justfile changes, no story-index changes — state-manager handles index
   registration).

## Token Budget Estimate

| Context component | Estimated tokens |
|-------------------|-----------------|
| This story spec (v1.4, ~695 lines) | ~8,300 |
| `.config/nextest.toml` (full file, ~213 lines — read + modify) | ~2,500 |
| AC verification grep outputs (8 commands × ~2 lines each) | ~400 |
| `cargo nextest show-config` output (AC-009 resolution check) | ~300 |
| `cargo nextest run` output (workspace run) | ~1,000 |
| **Total** | **~12,500** |

Well within the implementer agent's context window. Similar complexity to S-PERF-GATE-004
(two test-groups + four override stanzas vs one test-group + two override stanzas).

## Previous Story Intelligence

### From S-PERF-GATE-004 (PR #209, develop@e3148007)

- The `[test-groups]` inline table + `[[profile.prepush.overrides]]` / `[[profile.ci.overrides]]`
  pattern is the established template. Copy-adapt: add two group names with max-threads=4,
  add two pairs of override stanzas (one per profile per group).
- `red_gate_tests: 0` for config-only stories — applies here.
- grep-count-drift lesson: every AC grep was source-verified against the actual file before
  promotion to `ready`. Do the same here: verify that `spec-engine-wasm-cap` and
  `spec-engine-http-cap` do NOT appear anywhere in `.config/nextest.toml` before the change
  (count 0), and appear with count 1 (group definition) and count 2 (profile assignments)
  after.

### From S-PERF-GATE-005 (PR #210, develop@8bc0404e)

- S-PERF-GATE-005 is MERGED. The DTU stop() graceful-shutdown fix is active. The DTU
  per-test cost dropped from ~5s to ~0.02s per `stop()`. The remaining bottleneck is now
  WASMtime Engine init contention (this story).
- The S-PERF-GATE-005 cap-reassessment confirmed dtu-cap=4 is optimal (cap=8: slower + 1
  flake; cap=16: slower). The same logic applies to wasm-cap and http-cap — start with
  max-threads=4.

### nextest `binary(...)` filter semantics

- `binary(name)` matches tests by their compiled TEST BINARY NAME (the filename stem of
  the `.rs` file under `tests/`, e.g., `tests/plugin_integration_tests.rs` → binary name
  `plugin_integration_tests`).
- This is DIFFERENT from `package(/regex/)` which matches by Cargo package name. The 7
  WASMtime binaries and 4 HTTP binaries span three Cargo packages (`prism-spec-engine`,
  `prism-ocsf`, `prism-bin`) — a package regex would over-match (capping all tests in
  those packages, including fast unit tests).
- `binary(name)` is precise: only the specific named test binary is assigned to the group.
  Inline unit tests in `prism-spec-engine/src/` run under the `prism-spec-engine` lib
  binary (named `prism_spec_engine` by Rust conventions, with underscores), NOT any of the
  specific named integration test binaries in the filter.

### Why `infusion_tests` is NOT in the wasm-cap filter

`prism-spec-engine::infusion_tests` (from `tests/infusion_tests.rs`) has 5 helpers using
`PluginRuntime::new()` but 54 total tests, most of which are non-WASMtime-heavy (fast unit-
style tests). It is NOT in the profiling report REC-1 recommended binary list (4b table).
Including it would over-constrain fast tests unnecessarily. Follow the report's REC-1 list
exactly. If future profiling shows `infusion_tests` is a bottleneck, it can be added in a
follow-up.

Note: `prism-bin::infusion_boot_integration` (from `prism-bin/tests/infusion_boot_integration.rs`)
IS in the wasm-cap filter — it has 5 tests all averaging 9.68s due to WASMtime usage.
Do NOT confuse these two different binaries.

### `bc_2_16_002_crowdstrike_two_step` in http-cap filter

`prism-spec-engine::bc_2_16_002_crowdstrike_two_step` is included in the http-cap filter
per the profiling report REC-1 code sample. It is a wiremock-based HTTP integration test
for the two-step CrowdStrike OAuth flow (BC-2.16.002). It is not in the top-25 binary table
in the profiling report (its serial time may be below the top-25 threshold), but it uses
the same wiremock HTTP server pattern as `pipeline_http_integration` and `pipeline_oauth_retry`.
Include it in the filter as specified in REC-1.

## Architecture Compliance Rules

Extracted from architecture sections and ADRs relevant to this story:

1. **ADR-022 (Arc-DI wiring)**: Not applicable — no production Rust code modified.

2. **Single-workspace MSRV (rust-toolchain.toml)**: Not applicable — no Rust code.

3. **TD-VSDD-053 (single-commit-per-burst)**: The implementer must deliver this story's
   changes in a SINGLE commit. No multi-step "Stage 1 / Stage 2" commits.

4. **No `--no-verify` hook bypass**: `just check` must pass normally. Do not bypass hooks.

5. **`.config/nextest.toml` syntax constraint**: nextest overrides use `[[profile.X.overrides]]`
   (double brackets, array of tables). A single-bracket `[profile.X.overrides]` is a TOML
   syntax error. Always use `[[...]]` for override entries.

6. **`binary()` filter vs `package()` filter**: Use `binary(name)` (test binary filename
   stem) for these binaries — NOT `package(name)` (Cargo package name). The binary names
   match the `.rs` filenames under `tests/` without path and extension. Using `package(...)`
   would incorrectly cap ALL tests in `prism-spec-engine`, `prism-ocsf`, and `prism-bin`,
   including fast unit test binaries.

7. **nextest last-match wins for overlapping overrides**: If a binary matches multiple
   `[[profile.X.overrides]]` entries, the LAST match takes effect. The new wasm-cap and
   http-cap entries are added AFTER the existing signal_handlers, adv_p02, bc_2_01_013,
   and dtu-cap entries. No existing filter entry overlaps with the new binary names —
   there is no ordering conflict in practice.

8. **No AI attribution in commits** per project git conventions (CLAUDE.md).

9. **`just check` must exit 0 before the PR is opened.**

10. **`.factory/` not modified by this story**: state-manager handles STORY-INDEX
    registration.

## Library and Framework Requirements

This story does not introduce new library dependencies. The only toolchain requirement is
`cargo-nextest` (already required by `just check`). No version pins change.

The `binary(name)` filter syntax is supported in nextest's filtering DSL. This filter form
has been available in nextest since v0.9.x (well before the workspace's current version,
which already uses this form for signal_handlers, adv_p02, and bc_2_01_013). The `|` OR
operator for combining multiple binary() expressions is also supported.

The `|` operator in nextest filter expressions is evaluated as a logical OR. A test binary
matching ANY of the `binary(...)` terms is assigned to the group. The expression can be
arbitrarily long on one line — nextest parses it correctly.

## File Structure Requirements

| File | Change type | Details |
|------|-------------|---------|
| `.config/nextest.toml` | Modify | Add `spec-engine-wasm-cap = { max-threads = 4 }` and `spec-engine-http-cap = { max-threads = 4 }` to `[test-groups]`; add 2 new `[[profile.prepush.overrides]]` stanzas; add 2 new `[[profile.ci.overrides]]` stanzas |

**Files explicitly excluded from this story:**

- `Justfile` — no change needed (`--profile prepush` already set by S-PERF-GATE-001)
- `.cargo/config.toml` — no change needed
- `docs/dev-setup.md` — no change needed
- Any `crates/**/*.rs` files — no code changes
- Any `.factory/` file — state-manager handles STORY-INDEX registration post-delivery

## Scheduling Note

**S-PERF-GATE-005 ALREADY MERGED (PR #210, develop@8bc0404e). Hard dependency satisfied.**

The implementer must branch `feature/S-PERF-GATE-007` off develop HEAD 8bc0404e (or later).
No merge conflict risk on `.config/nextest.toml` — S-PERF-GATE-005 only modified
`crates/prism-dtu-*/src/` files.

S-PERF-GATE-006 and S-PERF-GATE-007 are INDEPENDENT of each other (different files):
- S-PERF-GATE-006 → `Justfile` only
- S-PERF-GATE-007 → `.config/nextest.toml` only

They may be developed in parallel on separate worktrees or sequentially; no ordering
constraint between them. When both are merged to develop, the combined savings from the
RUSTFLAGS alignment (S-PERF-GATE-006, ~150s) and the WASMtime/HTTP cap groups (this story,
~150-200s) will compound.

```
develop (after S-PERF-GATE-005 merge — 8bc0404e)
  └── feature/S-PERF-GATE-007   ← branch from here
        └── Edit .config/nextest.toml (2 new groups + 4 new override stanzas)
```

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | New `prism-spec-engine` WASMtime test binary added after this story (e.g., `tests/new_plugin_tests.rs`) | The new binary is NOT automatically capped unless its binary name is added to the wasm-cap filter. Per-binary filters require explicit enumeration. A future story or in-scope amendment is needed. The AC-001/AC-003 grep counts remain unchanged; no automatic regression detection. This is a maintenance burden acknowledged in the design. |
| EC-002 | `binary(spec_driven_mapper_fixtures)` matches a hypothetical second binary with the same name in a different package | nextest `binary(name)` matches by binary FILENAME STEM across all packages. If another package adds `tests/spec_driven_mapper_fixtures.rs`, that binary would also be capped under wasm-cap. This is acceptable — any binary with that name would likely be WASMtime-related (it is a WASMtime-specific fixture runner). No known package conflict exists. |
| EC-003 | `bc_2_11_007_pushdown_test` binary is renamed in a future story | The `binary(bc_2_11_007_pushdown_test)` filter would stop matching. The DTU-cap gap would silently re-open. The AC-005 grep count would still return 2 (the filter text is unchanged), but it would have no effect. A future story renaming the binary MUST also update this filter. |
| EC-004 | `max-threads = 4` too restrictive on a 2-core CI runner | nextest respects `min(global_thread_count, max-threads)`. On a 2-core runner, the effective cap is already ≤2, so `max-threads = 4` is a no-op. No worse than current behavior. |
| EC-005 | Two override stanzas for wasm-cap in prepush (one from this story, one from a future story) conflict | nextest last-match-wins semantics: the LAST matching override determines the test-group. As long as both overrides assign to the same group (`spec-engine-wasm-cap`), the result is identical regardless of which one matches. If a future override assigns a wasm-cap binary to a DIFFERENT group, last-match silently overrides this story's intent — this must be caught in adversarial review. |
| EC-006 | `bc_2_11_007_pushdown_test` is also assigned to `dtu-cap` via a new `package()` or `binary()` rule | nextest last-match wins; whichever override appears LAST in the file takes effect. If the dtu-cap override is first and the http-cap override is last, `bc_2_11_007_pushdown_test` gets `spec-engine-http-cap` (correct). The implementer must ensure the new http-cap stanzas are appended AFTER the existing dtu-cap stanzas. |
| EC-007 | TOML inline-table syntax error (e.g., `spec-engine-wasm-cap = {max-threads = 4}` without spaces) | TOML spec permits both `{ max-threads = 4 }` (with spaces) and `{max-threads = 4}` (without). However, existing group entries in this file use spaces (`{ max-threads = 1 }`, `{ max-threads = 4 }`). The AC-001/AC-002 greps are anchored to `{ max-threads = 4 }` with spaces — an omission of spaces would cause these ACs to fail, catching the inconsistency. Always use spaces to match project style. |
| EC-008 | `pipeline_http_integration` or `pipeline_oauth_retry` already in `bc-2-01-013-serial` group | `bc-2-01-013-serial` filter is `binary(bc_2_01_013_spec_driven_adapter)`. None of the new http-cap filter binaries appear in any existing override. Confirmed: no overlap. |

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.4 | 2026-07-01 | story-writer | F-LOW-1 fix: §Tasks 2a comment sample for spec-engine-http-cap — replaced bare underscore token `bc_2_11_007_pushdown_test` with "The bc_2_11_007 pushdown test" (matching delivered nextest.toml) and added explicit AC-005 guard note so implementers do not inadvertently introduce a third grep match that would cause AC-005 to return 3 instead of 2. F-OBS-1 fix: §PR Evidence Framing Note — added four-part measurement provenance: (a) 585.84s baseline is GREEN (no TMT per profiling report); (b) 28-TMT figure comes from separate heavier-contention `just check` run pre-cap during PR #208 delivery; (c) 108.4s post-cap on same machine; (d) 5.4x headline compounds ~190-260s scheduling savings with TMT-elimination contribution; headline must not be cited as pure-scheduling effect without this provenance. Token budget updated: v1.4 / ~695 lines / ~12,500 tokens. |
| 1.3 | 2026-07-01 | story-writer | F-LOW-1 fix: §Background "Relation to Existing PERF-GATE Cap Groups" table — corrected S-PERF-GATE-004 rationale from "12 DTU HTTP-server + RocksDB binaries" to "12 DTU packages (194 test binaries)" per nextest.toml authoritative comment line 130 ("ALL 12 prism-dtu-* packages (194 test binaries total)"). OBS-1 fix: status draft→ready — behavioral_contracts: [BC-5.39.001] is non-empty (S-7.01 gate satisfied); story is implemented GREEN and in LOCAL adversarial 3-CLEAN cascade; S-PERF-GATE-002 precedent confirms status=ready is maintained through LOCAL cascade; state-manager flips to merged at PR merge per POL-14. Token budget updated: v1.3 / ~665 lines / ~12,300 tokens. |
| 1.2 | 2026-07-01 | story-writer | OBS-1 fix: §Evidence DTU-cap Gap section — documented REC-4 implementation variant: `bc_2_11_007_pushdown_test` is assigned to the new dedicated `spec-engine-http-cap` pool rather than folded into `dtu-cap` as REC-4's code sample shows; added one-line rationale (preserves `bc_2_11_007` throughput vs ~194 dtu-cap binaries; wasm-cap+http-cap groups remove 11 uncapped heavy binaries simultaneously; zero TMT, GREEN). Added PR Evidence Framing Note: the ~5.4x / ~108.4s headline compounds scheduling-cap savings with TMT-elimination and must not be read as a pure-scheduling effect on re-baseline. Updated token budget to v1.2 / ~660 lines / ~12,100 tokens. |
| 1.1 | 2026-07-01 | story-writer | F-1 fix: corrected §Evidence "Total WASMtime serial time" sentence — removed erroneous `bc_2_16_002_crowdstrike_two_step` (HTTP/wiremock binary, not WASMtime), clarified the 1022.7s covers report §3b's 8 WASMtime binaries including `infusion_tests` (intentionally uncapped). OBS-2: added AC-009 binary-name resolution check (`cargo nextest show-config test-groups --profile prepush`) to detect zero-match `binary()` filters that would leave `just check` false-green. |
| 1.0 | 2026-06-30 | story-writer | Initial draft (T-PERF-PROFILE initiative, D-1435) |
