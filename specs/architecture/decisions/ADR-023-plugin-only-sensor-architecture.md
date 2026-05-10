---
document_type: adr
adr_id: "ADR-023"
title: "Plugin-Only Sensor Architecture — TOML Specs + .prx WASM Plugins, No Compiled-In Sensor Rust"
status: COMMITTED
date: "2026-05-10"
version: "1.0"
producer: architect
subsystems_affected: [SS-01, SS-02, SS-16, SS-17, SS-21]
supersedes: null
superseded_by: null
amends: ADR-022
inputs:
  - .factory/cycles/wave-4-operations/audits/plugin-only-violations-2026-05-10.md
  - .factory/specs/architecture/decisions/ADR-022-production-runtime-wiring.md
  - crates/prism-sensors/src/auth/mod.rs
  - crates/prism-sensors/src/lib.rs
  - crates/prism-sensors/src/adapter.rs
  - crates/prism-sensors/src/registry.rs
  - crates/prism-spec-engine/src/custom_adapter.rs
  - crates/prism-spec-engine/src/pipeline.rs
  - crates/prism-spec-engine/src/spec_parser.rs
  - crates/prism-spec-engine/src/plugin/mod.rs
  - crates/prism-core/src/types.rs
  - crates/prism-ocsf/src/mappers/
  - crates/prism-query/src/explain.rs
  - crates/prism-query/src/write_dispatch.rs
  - crates/prism-query/src/virtual_fields.rs
  - crates/prism-query/src/invalidation.rs
  - crates/prism-query/src/materialization.rs
  - crates/prism-bin/src/boot.rs
input-hash: "2f64319"
---

# ADR-023: Plugin-Only Sensor Architecture

## Status

COMMITTED 2026-05-10, v1.0. Status is `COMMITTED` rather than `ACCEPTED` because five
infrastructure prerequisites (Constraints C1–C5 below) must land before the hardcoded sensor
adapters can be deleted. Once all five prerequisite stories ship and pass their gates, this
ADR transitions to `ACCEPTED`. Implementation is tracked by PLUGIN-MIGRATION-001 (13 stories,
Waves 0/1/2, approximately 100–140 SP).

This ADR amends ADR-022 Section G, Story 3 — the reference to "the four built-in sensor
adapters" in that section is retired concurrent with Wave 1 of PLUGIN-MIGRATION-001. All other
sections of ADR-022 remain in force. ADR-022 is not superseded.

---

## Context

The 2026-05-10 codebase audit (PLUGIN-AUDIT-001, referenced in full at
`.factory/cycles/wave-4-operations/audits/plugin-only-violations-2026-05-10.md`) catalogued
21 violations across 10 crates. The audit revealed a material divergence between the platform's
declared architecture and its actual implementation. The Prism platform documentation claims
"no compiled-in sensor-specific Rust code" and describes TOML spec files plus WASM plugins as
the canonical sensor authorship surface. The codebase contradicts this at every layer.

The root cause is structural. The `SensorType` type in `crates/prism-core/src/types.rs` is a
closed Rust enum with one variant per sensor: `CrowdStrike`, `Armis`, `Claroty`, `Cyberint`.
This keystone forces every new sensor to be a source-code change — the enum is dispatched by
`match` arms in at least 7 downstream locations across 4 crates. Concurrent with this, four
sensor-named Rust auth modules at `crates/prism-sensors/src/auth/` encode sensor-specific
authentication logic in compiled Rust. The `SensorAuth` trait is sealed via `private::Sealed`
in `crates/prism-sensors/src/auth/mod.rs`, actively preventing external implementors — making
the "external TOML authorship" claim false at the type level.

A `CustomAuth` placeholder in `crates/prism-spec-engine/src/custom_adapter.rs` duplicates
`SensorAuth` — evidence that the spec-engine needed a parallel auth surface because it cannot
implement the sealed one. The `PipelineExecutor::execute` method in
`crates/prism-spec-engine/src/pipeline.rs` returns `Ok(Vec::new())` unconditionally — the
TOML spec model is entirely non-functional at runtime. The `CustomAdapterRegistry` and
`PluginRuntime` types in `crates/prism-spec-engine/src/custom_adapter.rs` and
`crates/prism-bin/src/boot.rs` are instantiated at boot but immediately dropped — dead code.

The user (project owner) identified this as architectural fraud on 2026-05-10: "we arent
suppose to have anything built in, everything uses the plugin system. We need to do a full
audit to make sure we are following that." The audit result confirmed the mandate: the platform
must be rebuilt to match its documented architecture. Five user-decided positions (Decision
Rules 1–5 below) govern the migration approach.

---

## Decision

The Prism platform ships ZERO compiled-in sensor-specific Rust code. Sensor behavior is
expressed exclusively through (a) declarative TOML spec files loaded by `prism-spec-engine`
and (b) sandboxed `.prx` WASM plugins for non-declarative quirks. The four initial sensors
(CrowdStrike, Cyberint, Claroty, Armis) ship as TOML specs plus, where required, in-repo
`.prx` WASM plugins, identical in form to third-party plugins. The platform team eats its own
dog food — Prism's own sensor specs flow through the same authorship pipeline as external
sensor authors.

### Decision Rules

**Rule 1 — OCSF Mapping: Hybrid TOML/WASM**

Column-level OCSF field mapping is declarative via TOML — each `SensorSpec.columns[N].ocsf_field`
carries the target OCSF field name for simple direct mappings (approximately 80% of cases).
Complex transforms (multi-field combinations, custom timestamp parsing, nested struct flattening,
enum coercions) ship as in-repo `.prx` WASM transformer plugins addressed by `(sensor_id, table)`
pair. The four per-sensor mapper modules in `crates/prism-ocsf/src/mappers/` are retired. A
`SpecDrivenMapper` in prism-ocsf consults the spec-catalog for column-level `ocsf_field`
annotations and dispatches to WASM plugins for the complex-transform remainder.

**Rule 2 — SensorAuth Trait Un-Sealing**

The `private::Sealed` marker pattern in `crates/prism-sensors/src/auth/mod.rs` is removed
entirely. Cross-sensor auth-composition prevention moves from compile-time type-system
enforcement to runtime spec-validation enforcement at TOML load time. The spec-catalog
validation layer rejects specs that claim unsupported or incompatible auth combinations. Plugin
authors implement `SensorAuth` directly without trait-sealing workarounds. The `CustomAuth`
placeholder duplicate in `crates/prism-spec-engine/src/custom_adapter.rs` is deleted — it
served only as an escape hatch around the sealed trait and is unnecessary once sealing is
removed.

**Rule 3 — TOML Authorship: Reverse-Engineered with DTU-Parity Tests**

Production TOML sensor specs for the four initial sensors are reverse-engineered from current
Rust adapter behavior. Replacement sensor specs must produce identical record outputs compared
to the to-be-deleted Rust adapters when fed identical inputs against DTU clone fixtures. Each
production TOML spec is validated by a DTU-parity integration test before the corresponding
Rust adapter is deleted. This approach minimizes behavioral-regression risk during migration —
the DTU clone becomes the behavioral ground truth.

**Rule 4 — CrowdStrike OAuth2-Refresh-on-401: In-Repo .prx WASM Plugin**

CrowdStrike's OAuth2 token refresh-on-401 flow ships as an in-repo `.prx` WASM plugin, built
targeting `wasm32-wasi`, signed with the in-repo ed25519 keypair, and loaded at boot by
`PluginRuntime`. It is NOT implemented as in-tree Rust. It is built, signed, and loaded
identically to third-party plugins via the platform's `PluginRuntime`. The platform team ships
its own auth-quirk plugin through the same pipeline as external authors. This makes CrowdStrike
the first real exercise of the in-repo plugin path and validates the entire `.prx`
build/sign/load pipeline before external authors use it.

**Rule 5 — CustomAdapter Rust Trait Retirement**

The `CustomAdapter` Rust trait in `crates/prism-spec-engine/src/custom_adapter.rs` is
deprecated and removed. The placeholder duplicate `SensorAuth` declaration in the same file is
also removed (un-sealing in Rule 2 eliminates its purpose). The `CustomAdapterRegistry` dead
code in `crates/prism-spec-engine/src/custom_adapter.rs` is deleted. The boot sequence dead
code in `crates/prism-bin/src/boot.rs` that instantiates and immediately drops both the
registry and the plugin runtime is replaced with a live wiring step that loads `.prx` plugins
via `PluginRuntime` (Constraint C4). The `.prx` WASM plugin model becomes the SOLE escape
hatch for non-declarative sensor behavior. No Rust-trait-based escape hatch survives.

---

## Forbidden Patterns

The following patterns are prohibited in production source files — files under
`crates/prism-{core,sensors,query,ocsf,mcp,bin,spec-engine,credentials,security,audit,operations,storage}/src/`.
Test-only code (under `tests/` or within `#[cfg(test)]` blocks) is exempt from items marked
`[test-ok]`.

| Pattern | Original location | Enforcement |
|---|---|---|
| `pub mod crowdstrike;` / `pub mod armis;` / `pub mod claroty;` / `pub mod cyberint;` under `crates/prism-sensors/` | `auth/mod.rs`, `lib.rs` | compile-fail perimeter test + CI grep |
| `pub use crowdstrike::*Adapter` / `pub use armis::*Adapter` etc. in `crates/prism-sensors/src/lib.rs` | `lib.rs` | compile-fail perimeter test + CI grep |
| `match SensorType::` dispatch in production code | any production crate | CI grep gate |
| `enum SensorType` definition with hardcoded sensor variants | `prism-core/src/types.rs` | CI grep gate |
| `private::Sealed` in any sensor-related trait | `auth/mod.rs` | CI grep gate |
| `crate::custom_adapter::CustomAdapter` in production paths | `prism-spec-engine` | CI grep gate |
| `init_registry_for_org` or any successor with hardcoded per-sensor credential parameter lists | `lib.rs` | CI grep gate |
| `prism_ocsf::mappers::<sensor_name>` module paths | `prism-ocsf/src/mappers/` | compile-fail perimeter test |

Enforcement is via three mechanisms:

1. A compile-fail perimeter test crate at `tests/external/no-hardcoded-sensors/` (modeled on
   the existing perimeter at `tests/external/perimeter-violation/`).
2. A CI grep gate scanning for the disallowed string patterns on every PR to `develop`.
3. A code-review checklist entry in `.github/PULL_REQUEST_TEMPLATE.md` (added in Wave 2/G).

---

## Permitted Patterns

The following usages are explicitly permitted and must not be blocked by the enforcement gates:

- `crates/prism-spec-engine/src/spec_parser.rs` defines the TOML grammar for sensor specs.
- `crates/prism-spec-engine/src/pipeline.rs` — `PipelineExecutor` executes sensor specs
  declaratively (HTTP fetch, JSONPath extraction, pagination, retry, WASM plugin hook dispatch).
- `crates/prism-spec-engine/src/plugin/` directory — the `PluginRuntime` that loads, verifies,
  and invokes `.prx` WASM plugins.
- DTU clone crates at `crates/prism-dtu-{crowdstrike,armis,claroty,cyberint}/` — test
  infrastructure only (dev-dependency), per ADR-022 dev-dep policy. Sensor naming in DTU crates
  is correct and expected. These crates do not participate in production type dispatch.
- In-repo `.prx` WASM plugin source crates at `crates/plugins/{plugin-name}/` (new path) —
  these compile to `.prx` artifacts consumed by `PluginRuntime`. They do NOT compile to
  in-process Rust modules and do NOT participate in the production type system.
- Compile-fail tests, integration test fixtures, and DTU-backed integration tests may name
  sensors — test-only context is acceptable.
- `SensorId(Arc<str>)` newtype in `prism-core` — the open replacement for `SensorType`.

---

## Architectural Constraints

For Decision Rules 1–5 to be enforceable, the following infrastructure must exist before
deletion of the hardcoded adapters. These are the five prerequisite stories of Wave 0
(PLUGIN-MIGRATION-001 Waves 0/A through 0/E):

**C1 — SensorId newtype (PLUGIN-PREREQ-A):** `SensorId(Arc<str>)` open newtype replaces the
closed `SensorType` enum in `prism-core`. `SensorAdapter::sensor_type` return type changes to
`SensorId`. `AdapterRegistry` storage changes from a `SensorType`-keyed map to a
`SensorId`-keyed map. All downstream `match SensorType::X` arms across seven locations in four
crates are converted to spec-catalog lookup stubs. This is the keystone change — approximately
15 files across 5 crates, atomic commit preferred over feature-flagged dual-definition.

**C2 — Real PipelineExecutor (PLUGIN-PREREQ-B):** The `PipelineExecutor::execute` stub in
`crates/prism-spec-engine/src/pipeline.rs` (currently returns `Ok(Vec::new())`) is replaced
with a real implementation: HTTP client (reqwest), JSONPath extraction (serde_json pointer or
jsonpath-lib), pagination loop consuming cursor fields, retry with exponential backoff, and
WASM plugin hook invocation for the `retry_action = "refresh_auth"` hook point. Without this,
the TOML spec model is non-functional at runtime regardless of all other migrations.

**C3 — TOML grammar extensions (PLUGIN-PREREQ-C):** The grammar in
`crates/prism-spec-engine/src/spec_parser.rs` is extended to support:
`[fetch_step.batch]` for pagination cursor fields; `[fetch_step.retry]` with
`retry_action = "refresh_auth"` for OAuth2 token refresh; two-step fetch sequencing
(`auth_step + data_step`) for the CrowdStrike flow; `columns[N].ocsf_field` for column-level
OCSF annotation; `columns[N].virtual_field_aliases` for virtual field resolution; `cache_ttl_secs`
for invalidation policy; and `table_name` canonical field for table prefix resolution. All
extensions are additive — existing TOML files remain valid.

**C4 — PluginRuntime wired into boot, .prx pipeline functional (PLUGIN-PREREQ-D):** Boot step
7 in `crates/prism-bin/src/boot.rs` loads `.prx` WASM plugins from the plugin directory via
`PluginRuntime`. Plugin signing uses ed25519 keypairs; in-repo plugins are signed at build
time. The `.prx` plugin manifest format declares name, version, and hook points
(`retry_action`, `ocsf_transform`). The plugin sandbox enforces a WASI subset: no network
access, read-only access to spec context. Until this lands, the in-repo CrowdStrike OAuth2
refresh plugin cannot be loaded at boot.

**C5 — SensorAuth un-sealed, CustomAdapter removed (PLUGIN-PREREQ-E):** The `private::Sealed`
marker is removed from `SensorAuth` in `crates/prism-sensors/src/auth/mod.rs`. The `CustomAuth`
duplicate is deleted from `crates/prism-spec-engine/src/custom_adapter.rs`. The `CustomAdapter`
Rust trait is deprecated then removed from the same file. The `CustomAdapterRegistry` dead code
is deleted. Boot steps 7–8 cleanup: the dead `custom_adapter_registry` and `plugin_runtime`
instantiations in `crates/prism-bin/src/boot.rs` are replaced with the live `PluginRuntime`
wiring from C4.

---

## Verification Properties

**VP-PLUGIN-001 — No production hardcoded sensor references:** No production source file in
`crates/prism-{core,sensors,query,ocsf,mcp,bin}/` references `SensorType`, `CrowdStrikeAdapter`,
`ArmisAdapter`, `ClarotyAdapter`, or `CyberintAdapter`. Enforced by a compile-fail perimeter
test crate at `tests/external/no-hardcoded-sensors/` plus a CI grep gate. Both must pass on
every PR targeting `develop`.

**VP-PLUGIN-002 — Unknown sensor registers without code change:** A `prism start --spec-dir
<fixture>` invocation with a previously-unknown sensor TOML (for example,
`hypothetical_sensor.sensor.toml`) successfully registers the sensor and the sensor appears in
the MCP `list_sensors` tool output. Zero Prism source-code changes are required to add the
sensor. This is the acceptance criterion for the architecture mandate — verified by an
end-to-end integration test in Wave 1.

**VP-PLUGIN-003 — DTU byte-level output parity:** Output-record parity between the deleted
Rust adapter path and the replacement TOML plus `.prx` plugin path, validated against DTU
clone fixtures per sensor. Row count within 5% and field schema byte-for-byte identical. One
parity test per sensor (four total), gated in Wave 1 before any adapter deletion.

**VP-PLUGIN-004 — PluginRuntime rejects unsigned plugins:** `PluginRuntime` correctly enforces
signature verification on `.prx` plugins. Unsigned plugins and plugins with tampered signatures
fail to load with a structured error. Verified by an integration test that attempts to load a
malformed-signature `.prx` file and asserts the expected error code.

**VP-PLUGIN-005 — OAuth2 refresh-on-401 via .prx plugin:** The TOML grammar's
`retry_action = "refresh_auth"` hook correctly invokes the in-repo CrowdStrike `.prx` plugin
on an HTTP 401 response. The plugin acquires a new token and retries the request. Verified by
an integration test against the DTU clone with 401-injection mode enabled.

---

## Rationale

The closed `SensorType` enum is not a detail — it is the architectural keystone that makes
every other hardcoding finding load-bearing. Seven downstream `match` arms across four crates
must change in lockstep whenever a new sensor is added. This is structurally incompatible with
the platform's extensibility contract. No partial fix resolves the contradiction; the enum must
be replaced wholesale with an open identity type.

The hybrid TOML-plus-WASM model (Rule 1) balances declarative readability against escape-hatch
necessity. Approximately 80% of OCSF mappings are simple column remappings expressible in one
`ocsf_field` annotation. Forcing these into WASM would be over-engineering. But timestamp
normalization, nested object flattening, and multi-field combination cannot be expressed
declaratively without inventing a second mini-language inside TOML — at that point, a typed
WASM function is cleaner. The 80/20 split was the user's explicit decision.

Un-sealing `SensorAuth` (Rule 2) is the minimum change required to unblock external auth
implementors. The sealed trait was presumably introduced to prevent cross-sensor auth
composition bugs, but this is a blunt instrument that also prevents any external auth extension.
Moving the enforcement to runtime spec-validation is more precise: the validator can reject
specific incompatible auth combinations while permitting valid novel ones.

Retiring the `CustomAdapter` Rust trait (Rule 5) is the dogfood completion step. If the
platform team maintained a Rust-trait escape hatch alongside the WASM escape hatch, third-party
plugin authors would reasonably ask why they cannot use the Rust path. The answer — "that path
is for us, not you" — contradicts the eat-own-dog-food principle. One escape hatch (WASM),
used by everyone including the platform team, is the correct invariant.

The TOML-authorship-via-reverse-engineering approach (Rule 3) is chosen over a greenfield TOML
spec authoring approach because the behavioral ground truth already exists in the Rust adapters.
DTU clone fixtures provide deterministic inputs. Parity tests provide automatic verification
that the migration did not change behavior. This is the lowest-risk migration path for a
production system.

---

## Consequences

### Positive

- Removes architectural fraud: code matches the documented architecture after migration.
- True extensibility: third-party sensors can be added via TOML or `.prx` without forking
  Prism or modifying any Rust source file.
- Single escape hatch model: `.prx` WASM plugins are the only non-declarative extension point,
  reducing the mental model complexity for plugin authors.
- Eat-own-dog-food: the platform team ships its CrowdStrike OAuth2 refresh plugin through the
  same pipeline as external authors, validating the pipeline before external use.
- Compile-fail perimeter test proves no regression to the hardcoded model post-migration.

### Negative / Trade-offs

- High implementation cost: approximately 100–140 story points across 13 stories in 3 waves.
- Increased runtime complexity: WASM toolchain, `.prx` signing infrastructure, `wasm32-wasi`
  build targets, and sandbox enforcement add operational surface area.
- Error-class shift: spec-catalog validation errors that were previously compile-time type
  errors (e.g., invalid `SensorType` variant) become spec-load-time runtime errors. Operators
  encounter these at startup, not build time.
- The `PipelineExecutor` stub in `crates/prism-spec-engine/src/pipeline.rs` is the longest-pole
  item in Wave 0 and blocks all Wave 1 deletions — until it ships, the TOML spec model produces
  no data.
- Performance must be benchmarked: the spec-engine execution path carries overhead compared to
  the hardcoded Rust adapter path; per-query performance budgets from Phase 6 must be verified
  to hold.

### Status as of 2026-05-10

COMMITTED, pending implementation of Constraints C1–C5 (PLUGIN-MIGRATION-001 Wave 0). The
five hardcoded sensor auth modules, the four OCSF mapper modules, the `SensorType` enum, and
the `CustomAdapter` trait all remain in the codebase until their corresponding Wave 0/1 stories
ship and pass DTU-parity gates.

---

## Alternatives Considered

**Option A — Maintain two authorship paths (Rust trait + WASM):** Keep the `CustomAdapter`
Rust trait as a "power user" escape hatch alongside the WASM path. Rejected because it creates
a two-tier author model: platform team uses Rust, external authors use WASM. This contradicts
the eat-own-dog-food principle and means the WASM path is never fully exercised by the people
who maintain it.

**Option B — Pure TOML, no WASM escape hatch:** Express all sensor behavior declaratively in
TOML, including CrowdStrike's OAuth2 refresh flow. Rejected because complex auth flows require
either an embedded scripting language inside TOML (a mini-language with its own failure modes)
or a combinatorial explosion of TOML grammar extensions. A single well-defined WASM hook point
is cleaner than an unbounded declarative expression surface.

**Option C — Feature-flagged dual-definition for SensorType to SensorId migration:** Maintain
both `SensorType` and `SensorId` simultaneously during the migration, gradually porting dispatch
sites. Rejected because the dual-definition increases integration complexity without proportionate
safety benefit given DTU-parity test coverage. The audit recommends an atomic 15-file commit
(PLUGIN-PREREQ-A) as the safer approach — all 7 downstream `match` arms convert in one commit,
eliminating the integration window where old and new dispatch paths coexist.

**Option D — Keep sealed SensorAuth, provide a parallel public trait for external use:**
Introduce a second `ExternalSensorAuth` trait that external authors implement, with an adapter
to bridge into `SensorAuth`. Rejected because this is exactly what the `CustomAuth` duplicate
in `crates/prism-spec-engine/src/custom_adapter.rs` already attempted — and it diverged from
`SensorAuth` immediately. Two auth trait surfaces are harder to maintain than one.

---

## Migration Plan

PLUGIN-MIGRATION-001: 13 stories, 3 waves, approximately 100–140 SP, HIGH risk.

**Wave 0 — Prerequisites (5 stories, approximately 40–60 SP, no deletions):**

- PLUGIN-PREREQ-A: `SensorType` to `SensorId(Arc<str>)` keystone migration (13–18 SP)
- PLUGIN-PREREQ-B: Real `PipelineExecutor` — HTTP, JSONPath, pagination, retry, WASM hook (13–18 SP)
- PLUGIN-PREREQ-C: TOML grammar extensions — batch, retry, two-step, ocsf_field, virtual aliases, cache_ttl, table_name (5–8 SP)
- PLUGIN-PREREQ-D: Wire `PluginRuntime` into boot step 7; build `.prx` sign/load pipeline (8–13 SP)
- PLUGIN-PREREQ-E: Un-seal `SensorAuth`; deprecate and remove `CustomAdapter` Rust trait (3–5 SP)

**Wave 1 — Primary deletion and replacement (5 stories, approximately 40–60 SP):**

- PLUGIN-MIGRATION-001-A: Delete four named auth modules + re-exports + `init_registry_for_org`; introduce `SpecDrivenAdapter` and `build_registry_from_specs` (8–13 SP)
- PLUGIN-MIGRATION-001-B: Convert five prism-query dispatch sites to spec-catalog lookups — explain, write_dispatch, virtual_fields, invalidation, materialization (8–13 SP)
- PLUGIN-MIGRATION-001-C: Delete four OCSF mapper modules; implement `SpecDrivenMapper`; ship in-repo `.prx` WASM complex-transform plugins (8–13 SP)
- PLUGIN-MIGRATION-001-D: Author four production TOMLs via reverse-engineering; DTU-parity tests per sensor (8–13 SP)
- PLUGIN-MIGRATION-001-E: Build CrowdStrike OAuth2-refresh-on-401 as in-repo `.prx` WASM plugin; integration test with DTU clone 401-injection (5–8 SP)

**Wave 2 — Cleanup (3 stories, approximately 15–25 SP):**

- PLUGIN-MIGRATION-001-F: Rewrite approximately 10 sensor-named test files to TOML fixture loading; compile-fail perimeter test at `tests/external/no-hardcoded-sensors/` (5–8 SP)
- PLUGIN-MIGRATION-001-G: Doc sweep — module-decomposition, production-runtime-wiring decision record section discussing the override absence, BC catalog sensor-name grep, sensor-adapters.md (5–8 SP)
- PLUGIN-MIGRATION-001-H: Story supersession — mark S-2.06, S-2.07, W3-FIX-S307-001, S-3.1.06-ImplPhase superseded in STORY-INDEX (2–3 SP)

**Risk posture:** HIGH. The `SensorType` keystone change (PLUGIN-PREREQ-A) touches approximately
15 files across 5 crates and must be atomic. The `PipelineExecutor` implementation
(PLUGIN-PREREQ-B) is the longest-pole story and blocks all Wave 1 deletions. Wave 1 deletions
carry behavioral regression risk mitigated by DTU-parity tests (VP-PLUGIN-003).

---

## Source / Origin

- **User directive:** 2026-05-10, verbatim: "we arent suppose to have anything built in, everything uses the plugin system. We need to do a full audit to make sure we are following that." This directive supersedes any prior story or ADR that implied sensor Rust authorship as a valid path.
- **Audit document:** `.factory/cycles/wave-4-operations/audits/plugin-only-violations-2026-05-10.md` (PLUGIN-AUDIT-001) — 21 violations across 10 crates, catalogued by codebase-analyzer on 2026-05-10. All five user-decided positions are recorded in the audit's "User-Decided Migration Approach" section.
- **Amended document:** `ADR-022-production-runtime-wiring.md` Section G, Story 3 — the reference to "the four built-in sensor adapters" is retired by this ADR concurrent with PLUGIN-MIGRATION-001 Wave 1.
- **Code as-built (violations):** Closed `SensorType` enum in `crates/prism-core/src/types.rs`; sealed `SensorAuth` trait in `crates/prism-sensors/src/auth/mod.rs`; stub `PipelineExecutor` in `crates/prism-spec-engine/src/pipeline.rs`; four concrete adapter re-exports in `crates/prism-sensors/src/lib.rs`.
- **Architecture intent:** `.factory/specs/architecture/sensor-adapters.md` — states "no compiled-in sensor-specific Rust code" as the adapter layer goal. This ADR makes that intent enforceable.
