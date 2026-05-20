---
document_type: story
story_id: PLUGIN-MIGRATION-001-D
title: "Author 4 Production TOML Sensor Specs — Reverse-Engineered + DTU-Parity Tests"
wave: 1
epic_id: PLUGIN-MIGRATION-001
priority: P0
status: draft
version: "v1.1"
level: "L4"
producer: story-writer
timestamp: "2026-05-20T00:00:00Z"
modified: "2026-05-20"
input-hash: null
traces_to: []
cycle: "v1.0.0-greenfield"
phase: 3
tdd_mode: strict
subsystems: [SS-01, SS-16]
# Subsystem anchor justifications:
#   SS-16 (Spec Engine, prism-spec-engine) owns BC-2.16.001/002/009/012/013 — all spec-loading,
#   pipeline-execution, spec-validation, and spec-catalog dispatch contracts. The 4 bundled TOML
#   files are the primary artifacts; they are loaded, validated, and dispatched via SS-16 machinery.
#   SS-01 (Sensor Adapters, prism-sensors) owns BC-2.01.013 (DataSource trait) and
#   BC-2.01.016 (SensorAuth open trait). The 4 bundled TOML specs declare auth_type values that
#   resolve through the SS-01 SensorAuth hierarchy and feed the DataSource dispatch chain.
crates_touched: [prism-sensors, prism-spec-engine]
target_module: prism-sensors
capabilities: [CAP-029]
behavioral_contracts:
  - BC-2.01.013  # DataSource Trait — spec-driven adapter pattern; confirms adapter implementations
                 #   are produced from TOML SensorSpec declarations, not handwritten Rust
  - BC-2.01.016  # SensorAuth Open Trait — auth_type values in the 4 specs resolve through this
                 #   open trait hierarchy; each spec's auth_type must match a canonical SensorAuth
                 #   implementation (INV-AUTH-OPEN-003 runtime enforcement)
  - BC-2.16.001  # Sensor Spec File Loading — the 4 TOML files must be discoverable, parseable,
                 #   and their tables registered in DataFusion at startup (postcondition 1–9)
  - BC-2.16.002  # Multi-Step Fetch Pipeline — CrowdStrike 2-step QueryV2→PostEntities pipeline
                 #   is the canonical test of multi-step TOML spec execution; parity tests run
                 #   PipelineExecutor against DTU clone per this BC's postconditions
  - BC-2.16.009  # Spec File Validation — all 4 bundled specs must pass schema validation,
                 #   variable reference resolution, OCSF field validation, and pagination config
                 #   validation at CI time; any E-SPEC-001 failure is a pre-merge gate (§Error Conditions)
  - BC-2.16.012  # PluginRegistry Dispatch — the spec_parser.rs open dispatch path (INV-SPEC-PARSER-OPEN-001)
                 #   must handle the 4 new specs without hardcoded sensor name match arms; parity
                 #   AC asserts no dispatch regression vs the pre-TOML path
  - BC-2.16.013  # Bundled Sensor Spec Authoring and DTU-Parity Verification (v1.1 FB-IMPL-P1-PO) —
                 #   primary contract: 4 TOML files authored at crates/prism-sensors/specs/,
                 #   validated, DTU-parity tests authored per TS-PLUGIN-PARITY-001 Rules A–I,
                 #   INV-PARITY-001 replacement-before-deletion enforced
verification_properties:
  - VP-148  # VP-PLUGIN-003: DTU parity — TOML+plugin path output matches deleted Rust adapter
            #   path per sensor, per TS-PLUGIN-PARITY-001 canonicalization. This story authors
            #   the integration tests that exercise VP-148. The tests live under
            #   crates/prism-spec-engine/tests/parity/.
depends_on:
  - S-PLUGIN-PREREQ-A  # SensorId newtype: spec files use sensor_id strings resolved to SensorId(Arc<str>)
  - S-PLUGIN-PREREQ-B  # PipelineExecutor + AuthProvider: the DTU parity tests drive PipelineExecutor
  - S-PLUGIN-PREREQ-C  # TOML grammar full implementation: spec_parser.rs parses the 4 specs
  - S-PLUGIN-PREREQ-D  # PluginRuntime boot wiring: open dispatch path in spec_parser.rs wired
  # S-PLUGIN-PREREQ-E is also merged (all 5 PREREQ stories done): SensorAuth is open, spec_parser
  # has no CustomAdapter references, WriteToolInvalidationMap is runtime-extensible. No explicit
  # depends_on needed because E is a sibling Wave 0 story fully merged before Wave 1 dispatch;
  # the INV-AUTH-OPEN-003 enforcement that E delivers is already live in develop@80ebe794.
blocks:
  - PLUGIN-MIGRATION-001-A  # INV-PARITY-001: 001-A MUST NOT proceed until VP-PLUGIN-003 is green
  - PLUGIN-MIGRATION-001-B  # prism-query dispatch stories presuppose stable sensor_id strings from specs
  - PLUGIN-MIGRATION-001-C  # SpecDrivenMapper uses the 4 bundled spec schemas
  - PLUGIN-MIGRATION-001-E  # CrowdStrike OAuth2 .prx plugin presupposes crowdstrike.sensor.toml exists
points: 5
# Points justification: Row 399 in STORY-INDEX carried 3 as a placeholder. Re-evaluated against
# actual AC enumeration (13 ACs: 4 spec files × validation + auth_type + DTU parity; 4 sensors
# each with distinct parity test harness setup; fixture path authoring; Cargo.toml dev-dep wiring;
# spec validation CI gate; workspace grep AC). 5 points (≈2–2.5 days) is appropriate:
#   - 4 TOML spec files with precise field-level fidelity to Rust source (~1 day authoring)
#   - 8 DTU parity integration tests with fixture scaffolding (~0.5 day)
#   - Red Gate test suite (9 tests) + spec validation gate (~0.5 day)
# This is below the 13-point cap. If DTU fixture scaffolding is blocked, parity tests are
# SKIP-tagged per EC-016-013-001; spec authoring alone is ~3 points but parity test harness
# scaffolding (even with #[ignore] bodies) adds the remainder.
estimated_days: 2
risk: MEDIUM
acceptance_criteria_count: 13
red_gate_tests: 9
estimated_passes: "3-5 LOCAL adversary passes"
holdout_scenarios:
  - HS-013  # CrowdStrike two-step parity — TOML+PipelineExecutor vs Rust adapter output (DTU)
  - HS-014  # Claroty POST-for-read parity — polymorphic ID normalisation (DTU)
  - HS-015  # Cyberint alerts cursor pagination parity (DTU; incidents in SKIP)
  - HS-016  # Armis AQL forwarding + timestamp fallback parity (DTU)
  - HS-017  # Negative: bundled spec fails BC-2.16.009 validation at CI (must block merge)
  - HS-018  # Negative: spec sensor_id/filename mismatch rejected at load time (E-SPEC-009)
assumption_validations: []
risk_mitigations:
  - "INV-PARITY-001 enforced structurally: PLUGIN-MIGRATION-001-A depends_on PLUGIN-MIGRATION-001-D in
    STORY-INDEX; AC-12 asserts VP-PLUGIN-003 must be verified (or all SKIP justified) before 001-A
    can be dispatched. The story pre-flight check for 001-A references this AC."
  - "DTU clone unavailability: parity tests tagged #[ignore] per EC-016-013-006 if DTU clone story
    (S-6.07–6.10) has not merged; CI still runs spec validation ACs (AC-2/5/6) unconditionally."
  - "TOML field-level drift from Rust adapter: each spec's column schema, auth_type, endpoint,
    and pagination config is reverse-engineered directly from crates/prism-sensors/src/auth/
    source code read in this story — not from memory or BC summaries. Test vector fixtures from
    crates/prism-dtu-*/fixtures/ validate the parity output."
inputs:
  - "crates/prism-sensors/src/auth/crowdstrike.rs"
  - "crates/prism-sensors/src/auth/cyberint.rs"
  - "crates/prism-sensors/src/auth/claroty.rs"
  - "crates/prism-sensors/src/auth/armis.rs"
  - ".factory/specs/behavioral-contracts/BC-2.16.013-bundled-sensor-spec-dtu-parity.md"
  - ".factory/specs/behavioral-contracts/BC-2.16.001-sensor-spec-file-loading.md"
  - ".factory/specs/behavioral-contracts/BC-2.16.002-multi-step-fetch-pipeline.md"
  - ".factory/specs/behavioral-contracts/BC-2.16.009-spec-file-validation.md"
  - ".factory/specs/behavioral-contracts/BC-2.16.012-plugin-registry-dispatch-migration.md"
  - ".factory/specs/behavioral-contracts/BC-2.01.013-datasource-trait-adapter-pattern.md"
  - ".factory/specs/behavioral-contracts/BC-2.01.016-sensor-auth-open-trait-contract.md"
  - ".factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md"
  - ".factory/specs/verification-properties/VP-INDEX.md"
  - "crates/prism-dtu-crowdstrike/src/clone.rs"
  - "crates/prism-dtu-claroty/src/"
  - "crates/prism-dtu-cyberint/src/"
  - "crates/prism-dtu-armis/src/"
---

# PLUGIN-MIGRATION-001-D: Author 4 Production TOML Sensor Specs — Reverse-Engineered + DTU-Parity Tests

**Story ID:** PLUGIN-MIGRATION-001-D  
**Status:** draft  
**Wave:** 1 (first unblocked Wave 1 story; all 5 PREREQ stories merged to develop@80ebe794)

---

## Story-Level Goal

At merge, four production TOML sensor spec files exist at `crates/prism-sensors/specs/` and are
loadable, validatable, and queryable through the existing spec-driven infrastructure (BC-2.16.001,
BC-2.16.009, BC-2.16.002). Each spec is paired with a DTU-parity integration test that — when all
four DTU clone stories have merged — proves the spec-driven path produces OCSF output that is
semantically equivalent to the prior hardcoded Rust adapter path against the same DTU clone.
PLUGIN-MIGRATION-001-A (deletion of the 4 Rust adapter modules) is gated on VP-PLUGIN-003 being
verified green for all 4 sensors (BC-2.16.013 INV-PARITY-001).

---

## Narrative

As the Prism platform, I want four production TOML sensor spec files (CrowdStrike, Cyberint,
Claroty, Armis) authored from the existing hardcoded Rust adapters and validated by the spec
pipeline and DTU-parity tests, so that the Wave 1 plugin-migration stories (001-A deletion,
001-B prism-query dispatch, 001-C OCSF mapper, 001-E OAuth2 plugin) can proceed with provable
behavioral continuity and without risk of silent OCSF regression.

---

## Functional Summary

1. **Four bundled TOML sensor spec files** authored at `crates/prism-sensors/specs/`, each
   reverse-engineered from the corresponding Rust adapter in `crates/prism-sensors/src/auth/`:
   - `crowdstrike.sensor.toml` — OAuth2 client credentials; two-step QueryV2→PostEntities pipeline
   - `claroty.sensor.toml` — bearer_static; POST-for-read offset pagination; polymorphic ID handling
   - `cyberint.sensor.toml` — cookie_roundtrip; cursor-based alerts; incidents SKIP-noted
   - `armis.sensor.toml` — api_key; AQL query forwarding; timestamp fallback chain

2. **Spec validation CI gate**: all 4 specs pass BC-2.16.009 validation (no E-SPEC-001 errors)
   and BC-2.16.001 loading at test time. A dedicated integration test exercises all 4 at once.

3. **DTU-parity integration tests** under `crates/prism-spec-engine/tests/parity/`: one test
   per sensor exercising `CrowdstrikeClone::new()` / equivalent DTU clone server, loading the
   bundled TOML spec, running `PipelineExecutor::execute()` with a `NullAuthProvider`, and
   asserting parity against reference OCSF output per TS-PLUGIN-PARITY-001 Rules A–I. Tests that
   require DTU clones not yet merged are tagged `#[ignore]` with the standard message.

4. **No new hardcoded sensor names** introduced in any Rust source. The 4 TOML files are the
   sole new artifacts; no Rust match arms, no sensor-specific code outside the TOML spec files.

5. **Workspace test count preserved**: `just check` passes with all existing tests green (3,681
   or higher if test count grows during implementation).

---

## Behavioral Contracts

| BC ID | Version | Title | Subsystem | Role in This Story |
|-------|---------|-------|-----------|-------------------|
| BC-2.16.013 | 1.1 | Bundled Sensor Spec Authoring and DTU-Parity Verification — 4 Initial Sensors | SS-16 | **Primary delivery** — defines the 4 spec files, their content, the parity test structure, and INV-PARITY-001 replacement-before-deletion gate |
| BC-2.16.001 | 1.3 | Sensor Spec File Loading — Parse TOML, Validate Schema, Register Tables | SS-16 | **Required** — specifies that `*.sensor.toml` files in `sensor_specs_dir` (here: `crates/prism-sensors/specs/`) are discovered, parsed, and registered at startup; virtual fields injected |
| BC-2.16.009 | 1.3 | Spec File Validation — Schema Validation, Variable Reference Resolution, OCSF Field Validation | SS-16 | **Required** — all 4 bundled specs must pass all 5 validation rule categories; CI gate via dedicated integration test |
| BC-2.16.002 | 1.35 | Multi-Step Fetch Pipeline Execution — Sequential Steps with Variable Interpolation | SS-16 | **Required** — CrowdStrike QueryV2→PostEntities two-step pipeline exercised in parity test; PipelineExecutor drives the parity test harness |
| BC-2.16.012 | 1.29 | PluginRegistry Dispatch in spec_parser.rs — Hardcoded Sensor Names Replaced with Registry Lookup | SS-16 | **Awareness + anti-regression** — the 4 new specs must flow through the open dispatch path (INV-SPEC-PARSER-OPEN-001); no new hardcoded sensor match arms introduced |
| BC-2.01.013 | 1.6 | DataSource Trait Eliminates Per-Sensor Code Duplication | SS-01 | **Awareness** — the 4 TOML specs are the adapter implementations produced at runtime per the postconditions of this BC; no hand-written adapter code for TOML-expressible sensors |
| BC-2.01.016 | 1.10 | SensorAuth Open Trait — Plugin-Implementable Auth Contract (No Sealed Marker) | SS-01 | **Required** — each spec's `auth_type` value resolves through the open SensorAuth hierarchy; runtime INV-AUTH-OPEN-003 Rule 2 enforcement applies at spec-load for each spec |

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec | ~8,000 |
| BC files (7 BCs, read in full) | ~28,000 |
| Rust adapter sources (4 files, read in full) | ~12,000 |
| DTU clone src (4 crates, partial reads) | ~6,000 |
| prism-spec-engine/tests/ existing test files (pattern reference) | ~4,000 |
| crates/prism-sensors/Cargo.toml + prism-spec-engine/Cargo.toml | ~1,000 |
| ADR-023 (key sections) | ~3,000 |
| **Total estimate** | **~62,000** |
| Agent context window (claude-sonnet-4-6) | ~200,000 |
| **% of context window** | **~31%** |

Note: This story is at the upper boundary of the 20–30% target. The bulk (adapter sources + BC
bodies) must be read in full to author correct specs. If the implementer finds context pressure,
splitting into sub-tasks (TOML authoring vs parity test authoring) is acceptable within the same
story; the story MUST NOT be split into two separate story IDs.

---

## Acceptance Criteria

Each AC traces to the specific BC clause it satisfies. ACs are numbered; the implementer ticks
them off as each Red Gate test passes.

### AC-001: `crowdstrike.sensor.toml` Exists and Parses (traces to BC-2.16.013 postcondition 1)

`crates/prism-sensors/specs/crowdstrike.sensor.toml` exists with:
- `sensor_id = "crowdstrike"` (matches filename per E-SPEC-009 / BC-2.16.013 §Error Conditions)
- `auth_type = "oauth2_client_credentials"`
- `base_url` placeholder using `${env.CROWDSTRIKE_BASE_URL}` or parameterized via
  `cloud_region` template variable (implementer must verify the spec grammar supports this;
  if not, use a canonical placeholder that DTU test can override via config injection)
- Tables: `detections` (2-step pipeline), `devices` (2-step pipeline), `incidents` (cursor)
- `version = "1.0.0"`
- `[rate_limit_hints]` `requests_per_second = 10.0`

`spec_parser::parse_spec_file("crates/prism-sensors/specs/crowdstrike.sensor.toml")` returns
`Ok(SensorSpec)` with `sensor_id == "crowdstrike"`, `auth_type == Oauth2ClientCredentials`,
`tables.len() == 3`.

(traces to BC-2.16.013 postcondition 1 — `crowdstrike.sensor.toml` authored and parsed correctly)

### AC-002: `claroty.sensor.toml` Exists and Parses (traces to BC-2.16.013 postcondition 1)

`crates/prism-sensors/specs/claroty.sensor.toml` exists with:
- `sensor_id = "claroty"`, `auth_type = "bearer_static"`
- Tables: `assets` (POST-for-read + offset), `alerts` (POST-for-read + offset), `audit_logs`
  (Claroty hybrid pagination via `paginate_claroty` semantics)
- Polymorphic ID column: `id` with `type = "string"` and `ocsf_field` mapping
- `version = "1.0.0"`

`parse_spec_file("crates/prism-sensors/specs/claroty.sensor.toml")` returns `Ok(SensorSpec)`
with `sensor_id == "claroty"`, `auth_type == BearerStatic`, `tables.len() == 3`.

(traces to BC-2.16.013 postcondition 1 — `claroty.sensor.toml` authored and parsed correctly)

### AC-003: `cyberint.sensor.toml` Exists and Parses (traces to BC-2.16.013 postcondition 1)

`crates/prism-sensors/specs/cyberint.sensor.toml` exists with:
- `sensor_id = "cyberint"`, `auth_type = "cookie_roundtrip"`
- Tables: `alerts` (cursor pagination), `incidents` (noted as SKIP in parity test per
  EC-016-013-002 — spec file still exists and must parse; parity test is SKIP not absent)
- Multi-format timestamp: `alerts.timestamp` column declares `type = "datetime"` with
  `timestamp_format = "multi"` extension field
- `version = "1.0.0"`

`parse_spec_file("crates/prism-sensors/specs/cyberint.sensor.toml")` returns `Ok(SensorSpec)`
with `sensor_id == "cyberint"`, `auth_type == CookieRoundtrip`, `tables.len() == 2`.

(traces to BC-2.16.013 postcondition 1 — `cyberint.sensor.toml` authored and parsed correctly)

### AC-004: `armis.sensor.toml` Exists and Parses (traces to BC-2.16.013 postcondition 1)

`crates/prism-sensors/specs/armis.sensor.toml` exists with:
- `sensor_id = "armis"`, `auth_type = "api_key"`
- Tables: `devices` (AQL forwarding via `${query.filter.aql}` template variable, seeded from
  `FetchContext::query_filters["aql"]`; page pagination), `alerts` (AQL forwarding; page pagination)
- Timestamp fallback chain expressed via `timestamp_fallback_chain = ["firstSeen", "lastSeen"]`
  extension field with WARN emission semantic preserved
- `version = "1.0.0"`

`parse_spec_file("crates/prism-sensors/specs/armis.sensor.toml")` returns `Ok(SensorSpec)`
with `sensor_id == "armis"`, `auth_type == ApiKey`, `tables.len() == 2`.

(traces to BC-2.16.013 postcondition 1 — `armis.sensor.toml` authored and parsed correctly)

### AC-005: All 4 Specs Pass BC-2.16.009 Validation (traces to BC-2.16.009 postconditions + BC-2.16.013 postcondition 1)

The integration test `test_BC_2_16_009_validates_all_4_bundled_specs` loads all 4 spec files
through the full BC-2.16.009 validation pipeline and asserts each returns `Ok(SensorSpec)` with
zero validation errors. Specifically:
- Schema validation passes (regex, enum, non-empty, semver per §Validation Rules 1)
- Variable reference resolution passes for CrowdStrike two-step step references (§Validation Rules 2)
- OCSF field validation: any unrecognized `ocsf_field` values produce WARNs, not errors (§3)
- Pagination config consistency passes for all declared pagination types (§4)
- Rate limit hints valid (§5)

The test runs unconditionally (no `#[ignore]`) because DTU clones are not required for
spec validation.

(traces to BC-2.16.009 postconditions — single-pass, all-errors-collected validation)

### AC-006: Specs Load via BC-2.16.001 Boot Mechanism (traces to BC-2.16.001 postconditions)

The integration test `test_BC_2_16_001_loads_4_bundled_specs_at_boot` invokes the spec loader
pointing `sensor_specs_dir` to `crates/prism-sensors/specs/` and asserts:
- All 4 specs are loaded (no E-SPEC-001 or E-SPEC-009 errors)
- Each spec's tables are registered in the DataFusion catalog under `{sensor_id}.{table_name}`
  (e.g., `crowdstrike.detections`, `claroty.assets`, `cyberint.alerts`, `armis.devices`)
- Virtual fields `sensor` and `source` are present in each registered schema
- An empty-credential scenario (no client credentials configured) results in tables marked
  unavailable per DEC-036 — not an error

(traces to BC-2.16.001 §Postconditions + §Table Registration with DataFusion + §Auth Type Resolution)

### AC-007: CrowdStrike DTU Parity Test — Two-Step Pipeline (traces to BC-2.16.013 postcondition 2 + BC-2.16.002)

`test_BC_2_16_013_dtu_parity_crowdstrike` in `crates/prism-spec-engine/tests/parity/crowdstrike.rs`:

1. Instantiate `CrowdstrikeClone` (via `CrowdstrikeClone::new()` or equivalent constructor per the
   crate's public API) and start the DTU server using the `BehavioralClone` trait method:
   ```rust
   let bound_addr = clone.start_on(
       "127.0.0.1:0".parse().unwrap(),
       None,    // no graceful-shutdown receiver needed in test
       None,    // tls: None — plain HTTP in test
   ).await.expect("CrowdStrike DTU clone failed to start");
   let dtu_base_url = format!("http://{}", bound_addr);
   ```
2. Load `crowdstrike.sensor.toml` via `spec_parser::parse_spec_file()`; override `base_url` in the
   loaded `SensorSpec` with `dtu_base_url` via test-only config injection (or read the spec with a
   test config that sets `base_url` to the DTU address).
3. Resolve the `detections` table spec: `spec.tables.iter().find(|t| t.table_name == "detections").unwrap()`.
4. Construct `FetchContext` with a test org slug and empty query filters:
   ```rust
   let ctx = FetchContext::new(
       OrgSlug::new_unchecked("test-org"),  // test-helpers feature; NOT in production code
       std::collections::HashMap::new(),
   );
   ```
5. Build a `reqwest::Client` with the required 30-second timeout per CLAUDE.md §Conventions:
   ```rust
   let http_client = reqwest::Client::builder()
       .timeout(std::time::Duration::from_secs(30))
       .build()
       .unwrap();
   ```
6. Execute the pipeline:
   ```rust
   let result = PipelineExecutor::execute(&spec, &table, &ctx, &http_client, &NullAuthProvider)
       .await
       .expect("PipelineExecutor::execute failed");
   ```
7. Load the reference fixture payload from `crates/prism-dtu-crowdstrike/fixtures/parity/` and
   apply the prior Rust adapter's normalization logic to produce reference OCSF output.
8. Apply TS-PLUGIN-PARITY-001 Rules A–I canonicalization and assert `parity_verdict != FAIL`.

The test is tagged `#[ignore = "requires prism-dtu-crowdstrike DTU clone"]` until S-6.07 merges.
When enabled, the test specifically exercises the batch boundary (EC-016-013-003): a fixture case
with exactly 100 detection IDs asserts the spec produces one PostEntities batch of ≤ 100 records.

(traces to BC-2.16.013 postcondition 2, canonical test vector "Happy-path CrowdStrike detections")

### AC-008: Claroty DTU Parity Test — POST-for-Read + Polymorphic ID (traces to BC-2.16.013 postcondition 2)

`test_BC_2_16_013_dtu_parity_claroty` in `crates/prism-spec-engine/tests/parity/claroty.rs`:

1. Instantiate `ClarotyClone` and start the DTU server via `BehavioralClone::start_on`:
   ```rust
   let bound_addr = clone.start_on("127.0.0.1:0".parse().unwrap(), None, None)
       .await.expect("Claroty DTU clone failed to start");
   let dtu_base_url = format!("http://{}", bound_addr);
   ```
2. Load `claroty.sensor.toml` via `spec_parser::parse_spec_file()` with `base_url` overridden to `dtu_base_url`.
3. Resolve the `assets` table: `spec.tables.iter().find(|t| t.table_name == "assets").unwrap()`.
4. Construct `FetchContext` and `reqwest::Client` with 30-second timeout (same pattern as AC-007 steps 4–5).
5. Execute `PipelineExecutor::execute(&spec, &table, &ctx, &http_client, &NullAuthProvider).await`.
6. Include two fixture cases per EC-016-013-004:
   - Integer ID fixture: asset with `"id": 12345`; assert `id` column value is `"12345"` (string-normalized)
   - UUID string ID fixture: asset with `"id": "550e8400-..."` ; assert `id` column value matches reference
7. Assert parity verdict PASS or WARN (zero FAILs) for all non-SKIP cases.

Tagged `#[ignore = "requires prism-dtu-claroty DTU clone"]` until S-6.08 merges.

(traces to BC-2.16.013 postcondition 2, EC-016-013-004)

### AC-009: Cyberint DTU Parity Test — Alerts Cursor; Incidents SKIP (traces to BC-2.16.013 postcondition 2 + EC-016-013-002)

`test_BC_2_16_013_dtu_parity_cyberint` in `crates/prism-spec-engine/tests/parity/cyberint.rs`:

1. Instantiate `CyberintClone` and start the DTU server via `BehavioralClone::start_on`:
   ```rust
   let bound_addr = clone.start_on("127.0.0.1:0".parse().unwrap(), None, None)
       .await.expect("Cyberint DTU clone failed to start");
   let dtu_base_url = format!("http://{}", bound_addr);
   ```
2. Load `cyberint.sensor.toml` via `spec_parser::parse_spec_file()` with `base_url` overridden to `dtu_base_url`.
3. Resolve the `alerts` table: `spec.tables.iter().find(|t| t.table_name == "alerts").unwrap()`.
4. Construct `FetchContext` and `reqwest::Client` with 30-second timeout (same pattern as AC-007 steps 4–5).
5. Execute `PipelineExecutor::execute(&spec, &table, &ctx, &http_client, &NullAuthProvider).await`.
6. Fixture cases: ISO-8601 timestamps normalized to UTC (Rule C); multi-format timestamp handling.
7. Assert parity verdict PASS or WARN for `alerts` table.

The `incidents` table parity test is a separate test function that immediately returns SKIP with
the standard message: `"cyberint incidents DTU gap — see TS-PLUGIN-PARITY-001 Cyberint DTU Gap Note"`.
It is NOT tagged `#[ignore]` — it is an explicit SKIP-assertion test that runs and passes CI.

Tagged `#[ignore = "requires prism-dtu-cyberint DTU clone"]` (alerts test only) until S-6.09 merges.

(traces to BC-2.16.013 postcondition 2, EC-016-013-002)

### AC-010: Armis DTU Parity Test — AQL Forwarding + Timestamp Fallback (traces to BC-2.16.013 postcondition 2 + EC-016-013-005)

`test_BC_2_16_013_dtu_parity_armis` in `crates/prism-spec-engine/tests/parity/armis.rs`:

1. Instantiate `ArmisClone` and start the DTU server via `BehavioralClone::start_on`:
   ```rust
   let bound_addr = clone.start_on("127.0.0.1:0".parse().unwrap(), None, None)
       .await.expect("Armis DTU clone failed to start");
   let dtu_base_url = format!("http://{}", bound_addr);
   ```
2. Load `armis.sensor.toml` via `spec_parser::parse_spec_file()` with `base_url` overridden to `dtu_base_url`.
3. Resolve the `devices` table: `spec.tables.iter().find(|t| t.table_name == "devices").unwrap()`.
4. AQL forwarding sub-case: Construct a `FetchContext` with `query_filters` seeded with the AQL
   expression under the `"aql"` key (which the pipeline interpolates as `${query.filter.aql}`):
   ```rust
   let mut filters = std::collections::HashMap::new();
   filters.insert("aql".to_string(), "in:devices timeFrame:\"Last 3 Hours\"".to_string());
   let ctx = FetchContext::new(OrgSlug::new_unchecked("test-org"), filters);
   ```
5. Build a `reqwest::Client` with the required 30-second timeout (same pattern as AC-007 step 5).
6. Execute `PipelineExecutor::execute(&spec, &table, &ctx, &http_client, &NullAuthProvider).await`;
   assert the DTU receives the verbatim AQL expression in the HTTP request's `aql` query parameter.
7. Timestamp fallback sub-case: Re-execute with a fixture where both `firstSeen` and `lastSeen`
   are absent; assert parity PASS by convention (both sides take fetch-time timestamp fallback)
   and that a WARN tracing event is emitted per Rust adapter precedent.

Tagged `#[ignore = "requires prism-dtu-armis DTU clone"]` until S-6.10 merges.

(traces to BC-2.16.013 postcondition 2, canonical test vectors "Armis AQL forwarding" + "Armis devices timestamp fallback")

### AC-011: auth_type Values Correctly Declare SensorAuth Compatibility (traces to BC-2.01.016 postconditions + BC-2.01.013 postcondition 4)

For each of the 4 bundled specs, the `auth_type` field value matches exactly one canonical
SensorAuth auth-type string declared by the corresponding `auth_type_name()` implementation:
- `crowdstrike`: `"oauth2_client_credentials"` (matches `CrowdStrikeAuth::auth_type_name()`)
- `claroty`: `"bearer_static"` (matches `ClarotyAuth::auth_type_name()`)
- `cyberint`: `"cookie_roundtrip"` (matches `CyberintAuth::auth_type_name()`)
- `armis`: `"api_key"` (matches `ArmisAuth::auth_type_name()`)

The `test_BC_2_16_009_validates_all_4_bundled_specs` test (AC-005) implicitly verifies this
because BC-2.16.009 §Validation Rules 1 rejects `auth_type` values not in the canonical
enumerated set {`oauth2_client_credentials`, `bearer_static`, `cookie_roundtrip`, `api_key`}.

Additionally, the spec-parser's ADR-023 Rule 2 runtime enforcement (BC-2.01.016 INV-AUTH-OPEN-003)
fires at spec-load time for any spec declaring a non-canonical auth_type — the 4 bundled specs
must not trigger E-SPEC-012.

(traces to BC-2.01.016 postconditions — spec declares a single canonical auth_type; to BC-2.01.013
postcondition 4 — auth-composition prevention enforced at spec-load)

### AC-012: PluginRegistry Open Dispatch — No New Hardcoded Sensor Names (traces to BC-2.16.012 INV-SPEC-PARSER-OPEN-001)

After this story merges:
```
grep -rn '"crowdstrike"\|"cyberint"\|"claroty"\|"armis"' crates/prism-spec-engine/src/spec_parser.rs
```
returns ZERO matches in dispatch-context code. The 4 new TOML files flow through the open
dispatch path established in S-PLUGIN-PREREQ-E (INV-SPEC-PARSER-OPEN-001). No new hardcoded
match arms introduced.

The test `test_BC_2_16_012_plugin_dispatch_uses_spec_catalog_not_hardcoded_names` runs the
spec_parser against `crowdstrike.sensor.toml` before and after the DTU parity test to confirm
the same `SensorSpec` is produced on both passes — behavioral equivalence is preserved.

(traces to BC-2.16.012 INV-SPEC-PARSER-OPEN-001 — dispatch is open, not closed)

### AC-013: Workspace Gate Passes — `just check` Green (traces to BC-2.16.013 postcondition 2 / production-grade default)

`just check` completes with all tests passing (minimum 3,681 / 3,681 tests; actual count may be
higher after parity test additions). No `unwrap()` or `expect()` in non-test parity test harness
code (only in the Red Gate test bodies themselves, under `assert_*` macros). No `println!` in
any parity test driver code (use `tracing::debug!` or `eprintln!` for test diagnostics only).
No `TODO`, `FIXME`, `MVP`, or "for now" comments in any shipped TOML spec file or parity test
harness file. No placeholder values in shipped TOML (`base_url` must use a valid URL template
or parameterized pattern, not `https://example.com`).

(traces to BC-2.16.013 postcondition 1 — "production-grade defaults: no TODOs, no placeholder values")

---

## Red Gate Test Set

These tests MUST be written FIRST (failing), then made green by implementation. All 9 are
load-bearing behavioral assertions — no `assert!(true)` placeholders (TD-VSDD-059).

| # | Test Name | Location | Gate | DTU Required? |
|---|-----------|----------|------|---------------|
| RG-01 | `test_BC_2_16_001_loads_4_bundled_specs_at_boot` | `crates/prism-spec-engine/tests/bc_2_16_001_bundled_spec_load.rs` | All 4 specs load without errors; tables registered | No |
| RG-02 | `test_BC_2_16_009_validates_all_4_bundled_specs` | `crates/prism-spec-engine/tests/bc_2_16_009_bundled_spec_validation.rs` | All 4 specs return `Ok(SensorSpec)` from validation pipeline | No |
| RG-03 | `test_BC_2_16_002_pipeline_executor_runs_crowdstrike_two_step_spec` | `crates/prism-spec-engine/tests/bc_2_16_002_crowdstrike_two_step.rs` | PipelineExecutor executes crowdstrike `detections` two-step spec against mock HTTP server; produces RecordBatch with correct column schema | No (uses in-process HTTP mock) |
| RG-04 | `test_BC_2_16_013_dtu_parity_crowdstrike` | `crates/prism-spec-engine/tests/parity/crowdstrike.rs` | Parity verdict PASS or WARN (zero FAILs) for CrowdStrike detections + batch-cap fixture | Yes (S-6.07); `#[ignore]` until DTU merges |
| RG-05 | `test_BC_2_16_013_dtu_parity_claroty` | `crates/prism-spec-engine/tests/parity/claroty.rs` | Parity verdict PASS or WARN; integer + UUID ID fixtures both PASS | Yes (S-6.08); `#[ignore]` until DTU merges |
| RG-06 | `test_BC_2_16_013_dtu_parity_cyberint` | `crates/prism-spec-engine/tests/parity/cyberint.rs` | Alerts parity PASS or WARN; incidents sub-test returns SKIP verdict (explicit SKIP assertion) | Yes (S-6.09); `#[ignore]` until DTU merges |
| RG-07 | `test_BC_2_16_013_dtu_parity_armis` | `crates/prism-spec-engine/tests/parity/armis.rs` | AQL forwarding verified; timestamp fallback parity PASS by convention | Yes (S-6.10); `#[ignore]` until DTU merges |
| RG-08 | `test_BC_2_16_012_plugin_dispatch_uses_spec_catalog_not_hardcoded_names` | `crates/prism-spec-engine/tests/bc_2_16_012_open_dispatch_bundled_specs.rs` | SensorSpec produced for crowdstrike.sensor.toml is byte-identical across two parse calls; grep gate asserts zero hardcoded sensor name match arms in spec_parser.rs | No |
| RG-09 | `test_BC_2_16_013_spec_id_mismatch_rejected` | `crates/prism-spec-engine/tests/bc_2_16_013_spec_id_mismatch.rs` | A TOML file where `sensor_id != filename_stem` (e.g., file named `crowdstrike.sensor.toml` with `sensor_id = "falcon"`) is rejected with `E-SPEC-009`-equivalent at load time | No |

**Total Red Gate tests: 9** (matches `red_gate_tests: 9` frontmatter field)

Note on RG-04 through RG-07: The `#[ignore]` tag means these tests are registered in the nextest
inventory and can be run explicitly with `--ignored` when the DTU clone stories have merged. CI
runs them as `#[ignore]` and they contribute 0 to the "pass/fail" count but are visible in the
test listing. This is the standard pattern established by the DTU story framework (EC-016-013-006).

---

## Task Breakdown

### Task 1: TOML Grammar Verification — Already Completed by PO (0.5 point)

The PO completed grammar verification during FB-IMPL-P1-PO (O-001, 2026-05-20) against
`crates/prism-spec-engine/src/spec_parser.rs` and `crates/prism-spec-engine/src/pipeline.rs`.
Verified status (authoritative — do NOT re-verify from memory):

| Field | Status | Action for Implementer |
|-------|--------|------------------------|
| `fan_out_batch_size` on `FetchStep` | **SUPPORTED** — `pub fan_out_batch_size: Option<u32>` in `FetchStep` (`spec_parser.rs:128`) | Use directly in `crowdstrike.sensor.toml` |
| `${query.filter.aql}` interpolation | **SUPPORTED** — `FetchContext::query_filters: HashMap<String, String>` seeded as `query.filter.{k}` in step_vars (`pipeline.rs:246-250`) | Use `${query.filter.aql}` in `armis.sensor.toml` path/body templates; caller must pass `query_filters["aql"]` in `FetchContext` |
| `timestamp_format = "multi"` | **NOT SUPPORTED** — no such field in `FetchStep`, `TableSpec`, `ColumnSpec`, or `SensorSpec` | Choose **Option A** (add `timestamp_format: Option<String>` to `ColumnSpec` via `#[serde(default)]`) OR **Option B** (WASM transformer plugin) before authoring `cyberint.sensor.toml`; document chosen option in a task comment |
| `timestamp_fallback_chain = [...]` | **NOT SUPPORTED** — no such field in `spec_parser.rs` | Choose **Option A** (add `timestamp_fallback_chain: Option<Vec<String>>` to `TableSpec` or `ColumnSpec` via `#[serde(default)]`) OR **Option B** (WASM transformer plugin) before authoring `armis.sensor.toml`; document chosen option in a task comment |

**Implementer decision for Options A/B:**
- **Option A (Grammar Extension):** Preferred when the transform is simple (threshold: if the Rust
  implementation in `pipeline.rs` is ≤ ~20 lines for the handler). Add the new field via
  `#[serde(default)]` to avoid breaking existing spec files. Handle in `pipeline.rs` execute_impl.
- **Option B (WASM Plugin):** Preferred when the transform requires complex parsing logic (e.g.,
  multi-format datetime parsing with fallback chains). The spec references the plugin via
  `(sensor_id, table)` dispatch in the SpecDrivenMapper.
- **Both options are in scope for PLUGIN-MIGRATION-001-D.** If scope is genuinely uncompletable
  in one story, notify the orchestrator for sub-story creation before beginning implementation.

Note: `sensor_config` key-value map was not evaluated in O-001; read `spec_parser.rs` to confirm
support if the Armis adapter's `aql_query` key requires it; otherwise use `${query.filter.aql}`
which is confirmed supported per O-001.

### Task 2: Create `crates/prism-sensors/specs/` Directory (0 points — housekeeping)

```
mkdir -p crates/prism-sensors/specs
```

No Cargo.toml change needed — the specs directory is a data directory, not a Rust source path.

### Task 3: Author `crowdstrike.sensor.toml` (0.75 point)

Reverse-engineer from `crates/prism-sensors/src/auth/crowdstrike.rs` (read in full):
- `CROWDSTRIKE_BATCH_SIZE = 100` → `fan_out_batch_size = 100`
- `base_url` pattern: `"https://api.{cloud_region}.crowdstrike.com"` → parameterized
- Endpoints:
  - `detections.step1` GET `/detects/queries/detects/v1` (QueryV2)
  - `detections.step2` POST `/detects/entities/summaries/GET/v1` (PostEntities) with IDs from step1
  - `devices.step1` GET `/devices/queries/devices/v1` (QueryV2)
  - `devices.step2` POST `/devices/entities/devices/v1` with IDs from step1
  - `incidents` GET `/incidents/queries/incidents/v1` cursor pagination
- Column schema from `CrowdStrikeAdapter::fetch_page()` Arrow schema construction
- OCSF field mappings from prior normalization logic

### Task 4: Author `claroty.sensor.toml` (0.75 point)

Reverse-engineer from `crates/prism-sensors/src/auth/claroty.rs` (read in full):
- `auth_type = "bearer_static"`
- Endpoints: POST `/xdome/api/v1/assets`, POST `/xdome/api/v1/alerts`, `/api/v1/audit_logs`
- POST-for-read pattern: use `method = "POST"` with body template
- Offset pagination: `page_size = 100`
- `id` column: `type = "string"` to handle polymorphic int/UUID ID (EC-016-013-004)
- `audit_logs` table: Claroty hybrid pagination via `paginate_claroty` semantics

### Task 5: Author `cyberint.sensor.toml` (0.5 point)

Reverse-engineer from `crates/prism-sensors/src/auth/cyberint.rs` (read in full):
- `auth_type = "cookie_roundtrip"`; base URL from `${env.CYBERINT_ENVIRONMENT}`
- `alerts`: GET `/api/v1/alerts`; cursor pagination; `timestamp_format = "multi"`
- `incidents`: GET `/api/v1/incidents`; cursor pagination; note in spec comments that DTU gap
  means parity is SKIP per EC-016-013-002

### Task 6: Author `armis.sensor.toml` (0.5 point)

Reverse-engineer from `crates/prism-sensors/src/auth/armis.rs` (read in full):
- `auth_type = "api_key"`; base URL from `${env.ARMIS_INSTANCE_URL}`
- `devices`: GET `/api/v1/search/`; AQL in `${query.filter.aql}` (interpolated from
  `FetchContext::query_filters["aql"]`); page pagination
- `alerts`: GET `/api/v1/alerts/`; AQL forwarding via `${query.filter.aql}`; page pagination
- `timestamp_fallback_chain = ["firstSeen", "lastSeen"]` extension

### Task 7: Author Red Gate Tests — Non-DTU (0.5 point)

Write RG-01, RG-02, RG-03, RG-08, RG-09 as failing tests first. Use existing
`crates/prism-spec-engine/tests/bc_2_16_001_test.rs` etc. as template for test structure.

RG-03 (`crowdstrike_two_step`) uses an in-process HTTP mock (e.g., `wiremock-rs` or
`mockito`) rather than the full DTU clone — this keeps it DTU-independent and fast.

### Task 8: Create `crates/prism-spec-engine/tests/parity/` Directory (0 points — housekeeping)

Add `parity/mod.rs` or individual test files under `crates/prism-spec-engine/tests/parity/`.
Add the parity test files to the existing `[[test]]` table or as integration tests. Verify the
directory is picked up by nextest automatically (since it's under `tests/`).

### Task 9: Author Red Gate Tests — DTU Parity (with `#[ignore]` tags) (0.5 point)

Write RG-04, RG-05, RG-06, RG-07. These tests have `#[ignore]` so they compile and run in
"SKIP" mode. The test body MUST be a complete behavioral assertion (not `todo!()`) — the
`#[ignore]` tag is the mechanism that prevents CI failure, not an incomplete test body.

Use the `BehavioralClone::start_on("127.0.0.1:0".parse().unwrap(), None, None).await` pattern
documented in BC-2.16.013 v1.1 §Postconditions §2 and confirmed from `prism-dtu-common/src/clone.rs`.
The returned `SocketAddr` is used to construct the DTU base URL for spec override.
The `NullAuthProvider` must be imported or defined in the parity test harness (it exists in
`prism-spec-engine` under `test-helpers` feature per the existing test suite convention).

### Task 10: Wire `prism-dtu-{sensor}` as `[dev-dependencies]` in `prism-spec-engine` (0.5 point)

Add to `crates/prism-spec-engine/Cargo.toml` under `[dev-dependencies]`:
```toml
prism-dtu-crowdstrike = { path = "../prism-dtu-crowdstrike", features = [] }
prism-dtu-claroty = { path = "../prism-dtu-claroty", features = [] }
prism-dtu-cyberint = { path = "../prism-dtu-cyberint", features = [] }
prism-dtu-armis = { path = "../prism-dtu-armis", features = [] }
```
Confirm these paths exist and compile before the parity tests are authored.

Note: `prism-spec-engine` MUST NOT add these as production `[dependencies]` — only `[dev-dependencies]`.
This is an architecture compliance rule: the spec engine must not have a production dependency on
test-infrastructure crates.

### Task 11: Full Workspace Gate (0 points — process gate)

Run `just check` once at the end of the fix-burst. All 9 Red Gate tests must appear in the
nextest run (RG-01 through RG-09); 5 pass unconditionally; 4 are `#[ignore]`-skipped. No new
clippy warnings. `just fmt` clean.

---

## Previous Story Intelligence

**Most recent merged predecessor:** S-PLUGIN-PREREQ-E (PR #151, develop@80ebe794, 2026-05-19)

Key lessons applicable to this story:
1. **BC↔body bidirectional trace mandatory before status=ready.** Every AC in this story body
   cites `(traces to BC-S.SS.NNN ...)`. Every BC in the `behavioral_contracts` frontmatter array
   is cited by at least one AC. Pre-commit verification required (BC Array Propagation Policy).

2. **Structured event catalog discipline (PG-LP11-001).** If any parity test harness emits a new
   `tracing::*!(event_type = "...")` site, it MUST be registered in BC-2.16.002 §Postconditions
   Canonical Structured Event Catalog in the same commit. The test harness runs within the same
   process as PipelineExecutor — new `event_type` values in test code are not exempt.
   (Exception: test-only `eprintln!` / `tracing::debug!` diagnostic emissions with no `event_type`
   field do not need catalog entries.)

3. **Single-commit-per-burst (TD-VSDD-053).** Do NOT commit the 4 TOML files separately from
   the tests. All story artifacts MUST be in a single squash-commit to develop.

4. **`#[non_exhaustive]` discipline.** No new public types added in this story. If the parity
   test harness introduces a public `ParityVerdict` enum or similar, it must be `#[non_exhaustive]`
   if it is in a non-test module. Parity test code under `#[cfg(test)]` or `tests/` is exempt.

5. **Sibling-sweep on any new error code.** If the implementation introduces a new validation
   error code (unlikely — specs should not need new codes), run TD-VSDD-060 sibling-sweep to
   update error-taxonomy.md in the same commit.

6. **`sensor_id`/filename mismatch uses `E-SPEC-009` (confirmed during BC-2.16.013 v1.1 authoring).**
   The previously cited `E-SPEC-016` was a fabricated code that does not exist in error-taxonomy.md.
   BC-2.16.013 v1.1 §Error Conditions now correctly documents that `E-SPEC-009` (existing canonical
   code for sensor_id/filename-stem mismatch) is emitted by BC-2.16.001 at spec-load time. The
   implementer should verify the exact call site in `spec_parser.rs` and confirm `E-SPEC-009` is
   the error variant returned — the BC is the authority, not any internal label.

---

## Architecture Compliance Rules

These rules are extracted from the architecture section files referenced in `inputs:` above.
The implementer MUST NOT violate any of these — violations are P0 findings in adversarial review.

| Rule | Source | Constraint |
|------|--------|-----------|
| Spec files at `crates/prism-sensors/specs/` | BC-2.16.013 postcondition 1 + ADR-023 §Rule 1 | Bundled spec canonical path. Do NOT place at `sensor-specs/`, `specs/`, or any other root-level directory. The path `crates/prism-sensors/specs/` is the canonical location per BC-2.16.013. |
| File naming: `{sensor_id}.sensor.toml` | BC-2.16.013 §Error Conditions E-SPEC-009 | filename stem MUST match `sensor_id` field. `crowdstrike.sensor.toml` → `sensor_id = "crowdstrike"`. |
| No new hardcoded sensor dispatch arms | BC-2.16.012 INV-SPEC-PARSER-OPEN-001 | After this story, `grep -rn '"crowdstrike"\|"cyberint"' crates/prism-spec-engine/src/spec_parser.rs` in dispatch-context code returns ZERO. |
| `prism-spec-engine` MUST NOT depend on `arrow` or `datafusion` in production | `crates/prism-spec-engine/Cargo.toml` line 8 | DTU crates are `[dev-dependencies]` only. Arrow types visible in test code must be imported via `#[cfg(test)]` or test module guards. |
| `NullAuthProvider` is test-helpers-gated | Existing convention in `prism-spec-engine` | Use `#[cfg(feature = "test-helpers")]` guard on `NullAuthProvider` usage in parity tests; the `prism-spec-engine = { path = ".", features = ["test-helpers"] }` dev-dep in `prism-spec-engine/Cargo.toml` (line 50) already enables this. |
| No production `reqwest::Client::new()` without timeout | CLAUDE.md §Conventions + TD-S-PLUGIN-PREREQ-B-005 | The parity test harness drives PipelineExecutor which uses its own internal HTTP client wired at boot. The parity tests themselves do NOT construct their own reqwest::Client for the DUT path. |
| No `println!` in any parity test harness (non-assertion) code | CLAUDE.md §Conventions | Use `tracing::debug!` or `eprintln!` for test-only diagnostics. `println!` is restricted to examples and CLI formatting helpers. |
| Single `auth_type` per spec (ADR-023 Rule 2) | BC-2.01.013 postcondition 4 + BC-2.01.016 INV-AUTH-OPEN-003 | Each of the 4 specs declares exactly ONE `auth_type` value. Spec-load runtime rejects arrays. |

---

## Library & Framework Requirements

| Library | Version | Purpose | Source |
|---------|---------|---------|--------|
| `reqwest` | `"0.12"` | HTTP client within `PipelineExecutor` (parity tests drive this indirectly) | `crates/prism-spec-engine/Cargo.toml` line 41 |
| `toml` | `"0.8"` | TOML spec file parsing in `spec_parser.rs` | `crates/prism-spec-engine/Cargo.toml` line 15 |
| `serde` / `serde_json` | `"1"` | Response deserialization in parity test harness | `crates/prism-spec-engine/Cargo.toml` line 14 / 18 |
| `arrow` | `"58"` | Arrow RecordBatch comparison in parity assertion (dev-dep only; do NOT add to `[dependencies]`) | `crates/prism-sensors/Cargo.toml` line 29 — use same `"58"` pin if added to `prism-spec-engine` dev-deps |
| `prism-dtu-crowdstrike` | `0.1.0` | CrowdStrike DTU clone; `BehavioralClone::start_on(bind, shutdown, tls)` via `prism_dtu_common::BehavioralClone` trait | `crates/prism-dtu-crowdstrike/Cargo.toml` |
| `prism-dtu-claroty` | `0.1.0` | Claroty DTU clone | `crates/prism-dtu-claroty/Cargo.toml` |
| `prism-dtu-cyberint` | `0.1.0` | Cyberint DTU clone | `crates/prism-dtu-cyberint/Cargo.toml` |
| `prism-dtu-armis` | `0.1.0` | Armis DTU clone | `crates/prism-dtu-armis/Cargo.toml` |
| `axum` | `"0.7"` | Used internally by DTU clones (transitive; no direct use in parity tests) | `crates/prism-dtu-crowdstrike/Cargo.toml` |
| `tokio` | `"1"` | Async runtime in parity tests (`#[tokio::test]`) | Already in `prism-spec-engine` dev-deps |

**Do NOT invent version numbers.** All versions above are confirmed from the named `Cargo.toml`
files. If a version is not listed above, read the relevant `Cargo.toml` before adding a dep.

---

## File Structure Requirements

Files to CREATE:

| File | Action | Description |
|------|--------|-------------|
| `crates/prism-sensors/specs/crowdstrike.sensor.toml` | CREATE | CrowdStrike OAuth2 two-step pipeline spec |
| `crates/prism-sensors/specs/claroty.sensor.toml` | CREATE | Claroty bearer_static POST-for-read spec |
| `crates/prism-sensors/specs/cyberint.sensor.toml` | CREATE | Cyberint cookie_roundtrip cursor spec |
| `crates/prism-sensors/specs/armis.sensor.toml` | CREATE | Armis api_key AQL forwarding spec |
| `crates/prism-spec-engine/tests/bc_2_16_001_bundled_spec_load.rs` | CREATE | RG-01: 4 specs load at boot |
| `crates/prism-spec-engine/tests/bc_2_16_009_bundled_spec_validation.rs` | CREATE | RG-02: validation pipeline passes |
| `crates/prism-spec-engine/tests/bc_2_16_002_crowdstrike_two_step.rs` | CREATE | RG-03: two-step pipeline (in-process HTTP mock) |
| `crates/prism-spec-engine/tests/bc_2_16_012_open_dispatch_bundled_specs.rs` | CREATE | RG-08: open dispatch + grep gate |
| `crates/prism-spec-engine/tests/bc_2_16_013_spec_id_mismatch.rs` | CREATE | RG-09: sensor_id/filename mismatch rejected |
| `crates/prism-spec-engine/tests/parity/crowdstrike.rs` | CREATE | RG-04: CrowdStrike DTU parity (#[ignore]) |
| `crates/prism-spec-engine/tests/parity/claroty.rs` | CREATE | RG-05: Claroty DTU parity (#[ignore]) |
| `crates/prism-spec-engine/tests/parity/cyberint.rs` | CREATE | RG-06: Cyberint DTU parity (#[ignore]) + SKIP incidents |
| `crates/prism-spec-engine/tests/parity/armis.rs` | CREATE | RG-07: Armis DTU parity (#[ignore]) |
| `crates/prism-spec-engine/tests/parity/mod.rs` | CREATE (if needed) | Parity module declaration |

Files to MODIFY:

| File | Action | Description |
|------|--------|-------------|
| `crates/prism-spec-engine/Cargo.toml` | MODIFY | Add 4 DTU crates as `[dev-dependencies]`; optionally add `arrow = "58"` as dev-dep for RecordBatch assertion |
| `crates/prism-sensors/Cargo.toml` | MODIFY (if needed) | Only if a new `include!` or `build.rs` mechanism is needed to embed specs; otherwise NO CHANGE — the specs directory is discovered at runtime via `sensor_specs_dir` config, not at compile time |

**Forbidden file changes:**
- `crates/prism-spec-engine/src/spec_parser.rs` — no new hardcoded sensor name match arms
- `crates/prism-sensors/src/auth/*.rs` — these are read-only reference implementations in this story; deletion is PLUGIN-MIGRATION-001-A scope
- Any `.factory/` spec file — story-writer does NOT modify BCs or VPs

---

## Implementation Discipline

The following TD/rule anchors MUST be observed. They are listed here so the implementer cannot
miss them:

| Anchor | Constraint | Consequence if Violated |
|--------|------------|------------------------|
| TD-VSDD-053 | Single commit per burst. Do NOT commit specs separately from tests. Stage all files, single commit. | Multi-commit chain blocker (factory-dispatcher hook) |
| TD-VSDD-059 | No paper-fixes. DTU parity tests must have load-bearing assertions, not `assert!(true)` or `todo!()` bodies (even in `#[ignore]` tests). | P1 adversarial finding per paper-fix detection rule |
| TD-VSDD-060 | If any function signature, constant, or canonical identifier changes during implementation, sibling-sweep all callsites in the same crate and adjacent crates before committing. | P1 adversarial finding |
| INV-PARITY-001 | PLUGIN-MIGRATION-001-A MUST NOT proceed until VP-PLUGIN-003 is verified GREEN (or all SKIP justified). This story is the prerequisite gate. AC-012 + the STORY-INDEX `depends_on` edge enforce this structurally. | Replacement-before-deletion invariant violated; potential OCSF regression in production |
| BC-2.16.013 INV-PARITY-002 | Once committed, `sensor_id` in each spec is IMMUTABLE. Do not change it post-merge without a new BC. | Breaking change in DataFusion table namespace |
| BC-2.16.013 INV-PARITY-003 | After this story merges, TOML spec files (not Rust adapter source) are the source of truth for the 4 sensor table schemas. | Architecture drift if Rust adapter code is modified without updating the spec |
| PG-LP11-001 | New `event_type` tracing sites in the parity harness (if any) must be enumerated in BC-2.16.002 §Postconditions Canonical Structured Event Catalog in the same commit. | P1 adversarial finding (BC↔impl catalog drift) |

---

## Forbidden Dependencies

The following modules/packages MUST NOT appear in the production dependency graph of
`prism-spec-engine` after this story:

1. `arrow` / `datafusion` — production dependencies (dev-deps allowed; see `Cargo.toml` line 8)
2. `prism-dtu-crowdstrike` / `prism-dtu-claroty` / `prism-dtu-cyberint` / `prism-dtu-armis` —
   production dependencies (dev-deps only)
3. `prism-bin` — `prism-spec-engine` must never depend on `prism-bin` (circular risk)
4. Any new runtime dep on `prism-sensors` from `prism-spec-engine` (the existing `prism-spec-engine`
   dev-dep self-reference is the only cross-crate dep allowed in this relationship)

If any forbidden dep appears in `prism-spec-engine/Cargo.toml` under `[dependencies]` after this
story, the build MUST fail the architecture compliance check.

---

## Edge Cases

| ID | Source | Description | Expected Behavior |
|----|--------|-------------|-------------------|
| EC-016-013-001 | BC-2.16.013 | DTU clone not started when parity test runs | Test skipped (`#[ignore]`); marked as DTU debt in parity report |
| EC-016-013-002 | BC-2.16.013 | `cyberint.incidents` table parity test | Explicit SKIP assertion with message; NOT `#[ignore]` — the test runs and asserts SKIP |
| EC-016-013-003 | BC-2.16.013 | CrowdStrike two-step batch boundary (exactly 100 IDs) | Synthesized fixture in RG-04; spec MUST NOT produce a 101-item batch |
| EC-016-013-004 | BC-2.16.013 | Claroty polymorphic ID (integer vs UUID string) | Both fixture cases in RG-05; `id` column `type: "string"` normalizes both |
| EC-016-013-005 | BC-2.16.013 | Armis timestamp fallback to `now()` | Parity PASS by convention (Rule C); WARN logged |
| EC-016-013-006 | BC-2.16.013 | DTU clone not in scope for Wave 1 test run | `#[ignore]` tag on RG-04/05/06/07 with standard message |
| EC-016-013-007 | BC-2.16.013 | Null OCSF field in reference output absent from actual | Parity WARN (not FAIL); does not block VP-148 verification |
| EC-016-013-008 | BC-2.16.013 | `sensor_specs_dir` not configured to include bundled path | Integration tests (RG-01, RG-05) set `sensor_specs_dir` explicitly; mis-config is a test authoring defect |

---

## Risks and Open Questions

**Risks:**

1. **Two TOML grammar extension fields require implementer action before spec authoring.** The PO
   completed grammar verification in O-001 (FB-IMPL-P1-PO, 2026-05-20):

   - `fan_out_batch_size` — **SUPPORTED** (no action required; use directly in `crowdstrike.sensor.toml`)
   - `${query.filter.aql}` — **SUPPORTED** (no action required; use in `armis.sensor.toml` with
     `FetchContext::query_filters["aql"]` caller semantics)
   - `timestamp_format = "multi"` — **NOT SUPPORTED** — requires either:
     - **Option A:** Grammar extension — add `timestamp_format: Option<String>` to `ColumnSpec` in
       `spec_parser.rs` with `#[serde(default)]` and handle in `pipeline.rs::execute_impl`
     - **Option B:** WASM transformer plugin — reference via `(sensor_id, table)` dispatch
   - `timestamp_fallback_chain = [...]` — **NOT SUPPORTED** — requires either:
     - **Option A:** Grammar extension — add `timestamp_fallback_chain: Option<Vec<String>>` to
       `TableSpec` or `ColumnSpec` with `#[serde(default)]` and handle in `pipeline.rs::execute_impl`
     - **Option B:** WASM transformer plugin

   Both Option A and Option B are in scope for PLUGIN-MIGRATION-001-D. The implementer MUST
   choose one option for each unsupported field and implement it before authoring the Cyberint
   and Armis TOML specs. This is NOT a deferral — it is an implementation prerequisite with
   clear options. If scope cannot be completed in one story, notify the orchestrator immediately
   for sub-story creation before beginning TOML authoring.

2. **DTU parity fixture coverage.** BC-2.16.013 postcondition 2 requires minimum 3 real-sensor
   recordings AND 3 synthesized cases per `(sensor_id, table)` pair in `crates/prism-dtu-*/fixtures/parity/`.
   Those fixture directories do NOT exist yet (confirmed: no `parity/` subdir in any DTU crate).
   The implementer must either create the `fixtures/parity/` directories with synthesized fixtures
   (from the existing `fixtures/` top-level files) or document that the DTU story (S-6.07–6.10)
   must create them. This story creates the test HARNESS (RG-04 through RG-07) that will use
   those fixtures; if the fixture path is not ready, the `#[ignore]` tag is the correct mechanism.

3. **TS-PLUGIN-PARITY-001 not yet read.** BC-2.16.013 references `TS-PLUGIN-PARITY-001` (a test
   strategy document). The implementer must locate and read this file before authoring parity
   tests. Expected path: `.factory/specs/test-strategy/TS-PLUGIN-PARITY-001-dtu-canonicalization.md`
   (per BC-2.16.013 inputs). If the file does not exist, escalate to the orchestrator — it may
   need to be authored by the product-owner as a pre-flight dependency.

**Open questions:** None that block authoring. The 3 risks above are implementation-time decisions,
not spec-blocking uncertainties.

---

## Traceability Table

| AC | BC | BC Clause | VP | Test |
|----|----|-----------|----|------|
| AC-001 | BC-2.16.013 | Postcondition 1 (crowdstrike spec authored) | VP-148 | RG-01, RG-02 |
| AC-002 | BC-2.16.013 | Postcondition 1 (claroty spec authored) | VP-148 | RG-01, RG-02 |
| AC-003 | BC-2.16.013 | Postcondition 1 (cyberint spec authored) | VP-148 | RG-01, RG-02 |
| AC-004 | BC-2.16.013 | Postcondition 1 (armis spec authored) | VP-148 | RG-01, RG-02 |
| AC-005 | BC-2.16.009 | Postconditions (validation pipeline) | — | RG-02 |
| AC-006 | BC-2.16.001 | Postconditions 1, 2, 3, 6, 8 | — | RG-01 |
| AC-007 | BC-2.16.013, BC-2.16.002 | Postcondition 2 (CrowdStrike DTU parity); BC-2.16.002 fan-out + 2-step | VP-148 | RG-03, RG-04 |
| AC-008 | BC-2.16.013 | Postcondition 2 (Claroty DTU parity); EC-016-013-004 | VP-148 | RG-05 |
| AC-009 | BC-2.16.013 | Postcondition 2 (Cyberint parity); EC-016-013-002 (incidents SKIP) | VP-148 | RG-06 |
| AC-010 | BC-2.16.013 | Postcondition 2 (Armis parity); EC-016-013-005 | VP-148 | RG-07 |
| AC-011 | BC-2.01.016, BC-2.01.013 | INV-AUTH-OPEN-003; Postcondition 4 | — | RG-02 (implicit via validation) |
| AC-012 | BC-2.16.012 | INV-SPEC-PARSER-OPEN-001 | — | RG-08 |
| AC-013 | BC-2.16.013 | Postcondition 1 (production-grade defaults) | — | RG-01 through RG-09 (all must pass `just check`) |

---

## Definition of Done

This story is DONE when ALL of the following are simultaneously true — no exceptions, no
"MVP" deferrals, no "for now" placeholders:

1. Four TOML spec files exist at `crates/prism-sensors/specs/` with the exact canonical paths
   `crowdstrike.sensor.toml`, `claroty.sensor.toml`, `cyberint.sensor.toml`, `armis.sensor.toml`.

2. All 4 specs contain production-grade `sensor_id`, `auth_type`, `base_url`, `tables`,
   column schemas with accurate `type` and `ocsf_field` values reverse-engineered from the
   Rust adapter source. No `TODO`, `FIXME`, `placeholder`, or `example.com` values.

3. `test_BC_2_16_009_validates_all_4_bundled_specs` passes in CI with zero validation errors
   across all 4 specs.

4. `test_BC_2_16_001_loads_4_bundled_specs_at_boot` passes in CI with all 4 specs registered
   in the DataFusion catalog under the correct `{sensor_id}.{table_name}` namespaces.

5. `test_BC_2_16_002_pipeline_executor_runs_crowdstrike_two_step_spec` passes in CI using an
   in-process HTTP mock (no DTU clone required).

6. RG-08 (`test_BC_2_16_012_plugin_dispatch_uses_spec_catalog_not_hardcoded_names`) passes in
   CI confirming the open dispatch path works for the 4 bundled specs.

7. RG-09 (`test_BC_2_16_013_spec_id_mismatch_rejected`) passes in CI confirming sensor_id /
   filename mismatch is rejected.

8. Four DTU parity test files exist under `crates/prism-spec-engine/tests/parity/` (RG-04
   through RG-07) with complete non-placeholder test bodies, tagged `#[ignore]` with the
   standard message. The `cyberint.incidents` SKIP assertion test is NOT `#[ignore]`.

9. `crates/prism-spec-engine/Cargo.toml` lists all 4 DTU crates as `[dev-dependencies]`
   (no production dep).

10. `just check` completes with all tests passing (minimum 3,681 tests; SKIP-tagged parity tests
    appear in the nextest inventory but do not count as failures).

11. No new hardcoded sensor name match arms in `crates/prism-spec-engine/src/spec_parser.rs`
    dispatch code (grep gate confirmed by RG-08).

12. The squash-merge commit to `develop` passes `lefthook` pre-push hooks
    (`just check` including `fmt`, `clippy`, `nextest`, `crate-layout`).

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| v1.1 | 2026-05-20 | story-writer | FB-IMPL-P1-SW (fix-burst-1, D-733-pending): closes pass-1 adversarial findings F-001/F-002 (AC-007..010 rewritten with real `BehavioralClone::start_on` API + 5-arg `PipelineExecutor::execute` signature from `pipeline.rs`), F-005 (BC table titles corrected to full H1 per POL-7; BC-2.16.013 pin 1.0→1.1), F-008 (frontmatter comment "Sensor Adapter Layer" → "Sensor Adapters" per POL-6), F-009 (PG-LP11-001 anchor corrected to §Postconditions Canonical Structured Event Catalog), F-011 (AC-006 positional postcondition cite replaced with named-section cite). Cascade from PO: S-001 (BC-2.16.013 v-pin 1.0→1.1 throughout), S-002 (holdout scenarios HS-MIGRATION-D-001..006 → HS-013..HS-018 in frontmatter), S-003 (E-SPEC-015 references removed; E-SPEC-016 → E-SPEC-009 throughout; Previous Story Intelligence item 6 and AC-001 and Architecture Compliance table updated), S-004 (Task 1 rewritten with O-001 verified grammar status; Risk #1 rewritten with verified field statuses, no deferral language), S-005 (`${query.aql}` → `${query.filter.aql}` in AC-004, Task 6, and AC-010). Library table DTU clone API cite corrected. Task 9 start_on pattern corrected. |
| v1.0 | 2026-05-20 | story-writer | Initial materialization at D-732. Authored from 7 BC anchors (BC-2.16.013 v1.0 NEW + 6 existing), VP-148, ADR-023, and 4 Rust adapter source surveys. 13 ACs, 9 Red Gate tests, 5 points, 6 holdout scenarios. |
