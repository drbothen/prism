---
document_type: story
story_id: S-PERF-GATE-008
title: "wasmtime compilation cache — enable on-disk native-code cache in PluginRuntime with degradable boot semantics (D3), SAP-1 structured event, and nextest spec-engine-wasmtime serialization group (max-threads=1)"
epic_id: EPIC-MAINTENANCE
version: "1.10"
status: merged
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
#   BC-2.16.002 — multi-step fetch pipeline; already ACTIVE (v1.92). This story amended
#     its §Postconditions catalog only (added plugin.compilation_cache_init_skipped row,
#     SAP-1 D8 obligation discharged in the spec burst). No lifecycle transition — POL-14
#     is a NO-OP at merge for BOTH BCs (both already ACTIVE before this story).
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
the cache warms — reducing per-call `PluginRuntime::new()` cost from ~8-9 s (cold, parallel
workspace contention; profiling §3c) to ~1-2 s (warm cache hit; per §Evidence §3c / ADR-049 §Consequences), saving ~150-200 s total
wall-clock across the wasmtime test group (profiling §REC-1).

## Narrative

As a Prism developer, I want `PluginRuntime::new_with_audit_sink()` to initialize the
wasmtime on-disk compilation cache with degradable semantics (cache-init failure logs a
`WARN` and continues — it does NOT abort boot), so that repeated nextest runs and production
process restarts load compiled `.prx` native code from disk instead of re-running the
Cranelift JIT compiler, reducing per-call `PluginRuntime::new()` cost from ~8-9 s (cold,
parallel workspace contention; profiling §3c) to ~1-2 s (warm cache hit; per §Evidence §3c / ADR-049 §Consequences), saving ~150-200 s
total wall-clock across the wasmtime test group (profiling §REC-1), and keeping the 6
wasmtime-heavy test binaries within the nextest 180 s hard-kill ceiling under full
workspace load.

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
(baseline develop@8bc0404e (the S-PERF-GATE-005 merge; profiled before S-PERF-GATE-006/007 merged)):**

| Condition | wasmtime `PluginRuntime::new()` per-call cost (cold = ~8–9s; warm = ~1–2s) |
|-----------|----------------------------------------------------------------------------|
| Cold start, isolated | ~1-2 s (Cranelift JIT, single process; profiling §3c) |
| Cold start, parallel workspace (before this story) | ~8-9 s (Cranelift JIT under CPU contention; profiling §3c) |
| Warm cache hit (after this story) | ~1-2 s (cache load, no Cranelift; see ADR-049 §Consequences) |

The nextest `spec-engine-wasmtime = { max-threads = 1 }` group (D5) ensures that during
the cold cache warm-up run, only one wasmtime-heavy binary runs at a time. Once the first
binary has compiled and cached each `.prx` file, all subsequent binaries load from cache
at ~1-2 s per plugin (per §Evidence §3c / ADR-049 §Consequences). The `max-threads = 1` serialization is intentionally more aggressive
than the `spec-engine-wasm-cap = { max-threads = 4 }` group from S-PERF-GATE-007 — with
cache hits, there is no CPU contention to manage; serialization maximizes cache hit rate
on the cold-start run.

**Interaction with S-PERF-GATE-007 (nextest first-match-wins per setting):**

S-PERF-GATE-007 added `spec-engine-wasm-cap = { max-threads = 4 }` for 7 binaries. This
story adds `spec-engine-wasmtime = { max-threads = 1 }` for 6 binaries. The 5 binaries
present in BOTH filters (`plugin_tests`, `crowdstrike_oauth2_plugin_tests`,
`enrichment_pivot_002_tests`, `plugin_integration_tests`, `spec_driven_mapper_fixtures`)
will be governed by `spec-engine-wasmtime` (max-threads=1) because the new override stanzas
must appear BEFORE the existing `spec-engine-wasm-cap` override stanzas — nextest
first-match-wins per setting (confirmed via `cargo nextest show-config`, empirically
verified; F-PG008-P1-HIGH-001). Placing wasmtime AFTER wasm-cap silently leaves 5 of 6
binaries at max-threads=4 — the exact functional defect this correction closes. The 2
binaries only in `spec-engine-wasm-cap` (`plugin_boot_tests`, `infusion_boot_integration`)
do not match the `spec-engine-wasmtime` filter and fall through to max-threads=4 via the
later wasm-cap stanza. The 1 binary only in `spec-engine-wasmtime` (`infusion_tests`) gains
a max-threads=1 constraint not previously applied.

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
// Warm cache hits skip Cranelift entirely (see ADR-049 / S-PERF-GATE-008 for
// measured figures). Cache directory:
// OS default (~/.cache/wasmtime/ or
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
  full Cranelift cold-compile cost (~1-2 s per plugin in isolation (profiling §3c)). Subsequent binaries
  load the same `.prx` files from cache at ~1-2 s each (per §Evidence §3c / ADR-049 §Consequences).
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

Traces to: ADR-049 D4 (wasmtime `"cache"` feature addition — activates `Cache` and
`CacheConfig` types); BC-5.39.001 postcondition (delivery quality).

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

Traces to: ADR-049 D3 (LOCKED decision — cache-init failure must not abort plugin
runtime construction); BC-5.39.001 postcondition (delivery quality; ADR-049 D3 is a
locked architectural constraint).

### AC-003 — Degradable path emits `plugin.compilation_cache_init_skipped` WARN with `error` field (D8 / SAP-1)

```
grep 'event_type = "plugin.compilation_cache_init_skipped"' crates/prism-spec-engine/src/plugin/mod.rs | grep -vE '^[[:space:]]*(///|//)' | wc -l
```

Expected output: `1`. Intent: verify exactly ONE production emission (non-comment) of the
SAP-1 event per SAP-1 single-emission requirement. The bare `event_type = "..."` string also
appears in a doc comment (`///`) and a code comment (`//`) within the test module; those are
excluded by the `grep -vE '^[[:space:]]*(///|//)'` filter so that only the unconditional `tracing::warn!`
call in the `Err` arm of `apply_wasmtime_cache` is counted.

Verify the `error = %e` field is present:

```
grep -A3 'plugin.compilation_cache_init_skipped' crates/prism-spec-engine/src/plugin/mod.rs | grep -c 'error = %e'
```

Expected output: `1`.

Verify the emission is unconditional (not inside `#[cfg(test)]`):

The load-bearing proof is RG-002 (`test_S_PERF_GATE_008_apply_wasmtime_cache_emits_warn_on_err`):
RG-002 calls `apply_wasmtime_cache` at runtime and asserts the `plugin.compilation_cache_init_skipped`
WARN fires — this test cannot pass if the function is `#[cfg(test)]`-gated (the helper would not
be visible as a production function callable from non-test code, and the tracing capture would
not fire). The adversary confirmed SAP-1 satisfied via RG-002.

For a deterministic static check, verify `apply_wasmtime_cache` is defined OUTSIDE
`#[cfg(test)] mod tests` by asserting no `#[cfg(test)]` attribute immediately precedes
the function definition:

```
grep -B1 'fn apply_wasmtime_cache' crates/prism-spec-engine/src/plugin/mod.rs | grep -c '#\[cfg(test)\]'
```

Expected output: `0`. (The line immediately preceding the function definition is a doc comment
`/// S-PERF-GATE-008 / ADR-049 D3.`, not a `#[cfg(test)]` attribute.)

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

Traces to: ADR-049 D3 (LOCKED decision — `Err` branch must not abort plugin runtime
construction); SID-1 (CLAUDE.md §Standing Implementer Disciplines — no
`#[ignore]`-rationalization prohibition); BC-5.39.001 postcondition (delivery quality).

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
grep -c 'binary(infusion_tests)' .config/nextest.toml
```

Expected output: `2` (one in prepush filter, one in ci filter — both under
`spec-engine-wasmtime`). Anchored to the filter form `binary(infusion_tests)` to exclude
the D5 comment token (the `# Binaries:` comment listing `infusion_tests` by name) which
contains the bare string `infusion_tests` but is not a filter reference.

The `infusion_tests` binary is the meaningful addition in this story's 6-binary filter
vs. the S-PERF-GATE-007 wasm-cap 7-binary filter. Its presence confirms the filter
was correctly written. Additionally verify the crowdstrike binary:

```
grep -c "binary(crowdstrike_oauth2_plugin_tests)" .config/nextest.toml
```

Expected output: `4` (2 from S-PERF-GATE-007 `spec-engine-wasm-cap` prepush+ci, 2 from
this story's `spec-engine-wasmtime` prepush+ci — first-match-wins per setting: the
`spec-engine-wasmtime` override stanza appears BEFORE `spec-engine-wasm-cap` in each
profile, so `crowdstrike_oauth2_plugin_tests` resolves to `spec-engine-wasmtime`
(max-threads=1) on first match).

Note: `infusion_tests` (`tests/infusion_tests.rs` in `prism-spec-engine`) is DISTINCT from
`infusion_boot_integration` (`tests/infusion_boot_integration.rs` in `prism-bin`). The
former is in the `spec-engine-wasmtime` filter (this story). The latter is in
`spec-engine-wasm-cap` (S-PERF-GATE-007) and is NOT added here.

Traces to: ADR-049 D5; BC-5.39.001 postcondition (delivery quality; all 6 binaries must
appear in the filter for the serialization to take effect).

### AC-009 — Binary-name resolution: all 6 spec-engine-wasmtime binaries resolve to max-threads=1 in BOTH profiles, with full output saved (D5)

Both profiles must be checked independently. Run BOTH commands and redirect the combined
output to the evidence file:

```
cargo nextest show-config test-groups --profile prepush  > docs/demo-evidence/S-PERF-GATE-008/show-config-evidence.txt
cargo nextest show-config test-groups --profile ci       >> docs/demo-evidence/S-PERF-GATE-008/show-config-evidence.txt
```

Create the directory `docs/demo-evidence/S-PERF-GATE-008/` if absent before running.

**A summarized "resolves ✓" in the PR description is insufficient (F-PG008-P1-HIGH-001
under-verification).** The actual raw `show-config` output must be committed to the
evidence file so reviewers can independently verify that each of the 6 binaries is listed
under `spec-engine-wasmtime` with `max-threads = 1` — not under `spec-engine-wasm-cap`
with `max-threads = 4` (the pre-fix ordering silently left 5 of 6 binaries uncapped-to-intent).

Expected output for EACH profile section in the evidence file: the `spec-engine-wasmtime`
group resolves all 6 binaries with non-empty test lists AND each binary shows
max-threads = 1 (not 4):

- `plugin_tests` (prism-spec-engine) — resolves under `spec-engine-wasmtime`, max-threads=1
- `crowdstrike_oauth2_plugin_tests` (prism-spec-engine) — resolves under `spec-engine-wasmtime`, max-threads=1
- `enrichment_pivot_002_tests` (prism-spec-engine) — resolves under `spec-engine-wasmtime`, max-threads=1
- `plugin_integration_tests` (prism-spec-engine) — resolves under `spec-engine-wasmtime`, max-threads=1
- `infusion_tests` (prism-spec-engine) — resolves under `spec-engine-wasmtime`, max-threads=1
- `spec_driven_mapper_fixtures` (prism-ocsf) — resolves under `spec-engine-wasmtime`, max-threads=1

If any of these 6 binaries resolves under `spec-engine-wasm-cap` (max-threads=4), the
`spec-engine-wasmtime` override stanza is incorrectly positioned AFTER the wasm-cap stanza
for that profile — reverse the ordering.

A binary name that appears in the filter but resolves to an empty test list indicates a
mistyped binary name. nextest silently no-ops a zero-match `binary()` filter, so a mistyped
name leaves `just check` GREEN while the serialization constraint is binding to nothing.
Both profiles must be checked independently — a ci-only mistyped name produces a false-green
on the prepush check (S-PERF-GATE-007 AC-009 lesson, version 1.7).

Traces to: ADR-049 D5; BC-5.39.001 postcondition (delivery quality; caps are only
effective when binary-name filters resolve to real compiled test binaries and the
show-config output is verified, not merely summarized).

### AC-010 — D9 relabeling: no working-source `S-PERF-GATE-006` references (D9)

```
grep -rn 'S-PERF-GATE-006' crates/prism-spec-engine/ .config/nextest.toml
```

Expected output: no hits.

The prototype incorrectly labels all comments `S-PERF-GATE-006`. Delivered working-source
comments in `crates/prism-spec-engine/src/plugin/mod.rs` and `.config/nextest.toml` must
cite `S-PERF-GATE-008`.

`Justfile` is EXCLUDED from the scope of this check. S-PERF-GATE-008 does NOT modify
`Justfile`; the 4 `S-PERF-GATE-006` references in `Justfile` are the merged S-PERF-GATE-006
story's own legitimate RUSTFLAGS rationale comments — they are in-scope for S-PERF-GATE-006,
not a relabeling obligation for S-PERF-GATE-008. Similarly, `crates/` beyond
`crates/prism-spec-engine/` is not in scope (S-PERF-GATE-008 touches only
`crates/prism-spec-engine/`).

Allowed exceptions (not flagged as failures):
- `.factory/` files: historical changelog rows, ADR cross-references, and BC amendment
  rows that cite `S-PERF-GATE-006` are immutable historical records (TD-VSDD-091).
- Merged PR commit messages and git history: immutable.
- The STORY-INDEX entry for S-PERF-GATE-006 itself.
- `Justfile`: out of S-PERF-GATE-008's modification scope; its `S-PERF-GATE-006` references
  are the sibling story's own rationale comments, not prototype mislabeling.

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
      # (ADR-049 D1), warm cache hits skip Cranelift JIT entirely (see ADR-049 /
      # S-PERF-GATE-008 for measured figures). max-threads=1
      # serializes the cold-start warm-up run so each .prx file is compiled once,
      # then all subsequent binaries load from cache.
      # For the degradable path (ADR-049 D3, cache unavailable), max-threads=1 also
      # bounds total WASMtime CPU overhead — correctness is unaffected.
      # Binaries (all call PluginRuntime::new_with_audit_sink with a real .prx file):
      #   plugin_tests, crowdstrike_oauth2_plugin_tests, enrichment_pivot_002_tests,
      #   plugin_integration_tests, infusion_tests (prism-spec-engine)
      #   spec_driven_mapper_fixtures (prism-ocsf)
      # nextest first-match-wins per setting: for the 5 binaries also in spec-engine-wasm-cap
      # (S-PERF-GATE-007), this override wins because the spec-engine-wasmtime override stanza
      # is placed BEFORE spec-engine-wasm-cap in each profile's override list (see steps 6b/6c).
      # Placing wasmtime AFTER wasm-cap silently leaves 5 of 6 binaries at max-threads=4
      # (F-PG008-P1-HIGH-001 — functional defect, not a prose nit).
      spec-engine-wasmtime = { max-threads = 1 }
      ```

   b. BEFORE the existing `[[profile.prepush.overrides]]` stanza for `spec-engine-wasm-cap`
      (nextest first-match-wins per setting — the wasmtime stanza must appear first so the
      5 overlapping binaries resolve to max-threads=1 before the wasm-cap stanza is reached;
      inserting AFTER the wasm-cap stanza is the F-PG008-P1-HIGH-001 defect), add:
      ```toml
      [[profile.prepush.overrides]]
      # S-PERF-GATE-008: Serialize wasmtime-heavy test binaries via compilation cache group.
      filter = 'binary(plugin_tests) | binary(crowdstrike_oauth2_plugin_tests) | binary(enrichment_pivot_002_tests) | binary(plugin_integration_tests) | binary(infusion_tests) | binary(spec_driven_mapper_fixtures)'
      test-group = 'spec-engine-wasmtime'
      ```

   c. BEFORE the existing `[[profile.ci.overrides]]` stanza for `spec-engine-wasm-cap`
      (same first-match-wins principle as step (b)), add the same override with
      `[[profile.ci.overrides]]` header (identical filter and test-group).

7. **Verify** AC-001 through AC-011 grep/check commands each return their expected values.
   Run each before running `just check`.

8. **Run** AC-009 evidence capture (F-PG008-P1-HIGH-001 tightening — a summarized
   "resolves ✓" is no longer accepted):
   ```
   mkdir -p docs/demo-evidence/S-PERF-GATE-008
   cargo nextest show-config test-groups --profile prepush  > docs/demo-evidence/S-PERF-GATE-008/show-config-evidence.txt
   cargo nextest show-config test-groups --profile ci       >> docs/demo-evidence/S-PERF-GATE-008/show-config-evidence.txt
   ```
   Verify in the saved file that all 6 expected binaries appear under `spec-engine-wasmtime`
   with max-threads=1 for EACH profile. If any shows max-threads=4 the stanza ordering is
   wrong — fix the ordering before proceeding. Commit the evidence file with the
   implementation commit (AC-009).

9. **Run** `cargo nextest run -p prism-spec-engine -E 'test(S_PERF_GATE_008)'` to verify
   both Red Gate tests pass GREEN (not `todo!()` panic).

10. **Run** `just check` once to verify AC-012 (exit 0). Record the wall-clock nextest
    execution time for the PR evidence bundle.

11. **Verify** AC-010 (D9 relabeling): `grep -rn 'S-PERF-GATE-006' crates/prism-spec-engine/ .config/nextest.toml`
    returns zero hits. `Justfile` is excluded — its `S-PERF-GATE-006` references are the sibling
    story's own rationale comments, outside S-PERF-GATE-008's modification scope (see AC-010).

12. **Confirm** modified files are exactly:
    `crates/prism-spec-engine/Cargo.toml`,
    `crates/prism-spec-engine/src/plugin/mod.rs`,
    `.config/nextest.toml`.
    No `.factory/` changes — state-manager handles those.

## Token Budget Estimate

| Context component | Estimated tokens |
|-------------------|-----------------|
| This story spec (v1.10, ~900 lines) | ~24,000 |
| `plugin/mod.rs` (1696 lines — read in full for test pattern context) | ~20,000 |
| `Cargo.toml` (prism-spec-engine, ~100 lines — read + edit 1 line) | ~1,000 |
| `.config/nextest.toml` (~215 lines — read + add ~20 lines) | ~2,500 |
| `BC-2.16.002` (catalog row verification — read v1.92 catalog section only) | ~2,000 |
| AC verification grep outputs (~12 commands) | ~600 |
| `cargo nextest show-config` output (2 profiles) | ~600 |
| `just check` output (one workspace run) | ~2,000 |
| **Total** | **~52,700** |

Well within the implementer agent's context window (~180k tokens). The full plugin/mod.rs
read is required to understand the tracing capture test pattern and to correctly position
the `apply_wasmtime_cache` helper and its call site.

## Previous Story Intelligence

### From S-PERF-GATE-007 (PR #211, merged develop@c6d6e4fa)

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
| EC-005 | Two spec-engine-wasmtime overrides conflict with a future spec-engine-wasm-cap amendment | nextest first-match-wins per setting. The spec-engine-wasmtime override stanzas must be placed BEFORE spec-engine-wasm-cap override stanzas in each profile so that spec-engine-wasmtime (max-threads=1) is the first match for the 5 overlapping binaries. Any future amendment to spec-engine-wasm-cap that inserts a new stanza before the wasmtime stanza will silently re-cap those binaries at max-threads=4; verify ordering via `cargo nextest show-config test-groups` after any amendment. |
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
| 1.10 | 2026-07-02 | story-writer | F-P3-MED-001 warm-cache-figure reconciliation (spec-only; PR HEAD 091f1af8 frozen). (1) §Evidence column header corrected: "wasmtime Component::new() per-call cost" → "wasmtime `PluginRuntime::new()` per-call cost (cold = ~8–9s; warm = ~1–2s)" — the metric scope was previously mislabeled as the inner `Component::new()` step rather than the full `PluginRuntime::new()` per-call cost that §3c measures. (2) §Evidence warm row corrected: "<1 s (cache load, no Cranelift)" → "~1-2 s (cache load, no Cranelift; see ADR-049 §Consequences)" — aligns the canonical figure with ADR-049 v1.3 §Consequences authority. (3) Opening description line 46: "<1 s (warm cache hit)" → "~1-2 s (warm cache hit; per §Evidence §3c / ADR-049 §Consequences)". (4) §Narrative line 56: same substitution. (5) §Evidence narrative (post-table paragraph): "at <1 s per plugin" → "at ~1-2 s per plugin (per §Evidence §3c / ADR-049 §Consequences)". (6) §Why max-threads = 1 second bullet: "from cache at <1 s each" → "from cache at ~1-2 s each (per §Evidence §3c / ADR-049 §Consequences)". §Evidence table is now the single canonical figure source; all other mentions cross-reference it. Changelog rows 1.8/1.9 (immutable historical records, TD-VSDD-091) left unchanged. Token Budget self-reference updated v1.9 → v1.10. |
| 1.9 | 2026-07-02 | story-writer | F-PRLx-MED-001 + F-PRL-P2-MED-001 + definitive figure/version/ref consolidation. (1) §Evidence performance-context label corrected: "baseline develop@8bc0404e after S-PERF-GATE-004/005/006/007" → "baseline develop@8bc0404e (the S-PERF-GATE-005 merge; profiled before S-PERF-GATE-006/007 merged)" matching ADR-049 §Context ("S-PERF-GATE-004 and -005"). (2) §Why max-threads = 1 first bullet corrected: "~1-5 s per plugin in isolation" (retired stale figure) → "~1-2 s per plugin in isolation (profiling §3c)" matching §Evidence table row. Definitive consolidation sweep — every perf-figure location audited: opening description (~8-9 s §3c / <1 s / ~150-200 s §REC-1), §Narrative (same, canonical), §Evidence table (canonical source: ~1-2 s §3c / ~8-9 s §3c / <1 s), §Background Degradable-Path Code Rust comment and §Tasks 6a TOML comment template (both qualitative "see ADR-049 / S-PERF-GATE-008 for measured figures" from v1.8) — no additional figures found; §Why max-threads (the single remaining stale value) now reconciled. Cross-story PR#/SHA sweep: develop@8bc0404e = S-PERF-GATE-005 merge (PR #210) ✓; PR #211/c6d6e4fa = S-PERF-GATE-007 ✓; no 004/006 PR# mis-references found. BC-2.16.002 v1.92 citations consistent (AC-003, AC-005, Behavioral Contracts table). Token Budget self-reference updated v1.8 → v1.9. TD-VSDD-091. |
| 1.8 | 2026-07-02 | story-writer | F-P1a-LOW-001 (code-comment templates de-figured, S-7.02): removed time figures from §Background Degradable-Path Code Rust comment template (lines "reducing per-call load from ~8-9 s / (cold, parallel workspace contention; profiling §3c) to <1 s") and §Tasks step 6a TOML comment template ("warm cache hits skip Cranelift JIT entirely (<1 s per call vs / ~8-9 s cold, parallel workspace contention (profiling §3c))") — replaced both with qualitative "see ADR-049 / S-PERF-GATE-008 for measured figures" form matching the delivered code (mod.rs + nextest.toml comments are already qualitative per S-7.02 as confirmed by ADR-049 v1.1 and the v1.7 code sweep). F-P3-LOW-002 (S-PERF-GATE-007 PR#/SHA corrected): §Previous Story Intelligence heading corrected from "PR #209, merged develop@e3148007" (that is S-PERF-GATE-004's merge info) to "PR #211, merged develop@c6d6e4fa" (S-PERF-GATE-007's actual merge info per STORY-INDEX D-1479). Cross-story PR#/SHA sweep results: develop@8bc0404e in §Evidence is the correct profiling-baseline SHA (= S-PERF-GATE-005 merge SHA; not a PR# misattribution); no further PR#/SHA misattributions found in §Evidence, §Background, §Dependencies, or any other section. Token Budget self-reference updated to v1.8. Version bump 1.7 → 1.8. |
| 1.7 | 2026-07-02 | story-writer | F-PG008-PRL-P1-MED-001 remediation: retired "80-150 s" / "80-150s" figure swept from all live story content (opening description, §Narrative, §Evidence table, §Background §Degradable-Path Code Rust comment, §Tasks step 6a TOML comment) and replaced with profiling-sourced figures — per-call `PluginRuntime::new()` cost ~8-9 s under workspace-parallel CPU contention / ~1-2 s in isolation (profiling §3c); group savings ~150-200 s (profiling §REC-1). False profiling-report attribution corrected: "80-150 s" never appeared in `.factory/research/test-suite-perf-profile-2026-06-30.md`; the §Evidence table column header changed from "per-test cost" to "per-call cost" and the isolated-case figure updated from ~1-5 s to ~1-2 s to match profiling §3c. Token Budget self-reference updated to v1.7 (line count ~unchanged). ADR-049 v1.1 already retired the same figure; story now matches ADR-049 v1.1 §Context/§Consequences. Version bump 1.6 → 1.7. |
| 1.6 | 2026-07-02 | story-writer | Pass-2 LOW + comprehensive self-reference currency sweep. Token Budget self-reference corrected: story spec row updated from (v1.0, ~340 lines, ~9,000 tokens) to (v1.6, ~900 lines, ~24,000 tokens); total recomputed ~37,700 → ~52,700. Comprehensive self-reference sweep: no other stale self-references found — 12 ACs (AC-001..AC-012), 2 Red Gate tests (red_gate_tests: 2 frontmatter and RG-001/RG-002 in body), and 2 BCs (behavioral_contracts: [BC-5.39.001, BC-2.16.002] in frontmatter and Behavioral Contracts table in body) all match current story content; external artifact versions confirmed current: ADR-049 v1.0 (read and verified), BC-2.16.002 v1.92 (read and verified). Version bump 1.5 → 1.6. |
| 1.5 | 2026-07-02 | story-writer | F-PG008-PRL2-MED-001 + F-PG008-PRL2-LOW-001 remediation (spec-only; PR HEAD e6a357fe frozen). MED-001: corrected frontmatter `# BC status:` comment that falsely stated "BC-2.16.002 transitions draft→active at merge per POL-14" — BC-2.16.002 is already ACTIVE at v1.92; this story amended its §Postconditions catalog only (added plugin.compilation_cache_init_skipped row, SAP-1 D8 obligation); no lifecycle transition occurs at merge; POL-14 is a NO-OP for BOTH BCs (both already ACTIVE before this story). LOW-001: removed fabricated BC-2.16.002 postcondition attributions from AC-001 ("WASM plugin runtime correctness"), AC-002 ("degradable boot"), and AC-004 ("degradable path") — none of these postconditions exist in BC-2.16.002 (Multi-Step Fetch Pipeline Execution / CAP-029); re-anchored AC-001 to ADR-049 D4 (wasmtime cache feature), AC-002 and AC-004 to ADR-049 D3 (LOCKED degradable decision). BC-2.16.002 citations now reserved for AC-003 and AC-005 (catalog-row tracing) only. POL-8 bidirectional coherence verified: BC-2.16.002 remains in frontmatter array, cited by AC-003 + AC-005; BC-5.39.001 cited by delivery-quality ACs throughout. |
| 1.4 | 2026-07-02 | story-writer | F-PG008-P1-HIGH-001 remediation: corrected a genuine functional defect — nextest per-test override resolution is first-match-wins per setting (not last-match-wins as previously stated). The prior spec asserted the wasmtime override stanzas should appear AFTER wasm-cap, which caused the delivered `.config/nextest.toml` to leave 5 of 6 wasmtime-heavy binaries silently at max-threads=4 instead of the intended 1. Corrections: (1) §Evidence heading + body: "last-match-wins semantics" → "first-match-wins per setting"; "appear AFTER" → "must appear BEFORE"; added defect summary sentence. (2) AC-008: "last-match semantics means the binary ultimately uses spec-engine-wasmtime" → "first-match-wins per setting: spec-engine-wasmtime stanza appears BEFORE spec-engine-wasm-cap". (3) AC-009: tightened from "capture output in PR evidence bundle" to require saving actual `cargo nextest show-config` output for both profiles to `docs/demo-evidence/S-PERF-GATE-008/show-config-evidence.txt`; explicit "summarized ✓ is insufficient"; added per-binary max-threads=1 verification requirement. (4) Tasks step 6a TOML comment: "last-match semantics … appears later in the file" → "first-match-wins … placed BEFORE wasm-cap". (5) Tasks step 6b: "After the existing [[profile.prepush.overrides]] stanzas" → "BEFORE the existing [[profile.prepush.overrides]] stanza for spec-engine-wasm-cap". (6) Tasks step 6c: same correction for ci profile. (7) Tasks step 8: replaced "capture outputs for the PR evidence bundle" with mkdir + redirect commands to the evidence file and explicit max-threads=1 verification. (8) EC-005: "last-match wins … appended AFTER" → "first-match-wins … placed BEFORE"; added future-amendment verification instruction. Version bump 1.3 → 1.4 per POL-32 (newest-first changelog). |
| 1.3 | 2026-07-01 | story-writer | F-M1 (MED) remediation: portability class-sweep for grep-recipe BSD/GNU divergence. AC-003 check-1: replaced GNU-only `\s` with POSIX bracket class `[[:space:]]` in `grep -vE '^[[:space:]]*(///|//)'` — `\s` is not guaranteed portable on BSD/macOS grep (treated as literal `s` on some systems, causing indented comment lines to pass the filter and returning 3 instead of Expected 1). Updated matching prose description in AC-003 body to stay consistent with the executable recipe. Full class-sweep of all grep/rg/shell recipes in the story (AC-001 through AC-012, Red Gate, §FSR, Tasks): no other GNU-only constructs found — all remaining recipes use only fixed-string literals, POSIX ERE alternation `(a|b)`, standard anchors `^` and `$`, and quantifiers `*` and `+`. Codified fix for the grep-recipe-portability sub-class of the F-1 lesson (recurrence prevention). Version bump 1.2 → 1.3 per POL-32 (newest-first changelog). Spec-only; frozen HEAD 5d2d7aad unchanged. |
| 1.2 | 2026-07-01 | story-writer | F-1 (MED) remediation: four grep-based AC self-verification recipes made precise, anchored, and scope-restricted so each returns its stated Expected value against correct delivered code (code unchanged). AC-003 check 1: replaced bare `grep -c 'event_type = "..."'` (returned 3 due to doc/code comments) with comment-excluding pipeline `grep ... | grep -vE '^\s*(///|//)' | wc -l` → Expected 1. AC-003 check 3: replaced fragile `grep -B10 'plugin.compilation_cache_init_skipped' | grep -c 'cfg(test)'` (returned 1 due to anti-pattern comment in B10 context of a string-literal match) with `grep -B1 'fn apply_wasmtime_cache' | grep -c '#\[cfg(test)\]'` → Expected 0; cited RG-002 as load-bearing runtime proof per adversary confirmation. AC-008: replaced bare `grep -c 'infusion_tests'` (returned 3 due to D5 comment token) with `grep -c 'binary(infusion_tests)'` → Expected 2. AC-010: removed `Justfile` from grep scope (4 legitimate S-PERF-GATE-006 sibling-story comments, outside 008's modification perimeter); restricted to `crates/prism-spec-engine/ .config/nextest.toml` → Expected no hits; updated Tasks step 11 to match. Pattern: recurrence of S-PERF-GATE-007 F-LOW-1 (grep-recipe false-failure on correct artifacts). |
| 1.1 | 2026-07-01 | story-writer | Remove-uncertainty pass: confirmed wasmtime 44 API signatures (`Cache::new -> Result<Cache, wasmtime::Error>`, `Config::cache(Option<Cache>)`); replaced synthetic `anyhow::anyhow!` SID-1 forced-failure with verified `CacheConfig::with_directory("relative/not/absolute")` mechanism (is_absolute check, pre-FS, deterministic cross-platform, zero side effects); updated helper signature from `anyhow::Error` to `wasmtime::Error`; added §References citation to `.factory/research/wasmtime-44-cache-api-S-PERF-GATE-008.md`; removed line-number citations from RG-002 narrative, Previous Story Intelligence, and Tasks step 1 per TD-VSDD-091. |
| 1.0 | 2026-07-01 | story-writer | Initial draft. Human-directed perf story per ADR-049 (ACCEPTED 2026-07-01). Prototype branch 76821af7 verified green. Two required corrections: D3 degradable path (override prototype `?` pattern) + D9 relabeling (S-PERF-GATE-006 → S-PERF-GATE-008). BC-2.16.002 v1.92 catalog row already present (spec burst). |
