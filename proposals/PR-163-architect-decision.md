---
document_type: pr-architect-decision
pr_number: 163
story_id: S-5.01-FOLLOWUP-MCP-BOOT
adjudicator: architect
adjudicated_at: 2026-05-28T00:00:00Z
findings_addressed: [F-PR163-IMP-4, F-PR163-IMP-8]
---

# PR-163 Architect Decision Doc

## F-PR163-IMP-4 — GAP-002-A Stale References

### Investigation Summary

**What GAP-002-A actually is:** The gap between "sensor TOML specs are loaded at boot" and "sensor TOML specs are wired into `AdapterRegistry` so direct sensor adapter fan-out works." Currently all sensor auth flows through WASM `PluginAuthProvider` (ADR-028 §D10), not through `AdapterRegistry`. `AdapterRegistry::new()` is intentionally empty — no direct adapters are populated — because spec-catalog → AdapterRegistry wiring is a prism-bin boot-time concern that must not create a prism-sensors → prism-spec-engine import cycle (ADR-028 §D3).

**What S-3.02-FOLLOWUP-RUNTIME actually closed:** The write-tool registration window (`mark_query_phase_started()` call in `step8_init_query_engine`). It did NOT touch AdapterRegistry or GAP-002-A.

**What S-WAVE5-PREP-01 closed:** The prism-bin chassis (boot steps 1-9 scaffolding). Did not close GAP-002-A.

**Current runtime impact of GAP-002-A:** Zero for the WASM plugin path (CrowdStrike, Cyberint, etc. all route through PluginAuthProvider). The stale comments are documentation debt only. The `ProductionCredentialResolver::resolve` error body in boot.rs is only reachable if something calls the `CredentialResolver` trait path — which nothing currently does because `AdapterRegistry` is empty (no fan-out adapters to dispatch). It is NOT surfaced to MCP callers in production.

**Scope assessment for closing GAP-002-A in this PR:** Closing GAP-002-A requires wiring `parse_spec_directory` output into `AdapterRegistry::register_adapter(sensor_id, Arc<dyn SensorAdapter>)` for each loaded sensor spec. This requires instantiating concrete sensor adapter types (e.g., `CrowdstrikeAdapter`, `ArmisAdapter`) per org, which requires the full credential resolution chain (S-2.07 scope). This is NOT wiring of existing pieces — it requires S-2.07 (per-sensor auth resolution) and S-5.04 (sensor health subsystem) to exist first. Closing GAP-002-A in this PR scope would require importing and expanding the scope into at least two unmerged stories.

**Decision: A2** — Update all 4 stale comment sites to cite the real future story that will close GAP-002-A. Create `S-5.04-SENSOR-HEALTH-ADAPTER-DISPATCH` as the canonical home (S-5.04 is "Sensor Health Subsystem" in STORY-INDEX and is the correct conceptual owner — it depends on S-5.03,S-2.07 and has not yet been implemented). The story already exists in the wave plan.

**Why A2 is a legitimate scope-boundary deferral (not a defer-pattern):**
- Concrete future dependency: S-5.04 depends on S-2.07 (credential resolution per sensor spec) which has not merged.
- The deferral target is a REAL story with a REAL ID in STORY-INDEX.
- The current "deferred to S-3.02-FOLLOWUP-RUNTIME" citations are stale (that story merged without closing this gap) — updating them to the accurate future story is a correction, not a new deferral.
- Human authorization: User Option A "fix ALL findings" authorizes story creation and comment correction in scope.

### Implementer Checklist — F-PR163-IMP-4

**1. `crates/prism-mcp/src/server.rs` line ~2093 (`check_sensor_health` body)**

Change:
```
"sensor health — adapter registry empty (GAP-002-A deferred to S-3.02-FOLLOWUP-RUNTIME)",
```
To:
```
"sensor health — adapter registry empty (GAP-002-A; full sensor adapter dispatch wires in S-5.04-SENSOR-HEALTH-ADAPTER-DISPATCH)",
```

Also update the comment block at lines ~2087-2090:
```rust
// CRIT-4 fix: sensor health check requires live adapter pings (GAP-002-A).
// AdapterRegistry is currently empty — sensor adapters are dispatched via WASM plugins
// (PluginAuthProvider / GAP-002-A deferred to S-3.02-FOLLOWUP-RUNTIME).
```
To:
```rust
// CRIT-4 fix: sensor health check requires live adapter pings (GAP-002-A).
// AdapterRegistry is intentionally empty — all sensor auth routes through WASM
// PluginAuthProvider (ADR-028 §D10). Direct adapter fan-out wires in S-5.04.
```

**2. `crates/prism-mcp/src/server.rs` line ~2134 (`get_diagnostics` body)**

Change:
```
"sensor diagnostics — adapter registry empty (GAP-002-A deferred to S-3.02-FOLLOWUP-RUNTIME)",
```
To:
```
"sensor diagnostics — adapter registry empty (GAP-002-A; full sensor adapter dispatch wires in S-5.04-SENSOR-HEALTH-ADAPTER-DISPATCH)",
```

Also update the comment block at lines ~2130-2132:
```rust
// CRIT-4 fix: sensor diagnostics require live adapter queries (GAP-002-A).
// AdapterRegistry is currently empty — return a structured not-yet-available
// response rather than Internal (architectural gap, not a wiring defect).
```
To:
```rust
// CRIT-4 fix: sensor diagnostics require live adapter queries (GAP-002-A).
// AdapterRegistry is intentionally empty — all sensor auth routes through WASM
// PluginAuthProvider (ADR-028 §D10). Direct adapter wiring is in S-5.04.
```

**3. `crates/prism-bin/src/boot.rs` `step9_start_mcp_server` doc comment (~line 1822)**

Change:
```
/// is deferred to spec-catalog dispatch (GAP-002-A, S-WAVE5-PREP-01/S-3.02-FOLLOWUP-RUNTIME).
```
To:
```
/// is deferred to spec-catalog dispatch (GAP-002-A, S-5.04-SENSOR-HEALTH-ADAPTER-DISPATCH).
```

**4. `crates/prism-bin/src/boot.rs` `ProductionCredentialResolver::resolve` error body (~line 1916)**

Change:
```
"Direct sensor auth for client '{client_id}' sensor '{sensor_id}' requires \
 spec-catalog dispatch (S-3.02-FOLLOWUP-RUNTIME / GAP-002-A). \
 WASM plugin auth uses PluginAuthProvider, not this resolver (ADR-028 §D10)."
```
To:
```
"Direct sensor auth for client '{client_id}' sensor '{sensor_id}' requires \
 spec-catalog adapter dispatch (GAP-002-A; target: S-5.04-SENSOR-HEALTH-ADAPTER-DISPATCH). \
 WASM plugin auth uses PluginAuthProvider, not this resolver (ADR-028 §D10)."
```

**5. `crates/prism-bin/src/boot.rs` `step9_start_mcp_server` inline comment (~line 1864)**

Change:
```
// spec-catalog dispatch (GAP-002-A deferred to S-WAVE5-PREP-01/S-3.02-FOLLOWUP-RUNTIME).
```
To:
```
// spec-catalog dispatch (GAP-002-A; target S-5.04-SENSOR-HEALTH-ADAPTER-DISPATCH).
```

**6. `crates/prism-bin/src/boot.rs` `ClientRegistry` comment (~line 1875)**

Change:
```
// Full ClientRegistry population from OrgRegistry requires adding list_slugs() to
// prism-core::OrgRegistry (S-MULTI-TENANT-002 scope).
```
To:
```
// Full ClientRegistry population from OrgRegistry requires list_slugs() on
// prism-core::OrgRegistry (added in this PR — see F-PR163-IMP-8 fix).
// Wire here after OrgRegistry gains the accessor: see implementer checklist F-PR163-IMP-8 item 7.
```

Note: This comment will be superseded by the IMP-8 fix below which actually adds the wiring.

---

## F-PR163-IMP-8 — AliasStore CRUD Path Bypasses Client Allowlist + Capability Gate

### Investigation Summary

**S-MULTI-TENANT-002 status:** Does NOT exist. No story file, no STORY-INDEX entry matching `S-MULTI-TENANT-002`. This is a phantom story ID — a Canonical Principle Rule 3 violation (deferral with no concrete real story). The comment must be corrected.

**OrgRegistry slug iteration:** `OrgRegistry` currently has `new()`, `resolve()`, `slug_exists()`, `slug_for()`, `register()`, `len()`, `is_empty()`. No `list_slugs()` method. The backing store is `BiMap<OrgSlug, OrgId>` wrapped in `RwLock`. Adding `list_slugs() -> Vec<String>` is a single-method addition with no architectural ramification — the BiMap's left-key iterator yields `OrgSlug`, and `OrgSlug::as_str()` (or `to_string()`) gives the raw slug string. This is "wiring, not redesign" per ADR-022 §C.

**OrgRegistry accessibility from PrismServer:** `PrismServer` currently does NOT hold `Arc<OrgRegistry>`. `QueryEngine` holds `pub(crate) org_registry: Option<Arc<OrgRegistry>>` — not accessible from `prism-mcp`. The correct fix is to add `org_registry: Option<Arc<OrgRegistry>>` to `PrismServer` (same pattern as `config_manager`, `write_executor`, `audit_writer`) and wire it from `step9_start_mcp_server` via `with_deps()`.

**FeatureFlagEvaluator accessibility from PrismServer:** `WriteExecutor::feature_flags()` is `pub` and returns `&Arc<FeatureFlagEvaluator>`. `PrismServer` already holds `write_executor: Option<Arc<WriteExecutor>>`. Therefore `self.write_executor.as_ref().map(|we| we.feature_flags())` gives the evaluator in any tool handler. No new field needed.

**alias.write compile gate:** `prism_query::alias_capability::alias_write_compile_gate()` is the canonical function. It returns `CompileTimeGate::Present` when the `alias-write` Cargo feature is compiled in. Import path: `prism_query::alias_capability::alias_write_compile_gate`.

**valid_client_ids for list_aliases:** `list_aliases(input, store, valid_client_ids)` uses `valid_client_ids` only to validate per-client scope requests — it does NOT gate the global list operation. With `valid_client_ids = &[]`, a request for `scope = Some("client:acme-corp")` returns `ConfigNotFound` (no such client). When `org_registry.list_slugs()` is wired, scope filtering will correctly admit registered client slugs. This is the right fix.

**Decision: Wire in-scope.** All required pieces exist. The fix requires:
1. Add `list_slugs()` to `OrgRegistry` in `prism-core` (new pub method, 4 lines)
2. Add `org_registry: Option<Arc<OrgRegistry>>` field to `PrismServer`
3. Update `PrismServer::with_deps()` to accept `Arc<OrgRegistry>`
4. Update `PrismServer::new()` (test constructor) to set `org_registry: None`
5. Update `step9_start_mcp_server` to pass `Arc::clone(&ctx.org_registry)` to `with_deps()`
6. In `create_alias`, `delete_alias`: pass `valid_client_ids` (from `org_registry.list_slugs()`) and `capability_gate` (from `write_executor.feature_flags()`)
7. In `list_aliases`: pass `valid_client_ids` (from `org_registry.list_slugs()`)
8. Update the `ClientRegistry` construction in `step9_start_mcp_server` to use the same slug list

This is pure wiring — no new crates, no ADR decisions, no architectural redesign.

### Implementer Checklist — F-PR163-IMP-8

**1. `crates/prism-core/src/org_registry.rs` — Add `list_slugs()` method**

After the `is_empty()` method (line ~184), add:

```rust
/// Return all registered org slugs as raw strings.
///
/// Used by prism-bin boot step 9 to populate `ClientRegistry` and `valid_client_ids`
/// for alias capability gating (F-PR163-IMP-8). Pure read; no I/O.
pub fn list_slugs(&self) -> Vec<String> {
    self.inner
        .read()
        .expect("OrgRegistry RwLock poisoned")
        .left_values()
        .map(|slug| slug.as_str().to_owned())
        .collect()
}
```

Verify `OrgSlug` has `as_str(&self) -> &str`. If it uses `Deref<Target = str>` instead, use `slug.as_ref()` or `slug.to_string()` — check `prism-core/src/tenant.rs` for the correct method. The BiMap crate's `left_values()` returns an iterator over left-type values (`&OrgSlug`).

**2. `crates/prism-mcp/src/server.rs` — Add `org_registry` field to `PrismServer`**

In the `PrismServer` struct (after the `alias_store` field, ~line 94):

```rust
/// OrgRegistry — wired in production for client allowlist validation (F-PR163-IMP-8).
///
/// Provides the authoritative list of registered org slugs for:
/// - `valid_client_ids` passed to alias CRUD gated functions (SEC-005 / BC-2.11.008)
/// - `ClientRegistry` construction for multi-tenant client scoping
org_registry: Option<Arc<prism_core::OrgRegistry>>,
```

**3. `crates/prism-mcp/src/server.rs` — Update `PrismServer::new()` (test constructor)**

Add to the struct literal in `new()`:
```rust
org_registry: None,
```

**4. `crates/prism-mcp/src/server.rs` — Update `PrismServer::with_deps()` signature and body**

Add parameter:
```rust
org_registry: Arc<prism_core::OrgRegistry>,
```

Add to body struct literal:
```rust
org_registry: Some(org_registry),
```

Update the doc comment to list `org_registry` in the Parameters section.

**5. `crates/prism-mcp/src/server.rs` — Add helper method for slug list**

Add a private helper to `PrismServer` (after `with_deps`, before `serve_stdio`):

```rust
/// Build the list of valid client slugs from the wired OrgRegistry.
///
/// Returns `Vec<String>` of all registered org slugs, or an empty vec if
/// OrgRegistry is not wired (test-only path). Used by alias CRUD tools for
/// valid_client_ids enforcement (F-PR163-IMP-8 / SEC-005 / BC-2.11.008).
fn valid_client_ids(&self) -> Vec<String> {
    self.org_registry
        .as_ref()
        .map(|r| r.list_slugs())
        .unwrap_or_default()
}
```

**6. `crates/prism-mcp/src/server.rs` — Fix `create_alias` handler**

In the `create_alias` handler, find the `create_alias_with_clients_gated` call (~line 1377). Replace:
```rust
let result = prism_query::alias_tools::create_alias_with_clients_gated(
    input,
    &mut store,
    &ocsf_reserved,
    &[], // valid_client_ids: empty — client validation deferred to S-MULTI-TENANT-002
    None,
    token_store,
)
```
With:
```rust
let valid_ids = self.valid_client_ids();
let capability_gate = self.write_executor.as_ref().map(|we| {
    (
        we.feature_flags().as_ref(),
        prism_query::alias_capability::alias_write_compile_gate(),
    )
});
let result = prism_query::alias_tools::create_alias_with_clients_gated(
    input,
    &mut store,
    &ocsf_reserved,
    &valid_ids,
    capability_gate,
    token_store,
)
```

**7. `crates/prism-mcp/src/server.rs` — Fix `list_aliases` handler**

In the `list_aliases` handler, find the `list_aliases` call (~line 1456). Replace:
```rust
let result =
    prism_query::alias_tools::list_aliases(input, &store, &[]).map_err(to_error_data)?;
```
With:
```rust
let valid_ids = self.valid_client_ids();
let result =
    prism_query::alias_tools::list_aliases(input, &store, &valid_ids).map_err(to_error_data)?;
```

**8. `crates/prism-mcp/src/server.rs` — Fix `delete_alias` handler**

In the `delete_alias` handler, find the `delete_alias_gated` call (~line 1541). Replace:
```rust
let result =
    prism_query::alias_tools::delete_alias_gated(input, &mut store, token_store, &[], None)
        .map_err(to_error_data)?;
```
With:
```rust
let valid_ids = self.valid_client_ids();
let capability_gate = self.write_executor.as_ref().map(|we| {
    (
        we.feature_flags().as_ref(),
        prism_query::alias_capability::alias_write_compile_gate(),
    )
});
let result =
    prism_query::alias_tools::delete_alias_gated(
        input,
        &mut store,
        token_store,
        &valid_ids,
        capability_gate,
    )
    .map_err(to_error_data)?;
```

**9. `crates/prism-bin/src/boot.rs` — Update `step9_start_mcp_server` call to `with_deps()`**

In `step9_start_mcp_server`, find the `PrismServer::with_deps(...)` call and add `Arc::clone(&org_registry)` as the new parameter (after `alias_store`).

**10. `crates/prism-bin/src/boot.rs` — Wire `ClientRegistry` from org slugs**

Replace the current empty `ClientRegistry` construction (~line 1876):
```rust
// ClientRegistry: currently empty (no public OrgRegistry iteration API).
// ...
let client_registry = Arc::new(ClientRegistry::new(vec![]));
```
With:
```rust
// ClientRegistry: populated from OrgRegistry slug list (F-PR163-IMP-8).
// OrgRegistry::list_slugs() added to prism-core in this fix burst.
let client_slugs = org_registry.list_slugs();
let client_registry = Arc::new(ClientRegistry::new(client_slugs));
```

**11. Imports to add in `server.rs`**

Add `prism_query::alias_capability` to the import block in `server.rs` if not already present:
```rust
use prism_query::alias_capability;
```
Or use the fully-qualified path as shown in checklist items 6 and 8 above.

**12. Add `prism-core` crate to `prism-mcp/Cargo.toml` if not already a direct dep**

Check `prism-mcp/Cargo.toml` for `prism-core` dependency. The struct field `Arc<prism_core::OrgRegistry>` requires it. If it is already a transitive dep, make it explicit:
```toml
prism-core = { path = "../prism-core" }
```

**13. Fix confirm_action two-step alias path**

The `confirm_action` handler also calls `create_alias_with_clients_gated` (second-call path, ~line 1919) and `delete_alias_gated` (~line 1997). Apply the same `valid_ids` and `capability_gate` wiring to those call sites. Use `self.valid_client_ids()` and the same `write_executor.feature_flags()` accessor.

### Compile verification

After making these changes, verify:
```bash
just iter prism-core
just iter prism-mcp
just iter prism-bin
```

Then workspace gate before PR push:
```bash
just check
```

---

## Risk Disposition

No new tech-debt-register entries are created by this decision. Both findings are fixed in-scope.

**Architectural note — GAP-002-A remains open:** GAP-002-A (spec-catalog → AdapterRegistry dispatch) is a legitimate future-story deferral correctly anchored to S-5.04-SENSOR-HEALTH-ADAPTER-DISPATCH. It is NOT a tech-debt entry. The comments now cite the correct target story. When S-5.04 ships, the stale-comment pattern will not recur because `AdapterRegistry::new()` placeholder is replaced by populated adapters and `check_sensor_health`/`get_diagnostics` will have real implementations.

**Security note — alias capability gate:** Wiring `FeatureFlagEvaluator` to alias CRUD closes the capability bypass documented in F-PR163-IMP-8. After this fix, `create_alias` and `delete_alias` in `client:<id>` scope will correctly require the `alias.write` capability in the per-client TOML config. `list_aliases` will correctly reject unregistered client IDs. This is a security correctness fix, not a behavioral change for properly-configured deployments.
