---
document_type: adr
adr_id: "ADR-049"
title: "wasmtime Compilation Cache — On-Disk Native-Code Cache for PluginRuntime, Degradable Boot Failure Semantics, and Test-Binary Serialization"
status: ACCEPTED
date: "2026-07-01"
modified: "2026-07-01"
version: "1.0"
producer: architect
subsystems_affected: [SS-17]
supersedes: []
superseded_by: null
amends: null
anchor_stories: [S-PERF-GATE-008]
related_adrs: [ADR-023, ADR-040]
related_bcs: [BC-2.16.002]
locked_decisions: [D3]
wiring_deferred_to: null
---

# ADR-049: wasmtime Compilation Cache — On-Disk Native-Code Cache for PluginRuntime, Degradable Boot Failure Semantics, and Test-Binary Serialization

## Status

ACCEPTED v1.0 (2026-07-01). Human-directed performance story S-PERF-GATE-008.
Enables the wasmtime on-disk compilation cache in `PluginRuntime::new_with_audit_sink()`,
defines degradable boot semantics for cache-init failure, mandates SAP-1 structured
event registration, and codifies nextest test-group serialization for all
wasmtime-heavy test binaries.

---

## Context

### Root-Cause Performance Problem

`wasmtime::Component::new()` (WASM-to-native Cranelift compilation) compiles each `.prx`
plugin binary into CPU-ISA-specific native code on every process start. Under full
workspace parallel load (nextest spawns one OS process per test binary), multiple
concurrent `Engine::new()` initializations and `Component::new()` compilations saturate
all CPU cores simultaneously. The resulting per-test wall-clock time inflates from
<1 s (isolation) to 150–170 s (workspace-parallel) — within 10 s of the nextest
180 s hard-kill ceiling. S-PERF-GATE-004 and S-PERF-GATE-005 (merged PRs #209, #210)
reduced DTU and clone oversubscription but could not eliminate the wasmtime compilation
cost itself.

### Prototype

Reference branch `wip/perf-wasmtime-exploration-76821af7` (single commit 76821af7,
base 0978983f) verified the fix green (`just check`, 5 079 tests pass, zero timeouts):

1. `crates/prism-spec-engine/Cargo.toml` — adds `"cache"` to wasmtime features.
2. `crates/prism-spec-engine/src/plugin/mod.rs` — calls `wasmtime::Cache::new(CacheConfig::new())`
   and passes the result to `config.cache(Some(cache))` before `Engine::new`.
3. `.config/nextest.toml` — adds `spec-engine-wasmtime` test group (`max-threads = 1`)
   applied to all six wasmtime-heavy test binaries via `prepush` and `ci` profile overrides.

**Prototype correctness note:** all inline comments in the prototype reference
`S-PERF-GATE-006` (the wrong story ID). The delivered code must relabel these to
`S-PERF-GATE-008` (D9 below).

**Prototype safety note:** the prototype uses `?` on `Cache::new`, making cache-init
failure fatal at boot. This ADR overrides that decision: D3 mandates degradable
semantics (see below). The implementer must NOT copy the `?` propagation pattern
from the prototype.

### wasmtime Cache Mechanism

wasmtime's `"cache"` feature provides an on-disk compilation artifact cache:

- **Content addressing:** each `.prx` binary is hashed; the cache key is
  `(wasm_binary_hash, compiler_version, cpu_isa_flags)`.
- **Cache directory:** OS-default user-home cache directory
  (macOS: `~/Library/Caches/wasmtime/`, Linux: `~/.cache/wasmtime/`).
  Created automatically on first use.
- **Cold vs warm:** cold start compiles and stores to disk; warm start loads the
  cached native artifact in <1 s, skipping Cranelift entirely.
- **No external config file:** `CacheConfig::new()` uses built-in defaults.
  Works without `~/.config/wasmtime/config.toml`.

### Subsystem Scope

This decision touches SS-17 (WASM Plugin Runtime, `prism-spec-engine`) only.
The cache is transparent to all other subsystems. No API surface changes.

---

## Decisions

### D1 — Enable wasmtime cache in PluginRuntime constructor

`PluginRuntime::new_with_audit_sink()` in `crates/prism-spec-engine/src/plugin/mod.rs`
initializes a `wasmtime::Cache` using `CacheConfig::new()` and passes it to
`config.cache(Some(cache))` before constructing `wasmtime::Engine`.

**Wiring point:** the cache must be constructed and applied between
`config.wasm_component_model(true); config.epoch_interruption(true);` and
`wasmtime::Engine::new(&config)` — the existing sequence in
`PluginRuntime::new_with_audit_sink()`.

### D2 — OS-default cache directory; no external config file required

`CacheConfig::new()` uses built-in defaults. The cache directory is the OS-default
user-home cache location. No `~/.config/wasmtime/config.toml` is created, required,
or read by prism. Operators may optionally customize cache behavior via a
wasmtime config file, but prism does not manage or require one.

### D3 — Cache-init failure is DEGRADABLE (LOCKED decision)

**Decision:** if `Cache::new(CacheConfig::new())` returns an `Err`, prism logs a
`WARN`-level structured tracing event and continues startup WITHOUT the cache.
`config.cache(None)` is the correct call when cache initialization fails.
`PluginRuntime::new_with_audit_sink()` does NOT propagate the error to its caller.

**Rationale:**
- The cache is a performance optimization, not a correctness requirement. Without it,
  wasmtime compiles WASM to native on every process start — slower, but functionally
  correct.
- In production there is typically one process start per analyst session. The cold-start
  cost (1–5 s per plugin, not 150 s under concurrency) is acceptable.
- Failing to open a cache directory (disk-full, permissions issue, read-only filesystem)
  should not prevent the analyst's toolchain from starting. Boot failure on a cache I/O
  issue would be a confusing and unhelpful operator experience.
- The nextest `max-threads = 1` serialization (D5 below) bounds worst-case test
  wall-clock independently of cache availability. The cache accelerates tests but is
  not required to keep them under the 180 s kill ceiling when serialized.

**Override of prototype:** the prototype uses `?` on `Cache::new` (fatal). This ADR
overrides that choice. Implementer must change the prototype code to:
```rust
match wasmtime::Cache::new(wasmtime::CacheConfig::new()) {
    Ok(cache) => { config.cache(Some(cache)); }
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

The `PrismError::Internal` construction in the prototype is **not used**. No error
is returned from `new_with_audit_sink` for this condition.

### D4 — wasmtime `"cache"` feature; no new transitive dependencies

The `"cache"` feature is added to the `wasmtime` dependency in
`crates/prism-spec-engine/Cargo.toml`. wasmtime 44 is already pinned and audited
(the existing comment "resolves 17 RUSTSEC advisories"). Enabling `"cache"` activates
code paths within the already-audited wasmtime 44 crate. `cargo deny` and `cargo audit`
pass on the prototype branch (confirmed via `just check`). No dependency-manifest
review action is required from story-writer or devops-engineer.

### D5 — nextest `spec-engine-wasmtime` test group (max-threads = 1)

`.config/nextest.toml` gains:
1. A new test group `spec-engine-wasmtime = { max-threads = 1 }` under `[test-groups]`.
2. Two `[[profile.prepush.overrides]]` and `[[profile.ci.overrides]]` entries using
   filter `binary(plugin_tests) | binary(crowdstrike_oauth2_plugin_tests) | binary(enrichment_pivot_002_tests) | binary(plugin_integration_tests) | binary(infusion_tests) | binary(spec_driven_mapper_fixtures)`.

**Rationale:** these six binaries all call `PluginRuntime::new_with_audit_sink()` with
a real `.prx` file. Serializing them to `max-threads = 1` ensures: (a) concurrent
`Engine::new()` initializations do not race for CPU during the cache warm-up phase;
(b) total wasmtime CPU overhead is bounded regardless of cache availability;
(c) cache warm-up happens in a defined sequential order, maximizing cache hit rate
for subsequent binaries.

This complements ADR-014 (Local Pre-Push vs CI Gate Asymmetry): the serialization
applies to both `prepush` and `ci` profiles, consistent with the parity principle.

### D6 — Security: OS-default cache directory is in the same trust domain

The cache directory (e.g., `~/Library/Caches/wasmtime/`) stores compiled native
code artifacts. Security properties:
- **Trust domain:** the cache directory is in the user's home directory — the same
  trust boundary as the prism binary, `.prx` plugin files, and prism configuration.
  No cross-trust-domain exposure.
- **Cache poisoning protection:** wasmtime validates the content hash on every cache
  load. A tampered cache entry fails hash validation and triggers fresh recompilation.
  Silent cache poisoning is not possible.
- **No credential content:** cached artifacts are compiled native code — no credential
  values, API keys, or OCSF event data.
- **No hardening required:** accept OS-default directory permissions. No
  prism-specific chmod, directory ACLs, or cache-directory configuration is needed
  or recommended.

### D7 — No dedicated E-PLUGIN error code for cache-init failure

Because cache-init failure is degradable (D3), it does not propagate as a
`PrismError` or `PluginError` and does not surface to MCP callers. The condition is
operator-observable only through the structured tracing event (D8). A dedicated
E-PLUGIN-NNN taxonomy code is not required.

If the production posture is revised to treat cache-init as fatal in a future story,
that story must allocate an E-PLUGIN-024 code and document it in error-taxonomy.md.

### D8 — Structured event obligation (SAP-1)

The degraded path emits exactly one structured event:
```
event_type = "plugin.compilation_cache_init_skipped"
```
Fields: `error = %e` (the `Display` of the `anyhow::Error` from `Cache::new`).

Per SAP-1 (CLAUDE.md §Standing Adversary Probes), this `event_type` value MUST be
registered as a row in the BC-2.16.002 §Postconditions Canonical Structured Event
Catalog **in the same commit** as the code that emits it, before the PR merges.

Product-owner deliverable for S-PERF-GATE-008: add catalog row for
`plugin.compilation_cache_init_skipped` to BC-2.16.002. (See §Product-Owner TODO
in the Scoping Summary.)

### D9 — Story relabeling obligation

All inline comments in `.config/nextest.toml` and
`crates/prism-spec-engine/src/plugin/mod.rs` that reference `S-PERF-GATE-006`
must be changed to `S-PERF-GATE-008`. This includes:
- The `# S-PERF-GATE-006:` comment prefix on the `spec-engine-wasmtime` test group
  and both override blocks in nextest.toml.
- The `// Enable the wasmtime compilation cache (S-PERF-GATE-006).` comment in mod.rs.

TD-VSDD-060 (sibling-site sweep) applies: grep for `S-PERF-GATE-006` across the
entire workspace before the PR is submitted; all hits must be either
(a) historical changelog rows in `.factory/` (immutable per TD-VSDD-091) or
(b) already-merged PRs referenced in git history (not in working source files).

---

## Considered Alternatives

### Alt-A: Fatal boot on cache-init failure (prototype behavior)

Rejected. Cache is a performance optimization; degraded-but-functional startup is
clearly preferable to a boot failure triggered by a disk-full or permissions issue
on the OS cache directory. The analyst's session should start regardless.

### Alt-B: Configurable cache directory in prism.toml

Rejected for v1. The OS-default location is correct for the per-analyst stdio model
(AD-001: single binary, per-analyst process). A custom cache directory config field
in `prism.toml` adds configuration surface with no immediate benefit. Future-wave
story if cross-analyst cache sharing or CI-specific cache paths are needed.

### Alt-C: Disable cache in CI, enable only locally

Rejected. CI and prepush should have the same performance characteristics per
ADR-014 (Local Pre-Push vs CI Gate Asymmetry). The cache accelerates CI builds
equally. The nextest `ci` profile override was specifically added to the prototype
to ensure CI coverage of the serialized test behavior.

---

## Consequences

### Positive
- wasmtime `Component::new()` warm-load time drops from 80–150 s (parallel) to <1 s
  (cache hit) for tests; previously-seen 150–170 s wall-clock times are eliminated.
- Nextest `spec-engine-wasmtime` group enforces a strict serial order for all
  wasmtime-heavy binaries in both prepush and CI profiles.
- Production process restart (e.g., hot-reload) benefits from cached compilation on
  the warm path.
- Zero new transitive Cargo dependencies; no RUSTSEC surface change.

### Neutral
- First cold run after a wasmtime upgrade or `.prx` file change clears the cache
  and recompiles. Wall-clock for the first cold run is unchanged from today; all
  subsequent runs benefit from the cache.
- OS-default cache directory (~/Library/Caches/wasmtime/ or ~/.cache/wasmtime/) is
  created automatically; no operator action required.

### Risk
- If the OS-default cache directory fills disk, wasmtime silently falls back to
  fresh compilation. This is handled gracefully by D3 (degradable boot) and by
  wasmtime's own cache-write retry semantics.

---

## Scoping Summary (Routing Reference for Orchestrator)

| Role | Deliverable | Notes |
|------|-------------|-------|
| **ARCHITECT** | ADR-049 (this doc) | DONE |
| **PRODUCT-OWNER** | Add `plugin.compilation_cache_init_skipped` catalog row to BC-2.16.002 §Postconditions (SAP-1 D8 obligation); fields: `event_type = "plugin.compilation_cache_init_skipped"`, `error = %e`, level WARN, category perf-degradation, audit role: operator-diagnostic, recurrence: once-per-cold-start-failure | Required before PR can merge (SAP-1) |
| **STORY-WRITER** | S-PERF-GATE-008 story spec citing this ADR; AC must cover: D1 implementation, D3 degradable path (test it), D4 feature flag, D5 nextest entries, D9 relabeling; scope = prism-spec-engine + nextest.toml | Standard perf-gate story form |
| **IMPLEMENTER** | Wire D1–D5 + D8 structured event; must NOT use `?` on Cache::new (D3); relabel S-PERF-GATE-006 → S-PERF-GATE-008 in all working-source comments (D9); add BC-2.16.002 catalog row in same commit as tracing::warn! emission (SAP-1) | Prototype (76821af7) is the reference; override D3 (change `?` to WARN pattern) |
| **FORMAL-VERIFIER / ADVERSARY** | SAP-1 probe: verify `plugin.compilation_cache_init_skipped` event_type has BC-2.16.002 catalog row before declaring CLEAN | Standard adversarial probe; no new VP needed |

No new VP (Verification Property) is required for this story. The compilation cache
affects performance only; correctness properties are unchanged. The degradable path
(D3) should be covered by a unit test in `mod.rs #[cfg(test)]` that confirms
`PluginRuntime::new_with_audit_sink` succeeds even when passed a deliberately
broken cache config (e.g., a read-only temp dir) — per SID-1.

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-07-01 | architect | Initial ACCEPTED. Human-directed perf story S-PERF-GATE-008. |
