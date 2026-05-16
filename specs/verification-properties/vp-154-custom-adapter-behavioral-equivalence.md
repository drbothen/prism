---
document_type: verification-property
level: L4
version: "0.3"
status: draft
producer: architect
timestamp: 2026-05-15T00:00:00Z
phase: prereq-e
inputs:
  - .factory/specs/architecture/decisions/ADR-027-custom-adapter-deprecation-removal.md
  - .factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md
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
introduced: plugin-prereq-e
modified: []
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

BC-2.16.011 does not specify a record schema for behavioral equivalence — it specifies grep gates
and build gates only. This VP defines the minimal acceptance schema for the WASM override path:

**Minimum record schema (OCSF-normalized, one event per record):**

```json
{
  "id": "<string — unique event ID from the fixture plugin>",
  "occurred_at": "<RFC 3339 timestamp string — e.g. '2026-01-01T00:00:00Z'>",
  "raw": "<JSON object — arbitrary plugin-returned fields, may be empty {}>"
}
```

The behavioral-equivalence integration test (`VP-154 harness`) asserts:
- `records.len() >= 1` (non-empty; WASM override returned at least one record)
- `records[0]["id"]` is a non-empty string
- `records[0]["occurred_at"]` is parseable as RFC 3339
- `records[0]["raw"]` is a JSON object (not null, not string)

These three fields (`id`, `occurred_at`, `raw`) are the OCSF base-event fields that all sensor
adapters normalize to (per OCSF + protobuf shapes per project core architecture). The fixture
WASM plugin MUST return at least this shape. Additional OCSF fields (e.g., `class_uid`,
`severity_id`, `type_uid`) are welcome but not required for the equivalence test.

**Proposed fixture record (canonical test vector):**

```json
{
  "id": "vp154-test-001",
  "occurred_at": "2026-01-01T00:00:00Z",
  "raw": {"source": "minimal_sensor_fetch_fixture", "event": "mock_event"}
}
```

**Routing note:** This schema is proposed by the architect (VP owner). The product-owner should
confirm or amend the three-field minimum before the test-writer authors the fixture WASM. If no
amendment arrives before test-writer dispatch, the three-field schema above is the binding
acceptance criterion per production-grade default (the architect's proposal holds; PO
confirmation is courteous, not blocking).

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
//     // 5. Assert behavioral equivalence to CustomAdapter::override_fetch returning Some(records)
//     let records = result.expect("execution must succeed");
//     assert!(!records.is_empty(), "WASM override path must return non-empty records (VP-154)");
//     assert_eq!(records[0]["id"], "test-001",
//         "WASM override must return the fixture plugin's records (VP-154)");
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
