---
document_type: architecture-design
version: "1.0"
status: final
producer: architect
timestamp: 2026-06-20T00:00:00Z
story: S-DEMO-PRISMQL-ONBOARDING-001-A (authorized deferral; this design targets the follow-up story)
adr: ADR-042
decision_log: D-1267
traces_to: [BC-2.10.013, BC-2.10.012, EC-10-029, EC-10-030]
---

# Resolved-Spec-Map Reload-Aware Design

## Summary

`resolved_spec_map` is currently stored as a plain `Option<Arc<HashMap<...>>>` field on
`QueryEngine` — set once in `new_full` at boot and never updated. This design replaces that
plain field with an `ArcSwap`-backed field following the existing AD-007 pattern, and adds
a rebuild-and-swap method callable from the hot-reload path (prism-mcp `reload_config`,
prism-bin filesystem watcher). The notify-diff in `reload_config` is updated to derive its
per-org diff from the newly rebuilt map rather than from `config_manager.sensor_specs`.

This is the implementation design produced for the follow-up story authorized in
`001-A-reload-notify-diff-adjudication.md`. The human overrode the deferral recommendation
on 2026-06-20 (D-1267) and directed building this immediately.

---

## Current State (Boot-Frozen Pattern)

**Location:** `crates/prism-query/src/engine.rs` line 229–236

```rust
pub(crate) resolved_spec_map: Option<
    Arc<
        std::collections::HashMap<
            prism_spec_engine::ResolvedSpecKey,
            prism_spec_engine::ResolvedSensorSpec,
        >,
    >,
>,
```

**Construction:** `QueryEngine::new_full` receives the map as a parameter and stores it as
`Some(arc)`. The `resolved_spec_map()` accessor at line 516 clones the Arc.

**Read path:** `prism_describe` / `build_tables_for_client` calls `query_engine.resolved_spec_map()`
on every invocation — reads from the boot-time Arc. Post-reload schema changes are invisible in
multi-tenant mode.

**Notify-diff path:** `reload_config` (server.rs line 3218–3280) reads `config_manager.sensor_specs`
directly — correct for single-tenant mode, misses org-slug-keyed entries in multi-tenant mode.

**arc-swap dependency:** `prism-query/Cargo.toml` does NOT currently depend on `arc-swap`.
`prism-mcp`, `prism-spec-engine`, and `prism-bin` already have `arc-swap = "1"`.

---

## Design Decisions

### D1: Storage mechanism — ArcSwap inside Option

The `resolved_spec_map` field changes from:
```rust
Option<Arc<HashMap<ResolvedSpecKey, ResolvedSensorSpec>>>
```
to:
```rust
Option<Arc<arc_swap::ArcSwap<HashMap<ResolvedSpecKey, ResolvedSensorSpec>>>>
```

The outer `Option` is preserved: `None` means the engine is running in single-tenant/test mode
with no overlay config. The `ArcSwap` is only allocated when the overlay map is non-None
(i.e., when `new_full` is called with a populated map).

**Why not a bare ArcSwap on the engine?** Preserving `Option` semantics avoids changing the
existing single-tenant fallback paths in `prism_describe` and the resource handler, which
correctly fall through to `config_manager` when `resolved_spec_map()` returns `None`.

**Why not `RwLock<HashMap>`?** Violates AD-007 and AD-018: all hot-reload state uses ArcSwap,
not RwLock. In-flight queries must not be blocked while the map is rebuilt.

**Thread-safety:** `ArcSwap::load()` is wait-free for readers. `ArcSwap::store()` is the only
write path; it is called once per reload event, never concurrently (the reload is serialized
through `reload_config_core`). In-flight queries that have already called `load()` hold a
`Guard<Arc<HashMap>>` for the duration of their fan-out — the old map reference is not freed
until all such guards are dropped (standard Arc refcount).

### D2: Accessor contract — unchanged external shape

The public `resolved_spec_map(&self)` method signature is unchanged:
```rust
pub fn resolved_spec_map(&self) -> Option<Arc<HashMap<ResolvedSpecKey, ResolvedSensorSpec>>>
```
Callers (prism-mcp, prism-describe) continue to receive `Option<Arc<HashMap>>`. Internally the
implementation changes from `self.resolved_spec_map.as_ref().map(Arc::clone)` to
`self.resolved_spec_map.as_ref().map(|swap| swap.load_full())`.

`ArcSwap::load_full()` returns a new `Arc<HashMap>` (a clone) — the same shape as the current
accessor. Callers that hold this `Arc` for the duration of a query see a consistent snapshot
even if a reload fires mid-query. This is the correct in-flight-query consistency model.

### D3: New method — `rebuild_resolved_spec_map`

```rust
/// Rebuild the resolved spec map from a new ConfigSnapshot and org registry.
///
/// Called from the hot-reload path after the ConfigSnapshot has been swapped.
/// Atomically swaps the ArcSwap; in-flight queries holding a prior Arc are unaffected.
///
/// # Thread-safety
/// ArcSwap::store() is called once per reload; serialized through reload_config_core.
/// No mutex needed; the outer Option is set once at construction time.
///
/// Returns Ok(()) when the map is wired (multi-tenant mode) or when no overlay map
/// was configured at boot (no-op; single-tenant mode). Returns Err only on
/// OverlayLoader I/O or validation failure — caller logs and continues (non-fatal).
pub fn rebuild_resolved_spec_map(
    &self,
    customers_dir: &std::path::Path,
    type_specs: &std::collections::HashMap<String, prism_spec_engine::spec_parser::SensorSpec>,
    org_registry: &prism_core::OrgRegistry,
) -> Result<(), prism_spec_engine::error::SpecEngineError>
```

Implementation sketch (NOT production code — implementer writes the real version):
```rust
let Some(ref swap) = self.resolved_spec_map else {
    return Ok(()); // single-tenant mode — no-op
};
use prism_spec_engine::overlay::OverlayLoader;
let result = OverlayLoader::load_overlays(customers_dir, type_specs, org_registry);
if !result.errors.is_empty() {
    // Non-fatal: log errors, keep the existing map.
    // Rationale: a partial overlay validation failure during reload should not
    // wipe the existing multi-tenant schema. Callers see the previous map.
    return Err(SpecEngineError::...);
}
swap.store(Arc::new(result.resolved));
Ok(())
```

**Error policy:** validation errors from `OverlayLoader` during a reload are NON-FATAL — the
existing map is retained. This matches the `reload_config` existing policy for validation
failures (DI-031 fail-closed: retain current config; do not swap on partial failure).

### D4: Cargo dependency — add arc-swap to prism-query

`crates/prism-query/Cargo.toml` must add:
```toml
arc-swap = "1"
```
This matches the version used by prism-mcp, prism-bin, and prism-spec-engine. No workspace
version pin conflict — all four crates currently use the same `"1"` spec.

---

## Per-Crate Change Spec

### prism-query (`crates/prism-query/`)

**Files:** `src/engine.rs`, `Cargo.toml`

**Cargo.toml change:**
```toml
# Add (line after prism-core dependency is idiomatic):
arc-swap = "1"
```

**engine.rs field change** (`QueryEngine` struct, line ~229–236):
```rust
// BEFORE:
pub(crate) resolved_spec_map: Option<
    Arc<std::collections::HashMap<
        prism_spec_engine::ResolvedSpecKey,
        prism_spec_engine::ResolvedSensorSpec,
    >>,
>,

// AFTER:
pub(crate) resolved_spec_map: Option<
    Arc<arc_swap::ArcSwap<std::collections::HashMap<
        prism_spec_engine::ResolvedSpecKey,
        prism_spec_engine::ResolvedSensorSpec,
    >>>,
>,
```

**engine.rs `new_full` change** (line ~423):
```rust
// BEFORE:
resolved_spec_map: Some(resolved_spec_map),

// AFTER:
resolved_spec_map: Some(Arc::new(arc_swap::ArcSwap::from_pointee(
    // resolved_spec_map is Arc<HashMap<...>>; deref to get HashMap for ArcSwap::from_pointee
    // Actually: ArcSwap wraps the inner Arc directly
    (*resolved_spec_map).clone()
    // NOTE: implementer must check the actual type — new_full receives
    // Arc<HashMap<...>>; ArcSwap<HashMap<...>> needs Arc<HashMap<...>> internally.
    // Use ArcSwap::new(resolved_spec_map) which takes Arc<T>.
))),
```
More precisely:
```rust
resolved_spec_map: Some(Arc::new(arc_swap::ArcSwap::new(resolved_spec_map))),
```
(`ArcSwap::new` takes `Arc<T>` directly; `from_pointee` takes `T` and wraps it.)

**engine.rs `new` / `new_with_cache_config` constructors** — no change; they set
`resolved_spec_map: None`, which is unchanged semantics.

**engine.rs test helpers** (line ~1910):
```rust
// BEFORE:
engine.resolved_spec_map = Some(Arc::new(spec_map));

// AFTER:
engine.resolved_spec_map = Some(Arc::new(arc_swap::ArcSwap::new(Arc::new(spec_map))));
```

**engine.rs `resolved_spec_map()` accessor** (line ~516–526):
```rust
// BEFORE:
self.resolved_spec_map.as_ref().map(Arc::clone)

// AFTER:
self.resolved_spec_map.as_ref().map(|swap| swap.load_full())
```
`ArcSwap::load_full()` returns `Arc<T>` — same return type as before.

**engine.rs new method `rebuild_resolved_spec_map`** — add after `resolved_spec_map()` accessor:

```rust
/// Atomically rebuild and swap the resolved spec map from a new ConfigSnapshot.
///
/// Called by the hot-reload path after `ConfigSnapshot` has been swapped in
/// `ConfigManager`. In-flight queries that have already called `resolved_spec_map()`
/// hold their prior `Arc<HashMap>` for their lifetime; the swap is invisible to them
/// (ADR-042 / AD-007 in-flight-query consistency guarantee).
///
/// # Arguments
/// - `customers_dir` — path to the `customers/` overlay directory.
/// - `type_specs` — TYPE specs from the POST-reload `ConfigSnapshot.sensor_specs`.
/// - `org_registry` — the engine's `OrgRegistry`.
///
/// # Behavior
/// - If `resolved_spec_map` is `None` (single-tenant mode): no-op, returns `Ok(0)`.
/// - If `OverlayLoader::load_overlays` returns validation errors: existing map retained,
///   errors logged, returns `Err`. Caller should log and continue (non-fatal; DI-031).
/// - On success: swaps in the new map, returns `Ok(overlay_count)`.
pub fn rebuild_resolved_spec_map(
    &self,
    customers_dir: &std::path::Path,
    type_specs: &std::collections::HashMap<String, prism_spec_engine::spec_parser::SensorSpec>,
    org_registry: &prism_core::OrgRegistry,
) -> Result<usize, prism_spec_engine::error::SpecEngineError> {
    let Some(ref swap) = self.resolved_spec_map else {
        return Ok(0); // single-tenant mode — no-op
    };
    use prism_spec_engine::overlay::OverlayLoader;
    let result = OverlayLoader::load_overlays(customers_dir, type_specs, org_registry);
    if !result.errors.is_empty() {
        // Non-fatal: log and retain existing map (DI-031 fail-closed on reload).
        tracing::warn!(
            event_type = "reload.overlay_rebuild_failed",
            error_count = result.errors.len(),
            "Hot-reload overlay rebuild failed; retaining prior resolved_spec_map (ADR-042)"
        );
        // Return first error as representative; caller logs the rest.
        return Err(/* map PrismError → SpecEngineError */ ...);
    }
    let count = result.resolved.len();
    swap.store(Arc::new(result.resolved));
    tracing::info!(
        event_type = "reload.overlay_rebuilt",
        overlay_count = count,
        "resolved_spec_map rebuilt and swapped atomically (ADR-042)"
    );
    Ok(count)
}
```

**BC-2.16.002 Canonical Structured Event Catalog:** The two new `event_type` values
(`reload.overlay_rebuild_failed`, `reload.overlay_rebuilt`) MUST be added as rows in the
BC-2.16.002 §Postconditions catalog before the story's PR merges (SAP-1 discipline).

**Visibility:** `rebuild_resolved_spec_map` should be `pub` (or at minimum `pub(crate)` if
only called from prism-mcp/prism-bin within the same binary). Since prism-bin calls it
through the `QueryEngine` handle in `RunningServer`, `pub` is the correct level.

---

### prism-spec-engine (`crates/prism-spec-engine/`)

**No new public API required.** `OverlayLoader::load_overlays` is already `pub`:
```rust
pub fn load_overlays(
    customers_dir: &std::path::Path,
    type_specs: &HashMap<String, SensorSpec>,
    org_registry: &OrgRegistry,
) -> OverlayLoadResult
```

The function signature already matches what `rebuild_resolved_spec_map` needs. No changes.

**The implementer must confirm:** the `SensorSpec` type in the `type_specs` parameter is
`prism_spec_engine::spec_parser::SensorSpec`, which is the same type stored in
`ConfigSnapshot::sensor_specs`. The step4 function extracts `snapshot.sensor_specs.clone()`
and passes it directly. The rebuild path must do the same: extract `type_specs` from the
post-reload `ConfigSnapshot.sensor_specs` map before calling `OverlayLoader::load_overlays`.

---

### prism-bin (`crates/prism-bin/src/boot.rs`)

**Hot-reload wiring (filesystem watcher path):** If the step10 hot-reload watcher
(`step10_start_hot_reload`) eventually calls into a reload path, it must also call
`rebuild_resolved_spec_map` after the `ConfigSnapshot` swap. At present, `step10_start_hot_reload`
is a stub (returns `Ok(())`; see boot.rs line 2911). When it is implemented, it MUST include
the rebuild call.

**The ArcSwap swap listener** (boot.rs line ~2447, `step_register_table_registry_swap_listener`):
This listener fires on `ConfigSnapshot` swap and currently updates `TableRegistry`. After ADR-042,
it should ALSO call `rebuild_resolved_spec_map`. However, the existing listener does NOT have
access to the `customers_dir` path or the `QueryEngine` reference — those are wired separately
in `RunningServer`. The implementer has two options:

**Option A (recommended):** Move the `rebuild_resolved_spec_map` call to `reload_config_core`
in prism-mcp, immediately after `prism_spec_engine::reload_config::reload_config(...)` returns.
This is the manual-reload path; the filesystem watcher path can be addressed when it ships.

**Option B:** Thread `QueryEngine` and `customers_dir` into the swap listener at registration
time. More complex; deferred to when the filesystem watcher actually ships.

**Recommendation:** Use Option A. The manual-reload path is the only reload path currently
wired in production. The filesystem watcher is a stub. Option A adds zero complexity to boot.rs.

---

### prism-mcp (`crates/prism-mcp/src/server.rs`)

**Location of change:** `reload_config_core` (line ~3135).

**Change:** After `prism_spec_engine::reload_config::reload_config(...)` succeeds (line ~3163),
and BEFORE constructing the JSON result, add:

```rust
// ADR-042: rebuild resolved_spec_map atomically from the new ConfigSnapshot.
// Must happen AFTER the ConfigSnapshot swap (which reload_config() performs internally).
// Non-fatal: if overlay rebuild fails, existing map is retained (DI-031).
if let (Some(qe), Some(spec_dir)) = (self.query_engine.as_ref(), self.spec_dir.as_ref()) {
    // Extract post-reload type_specs from the new ConfigSnapshot.
    let cm_guard = cm_arc.load();
    let type_specs = {
        let snap = cm_guard.load();
        snap.sensor_specs.clone()
    };
    // Derive customers_dir from spec_dir (same derivation as step4).
    let customers_dir = spec_dir.join("customers");
    if let Some(org_registry) = qe.org_registry() {
        if let Err(e) = qe.rebuild_resolved_spec_map(&customers_dir, &type_specs, &org_registry) {
            tracing::warn!(
                error = %e,
                event_type = "reload.overlay_rebuild_failed",
                "resolved_spec_map rebuild failed during reload_config; \
                 multi-tenant schema reads will serve prior map (ADR-042 / DI-031)"
            );
            // Non-fatal: continue with reload result.
        }
    }
}
```

**Ordering invariant:** The rebuild MUST happen AFTER the `ConfigSnapshot` swap
(which `prism_spec_engine::reload_config::reload_config()` performs) and BEFORE the
notify-diff computation. This ensures the per-client diff reads from the freshly rebuilt map.

**Accessing `customers_dir`:** `PrismServer.spec_dir` is `Option<PathBuf>` (line ~113). The
customers directory is always `spec_dir/customers`. This is the same derivation used in
`step4_load_sensor_specs_with_overlays` (boot.rs line ~1222).

---

## Multi-Tenant Notify-Diff Update

After `rebuild_resolved_spec_map` succeeds, the per-client notify-diff in `reload_config`
(lines 3218–3280) needs to change from sensor_id-keyed lookup to org-slug-keyed lookup.

**Current pattern (single-tenant, correct for sensor_id == org_slug):**
```rust
snap.sensor_specs.get(slug.as_str())  // looks up by sensor_id string
```

**New pattern (multi-tenant, correct for all modes):**

The notify-diff should compute per-org table sets by reading from the rebuilt
`resolved_spec_map` rather than `config_manager.sensor_specs`. For org slug "acme":
1. Load the current resolved map: `qe.resolved_spec_map()` → `Arc<HashMap<(OrgSlug, SensorId), ResolvedSensorSpec>>`
2. Filter entries where the OrgSlug matches "acme"
3. Collect all table names across all matching sensors

Old table set: capture BEFORE rebuild (from the previous ArcSwap generation).
New table set: capture AFTER rebuild (from the post-reload ArcSwap generation).

**Implementation note for the implementer:** The old table set must be captured BEFORE calling
`rebuild_resolved_spec_map`. The current code captures old_tables BEFORE calling
`reload_config_core`. The rebuild happens inside `reload_config_core`. Therefore:
- old_tables: captured at line ~3218 (before `reload_config_core`) — correct, unchanged.
- new_tables: after `reload_config_core` returns (line ~3254) — switch from reading
  `config_manager.sensor_specs` to reading from `qe.resolved_spec_map()`.

**Single-tenant fallback:** When `qe.resolved_spec_map()` returns `None`, fall back to the
existing `config_manager.sensor_specs` lookup (current behavior). This preserves backward
compatibility for single-tenant deployments.

---

## In-Flight Query Consistency Guarantee (ADR-042 core property)

A query executing concurrently with a `reload_config` call follows this sequence:

1. Query calls `qe.resolved_spec_map()` at fan-out time → receives `Arc<HashMap>` snapshot.
2. Hot-reload fires → `rebuild_resolved_spec_map` calls `ArcSwap::store(new_arc)`.
3. The `ArcSwap` atomically replaces the inner Arc; the query's `Arc<HashMap>` snapshot
   (from step 1) remains valid until the query releases it.
4. Subsequent queries call `resolved_spec_map()` → receive the new Arc.

The Arc refcount on the old HashMap is not decremented to zero until all in-flight queries
that hold a reference to it complete. Memory overhead: at most two HashMap copies exist
simultaneously during a reload (old + new). For a typical deployment with O(10) sensors and
O(5) tables each, the HashMap is O(50) entries — negligible memory.

**No mutex, no blocking.** `ArcSwap::load_full()` is wait-free. `ArcSwap::store()` is a
single atomic pointer exchange plus Arc reference bookkeeping — not a lock.

---

## BC-2.10.013 Update Guidance

### For the product-owner (BC-2.10.013 v1.1 → v1.2)

**Remove** the single-tenant-only limitation that was added in the adjudication (or mark it
resolved). Specifically:

1. **EC-10-029** — current wording: satisfied when sensor_id == org_slug. After this story ships,
   remove the limitation note. The new wording:

   > `resources/subscribe` for `prismql://schema/acme` followed by a hot-reload that adds a
   > CrowdStrike table for "acme" → Server sends `notifications/resources/updated` with
   > `uri: "prismql://schema/acme"` within 1 second. Satisfied in both single-tenant mode
   > and multi-tenant overlay mode (org "acme" → sensor "crowdstrike").

2. **EC-10-029-MT** (added as a future caveat by the adjudication) — **remove entirely** once
   this story ships. It is no longer a limitation.

3. **Add EC-10-034** (new multi-tenant edge case):

   | ID | Description | Expected Behavior |
   |----|-------------|-------------------|
   | EC-10-034 | `resources/subscribe` for `prismql://schema/acme`; org "acme" is a pure overlay org (acme → crowdstrike sensor); hot-reload modifies the crowdstrike TYPE spec to add a new table | Server sends `notifications/resources/updated` with `uri: "prismql://schema/acme"` within 1 second — because the rebuilt `resolved_spec_map` now contains the new table under the `(acme, crowdstrike)` key, and the notify-diff detects it |

4. **Postcondition §Server-side `subscribe` / `listChanged` support** — item 2: change
   "when `TableRegistry` changes for 'acme'" to "when the resolved schema for 'acme' changes —
   either via `TableRegistry` change (table added/removed) or via hot-reload of the underlying
   sensor spec (ADR-042)."

5. **§Architecture Anchors** — add: `ADR-042 — resolved_spec_map reload-aware redesign: the
   ArcSwap-backed resolved_spec_map guarantees that post-reload `resources/read` returns fresh
   multi-tenant overlay schema; the notify-diff reads from the rebuilt map.`

---

## Test Guidance

### For the test-writer (multi-tenant notify + read path tests)

#### Test 1: Multi-tenant notify — org != sensor, hot-reload triggers notify

**BC:** BC-2.10.013 EC-10-034 + EC-10-029 (multi-tenant variant)
**Type:** Integration test (unit-like — no external services; uses in-process PrismServer)
**File:** `crates/prism-mcp/src/server.rs` `#[cfg(test)]` section, adjacent to existing
`test_BC_2_16_007_reload_config_wires_dispatch_hot_reload_notifications`

**Setup:**
1. Build OrgRegistry: register "acme" (OrgSlug), "globex" (OrgSlug).
2. Build TYPE spec for sensor "crowdstrike" with initial table set: `["crowdstrike_alerts"]`.
3. Build overlay: `customers/acme/crowdstrike.sensor.toml` maps acme → crowdstrike.
4. Build `resolved_spec_map` initial state: `{(acme, crowdstrike): ResolvedSensorSpec{tables: [crowdstrike_alerts]}}`.
5. Wire a `QueryEngine::new_full` with the ArcSwap-backed resolved_spec_map.
6. Build `PrismServer` with the wired QueryEngine + a spec_dir that has `customers/` matching the overlay.
7. Subscribe acme to `prismql://schema/acme`.

**Reload step:**
8. Update the TYPE spec file for "crowdstrike" on disk to add table "crowdstrike_hosts".
9. Call `server.reload_config(peer).await` via JSON-RPC.

**Assertions:**
10. Verify `notifications/resources/updated` received with `uri: "prismql://schema/acme"` (EC-10-034).
11. Verify `notifications/resources/updated` NOT received for `prismql://schema/globex` (EC-10-030 extended).
12. Call `resources/read("prismql://schema/acme")` — assert "crowdstrike_hosts" table present in response (post-reload read freshness).

**SID-1 note:** If the `reload_config` test harness requires external filesystem writes (spec
TOML on disk), the test must use `tempfile::TempDir` for isolation. An `#[ignore]` annotation is
forbidden here — this test has no external service dependency. All dependencies are in-process.

#### Test 2: Multi-tenant read path — prism_describe reflects reload

**BC:** BC-2.10.012 §post-reload freshness in multi-tenant mode
**Type:** Unit test in `crates/prism-mcp/src/server.rs` or dedicated describe test module

**Setup:**
1. Same org/overlay setup as Test 1.
2. Call `prism_describe("acme")` → assert "crowdstrike_alerts" present, "crowdstrike_hosts" absent.
3. Trigger reload (add "crowdstrike_hosts" to the TYPE spec).
4. Call `prism_describe("acme")` again → assert BOTH tables present.

**Key assertion:** The second `prism_describe` call reads from the updated `ArcSwap`-backed map
(ADR-042 read freshness guarantee).

#### Test 3: Single-tenant fallback — rebuild is no-op when no overlay

**BC:** backward compatibility
**Type:** Unit test in `crates/prism-query/src/engine.rs` `#[cfg(test)]`

1. Build engine with `resolved_spec_map: None` (single-tenant mode, `new()` constructor).
2. Call `engine.rebuild_resolved_spec_map(dummy_path, &type_specs, &org_registry)`.
3. Assert `Ok(0)` returned — no-op.
4. Assert `engine.resolved_spec_map()` returns `None` — no side effect.

#### Test 4: In-flight query consistency — snapshot isolation during rebuild

**BC:** ADR-042 in-flight query consistency guarantee
**Type:** Unit test in `crates/prism-query/src/engine.rs`

1. Build engine with initial map containing `{(acme, crowdstrike): spec_A}`.
2. Call `engine.resolved_spec_map()` → hold `old_arc` (simulates in-flight query).
3. Call `engine.rebuild_resolved_spec_map(...)` with updated map containing `{(acme, crowdstrike): spec_B}`.
4. Assert `old_arc` still contains `spec_A` — the held reference is unaffected.
5. Call `engine.resolved_spec_map()` again → assert new arc contains `spec_B`.

---

## Risk Register

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| `OverlayLoader::load_overlays` fails during reload (I/O error or validation failure) | LOW | LOW | Non-fatal: retain prior map (DI-031). Log `reload.overlay_rebuild_failed`. |
| Two HashMap copies in memory during rebuild | LOW | NEGLIGIBLE | O(50) entries typical; Arc is freed when in-flight queries complete. |
| `customers_dir` derivation in `reload_config_core` diverges from `step4` | LOW | MEDIUM | Both paths use `spec_dir.join("customers")`. Implementer must use the same derivation. Extract to a shared helper if desired. |
| `SensorSpec` type mismatch between `ConfigSnapshot::sensor_specs` and `OverlayLoader::load_overlays` parameter | LOW | HIGH | ADR-030 unified type: `ConfigSnapshot::sensor_specs` is `HashMap<String, SensorSpec>` using the unified `spec_parser::SensorSpec` type. `OverlayLoader::load_overlays` takes the same type. Verified in boot.rs line 1188. |
| Notify-diff captures old_tables AFTER the rebuild (wrong snapshot) | MEDIUM | MEDIUM | The old_tables snapshot MUST be captured BEFORE calling `reload_config_core`. Current code does this correctly at line 3218. The rebuild moves inside `reload_config_core`. New_tables snapshot is captured AFTER. This ordering is explicit in the implementation guidance. |

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-06-20 | architect | Initial design. D-1267: human overrides deferral; build it. ArcSwap field, rebuild method, per-crate change spec, BC-2.10.013 update guidance, test guidance. |
