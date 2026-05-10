---
document_type: audit
audit_id: PLUGIN-AUDIT-001
date: 2026-05-10
auditor: codebase-analyzer
version: "1.0"
producer: codebase-analyzer
related_tasks: [94, 95, 44, 91]
status: complete
mandate_origin: "user 2026-05-10 directive: 'we arent suppose to have anything built it, everything uses the plugin system. We need to do a full audit'"
inputs:
  - "crates/prism-sensors/src/auth/mod.rs"
  - "crates/prism-sensors/src/lib.rs"
  - "crates/prism-spec-engine/src/custom_adapter.rs"
  - "crates/prism-spec-engine/src/plugin/mod.rs"
  - "crates/prism-core/src/types.rs"
  - "crates/prism-spec-engine/src/spec_parser.rs"
  - "crates/prism-spec-engine/src/pipeline.rs"
  - "crates/prism-bin/src/boot.rs"
  - "crates/prism-query/src/explain.rs"
  - "crates/prism-query/src/write_dispatch.rs"
  - "crates/prism-query/src/virtual_fields.rs"
  - "crates/prism-query/src/invalidation.rs"
  - "crates/prism-query/src/materialization.rs"
input-hash: "[live-state]"
---

# Plugin-Only Architecture Audit (PLUGIN-AUDIT-001)

## Executive Summary

This audit catalogues 21 violations of the plugin-only sensor architecture mandate across 10 crates. Three root causes dominate: (1) the `SensorType` enum in `prism-core` is closed — every new sensor requires a source-code change, making the enum the architectural keystone that prevents external extensibility; (2) four sensor-named Rust files (`crowdstrike`, `armis`, `claroty`, `cyberint`) in `prism-sensors/src/auth/` encode sensor-specific authentication logic in compiled-in Rust rather than TOML/WASM specs, and the `SensorAuth` trait is actively sealed against external implementors; and (3) five sites in `prism-query` dispatch by sensor name via `match` arms rather than via spec-catalog lookup, meaning every new sensor also requires query-engine changes. The user-decided migration approach addresses all three root causes: the `SensorType` enum becomes an open `SensorId(Arc<str>)` newtype, OCSF mapping moves to a hybrid TOML-column-level (80%) plus in-repo `.prx` WASM transformer plugin (20%) model, the four Rust auth modules are replaced by reverse-engineered TOML sensor specs with DTU-parity tests, and the CrowdStrike OAuth2 refresh-on-401 flow ships as an in-repo signed WASM plugin loaded by `PluginRuntime` at boot. The migration requires 13 stories across 3 waves (~100–140 story points) and is rated HIGH risk due to the cross-cutting nature of the `SensorType` keystone change.

## Architectural Intent vs Reality

### Declared Intent

`crates/prism-spec-engine/src/custom_adapter.rs:1-10` opens with the claim that the design is "~80% TOML, ~20% custom adapter, and the 4 initial sensors are pure TOML." The document asserts external parties can author sensor specs without modifying Prism source code.

`.factory/specs/architecture/sensor-adapters.md:469-487` states the adapter layer goal as "no compiled-in sensor-specific Rust code" and describes the TOML spec + WASM plugin path as the canonical authorship surface.

### Counter-Evidence

`crates/prism-sensors/src/auth/mod.rs:18-26` declares four public submodules by sensor name:

```rust
pub mod crowdstrike;
pub mod armis;
pub mod claroty;
pub mod cyberint;
```

`crates/prism-sensors/src/lib.rs:181-194` shows `init_registry_for_org` constructing all four sensors by their concrete Rust types — there is no spec-catalog lookup, no TOML loading, and no plugin dispatch in this path.

The auth module actively prevents external implementors via a `private::Sealed` marker trait at `crates/prism-sensors/src/auth/mod.rs:32-39, 55-62`. The sealed trait means anyone outside this crate cannot implement `SensorAuth`, contradicting the "external TOML authorship" claim.

`crates/prism-spec-engine/src/custom_adapter.rs:57-58` defines a `CustomAuth` placeholder that duplicates `SensorAuth` — an admission that the two trait surfaces have diverged from the claimed single model.

## Findings (21 total — table of contents)

| ID | Tier | Title | Location |
|----|------|-------|----------|
| CRIT-1 | CRIT | Closed SensorType enum keystone | crates/prism-core/src/types.rs:70-89 |
| CRIT-2 | CRIT | prism-query::explain dual hardcoded match arms | crates/prism-query/src/explain.rs:660-665, 1046-1054 |
| CRIT-3 | CRIT | prism-query::write_dispatch hardcoded sensor lookup | crates/prism-query/src/write_dispatch.rs:280-283 |
| CRIT-4 | CRIT | prism-query::virtual_fields duplicate sensor name source | crates/prism-query/src/virtual_fields.rs:2099-2105 |
| CRIT-5 | CRIT | prism-query::invalidation hardcoded per-sensor cache map | crates/prism-query/src/invalidation.rs:2162-2197 |
| CRIT-6 | CRIT | materialization::sensor_type_from_table_name closed prefix dispatch | crates/prism-query/src/materialization.rs:781-794 |
| CRIT-7 | CRIT | SensorAdapter trait returns closed-set sensor identity | crates/prism-sensors/src/adapter.rs (sensor_type method) |
| CRIT-8 | CRIT | prism-ocsf 4 hardcoded per-sensor mapper modules | crates/prism-ocsf/src/mappers/{4 sensors}.rs |
| HIGH-1 | HIGH | prism-sensors public API exposes 4 concrete adapter types | crates/prism-sensors/src/lib.rs:56-60 |
| HIGH-2 | HIGH | SensorAuth sealed via private::Sealed; placeholder duplicate trait in spec-engine | crates/prism-sensors/src/auth/mod.rs:32-39, 55-62; crates/prism-spec-engine/src/custom_adapter.rs:57-58 |
| HIGH-3 | HIGH | CustomAdapterRegistry + PluginRuntime are dead code | crates/prism-spec-engine/src/custom_adapter.rs:64-152; crates/prism-bin/src/boot.rs:805-855 |
| HIGH-4 | HIGH | TOML grammar insufficient for CrowdStrike flow + production specs are stubs | crates/prism-spec-engine/src/spec_parser.rs:24-77; sensors/{4}.sensor.toml |
| HIGH-5 | HIGH | PipelineExecutor::execute is a stub returning canned empty values | crates/prism-spec-engine/src/pipeline.rs:54-66 |
| MED-1 | MED | boot.rs step 4 wires only parse_spec_directory; no plugin/custom-adapter wiring | crates/prism-bin/src/boot.rs:495-530, 805-855 |
| MED-2 | MED | AdapterRegistry::register requires SensorType return | crates/prism-sensors/src/registry.rs:40, 63-66 |
| MED-3 | MED | init_registry/init_registry_for_org parameter list hardcodes 4-sensor model | crates/prism-sensors/src/lib.rs:124-145, 166-197 |
| LOW-1 | LOW | Architecture docs name 4 sensors directly | .factory/specs/architecture/module-decomposition.md; .factory/specs/architecture/decisions/ADR-022-production-runtime-wiring.md |
| LOW-2 | LOW | 4 stories embed wrong architecture as production deliverables | .factory/stories/{S-2.06, S-2.07, W3-FIX-S307-001, S-3.1.06-ImplPhase}.md |
| LOW-3 | LOW | BC catalog likely names sensor-specific behaviors (REQUIRES_VERIFICATION) | .factory/specs/behavioral-contracts/ |
| OBS-1 | OBS | 10+ test files import concrete adapter types | crates/prism-sensors/tests/, crates/prism-query/tests/execute_integration_tests.rs:3517 |
| OBS-2 | OBS | DTU clone crates correctly named — TEST-ONLY ACCEPTABLE | crates/prism-dtu-{4 sensors}/ |

## Findings (full detail)

### CRIT-1 — Closed SensorType enum keystone

**Location:** `crates/prism-core/src/types.rs:70-89`

**Symptom:** `SensorType` is a closed Rust enum with one variant per sensor:

```rust
pub enum SensorType {
    CrowdStrike,
    Armis,
    Claroty,
    Cyberint,
}
```

**Violation:** Any code that `match`es on `SensorType` must be recompiled when a new sensor is added. External parties cannot extend the enum. This is the architectural keystone that makes every other hardcoding finding load-bearing — change this enum and all downstream `match` arms must change in lockstep.

**Evidence:** The enum is `match`-dispatched in at least 7 downstream locations (CRIT-2 through CRIT-6, HIGH-1, MED-2) across 4 crates.

**Migration:** Replace `SensorType` with `SensorId(Arc<str>)` — an open newtype wrapping the sensor's canonical name string. All existing `match SensorType::X` arms become spec-catalog lookups using `SensorId` as the key. This is a 15-file change touching prism-core, prism-sensors, prism-query, prism-ocsf, and test helpers.

**Blast radius:** HIGH — keystone change; every downstream match arm must be ported in the same atomic commit or feature-flagged dual-definition stage.

---

### CRIT-2 — prism-query::explain dual hardcoded match arms

**Location:** `crates/prism-query/src/explain.rs:660-665, 1046-1054`

**Symptom:** Two separate `match sensor_type { SensorType::CrowdStrike => ..., ... }` blocks in the explain module, one for query plan annotation and one for schema description.

**Violation:** A hypothetical fifth sensor added in TOML would produce no explain output — explain would silently omit it or panic at a non-exhaustive match.

**Evidence:** The explain module has no spec-catalog integration path; it derives sensor labels directly from the closed enum.

**Migration:** Replace both match blocks with spec-catalog lookups: `SpecCatalog::describe_sensor(sensor_id)` → `SensorExplainSpec { table_names, schema_summary }`. The catalog is already loaded at query-engine initialization time; threading the reference requires one constructor change.

**Blast radius:** MED — contained within explain.rs; no cross-crate impact beyond SensorType→SensorId rename.

---

### CRIT-3 — prism-query::write_dispatch hardcoded sensor lookup

**Location:** `crates/prism-query/src/write_dispatch.rs:280-283`

**Symptom:** Write dispatch selects the concrete adapter instance by matching the sensor name string against a hardcoded list:

```rust
match sensor_name.as_str() {
    "crowdstrike" => ...,
    "armis" => ...,
    ...
}
```

**Violation:** This is the exact site where Bundle B Phase B-2 work (W3-FIX-S307-001's `match endpoint.pipe_verb { ... }`) was about to entrench the violation further. Any write operation against a sensor not in this list silently fails or panics.

**Evidence:** `crates/prism-query/src/write_dispatch.rs:280-283` — the match arm is the only dispatch path for sensor write operations.

**Migration:** Replace the match arm with a `WriteEndpointSpec` lookup via spec-catalog: `SpecCatalog::write_endpoint(sensor_id, verb)` returns a declarative `WriteEndpointSpec` that `PipelineExecutor::execute_write` consumes. No compiled-in sensor knowledge required.

**Blast radius:** HIGH — this is the active story entanglement point; W3-FIX-S307-001 must be re-scoped from "add concrete write overrides" to "implement declarative write-endpoint spec execution."

---

### CRIT-4 — prism-query::virtual_fields duplicate sensor name source

**Location:** `crates/prism-query/src/virtual_fields.rs:2099-2105`

**Symptom:** Virtual field expansion uses a parallel hardcoded sensor list to determine which fields are valid for a given table prefix. The list is not derived from the spec-catalog.

**Violation:** A TOML-authored sensor's virtual fields are invisible to the query engine. Queries referencing virtual fields on the new sensor would return "unknown field" errors rather than resolving to the spec-defined field mapping.

**Evidence:** `crates/prism-query/src/virtual_fields.rs:2099-2105` — hardcoded sensor→virtual-field map.

**Migration:** Load virtual field definitions from `SensorSpec.columns[].virtual_field_aliases` (a proposed TOML extension) at spec-catalog load time. Virtual field expansion consults the catalog by `SensorId`.

**Blast radius:** MED — contained within prism-query; requires TOML grammar extension (HIGH-4 prerequisite).

---

### CRIT-5 — prism-query::invalidation hardcoded per-sensor cache map

**Location:** `crates/prism-query/src/invalidation.rs:2162-2197`

**Symptom:** Cache invalidation rules are stored in a `HashMap<SensorType, InvalidationPolicy>` populated by hardcoded sensor entries at initialization time.

**Violation:** A TOML-authored sensor has no invalidation policy — its cache never expires or expires on incorrect triggers.

**Evidence:** `crates/prism-query/src/invalidation.rs:2162-2197` — the initialization loop iterates `[SensorType::CrowdStrike, SensorType::Armis, SensorType::Claroty, SensorType::Cyberint]`.

**Migration:** Derive invalidation policy from `SensorSpec.fetch_step.cache_ttl_secs` (a proposed TOML field) at spec-catalog load. Invalidation map keys become `SensorId` strings.

**Blast radius:** MED — contained within invalidation.rs; requires TOML grammar extension (HIGH-4 prerequisite).

---

### CRIT-6 — materialization::sensor_type_from_table_name closed prefix dispatch

**Location:** `crates/prism-query/src/materialization.rs:781-794`

**Symptom:** `sensor_type_from_table_name` dispatches on hardcoded table name prefixes to return a `SensorType`:

```rust
fn sensor_type_from_table_name(name: &str) -> Option<SensorType> {
    if name.starts_with("cs_") { Some(SensorType::CrowdStrike) }
    else if name.starts_with("armis_") { Some(SensorType::Armis) }
    ...
}
```

**Violation:** A TOML-authored sensor whose table names use a novel prefix (e.g., `acme_`) returns `None` — its tables are invisible to the query engine's materialization path.

**Evidence:** `crates/prism-query/src/materialization.rs:781-794` — the prefix dispatch is the only resolution mechanism.

**Migration:** Replace prefix dispatch with spec-catalog lookup: `SpecCatalog::sensor_for_table(table_name)` returns `Option<SensorId>` by consulting the spec's `fetch_step.table_name` field. The catalog builds a `HashMap<String, SensorId>` at load time from all registered specs.

**Blast radius:** HIGH — materialization is the hot path for every SELECT query; this change requires careful correctness testing.

---

### CRIT-7 — SensorAdapter trait returns closed-set sensor identity

**Location:** `crates/prism-sensors/src/adapter.rs` (sensor_type method)

**Symptom:** The `SensorAdapter` trait includes a `sensor_type(&self) -> SensorType` method. Every implementor must return one of the four hardcoded enum variants.

**Violation:** External adapter implementors (the claimed extensibility path) cannot return a valid `SensorType` for a novel sensor — they must return one of the four existing variants, which is semantically incorrect.

**Evidence:** The adapter trait's `sensor_type` return type is the closed `SensorType` enum; there is no `SensorId` escape hatch.

**Migration:** Change the return type of `sensor_type` to `SensorId` (the open newtype from CRIT-1). Concrete adapters return `SensorId::from("crowdstrike")` etc. TOML-loaded adapters return the name from the `SensorSpec.sensor_id` field.

**Blast radius:** MED — contained to adapter.rs + four concrete adapter implementations; follows from CRIT-1.

---

### CRIT-8 — prism-ocsf 4 hardcoded per-sensor mapper modules

**Location:** `crates/prism-ocsf/src/mappers/{crowdstrike,armis,claroty,cyberint}.rs`

**Symptom:** Four Rust files each contain OCSF mapping logic specific to one sensor. The mapper dispatch is a match on `SensorType`.

**Violation:** A TOML-authored sensor has no OCSF mapper. Normalization produces empty or malformed OCSF output for novel sensors.

**Evidence:** `crates/prism-ocsf/src/mappers/` contains exactly four files, one per sensor. The dispatch in `mod.rs` is a closed match.

**Migration:** User-decided approach: hybrid TOML + WASM. Each `SensorSpec.columns[N].ocsf_field` carries the OCSF field name for simple direct mappings (80% of cases). Complex transforms (multi-field combos, timestamp format normalization, nested object flattening) ship as in-repo `.prx` WASM transformer plugins. A `SpecDrivenMapper` in prism-ocsf consults the spec-catalog for column-level ocsf_field annotations and dispatches to WASM plugins for the remainder. The four in-tree Rust mapper modules are deleted.

**Blast radius:** HIGH — touches prism-ocsf, prism-spec-engine (TOML grammar), and requires the WASM transformer plugin build pipeline (HIGH-4, S-PLUGIN-PREREQ-D prerequisites).

---

### HIGH-1 — prism-sensors public API exposes 4 concrete adapter types

**Location:** `crates/prism-sensors/src/lib.rs:56-60`

**Symptom:** The crate's public API re-exports four concrete adapter structs:

```rust
pub use crowdstrike::CrowdStrikeAdapter;
pub use armis::ArmisAdapter;
pub use claroty::ClarotyAdapter;
pub use cyberint::CyberintAdapter;
```

**Violation:** Any downstream crate that imports `prism_sensors::CrowdStrikeAdapter` creates a compile-time dependency on the concrete adapter. This makes the "plugin-only" surface claim false at the type level.

**Evidence:** `crates/prism-sensors/src/lib.rs:56-60` — four pub use re-exports at crate root.

**Migration:** Remove the four re-exports after all downstream consumers are migrated to spec-catalog dispatch. The compile-fail perimeter test (Wave 2/F story) asserts `use prism_sensors::CrowdStrikeAdapter` does not compile post-migration.

**Blast radius:** MED — requires downstream consumer sweep before removal is safe.

---

### HIGH-2 — SensorAuth sealed; placeholder duplicate in spec-engine

**Location:** `crates/prism-sensors/src/auth/mod.rs:32-39, 55-62`; `crates/prism-spec-engine/src/custom_adapter.rs:57-58`

**Symptom:** The `SensorAuth` trait is sealed via `private::Sealed` at lines 32-39 and 55-62 in `auth/mod.rs`. A `CustomAuth` placeholder duplicate exists in spec-engine's custom_adapter.rs at lines 57-58 — evidence that the spec-engine needed a parallel auth surface because it could not implement the sealed one.

**Violation:** External TOML-authored sensors cannot provide their own authentication strategy. The sealed trait actively prevents spec-driven auth extension. The duplicate placeholder confirms the architectural divergence.

**Evidence:** The `private::Sealed` marker at lines 32-39 restricts `SensorAuth` implementors to within `prism-sensors`. The `CustomAuth` stub at custom_adapter.rs:57-58 has zero callers.

**Migration (user-decided):** Un-seal `SensorAuth` entirely. Remove the `private::Sealed` marker. Cross-sensor auth-composition prevention moves from compile-time to runtime: the spec-catalog validation layer rejects specs that claim unsupported auth combinations. The `CustomAuth` duplicate is deleted as part of retiring the `CustomAdapter` Rust trait (Wave 0/E).

**Blast radius:** LOW — un-sealing is additive. No existing code breaks; new code can now implement `SensorAuth`.

---

### HIGH-3 — CustomAdapterRegistry + PluginRuntime are dead code

**Location:** `crates/prism-spec-engine/src/custom_adapter.rs:64-152`; `crates/prism-bin/src/boot.rs:805-855`

**Symptom:** `CustomAdapterRegistry` at custom_adapter.rs:64-152 defines a registration mechanism for custom adapters. `PluginRuntime` at custom_adapter.rs:100-140 wraps WASM plugin loading. Both are instantiated in boot.rs steps 7-8 (lines 805-855) but nothing registers a plugin or adapter via these paths in production.

**Violation:** The plugin extensibility mechanism exists in code but is never invoked. The boot sequence creates the registry, then does nothing with it. External sensor TOMLs are never loaded through this path.

**Evidence:** `crates/prism-bin/src/boot.rs:805-855` — the `custom_adapter_registry` and `plugin_runtime` locals are created and immediately dropped at the end of the boot function scope. No step wires them into the query engine or sensor registry.

**Migration (user-decided):** The `CustomAdapterRegistry` and `CustomAdapter` Rust trait are RETIRED. `PluginRuntime` is kept but promoted: it becomes the sole WASM plugin loader, wired into boot.rs step 7 (Wave 0/D). In-repo `.prx` plugins (CrowdStrike OAuth2 refresh, OCSF complex transformers) are loaded via `PluginRuntime` at boot. External spec-authored TOML sensors do not need a `CustomAdapterRegistry` — the spec-catalog IS the registry.

**Blast radius:** MED — retiring CustomAdapter Rust trait removes 158 lines from custom_adapter.rs; requires boot.rs step 7 rewiring.

---

### HIGH-4 — TOML grammar insufficient for CrowdStrike flow + production specs are stubs

**Location:** `crates/prism-spec-engine/src/spec_parser.rs:24-77`; `sensors/{crowdstrike,armis,claroty,cyberint}.sensor.toml`

**Symptom:** The spec parser at lines 24-77 supports a `[fetch_step]` section for single-step HTTP fetch. CrowdStrike's authentication flow requires a two-step sequence: step 1 is a token endpoint POST; step 2 is the data endpoint GET using the token from step 1. The TOML grammar has no mechanism for this. The four production sensor TOML files are stubs — they contain the table names and credential refs but no fetch pipelines.

**Violation:** Even if the spec-engine's `PipelineExecutor` were fully implemented (it is not — see HIGH-5), it could not execute the CrowdStrike flow because the grammar cannot express it.

**Evidence:** `crates/prism-spec-engine/src/spec_parser.rs:24-77` — `FetchStep` struct has `url: String`, `method: String`, `headers: HashMap`, `body: Option<String>`. No sequencing field. No `retry_action` field.

**Migration (user-decided):** TOML grammar extensions required: `[fetch_step.batch]` for pagination; `[fetch_step.retry]` with `retry_action = "refresh_auth"` for OAuth2 token refresh; `two_step_fetch = { auth_step = ..., data_step = ... }` for the CrowdStrike two-step flow. CrowdStrike's OAuth2 refresh-on-401 ships as an in-repo `.prx` WASM plugin that implements the `retry_action` hook — NOT as in-tree Rust.

**Blast radius:** MED — grammar extensions are additive; existing TOML files remain valid.

---

### HIGH-5 — PipelineExecutor::execute is a stub returning canned empty values

**Location:** `crates/prism-spec-engine/src/pipeline.rs:54-66`

**Symptom:** `PipelineExecutor::execute` returns an empty `Vec<Row>` unconditionally:

```rust
pub async fn execute(&self, _spec: &SensorSpec, _org_id: &OrgId) -> Result<Vec<Row>> {
    // TODO: implement HTTP fetch, JSONPath extraction, pagination, retry
    Ok(Vec::new())
}
```

**Violation:** This is the stub that must become a real HTTP client, JSONPath extractor, paginator, and retry handler before any TOML-authored sensor can produce data. Without this, the entire TOML spec model is non-functional at runtime regardless of all other migrations.

**Evidence:** `crates/prism-spec-engine/src/pipeline.rs:54-66` — the function body contains only a comment and `Ok(Vec::new())`.

**Migration:** Build a real `PipelineExecutor` with: HTTP client (reqwest), JSONPath extraction (jsonpath-lib or serde_json pointer), fan-out across fetch steps, pagination loop consuming `next_page_token`, retry with exponential backoff, WASM plugin hook invocation for `retry_action = "refresh_auth"`. This is Wave 0/B (S-PLUGIN-PREREQ-B, ~13-18 SP).

**Blast radius:** HIGH — this is the fundamental enabling story for all TOML-authored sensor execution.

---

### MED-1 — boot.rs step 4 wires only parse_spec_directory; no plugin wiring

**Location:** `crates/prism-bin/src/boot.rs:495-530, 805-855`

**Symptom:** Boot step 4 calls `parse_spec_directory` and stores the resulting `SpecStore`. Steps 5-8 wire `SpecStore` into the query engine for read-side schema resolution but do NOT wire it into the `SensorRegistry` for fetch dispatch or into `PluginRuntime` for WASM plugin loading.

**Violation:** Even after a TOML sensor spec is loaded into `SpecStore`, no query reaches the spec-driven fetch path — the query engine uses `SensorRegistry` which is populated only by `init_registry_for_org` (the hardcoded Rust path in MED-3).

**Evidence:** `crates/prism-bin/src/boot.rs:495-530` — step 4 completes; `crates/prism-bin/src/boot.rs:805-855` — steps 7-8 wire query engine but not sensor registry.

**Migration:** Boot step 7 (new): after `SpecStore` is loaded, iterate all registered `SensorSpec` entries and register a `SpecDrivenAdapter` instance for each in `SensorRegistry`. Boot step 7b: load `.prx` WASM plugins from the plugin directory via `PluginRuntime`.

**Blast radius:** MED — boot.rs step 7 is the integration point; wiring is additive.

---

### MED-2 — AdapterRegistry::register requires SensorType return

**Location:** `crates/prism-sensors/src/registry.rs:40, 63-66`

**Symptom:** `AdapterRegistry::register` signature at line 40 takes `impl SensorAdapter` where `SensorAdapter: sensor_type(&self) -> SensorType`. Since `SensorType` is closed (CRIT-1), any adapter registered into the registry must claim one of the four hardcoded variants.

**Violation:** A `SpecDrivenAdapter` representing a TOML-authored sensor cannot be registered because it has no `SensorType` variant to return.

**Evidence:** `crates/prism-sensors/src/registry.rs:63-66` — the registry storage is `HashMap<SensorType, Box<dyn SensorAdapter>>`.

**Migration:** After CRIT-1 lands, change registry storage to `HashMap<SensorId, Box<dyn SensorAdapter>>` and change `SensorAdapter::sensor_type` return type to `SensorId`. This is a follow-on from S-PLUGIN-PREREQ-A.

**Blast radius:** LOW — contained to registry.rs; follows mechanically from CRIT-1.

---

### MED-3 — init_registry/init_registry_for_org hardcodes 4-sensor model

**Location:** `crates/prism-sensors/src/lib.rs:124-145, 166-197`

**Symptom:** Both `init_registry` and `init_registry_for_org` accept four explicit credential parameters (one per sensor) and construct exactly four adapters. The function signatures encode the number of sensors.

**Violation:** Adding a fifth sensor requires changing the function signature, all call sites, and all tests that call these functions. External TOML sensors cannot be added without modifying these functions.

**Evidence:** `crates/prism-sensors/src/lib.rs:166-197` — `init_registry_for_org(crowdstrike_creds: CrowdStrikeCredentials, armis_creds: ArmisCredentials, ...)` — four explicitly typed parameters.

**Migration:** Replace both functions with a single `build_registry_from_specs(spec_store: &SpecStore, org_id: &OrgId) -> Result<SensorRegistry>` that iterates `spec_store.all_sensors()` and constructs a `SpecDrivenAdapter` for each. This is part of Wave 1/A (PLUGIN-MIGRATION-001-A).

**Blast radius:** HIGH — this is a public API change touching all integration test call sites.

---

### LOW-1 — Architecture docs name 4 sensors directly

**Location:** `.factory/specs/architecture/module-decomposition.md`; `.factory/specs/architecture/decisions/ADR-022-production-runtime-wiring.md` (sections discussing production sensor wiring and the override absence)

**Symptom:** Module decomposition lists the four concrete sensor crates as top-level architecture components rather than as examples of the sensor extension model. The production-runtime-wiring decision record's section discussing override absence names the four sensors explicitly.

**Violation:** Documentation establishes the four sensors as the architectural definition rather than as examples. New contributors reading the docs would conclude new sensors require new Rust crates.

**Migration:** Wave 2/G (PLUGIN-MIGRATION-001-G) — doc/ADR/BC sweep. Update module-decomposition to describe `SpecDrivenAdapter` as the architecture and the four sensors as legacy-migration examples. Update the production-runtime-wiring decision record to reference the TOML+WASM canonical path and deprecate the Rust-crate sensor authorship model.

**Blast radius:** LOW — docs only; no code impact.

---

### LOW-2 — 4 stories embed wrong architecture as production deliverables

**Location:** `.factory/stories/{S-2.06, S-2.07, W3-FIX-S307-001, S-3.1.06-ImplPhase}.md`

**Symptom:** These stories define acceptance criteria that require the hardcoded sensor Rust architecture. S-2.06 and S-2.07 spec out concrete adapter types. W3-FIX-S307-001 specifies implementing `match endpoint.pipe_verb { ... }` inside named-sensor files. S-3.1.06-ImplPhase references the closed `SensorType` enum in its task list.

**Violation:** If delivered as-written, these stories would entrench the violation rather than begin the migration. W3-FIX-S307-001 was the active dispatch target before the PLUGIN-AUDIT-001 blocking decision.

**Migration:** Wave 2/H (PLUGIN-MIGRATION-001-H) — story supersession. Mark each story superseded by the appropriate Wave 0/1/2 plugin migration story. Update STORY-INDEX status from pending/draft to superseded.

**Blast radius:** LOW — stories only; no code impact.

---

### LOW-3 — BC catalog likely names sensor-specific behaviors (REQUIRES_VERIFICATION)

**Location:** `.factory/specs/behavioral-contracts/`

**Symptom:** Given the depth of sensor-specific hardcoding in the codebase, it is probable that some behavioral contracts specify CrowdStrike-specific, Armis-specific, or similar behavior rather than sensor-agnostic contracts. This was not verified by the audit due to BC file volume.

**Violation (if verified):** Sensor-specific BCs would anchor test coverage to the wrong implementation model. The wave 2/G sweep must verify and generalize any sensor-named BCs.

**Migration:** Wave 2/G sweep — grep BC bodies for sensor names; generalize any sensor-named BCs to spec-catalog model.

**Blast radius:** LOW to MED depending on how many BCs name sensors directly.

---

### OBS-1 — 10+ test files import concrete adapter types

**Location:** `crates/prism-sensors/tests/`, `crates/prism-query/tests/execute_integration_tests.rs:3517`

**Symptom:** Integration tests import `CrowdStrikeAdapter`, `ArmisAdapter`, etc. directly. They construct adapters with hardcoded credentials rather than loading a TOML spec.

**Violation:** Not a production code violation — test code using concrete types is expected during migration. However, these tests will break when the concrete re-exports are removed (HIGH-1 migration), and they provide no coverage for the spec-driven path.

**Migration:** Wave 2/F (PLUGIN-MIGRATION-001-F) — rewrite ~10 sensor-named test files to use the TOML fixture loading path. This provides coverage for the spec-driven adapter lifecycle.

**Blast radius:** LOW — tests only; no production impact.

---

### OBS-2 — DTU clone crates correctly named — TEST-ONLY ACCEPTABLE

**Location:** `crates/prism-dtu-{crowdstrike,armis,claroty,cyberint}/`

**Symptom:** Four DTU clone crates are named after the sensors they replicate.

**Assessment:** ACCEPTABLE. DTU crates are test infrastructure — they replicate the real sensor APIs under controlled conditions. Naming them after the sensors they clone is correct and expected. These crates do NOT participate in production dispatch and do NOT use the closed `SensorType` enum for runtime decisions.

**No migration required.**

---

## User-Decided Migration Approach

The user (project owner) made these decisions on 2026-05-10:

1. **OCSF mapping:** Hybrid — TOML column-level `ocsf_field` for 80% of cases; in-repo `.prx` WASM transformer plugins for the 20% requiring complex transforms (multi-field combos, timestamp formats, nested flattening). Drop the four in-tree Rust mappers in `prism-ocsf/src/mappers/`.

2. **SensorAuth sealing:** Un-seal entirely. Remove the `private::Sealed` marker. Cross-sensor-auth-composition prevention moves from compile-time to runtime spec validation.

3. **TOML authorship:** Reverse-engineer from existing Rust adapters with DTU-parity tests. Lowest behavioral-regression risk — each new TOML spec is validated against the DTU clone of the sensor it replaces.

4. **CrowdStrike OAuth2 refresh-on-401:** Ship as in-repo `.prx` WASM plugin. Signed by us. Loaded by `PluginRuntime` at boot. NOT in-tree Rust. This addresses the TOML grammar gap for two-step auth flows.

5. **Implied (orchestrator-derived from user mandate):** The `CustomAdapter` Rust trait at `crates/prism-spec-engine/src/custom_adapter.rs` is RETIRED. The `.prx` WASM plugin model becomes the SOLE escape hatch for behavior that cannot be expressed in TOML. Eat own dog food — Prism's own CrowdStrike refresh-on-401 plugin is the first exercise of this path.

## Migration Sequence

### Wave 0 — Prerequisites (5 stories, ~40-60 SP, no deletions)

**S-PLUGIN-PREREQ-A — SensorType → SensorId(Arc<str>) keystone migration**
- Replace `SensorType` closed enum with `SensorId(Arc<str>)` open newtype in prism-core
- Update `SensorAdapter::sensor_type` return type
- Update `AdapterRegistry` storage key type
- Update all downstream `match SensorType::X` arms to spec-catalog lookup stubs (real dispatch wired in Wave 1)
- Atomic 15-file commit; feature-flagged dual-definition NOT recommended (atomic preferred)
- Estimate: 13-18 SP

**S-PLUGIN-PREREQ-B — Real PipelineExecutor (HTTP, JSONPath, fan-out, paginate, retry, declarative)**
- Implement `PipelineExecutor::execute` with reqwest HTTP client, serde_json JSONPath extraction, pagination loop, retry with exponential backoff
- WASM plugin hook invocation for `retry_action = "refresh_auth"` (hook dispatch stub, fulfilled by S-PLUGIN-PREREQ-D)
- Estimate: 13-18 SP

**S-PLUGIN-PREREQ-C — TOML grammar extensions**
- `[fetch_step.batch]` for pagination cursor fields
- `[fetch_step.retry]` with `retry_action = "refresh_auth"` hook
- Two-step fetch: `auth_step + data_step` sequencing for CrowdStrike flow
- `columns[N].ocsf_field` for column-level OCSF annotation (hybrid mapper prerequisite)
- `columns[N].virtual_field_aliases` for virtual field resolution (CRIT-4 prerequisite)
- `cache_ttl_secs` for invalidation policy (CRIT-5 prerequisite)
- `table_name` canonical field for table prefix resolution (CRIT-6 prerequisite)
- Estimate: 5-8 SP

**S-PLUGIN-PREREQ-D — Wire PluginRuntime into boot.rs step 7; build .prx pipeline**
- Boot step 7 loads `.prx` WASM plugins from plugin directory via `PluginRuntime`
- Plugin signing infrastructure (ed25519 keypair; in-repo plugins signed at build time)
- `.prx` plugin manifest format (name, version, hook_points: [retry_action, ocsf_transform])
- Plugin sandbox security model (WASI subset, no network, read-only access to spec context)
- Estimate: 8-13 SP

**S-PLUGIN-PREREQ-E — Un-seal SensorAuth + deprecate-then-remove CustomAdapter Rust trait**
- Remove `private::Sealed` from `SensorAuth`
- Delete `CustomAuth` duplicate from custom_adapter.rs
- Mark `CustomAdapter` Rust trait as deprecated with doc redirect to WASM plugin docs
- Delete `CustomAdapterRegistry` dead code (custom_adapter.rs:64-152)
- Boot.rs steps 7-8 cleanup: remove `custom_adapter_registry` dead instantiation
- Estimate: 3-5 SP

### Wave 1 — Primary deletion + replacement (5 stories, ~40-60 SP)

**PLUGIN-MIGRATION-001-A — Delete prism-sensors/src/auth/{crowdstrike,armis,claroty,cyberint}.rs + re-exports + init_registry_for_org**
- Delete four named auth modules and their compiled-in auth logic
- Remove four `pub use` re-exports from lib.rs (HIGH-1 fix)
- Replace `init_registry_for_org(crowdstrike_creds, ...)` with `build_registry_from_specs(spec_store, org_id)` (MED-3 fix)
- Introduce `SpecDrivenAdapter` that loads credentials via `CredentialRef` (not hardcoded structs)
- Wire `SpecDrivenAdapter` registration into boot.rs step 7 (MED-1 fix)
- Estimate: 8-13 SP

**PLUGIN-MIGRATION-001-B — Convert prism-query to SensorId spec-catalog lookup (5 files)**
- explain.rs: replace two `match SensorType::X` blocks with `SpecCatalog::describe_sensor(sensor_id)` (CRIT-2 fix)
- write_dispatch.rs: replace sensor-name match with `WriteEndpointSpec` catalog lookup (CRIT-3 fix)
- virtual_fields.rs: replace hardcoded field map with `SensorSpec.columns[].virtual_field_aliases` lookup (CRIT-4 fix)
- invalidation.rs: replace hardcoded cache map with `SensorSpec.cache_ttl_secs` lookup (CRIT-5 fix)
- materialization.rs: replace `sensor_type_from_table_name` prefix dispatch with catalog lookup (CRIT-6 fix)
- Estimate: 8-13 SP

**PLUGIN-MIGRATION-001-C — Merge prism-ocsf 4 mappers → SpecDrivenMapper; ship .prx WASM transformers**
- Delete four named mapper modules in `prism-ocsf/src/mappers/`
- Implement `SpecDrivenMapper`: consult `SensorSpec.columns[].ocsf_field` for direct mappings; invoke WASM plugin for complex transforms
- Build in-repo `.prx` WASM transformer plugins for the 20% complex transforms (timestamp normalization, nested object flattening, multi-field combinators) — one plugin per complex transform class, not per sensor
- Estimate: 8-13 SP

**PLUGIN-MIGRATION-001-D — Author 4 production TOMLs (reverse-engineered + DTU-parity tests)**
- Reverse-engineer CrowdStrike, Armis, Claroty, Cyberint TOML sensor specs from existing Rust adapter implementations
- Add `fetch_step`, `columns`, `ocsf_field`, `cache_ttl_secs`, `table_name`, `credential_refs` fields
- Write DTU-parity integration tests: each test loads the TOML spec through `PipelineExecutor` against the DTU clone and compares output schema against the Rust adapter's known schema
- Estimate: 8-13 SP

**PLUGIN-MIGRATION-001-E — Build CrowdStrike OAuth2-refresh-on-401 as in-repo .prx WASM plugin**
- Implement the CrowdStrike token-refresh flow in Rust targeting `wasm32-wasi`
- Sign the plugin with the in-repo ed25519 keypair
- Wire into `PluginRuntime`'s `retry_action = "refresh_auth"` hook point
- Integration test: CrowdStrike TOML spec + refresh plugin against DTU clone that returns 401 on first request
- Estimate: 5-8 SP

### Wave 2 — Cleanup (3 stories, ~15-25 SP)

**PLUGIN-MIGRATION-001-F — Rewrite ~10 sensor-named test files**
- Convert all `CrowdStrikeAdapter`, `ArmisAdapter`, etc. direct imports to TOML fixture loading
- Ensure test coverage of the spec-driven adapter lifecycle (load spec → build adapter → execute fetch → validate output)
- Compile-fail perimeter test: assert `use prism_sensors::CrowdStrikeAdapter` does not compile
- Estimate: 5-8 SP

**PLUGIN-MIGRATION-001-G — Doc/ADR/BC sweep**
- Update `module-decomposition.md` to describe `SpecDrivenAdapter` as canonical; four sensors as migration examples
- Update production-runtime-wiring decision record's section discussing override absence to reference TOML+WASM canonical path
- Grep BC bodies for sensor names; generalize any sensor-named BCs to spec-catalog model
- Update `sensor-adapters.md` with accurate post-migration architecture
- Estimate: 5-8 SP

**PLUGIN-MIGRATION-001-H — Story supersession**
- Mark S-2.06, S-2.07, W3-FIX-S307-001, S-3.1.06-ImplPhase as superseded in STORY-INDEX
- Update superseded story files with `status: superseded, superseded_by: [PLUGIN-MIGRATION-001-*]`
- Estimate: 2-3 SP

**Total: 13 stories, ~100-140 SP, HIGH risk.** Cross-cutting workspace change touching ~10 crates. The SensorType keystone (S-PLUGIN-PREREQ-A) and the PipelineExecutor stub (S-PLUGIN-PREREQ-B) are the longest-pole items.

## Test Strategy

1. **Spec parser fixtures** — unit test suite for all new TOML grammar extensions (batch, retry, two-step fetch, ocsf_field, virtual_field_aliases, cache_ttl_secs, table_name). Each grammar feature has a valid and an invalid TOML fixture. Parser rejects invalid fixtures with structured errors.

2. **PipelineExecutor units** — against a mock HTTP server (wiremock or similar): single-step fetch, paginated fetch, two-step auth+data fetch, retry-on-401 with mock refresh endpoint, JSONPath extraction correctness.

3. **DTU-backed integration parity tests** — each of the four production TOML specs is loaded through `PipelineExecutor` against the corresponding DTU clone. Output schema compared against the Rust adapter's known schema. Row count within 5% of Rust adapter output (DTU fixture parity). This is the behavioral-regression gate for the TOML authorship migration.

4. **CrowdStrike .prx plugin tests** — the OAuth2 refresh plugin is loaded via `PluginRuntime` in a test harness. Test: plugin fires on 401, acquires new token, retries request, succeeds. Plugin signing verification: unsigned plugins are rejected.

5. **Boot-sequence test with hypothetical third-party sensor TOML** — end-to-end test: place a `hypothetical_sensor.sensor.toml` file in the sensor spec directory; boot the binary; verify the hypothetical sensor appears in the MCP `list_sensors` tool output without any Prism source code changes. This is the acceptance criterion for the architecture mandate.

6. **Compile-fail perimeter test** — asserts `use prism_sensors::CrowdStrikeAdapter;` does not compile post-Wave 1. Located alongside the existing `tests/external/perimeter-violation/` compile-fail harness.

## Closed Open Questions

Decided by the user on 2026-05-10:

- **OCSF mapper fate:** Hybrid (TOML 80% + .prx WASM 20%) — addressed in Wave 1/C
- **SensorAuth sealing:** Un-seal entirely — addressed in Wave 0/E
- **TOML authorship:** Reverse-engineer with DTU parity — addressed in Wave 1/D
- **CrowdStrike adapter form:** .prx WASM in-repo plugin — addressed in Wave 1/E
- **Retirement of CustomAdapter Rust trait:** Yes (orchestrator-derived from dogfood mandate) — addressed in Wave 0/E

## Remaining Open Questions (carry into Wave 0/A story-writer brief)

- **CRIT-1 commit shape:** Atomic 15-file commit vs feature-flagged dual-definition stage — RECOMMEND atomic. The dual-definition stage requires maintaining two dispatch paths simultaneously and increases integration complexity without proportionate safety benefit given the DTU test coverage.

- **SensorAdapter::write story:** Declarative `WriteEndpointSpec` executed by `PipelineExecutor::execute_write` OR `.prx` WASM trampoline for sensors with complex write flows — RECOMMEND TOML write-endpoint executor with `.prx` fallback for sensors requiring non-HTTP write semantics. CrowdStrike write is HTTP; TOML suffices. If a future sensor requires custom binary write protocol, that is the `.prx` escape hatch.
