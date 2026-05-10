---
document_type: adr
adr_id: "ADR-023"
title: "Plugin-Only Sensor Architecture — TOML Specs + .prx WASM Plugins, No Compiled-In Sensor Rust"
status: COMMITTED
date: "2026-05-10"
version: "v1.2"
producer: architect
subsystems_affected: [SS-01, SS-02, SS-16, SS-17, SS-21, SS-22]
supersedes: null
superseded_by: null
amends: ADR-022
amends_bcs: ["BC-2.16.004", "BC-2.01.013"]
amends_bcs_pending_full_amendment_in_wave_2_g:
  - BC-2.01.005-crowdstrike-oauth2-two-step-fetch
  - BC-2.01.006-cyberint-cookie-auth
  - BC-2.01.007-claroty-bearer-polymorphic-ids
  - BC-2.01.008-armis-bearer-aql
  - BC-2.02.003-crowdstrike-field-mapping
  - BC-2.02.004-cyberint-field-mapping
  - BC-2.02.005-claroty-field-mapping
  - BC-2.02.006-armis-field-mapping
retires_bcs: ["BC-2.16.004"]
amends_dis: ["DI-012"]
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
  - crates/prism-spec-engine/src/plugin/host_functions.rs
  - crates/prism-spec-engine/src/plugin/loader.rs
  - crates/prism-core/src/types.rs
  - crates/prism-ocsf/src/mappers/
  - crates/prism-query/src/explain.rs
  - crates/prism-query/src/write_dispatch.rs
  - crates/prism-query/src/virtual_fields.rs
  - crates/prism-query/src/invalidation.rs
  - crates/prism-query/src/materialization.rs
  - crates/prism-bin/src/boot.rs
  - .factory/specs/behavioral-contracts/BC-2.16.004-rust-escape-hatch.md
  - .factory/specs/behavioral-contracts/BC-2.01.013-datasource-trait-adapter-pattern.md
  - .factory/specs/domain-spec/invariants.md
input-hash: "[live-state]"
---

# ADR-023: Plugin-Only Sensor Architecture

## Status

COMMITTED 2026-05-10, v1.2. Status is `COMMITTED` rather than `ACCEPTED` because six
infrastructure prerequisites (Constraints C1–C5 plus Wave 0/F BC+DI amendments) must land
before the hardcoded sensor adapters can be deleted. Once all prerequisite stories ship and
pass their gates, this ADR transitions to `ACCEPTED`. Implementation is tracked by
PLUGIN-MIGRATION-001 (14 stories, Waves 0/1/2, approximately 94–146 SP (F-MED-NEW-004 corrected; Wave 0: 45–67, Wave 1: 37–60, Wave 2: 12–19)).

This ADR amends ADR-022 Section G, Story 3 — the reference to "the four built-in sensor
adapters" in that section is retired concurrent with Wave 1/A of PLUGIN-MIGRATION-001. All
other sections of ADR-022 remain in force. ADR-022 is not superseded. ADR-022 will be amended
to v1.2 (adding `superseded_by_partial: ADR-023` annotation to §G Story 3) as a sub-deliverable
of the Wave 1/A cutover commit — the same commit that deletes the Rust adapter files and
transitions ADR-023 from COMMITTED to ACCEPTED (F-HIGH-NEW-003: consistent timing across all
three locations).

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
(CrowdStrike, Cyberint, Claroty, Armis) will ship as TOML specs plus, where required, in-repo
`.prx` WASM plugins, identical in form to third-party plugins. The platform team eats its own
dog food — Prism's own sensor specs will flow through the same authorship pipeline as external
sensor authors.

### Decision Rules

**Rule 1 — OCSF Mapping: Hybrid TOML/WASM**

Column-level OCSF field mapping is declarative via TOML — each `SensorSpec.columns[N].ocsf_field`
carries the target OCSF field name for simple direct mappings (approximately 80% of cases).
Complex transforms ship as in-repo `.prx` WASM transformer plugins addressed by `(sensor_id, table)`
pair. The four per-sensor mapper modules in `crates/prism-ocsf/src/mappers/` will be retired.
A `SpecDrivenMapper` in prism-ocsf will consult the spec-catalog for column-level `ocsf_field`
annotations and dispatch to WASM plugins for the complex-transform remainder.

**Closed grammar for `ocsf_field` column values (TOML-mappable, the 80% case):**

The following patterns are expressible in TOML `ocsf_field` and do NOT require a WASM plugin
(F-HIGH-NEW-002: grammar re-authored from mapper source to include all observed patterns):

- Single source field, string-to-string: source field mapped to OCSF path with implicit
  string-to-string coercion (e.g., `detection_id` → `finding_info.uid`).
- Nullable mapping: source `Option<T>` mapped to OCSF `Optional[T]` path with implicit
  null-propagation.
- Integer-to-string cast: source integer (e.g., Armis device `id` as `i64`) mapped to OCSF
  string field via `cast = "string"` alongside `ocsf_field`. Observed in Armis mapper where
  `id` and `alertId` are extracted as integer-or-string with `n.to_string()` fallback.
- Identity / no-op: source column already has the canonical OCSF field name; `ocsf_field`
  annotation declares the target but requires no transformation (pass-through).
- RFC3339 string timestamp: source field is an RFC3339 string (e.g., Armis `last_seen`,
  `created_at`); TOML annotation `ocsf_field = "time"` with `format = "rfc3339"` maps to
  OCSF epoch-millis. Observed in Armis `parse_armis_timestamp_str` which tries `parse_from_rfc3339`
  then `%Y-%m-%dT%H:%M:%S` naive fallback.

**WASM-required patterns (the 20% case):**

The following patterns cannot be expressed in TOML `ocsf_field` and MUST use a `.prx` WASM
transformer plugin:

- Multi-field combination (concatenation, struct construction, value derivation from two or more
  source fields — e.g., CrowdStrike `ioc_type` + `ioc_value` → `evidences[0].data.type` +
  `evidences[0].data.value`).
- Non-RFC3339 / integer unix timestamp: source field is a unix epoch integer (e.g., Armis
  `last_seen` when present as `i64`); requires runtime branch on field type. The fallback chain
  (try string → try integer → use current time) observed in Armis mapper cannot be expressed
  declaratively.
- Array/list mapping: source array elements mapped to OCSF array paths (e.g., CrowdStrike
  `behaviors[*].tactic` → `attacks[*].tactic.name`). Requires iteration and struct construction.
- Enum value coercion: sensor-specific string enum mapped to OCSF integer id (e.g., CrowdStrike
  `severity` string → `severity_id` integer via `crowdstrike_severity_to_id` match table).
- Nested struct flattening: denormalization of a nested source object into multiple OCSF fields
  (e.g., CrowdStrike `device.hostname` → `device.name`, `device.device_id` → `device.uid`).
- Conditional mapping: output value depends on another field's runtime value (e.g., Armis
  source ID is `alertId` for record_type="alert", `id` for all others).
- Timestamp fallback chain: try multiple source fields in order with type-branch logic; fall
  back to current time with warning on exhaustion (Armis pattern). Requires imperative logic.
- Unit conversion (bytes to KB, milliseconds to seconds, etc.).

Total: 13 patterns — 5 TOML-mappable, 8 WASM-required. A VP-PLUGIN-006 fixture catalog will
test at least 6 representative cases — at least 3 that must resolve via TOML `ocsf_field`
annotation and at least 3 that must resolve via WASM transformer. This is authored in Wave 1/C
scope.

**Rule 2 — SensorAuth Trait Un-Sealing**

The `private::Sealed` marker pattern in `crates/prism-sensors/src/auth/mod.rs` will be removed
entirely. Cross-sensor auth-composition prevention moves from compile-time type-system
enforcement to runtime spec-validation enforcement at TOML load time. The spec-catalog
validation layer will reject specs that claim unsupported or incompatible auth combinations.
Plugin authors will implement `SensorAuth` directly without trait-sealing workarounds.
The `CustomAuth` placeholder duplicate in `crates/prism-spec-engine/src/custom_adapter.rs` will
be deleted — it served only as an escape hatch around the sealed trait and is unnecessary once
sealing is removed.

This ADR downgrades domain invariant DI-012 (sealed-auth-trait) from compile-time to
runtime enforcement. The specific runtime cross-sensor auth-composition rejection rules that
replace the sealed trait are:

- The spec `auth_type` field must be a single value from the enumerated set
  `{oauth2_client_credentials, bearer_static, cookie_roundtrip, api_key, custom_via_plugin}`.
  Mixed auth types in one spec are rejected at TOML load time.
- The `credential_refs` field must reference exactly one credential per auth method. Multiple
  credential types in one spec are rejected at TOML load time.
- The auth credential type (resolved via credential store) must structurally match the spec's
  `auth_type` variant. Mismatch is rejected at credential resolution time, before any HTTP
  request is issued.

These three rules preserve the original threat model (CrowdStrike OAuth tokens cannot be routed
through Cyberint cookie middleware) while moving the enforcement earlier in the pipeline
(spec-load time and credential-resolution time) rather than at compile time.

Wave 0/F (PLUGIN-PREREQ-F) executes the domain-invariant amendment concurrently with this rule.

**Rule 3 — TOML Authorship: Reverse-Engineered with DTU-Parity Tests**

Production TOML sensor specs for the four initial sensors will be reverse-engineered from
current Rust adapter behavior. Replacement sensor specs must produce parity record outputs
compared to the to-be-deleted Rust adapters when fed identical inputs against DTU clone
fixtures. Schema parity is exact (identical column count, types, and names). Row count must
be within 5% tolerance. Value parity uses the canonicalization rules defined in
TS-PLUGIN-PARITY-001 (authored in Wave 0/F or Wave 0/D). See VP-PLUGIN-003 for the full
parity criterion. This approach minimizes behavioral-regression risk during migration — the
DTU clone becomes the behavioral ground truth.

The Cyberint DTU clone in `prism-dtu-cyberint` has known gaps in `incidents` endpoint
pagination behavior. Wave 1/D must verify Cyberint DTU clone covers the `incidents` endpoint
pagination before authoring the parity test. The DTU gap is annotated in the PREREQ-B
constraint description.

**Rule 4 — CrowdStrike OAuth2-Refresh-on-401: In-Repo .prx WASM Plugin**

CrowdStrike's OAuth2 token refresh-on-401 flow will ship as an in-repo `.prx` WASM plugin,
built targeting `wasm32-wasi`, and loaded at boot by `PluginRuntime`. Plugin signing is
deferred to v1.0+1 per TD-PLUGIN-SIGNING-001 (P0, v1.0+1). The v1.0 plugin is unsigned and
loads with a boot-time security warning (see C4 and Negative Consequences). The plugin is NOT
implemented as in-tree Rust. It is built and loaded identically to third-party plugins via
the platform's `PluginRuntime`. The platform team ships its own auth-quirk plugin through the
same pipeline as external authors. This makes CrowdStrike the first real exercise of the in-repo
plugin path and validates the entire `.prx` build/load pipeline before external authors use it.

**Rule 5 — CustomAdapter Rust Trait Retirement**

The `CustomAdapter` Rust trait in `crates/prism-spec-engine/src/custom_adapter.rs` is removed.
The placeholder duplicate `SensorAuth` declaration in the same file is also removed (un-sealing
in Rule 2 eliminates its purpose). The `CustomAdapterRegistry` dead code in
`crates/prism-spec-engine/src/custom_adapter.rs` is deleted. The boot sequence dead code in
`crates/prism-bin/src/boot.rs` that instantiates and immediately drops both the registry and
the plugin runtime will be replaced with a live wiring step that loads `.prx` plugins via
`PluginRuntime` (Constraint C4). The `.prx` WASM plugin model becomes the SOLE escape hatch
for non-declarative sensor behavior. No Rust-trait-based escape hatch survives.

Rule 5 was confirmed by the user on 2026-05-10 in response to adversary pass-1 finding
F-MED-001 surfacing a prior orchestrator-derived caveat. The confirmation is durable and
user-decided.

Pre-condition for retirement: `cargo metadata` and `crates.io` confirm `prism-spec-engine`
has never been published with `CustomAdapter` exposed externally. The PLUGIN-AUDIT-001 HIGH-3
finding confirms no in-tree callers. Therefore same-burst removal is safe — no deprecation
period is required for external implementors. If a published version with `CustomAdapter` is
ever discovered, this rule is amended to add a one-cycle deprecation window with a
`#[deprecated]` annotation.

Wave 0/E scope note (F-CRIT-NEW-001-PASS2-RESIDUAL corrected): `spec_parser.rs` contains
zero `CustomAdapter` or `CustomAdapterRegistry` references (verified by grep). The live call
sites that must be retired before `custom_adapter.rs` is deleted are:
`crates/prism-spec-engine/src/lib.rs` (re-exports `CustomAdapter` and `CustomAdapterRegistry`
via `pub use custom_adapter::*` — the public API surface that must be removed),
`crates/prism-spec-engine/examples/demo_spec_loading.rs` (imports and exercises the registry
against a mock adapter — becomes dead after retirement), and
`crates/prism-spec-engine/tests/bc_2_16_004_test.rs` (BC test exercising `CustomAdapterRegistry`
— superseded when BC-2.16.004 is retired and replaced by WASM plugin tests). Migration of all
three sites is in scope for PLUGIN-PREREQ-E.

---

## Retired and Amended Contracts

This ADR directly retires one behavioral contract and amends one behavioral contract and one
domain invariant. The retirement and amendments take effect when Wave 0/F (PLUGIN-PREREQ-F)
lands. All spec frontmatter must be consistent before Wave 0/A through Wave 0/E dispatch.

**Retired — rust-escape-hatch behavioral contract (BC-2.16.004):**

The behavioral contract mandating `CustomAdapter` as a compile-time extensibility mechanism
with `DataSourceAdapter` trait bounds is retired. Effective date: when PLUGIN-PREREQ-F lands.
The `lifecycle_status` in the behavioral contract file transitions from `active` to `deprecated`,
with `deprecated_by: ADR-023`. The `.prx` WASM plugin model supersedes this contract as the
sole escape hatch. Rationale: a Rust-trait escape hatch alongside the WASM escape hatch creates
a two-tier author model that contradicts the eat-own-dog-food principle (Rule 5).

**Amended — datasource-trait-adapter-pattern behavioral contract (BC-2.01.013):**

The behavioral contract is amended to reflect the un-sealing of `SensorAuth` implications and
to adopt the runtime-validation rules and spec-driven adapter pattern introduced by Rule 2.
The amended contract removes any sealed-trait language and replaces it with the three runtime
cross-sensor auth-composition rejection rules documented under Rule 2. Full amendment executes
in PLUGIN-PREREQ-F.

**Amended — sealed-auth-trait domain invariant (DI-012):**

Domain invariant DI-012 is downgraded from compile-time enforcement (the `private::Sealed`
marker trait in `prism-sensors`) to runtime enforcement (spec-load time + credential-resolution
time validation). The threat model is preserved: the specific cross-sensor auth-composition
patterns that the sealed trait prevented are now prevented by the three runtime rejection rules
documented under Rule 2. Full amendment executes in PLUGIN-PREREQ-F.

**Sweep — sensor-named behavioral contracts (F-HIGH-NEW-004 enumerated):**

The following 8 sensor-named BCs will receive a prefix note in PLUGIN-PREREQ-F indicating that
a full amendment is pending and pointing to ADR-023. Full per-BC body amendment is authored in
Wave 2/G:

- BC-2.01.005-crowdstrike-oauth2-two-step-fetch
- BC-2.01.006-cyberint-cookie-auth
- BC-2.01.007-claroty-bearer-polymorphic-ids
- BC-2.01.008-armis-bearer-aql
- BC-2.02.003-crowdstrike-field-mapping
- BC-2.02.004-cyberint-field-mapping
- BC-2.02.005-claroty-field-mapping
- BC-2.02.006-armis-field-mapping

Each prefix note (added in PLUGIN-PREREQ-F scope) states: "This BC is being amended for
plugin-only architecture per ADR-023. The sensor auth and field-mapping behavior described here
will be replaced by TOML spec configuration and, where required, `.prx` WASM plugins. Full BC
amendment language is authored in PLUGIN-MIGRATION-001-G (Wave 2/G)."

BC-2.16.004 contradiction note: BC-2.16.004 lines 36–42 currently state "All four initial
sensors (CrowdStrike, Cyberint, Claroty, Armis) ship as pure TOML specs" — this directly
contradicts ADR-023 Rule 4, which specifies that CrowdStrike ships as TOML plus a `.prx` WASM
plugin for its OAuth2 refresh-on-401 flow. This contradiction window is bounded by PLUGIN-PREREQ-F
lead-time; PREREQ-F's retirement of BC-2.16.004 resolves it.

---

## Forbidden Patterns

The following patterns are prohibited in production source files — files under the production
crates listed below. Test-only code (under `tests/` or within `#[cfg(test)]` blocks) is exempt
from items marked `[test-ok]`.

**Production crate scope (derived from `cargo metadata` workspace members):**

```
crates/prism-core/src/
crates/prism-credentials/src/
crates/prism-security/src/
crates/prism-storage/src/
crates/prism-audit/src/
crates/prism-mcp/src/
crates/prism-ocsf/src/
crates/prism-spec-engine/src/
crates/prism-sensors/src/
crates/prism-query/src/
crates/prism-bin/src/
crates/prism-customer-config/src/
```

Forward-compatibility rule: any new `[lib]` crate added under `crates/prism-*/src/` is
automatically in scope unless explicitly exempted by an amendment to this ADR. Test crates
(`crates/prism-dtu-*/`) and plugin source crates (`crates/plugins/*/`) are explicitly out of
scope per the Permitted Patterns section. The `ocsf-proto-gen` build-helper crate is out of
scope (build-only, no production sensor dispatch).

DTU clone crates correctly named `crates/prism-dtu-{sensor}/` are exempt from
forbidden-pattern checks because they are dev-dependency-only test fixtures.

| Pattern | Original location | Enforcement |
|---|---|---|
| `pub mod crowdstrike;` / `pub mod armis;` / `pub mod claroty;` / `pub mod cyberint;` under `crates/prism-sensors/` | `auth/mod.rs`, `lib.rs` | compile-fail perimeter test + CI grep |
| `pub use crowdstrike::CrowdStrikeAdapter` / `pub use armis::ArmisAdapter` / `pub use claroty::ClarotyAdapter` / `pub use cyberint::CyberintAdapter` in `crates/prism-sensors/src/lib.rs` | `lib.rs` | compile-fail perimeter test + CI grep |
| `match SensorType::` dispatch in production code | any production crate | CI grep gate |
| `enum SensorType` definition with hardcoded sensor variants | `prism-core/src/types.rs` | CI grep gate |
| `private::Sealed` in any sensor-related trait | `auth/mod.rs` | CI grep gate |
| `crate::custom_adapter::CustomAdapter` in production paths | `prism-spec-engine` | CI grep gate |
| `init_registry_for_org` or any successor with hardcoded per-sensor credential parameter lists | `lib.rs` | CI grep gate |
| `prism_ocsf::mappers::<sensor_name>` module paths | `prism-ocsf/src/mappers/` | compile-fail perimeter test |

**Forbidden symbols catalog (FORBIDDEN-SYMBOLS-001) — exact type names banned from production imports:**

`CrowdStrikeAdapter`, `CrowdStrikeAuth`, `ArmisAdapter`, `ArmisAuth`, `ClarotyAdapter`,
`ClarotyAuth`, `CyberintAdapter`, `CyberintAuth`, `SensorType` (the closed enum).

Total: 9 reserved type names. The compile-fail perimeter test crate at
`tests/external/no-hardcoded-sensors/` will have one test file per forbidden symbol (9+ files),
each attempting `use prism_sensors::<ForbiddenType>;` and asserting compile failure.

Enforcement is via three mechanisms:

1. A compile-fail perimeter test crate at `tests/external/no-hardcoded-sensors/` (modeled on
   the existing perimeter at `tests/external/perimeter-violation/`). Each forbidden symbol
   (FORBIDDEN-SYMBOLS-001) gets one `tests/import_forbidden_<type>.rs` file.
2. A CI grep gate scanning for the disallowed string patterns on every PR to `develop`.
3. A code-review checklist in `.github/PULL_REQUEST_TEMPLATE.md` (created in Wave 0/D as part
   of the PluginRuntime wiring deliverables — F-LOW-NEW-001: repo-relative path, not absolute).
   Checklist items (minimum):
   - "Does this PR introduce any `pub mod <sensor_name>;` or `pub use <SensorAdapter>` patterns? If yes, this PR is not mergeable."
   - "Does this PR construct concrete sensor types (`CrowdStrikeAuth::new`, `ArmisAuth::new`, etc.) outside test code or .prx WASM plugins? If yes, this PR is not mergeable."
   - "Does this PR add new `match SensorType::Variant` patterns? If yes, this PR is not mergeable — use spec-catalog lookup against `SensorId`."

Active enforcement status (F-LOW-NEW-002): Until PREREQ-D lands and creates
`.github/PULL_REQUEST_TEMPLATE.md`, enforcement mechanism 3 (PR-template checklist) is
INACTIVE. The CI grep gate (mechanism 2) and compile-fail perimeter test crate (mechanism 1,
scoped to PREREQ-D delivery) are the active enforcement layers during the migration window.
Once PREREQ-D merges, all three mechanisms are active.

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
  in-process Rust modules and do NOT participate in the production type system. Plugin source
  crates are excluded from the main workspace `[workspace.members]` list; each declares its own
  `[lib]` with `crate-type = ["cdylib"]` targeting `wasm32-wasi`. Built via
  `just plugin-build <name>`. Output artifacts go to
  `target/wasm32-wasi/release/<name>.wasm` then post-processed to `<name>.prx`. The
  `wasm32-wasi` rustup target is a required dev toolchain component (added to `dev-setup.sh`
  in PLUGIN-PREREQ-D scope). Plugin source crates do NOT participate in `cargo build`,
  `cargo test`, or `cargo clippy` workspace runs.
- Compile-fail tests, integration test fixtures, and DTU-backed integration tests may name
  sensors — test-only context is acceptable.
- `SensorId(Arc<str>)` newtype in `prism-core` — the open replacement for `SensorType`.

---

## Architectural Constraints

For Decision Rules 1–5 to be enforceable, the following infrastructure must exist before
deletion of the hardcoded adapters. Wave 0/F lands first; Constraints C1–C5 (PLUGIN-PREREQ-A
through E) all depend on PLUGIN-PREREQ-F having landed.

**Wave 0/F — BC + DI Catalog Amendments (PLUGIN-PREREQ-F, NEW, 5–8 SP, LANDS FIRST — F-HIGH-NEW-004: SP updated to reflect 8 additional sensor-named BC prefix-note edits):**

Before any code changes begin, the behavioral contract and domain invariant catalog must be
consistent with this ADR. PLUGIN-PREREQ-F tasks:
- Deprecate the rust-escape-hatch behavioral contract (BC-2.16.004): `lifecycle_status:
  active` to `deprecated`; `deprecated_by: ADR-023`; body note retiring the trait.
- Amend the datasource-trait-adapter-pattern behavioral contract (BC-2.01.013): remove sealed-trait
  language; add runtime-validation rules and spec-driven adapter pattern per Rule 2.
- Amend the sealed-auth-trait domain invariant (DI-012): downgrade from compile-time to
  runtime enforcement; document the three cross-sensor auth-composition rejection rules per Rule 2.
- Sweep sensor-named behavioral contracts: add prefix note "This BC is being amended for
  plugin-only architecture per ADR-023; see PLUGIN-MIGRATION-001-D for replacement TOMLs."
  Full amendment language lands in Wave 2/G.
- Author test-strategy document TS-PLUGIN-PARITY-001 (canonicalization rules for VP-PLUGIN-003
  DTU-parity evaluation).

Acceptance criterion: BC catalog updated, all spec frontmatter consistent. No code changes in
this story. All stories PLUGIN-PREREQ-A through E and all Wave 1 stories depend on this story.

**C1 — SensorId newtype (PLUGIN-PREREQ-A):** `SensorId(Arc<str>)` open newtype replaces the
closed `SensorType` enum in `prism-core`. `SensorAdapter::sensor_type` return type changes to
`SensorId`. `AdapterRegistry` storage changes from a `SensorType`-keyed map to a
`SensorId`-keyed map. All downstream `match SensorType::X` arms across seven locations in four
crates are converted to spec-catalog lookup stubs. This is the keystone change — approximately
15 files across 5 crates; atomic 15-file commit aligned with the single-commit-per-burst
protocol (TD-VSDD-053). The whole rename is one commit; if review is too large, use a draft PR
with reviewable annotations rather than commit-splitting.

Pre-implementation note (F-MED-NEW-001-PASS2-RESIDUAL corrected): `SensorType` is a plain Rust
enum in `crates/prism-core/src/types.rs` with derives `Clone, Copy, Debug, PartialEq, Eq, Hash,
Serialize, Deserialize` and a hand-written `Display` impl. There are NO strum derives and NO
proc-macro-generated match arms (grep confirms zero strum references in prism-core). All
pattern-match sites are enumerable via `rg 'match\s+\w+|SensorType::\w+'` across the workspace.
The PLUGIN-AUDIT-001 CRIT-2 through CRIT-7 findings cite 7 downstream match sites in 4 crates
— these are plain `match` arms and are fully visible to line-level grep and static analysis.

Depends on: PLUGIN-PREREQ-F.

**C2 — Real PipelineExecutor (PLUGIN-PREREQ-B):** The `PipelineExecutor::execute` stub in
`crates/prism-spec-engine/src/pipeline.rs` (currently returns `Ok(Vec::new())`) is replaced
with a real implementation: HTTP client (reqwest), JSONPath extraction (serde_json pointer or
jsonpath-lib), pagination loop consuming cursor fields, retry with exponential backoff, and
WASM plugin hook invocation for the `retry_action = "refresh_auth"` hook point. Without this,
the TOML spec model is non-functional at runtime regardless of all other migrations.

Note: the Cyberint DTU clone has a known gap in `incidents` endpoint pagination behavior.
PREREQ-B must annotate this gap; Wave 1/D must verify Cyberint DTU clone coverage before
authoring the parity test.

Depends on: PLUGIN-PREREQ-F.

**C3 — TOML grammar extensions (PLUGIN-PREREQ-C):** The grammar in
`crates/prism-spec-engine/src/spec_parser.rs` is extended. Revised scope (3–5 SP, not 5–8 SP):

Already present in `spec_parser.rs` — verify behavior matches spec before ship:
- `pagination: Option<PaginationConfig>` with `CursorToken`/`OffsetLimit` variants (lines ~38–45)
- `columns[N].ocsf_field` column-level OCSF annotation (line ~87)
- `TableSpec.table_name` canonical field (referenced at pipeline.rs line ~63)

New grammar work (the actual PREREQ-C scope):
- `[fetch_step.retry]` with `retry_on_status = [...]` and `retry_action = "refresh_auth"` —
  does not exist; new work.
- `columns[N].virtual_field_aliases` for virtual field resolution — does not exist; new work.
- `cache_ttl_secs` for invalidation policy — does not exist; new work.
- `[fetch_step.batch]` with `{ ids_from_step, batch_size, batch_method, batch_body_template }` —
  partial; existing `FetchStep` supports steps via `variables_produced` but the explicit batch
  construct is new work.

All extensions are additive — existing TOML files remain valid. Depends on: PLUGIN-PREREQ-F.

**C4 — PluginRuntime wired into boot, .prx pipeline functional (PLUGIN-PREREQ-D):** PREREQ-D
delivers both the `PluginRuntime` infrastructure (engine, linker, loader, epoch-interruption
config) AND wires it into boot.rs step 7, replacing the dead instantiation with a live
`PluginRuntime::load_all_plugins` call (F-MED-NEW-005: PREREQ-D owns step 7; PREREQ-E owns
only step-8 dead-code deletion). Boot step 7 in `crates/prism-bin/src/boot.rs` will load `.prx`
WASM plugins from the plugin directory via `PluginRuntime`. Plugin signing is deferred to
v1.0+1 per TD-PLUGIN-SIGNING-001.

v1.0 plugin load behavior:
- Boot-time stderr and `tracing::warn!` message on every startup: "WARNING: Plugin signing not
  yet implemented (TD-PLUGIN-SIGNING-001). Loaded plugins are NOT cryptographically verified.
  Do not run untrusted plugins."
- Audit log entry on every plugin load: `event_type: plugin_load_unsigned`, `plugin_path: ...`,
  `plugin_hash: <sha256>`.
- `PRISM_DISABLE_PLUGIN_LOAD=1` environment variable: skip plugin loading entirely (emergency
  escape valve).

Sandbox model: wasmtime-based, WASI subset. Plugins have NO direct WASI network sockets and NO
filesystem access. Outbound HTTP is permitted only via the host-provided `host_http_request`
host function.

Current state (as-built, F-CRIT-NEW-002): `make_host_state()` in
`crates/prism-spec-engine/src/plugin/mod.rs` constructs `HostState { allowed_urls: None, ... }`
with an explicit `TODO(S-4.08)` comment. The `host_http_request` implementation in
`crates/prism-spec-engine/src/plugin/host_functions.rs` short-circuits when `allowed_urls` is
`None`, permitting ALL URLs. There is NO plugin-load-time allowlist validation in the current
codebase.

Target state (delivered by PREREQ-D): The `.prx` plugin manifest format is extended with an
`allowed_urls: [String]` field. `PluginRuntime::load_plugin` parses the manifest and constructs
`HostState { allowed_urls: Some(parsed_hostnames) }`. The `TODO(S-4.08)` in `mod.rs` is closed
by PREREQ-D. At that point, `host_http_request` enforces host-only comparison (not substring
matching) against the allowlist. The CrowdStrike OAuth refresh plugin manifest must declare the
CrowdStrike token endpoint hostname (cloud-region-aware). Direct WASI network syscalls remain
prohibited; all network I/O must flow through the declared host function interface.

The `.prx` plugin manifest format declares name, version, format_version, and hook points
(`retry_action`, `ocsf_transform`). The loader validates manifest `format_version` against
`CURRENT_SUPPORTED_VERSION` (a crate constant); plugins with `format_version` exceeding the
supported version are rejected with a clear error.

The host function import list in `host_functions.rs` must be validated against the
`wasmtime::Linker` registration list at build time via a `#[cfg(test)]` assertion. This prevents
import list drift as new host functions are added.

PR template creation (`.github/PULL_REQUEST_TEMPLATE.md` with the three-item sensor-pattern
checklist) is delivered in this story, making PREREQ-D the gating infrastructure delivery.

Until this lands, the in-repo CrowdStrike OAuth2 refresh plugin cannot be loaded at boot.
Depends on: PLUGIN-PREREQ-F.

**C5 — SensorAuth un-sealed, CustomAdapter removed (PLUGIN-PREREQ-E):** The `private::Sealed`
marker will be removed from `SensorAuth` in `crates/prism-sensors/src/auth/mod.rs`. The
`CustomAuth` duplicate will be deleted from `crates/prism-spec-engine/src/custom_adapter.rs`.
The `CustomAdapter` Rust trait is removed from the same file. The `CustomAdapterRegistry` dead
code is deleted. Boot step 8 cleanup: the dead `custom_adapter_registry` instantiation in
`crates/prism-bin/src/boot.rs` is removed (step 7 live wiring is delivered by PREREQ-D; PREREQ-E
only deletes the now-dead step-8 code). The actual `CustomAdapter` call sites that must be
retired before `custom_adapter.rs` is deleted are the re-export in `lib.rs`, the example in
`examples/demo_spec_loading.rs`, and the BC test in `tests/bc_2_16_004_test.rs` — all three are
in scope for this story (F-CRIT-NEW-001-PASS2-RESIDUAL: spec_parser.rs has zero such references).

Depends on: PLUGIN-PREREQ-F, PLUGIN-PREREQ-D (for live PluginRuntime wiring at step 7).

---

## Verification Properties

**VP-PLUGIN-001 — No production hardcoded sensor references:** No production source file in the
production crates listed in the Forbidden Patterns section references any of the 9 forbidden type
names in FORBIDDEN-SYMBOLS-001 (`CrowdStrikeAdapter`, `CrowdStrikeAuth`, `ArmisAdapter`,
`ArmisAuth`, `ClarotyAdapter`, `ClarotyAuth`, `CyberintAdapter`, `CyberintAuth`, `SensorType`).
Enforced by a compile-fail perimeter test crate at `tests/external/no-hardcoded-sensors/` plus
a CI grep gate. Each forbidden symbol gets one `tests/import_forbidden_<type>.rs` file
asserting compile failure. Both must pass on every PR targeting `develop`.

Additional enforcement (F-HIGH-NEW-005): A CI step asserts that the count of
`tests/import_forbidden_*.rs` files equals the count of entries in the FORBIDDEN-SYMBOLS-001
catalog (9 entries as of v1.2). Count mismatch fails CI with the message: "FORBIDDEN-SYMBOLS-001
catalog has N entries but tests/external/no-hardcoded-sensors/ contains M compile-fail test
files — add or remove files to restore 1:1 correspondence." A positive-coverage log line is
emitted on success: "Perimeter check passed: N forbidden symbols verified by N compile-fail
tests" (per POL-11 ci_positive_coverage_assertion).

**VP-PLUGIN-002 — Unknown sensor registers without code change:** A `prism start --spec-dir
<fixture>` invocation with a previously-unknown sensor TOML (for example,
`hypothetical_sensor.sensor.toml`) will:
- (a) Show the sensor in MCP `list_sensors` tool output.
- (b) Show the sensor's tables in PrismQL `SHOW TABLES IN <sensor_id>`.
- (c) Succeed on at least one field-level query: `SELECT field FROM <sensor_id>.<table> LIMIT 1`.
- (d) Require zero Prism source-code changes (verified by git diff empty).
- (e) Resolve OCSF mapping correctly for any column with `ocsf_field` declared.

This is the acceptance criterion for the architecture mandate — verified by an end-to-end
integration test in Wave 1.

**VP-PLUGIN-003 — DTU parity:** Output-record parity between the deleted Rust adapter path and
the replacement TOML plus `.prx` plugin path, validated against DTU clone fixtures per sensor.
Full parity criterion (see TS-PLUGIN-PARITY-001 for the canonical test-strategy document):

- Schema parity: column count, column types, and column names are exact-match (byte-identical).
- Row-count parity: within 5% tolerance (accommodates DTU clone non-determinism).
- Value parity for canonical OCSF projection: values byte-identical after canonicalization.

Canonicalization rules (from TS-PLUGIN-PARITY-001):
- Timestamps: stripped to date+hour granularity (ignores millisecond drift in DTU response).
- Request IDs: stripped (response-time-dependent).
- JSON object key order: normalized to alphabetical.
- Floating-point values: tolerance ±1 ULP for f64.
- Nullable fields: `null` and absent-key are treated as equal.

Intentional-divergence allowlist: a PR author may flag a record-level divergence as intentional
with reason; reviewer approves. Approved divergences are maintained in TS-PLUGIN-PARITY-001
§"Approved Divergences". One parity test per sensor (four total), gated in Wave 1 before any
adapter deletion.

**VP-PLUGIN-004 — Boot warning fires on unsigned plugin load (v1.0 scope):** For v1.0, this
property verifies the unsigned-plugin boot-warning behavior (signing is deferred to v1.0+1 per
TD-PLUGIN-SIGNING-001). Acceptance criteria: (a) boot-time WARN-level log message fires on
every startup with plugins present, (b) audit log entry with `event_type: plugin_load_unsigned`
and `plugin_hash: <sha256>` is recorded for every `.prx` file loaded, (c) `PRISM_DISABLE_PLUGIN_LOAD=1`
causes zero plugins to load and no warning fires. For v1.0+1: property will be amended to
assert that unsigned plugins fail to load with a structured error.

Test implementation details (F-MED-NEW-002): Test location:
`crates/prism-bin/tests/plugin_load_warning.rs`. Fixture: a minimal valid `.prx` at
`tests/fixtures/minimal.prx` (either committed as a pre-built artifact or generated via
`just plugin-build minimal`). Mechanism: `tracing-test::traced_test` macro to capture WARN-level
log output in the test thread; `tempfile::TempDir` for the audit-log sink directory. Assertion:
the audit JSONL file in the temp directory contains exactly one entry with
`event_type: "plugin_load_unsigned"` and a `plugin_hash` field whose value matches the
SHA-256 hex digest of the `minimal.prx` file bytes.

**VP-PLUGIN-005 — OAuth2 refresh-on-401 via .prx plugin:** The TOML grammar's
`retry_action = "refresh_auth"` hook correctly invokes the in-repo CrowdStrike `.prx` plugin
on an HTTP 401 response. The plugin acquires a new token via the `host_http_request`
allowlist-mediated host function and retries the request.

Test implementation details (F-MED-NEW-003): 401-injection uses a `wiremock::MockServer` with
a sequential responder (first request to the data endpoint returns HTTP 401, second returns
HTTP 200 with fixture data). No DTU clone enhancement is required — wiremock (version 0.6,
already in `crates/prism-sensors/Cargo.toml`) provides the mock server. The test asserts:
(a) the first data request receives 401, (b) `host_http_request` subsequently fires to the
CrowdStrike token endpoint (captured by a second wiremock mock on the token endpoint path),
(c) the retry data request succeeds with 200, (d) the final result set is non-empty.

**VP-PLUGIN-007 — Plugin manifest allowlist not-None after PREREQ-D (F-CRIT-NEW-002):**
After PREREQ-D lands, no `.prx` plugin loaded by `PluginRuntime` may have `allowed_urls = None`
in its `HostState`. Boot integration test (PREREQ-D scope): attempt to load a plugin whose
manifest omits the `allowed_urls` field — `PluginRuntime::load_plugin` must reject it with a
structured error (not silently default to allow-all). Acceptance criteria: (a) manifest without
`allowed_urls` field → load fails with error citing missing field, (b) manifest with
`allowed_urls: []` (empty list) → load succeeds but all HTTP requests are blocked (403 from
`host_http_request`), (c) manifest with `allowed_urls: ["api.crowdstrike.com"]` → requests to
that host succeed; requests to any other host are blocked.

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
WASM function is cleaner. The 80/20 split was the user's explicit decision. The closed grammar
above makes the boundary machine-verifiable.

Un-sealing `SensorAuth` (Rule 2) is the minimum change required to unblock external auth
implementors. The sealed trait was presumably introduced to prevent cross-sensor auth
composition bugs, but this is a blunt instrument that also prevents any external auth extension.
Moving the enforcement to runtime spec-validation is more precise: the three runtime rejection
rules explicitly block the threat model cases (CrowdStrike OAuth tokens routed through Cyberint
cookie middleware) while permitting valid novel auth implementations.

Retiring the `CustomAdapter` Rust trait (Rule 5) is the dogfood completion step. Rule 5 was
confirmed by the user on 2026-05-10. If the platform team maintained a Rust-trait escape hatch
alongside the WASM escape hatch, third-party plugin authors would reasonably ask why they cannot
use the Rust path. The answer — "that path is for us, not you" — contradicts the eat-own-dog-food
principle. One escape hatch (WASM), used by everyone including the platform team, is the correct
invariant.

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
- WASM sandbox provides stronger isolation than `catch_unwind` (process-level vs thread-level),
  delivering superior fault tolerance relative to the retired CustomAdapter path.

### Negative / Trade-offs

- High implementation cost: approximately 94–146 story points across 14 stories in 3 waves (F-MED-NEW-004 corrected)
  (including new Wave 0/F).
- Increased runtime complexity: WASM toolchain, `.prx` build pipeline, `wasm32-wasi` build
  targets, and sandbox enforcement add operational surface area.
- Error-class shift: spec-catalog validation errors that were previously compile-time type
  errors (e.g., invalid `SensorType` variant) become spec-load-time runtime errors. Operators
  encounter these at startup, not build time.
- The `PipelineExecutor` stub in `crates/prism-spec-engine/src/pipeline.rs` is the longest-pole
  item in Wave 0 and blocks all Wave 1 deletions — until it ships, the TOML spec model produces
  no data.
- WASM cold-start latency: CrowdStrike auth refresh pays `instantiate(&mut store)` overhead
  per call. Mitigation (as-built): `PluginRuntime::load_plugin` precompiles each plugin's WASM
  binary into a `wasmtime::component::InstancePre<HostState>` (stored in `LoadedPlugin`). Per
  call, the cost is `InstancePre::instantiate(&mut store)` — approximately 1ms — rather than
  a full recompile. True live-instance pooling (a pool of ready `(Store, Instance)` pairs) does
  NOT exist in the current `PluginRuntime` struct (`engine`, `linker`, `registry`, `http_client`,
  `_epoch_ticker` — no pool field). If the `bench_plugin_invocation` benchmark in
  `crates/prism-spec-engine/benches/` shows per-call latency exceeding the sub-200ms tool
  latency target, PREREQ-D MAY add an instance pool as new work. That decision is deferred to
  benchmark results, not pre-committed (F-HIGH-NEW-001 corrected). Performance budgets per the
  existing 200MB-per-query memory budget behavioral contract and sub-200ms tool latency targets.
- Plugin call observability: `tracing` span propagation from Rust into WASM and back is
  non-trivial. Mitigation: host-side spans wrap each plugin call; plugin-internal logs surface
  via host log capture (existing infrastructure in `host_functions.rs`).
- Rollback story: a corrupt `.prx` plugin discovered post-load cannot be unloaded without
  restart (unless hot-reload replaces it). Mitigation: leverage existing PluginRuntime
  hot-reload for replacement; `PRISM_DISABLE_PLUGIN_LOAD=1` for emergency disable. Once Rust
  adapters are deleted in Wave 1/A, reverting requires reverting multiple merged PRs.
- Plugin version skew: an operator upgrades `prism-bin` while retaining older `.prx` plugins.
  Mitigation: WASM Component Model interface-typing version check at plugin-load time
  (manifest `format_version` validation added to loader); fail-fast on incompatibility with
  a clear structured error.
- Plugin debugging: stepping into sandboxed WASM is not supported by standard Rust debugging
  tooling. Mitigation: `PRISM_PLUGIN_DEV_MODE=1` environment variable enables verbose plugin
  tracing and retains intermediate artifacts.
- Panic isolation model change: the retired `CustomAdapter` Rust trait path used `catch_unwind`
  for thread-level panic isolation. The replacement `.prx` path uses Wasmtime epoch-interruption
  plus StoreLimits — a different fault model. Equivalence claim: WASM sandbox provides stronger
  isolation than `catch_unwind` (process-level isolation vs thread-level), at cost of
  approximately 50ms cold-start overhead. Net: superior fault tolerance with an explicit
  performance trade-off.
- v1.0 ships unsigned plugins. This is a known security exposure tracked as
  TD-PLUGIN-SIGNING-001 (P0, v1.0+1). Operators must avoid running untrusted plugins until
  signing lands. The boot-time warning and audit log entry make this exposure explicit and
  auditable.

### Status as of 2026-05-10

COMMITTED v1.2, pending implementation of Wave 0/F (PLUGIN-PREREQ-F) and Constraints C1–C5
(PLUGIN-MIGRATION-001 Wave 0). The five hardcoded sensor auth modules, the four OCSF mapper
modules, the `SensorType` enum, and the `CustomAdapter` trait all remain in the codebase until
their corresponding Wave 0/1 stories ship and pass DTU-parity gates.

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

PLUGIN-MIGRATION-001: 14 stories, 3 waves, approximately 94–146 SP (F-MED-NEW-004 corrected; Wave 0: 45–67, Wave 1: 37–60, Wave 2: 12–19), HIGH risk.

**Wave 0 — Prerequisites (6 stories, approximately 45–67 SP, no deletions — F-MED-NEW-004 corrected):**

- PLUGIN-PREREQ-F: BC + DI catalog amendments — deprecate rust-escape-hatch BC, amend
  datasource-trait-adapter BC, amend sealed-auth-trait DI, add prefix notes to 8 sensor-named
  BCs (BC-2.01.005 through BC-2.01.008, BC-2.02.003 through BC-2.02.006), author
  TS-PLUGIN-PARITY-001 (5–8 SP, F-HIGH-NEW-004 updated) — LANDS FIRST; all other stories depend
  on this.
- PLUGIN-PREREQ-A: `SensorType` to `SensorId(Arc<str>)` keystone migration (13–18 SP); depends
  on PREREQ-F.
- PLUGIN-PREREQ-B: Real `PipelineExecutor` — HTTP, JSONPath, pagination, retry, WASM hook
  (13–18 SP); depends on PREREQ-F.
- PLUGIN-PREREQ-C: TOML grammar extensions — new grammar only: `[fetch_step.retry]` with
  `retry_action`, `virtual_field_aliases`, `cache_ttl_secs`, `[fetch_step.batch]` (3–5 SP;
  revised from 5–8 SP per F-HIGH-006 close); depends on PREREQ-F.
- PLUGIN-PREREQ-D: Deliver `PluginRuntime` infrastructure AND wire it into boot.rs step 7
  (live plugin load replaces dead instantiation); build `.prx` load pipeline; unsigned-plugin
  boot warning + audit log; host function import validation; PR template creation; allowlist
  manifest field + TODO(S-4.08) closure (8–13 SP); depends on PREREQ-F. Step 7 wiring is
  in PREREQ-D scope (F-MED-NEW-005).
- PLUGIN-PREREQ-E: Un-seal `SensorAuth`; retire `CustomAdapter` Rust trait, its re-export in
  `lib.rs`, `examples/demo_spec_loading.rs`, and `tests/bc_2_16_004_test.rs`; delete
  `custom_adapter.rs`; remove dead step-8 `custom_adapter_registry` code from boot.rs
  (F-MED-NEW-005: PREREQ-E owns step-8 cleanup only, not step-7 wiring) (3–5 SP); depends on
  PREREQ-F and PREREQ-D.

**Wave 1 — Primary deletion and replacement (5 stories, approximately 37–60 SP, ordered:
REPLACEMENT BEFORE DELETION):**

- PLUGIN-MIGRATION-001-D (FIRST): Author 4 production TOMLs via reverse-engineering; DTU-parity
  tests per sensor (TS-PLUGIN-PARITY-001 canonicalization rules apply). Acceptance criterion:
  each sensor's TOML produces parity records versus current Rust adapter against DTU clone
  (VP-PLUGIN-003 passes for all 4 sensors) (8–13 SP).
- PLUGIN-MIGRATION-001-E (SECOND): Build CrowdStrike OAuth2-refresh-on-401 as in-repo `.prx`
  WASM plugin; integration test against DTU clone with 401-injection mode. Acceptance criterion:
  plugin handles 401 via `host_http_request` allowlist-mediated re-auth; VP-PLUGIN-005 passes
  (5–8 SP).
- PLUGIN-MIGRATION-001-B: Convert 5 prism-query dispatch sites to spec-catalog lookups —
  explain, write_dispatch, virtual_fields, invalidation, materialization. Acceptance criterion:
  all `match SensorType::Variant` patterns replaced; production code uses
  `ConfigSnapshot::sensor_specs.iter()` lookup (8–13 SP).
- PLUGIN-MIGRATION-001-C: Delete 4 OCSF mapper modules; implement `SpecDrivenMapper`; ship
  in-repo `.prx` WASM complex-transform plugins; author VP-PLUGIN-006 fixture catalog
  (6 representative cases). Acceptance criterion: per-sensor `mappers/{sensor}.rs` deleted;
  tests pass against new mapper (8–13 SP).
- PLUGIN-MIGRATION-001-A (LAST — CUTOVER): Delete `prism-sensors/src/auth/{4 sensors}.rs`
  files, lib.rs re-exports, and `init_registry_for_org`. Acceptance criterion: compile-fail
  perimeter test (VP-PLUGIN-001) passes — all 9 forbidden imports fail to compile
  (8–13 SP).

**Per-PR-boundary invariant:** At every PR boundary in Wave 1, ALL FOUR sensors must remain
functional via either (a) the legacy Rust adapter path (before Wave 1/A merges), or (b) the
new TOML+plugin path (after Wave 1/D and 1/E merge). Wave 1/A is the cutover commit: it deletes
the Rust path. Wave 1/A may not merge until VP-PLUGIN-003 (parity) passes for all four sensors
AND VP-PLUGIN-005 (CrowdStrike refresh) passes AND VP-PLUGIN-001 (compile-fail perimeter)
passes.

**Wave 2 — Cleanup (3 stories, approximately 12–19 SP — F-MED-NEW-004 corrected):**

- PLUGIN-MIGRATION-001-F: Rewrite approximately 10 sensor-named test files to TOML fixture
  loading; compile-fail perimeter test at `tests/external/no-hardcoded-sensors/` (5–8 SP).
- PLUGIN-MIGRATION-001-G: Doc sweep — module-decomposition, production-runtime-wiring decision
  record (inline note directing readers to ADR-023 and PLUGIN-PREREQ-F), BC catalog sensor-name
  grep, full body amendment of the 8 sensor-named BCs (BC-2.01.005 through BC-2.01.008 and
  BC-2.02.003 through BC-2.02.006), sensor-adapters.md (5–8 SP). Note: ADR-022 v1.2 amendment
  (adding `superseded_by_partial: ADR-023` annotation to §G Story 3) lands in Wave 1/A, NOT
  Wave 2/G. Wave 2/G scope is the BC body sweep only (F-HIGH-NEW-003 corrected).
- PLUGIN-MIGRATION-001-H: Story supersession — mark S-2.06, S-2.07, W3-FIX-S307-001,
  S-3.1.06-ImplPhase superseded in STORY-INDEX (2–3 SP).

**Risk posture:** HIGH. The `SensorType` keystone change (PLUGIN-PREREQ-A) touches approximately
15 files across 5 crates and must be atomic. The `PipelineExecutor` implementation
(PLUGIN-PREREQ-B) is the longest-pole story and blocks all Wave 1 deletions. Wave 1 deletions
carry behavioral regression risk mitigated by DTU-parity tests (VP-PLUGIN-003). The reordering
of Wave 1 (MIGRATION-001-D and -E before -A) eliminates the broken-develop window risk
identified in adversary pass-1 finding F-HIGH-007.

---

## Source / Origin

- **User directive:** 2026-05-10, verbatim: "we arent suppose to have anything built in,
  everything uses the plugin system. We need to do a full audit to make sure we are following
  that." This directive supersedes any prior story or ADR that implied sensor Rust authorship
  as a valid path.
- **User decisions (2026-05-10, verbatim):**
  - Rule 5 confirmed: "Confirm Rule 5 — retire CustomAdapter Rust trait, .prx WASM is sole
    escape hatch." Rule 5 is user-decided and durable.
  - Signing deferred: "Signing deferred to v1.0+1 — v1.0 uses unsigned plugins with explicit
    security warning at boot." Tracked as TD-PLUGIN-SIGNING-001 P0.
  - Wave 1 reordering: "Reorder Wave 1: ship replacements BEFORE deletion (Recommended)."
    Wave 1/D and 1/E land before Wave 1/A.
  - Wave 0/F added: "Add Wave 0/F: BC + DI amendments BEFORE any code changes (Recommended)."
    New PLUGIN-PREREQ-F lands first in Wave 0.
- **Adversary pass-1:** ADR-023-pass-1.md — 26 findings (4 CRIT / 9 HIGH / 7 MED / 4 LOW /
  5 OBS process-gap). 24 findings closed in this v1.1 amendment. 5 process-gap OBS findings
  tracked as TD items (TD-ADR-AMEND-001, TD-AUDIT-ADR-001, TD-USER-DECISION-001,
  TD-SIGNING-PREREQ-001, TD-ADR-OPEN-Q-001) at
  `.factory/cycles/wave-4-operations/td-from-adr-023-pass-1.md`.
- **Audit document:** `.factory/cycles/wave-4-operations/audits/plugin-only-violations-2026-05-10.md`
  (PLUGIN-AUDIT-001) — 21 violations across 10 crates, catalogued by codebase-analyzer on
  2026-05-10. All five user-decided positions are recorded in the audit's "User-Decided
  Migration Approach" section.
- **Amended document:** `ADR-022-production-runtime-wiring.md` Section G, Story 3 — the
  reference to "the four built-in sensor adapters" is retired by this ADR concurrent with
  PLUGIN-MIGRATION-001 Wave 1/A. ADR-022 v1.2 amendment (adding `superseded_by_partial: ADR-023`
  annotation and inline note) lands as a sub-deliverable of the Wave 1/A cutover commit, not
  Wave 2/G. Wave 2/G delivers the full BC body amendment for the 8 sensor-named BCs
  (F-HIGH-NEW-003 corrected).
- **Code as-built (violations):** Closed `SensorType` enum in `crates/prism-core/src/types.rs`;
  sealed `SensorAuth` trait in `crates/prism-sensors/src/auth/mod.rs`; stub `PipelineExecutor`
  in `crates/prism-spec-engine/src/pipeline.rs`; four concrete adapter re-exports in
  `crates/prism-sensors/src/lib.rs`.
- **Architecture intent:** `.factory/specs/architecture/sensor-adapters.md` — states "no
  compiled-in sensor-specific Rust code" as the adapter layer goal. This ADR makes that intent
  enforceable.

---

## Changelog

| Version | Date | Description |
|---------|------|-------------|
| v1.2 | 2026-05-10 | Closes 16 spec defects from adversary pass-2 (2 CRIT + 4 HIGH + 5 MED + 3 LOW) per TD-FIX-BURST-VERIFY-001 source-of-truth verification discipline. F-CRIT-NEW-001-PASS2-RESIDUAL closed: corrected CustomAdapter call-site enumeration — spec_parser.rs contains zero CustomAdapter references (grep confirmed); actual call sites are lib.rs re-export, examples/demo_spec_loading.rs, tests/bc_2_16_004_test.rs; both body locations updated. F-CRIT-NEW-002 closed: rewrote C4 sandbox section with explicit current-state vs target-state delineation — make_host_state() returns HostState with allowed_urls=None and TODO(S-4.08); host_functions.rs short-circuits None as ALL-permitted; no plugin-load-time allowlist validation today; target state (PREREQ-D) specified; VP-PLUGIN-007 added. F-HIGH-NEW-001 closed: corrected instance-pool claim — LoadedPlugin.pre_instance is InstancePre<HostState> (precompiled component, not a live-instance pool); no pool field exists in PluginRuntime; InstancePre-based reuse semantics described accurately. F-HIGH-NEW-002 closed: re-authored OCSF closed grammar — added 5 missing patterns from mapper source (array/list mapping, identity/no-op, RFC3339 string parsing with fallback chain, integer unix timestamp, integer-to-string cast); total is now 13 patterns (TOML: 5, WASM: 8). F-HIGH-NEW-003 closed: resolved ADR-022 v1.2 amendment timing contradiction — all three locations now consistently say ADR-022 v1.2 lands as sub-deliverable of Wave 1/A cutover commit; Wave 2/G retains only BC body sweep. F-HIGH-NEW-004 closed: enumerated 8 sensor-named BCs in Wave 0/F sweep; added amends_bcs_pending_full_amendment_in_wave_2_g frontmatter; updated PREREQ-F SP from 3-5 to 5-8; noted BC-2.16.004 contradiction window. F-HIGH-NEW-005 closed: added CI sync-check and positive-coverage log assertion to VP-PLUGIN-001. F-MED-NEW-001-PASS2-RESIDUAL closed: corrected SensorType strum claim — prism-core types.rs uses plain derives (Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize) with hand-written Display impl; no strum derives (grep confirmed zero strum references in prism-core). F-MED-NEW-002 closed: added test fixture details to VP-PLUGIN-004. F-MED-NEW-003 closed: scoped 401-injection via wiremock::MockServer (wiremock 0.6 already in prism-sensors); no DTU clone enhancement required. F-MED-NEW-004 closed: corrected SP arithmetic — actual sum is 94-146 SP (Wave 0: 45-67, Wave 1: 37-60, Wave 2: 12-19, inclusive of PREREQ-F update to 5-8 from F-HIGH-NEW-004); all six cited locations updated. F-MED-NEW-005 closed: resolved PREREQ-D vs PREREQ-E boot.rs ownership — PREREQ-D delivers PluginRuntime infra AND wires step 7; PREREQ-E only deletes dead custom_adapter_registry and step-8 code. F-LOW-NEW-001 closed: replaced absolute filesystem path with repo-relative .github/PULL_REQUEST_TEMPLATE.md. F-LOW-NEW-002 closed: added PR-template-inactive acknowledgment to enforcement section. F-LOW-NEW-003 closed: added 3 entries to inputs frontmatter. 2 OBS process-gap findings tracked as TDs. |
| v1.1 | 2026-05-10 | Closes 24 spec defects from adversary pass-1 (4 CRIT + 9 HIGH + 7 MED + 4 LOW). F-CRIT-001 closed: added amends_bcs/retires_bcs frontmatter + Retired/Amended Contracts section. F-CRIT-002 closed: added amends_dis frontmatter + DI-012 runtime rejection rules. F-CRIT-003 closed: replaced ungroundable glob with explicit crate list from cargo metadata; dropped prism-operations; added forward-compatibility rule. F-CRIT-004 closed: moved PR template to Wave 0/D; specified path + three checklist items. F-HIGH-001 closed: added closed grammar for ocsf_field (TOML-mappable vs WASM-required); added VP-PLUGIN-006 fixture catalog. F-HIGH-002 closed: deferred signing to v1.0+1 per TD-PLUGIN-SIGNING-001; updated Rule 4, C4, VP-PLUGIN-004, Negative Consequences with boot warning + audit log + escape valve. F-HIGH-003 closed: reconciled VP-PLUGIN-001 with actual codebase types; added 9-symbol FORBIDDEN-SYMBOLS-001 catalog. F-HIGH-004 closed: replaced byte-level parity with schema+row-count+canonical-value parity with canonicalization rules; referenced TS-PLUGIN-PARITY-001. F-HIGH-005 closed: rewrote C4 sandbox model to align with host_http_request allowlist; updated VP-PLUGIN-005. F-HIGH-006 closed: re-authored C3 with NEW vs already-present split; revised PREREQ-C estimate to 3-5 SP. F-HIGH-007 closed: reordered Wave 1 D→E→B→C→A with per-PR-boundary functionality invariant per user decision. F-HIGH-008 closed: expanded Negative Consequences with 6 omitted risk categories (cold-start, observability, rollback, version-skew, debugging, panic-isolation). F-HIGH-009 closed: added ADR-022 v1.2 amendment to Wave 2/G scope. F-MED-001 closed: removed orchestrator-derived caveat from Rule 5; added user-confirmation note. F-MED-002 closed: added same-burst removal justification; added pre-condition for retirement. F-MED-003 closed: cross-referenced VP-PLUGIN-003 for parity threshold. F-MED-004 closed: strengthened VP-PLUGIN-002 with 5-point acceptance criteria. F-MED-005 closed: added hot-reload semantics to C4. F-MED-006 closed: cross-referenced Rule 2 for auth composition rejection rules. F-MED-007 closed: added Wave 0/F (PLUGIN-PREREQ-F, 3-5 SP) per user decision; updated PREREQ-A through E depends_on. F-LOW-001 closed: added atomic-commit protocol note to PREREQ-A. F-LOW-002 closed: added performance budget and benchmark recipe to Negative Consequences. F-LOW-003 closed: added SS-22 to subsystems_affected. F-LOW-004 closed: added plugin source crate workspace integration rules to Permitted Patterns + C4. 5 OBS process-gap findings tracked as TDs, not closed here. |
| v1.0 | 2026-05-10 | Initial COMMITTED version — plugin-only sensor architecture mandate. |
