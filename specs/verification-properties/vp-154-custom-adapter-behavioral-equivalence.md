---
document_type: verification-property
level: L4
version: "0.6"
status: draft
producer: architect
timestamp: 2026-05-15T00:00:00Z
phase: prereq-e
inputs:
  - .factory/specs/architecture/decisions/ADR-027-custom-adapter-deprecation-removal.md
  - .factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md
  - .factory/specs/behavioral-contracts/BC-2.16.011-customadapter-rust-trait-retirement.md
input-hash: "[pending-recompute]"
traces_to: .factory/specs/architecture/decisions/ADR-027-custom-adapter-deprecation-removal.md
source_bc: BC-2.16.011
source_adr: ADR-027
source_invariant: null
module: prism-spec-engine
priority: P1
proof_method: integration_test
verification_method: integration_test
feasibility: feasible
verification_lock: false
proof_completed_date: null
proof_file_hash: null
lifecycle_status: draft
introduced: "2026-05-15"
modified: "2026-05-15"
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-154: CustomAdapter Behavioral Equivalence — PluginRuntime Dispatch Produces Equivalent Output

## Property Statement

The deletion of `CustomAdapter` and `CustomAdapterRegistry` (ADR-027) must not cause a
behavioral regression: the WASM plugin dispatch path through `PluginRuntime` (delivered by
PREREQ-D and PREREQ-B) MUST produce equivalent output to the deleted `CustomAdapter::override_fetch`
path for the canonical mock-sensor test case.

Specifically, for a mock sensor spec with a registered WASM plugin providing the `sensor_fetch`
hook, a call to `PipelineExecutor::execute` with that sensor's `SensorId` MUST:

1. Invoke the WASM plugin hook rather than (or in addition to) the TOML-driven HTTP fetch,
   exactly as `CustomAdapter::override_fetch` returning `Some(records)` did.
2. Return the same record set (modulo OCSF normalization) as the mock plugin returns from its
   `sensor_fetch` export. The minimal accepted record schema is specified in §Acceptance Criteria.
3. NOT return `Ok(Vec::new())` (the pre-PREREQ-B stub behavior).
4. NOT panic or produce `Err` for a valid mock sensor + valid WASM plugin combination.

This is a behavioral-equivalence integration test: it validates that the WASM dispatch path
offers the same override semantics as the deleted Rust trait, ensuring no capability regression
for the non-declarative escape hatch use case.

## Acceptance Criteria

**Authoritative schema source: BC-2.16.011 §VP-154 Fixture Acceptance Criterion (imported verbatim).**

BC-2.16.011 v1.1 (S-PLUGIN-PREREQ-E-reconciliation, 2026-05-15) defines the canonical schema
that the integration test fixture MUST produce. That criterion is the single source of truth;
this VP imports it here to prevent drift (F-LP1-CRIT-001 resolution).

**Canonical OCSF Detection Finding (class 2004) record schema — fixture MUST conform to:**

```json
{
  "type_uid":        2004001,
  "class_uid":       2004,
  "category_uid":    2,
  "severity_id":     3,
  "severity":        "Medium",
  "time":            "<RFC 3339 timestamp, e.g., 2026-05-15T00:00:00Z>",
  "message":         "Mock sensor fetch result from WASM plugin fixture",
  "finding_info": {
    "uid": "test-001",
    "title": "mock_event"
  },
  "raw_data":        "{\"source\": \"minimal_sensor_fetch.prx\", \"id\": \"test-001\"}"
}
```

**Required fields (9):** `type_uid`, `class_uid`, `category_uid`, `severity_id`, `severity`,
`time`, `message`, `finding_info.uid`, `raw_data`. The `raw_data` field carries the
fixture-specific payload as a JSON-encoded string per the OCSF `raw_data` convention.

**Count threshold:** The integration test asserts `records.len() >= 1` (at least one record per
plugin-hook invocation when the `.prx` fixture is loaded and `PipelineExecutor::execute` is
issued for the mock sensor).

**Behavioral equivalence definition (semantic, not byte-identical):**
The `time` field varies per invocation; byte-identical comparison would produce flaky CI.
The harness asserts semantic equality on stable fields:
1. `records[0]["finding_info"]["uid"]` == `"test-001"` (fixture-controlled stable ID)
2. `records[0]["class_uid"]` == `2004` (Detection Finding class)
3. `records[0]["severity_id"]` is a valid OCSF integer (1–5 or 99)
4. `records.len() >= 1`

The fixture SHOULD emit a hardcoded timestamp (`"2026-01-01T00:00:00Z"`) to allow an optional
byte-identical CI mode. This is consistent with BC-2.16.011 §VP-154 Fixture Acceptance
Criterion (count threshold + behavioral equivalence definition subsections).

**Relationship to HS-PREREQ-E-002-04:** The holdout scenario HS-PREREQ-E-002-04 already follows
the BC-2.16.011 9-field schema. This VP is now aligned with both BC-2.16.011 and the holdout
scenario. No field-set contradiction remains.

## Source Contract

- **BC:** BC-2.16.011 — CustomAdapter removal postconditions: WASM plugin dispatch path through
  `PluginRuntime` MUST be behaviorally equivalent to the deleted `CustomAdapter::override_fetch`
  path. VP-154 is the primary verification property claimed by BC-2.16.011 §Verification
  Properties. This VP verifies BC-2.16.011's behavioral-equivalence postconditions directly.
- **ADR:** ADR-027 D3 / D5 — deletion of CustomAdapter, PluginRuntime as replacement
- **ADR:** ADR-023 Rule 5 — CustomAdapter retired; .prx WASM is the sole escape hatch
- **Companion VP:** VP-147 (VP-PLUGIN-002) — PipelineExecutor::execute returns non-empty records
  against wiremock DTU clone (PREREQ-B anchor); VP-154 adds the CustomAdapter-equivalence
  scenario on top of that infrastructure, verifying the non-declarative override semantics
- **Module:** prism-spec-engine (PipelineExecutor + PluginRuntime integration)
- **Category:** Behavioral Equivalence / Regression Prevention

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| integration_test | wiremock + minimal WASM fixture | Yes — canonical mock-sensor scenario | WASM override path produces non-empty, non-panicking result matching the expected record set |

**Feasibility:** PREREQ-B already establishes end-to-end integration test infrastructure
against a wiremock DTU clone (VP-PLUGIN-002). This VP adds a narrow WASM-plugin-override
scenario to that test infrastructure. A minimal WASM fixture that implements `sensor_fetch`
and returns a known record set is the test double; the integration test then verifies
`PipelineExecutor::execute` calls the WASM hook and returns the expected records.

**Dependency:** This VP requires PREREQ-B (real `PipelineExecutor::execute`) AND PREREQ-D
(`PluginRuntime::load_all_plugins` wired) to be delivered before the harness is meaningful.
The VP is authored in PLUGIN-MIGRATION-001-A scope (Wave 1/A), after both prereqs are merged.
Its priority is P1 (not blocking PREREQ-E delivery) but it MUST pass before Wave 1/A closes.

## Proof Harness Skeleton

```rust
// crates/prism-spec-engine/tests/vp154_custom_adapter_behavioral_equivalence.rs
//
// VP-154: CustomAdapter behavioral equivalence — PluginRuntime dispatch
// Method: integration_test
// Target: prism_spec_engine::pipeline::PipelineExecutor + prism_spec_engine::plugin::PluginRuntime
// Requires: PREREQ-B (real PipelineExecutor) + PREREQ-D (wired PluginRuntime) both merged
//
// #[tokio::test]
// async fn wasm_plugin_override_returns_expected_records() {
//     // 1. Build a minimal WASM fixture (.prx) that implements `sensor_fetch`
//     //    and returns a known record set: [ {"id": "test-001", "event": "mock_event"} ]
//     let wasm_fixture_path = fixture_path("minimal_sensor_fetch.prx");
//
//     // 2. Construct a PluginRuntime and load the fixture plugin
//     let runtime = PluginRuntime::new_for_test();
//     runtime.load_plugin(&wasm_fixture_path).expect("fixture plugin must load");
//
//     // 3. Build a SensorSpec for sensor_id="mock-override-sensor" that declares
//     //    plugin_id = "minimal_sensor_fetch" as its fetch hook
//     let spec = SensorSpec::fixture_with_plugin_hook("mock-override-sensor", "minimal_sensor_fetch");
//
//     // 4. Execute via PipelineExecutor
//     let executor = PipelineExecutor::new_for_test(Arc::new(runtime));
//     let result = executor.execute(&spec, &FetchContext::default()).await;
//
//     // 5. Assert behavioral equivalence: semantic equality on stable fields (BC-2.16.011 §VP-154)
//     let records = result.expect("execution must succeed");
//     assert!(!records.is_empty(), "WASM override path must return non-empty records (VP-154)");
//     // Required OCSF 2004 Detection Finding fields (BC-2.16.011 §VP-154 Fixture Acceptance Criterion)
//     assert_eq!(records[0]["class_uid"], 2004,
//         "WASM override must return OCSF class_uid=2004 Detection Finding (VP-154)");
//     assert_eq!(records[0]["finding_info"]["uid"], "test-001",
//         "WASM override must return fixture-controlled stable ID (VP-154)");
//     let severity_id = records[0]["severity_id"].as_i64().expect("severity_id must be integer");
//     assert!((1..=5).contains(&severity_id) || severity_id == 99,
//         "severity_id must be valid OCSF integer 1-5 or 99 (VP-154)");
// }
//
// #[tokio::test]
// async fn wasm_plugin_absent_falls_through_to_toml_pipeline() {
//     // Verifies that when no WASM plugin is registered for a sensor_id,
//     // PipelineExecutor falls through to the TOML-driven HTTP fetch path —
//     // matching the CustomAdapter::override_fetch returning None behavior.
//     let executor = PipelineExecutor::new_for_test(Arc::new(PluginRuntime::empty_for_test()));
//     let spec = SensorSpec::fixture_toml_only("mock-toml-sensor");
//     // Use wiremock to provide a minimal HTTP response
//     let server = wiremock_server_with_response(json!([{"id": "toml-001"}]));
//     let result = executor.execute(&spec, &FetchContext::from_server(&server)).await;
//     let records = result.expect("TOML fallthrough must succeed");
//     assert!(!records.is_empty(), "TOML path must return non-empty records (VP-154)");
// }
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|-----------|-------|
| Input space size | Bounded | Two scenarios: WASM override path + TOML fallthrough path |
| Proof complexity | Moderate | Requires real PluginRuntime + PipelineExecutor integration; fixture WASM must be compilable |
| Tool support | Full | tokio::test, wiremock, wasmtime; all in workspace |
| Harness dependencies | High | Both PREREQ-B and PREREQ-D must be merged; fixture WASM must be authored |
| Estimated proof time | 5–30 seconds | Integration test startup + WASM load overhead |

**Harness authoring note:** The minimal `.prx` fixture for this test is a new artifact that
must be authored as part of PLUGIN-MIGRATION-001-A. It should live at
`crates/prism-spec-engine/tests/fixtures/minimal_sensor_fetch.prx` or compiled from a Rust
source at `crates/plugins/test-minimal-sensor-fetch/`. The test-writer for PLUGIN-MIGRATION-001-A
owns the fixture authoring.

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-05-15 | architect (PREREQ-E ADR burst) |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 0.1 | plugin-prereq-e-adr-burst | 2026-05-15 | architect | Initial stub. Traces to ADR-027 D3/D5 and ADR-023 Rule 5. Harness skeleton provided; full authoring deferred to PLUGIN-MIGRATION-001-A (requires PREREQ-B + PREREQ-D merged). Priority P1 — does not block PREREQ-E gate but must pass before Wave 1/A closes. |
| 0.2 | plugin-prereq-e-cross-review | 2026-05-15 | architect | Q4 resolution: BC-2.16.011 does not specify a record schema for behavioral equivalence (grep/build gates only). Architect proposes minimal three-field OCSF schema (id, occurred_at, raw) as the acceptance criterion per production-grade default. §Acceptance Criteria section added. Routing note to PO for confirmation before test-writer dispatch. |
| 0.3 | plugin-prereq-e-spec-gate | 2026-05-15 | architect | source_bc anchor set to BC-2.16.011 (bidirectional traceability fix; consistency-validator D-574 invariant 10). §Source Contract rewritten to lead with BC-2.16.011 as the owning contract; ADR-027/ADR-023 and companion VP-147 remain as supporting references. |
| 0.4 | prereq-e-fix-burst-1 | 2026-05-15 | architect | F-LP1-CRIT-001 resolution: §Acceptance Criteria completely rewritten to import BC-2.16.011 §VP-154 Fixture Acceptance Criterion verbatim. Old 3-field schema (id/occurred_at/raw) replaced by the canonical OCSF 2004 Detection Finding 9-field schema (type_uid/class_uid/category_uid/severity_id/severity/time/message/finding_info.uid/raw_data). Proof harness skeleton updated to assert class_uid=2004, finding_info.uid="test-001", and valid severity_id range — aligning with BC-2.16.011 behavioral equivalence definition (semantic not byte-identical). BC-2.16.011 added to inputs frontmatter. |
| 0.5 | fix-burst-1 state-manager catch | 2026-05-15 | state-manager | (state-manager catch in fix-burst-1) F-LP1-HIGH-004 POL-20: introduced field canonicalized to ISO date 2026-05-15. Prior value `plugin-prereq-e` was informal slug; POL-20 requires `YYYY-MM-DD` for artifacts created outside greenfield cycles. |
| 0.6 | fix-burst-5 renumber-repair-redo | 2026-05-15 | state-manager | F-LP5-HIGH-003 renumber-repair-redo. FB4 assigned both the changelog-repair row and the modified-field-sync row to v0.5, producing two rows at the same version and violating monotonic strict order. Repair row renumbered 0.5→0.6. Absorbs FB4 modified-field-sync content: `modified:` field confirmed synced to ISO date "2026-05-15" per F-LP4-LOW-002 / POL-27 (most recent change: fix-burst-1 schema rewrite + fix-burst-3 happens-before retract). Content summary retained: prior changelog had duplicate 0.4 entries (architect prereq-e-fix-burst-1 + state-manager catch both labeled 0.4); state-manager catch correctly renumbered to 0.5. Each distinct content change now holds a unique version. Frontmatter version updated to 0.6. Monotonic sequence verified: 0.1 → 0.2 → 0.3 → 0.4 → 0.5 → 0.6. |
