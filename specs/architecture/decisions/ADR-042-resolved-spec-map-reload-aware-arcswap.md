---
document_type: adr
adr_id: "ADR-042"
title: "Reload-Aware resolved_spec_map — ArcSwap-Backed Overlay Map with Hot-Reload Rebuild"
status: proposed
date: "2026-06-20"
modified: "2026-06-20"
version: "1.1"
producer: architect
subsystems_affected: [SS-10, SS-11, SS-09]
supersedes: null
superseded_by: null
amends: null
anchor_stories: [S-DEMO-PRISMQL-ONBOARDING-001-A]
related_adrs: [ADR-007, ADR-022, ADR-030, ADR-039]
related_bcs: [BC-2.10.013, BC-2.10.012, BC-2.06.012]
locked_decisions: [D1, D2, D3, D4]
wiring_deferred_to: null
---

# ADR-042: Reload-Aware resolved_spec_map — ArcSwap-Backed Overlay Map with Hot-Reload Rebuild

## Status

PROPOSED v1.1 (2026-06-20). Design-ready; implementation owned by S-DEMO-PRISMQL-ONBOARDING-001-A (D-1267: folded into 001-A, not a separate follow-up story). Will become ACCEPTED on merge of that story's PR.

Authorized by D-1267 (human override of the deferral recommendation in
`001-A-reload-notify-diff-adjudication.md`; also directs folding this work into
S-DEMO-PRISMQL-ONBOARDING-001-A rather than a phantom follow-up story).

---

## Context

### The Boot-Frozen Problem

`QueryEngine.resolved_spec_map` is a `Option<Arc<HashMap<(OrgSlug, SensorId), ResolvedSensorSpec>>>`
field populated once in `QueryEngine::new_full` at boot time. When a hot-reload fires via
`reload_config`, `prism_spec_engine::reload_config::reload_config()` atomically swaps the inner
`ConfigSnapshot` inside `ConfigManager`, but `resolved_spec_map` is never updated.

This causes two concrete failures in multi-tenant overlay mode (org "acme" → sensor "crowdstrike"):

**Failure 1 — Read path stale schema:** `prism_describe("acme")` and
`resources/read("prismql://schema/acme")` read from `query_engine.resolved_spec_map()`, which
returns the boot-time Arc. A hot-reload that adds a table to the "crowdstrike" TYPE spec never
propagates to "acme"'s resolved schema in the multi-tenant path. Post-reload reads serve stale
schema (violates BC-2.10.012 post-reload freshness in multi-tenant mode).

**Failure 2 — Notify-diff misses org-keyed subscriptions:** `reload_config`'s per-client
notify-diff (server.rs lines 3218–3280) looks up subscribers by `config_manager.sensor_specs.get(slug.as_str())`,
keyed by sensor_id. When org_slug != sensor_id (multi-tenant overlay mode), the lookup returns
`None` and no notification fires. EC-10-029 is not satisfied for pure overlay orgs (violates
BC-2.10.013 EC-10-029 in multi-tenant mode).

### Current Architecture for Comparison

The existing `ConfigSnapshot` hot-reload already follows the correct pattern (AD-007):
- `ConfigManager` wraps `ArcSwap<ConfigSnapshot>`.
- `reload_config` calls `cm.store(new_snapshot)` — atomic, wait-free, in-flight queries
  unaffected.

`resolved_spec_map` should follow the same pattern.

### Why Not a Simpler Fix?

**Simpler Option A — rekey the diff from sensor_id to a secondary org→sensor map in ConfigSnapshot:**
`ConfigSnapshot` would need a new `HashMap<OrgSlug, Vec<SensorId>>` field. This partially fixes
Failure 2 but leaves Failure 1 (stale read path) unaddressed. Discarded.

**Simpler Option B — rebuild resolved_spec_map into a separate ArcSwap without changing the field type:**
Requires `PrismServer` to hold an `Arc<ArcSwap<HashMap>>` alongside `QueryEngine`. Adds
a second code path instead of fixing the existing one. Discarded.

**Correct approach:** Make `resolved_spec_map` itself reload-aware by changing its storage type
to `Option<Arc<ArcSwap<HashMap>>>`. This fixes both failures with a single structural change and
is consistent with AD-007.

---

## Decision

### D1: Change `resolved_spec_map` field type to `Option<Arc<ArcSwap<HashMap<...>>>>`

**Before:**
```
QueryEngine.resolved_spec_map: Option<Arc<HashMap<ResolvedSpecKey, ResolvedSensorSpec>>>
```

**After:**
```
QueryEngine.resolved_spec_map: Option<Arc<ArcSwap<HashMap<ResolvedSpecKey, ResolvedSensorSpec>>>>
```

The outer `Option` is preserved (single-tenant / no-overlay mode = `None`).
The `ArcSwap` is owned by a single `Arc` so that the engine can be cheaply cloned/shared
without extra indirection. The `ArcSwap` is never replaced — only its inner `Arc<HashMap>` is
swapped on reload.

**`arc-swap` dependency:** Add `arc-swap = "1"` to `crates/prism-query/Cargo.toml`.
This matches the version already used by prism-mcp, prism-bin, and prism-spec-engine.

### D2: Public accessor `resolved_spec_map()` — return type unchanged

The existing public signature:
```rust
pub fn resolved_spec_map(
    &self,
) -> Option<Arc<HashMap<ResolvedSpecKey, ResolvedSensorSpec>>>
```
is preserved. Internally, the body changes from `as_ref().map(Arc::clone)` to
`as_ref().map(|swap| swap.load_full())`.

`ArcSwap::load_full()` returns a new `Arc<HashMap>` — same return type. All callers
(prism-mcp, prism-query internals) are unaffected.

**In-flight query semantics:** A query that calls `resolved_spec_map()` at fan-out time
receives an `Arc<HashMap>` snapshot. If a reload fires before the query completes, the query's
held `Arc` is unaffected (refcount protects it). The query sees a consistent overlay map for its
entire lifetime. Subsequent queries see the new map. This matches the existing
`ConfigSnapshot` / `ConfigManager` in-flight-query semantics (AD-007).

### D3: New method `QueryEngine::rebuild_resolved_spec_map`

A new `pub` method on `QueryEngine`:
```rust
pub fn rebuild_resolved_spec_map(
    &self,
    customers_dir: &std::path::Path,
    type_specs: &HashMap<String, prism_spec_engine::spec_parser::SensorSpec>,
    org_registry: &prism_core::OrgRegistry,
) -> Result<usize, prism_spec_engine::error::SpecEngineError>
```

Behavior:
- If `self.resolved_spec_map` is `None` (single-tenant): returns `Ok(0)` — no-op.
- Calls `OverlayLoader::load_overlays(customers_dir, type_specs, org_registry)`.
- If `load_overlays` returns validation errors: logs, retains prior map, returns `Err`
  (DI-031 fail-closed — do not wipe a valid map on partial reload failure).
- On success: calls `swap.store(Arc::new(result.resolved))` atomically.
  Returns `Ok(entry_count)`.

**Error policy:** Non-fatal in the `reload_config` call path. The caller logs and continues.
The reload result JSON is still returned to the MCP caller; the overlay failure is recorded
in the tracing log only.

### D4: Hot-reload wiring in `reload_config_core` (prism-mcp)

After `prism_spec_engine::reload_config::reload_config(...)` returns `Ok` inside
`reload_config_core`, add:

1. Extract `type_specs` from the post-reload `ConfigSnapshot` (via `cm_guard.load().sensor_specs.clone()`).
2. Derive `customers_dir = spec_dir.join("customers")`.
3. Call `qe.rebuild_resolved_spec_map(&customers_dir, &type_specs, &org_registry)`.
4. Log the result (non-fatal on error).

**Ordering invariant:** rebuild MUST happen AFTER the ConfigSnapshot swap (step 1 of
`reload_config`) and BEFORE the notify-diff (step 2 of `reload_config`). The current
code structure in `reload_config` (which calls `reload_config_core` and then computes
the new_tables diff) already implies this ordering if the rebuild is placed inside
`reload_config_core`.

### D5: Per-org notify-diff update (prism-mcp)

After the rebuild, the notify-diff in `reload_config` (lines 3254–3280) switches from
reading `config_manager.sensor_specs.get(slug.as_str())` to reading from the rebuilt
`resolved_spec_map`:

**Old_tables (pre-rebuild):** captured before `reload_config_core` — use existing logic
(reads from `TableRegistry` or from the pre-reload `config_manager` snapshot). No change.

**New_tables (post-rebuild):**
- When `qe.resolved_spec_map()` returns `Some(arc_map)`:
  - For each subscribed OrgSlug `slug`: collect all table names from entries where
    `key.0 == slug` (filter by org slug, aggregate all sensor tables for that org).
  - This correctly handles the multi-tenant case where one org maps to multiple sensors.
- When `qe.resolved_spec_map()` returns `None` (single-tenant):
  - Fall back to existing `config_manager.sensor_specs.get(slug.as_str())` lookup.
  - Backward compatible.

---

## Rationale

### Why ArcSwap over RwLock

AD-007 (codified in `prism-spec-engine` lib.rs module-level doc): "Config hot reload MUST use
`ArcSwap<ConfigSnapshot>` — never `RwLock`." The same constraint applies to any reload-path
state. `ArcSwap` provides wait-free reads (O(1) atomic load), whereas `RwLock` would block
readers for the duration of the write. The resolved spec map is read on every fan-out
operation — blocking readers during rebuild (even briefly) would violate the latency
requirements of the query hot path.

### Why the outer Option is preserved

The `Option` represents whether multi-tenant overlay mode is configured. `None` means
"no `customers/` directory exists or no overlay files were found at boot." In this mode,
all read paths fall through to `config_manager.sensor_specs`, which is already reload-aware
via `ConfigManager`'s own `ArcSwap<ConfigSnapshot>`. Changing `None` to `Some(ArcSwap(empty))`
would require callers to handle both `None` and `Some(ArcSwap(empty_map))` as equivalent —
unnecessary complexity for no gain.

### Why `OverlayLoader::load_overlays` is called synchronously

`OverlayLoader::load_overlays` is a synchronous filesystem operation (reads
`customers/<slug>/*.overlay.toml` files). The same operation already runs synchronously at
boot in `step4_load_sensor_specs_with_overlays`. In the reload path it is called from
async Tokio context — it must be wrapped in `tokio::task::spawn_blocking` to avoid blocking
the async runtime thread pool. **The implementer must add this wrapper.** The design calls
`rebuild_resolved_spec_map` from `reload_config_core`, which is async — the implementer
must ensure the synchronous `load_overlays` call does not block the Tokio executor.

This is an implementation detail not a design decision; it mirrors the pattern used for
other synchronous spec operations in the reload path.

### Performance: rebuild cost

`OverlayLoader::load_overlays` reads and parses per-org overlay TOML files from disk.
For a typical MSSP deployment:
- 10–50 orgs × 1–3 overlay files each = 10–150 file reads per reload.
- Each file is a small TOML fragment (typically < 2 KB).
- Total rebuild time: < 5 ms on local NVMe (cold disk), < 1 ms on warm cache.

This is acceptable for a manual-triggered reload operation. Hot-path queries are unaffected
(they call `ArcSwap::load_full()` which is O(1) atomic).

### Consistency: both read path and notify-diff see the same map generation

Because the rebuild happens inside `reload_config_core` and the notify-diff runs in
`reload_config` immediately after `reload_config_core` returns, both observe the same
ArcSwap generation:
- The notify-diff's new_tables is computed from the same post-rebuild ArcSwap load.
- Subsequent `prism_describe` / `resources/read` calls load the same post-rebuild Arc.
- There is no window where the notify-diff fires on old data while the read path serves
  new data (or vice versa).

---

## Consequences

### Positive
- `prism_describe("acme")` returns fresh schema after any hot-reload that modifies the
  underlying sensor TYPE spec, even in multi-tenant overlay mode.
- `resources/read("prismql://schema/acme")` — same freshness guarantee.
- EC-10-029 satisfied in multi-tenant overlay mode (org != sensor).
- EC-10-030 extended: per-org notification is driven by the rebuilt `resolved_spec_map`,
  which correctly scopes changes to affected orgs only.
- Consistent with AD-007 / AD-018: no new concurrency mechanism introduced.

### Negative / trade-offs
- `arc-swap` becomes a dependency of `prism-query`. It was previously only needed by
  prism-spec-engine, prism-mcp, and prism-bin. Acceptable: `arc-swap` is already a
  direct dependency of three sibling crates; no new ecosystem dependency.
- `new_full` and test helpers that directly assign `engine.resolved_spec_map = Some(Arc::new(map))`
  must be updated to wrap in `ArcSwap`. Mechanical change; search for all direct assignments
  to the field (grep `resolved_spec_map\s*=\s*Some`).
- `reload_config_core` gains a filesystem I/O call (`load_overlays`). This is a blocking
  operation that must be wrapped in `spawn_blocking` in the async context.

### Neutral
- The existing `INV-OVL-006` invariant stated "read-only after boot." This ADR supersedes
  that constraint for `resolved_spec_map`: it is now "immutable per-query (snapshot semantics)
  but rebuildable on hot-reload via ArcSwap." The invariant is updated to:
  "A query's resolved spec snapshot is read-only for its lifetime; the ArcSwap may be
  rebuilt by the hot-reload path between queries."

---

## Implementation Notes

### Crates affected

| Crate | Change | Effort |
|-------|--------|--------|
| prism-query | Field type change, new method, add arc-swap dep | Medium |
| prism-mcp | `reload_config_core` rebuild call + notify-diff update | Medium |
| prism-bin | No change required (boot path unchanged; step10 watcher = stub) | None |
| prism-spec-engine | No change required (`OverlayLoader::load_overlays` already pub) | None |

### Mechanical field-change search

Grep: `resolved_spec_map\s*=\s*Some` across `crates/` workspace. All direct field
assignments to `resolved_spec_map` (in constructors, test helpers) must be updated to
wrap the `Arc<HashMap>` in `ArcSwap::new(arc)`. Expected sites (as of 2026-06-20):

1. `engine.rs` line ~423 (`new_full` constructor)
2. `engine.rs` line ~1911 (test helper `make_engine_with_resolved_spec_map`)
3. Any other direct assignments (full grep required)

### BC-2.16.002 catalog entries required

Two new `event_type` values (SAP-1 compliance):
- `reload.overlay_rebuilt` — info level, once per successful rebuild
- `reload.overlay_rebuild_failed` — warn level, non-fatal partial failure

Both must be registered in BC-2.16.002 §Postconditions Canonical Structured Event Catalog
before the story PR merges.

---

## References

| Ref | Description |
|-----|-------------|
| AD-007 | `arc-swap` mandate for hot config reload (lock-free reads on query hot path) |
| AD-018 | Automatic filesystem watching — same reload validation as manual reload |
| ADR-030 | Unified `SensorSpec` type: `ConfigSnapshot::sensor_specs` uses `spec_parser::SensorSpec` |
| ADR-039 | Org-scoped TableRegistry error filtering — `resolved_spec_map` as the per-org scoping authority |
| ADR-022 | Production runtime wiring discipline — Arc-DI pattern |
| BC-2.10.013 | `prismql://schema/{client_id}` resource template — subscribe/notify (EC-10-029, EC-10-034) |
| BC-2.10.012 | `prism_describe` schema discovery — post-reload freshness in multi-tenant mode |
| BC-2.06.012 | Per-tenant overlay loading — `OverlayLoader::load_overlays` is the single overlay entry point |
| DI-031 | Fail-closed on reload: retain current config on validation failure |
| `scoping/resolved-spec-map-reload-aware-design.md` | Implementer per-crate change spec |
| `scoping/001-A-reload-notify-diff-adjudication.md` | Origin adjudication (D-1267 deferral override) |

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-06-20 | architect | Initial PROPOSED. D-1267 human override of deferral. ArcSwap field design, rebuild method, reload wiring, notify-diff update, BC-2.10.013 guidance. |
| 1.1 | 2026-06-20 | architect | D-1267 anchor correction: `anchor_stories` repointed from phantom `S-DEMO-PRISMQL-ONBOARDING-001-A-followup` to real story `S-DEMO-PRISMQL-ONBOARDING-001-A`. Work was folded into 001-A per D-1267; no separate follow-up story will be created. Status text updated to reflect 001-A ownership. No technical content changed. |
