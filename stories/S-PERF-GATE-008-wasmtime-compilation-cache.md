---
document_type: story
story_id: S-PERF-GATE-008
title: "wasmtime compilation cache — enable on-disk native-code cache in PluginRuntime with degradable boot semantics (D3), SAP-1 structured event, and nextest spec-engine-wasmtime serialization group (max-threads=1)"
epic_id: EPIC-MAINTENANCE
version: "1.1"
status: draft
producer: story-writer
phase: 3
wave: maintenance
priority: P2
points: 3
tdd_mode: strict
# tdd_mode rationale: this story modifies production Rust code in
# crates/prism-spec-engine/src/plugin/mod.rs — it adds the wasmtime Cache initialization
# with degradable boot semantics (ADR-049 D3) and extracts a testable helper function.
# The unit tests for the degradable path (Red Gate tests) must be written as failing stubs
# BEFORE the implementation code is introduced. The Cargo.toml and nextest.toml changes
# are config-only and have no Red Gate tests of their own, but the story cannot use
# tdd_mode: n/a because production code is modified with new behavioral semantics.
target_module: prism-spec-engine
subsystems: [SS-17]
depends_on: [S-PERF-GATE-007]
blocks: []
behavioral_contracts: [BC-5.39.001, BC-2.16.002]
# BC status:
#   BC-5.39.001 — delivery-quality / 3-CLEAN convergence protocol (already ACTIVE).
#   BC-2.16.002 — multi-step fetch pipeline; v1.92 already includes the
#     plugin.compilation_cache_init_skipped catalog row (SAP-1 D8 obligation discharged by
#     product-owner in the same spec burst). POL-14 will be a NO-OP at merge for BC-5.39.001
#     (already ACTIVE); BC-2.16.002 transitions draft→active at merge per POL-14.
verification_properties: []
assumption_validations: []
risk_mitigations: []
red_gate_tests: 2
estimated_days: "0.5"
---

# S-PERF-GATE-008: wasmtime Compilation Cache

Enable the wasmtime on-disk compilation cache in `PluginRuntime::new_with_audit_sink()`,
implement degradable boot semantics per ADR-049 D3, emit the SAP-1 structured tracing event
on cache-init failure, and add the `spec-engine-wasmtime` nextest serialization group
(max-threads=1) so the 6 wasmtime-heavy spec-engine test binaries run sequentially after
the cache warms — turning 80-150 s per-test cold-compile waits into <1 s cache-hit loads.

## Narrative

As a Prism developer, I want `PluginRuntime::new_with_audit_sink()` to initialize the
wasmtime on-disk compilation cache with degradable semantics (cache-init failure logs a
`WARN` and continues — it does NOT abort boot), so that repeated nextest runs and production
process restarts load compiled `.prx` native code from disk instead of re-running the
Cranelift JIT compiler, reducing per-test wasmtime cost from 80-150 s (cold, parallel) to
<1 s (warm cache hit) and keeping the 6 wasmtime-heavy test binaries within the nextest
180 s hard-kill ceiling under full workspace load.

## §Evidence

Reference branch `wip/perf-wasmtime-exploration-76821af7` (single commit 76821af7, base
0978983f) verified the approach: `just check` passed GREEN, 5 079 tests passed, zero
timing-out tests (TMT). The prototype makes exactly 3 file changes:

| File | Change in prototype |
|------|-------------------|
| `crates/prism-spec-engine/Cargo.toml` | Adds `"cache"` to wasmtime features |
| `crates/prism-spec-engine/src/plugin/mod.rs` | Calls `Cache::new(CacheConfig::new())` + `config.cache(Some(cache))` |
| `.config/nextest.toml` | Adds `spec-engine-wasmtime = { max-threads = 1 }` + overrides for 6 binaries |

**This story delivers those 3 changes plus two required corrections vs. the prototype:**

1. **D3 correction (MANDATORY):** The prototype uses `?` on `Cache::new` — fatal propagation.
   ADR-049 D3 (LOCKED decision) mandates DEGRADABLE semantics: on `Err(e)` emit
   `tracing::warn!(event_type = "plugin.compilation_cache_init_skipped", error = %e, ...)`
   and continue with `config.cache(None)`. Do NOT abort boot on a cache I/O failure.

2. **D9 correction (MANDATORY):** All inline comments in the prototype reference
   `S-PERF-GATE-006`. Delivered working-source comments must cite `S-PERF-GATE-008`.

**Performance context from profiling report `.factory/research/test-suite-perf-profile-2026-06-30.md`
(baseline develop@8bc0404e after S-PERF-GATE-004/005/006/007):**

| Condition | wasmtime Component::new() per-test cost |
|-----------|----------------------------------------|
| Cold start, isolated | ~1-5 s (Cranelift JIT, single process) |
| Cold start, parallel workspace (before this story) | 80-150 s (engine contention) |
| Warm cache hit (after this story) | <1 s (cache load, no Cranelift) |

The nextest `spec-engine-wasmtime = { max-threads = 1 }` group (D5) ensures that during
the cold cache warm-up run, only one wasmtime-heavy binary runs at a time. Once the first
binary has compiled and cached each `.prx` file, all subsequent binaries load from cache
at <1 s per plugin. The `max-threads = 1` serialization is intentionally more aggressive
than the `spec-engine-wasm-cap = { max-threads = 4 }` group from S-PERF-GATE-007 — with
cache hits, there is no CPU contention to manage; serialization maximizes cache hit rate
on the cold-start run.

**Interaction with S-PERF-GATE-007 (nextest last-match semantics):**

S-PERF-GATE-007 added `spec-engine-wasm-cap = { max-threads = 4 }` for 7 binaries. This
story adds `spec-engine-wasmtime = { max-threads = 1 }` for 6 binaries. The 5 binaries
present in BOTH filters (`plugin_tests`, `crowdstrike_oauth2_plugin_tests`,
`enrichment_pivot_002_tests`, `plugin_integration_tests`, `spec_driven_mapper_fixtures`)
will be governed by `spec-engine-wasmtime` (max-threads=1) when the new overrides appear
AFTER the existing `spec-engine-wasm-cap` overrides — nextest last-match-wins semantics.
The 2 binaries only in `spec-engine-wasm-cap` (`plugin_boot_tests`,
`infusion_boot_integration`) remain at max-threads=4. The 1 binary only in
`spec-engine-wasmtime` (`infusion_tests`) gains a max-threads=1 constraint not previously
applied.

## Background

### ADR-049 Decision Chain

| Decision | Requirement for this story |
|----------|---------------------------|
| D1 | Wire `Cache::new(CacheConfig::new())` + `config.cache(Some(cache))` between `epoch_interruption` and `Engine::new` in `new_with_audit_sink` |
| D2 | OS-default cache directory; `CacheConfig::new()` uses built-in defaults; no external config file |
| D3 (LOCKED) | Cache-init failure → DEGRADABLE: emit WARN, call `config.cache(None)`, do NOT return Err. This is the mandatory override of the prototype's `?` pattern |
| D4 | Add `"cache"` to wasmtime features in `Cargo.toml` |
| D5 | `spec-engine-wasmtime = { max-threads = 1 }` in `[test-groups]`; overrides for 6 binaries in BOTH prepush and ci profiles |
| D8 | `plugin.compilation_cache_init_skipped` catalog row in BC-2.16.002 §Postconditions, same commit as the `tracing::warn!` emission — SAP-1 obligation. **BC-2.16.002 v1.92 already has this row** (discharged in the spec burst). The implementer MUST verify the row is present before opening the PR. |
| D9 | All working-source comments (nextest.toml + mod.rs) cite `S-PERF-GATE-008`, not `S-PERF-GATE-006` |

### Degradable-Path Code (ADR-049 D3)

The exact implementation required by ADR-049 D3 (do not deviate):

```rust
// Enable the wasmtime compilation cache (S-PERF-GATE-008).
//
// wasmtime::Component::new() (WASM-to-native Cranelift compilation) caches compiled
// native code to disk, addressed by (wasm_binary_hash, compiler_version, cpu_isa_flags).
// Warm cache hits skip Cranelift entirely, reducing per-plugin load from 80-150s
// (cold parallel) to <1s. Cache directory: OS default (~/.cache/wasmtime/ or
// ~/Library/Caches/wasmtime/). Created automatically on first use.
//
// Cache-init failure is DEGRADABLE (ADR-049 D3): a disk-full, permissions, or
// read-only-filesystem condition must not abort the analyst's session. PluginRuntime
// construction continues without the cache; plugins recompile on each cold start
// (slower but functionally correct).
match wasmtime::Cache::new(wasmtime::CacheConfig::new()) {
    Ok(cache) => {
        config.cache(Some(cache));
    }
    Err(e) => {
        tracing::warn!(
            event_type = "plugin.compilation_cache_init_skipped",
            error = %e,
            "wasmtime compilation cache init failed; proceeding without cache (degraded performance)"
        );
        // config.cache remains None — wasmtime will compile fresh each load
    }
}
```

### Testable Degradable Path (SID-1 requirement)

The degradable `Err` branch must be exercised by a non-`#[ignore]`'d unit test (SID-1).
Because `new_with_audit_sink` calls `Cache::new(CacheConfig::new())` internally, testing
the `Err` path requires extracting the match arm into a testable function. The
implementation must provide a `pub(super)` (or private-to-module) helper:

```rust
/// Attempts to enable the wasmtime compilation cache on `config`.
/// Cache-init failure is DEGRADABLE: emits a WARN and returns without error.
/// S-PERF-GATE-008 / ADR-049 D3.
fn apply_wasmtime_cache(config: &mut wasmtime::Config, cache_result: Result<wasmtime::Cache, wasmtime::Error>) {
    match cache_result {
        Ok(cache) => {
            config.cache(Some(cache));
        }
        Err(e) => {
            tracing::warn!(
                event_type = "plugin.compilation_cache_init_skipped",
                error = %e,
                "wasmtime compilation cache init failed; proceeding without cache (degraded performance)"
            );
        }
    }
}
```

And in `new_with_audit_sink`:

```rust
apply_wasmtime_cache(&mut config, wasmtime::Cache::new(wasmtime::CacheConfig::new()));
```

The unit tests in `#[cfg(test)] mod tests` can then call `apply_wasmtime_cache` directly
with a controlled `Err(...)` without requiring an external `.prx` artifact or live wasmtime
Engine — this is the SID-1 "mock/stub at the dependency boundary" pattern.

### Why max-threads = 1 (not 4)

`spec-engine-wasmtime = { max-threads = 1 }` is more restrictive than the max-threads=4 used
by `spec-engine-wasm-cap` (S-PERF-GATE-007). Rationale:

- With the compilation cache active, the FIRST binary in the serialized queue incurs the
  full Cranelift cold-compile cost (~1-5 s per plugin in isolation). Subsequent binaries
  load the same `.prx` files from cache at <1 s each.
- Serial order (max-threads=1) maximizes the cache hit rate for this warm-up sequence.
  Under max-threads=4, all four concurrent binaries might compile the same `.prx` file
  simultaneously on the first cold run, each writing to the cache — wasted duplicate work.
- The nextest serialization ensures the warm-up produces exactly one Cranelift compile
  per distinct `.prx` file, then all remaining binaries hit the cache.
- If the cache is unavailable (D3 degradable path), max-threads=1 serializes all Cranelift
  work, bounding total WASMtime CPU overhead similarly to max-threads=1 from ADR-049 D5.
  This provides performance protection even without a working cache.

## Scope

Three files modified:

| File | Change | Rationale |
|------|--------|-----------|
| `crates/prism-spec-engine/Cargo.toml` | Add `"cache"` to wasmtime features | Activates `wasmtime::Cache` and `wasmtime::CacheConfig` types (D4). wasmtime 44 is already pinned; no new transitive dependencies |
| `crates/prism-spec-engine/src/plugin/mod.rs` | Extract `apply_wasmtime_cache` helper; call it in `new_with_audit_sink` between `epoch_interruption` and `Engine::new`; add 2 unit tests | D1 (cache wiring) + D3 (degradable path) + D8 (WARN event) + SID-1 (unit tests) |
| `.config/nextest.toml` | Add `spec-engine-wasmtime = { max-threads = 1 }` to `[test-groups]`; add two `[[profile.prepush.overrides]]` and two `[[profile.ci.overrides]]` entries for 6 wasmtime-heavy binaries | D5 (test serialization group) + D9 (comments cite S-PERF-GATE-008) |

**NOT in scope:**

- Any other `crates/**/*.rs` file
- `Justfile` — no recipe change needed
- `crates/prism-spec-engine/Cargo.toml` beyond the `"cache"` feature addition
- `.factory/` files — state-manager handles STORY-INDEX registration and BC lifecycle
- BC-2.16.002 — catalog row v1.92 already added in the spec burst (product-owner deliverable)
- Any new VP (Verification Property) — ADR-049 §Scoping Summary confirms none needed

## Acceptance Criteria

### AC-001 — Cargo `"cache"` feature present in wasmtime dependency (D4)

```
grep -c '"component-model", "cache"' crates/prism-spec-engine/Cargo.toml
```

Expected output: `1`.

Source-verification: before this change, `grep '"cache"' crates/prism-spec-engine/Cargo.toml`
returns 0. After the change, it returns 1. The full grep string anchors to both features
to prevent false matches against any future wasmtime feature addition.

Traces to: BC-2.16.002 postcondition (WASM plugin runtime correctness — cache feature
enables the `Cache` and `CacheConfig` types); BC-5.39.001 postcondition (delivery quality).

### AC-002 — `apply_wasmtime_cache` helper extracted with degradable match arm (D3)

```
grep -c 'apply_wasmtime_cache' crates/prism-spec-engine/src/plugin/mod.rs
```

Expected output: at least `3` (one function definition, one call site in
`new_with_audit_sink`, one or more test call sites).

Additionally, verify the `?` propagation pattern from the prototype is NOT present:

```
grep -c 'Cache::new.*map_err.*cache init failed' crates/prism-spec-engine/src/plugin/mod.rs
```

Expected output: `0` (the prototype's fatal `map_err(|e| PrismError::Internal { ... })?`
pattern must be absent — it was replaced by the degradable match arm).

Traces to: BC-2.16.002 postcondition (degradable boot — cache-init error must not abort
plugin runtime construction); ADR-049 D3 (LOCKED decision); BC-5.39.001 postcondition
(delivery quality; ADR-049 D3 is a locked architectural constraint).

### AC-003 — Degradable path emits `plugin.compilation_cache_init_skipped` WARN with `error` field (D8 / SAP-1)

```
grep -c 'event_type = "plugin.compilation_cache_init_skipped"' crates/prism-spec-engine/src/plugin/mod.rs
```

Expected output: `1` (exactly one emission site in the production `Err` arm; not in a
`#[cfg(test)]` block).

Verify the `error = %e` field is present:

```
grep -A3 'plugin.compilation_cache_init_skipped' crates/prism-spec-engine/src/plugin/mod.rs | grep -c 'error = %e'
```

Expected output: `1`.

Verify the emission is unconditional (not inside `#[cfg(test)]`):

```
grep -B10 'plugin.compilation_cache_init_skipped' crates/prism-spec-engine/src/plugin/mod.rs | grep -c 'cfg(test)'
```

Expected output: `0`.

Traces to: BC-2.16.002 §Postconditions Canonical Structured Event Catalog row
`plugin.compilation_cache_init_skipped` (v1.92); ADR-049 D8; SAP-1
(CLAUDE.md §Standing Adversary Probes — tracing emission catalog completeness).
Per SAP-1, a `event_type = "plugin.compilation_cache_init_skipped"` emission site without
a BC-2.16.002 catalog row is a P1 finding. The implementer MUST verify the v1.92 catalog
row is present in BC-2.16.002 before the PR is opened.

### AC-004 — SID-1 unit test: `PluginRuntime` construction succeeds on cache-init failure (D3)

The `#[cfg(test)] mod tests` block in `plugin/mod.rs` must contain the test
`test_S_PERF_GATE_008_apply_wasmtime_cache_degradable_path_does_not_panic`. This test:
- Calls `apply_wasmtime_cache` directly (without `new_with_audit_sink` — no `.prx` needed)
- Constructs a forced-failure `CacheConfig` using the verified wasmtime 44 mechanism:
  `CacheConfig::new()` + `.with_directory("relative/not/absolute")` — the `is_absolute()` check
  in `validate()` fires before any filesystem I/O, returning `Err` deterministically on all platforms
- Pre-asserts `wasmtime::Cache::new(cfg).is_err()` to verify the forced-failure driver
  (real `wasmtime::Error` from the API, not a synthetic `anyhow::anyhow!` injection)
- Passes the real `wasmtime::Cache::new(cfg)` result to `apply_wasmtime_cache`
- Asserts the function returns normally (no panic, no `Err` propagation — degradable path confirmed)
- Does NOT assert the exact error substring ("has to be absolute") — error text is not a stability
  contract; `is_err()` is sufficient for the forced-failure assertion

```
grep -c 'test_S_PERF_GATE_008_apply_wasmtime_cache_degradable_path_does_not_panic' \
    crates/prism-spec-engine/src/plugin/mod.rs
```

Expected output: `1`.

This test must NOT be `#[ignore]`'d. If any external dependency is required, the test
strategy is wrong — `apply_wasmtime_cache` is a pure function over `(config, cache_result)`
and has no external deps.

Traces to: BC-2.16.002 postcondition (degradable path — `Err` branch must not abort
construction); ADR-049 D3; SID-1 (CLAUDE.md §Standing Implementer Disciplines — no
`#[ignore]`-rationalization prohibition).

### AC-005 — SID-1 unit test: WARN event emitted with correct `event_type` on cache-init failure (D8)

The `#[cfg(test)] mod tests` block must also contain
`test_S_PERF_GATE_008_apply_wasmtime_cache_emits_warn_on_err`. This test:
- Sets up a tracing subscriber (using the same `Arc<Mutex<Vec<u8>>>` buffer pattern
  established in `test_F_LP7_MED_001_host_emit_acquire_token_parse_error_fires_unconditionally`)
- Constructs the forced-failure via `CacheConfig::new().with_directory("relative/not/absolute")`,
  calls `wasmtime::Cache::new(cfg)` to obtain a real `wasmtime::Error`, and passes that result to
  `apply_wasmtime_cache`
- Asserts the captured output contains `plugin.compilation_cache_init_skipped`
- Must NOT be `#[ignore]`'d

```
grep -c 'test_S_PERF_GATE_008_apply_wasmtime_cache_emits_warn_on_err' \
    crates/prism-spec-engine/src/plugin/mod.rs
```

Expected output: `1`.

Traces to: BC-2.16.002 §Postconditions catalog row `plugin.compilation_cache_init_skipped`
(v1.92 — level WARN, field `error = %e`); ADR-049 D8; SAP-1 (load-bearing verification
that the WARN emission is unconditional and carries the correct `event_type` value).

### AC-006 — `spec-engine-wasmtime` group definition present in `[test-groups]` with max-threads = 1 (D5)

```
grep -c 'spec-engine-wasmtime = { max-threads = 1 }' .config/nextest.toml
```

Expected output: `1` (exactly the inline group definition line).

Source-verification: before this change, `spec-engine-wasmtime` is absent from
`.config/nextest.toml`. After the change, exactly one definition line is added. The grep
is anchored to the full inline-table string (including spaces per project style) to prevent
false matches against comments.

Traces to: ADR-049 D5; BC-5.39.001 postcondition (delivery quality; without this
definition, the `[[profile.*.overrides]]` references to `spec-engine-wasmtime` would be
invalid at nextest startup).

### AC-007 — `spec-engine-wasmtime` test-group assigned in BOTH prepush and ci profiles (D5)

```
grep -c "test-group = 'spec-engine-wasmtime'" .config/nextest.toml
```

Expected output: `2` (one `[[profile.prepush.overrides]]` entry, one
`[[profile.ci.overrides]]` entry).

Source-verification: absent before the change (count 0). A count of 1 indicates only one
profile was updated — the other would remain uncapped. Both profiles must apply the
serialization constraint.

Traces to: ADR-049 D5; ADR-014 (Local Pre-Push vs CI Gate Asymmetry — parity principle;
both profiles must be updated); BC-5.39.001 postcondition (delivery quality).

### AC-008 — All 6 expected binaries appear in the `spec-engine-wasmtime` filter on BOTH profiles (D5)

```
grep -c 'infusion_tests' .config/nextest.toml
```

Expected output: `2` (one in prepush filter, one in ci filter — both under
`spec-engine-wasmtime`).

The `infusion_tests` binary is the meaningful addition in this story's 6-binary filter
vs. the S-PERF-GATE-007 wasm-cap 7-binary filter. Its presence confirms the filter
was correctly written. Additionally verify the crowdstrike binary:

```
grep -c "binary(crowdstrike_oauth2_plugin_tests)" .config/nextest.toml
```

Expected output: `4` (2 from S-PERF-GATE-007 `spec-engine-wasm-cap` prepush+ci, 2 from
this story's `spec-engine-wasmtime` prepush+ci — last-match semantics means the binary
ultimately uses `spec-engine-wasmtime`).

Note: `infusion_tests` (`tests/infusion_tests.rs` in `prism-spec-engine`) is DISTINCT from
`infusion_boot_integration` (`tests/infusion_boot_integration.rs` in `prism-bin`). The
former is in the `spec-engine-wasmtime` filter (this story). The latter is in
`spec-engine-wasm-cap` (S-PERF-GATE-007) and is NOT added here.

Traces to: ADR-049 D5; BC-5.39.001 postcondition (delivery quality; all 6 binaries must
appear in the filter for the serialization to take effect).

### AC-009 — Binary-name resolution: all 6 spec-engine-wasmtime binaries resolve in BOTH profiles (D5)

Both profiles must be checked independently:

```
cargo nextest show-config test-groups --profile prepush
cargo nextest show-config test-groups --profile ci
```

Expected output for EACH profile: the `spec-engine-wasmtime` group resolves all 6 binaries
with non-empty test lists:

- `plugin_tests` (prism-spec-engine)
- `crowdstrike_oauth2_plugin_tests` (prism-spec-engine)
- `enrichment_pivot_002_tests` (prism-spec-engine)
- `plugin_integration_tests` (prism-spec-engine)
- `infusion_tests` (prism-spec-engine)
- `spec_driven_mapper_fixtures` (prism-ocsf)

A binary name that appears in the filter but resolves to an empty test list indicates a
mistyped binary name. nextest silently no-ops a zero-match `binary()` filter, so a mistyped
name leaves `just check` GREEN while the serialization constraint is binding to nothing.
Both profiles must be checked — a ci-only mistyped name produces a false-green on the
prepush check (S-PERF-GATE-007 AC-009 lesson, version 1.7).

Output of BOTH commands must be captured in the PR evidence bundle.

Traces to: ADR-049 D5; BC-5.39.001 postcondition (delivery quality; caps are only
effective when binary-name filters resolve to real compiled test binaries).

### AC-010 — D9 relabeling: no working-source `S-PERF-GATE-006` references (D9)

```
grep -rn 'S-PERF-GATE-006' crates/ .config/nextest.toml Justfile
```

Expected output: no hits (or only hits in git-history merge commits not in working source).

The prototype incorrectly labels all comments `S-PERF-GATE-006`. Delivered working-source
comments in `crates/prism-spec-engine/src/plugin/mod.rs` and `.config/nextest.toml` must
cite `S-PERF-GATE-008`.

Allowed exceptions (not flagged as failures):
- `.factory/` files: historical changelog rows, ADR cross-references, and BC amendment
  rows that cite `S-PERF-GATE-006` are immutable historical records (TD-VSDD-091).
- Merged PR commit messages and git history: immutable.
- The STORY-INDEX entry for S-PERF-GATE-006 itself.

Traces to: ADR-049 D9; TD-VSDD-060 (sibling-site sweep — when changing a story ID
reference, grep the workspace for all occurrences before the PR is opened).

### AC-011 — Existing S-PERF-GATE-001/002/003/004/007 groups unchanged

```
grep -c 'serial-subprocess = { max-threads = 1 }' .config/nextest.toml
grep -c 'adv-p02-serial = { max-threads = 1 }' .config/nextest.toml
grep -c 'bc-2-01-013-serial = { max-threads = 1 }' .config/nextest.toml
grep -c 'dtu-cap = { max-threads = 4 }' .config/nextest.toml
grep -c 'spec-engine-wasm-cap = { max-threads = 4 }' .config/nextest.toml
grep -c 'spec-engine-http-cap = { max-threads = 4 }' .config/nextest.toml
```

Expected output: `1` for each (all 6 groups present with their original `max-threads`
definitions unchanged).

Traces to: BC-5.39.001 postcondition — no regression in existing SIGTERM protection
(S-PERF-GATE-001), adv_p02 serialization (S-PERF-GATE-002), bc_2_01_013 stability
(S-PERF-GATE-003), DTU HTTP-server cap (S-PERF-GATE-004), or WASMtime/HTTP caps
(S-PERF-GATE-007).

### AC-012 — `just check` exits 0 with all changes applied

```
just check
echo "Exit: $?"
```

Expected output: `Exit: 0`.

A non-zero exit on the nextest step may indicate a TOML syntax error in `.config/nextest.toml`
(run `cargo nextest show-config --workspace --profile prepush` to validate TOML first).
A non-zero exit on the clippy step indicates a Rust compilation issue in `plugin/mod.rs`.

Run `just check` once at the end of the implementer's work, after all ACs are verified.

Traces to: BC-5.39.001 postcondition (delivery quality — the gate must not be broken).

## Red Gate

Two Red Gate tests. Both live in `crates/prism-spec-engine/src/plugin/mod.rs`
`#[cfg(test)] mod tests`. Both fail before implementation (compile error — `apply_wasmtime_cache`
does not exist; or for AC-005's test, the `event_type` assertion fails because the WARN
branch is unreachable via `new_with_audit_sink`). Both pass after implementation.

### RG-001 — `test_S_PERF_GATE_008_apply_wasmtime_cache_degradable_path_does_not_panic`

**Pre-implementation state:** `apply_wasmtime_cache` does not exist → compile error → test
binary fails to build.

**Post-implementation state:** `apply_wasmtime_cache` exists with the degradable match arm →
`Err(...)` branch does not panic, does not return `Err` → test passes.

**Why this is load-bearing:** if the prototype's `?` pattern is used instead of the
degradable match arm, calling `apply_wasmtime_cache` with `Err(...)` would panic (the
function returns `Result` and the `?` propagates — but since the test helper returns `()`
not `Result`, the match arm structure must not propagate). The test exposes this by
directly exercising the `Err` branch without an external `.prx` artifact.

**SID-1 compliance:**
- Not `#[ignore]`'d: forced-failure uses `CacheConfig::with_directory("relative/not/absolute")`
  which triggers the `is_absolute()` check before any filesystem I/O — no external service,
  no temp dirs, no permission juggling; deterministic on macOS/Linux/Windows.
- Exercises real production code path: `apply_wasmtime_cache` is extracted from
  `new_with_audit_sink`; the test calls it directly with a real `wasmtime::Cache::new` result.
- No `#[cfg(test)]` on the helper itself: the production WARN emission must be
  unconditional (SAP-1 / BC-2.16.002 catalog mandate).

### RG-002 — `test_S_PERF_GATE_008_apply_wasmtime_cache_emits_warn_on_err`

**Pre-implementation state:** `apply_wasmtime_cache` does not exist → compile error.
Alternatively: if extracted helper exists but the WARN `event_type` is incorrect or
absent → tracing assertion fails.

**Post-implementation state:** `apply_wasmtime_cache` exists; `tracing::warn!` fires with
`event_type = "plugin.compilation_cache_init_skipped"` and `error = %e` → captured output
contains the expected string → test passes.

**Why this is load-bearing (SAP-1):** Without this test, a refactor that removes or
renames the `event_type` field would not be caught by the adversary's SAP-1 probe
(which greps source code) if only the `grep` is relied upon. The test independently
asserts the WARN fires at runtime with the correct structured field.

**Implementation note:** use the `Arc<Mutex<Vec<u8>>>` buffer pattern established in
`test_F_LP7_MED_001_host_emit_acquire_token_parse_error_fires_unconditionally`. This pattern
is the project-established mechanism for capturing `tracing::warn!` emissions in unit tests
without external subscribers.

## Behavioral Contracts

| BC | Title | Role in this story |
|----|-------|--------------------|
| BC-5.39.001 | 3-CLEAN convergence protocol | Delivery-quality gate — this story's PR must pass 3-CLEAN before merge |
| BC-2.16.002 | Multi-Step Fetch Pipeline Execution — Sequential Steps with Variable Interpolation | §Postconditions Canonical Structured Event Catalog v1.92: row `plugin.compilation_cache_init_skipped` (WARN, SAP-1 D8 obligation). Implementer MUST verify the v1.92 row is present in the BC before opening the PR (same-commit rule is already satisfied — the row was added in the spec burst). |

## Tasks

1. **Read** `crates/prism-spec-engine/src/plugin/mod.rs`: the `new_with_audit_sink` function
   body (to understand the existing wasmtime initialization sequence) and the
   `#[cfg(test)] mod tests` block (to understand the tracing capture pattern used in
   `test_F_LP7_MED_001_*`).

2. **Read** `.factory/specs/behavioral-contracts/BC-2.16.002-multi-step-fetch-pipeline.md`
   and verify the `plugin.compilation_cache_init_skipped` catalog row is present with
   `error = %e` field and WARN level (BC-2.16.002 v1.92). If absent, STOP — escalate to
   orchestrator before proceeding (the product-owner SAP-1 obligation is incomplete).

3. **Edit** `crates/prism-spec-engine/Cargo.toml`:
   - Find the `wasmtime = { version = "44", features = ["component-model"] }` line.
   - Add `"cache"` to the features array:
     ```toml
     wasmtime = { version = "44", features = ["component-model", "cache"] }
     ```
   - Do not change the version number or any other field.

4. **Write Red Gate tests FIRST** in `crates/prism-spec-engine/src/plugin/mod.rs`:
   - Add two failing test stubs at the end of `#[cfg(test)] mod tests`:
     ```rust
     #[test]
     fn test_S_PERF_GATE_008_apply_wasmtime_cache_degradable_path_does_not_panic() {
         todo!("RG-001: implement after apply_wasmtime_cache is extracted")
     }

     #[test]
     fn test_S_PERF_GATE_008_apply_wasmtime_cache_emits_warn_on_err() {
         todo!("RG-002: implement after apply_wasmtime_cache is extracted")
     }
     ```
   - Verify these fail: `cargo nextest run -p prism-spec-engine -E 'test(S_PERF_GATE_008)'`
     must show 2 FAIL (panics from `todo!()`). This is the Red Gate.

5. **Implement** `plugin/mod.rs` production changes:
   a. Add `fn apply_wasmtime_cache` before `impl PluginRuntime`:
      ```rust
      fn apply_wasmtime_cache(config: &mut wasmtime::Config, cache_result: Result<wasmtime::Cache, wasmtime::Error>) {
          match cache_result {
              Ok(cache) => { config.cache(Some(cache)); }
              Err(e) => {
                  tracing::warn!(
                      event_type = "plugin.compilation_cache_init_skipped",
                      error = %e,
                      "wasmtime compilation cache init failed; proceeding without cache (degraded performance)"
                  );
              }
          }
      }
      ```

   b. In `new_with_audit_sink`, after `config.epoch_interruption(true);` and before
      `wasmtime::Engine::new(&config)`, insert:
      ```rust
      // Enable the wasmtime compilation cache (S-PERF-GATE-008).
      // ...  (full comment block per §Background)
      apply_wasmtime_cache(&mut config, wasmtime::Cache::new(wasmtime::CacheConfig::new()));
      ```

   c. Replace the `todo!()` stubs in the two test functions with full implementations
      following the tracing-capture pattern from
      `test_F_LP7_MED_001_host_emit_acquire_token_parse_error_fires_unconditionally`.

6. **Edit** `.config/nextest.toml`:
   a. In the `[test-groups]` block, after `spec-engine-http-cap = { max-threads = 4 }`, add:
      ```toml
      # S-PERF-GATE-008: Serialize wasmtime-heavy spec-engine + prism-ocsf binaries.
      # With the on-disk compilation cache enabled in PluginRuntime::new_with_audit_sink
      # (ADR-049 D1), warm cache hits skip Cranelift JIT entirely (<1s per plugin vs
      # 80-150s cold). max-threads=1 serializes the cold-start warm-up run so each .prx
      # file is compiled once, then all subsequent binaries load from cache.
      # For the degradable path (ADR-049 D3, cache unavailable), max-threads=1 also
      # bounds total WASMtime CPU overhead — correctness is unaffected.
      # Binaries (all call PluginRuntime::new_with_audit_sink with a real .prx file):
      #   plugin_tests, crowdstrike_oauth2_plugin_tests, enrichment_pivot_002_tests,
      #   plugin_integration_tests, infusion_tests (prism-spec-engine)
      #   spec_driven_mapper_fixtures (prism-ocsf)
      # nextest last-match semantics: for the 5 binaries also in spec-engine-wasm-cap
      # (S-PERF-GATE-007), this override wins (appears later in the file), downgrading
      # their cap from max-threads=4 to max-threads=1.
      spec-engine-wasmtime = { max-threads = 1 }
      ```

   b. After the existing `[[profile.prepush.overrides]]` stanzas, add:
      ```toml
      [[profile.prepush.overrides]]
      # S-PERF-GATE-008: Serialize wasmtime-heavy test binaries via compilation cache group.
      filter = 'binary(plugin_tests) | binary(crowdstrike_oauth2_plugin_tests) | binary(enrichment_pivot_002_tests) | binary(plugin_integration_tests) | binary(infusion_tests) | binary(spec_driven_mapper_fixtures)'
      test-group = 'spec-engine-wasmtime'
      ```

   c. After the existing `[[profile.ci.overrides]]` stanzas, add the same override with
      `[[profile.ci.overrides]]` header (identical filter and test-group).

7. **Verify** AC-001 through AC-011 grep/check commands each return their expected values.
   Run each before running `just check`.

8. **Run** `cargo nextest show-config test-groups --profile prepush` and
   `cargo nextest show-config test-groups --profile ci` (AC-009). Capture both outputs
   for the PR evidence bundle.

9. **Run** `cargo nextest run -p prism-spec-engine -E 'test(S_PERF_GATE_008)'` to verify
   both Red Gate tests pass GREEN (not `todo!()` panic).

10. **Run** `just check` once to verify AC-012 (exit 0). Record the wall-clock nextest
    execution time for the PR evidence bundle.

11. **Verify** AC-010 (D9 relabeling): `grep -rn 'S-PERF-GATE-006' crates/ .config/nextest.toml`
    returns zero hits in working source.

12. **Confirm** modified files are exactly:
    `crates/prism-spec-engine/Cargo.toml`,
    `crates/prism-spec-engine/src/plugin/mod.rs`,
    `.config/nextest.toml`.
    No `.factory/` changes — state-manager handles those.

## Token Budget Estimate

| Context component | Estimated tokens |
|-------------------|-----------------|
| This story spec (v1.0, ~340 lines) | ~9,000 |
| `plugin/mod.rs` (1696 lines — read in full for test pattern context) | ~20,000 |
| `Cargo.toml` (prism-spec-engine, ~100 lines — read + edit 1 line) | ~1,000 |
| `.config/nextest.toml` (~215 lines — read + add ~20 lines) | ~2,500 |
| `BC-2.16.002` (catalog row verification — read v1.92 catalog section only) | ~2,000 |
| AC verification grep outputs (~12 commands) | ~600 |
| `cargo nextest show-config` output (2 profiles) | ~600 |
| `just check` output (one workspace run) | ~2,000 |
| **Total** | **~37,700** |

Well within the implementer agent's context window (~180k tokens). The full plugin/mod.rs
read is required to understand the tracing capture test pattern and to correctly position
the `apply_wasmtime_cache` helper and its call site.

## Previous Story Intelligence

### From S-PERF-GATE-007 (PR #209, merged develop@e3148007)

- The `[test-groups]` + `[[profile.prepush.overrides]]` / `[[profile.ci.overrides]]`
  TOML pattern is established. Copy-adapt: add one group with max-threads=1 (more
  restrictive than the max-threads=4 groups already present).
- AC grep anchoring lesson: grep strings must be anchored to the exact whitespace form
  used in the file (`{ max-threads = 1 }` with spaces — consistent with project style).
  Use AC-006's exact grep string to pre-verify before and after.
- AC-009 lesson (v1.7): check BOTH `--profile prepush` AND `--profile ci` in
  `nextest show-config`. A ci-only mistyped binary name produces a false-green on the
  prepush check.

### From S-PERF-GATE-006 (Justfile RUSTFLAGS, merged)

- Config-only portions (Cargo.toml, nextest.toml) have zero Red Gate tests. The
  Red Gate tests in this story are for the production Rust changes in `plugin/mod.rs`
  only. `red_gate_tests: 2` in frontmatter refers to these.

### Tracing capture test pattern

- The `test_F_LP7_MED_001_host_emit_acquire_token_parse_error_fires_unconditionally`
  test uses `Arc<Mutex<Vec<u8>>>` + custom `BufWriter` + `tracing_subscriber::fmt`
  for capturing tracing output. Reuse this exact pattern for RG-002 — it is the
  project-established mechanism and was adversarially verified in PLUGIN-MIGRATION-001-E.
- The subscriber setup uses `tracing_subscriber::util::SubscriberInitExt::set_default`
  to avoid global subscriber conflicts between tests.

### D3 vs D5 independence

- The degradable path (D3, `plugin/mod.rs`) and the nextest serialization (D5,
  `nextest.toml`) are independent. D5 provides performance protection even when the cache
  is unavailable (D3 degradable path triggers). The story should deliver both in a single
  commit per TD-VSDD-053 (single-commit-per-burst).

## Architecture Compliance Rules

Extracted from architecture sections and ADRs relevant to this story:

1. **ADR-049 D3 (LOCKED) — Degradable cache-init.** `new_with_audit_sink` MUST NOT propagate
   cache-init failure to its caller. `PrismError::Internal` for a cache I/O failure is
   explicitly forbidden. The degradable match arm is the ONLY acceptable implementation.

2. **ADR-022 (Arc-DI wiring).** No changes to `PluginRuntime`'s constructor signature or
   its Arc<dyn ...> parameters. The `apply_wasmtime_cache` extraction is an internal
   refactor within the constructor — no plumbing changes visible to callers.

3. **SAP-1 (CLAUDE.md — tracing emission catalog completeness).** The
   `event_type = "plugin.compilation_cache_init_skipped"` emission must have a
   BC-2.16.002 §Postconditions catalog row with full field schema before the PR merges.
   The row was added in v1.92 (spec burst). Verify at PR time.

4. **SID-1 (CLAUDE.md — no-ignored-test rationalization prohibition).** Both Red Gate
   tests must be non-`#[ignore]`'d. `apply_wasmtime_cache` is a pure function with no
   external dependencies; there is no justification for `#[ignore]`.

5. **TD-VSDD-053 (single-commit-per-burst).** All three file changes (Cargo.toml,
   plugin/mod.rs, nextest.toml) must be in a single commit. No "Stage 1 / Stage 2"
   split commits.

6. **TD-VSDD-060 (sibling-site sweep).** Before committing, grep `S-PERF-GATE-006` across
   the entire workspace working source to verify D9 relabeling is complete (AC-010).

7. **TD-VSDD-091 (behavioral anchors, not line numbers).** Comments in `plugin/mod.rs`
   and `nextest.toml` must cite `S-PERF-GATE-008` and behavioral anchors (ADR-049 D3,
   ADR-049 D5) rather than file line numbers.

8. **No AI attribution in commits** per project git conventions (CLAUDE.md).

9. **No `--no-verify` hook bypass.** If `lefthook` pre-push fails, diagnose and fix.

10. **`apply_wasmtime_cache` helper must not use `#[cfg(test)]` gate** on the function
    definition. The production WARN emission must be reachable from non-test code paths
    (SAP-1 / BC-2.16.002). Using `#[cfg(test)]` on the helper is the paper-fix pattern
    (TD-VSDD-059) and is forbidden.

11. **`.config/nextest.toml` TOML syntax.** Override entries use `[[profile.X.overrides]]`
    (double brackets). A single-bracket `[profile.X.overrides]` is a TOML syntax error.

12. **`binary(name)` filter.** Use `binary(name)` (test binary filename stem from
    `tests/*.rs`), NOT `package(name)`. The 6 binaries span `prism-spec-engine` and
    `prism-ocsf` — a package regex would over-constrain all tests in those packages.

## Library and Framework Requirements

No new library dependencies are introduced. The `"cache"` feature is additive within the
already-pinned `wasmtime = { version = "44", ... }` dependency. wasmtime 44 is already
audited (the `Cargo.toml` comment "wasmtime 44 resolves 17 RUSTSEC advisories"). The
`CacheConfig` and `Cache` types live within the same crate version.

- `wasmtime`: `44` (pinned, unchanged — only feature `"cache"` added)
- `anyhow`: already in the dependency graph (used by existing tests via `tracing_subscriber`).
  Note: `wasmtime::Cache::new` returns `Result<Cache, wasmtime::Error>`; `wasmtime::Error` is
  `anyhow`-compatible under std but the helper signature uses `wasmtime::Error` directly per the
  verified API (see §References research file §1).
- `tracing_subscriber`: already present (used in the existing tracing capture tests)

No `cargo deny` or `cargo audit` action is required per ADR-049 D4.

## File Structure Requirements (§FSR)

| File | Change type | Details |
|------|-------------|---------|
| `crates/prism-spec-engine/Cargo.toml` | Modify | Add `"cache"` to wasmtime features array (1-line change) |
| `crates/prism-spec-engine/src/plugin/mod.rs` | Modify | Add `fn apply_wasmtime_cache` helper; add its call site in `new_with_audit_sink` between `epoch_interruption` and `Engine::new`; add 2 unit tests in `#[cfg(test)] mod tests` |
| `.config/nextest.toml` | Modify | Add `spec-engine-wasmtime = { max-threads = 1 }` to `[test-groups]`; add 1 `[[profile.prepush.overrides]]` stanza; add 1 `[[profile.ci.overrides]]` stanza |

**Files explicitly excluded from this story:**

- `Justfile` — no change needed
- `.cargo/config.toml` — no change needed
- Any other `crates/**/*.rs` file
- `.factory/` — state-manager handles STORY-INDEX and BC lifecycle

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Cache directory is read-only or disk is full | D3 degradable path: `Cache::new` returns `Err`; `apply_wasmtime_cache` emits `plugin.compilation_cache_init_skipped` WARN; `new_with_audit_sink` returns `Ok` and proceeds without cache. wasmtime compiles fresh on each cold start. Operator can investigate via WARN log. |
| EC-002 | Cache directory does not exist yet | `CacheConfig::new()` uses OS defaults; wasmtime creates the directory automatically on first cache write. No error from `Cache::new`. |
| EC-003 | `.prx` plugin binary changes (e.g., plugin upgrade) | wasmtime cache key includes `wasm_binary_hash`. The new binary gets a cache miss on first load; Cranelift recompiles and updates the cache entry. No intervention required. |
| EC-004 | wasmtime version upgrade | Cache key includes `compiler_version`. A new wasmtime version invalidates all existing cache entries (cold start on first run post-upgrade). Expected and correct behavior. |
| EC-005 | Two spec-engine-wasmtime overrides conflict with a future spec-engine-wasm-cap amendment | nextest last-match wins. The spec-engine-wasmtime overrides must be appended AFTER spec-engine-wasm-cap overrides in the file so that spec-engine-wasmtime (max-threads=1) takes precedence for the 5 overlapping binaries. |
| EC-006 | `apply_wasmtime_cache` called on a platform where wasmtime cache is unsupported | `Cache::new` returns `Err` on unsupported platforms; the D3 degradable path triggers; WARN emitted; boot continues. AC-004 / AC-005 unit tests exercise this path (inject `Err` directly). |
| EC-007 | `infusion_tests` binary renamed in a future story | The `binary(infusion_tests)` filter would stop matching. The binary would fall under `spec-engine-wasm-cap` (max-threads=4) via S-PERF-GATE-007's override, rather than `spec-engine-wasmtime` (max-threads=1). A future story renaming the binary MUST update this filter. |

## References

- `.factory/research/wasmtime-44-cache-api-S-PERF-GATE-008.md` — wasmtime 44 Cache / CacheConfig
  API verification (remove-uncertainty pass, 2026-07-01). Verified: `Cache::new(CacheConfig) ->
  Result<Cache, wasmtime::Error>`; `Config::cache(Option<Cache>)` is the correct modern attach
  method (not the legacy `cache_config_load*` TOML API); SID-1 forced-failure mechanism via
  `CacheConfig::with_directory("relative/not/absolute")` — the `is_absolute()` check in
  `validate()` fires before any filesystem I/O, returning `Err` deterministically on all platforms
  with zero side effects.

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.1 | 2026-07-01 | story-writer | Remove-uncertainty pass: confirmed wasmtime 44 API signatures (`Cache::new -> Result<Cache, wasmtime::Error>`, `Config::cache(Option<Cache>)`); replaced synthetic `anyhow::anyhow!` SID-1 forced-failure with verified `CacheConfig::with_directory("relative/not/absolute")` mechanism (is_absolute check, pre-FS, deterministic cross-platform, zero side effects); updated helper signature from `anyhow::Error` to `wasmtime::Error`; added §References citation to `.factory/research/wasmtime-44-cache-api-S-PERF-GATE-008.md`; removed line-number citations from RG-002 narrative, Previous Story Intelligence, and Tasks step 1 per TD-VSDD-091. |
| 1.0 | 2026-07-01 | story-writer | Initial draft. Human-directed perf story per ADR-049 (ACCEPTED 2026-07-01). Prototype branch 76821af7 verified green. Two required corrections: D3 degradable path (override prototype `?` pattern) + D9 relabeling (S-PERF-GATE-006 → S-PERF-GATE-008). BC-2.16.002 v1.92 catalog row already present (spec burst). |
