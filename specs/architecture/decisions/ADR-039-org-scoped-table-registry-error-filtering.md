---
document_type: adr
adr_id: "ADR-039"
title: "Org-Scoped Enumeration Filtering — Filter E-QUERY-037 Error Fields and explain_query available_tables to Requesting Org's Registered Tables"
status: ACCEPTED
date: "2026-06-16"
version: "1.1"
producer: architect
subsystems_affected: [SS-11, SS-16]
supersedes: null
superseded_by: null
amends: null
anchor_stories: [S-3.13]
related_adrs: [ADR-006, ADR-022, ADR-029, ADR-034]
related_bcs: [BC-2.11.001]
security_finding: [SEC-001, SEC-003]
security_cwe: CWE-200
locked_decisions: []
inputs:
  - .factory/stories/S-3.13-dynamic-table-availability.md
  - .factory/specs/architecture/decisions/ADR-029-multi-tenant-sensor-endpoint-overrides.md
  - .factory/specs/architecture/decisions/ADR-034-tier3-keyring-resolution-org-id-threading.md
  - .factory/specs/architecture/decisions/ADR-022-production-runtime-wiring.md
  - .factory/specs/architecture/decisions/ADR-006-multi-tenant-dtu-topology.md
input-hash: ""
wiring_deferred_to: null
---

# ADR-039: Org-Scoped Enumeration Filtering

## Status

ACCEPTED 2026-06-16, v1.0. Amended v1.1 2026-06-16 to extend scope to the
`explain_query` / `ExplainResult.available_tables` path (SEC-003).
Human directive: FIX IN-SCOPE NOW. No deferral.

---

## Context

### Covered Information Disclosure Findings

This ADR addresses two related CWE-200 findings in the multi-tenant overlay
deployment. Both use the same mitigation mechanism (§Design Specification).

- **SEC-001 (MEDIUM, CWE-200):** E-QUERY-037 error enumeration leak —
  `available_sensors` / `available_tables` in `PrismError::TableNotAvailable`
  enumerate the GLOBAL registry across org boundaries. **Original scope of v1.0.**
- **SEC-003 (MEDIUM, CWE-200):** `explain_query` enumeration leak —
  `ExplainResult.available_tables` uses the GLOBAL registry, leaking all
  configured sensor table names to the requesting org via the explain path.
  **Added to scope in v1.1** after security re-review identified this as a
  sibling leak site using the same cross-tenant enumeration pattern.

### The Information Disclosure Finding (SEC-001, CWE-200)

`TableRegistry` is a single process-level singleton built at startup from
`ConfigSnapshot.sensor_specs` — a `HashMap<String, SensorSpec>` keyed by
`sensor_id` (the TYPE-level sensor, not per-org). When a query targets an
unregistered table, `check_availability_gate` fires and constructs
`PrismError::TableNotAvailable`. The error populates two fields from the
GLOBAL registry:

- `available_sensors`: result of `registry.registered_sensor_ids()` — ALL sensor
  vendor types configured in the process
- `available_tables`: result of `registry.registered_tables()` — ALL sensor
  tables for ALL configured vendors

In the **single-tenant per-analyst deployment** (one `prism` process per MSSP
analyst, one org), this is correct behavior: the analyst should see everything
configured for their org.

In the **multi-tenant single-process overlay deployment** (one `prism` process
serving N orgs via the per-org overlay system from ADR-029 and
S-CONFIG-MULTI-TENANT-OVERRIDE-001), the error leaks globally-configured
sensor vendor names to any org that queries an unknown table. Org A sending
`SELECT * FROM unknown_table` receives a response listing the sensor vendors
configured for Org B, Org C, etc. — a cross-tenant vendor enumeration leak.

### Why This Is Real in the Multi-Tenant Case

The multi-tenant overlay deployment (S-DEMO-MULTI-TENANT-DTU-001 and
S-DEMO-001 per-org adapters) puts multiple orgs behind one Prism process. Each
org has its own per-org `ResolvedSensorSpec` entries via ADR-029's
`customers/<org>/<sensor>.sensor.toml` overlay system. The `QueryOptions.clients`
field (type `Option<Vec<OrgSlug>>`) carries the requesting org's scope from the
MCP tool call into `execute_inner`. However, the current `TableRegistry` is
populated only from TYPE-level `sensor_specs` and has no concept of which org
registered which sensor.

### Org Scope at Plan Time

The org scope enters the query pipeline via `QueryOptions.clients` — resolved
by `crate::scoping::resolve_clients()` in `execute_inner` AFTER the
`check_table_availability` gate fires. This means the current gate fires with
zero org context, always querying the GLOBAL registry.

---

## Decision Drivers

| Driver | Constraint |
|--------|------------|
| SEC-001 / CWE-200 | E-QUERY-037 must never enumerate sensor vendors belonging to other orgs |
| SEC-003 / CWE-200 | `ExplainResult.available_tables` must never enumerate tables belonging to other orgs |
| ADR-029 / ADR-022 Wiring-not-redesign | Build ON the existing multi-tenant machinery; do not redesign TableRegistry from scratch |
| ADR-006 OrgSlug compile-time enforcement | Org identity flows as `OrgSlug` newtype; solution must use this type |
| Hot-reload (AD-007 arc-swap) | TableRegistry is mutated on hot-reload via `register_sensor` / `deregister_sensor`. Any org-keying must survive hot-reload. |
| ArcSwap read-path performance | `is_registered()` is called on every query plan; no new mutex or blocking on the query path |
| Single-tenant backward compat | Single-tenant deployments (zero overlay files) must continue to work unchanged |
| Registry build cost | Avoid O(N_orgs × N_sensors) memory at singleton construction time when most sensors are SaaS with shared endpoints (CrowdStrike, Cyberint) |

---

## Options Considered

### Option A: Per-Org `TableRegistry` Instances Keyed by `OrgSlug`

**Mechanism:** Replace the single `Arc<TableRegistry>` on `QueryEngine` with a
`HashMap<OrgSlug, Arc<TableRegistry>>` (or `Arc<RwLock<HashMap<…>>>`). Each
org gets its own registry populated from its `ResolvedSensorSpec` entries.
`check_table_availability` receives the requesting `OrgSlug` and dispatches to
the per-org registry.

**Pros:**
- True isolation: each org's registry only ever contains its own tables.
- `registered_tables()` / `registered_sensor_ids()` are trivially safe.
- Hot-reload is clean: per-org add/remove calls the correct per-org registry.

**Cons:**
- Requires threading `OrgSlug` into `check_table_availability` before client
  scope resolution — `clients` is `Option<Vec<OrgSlug>>`, and the gate fires
  before `resolve_clients()`. We do not know the single requesting org at gate
  time without either (a) resolving clients earlier, or (b) defining special
  single-org semantics when `clients` is exactly one entry.
- In the multi-client fan-out case (`clients: None` = all orgs), the gate must
  check WHICH org's registry to use: the union of all orgs? The intersection?
  Neither is trivially correct.
- Multiplies the number of `RwLock<HashSet>` allocations by N_orgs. For 50
  tenants × 4 sensors, this is 50 distinct registry objects, each with its own
  lock. The lock count overhead is small but the hot-reload bookkeeping grows.
- Requires adding `resolved_spec_map` (from ADR-029's overlay loader) as a
  source for populating per-org registries — a non-trivial wiring change to the
  boot path and hot-reload listener.
- The primary query case (single-org MCP client scoped to one org) works cleanly.
  The secondary case (multi-org scheduled query, `clients: None`) requires a
  combined view that is harder to define safely.

**Overall assessment:** Architecturally purer for strict single-org queries but
introduces difficult semantics for the multi-org fan-out case and requires
non-trivial boot-path wiring changes. Disproportionate to the risk surface.

---

### Option B: Single Registry with Filter-at-Error-Construction Time (CHOSEN)

**Mechanism:** Keep the single `Arc<TableRegistry>` unchanged. Modify only the
error-construction site in `check_availability_gate` to accept an
`Option<&[OrgSlug]>` parameter representing the requesting org(s). When org
scope is known, filter `registered_tables()` and `registered_sensor_ids()` to
the intersection of the GLOBAL registry and the tables/sensors accessible to
those orgs.

**The key insight:** The security boundary is exclusively in the ERROR response
— specifically in the `available_sensors` and `available_tables` fields of
`TableNotAvailableDetails`. The EXISTENCE check (`is_registered(table_name)`)
is not the leak: the leak is in what the error ENUMERATES. Option B fixes the
leak at the only place it occurs without redesigning the registry topology.

**Pros:**
- Wiring-not-redesign (ADR-022 §C): the fix is additive — a new parameter and
  a filter at the error-construction site.
- No new lock contention. The filtering is a pure read of the global set plus
  the org's `ResolvedSensorSpec` entries; no new `RwLock` required.
- Single-tenant compatibility: when `org_scope` is `None`, the behavior is
  identical to the current implementation (filter is bypassed).
- Hot-reload remains simple: `register_sensor` / `deregister_sensor` continue
  to operate on the global registry. No per-org cleanup bookkeeping.
- Multi-org fan-out (`clients: None`): use the UNION of all orgs' accessible
  tables — which equals the global registry, so no leak for queries that
  genuinely span all orgs (because multi-org requests are already org-aware).
- The org scope for the error filter comes from `QueryOptions.clients` which is
  already threaded through `execute_inner`. We pass it to
  `check_table_availability` before `resolve_clients()` — a one-line change to
  the call site.

**Cons:**
- The filtering logic requires knowing which tables are accessible to a given org.
  This is derived from `resolved_spec_map` (the `HashMap<ResolvedSpecKey, ResolvedSensorSpec>`
  populated by the ADR-029 overlay loader). `resolved_spec_map` is already wired
  onto `QueryEngine` (field `resolved_spec_map: Option<Arc<HashMap<ResolvedSpecKey, ResolvedSensorSpec>>>`).
  Threading it to the gate requires passing it through two layers: `execute_inner`
  → `check_table_availability` → `check_availability_gate`. This is two additional
  function parameters.
- In single-tenant deployments with no overlay files, `resolved_spec_map` is `None`
  and the filter degrades to the global registry (correct behavior).
- The `did_you_mean` computation must also be filtered to avoid suggesting tables
  belonging to other orgs. The same filter applies.

**Overall assessment:** Fixes the exact leak surface with minimal code change,
builds directly on existing `resolved_spec_map` wiring, preserves all existing
semantics, and maintains backward compatibility for single-tenant deployments.

---

## Decision

**Adopt Option B: Single Registry with Filter-at-Error-Construction Time.**

The reasoning:
1. The leak is exclusively in the ERROR FIELDS (`available_sensors`, `available_tables`,
   `did_you_mean`). The `is_registered()` existence check does not leak org information
   — it is a boolean. Only the enumerated field values in the error response cross the
   org boundary. Option B fixes exactly and only what leaks.
2. ADR-022 §C (wiring-not-redesign) favors additive wiring over structural redesign.
   Option A requires redesigning the registry topology; Option B requires adding one
   optional parameter to an existing function.
3. `resolved_spec_map` is already wired onto `QueryEngine` and threaded into
   `MaterializationContext`. The data needed to compute "which tables is this org
   registered for" is already present in the process — we are not introducing a
   new data source, only a new consumer of an existing one.
4. Hot-reload semantics remain unchanged. The single `TableRegistry` continues to
   hold the global set; the org filter is applied transiently at error-construction
   time without modifying any persistent state.
5. Single-tenant deployments see zero behavioral change: when `org_scope` is `None`
   (no clients restriction) or when `resolved_spec_map` is `None` (no overlays),
   the filter is bypassed and the global registry is returned as before.

---

## Design Specification

### Org-Scope Data Flow at Plan Time

```
MCP tool call
  │  QueryOptions { clients: Option<Vec<OrgSlug>>, ... }
  ▼
QueryEngine::execute_inner(query_str, options)
  │
  ├─ Step 0: alias expansion  →  effective_query
  │
  ├─ Step 1a: check_table_availability(
  │             effective_query,
  │             self.table_registry.as_deref(),
  │             options.clients.as_deref(),          ← NEW parameter
  │             self.resolved_spec_map.as_deref(),   ← NEW parameter
  │           )?
  │             │
  │             └─ registry.check_availability_gate(
  │                  query_str,
  │                  org_scope,          ← forwarded
  │                  resolved_spec_map,  ← forwarded
  │                )?
  │                  │
  │                  └─ If unregistered table found:
  │                       org_visible_sensors = filter_to_org_visible(
  │                         registry.registered_sensor_ids(),
  │                         org_scope,
  │                         resolved_spec_map,
  │                       )
  │                       org_visible_tables = filter_to_org_visible(
  │                         registry.registered_tables(),
  │                         org_scope,
  │                         resolved_spec_map,
  │                       )
  │                       did_you_mean = registry.did_you_mean_filtered(
  │                         table_name,
  │                         &org_visible_tables,
  │                       )
  │                       return Err(PrismError::TableNotAvailable {
  │                         available_sensors: org_visible_sensors.join(", "),
  │                         available_tables:  org_visible_tables.join(", "),
  │                         did_you_mean,
  │                         ...
  │                       })
  │
  └─ Step 1: resolve_clients(options.clients, &self.client_registry)
```

The org scope is passed from `options.clients` — the same `Vec<OrgSlug>` that
`execute_inner` already receives. No new fields are added to `QueryOptions`.

### SEC-003: Org-Scope Data Flow for `explain_query` / `ExplainResult.available_tables`

`ExplainResult.available_tables` is constructed inside `QueryEngine::explain()` by
reading the global `TableRegistry`. In the multi-tenant overlay deployment it
exhibits the same cross-tenant enumeration leak as E-QUERY-037.

**Fix mechanism (same helpers, different call site):**

```
MCP explain_query tool call
  │  ExplainOptions { clients: Option<Vec<OrgSlug>>, resolved_spec_map: Option<Arc<…>>, ... }
  ▼
QueryEngine::explain(query_str, options)
  │
  ├─ Parse + plan query (unchanged)
  │
  ├─ Build ExplainResult.available_tables:
  │    org_visible_tables = filter_to_org_visible_tables(
  │      self.table_registry.as_deref()
  │        .map(|r| r.registered_tables())
  │        .unwrap_or_default(),
  │      options.clients.as_deref(),
  │      options.resolved_spec_map.as_deref(),
  │    )
  │    available_tables = org_visible_tables
  │
  └─ Return ExplainResult { available_tables, ... }
```

**`ExplainOptions` changes (v1.1):**

```rust
pub struct ExplainOptions {
    /// Org scope (forwarded from the MCP tool call's client restriction).
    /// When `Some`, `available_tables` is filtered to this org's sensors only.
    pub clients: Option<Vec<OrgSlug>>,
    /// Per-org resolved sensor map from the ADR-029 overlay loader.
    /// When `None` (single-tenant), filter is bypassed.
    pub resolved_spec_map: Option<Arc<HashMap<ResolvedSpecKey, ResolvedSensorSpec>>>,
    // ... existing fields unchanged
}
```

**`prism-mcp` call-site changes (v1.1):**

Both `ExplainOptions` construction sites in `prism-mcp` must be updated to inject
`clients` and `resolved_spec_map`. The `QueryEngine` already holds
`resolved_spec_map` as a field; `prism-mcp` must pass it through from the engine
context at explain-call construction time, exactly as it does for `execute_inner`
via `QueryOptions`.

**Visibility of filter helpers (v1.1):**

`filter_to_org_visible_sensors` and `filter_to_org_visible_tables` are elevated
from `pub(self)` (private to `table_registry.rs`) to `pub(crate)` so that
`engine.rs` can call them directly when constructing `ExplainResult.available_tables`
without the call flowing through `check_availability_gate`. The `pub(crate)`
boundary keeps them out of the public `prism-query` API surface.

### `filter_to_org_visible` Logic

```
fn filter_to_org_visible(
    global_tables: Vec<String>,
    org_scope: Option<&[OrgSlug]>,
    resolved_spec_map: Option<&HashMap<ResolvedSpecKey, ResolvedSensorSpec>>,
) -> Vec<String>
```

Rules:

1. **`org_scope` is `None` (no client restriction, or single-tenant with no
   overlay system):** Return `global_tables` unchanged. This is the existing
   single-tenant behavior.

2. **`org_scope` is `Some([])` (empty org list):** Return empty `Vec`. No tables
   are visible to an empty scope.

3. **`resolved_spec_map` is `None` (overlay system not configured):**
   Return `global_tables` unchanged. No per-org information exists; the
   deployment is single-tenant by definition.

4. **`org_scope` is `Some(orgs)` and `resolved_spec_map` is `Some(map)`:**
   Build `org_visible_sensor_ids` = union of `sensor_id` values from all
   `ResolvedSensorSpec` entries whose `org_slug` is in `orgs`.
   Then filter `global_tables` to those whose registered sensor (from
   `sensor_by_table` map) is in `org_visible_sensor_ids`.

The filter is `O(N_tables + N_org_specs)` with no allocations beyond the
filtered result `Vec`. Computed only at error-construction time (the non-hot
path — errors are rare).

### `did_you_mean_filtered` variant

Add a method (or extend the existing `did_you_mean`) that accepts a
`&[String]` pre-filtered table list rather than calling `registered_tables()`
internally. The Levenshtein computation must not suggest tables from other orgs.

```rust
pub fn did_you_mean_for_tables(&self, requested: &str, visible_tables: &[String]) -> String
```

This replaces the `did_you_mean` call in `check_availability_gate` when
org-filtered tables are in use.

### Signature Changes

**`table_registry.rs` — `TableRegistry::check_availability_gate`:**
```rust
pub fn check_availability_gate(
    &self,
    query_str: &str,
    org_scope: Option<&[OrgSlug]>,
    resolved_spec_map: Option<&HashMap<ResolvedSpecKey, ResolvedSensorSpec>>,
) -> Result<(), PrismError>
```

**`engine.rs` — `check_table_availability` (module-private function):**
```rust
fn check_table_availability(
    query_str: &str,
    registry: Option<&TableRegistry>,
    org_scope: Option<&[OrgSlug]>,
    resolved_spec_map: Option<&HashMap<(OrgSlug, SensorId), ResolvedSensorSpec>>,
) -> Result<(), PrismError>
```

**Call site in `execute_inner`** (line ~580):
```rust
check_table_availability(
    effective_query,
    self.table_registry.as_deref(),
    options.clients.as_deref(),
    self.resolved_spec_map.as_ref().map(|m| m.as_ref()),
)?;
```

**Call site in `execute_scheduled_inner`** (line ~831):
```rust
check_table_availability(
    query_str,
    self.table_registry.as_deref(),
    clients.as_deref(),
    self.resolved_spec_map.as_ref().map(|m| m.as_ref()),
)?;
```

Both existing call sites receive the new parameters. No third call site exists.

### Sibling-Site Sweep (TD-VSDD-060)

`check_availability_gate` is called from exactly one site:
`engine::check_table_availability`. `check_table_availability` is called from
exactly two sites: `execute_inner` and `execute_scheduled_inner`. Both must be
updated in the same commit. Implementer MUST grep the full `prism-query` crate
before declaring done:
```bash
rg 'check_table_availability\|check_availability_gate' crates/prism-query/
```

---

## Consequences

### Security Consequence

After this fix (v1.0 + v1.1):

- `PrismError::TableNotAvailable` (E-QUERY-037) in the multi-tenant overlay
  deployment enumerates only the sensor vendors and tables registered for the
  requesting org's scope. Cross-tenant vendor enumeration (CWE-200 / SEC-001)
  is eliminated at the error-construction site.
- `ExplainResult.available_tables` in the `explain_query` path is filtered by
  the same `filter_to_org_visible_tables` helper. Cross-tenant table enumeration
  via the explain surface (CWE-200 / SEC-003) is eliminated at the
  `ExplainResult` construction site in `QueryEngine::explain()`.

Both enumeration surfaces now apply the same filtering invariant: in multi-tenant
overlay deployments, org-scoped table/sensor lists reflect only what the
requesting org's overlays have registered.

### Backward Compatibility

Single-tenant deployments (no `customers/` overlays, `resolved_spec_map = None`):
behavior is byte-identical to the pre-fix implementation. The filter short-circuits
at the `resolved_spec_map is None` check and the global tables are returned.

### No Behavioral Change to `is_registered()`

The existence check (`is_registered(table_name)`) is NOT org-scoped. A table
registered for ANY org is "registered" in the global sense. The `is_registered`
function continues to check the GLOBAL registry. This is intentional:
the EXISTENCE of a table type in the process is not a secret — the leak was
in the error ENUMERATION fields. Existence is needed for correct fast-fail
behavior regardless of org scope.

### Hot-Reload Compatibility

The single `TableRegistry` and its `register_sensor` / `deregister_sensor`
methods are unchanged. Hot-reload listeners built in boot.rs continue to call
them exactly as before. No new per-org bookkeeping is introduced.

### Performance

The filter runs only at error-construction time — the unhappy path. The
`is_registered()` hot path is not touched. Cost: one `HashSet` intersection per
error response; negligible at the rate errors occur.

### ArcSwap Compatibility

`resolved_spec_map` is already an `Arc<HashMap<…>>` on `QueryEngine` — loaded
at boot and rebuilt on config hot-reload. The filter receives it as a shared
reference (`Option<&HashMap<…>>`); no new arc-clone is needed on the error path.

---

## Required Implementation Changes

### Files to Modify

| File | Change | Finding |
|------|--------|---------|
| `crates/prism-query/src/table_registry.rs` | Add `did_you_mean_for_tables(requested, visible_tables)` method. Change `check_availability_gate` signature to accept `org_scope: Option<&[OrgSlug]>` and `resolved_spec_map: Option<&HashMap<ResolvedSpecKey, ResolvedSensorSpec>>`. Add `filter_to_org_visible_sensors` and `filter_to_org_visible_tables` as `pub(crate)` functions (elevated from `pub(self)` to allow `engine.rs` explain path to call them directly). Apply filter when both parameters are non-None and org_scope is non-empty. | SEC-001, SEC-003 |
| `crates/prism-query/src/engine.rs` | Change `check_table_availability` signature to pass `org_scope` and `resolved_spec_map`. Update both call sites: `execute_inner` and `execute_scheduled_inner`. In `QueryEngine::explain()`, filter `ExplainResult.available_tables` using `filter_to_org_visible_tables` with `options.clients` and `options.resolved_spec_map`. | SEC-001, SEC-003 |
| `crates/prism-query/src/explain.rs` (or equivalent `ExplainOptions` definition site) | Add `clients: Option<Vec<OrgSlug>>` and `resolved_spec_map: Option<Arc<HashMap<ResolvedSpecKey, ResolvedSensorSpec>>>` fields to `ExplainOptions`. | SEC-003 |
| `crates/prism-mcp/src/tools/explain.rs` (and any sibling explain tool file) | Update both `ExplainOptions` construction sites to inject `clients` (from MCP tool call org scope) and `resolved_spec_map` (from `QueryEngine` context). Sibling-site sweep required: `rg 'ExplainOptions' crates/prism-mcp/`. | SEC-003 |

### New Dependencies

None. `ResolvedSensorSpec`, `ResolvedSpecKey`, and `OrgSlug` are already in
scope via `prism-spec-engine` and `prism-core` which are existing dependencies
of `prism-query`.

### BC Impact

The behavioral contract covering E-QUERY-037 is BC-2.11.001. The fix tightens
the postcondition: in multi-tenant deployments, the `available_sensors` and
`available_tables` fields in the error response are restricted to the requesting
org's configured sensors. This is a security tightening of BC-2.11.001's
postcondition — the interface contract does not change, only the scope of the
enumeration.

**BC change required?** Yes, but narrow. See "BC Impact" section below for the
exact specification handed to product-owner.

---

## BC Impact — Handoff to Product-Owner

The following BC change is required. The architect specifies the exact contract
change; the product-owner authors the BC amendment. The implementer MUST NOT
modify BC content.

**BC to amend:** BC-2.11.001 (query MCP tool — E-QUERY-037 postcondition)

**Current postcondition (paraphrased):** When a table is not found in the
registry, return E-QUERY-037 with `available_sensors` listing all globally
configured sensor vendors and `available_tables` listing all globally registered
table names.

**Required postcondition amendment:** In multi-tenant overlay deployments (where
`resolved_spec_map` is populated from ADR-029 per-org overlays), the
`available_sensors` and `available_tables` fields in E-QUERY-037 MUST be
filtered to the sensors and tables registered for the requesting client org(s)
(`QueryOptions.clients`). In single-tenant deployments (`resolved_spec_map` is
absent or `QueryOptions.clients` is None/unrestricted), the fields list all
globally configured sensors/tables as before.

**The invariant:** `available_sensors` and `available_tables` in E-QUERY-037
MUST NOT enumerate sensors or tables belonging to org(s) other than the
requesting client's org scope.

Product-owner should amend BC-2.11.001 §Postconditions to add an explicit
multi-tenancy scoping clause to the E-QUERY-037 precondition/postcondition.
A new dedicated BC is NOT required — this is an in-place strengthening of an
existing postcondition invariant.

---

## Test Requirements (for Implementer)

The following tests prove the org-scoping property. These are in addition to the
existing S-3.13 test suite.

### New Tests Required

**Location:** `crates/prism-query/src/tests/table_registry_tests.rs`

| Test Name | Assertion |
|-----------|-----------|
| `test_SEC_001_e_query_037_filters_available_sensors_to_requesting_org` | When org A queries an unknown table, `available_sensors` in E-QUERY-037 contains ONLY org A's configured sensors, NOT org B's sensors. Requires a `resolved_spec_map` with two orgs (acme, contoso) each having distinct sensor sets. |
| `test_SEC_001_e_query_037_filters_available_tables_to_requesting_org` | Same setup as above; `available_tables` contains ONLY org A's tables. |
| `test_SEC_001_e_query_037_did_you_mean_filtered_to_requesting_org` | `did_you_mean` suggestion comes only from the requesting org's visible tables — a table typo that matches an org B table does NOT appear in the suggestion. |
| `test_SEC_001_e_query_037_single_tenant_unaffected` | When `org_scope` is `None` and `resolved_spec_map` is `None` (single-tenant mode), `available_sensors` and `available_tables` equal the full global registry. Proves backward compatibility. |
| `test_SEC_001_e_query_037_no_resolved_spec_map_falls_back_to_global` | When `resolved_spec_map` is `None` but `org_scope` is `Some([acme])`, the filter is bypassed (can't compute org visibility without the map) and the global registry is used. Prevents a hard failure when overlay system is not configured. |

**Naming convention:** `test_SEC_001_*` prefix to distinguish SEC-001 regression
tests from the existing functional AC tests.

### Existing Tests Must Not Regress

All 22 existing Red Gate tests in `crates/prism-query/src/tests/table_registry_tests.rs`
and the 3 boot-integration tests in `crates/prism-bin/src/boot.rs` must continue to
pass. These tests operate with `org_scope = None` and `resolved_spec_map = None`
(single-tenant test context), which is the backward-compatible path.

---

## ADR Back-References to Existing ADRs

- **ADR-006** (OrgId/OrgSlug Identity): This fix uses `OrgSlug` exactly as ADR-006
  defines. No new identity type is introduced.
- **ADR-022** (Wiring-not-redesign): The fix adds two new parameters to existing
  functions rather than restructuring `QueryEngine` or `TableRegistry`. This is
  wiring, not redesign, per ADR-022 §C.
- **ADR-029** (Per-Org Overlay System): The `resolved_spec_map` used by the filter
  is the same map produced by ADR-029's overlay loader. This ADR consumes that
  output; it does not change it.
- **ADR-034** (OrgId Threading): ADR-034 established the pattern of threading org
  context (`OrgId`/`OrgSlug`) through function parameters rather than via global
  state. This fix follows the same pattern for the plan-time gate.

---

## Risk Register

| Risk | Severity | Mitigation |
|------|----------|------------|
| Filter over-restricts: org with no overlay sees empty available_sensors | LOW | Rule 3 of filter logic: `resolved_spec_map = None` → bypass filter, return global set. Orgs without overlays have no per-org `ResolvedSensorSpec` entries; their scope is the full TYPE registry. |
| Filter under-restricts: org A can still enumerate TYPE-level sensor names if no overlay is configured for that sensor | ACCEPTED | SaaS sensors (CrowdStrike, Cyberint) have no per-org overlay by design — all orgs share the same endpoint. Enumerating that a process supports CrowdStrike is not a cross-tenant secret in MSSP deployments; sensor TYPE membership is operationally visible. The real leak is per-org-specific sensors with private endpoint overrides. |
| `options.clients` is `None` (multi-org query, system context) — which org's set to show? | LOW | When `org_scope` is `None`, return global registry (Rule 1 of filter). Multi-org system queries legitimately have access to the full table catalog. |
| Forgot to update `execute_scheduled_inner` call site | HIGH | Sibling-site sweep (TD-VSDD-060) required before declaring done. Both call sites are specified in this ADR. |
| Sibling leak: `explain_query` / `ExplainResult.available_tables` (SEC-003) | ADDRESSED (v1.1) | `ExplainOptions` now carries `clients` and `resolved_spec_map`; `QueryEngine::explain()` filters `available_tables` via `filter_to_org_visible_tables`. Both `prism-mcp` `ExplainOptions` construction sites updated. |
| Sibling leak: `list_tables` / `list_sensors` MCP tools (if implemented) | OUT OF SCOPE (document) | If `list_tables` or `list_sensors` MCP tools are added in future stories, they MUST apply the same `filter_to_org_visible_*` helpers. This ADR does not cover tools that do not yet exist; the implementer of any future list-type tool MUST grep for this ADR and apply the same filter. |
| Forgot to update `prism-mcp` `ExplainOptions` construction sites | HIGH (v1.1) | Sibling-site sweep required: `rg 'ExplainOptions' crates/prism-mcp/`. Both construction sites must inject `clients` and `resolved_spec_map` before declaring SEC-003 closed. |

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.0 | 2026-06-16 | architect | Initial ADR for SEC-001 / CWE-200 fix. Human directive: FIX IN-SCOPE NOW. |
| 1.1 | 2026-06-16 | architect | Extended scope to cover SEC-003 (MEDIUM, CWE-200): `explain_query` / `ExplainResult.available_tables` cross-tenant table enumeration leak. Same `filter_to_org_visible_tables` helper applies; `ExplainOptions` gains `clients` + `resolved_spec_map` fields; helpers elevated to `pub(crate)`; both `prism-mcp` `ExplainOptions` construction sites swept. Risk register updated: SEC-003 row marked ADDRESSED; future `list_tables`/`list_sensors` tools noted as OUT OF SCOPE with implementation obligation. |
