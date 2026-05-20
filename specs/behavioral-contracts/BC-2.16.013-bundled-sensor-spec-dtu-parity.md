---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-05-20T00:00:00Z
phase: 1a
origin: brownfield
subsystem: "SS-16"
capability: "CAP-029"
lifecycle_status: draft
introduced: "2026-05-20"
modified: "2026-05-20"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
  - ".factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md"
  - ".factory/specs/test-strategy/TS-PLUGIN-PARITY-001-dtu-canonicalization.md"
  - "crates/prism-sensors/src/auth/crowdstrike.rs"
  - "crates/prism-sensors/src/auth/claroty.rs"
  - "crates/prism-sensors/src/auth/cyberint.rs"
  - "crates/prism-sensors/src/auth/armis.rs"
input-hash: null
traces_to:
  - "CAP-029"
extracted_from: ".factory/specs/prd.md"
---

# BC-2.16.013: Bundled Sensor Spec Authoring and DTU-Parity Verification — 4 Initial Sensors

## Description

The four initial sensors (CrowdStrike, Cyberint, Claroty, Armis) ship as production TOML
spec files bundled at `crates/prism-sensors/specs/` within the prism repository. These files
are reverse-engineered from the existing hardcoded Rust adapter implementations to preserve
exact behavioral parity. Each spec is validated by the existing pipeline (BC-2.16.001,
BC-2.16.009, BC-2.16.002) and paired with a DTU-parity integration test that proves:
spec-driven dispatch against the corresponding DTU clone produces OCSF-normalized output
that is semantically equivalent — per TS-PLUGIN-PARITY-001 Rules A–I — to the reference
output produced by the prior hardcoded Rust adapter path against the same DTU clone for
the same raw API response payload.

This BC is the behavioral anchor for VP-PLUGIN-003 (VP-148) and the correctness gate for
PLUGIN-MIGRATION-001-A (deletion of the 4 hardcoded Rust adapter modules). PLUGIN-MIGRATION-001-A
MUST NOT proceed until VP-PLUGIN-003 is verified (green parity tests) for all 4 sensors.

## Preconditions

- S-PLUGIN-PREREQ-A through S-PLUGIN-PREREQ-E have all merged to develop: `SensorId` newtype,
  `PipelineExecutor` with `AuthProvider`, TOML grammar (`spec_parser.rs` full implementation),
  `PluginRuntime` boot wiring, and `SensorAuth` open trait / `WriteToolInvalidationMap` runtime
  extensibility are all in production code.
- `PluginRegistry` dispatch is wired in `spec_parser.rs` (BC-2.16.012 active).
- The `CustomAdapter` Rust trait has been removed (BC-2.16.011 active; `lifecycle_status: removed`).

### O-001 TOML Grammar Verification (FB-IMPL-P1-PO, 2026-05-20)

The following four TOML grammar features were verified against the canonical implementation at
`crates/prism-spec-engine/src/spec_parser.rs` and `crates/prism-spec-engine/src/pipeline.rs`:

| Field | Status | Evidence |
|-------|--------|---------|
| `fan_out_batch_size` on `FetchStep` | **SUPPORTED** | Present as `pub fan_out_batch_size: Option<u32>` in the `FetchStep` struct (`FetchStep::fan_out_batch_size` field); handled by `fan_out_batches` in the pipeline executor |
| `${query.filter.KEY}` interpolation | **SUPPORTED** | `FetchContext::query_filters: HashMap<String, String>` seeded into `step_vars` via `PipelineExecutor::execute_impl` `query.filter.{k}` step-vars seeding; Armis AQL must use `${query.filter.aql}`, not the non-existent `${query.aql}` shorthand |
| `timestamp_format = "multi"` | **NOT SUPPORTED — Grammar Extension Required** | No such field exists in `FetchStep`, `TableSpec`, `ColumnSpec`, or `SensorSpec` in `spec_parser.rs`. This extension is NOT declaratively expressible in the current TOML grammar. |
| `timestamp_fallback_chain = [...]` | **NOT SUPPORTED — Grammar Extension Required** | No such field exists anywhere in `spec_parser.rs`. This extension is NOT declaratively expressible in the current TOML grammar. |

**Consequence:** The `cyberint.sensor.toml` multi-format timestamp behavior and `armis.sensor.toml`
timestamp fallback chain behavior CANNOT be expressed declaratively in the current TOML grammar.
Two prerequisite options exist:

**Option A (Grammar Extension):** Add `timestamp_format` and `timestamp_fallback_chain` fields to
`ColumnSpec` or `TableSpec` in `spec_parser.rs` as part of PLUGIN-MIGRATION-001-D or a dedicated
sub-story `PLUGIN-MIGRATION-001-D.1`. If implemented in the main story, these fields must be
deserialized via `#[serde(default)]` and handled in the pipeline executor.

**Option B (WASM Plugin):** Implement the timestamp parsing and fallback logic as an in-repo
`.prx` WASM transformer plugin per ADR-023 §Decision Rules Rule 1 (complex transforms). The spec
would reference the plugin via `(sensor_id, table)` dispatch in the SpecDrivenMapper.

The implementer MUST choose Option A or Option B and implement it before authoring the Cyberint
and Armis TOML specs. This BC treats both as valid — the postcondition is that the spec produces
parity output, not that it uses a specific grammar mechanism. **This is not a deferral —
implementing the grammar extension or WASM plugin is in scope for PLUGIN-MIGRATION-001-D.**
If the implementer discovers that the scope cannot be completed in one story, the orchestrator
must be notified for sub-story creation.

- DTU clones for all 4 sensors are built and available in the test harness:
  - `prism-dtu-crowdstrike` (S-6.07): OAuth2 token endpoint + two-step Falcon API (QueryV2 + PostEntities)
  - `prism-dtu-claroty` (S-6.08): Bearer token auth + POST-for-read + offset pagination
  - `prism-dtu-cyberint` (S-6.09): Cookie-roundtrip auth + multi-format timestamp responses
  - `prism-dtu-armis` (S-6.10): Bearer + AQL query forwarding + timestamp fallback chain
  Note: if DTU clones are not yet built, the parity tests are in SKIP status per TS-PLUGIN-PARITY-001
  Rule H (SKIP condition) — this BC still governs the spec authoring obligation.
- Fixture payloads (real-sensor recordings or synthesized per TS-PLUGIN-PARITY-001 Rule I) exist
  at `crates/prism-dtu-{sensor}/fixtures/parity/` with minimum 3 real-sensor recordings AND
  3 synthesized cases per `(sensor_id, table)` pair.

## Postconditions

### 1. Spec Files Authored and Validated

Four production TOML sensor spec files are created at `crates/prism-sensors/specs/`:

- `crowdstrike.sensor.toml` — `sensor_id: "crowdstrike"`, `auth_type: "oauth2_client_credentials"`,
  base URL pattern `https://api.{cloud_region}.crowdstrike.com`, tables:
  - `detections` — QueryV2 step (GET `/queries/detections`) → PostEntities step
    (POST `/entities/detections/GET`) with batch size ≤ 100 (CROWDSTRIKE_BATCH_SIZE)
  - `devices` — QueryV2 step (GET `/queries/devices`) → PostEntities step
    (POST `/entities/devices/GET`) with batch size ≤ 100
  - `incidents` — QueryV2 step (GET `/queries/incidents`) → PostEntities step
    (POST `/entities/incidents/GET`) with batch size ≤ 100
  URL patterns derived from `crowdstrike.rs`: `query_resource_ids` uses
  `format!("{}/queries/{}", self.base_url, resource_type)` (crowdstrike.rs:262) and
  `fetch_entities` uses `format!("{}/entities/{}/GET", self.base_url, resource_type)`
  (crowdstrike.rs:315); `resource_type` is derived via `resource_type_from_spec()` which
  strips `"crowdstrike_"` prefix and pluralizes (crowdstrike.rs:369-375).
  Each table's columns match the Arrow schema produced by the prior Rust adapter (BC-2.01.005
  field enumeration); OCSF field mappings reproduce the prior `CrowdStrikeAdapter::fetch()`
  (`SensorAdapter::fetch` trait method, `crowdstrike.rs`) normalization. Version: `"1.0.0"`. Rate limit hints: `requests_per_second: 10.0`.

- `claroty.sensor.toml` — `sensor_id: "claroty"`, `auth_type: "cookie_roundtrip"`,
  base URL from instance_url, tables:
  - `assets` — POST `/api/v1/assets` (POST-for-read pattern) with offset pagination
  - `alerts` — POST `/api/v1/alerts` with offset pagination
  - `audit_logs` — GET `/api/v1/audit_logs` via `paginate_claroty()` stream semantics
  URL patterns derived from `claroty.rs:endpoint_from_spec()` (claroty.rs:238-244):
  `"audit_logs"` → `"/api/v1/audit_logs"` (special case, claroty.rs:240-241); all other
  tables strip `"claroty_"` prefix, pluralize, and prepend `"/api/v1/"` (claroty.rs:243-244).
  NO `/xdome` prefix — that was a phantom; the code emits `/api/v1/{resource}s` directly.
  Polymorphic ID handling: `ClarotyId` (int or UUID string) expressed as column type `string`
  with OCSF `raw_extensions` passthrough. Version: `"1.0.0"`.

- `cyberint.sensor.toml` — `sensor_id: "cyberint"`, `auth_type: "bearer_static"`,
  base URL from environment (`https://{environment}.cyberint.io`), tables:
  - `alerts` — GET `/api/alerts` with cursor pagination
  - `incidents` — GET `/api/incidents` (Cyberint DTU gap: parity tests in SKIP per
    TS-PLUGIN-PARITY-001 Cyberint DTU Gap Note until DTU coverage of `incidents` pagination
    behavior is verified)
  URL patterns derived from `cyberint.rs:endpoint_from_spec()` (cyberint.rs:244-251):
  strips `"cyberint_"` prefix and pluralizes with `/api/` prefix (cyberint.rs:251:
  `format!("/api/{resource}s")`). NO `/v1` segment — that was a phantom.
  Multi-format timestamp parsing (`parse_timestamp()`) is expressed via column `type: "datetime"`
  with a WASM transformer plugin for multi-format parsing (see O-001 Grammar Verification note
  in §Preconditions — `timestamp_format: "multi"` is NOT present in the current TOML grammar
  and requires a grammar extension or WASM plugin as a prerequisite). Version: `"1.0.0"`.

- `armis.sensor.toml` — `sensor_id: "armis"`, `auth_type: "api_key"`,
  base URL from instance_url, tables:
  - `devices` — GET `/api/v1/search` (NO trailing slash) with AQL parameter `aql=in:devices`
    (default AQL derived from `DEFAULT_AQL_TEMPLATE = "in:{table}"` at armis.rs:72) or
    verbatim spec-supplied AQL via the `${query.filter.aql}` interpolation variable.
    Page-based pagination.
  - `alerts` — GET `/api/v1/search` (same single endpoint; discriminated by AQL expression
    `aql=in:alerts` default or spec-supplied AQL) with page-based pagination.
  URL derived from `armis.rs:get_search()` (armis.rs:517):
  `format!("{}/api/v1/search", self.instance_url)` — NO trailing slash. Both `devices` and
  `alerts` tables call the SAME `/api/v1/search` endpoint; the resource type is differentiated
  via the `aql` query parameter, not via separate endpoint paths.
  Timestamp fallback chain: `firstSeen` → `lastSeen` → `DateTime::now()` expressed via a
  WASM transformer plugin (see O-001 Grammar Verification note in §Preconditions —
  `timestamp_fallback_chain` is NOT present in the current TOML grammar and requires a grammar
  extension or WASM plugin as a prerequisite). WARN emission when falling back to `now()`
  preserves the existing `tracing::warn!` audit signal.
  Version: `"1.0.0"`.

All four specs pass BC-2.16.009 validation (no schema errors, no variable reference errors)
and are loaded by BC-2.16.001 at startup when `sensor_specs_dir` includes `crates/prism-sensors/specs/`.

### 2. DTU-Parity Tests Pass (VP-PLUGIN-003)

For each `(sensor_id, table)` pair with non-SKIP status:

- A parity integration test in `crates/prism-spec-engine/tests/` or
  `crates/prism-sensors/tests/parity/` exercises the spec-driven path against the DTU clone:
  1. Start DTU clone server by constructing the clone struct and calling
     `BehavioralClone::start_on(bind, shutdown, tls)` (from `prism_dtu_common::BehavioralClone`
     trait, implemented by `CrowdstrikeClone`, `ClarotyClone`, `CyberintClone`, `ArmisClone`):
     ```rust
     // Signature (all 4 clones — identical via BehavioralClone trait):
     async fn start_on(
         &mut self,
         bind: SocketAddr,                              // typically "127.0.0.1:0" for ephemeral
         shutdown: Option<broadcast::Receiver<()>>,
         #[cfg(feature = "tls")] tls: Option<Arc<axum_server::tls_rustls::RustlsConfig>>,
         #[cfg(not(feature = "tls"))] tls: Option<()>,
     ) -> anyhow::Result<SocketAddr>
     ```
     The returned `SocketAddr` is used to construct the test-override base URL.
  2. Load the bundled TOML spec via `SpecLoader::parse(toml_input: &str)` (spec_parser.rs:655)
     — read the spec file content to a string, then parse via `SpecLoader::parse(&content)`;
     override the spec's `base_url` field to the DTU `SocketAddr` via test-only config injection
  3. Execute `PipelineExecutor::execute()` with a `NullAuthProvider` (DTU does not validate tokens)
     or the DTU's mock auth provider:
     ```rust
     // Actual signature (crates/prism-spec-engine/src/pipeline.rs):
     pub async fn execute(
         spec: &SensorSpec,
         table: &TableSpec,
         context: &FetchContext,
         http_client: &reqwest::Client,
         auth_provider: &dyn AuthProvider,
     ) -> Result<PipelineResult, SpecEngineError>
     ```
  4. Execute the reference path: load the fixture payload from `prism-dtu-{sensor}/fixtures/parity/`
     and apply the prior Rust adapter's `SensorAdapter::fetch()` trait method (implemented as
     `CrowdStrikeAdapter::fetch()`, `ClarotyAdapter::fetch()`, `CyberintAdapter::fetch()`, and
     `ArmisAdapter::fetch()` respectively in `crates/prism-sensors/src/auth/{sensor}.rs`) to
     produce the reference OCSF output
  5. Apply TS-PLUGIN-PARITY-001 Rules A–I canonicalization and compare
  6. Assert parity verdict is PASS or WARN (zero FAILs) for the test case

- Minimum coverage per `(sensor_id, table)` pair: 3 real-sensor fixture cases + 3 synthesized cases
  (happy-path, null-field, unrecognized-enum, empty-result as applicable).

- The `crowdstrike.detections` table parity test specifically exercises the two-step pipeline:
  the DTU stub returns a detection IDs page from QueryV2 and full records from PostEntities;
  the spec-driven output must match the reference OCSF record set byte-by-byte on required
  fields (Rule A) and within timestamp tolerance (Rule C).

### 3. Behavioral Fidelity Preserved

The OCSF output of the spec-driven path is semantically equivalent to the prior hardcoded
adapter path for all test cases:
- Arrow schema column names and types match (string/integer/float/boolean/datetime/json)
- Virtual fields `sensor = "{sensor_id}"` and `source = "{table_name}"` are injected
  (BC-2.16.001 postcondition)
- OCSF field mappings from `ocsf_field` entries reproduce the prior per-adapter normalization
- The parity verdict is PASS or WARN for all non-SKIP test cases; zero FAILs

## Invariants

- **INV-PARITY-001 (Replacement-before-deletion):** PLUGIN-MIGRATION-001-A (deletion of
  hardcoded Rust adapter modules) MUST NOT proceed until VP-PLUGIN-003 is verified GREEN
  for all 4 sensors. This invariant is enforced by the STORY-INDEX dependency graph
  (PLUGIN-MIGRATION-001-A depends_on PLUGIN-MIGRATION-001-D) and by the VP-PLUGIN-003
  gate in the PLUGIN-MIGRATION-001-A story pre-flight check.

- **INV-PARITY-002 (Spec file immutability of sensor_id):** Once a spec file is committed
  as a bundled spec, its `sensor_id` value is immutable. Changing the `sensor_id` in the
  TOML file changes the DataFusion table namespace (`{sensor_id}.{table_name}`) and is
  therefore a breaking change requiring a new BC. (Spec files may be amended for non-ID
  fields without a new BC.)

- **INV-PARITY-003 (Spec file is the source of truth for table schema):** After PLUGIN-MIGRATION-001-D
  merges, the TOML spec files (not the Rust adapter source) are the source of truth for
  the schema of the 4 initial sensor tables. Schema changes require amending the spec file
  and re-validating parity tests.

- **DI-030 (partial-failure isolation):** A parity failure for one `(sensor_id, table)` pair
  does NOT block other sensor tables from loading. Each parity test is isolated.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-016-013-001 | DTU clone not started when parity test runs | Test skipped (SKIP verdict per TS-PLUGIN-PARITY-001 Rule H); marked as DTU debt in parity report |
| EC-016-013-002 | `cyberint.incidents` table parity test | SKIP status per TS-PLUGIN-PARITY-001 Cyberint DTU Gap Note until DTU `incidents` pagination coverage is verified |
| EC-016-013-003 | CrowdStrike two-step batch boundary (exactly 100 IDs, triggering batch split) | Parity test includes a synthesized fixture with 100 IDs to exercise batch capping at CROWDSTRIKE_BATCH_SIZE (100); spec MUST NOT produce a 101-item batch |
| EC-016-013-004 | Claroty polymorphic ID (integer vs UUID string) | Parity test includes one integer-ID fixture and one UUID-string-ID fixture; spec column `type: "string"` normalizes both; OCSF output must match reference for each |
| EC-016-013-005 | Armis timestamp fallback to `now()` | When `firstSeen` and `lastSeen` are absent, spec produces a fetch-time timestamp; reference does too (same fallback path); TS-PLUGIN-PARITY-001 Rule C "both took same fallback path" → PASS by convention |
| EC-016-013-006 | Spec file present but DTU clone not in scope (Wave 1 test run without all DTUs built) | Individual parity tests that require their DTU clone are `#[ignore]` tagged with the message `"requires prism-dtu-{sensor} DTU clone"` until the DTU story (S-6.07–6.10) merges |
| EC-016-013-007 | Null OCSF field in reference output absent from actual (Rule B null vs absent) | Parity WARN (not FAIL); logged in parity report; does not block VP-PLUGIN-003 verification |
| EC-016-013-008 | Spec loaded successfully but no `sensor_specs_dir` configured to include bundled path | The implementation test must set `sensor_specs_dir` to `crates/prism-sensors/specs/` (or equivalent test path) explicitly; mis-configuration in test is a test authoring defect, not a BC violation |

## Error Conditions

| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-SPEC-001` | Bundled spec file fails BC-2.16.009 validation at CI time | CI fails; spec file must be corrected before merge; this is a pre-merge gate |
| `E-SPEC-009` | Duplicate `sensor_id` across two spec files (e.g., two files both declare `sensor_id: "crowdstrike"`) | BC-2.16.001 rejects the second file with `E-SPEC-009` per error-taxonomy.md; first file wins. E-SPEC-009 covers ONLY the duplicate-sensor_id case — it does NOT cover filename-stem-vs-sensor_id mismatch (see E-SPEC-017 below). |
| `E-SPEC-017` | Spec `sensor_id` does not case-sensitively match the filename stem (e.g., `crowdstrike.sensor.toml` with `sensor_id: "falcon"`) | BC-2.16.001 rejects the offending file with `E-SPEC-017` per error-taxonomy.md v1.41. Bundled spec naming convention is `{sensor_id}.sensor.toml`; mismatch indicates a rename without sensor_id update or vice versa; reject at load time to prevent silent namespace drift. (Registered as new code E-SPEC-017 in FB-IMPL-P2-PO 2026-05-20 — prior pass-1 incorrectly cited E-SPEC-009 for this case; E-SPEC-009 has distinct duplicate-sensor_id semantics.) |

**Note on parity FAIL verdict (test verdict, not runtime error):** A parity test FAIL verdict
(where `PipelineExecutor` output does not match the reference OCSF output for a test case) is
a **test verdict**, not a runtime error code. When a parity test fails, the integration test
itself `assert!`s false — no runtime error code is emitted. The fix is to correct the TOML spec's
field mapping or step pipeline until the parity test passes. (The previously cited fabricated code
`E-SPEC-015` has been removed per F-004 fix-burst-1 FB-IMPL-P1-PO 2026-05-20; `E-SPEC-015` was
never registered in error-taxonomy.md and does not exist as a runtime error.)

## Canonical Test Vectors

| Scenario | Sensor | Input | Expected Outcome |
|----------|--------|-------|-----------------|
| Happy-path CrowdStrike detections | crowdstrike | DTU stub: QueryV2 returns 3 detection IDs; PostEntities returns 3 full detection records | Parity PASS: spec-driven OCSF matches reference OCSF for all 3 detections; `request_count == 2` |
| CrowdStrike batch cap | crowdstrike | DTU stub: QueryV2 returns 100 detection IDs in one page | Parity PASS: spec produces one PostEntities batch of 100 (not 101+); `batch_size` cap respected |
| Claroty integer ID | claroty | DTU stub: asset record with `"id": 12345` (integer) | Parity PASS: `id` column value `"12345"` (string-normalized) matches reference |
| Claroty UUID string ID | claroty | DTU stub: asset record with `"id": "550e8400-e29b-41d4-a716-446655440000"` | Parity PASS: `id` column value `"550e8400-..."` matches reference |
| Cyberint alerts happy path | cyberint | DTU stub: 5 alert records with ISO-8601 timestamps | Parity PASS: timestamps normalized to UTC per Rule C; OCSF fields match |
| Armis devices timestamp fallback | armis | DTU stub: device record with no `firstSeen` or `lastSeen` fields | Parity PASS by Rule C convention (both sides take fetch-time timestamp fallback path); WARN logged |
| Armis AQL forwarding | armis | Query with custom AQL expression passed in `${query.filter.aql}` (caller sets `FetchContext::query_filters["aql"]`) | Parity PASS: DTU receives verbatim AQL expression in `aql` parameter; response matches reference |
| Spec load validation — crowdstrike | crowdstrike | `crowdstrike.sensor.toml` content passed through `SpecLoader::parse(toml_input: &str)` (spec_parser.rs:655) | `Ok(SensorSpec)` with `sensor_id == "crowdstrike"`, correct auth_type and table count |
| Empty SKIP — cyberint.incidents | cyberint | Parity test targeting `cyberint.incidents` table | Test returns SKIP with message "cyberint incidents DTU gap — see TS-PLUGIN-PARITY-001 Cyberint DTU Gap Note" |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| VP-148 (VP-PLUGIN-003) | DTU parity: TOML+plugin path output matches deleted Rust adapter path per sensor — per TS-PLUGIN-PARITY-001 canonicalization. This BC is the primary source contract for VP-PLUGIN-003. Parity test must achieve zero FAILs across all non-SKIP `(sensor_id, table)` pairs for VP-PLUGIN-003 to be verified. |

## Related BCs

- BC-2.16.001: Sensor Spec File Loading — the mechanism by which bundled specs are discovered and loaded (composing with)
- BC-2.16.002: Multi-Step Fetch Pipeline — the execution engine for the CrowdStrike two-step spec (depends on)
- BC-2.16.009: Spec File Validation — the validator that each bundled spec must pass at load time (depends on)
- BC-2.16.012: PluginRegistry Dispatch — the dispatch mechanism whose behavioral output this BC asserts parity for (depends on)
- BC-2.01.013: DataSource Trait — the runtime adapter contract that TOML specs satisfy post-migration (composes with)
- BC-2.01.005: CrowdStrike OAuth2 Auth and Two-Step Fetch — the prior Rust implementation whose behavior this BC preserves (supersedes within spec-driven scope)
- BC-2.01.006: Cyberint Cookie-Based Auth — prior implementation preserved by cyberint.sensor.toml (supersedes within spec-driven scope)
- BC-2.01.007: Claroty Bearer Token Auth — prior implementation preserved by claroty.sensor.toml (supersedes within spec-driven scope)
- BC-2.01.008: Armis Bearer Token Auth — prior implementation preserved by armis.sensor.toml (supersedes within spec-driven scope)

## Architecture Anchors

- ADR-023 §Decision Rules — Rule 3 (VP-PLUGIN-003 parity gate — replacement-before-deletion prerequisite)
- ADR-023 §Decision Rules — Rule 1 (four initial sensors ship as pure TOML specs; no in-repo .prx plugin required for the four initial sensors; OCSF complex-transform plugins are a separate concern per Rule 1)
- TS-PLUGIN-PARITY-001 (canonicalization rules for parity comparison: Rules A–I, Rule I fixture minimum, Cyberint DTU Gap Note)
- ADR-023 §Architectural Constraints — C2 (PipelineExecutor as the spec-driven execution engine, replacing the `Ok(Vec::new())` stub; real implementation in PLUGIN-PREREQ-B)

## Story Anchor

PLUGIN-MIGRATION-001-D (implementing story; planned → draft after PO authoring complete)

## VP Anchors

- VP-148 (VP-PLUGIN-003): DTU parity verification property anchored to this BC

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-029 ("Config-Driven Sensor Adapters") per capabilities.md §CAP-029 |
| Capability Anchor Justification | CAP-029 ("Config-Driven Sensor Adapters") per capabilities.md §CAP-029 — this BC describes authoring the 4 production TOML sensor spec files that ARE the config-driven sensor adapter artifacts for the 4 initial sensors, plus their DTU parity verification. CAP-029 defines exactly this: "All sensor tables — including the four initial sensors (CrowdStrike, Cyberint, Claroty, Armis) shipped as bundled TOML spec files — are registered with DataFusion uniformly and queryable via the same `query` MCP tool (CAP-015)." |
| L2 Invariants | DI-008 (client scoping — specs do not cross client boundaries), DI-030 (partial-failure isolation — one spec failure does not block others), DI-012 (auth composition prevention — each spec declares exactly one auth_type) |
| L2 Entities | SensorSpec, TableSpec, ColumnSpec, PipelineResult |
| Priority | P0 |
| ADR anchors | ADR-023 §Decision Rules — Rule 1, §Decision Rules — Rule 3; ADR-023 §Architectural Constraints — C2; TS-PLUGIN-PARITY-001 Rules A–I |
| Subsystem | SS-16 (Spec Engine) |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | FB-IMPL-P3-PO fix-burst-3 | 2026-05-20 | product-owner | Closes pass-3 findings F-LP3-CRIT-001, F-LP3-CRIT-002, F-LP3-CRIT-003, F-LP3-HIGH-001, F-LP3-HIGH-002. F-LP3-CRIT-001: replaced phantom `spec_parser::parse_spec_file()` with `SpecLoader::parse(toml_input: &str)` in §Postconditions §2 step 2 and §Canonical Test Vectors (CODE-GROUNDED: spec_parser.rs:655). F-LP3-CRIT-002: corrected all CrowdStrike URL paths — `/detects/queries/detects/v1` etc. replaced with actual patterns from crowdstrike.rs:262,315: `/queries/{resource_type}` (QueryV2) and `/entities/{resource_type}/GET` (PostEntities); incidents table corrected to two-step (same pattern); URL derivation via `resource_type_from_spec()` (crowdstrike.rs:369-375) documented. F-LP3-CRIT-003: stripped `/xdome` prefix from all Claroty endpoints — actual pattern is `/api/v1/{resource}s` (claroty.rs:244); `/xdome` was never present in the code. F-LP3-HIGH-001: removed `/v1` segment from Cyberint endpoints — actual pattern is `/api/{resource}s` (cyberint.rs:251); no `/v1` in Cyberint URL construction. F-LP3-HIGH-002: corrected Armis endpoint — single `/api/v1/search` (no trailing slash, armis.rs:517) used for ALL queries including both `devices` and `alerts`; AQL discriminator `in:devices` / `in:alerts` via `DEFAULT_AQL_TEMPLATE` (armis.rs:72) documented; phantom per-resource endpoint paths removed. |
| 1.2 | FB-IMPL-P2-PO fix-burst-2 | 2026-05-20 | product-owner | Closes pass-2 findings F-001, F-002, F-003, F-004, F-005. F-001: swapped auth_type strings in §Postconditions §1 — claroty=`cookie_roundtrip` (was `bearer_static`), cyberint=`bearer_static` (was `cookie_roundtrip`), matching `ClarotyAuth::auth_type_name()` = `"cookie_roundtrip"` and `CyberintAuth::auth_type_name()` = `"bearer_static"` per code-grounded verification of `crates/prism-sensors/src/auth/{claroty,cyberint}.rs`. F-002: corrected §Error Conditions — E-SPEC-009 row now accurately describes ONLY duplicate-sensor_id (not filename-stem mismatch); added E-SPEC-017 row for filename-stem-vs-sensor_id mismatch (newly registered in error-taxonomy.md v1.41 per POL-1 append-only). F-003: replaced phantom `CrowdStrikeAdapter::fetch_page()` (non-existent) with actual `SensorAdapter::fetch()` trait method in §Postconditions §1 and §2 (CODE-GROUNDED: `crowdstrike.rs` has `fetch()` at trait impl, no `fetch_page()`; all 4 sensors use the same `SensorAdapter::fetch()` entry point). F-004: corrected `${query.aql}` → `${query.filter.aql}` in §Canonical Test Vectors Armis AQL forwarding row. F-005 (TD-VSDD-091): replaced `spec_parser.rs:128` → `FetchStep::fan_out_batch_size field` and `pipeline.rs:246-250` → `PipelineExecutor::execute_impl query.filter.{k} step-vars seeding` in §Preconditions O-001 table. |
| 1.1 | FB-IMPL-P1-PO fix-burst-1 | 2026-05-20 | product-owner | Closes pass-1 adversarial findings F-001/F-002/F-004/F-006/F-007/O-001. F-001: replaced fabricated `prism_dtu_{sensor}::server::spawn()` / `DtuHandle` API with actual `BehavioralClone::start_on(bind, shutdown, tls) -> anyhow::Result<SocketAddr>` trait (all 4 clones share via `prism_dtu_common::BehavioralClone`). F-002: replaced fabricated `PipelineExecutor::execute(spec, "<table_name>", &NullAuthProvider, ...)` with actual 5-arg signature `(spec: &SensorSpec, table: &TableSpec, context: &FetchContext, http_client: &reqwest::Client, auth_provider: &dyn AuthProvider) -> Result<PipelineResult, SpecEngineError>`. F-004: retired fabricated `E-SPEC-015` (parity FAIL is a test verdict, not a runtime error code) and replaced fabricated `E-SPEC-016` with `E-SPEC-009` (existing code already covers sensor_id/filename mismatch). F-006: corrected `ADR-023 §Rule 1` / `§Rule 3` phantom anchors to `ADR-023 §Decision Rules — Rule 1` / `§Decision Rules — Rule 3`. F-007: corrected `ADR-022 §C2` phantom anchor (C2 is in ADR-023, not ADR-022) to `ADR-023 §Architectural Constraints — C2`. O-001: added grammar verification table in §Preconditions confirming `fan_out_batch_size` SUPPORTED, `${query.filter.aql}` SUPPORTED (not `${query.aql}`), `timestamp_format = "multi"` NOT SUPPORTED, `timestamp_fallback_chain` NOT SUPPORTED — grammar extension or WASM plugin required as implementer prerequisite. Postconditions updated to reflect grammar gaps. |
| 1.0 | D-731 PLUGIN-MIGRATION-001-D PO authoring | 2026-05-20 | product-owner | Initial draft — BC anchor for PLUGIN-MIGRATION-001-D; DTU-parity contract for VP-PLUGIN-003; authored from ADR-023 §Rule 3 + TS-PLUGIN-PARITY-001 + 4 sensor adapter source surveys |
