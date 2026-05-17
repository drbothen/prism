---
document_type: holdout-scenario
level: L3
id: "HS-PREREQ-E-003"
title: "PluginRegistry Dispatch Migration — spec_parser.rs Open Dispatch Behavioral Equivalence"
category: "plugin-migration"
must_pass: true
priority: P0
epic_id: "PLUGIN-MIGRATION-001"
story_source: "S-PLUGIN-PREREQ-E"
version: "1.7"
status: draft
producer: product-owner
timestamp: 2026-05-16T00:00:00Z
phase: 4
inputs: []
input-hash: null
traces_to: "BC-2.16.012"
behavioral_contracts:
  - BC-2.16.012
  - BC-2.16.001
verification_properties:
  - VP-156
lifecycle_status: active
introduced: S-PLUGIN-PREREQ-E
last_evaluated: null
staleness_check: null
stale_reason: null
retired: null
assumption_source: null
risk_source: null
---

# HS-PREREQ-E-003: PluginRegistry Dispatch Migration — spec_parser.rs Open Dispatch and WriteToolInvalidationMap Extensibility

**Story:** S-PLUGIN-PREREQ-E
**Must Pass:** YES (P0 — behavioral equivalence for four initial sensors is a Wave 0 gate)
**BC Traced:** BC-2.16.012 (dispatch migration contract)

---

## Scenario Description

After PREREQ-E, `spec_parser.rs` no longer contains hardcoded match arms keyed on sensor name
strings for dispatch. This scenario verifies that (a) the open-dispatch path produces
byte-identical `SensorSpec` output for the four initial sensors, (b) a novel sensor name parses
generically without error, and (c) `WriteToolInvalidationMap` accepts runtime registration of
new write-tool entries (closing TD-S-PLUGIN-PREREQ-A-003).

---

## HS-PREREQ-E-003-01: Known-Good Corpus — Four Initial Sensor Specs Produce Identical SensorSpec Output

**Title:** Parsing each of the four built-in TOML sensor specs produces SensorSpec identical to pre-PREREQ-E baseline

**Preconditions:**

- S-PLUGIN-PREREQ-E is merged to `develop`
- Baseline `SensorSpec` snapshots for all four sensors are captured from `develop` before the PREREQ-E squash-merge (or from the last green `develop` SHA, recorded in the PR description)
- The spec_parser migration is complete: no hardcoded sensor-name match arms remain in `spec_parser.rs`

**Steps:**

1. For each of `crowdstrike.sensor.toml`, `cyberint.sensor.toml`, `claroty.sensor.toml`, `armis.sensor.toml`:
   a. Parse via `SpecParser::parse_file` (or equivalent public API)
   b. Capture the resulting `SensorSpec` struct (serialized to JSON or compared field-by-field)
   c. Compare against the pre-PREREQ-E baseline snapshot
2. Run `grep -rn '"crowdstrike"\|"cyberint"\|"claroty"\|"armis"' crates/prism-spec-engine/src/spec_parser.rs` (in dispatch contexts)

**Expected Outcome:**

- All four `SensorSpec` comparisons are byte-identical to baseline (same auth_type, same tables, same columns, same pagination config)
- Grep for hardcoded sensor names in dispatch context returns ZERO matches
- `cargo nextest run -p prism-spec-engine -E 'test(spec_parser)'` exits 0

**Repos Tested:** prism-spec-engine (spec_parser.rs)

---

## HS-PREREQ-E-003-02: Known-Problematic Corpus — Novel Sensor Name Parses Without Error

**Title:** A TOML spec with a novel, unregistered sensor name parses generically and registers tables

**Preconditions:**

- S-PLUGIN-PREREQ-E is merged
- A fixture TOML file `hypothetical.sensor.toml` is authored with:
  - `sensor_id = "hypothetical_sensor"`
  - `auth_type = "bearer_static"`
  - One table with two columns (string + integer)
  - `base_url = "https://api.example.com"`
- The `PluginRegistry` does NOT contain an entry for `hypothetical_sensor`

**Steps:**

1. Start `prism` with `hypothetical.sensor.toml` in the spec directory alongside the four built-in specs
2. Call `list_sensor_specs` MCP tool — observe `hypothetical_sensor` in the list
3. Run `SHOW TABLES IN hypothetical_sensor` via PrismQL — observe the table listed
4. Confirm `list_sensor_specs` output shows `status: loaded` for `hypothetical_sensor` (tables registered; queries will fail at credential resolution if no credentials are configured — that is acceptable behavior)

**Expected Outcome:**

- `hypothetical_sensor` appears in `list_sensor_specs` output
- Its table(s) appear in `SHOW TABLES IN hypothetical_sensor`
- No "unrecognized sensor" or "hardcoded sensor check failed" error is returned during loading
- The four built-in sensors are unaffected

**Repos Tested:** prism-spec-engine, prism-bin (spec loading path), prism-mcp (list_sensor_specs tool)

---

## HS-PREREQ-E-003-03: WriteToolInvalidationMap Runtime Extensibility (TD-S-PLUGIN-PREREQ-A-003 Closure)

**Title:** Plugin-registered write tool participates in cache invalidation after `register_write_tool` call

**Preconditions:**

- S-PLUGIN-PREREQ-E is merged
- `WriteToolInvalidationMap` container in `crates/prism-query/src/invalidation.rs` is runtime-extensible (RwLock or equivalent)
- A `register_write_tool(entry: WriteToolInvalidationMap)` API exists and is callable at runtime
- A test fixture registers a custom write tool entry for `SensorId::from("custom_sensor")` with tool name `"write_custom_sensor_record"`

**Steps:**

1. Construct a `WriteToolInvalidationMap` entry: `{ sensor_id: SensorId::from("custom_sensor"), tool_name: "write_custom_sensor_record", plugin_name: "plugin_a", ... }` (`plugin_name` sourced from plugin manifest `name` field per ADR-026 D7 v1.10)
2. Call `register_write_tool(entry)` after initial boot
3. Trigger a write via `write_custom_sensor_record` MCP tool (or a test stub that mimics the write dispatch)
4. Observe whether cache invalidation fires for `custom_sensor` tables

**Expected Outcome:**

- `register_write_tool` returns without error
- After registration, the invalidation map `RwLock::read()` contains the new entry
- A write dispatch to `write_custom_sensor_record` causes cache invalidation for `SensorId::from("custom_sensor")`
- The static built-in entries (CrowdStrike, Cyberint, etc.) remain unaffected

**Repos Tested:** prism-query (invalidation.rs), prism-spec-engine (PluginRuntime registration path)

---

## HS-PREREQ-E-003-04: Duplicate Write Tool Name — Second Registration Rejected with E-PLUGIN-012

**Title:** Two plugins registering the same write tool name — second call returns Err(E-PLUGIN-012)

**Preconditions:**

- S-PLUGIN-PREREQ-E is merged to `develop`
- `register_write_tool` API exists with signature `-> Result<(), SpecEngineError>`
- A test fixture prepares two `WriteToolInvalidationMap` entries sharing the same `tool_name` value (e.g., `"write_custom_sensor_record"`) but from different notional plugins (`plugin_name: "plugin_a"` for the first entry, `plugin_name: "plugin_b"` for the second, representing the two conflicting plugins — exercises E-PLUGIN-012 `{plugin}` and `{conflicting_plugin}` placeholders per ADR-026 D7 v1.10)

**Steps:**

1. Call `register_write_tool(entry_1)` where `entry_1.tool_name = "write_custom_sensor_record"` and `entry_1.plugin_name = "plugin_a"` — first registration
2. Assert the first call returns `.is_ok()`
3. Call `register_write_tool(entry_2)` where `entry_2.tool_name = "write_custom_sensor_record"` (same name) and `entry_2.plugin_name = "plugin_b"` — conflicting plugin
4. Assert the second call returns `.is_err()`
5. Assert the error variant is `SpecEngineError::DuplicateWriteToolRegistration("write_custom_sensor_record")` (E-PLUGIN-012)
6. Confirm the invalidation map still contains only the first entry (second was not written)

**Expected Outcome:**

- First registration: `Ok(())`
- Second registration: `Err(SpecEngineError::DuplicateWriteToolRegistration("write_custom_sensor_record"))` (E-PLUGIN-012)
- Invalidation map has one entry, not two
- No last-writer-wins behavior (ADR-026 D7 explicitly forbids it)

**Repos Tested:** prism-query (invalidation.rs register_write_tool path)

**BC Anchor:** BC-2.16.012 EC-016-012-004

**VP Traced:** VP-156 (Case 2 — duplicate name returns Err(DuplicateWriteToolRegistration))

---

## HS-PREREQ-E-003-05: Post-Boot Write Tool Registration — Rejected with E-PLUGIN-020

**Title:** `register_write_tool` called after step 8 (query engine init) returns Err(E-PLUGIN-020) and emits WARN log

**Preconditions:**

- S-PLUGIN-PREREQ-E is merged to `develop`
- `register_write_tool` API exists with signature `-> Result<(), SpecEngineError>`
- The public `prism_query::invalidation::mark_query_phase_started()` function has been invoked (as boot.rs will call it at step 8 start — as the first act of step 8, before QueryEngine construction proceeds, per ADR-026 D7), simulating post-step-8-start context. Per AC-9 (story v1.25), a direct `QUERY_PHASE_STARTED.store(true, ...)` in the test body is explicitly forbidden — the test must verify the production call site works via the public API.
- A `WriteToolInvalidationMap` entry is prepared for registration

**Steps:**

1. Invoke the public `prism_query::invalidation::mark_query_phase_started()` function (as boot.rs will call it). Per AC-9 (story v1.25), a direct `QUERY_PHASE_STARTED.store(true, ...)` in the test body is explicitly forbidden — the test must verify the production call site works via the public API. This step simulates post-step-8-start context (boot.rs step-8 first statement has executed).
2. Call `register_write_tool(entry)` after the flag is set
3. Assert the call returns `.is_err()`
4. Assert the error variant is `SpecEngineError::WriteToolRegistrationAfterBoot` (E-PLUGIN-020)
5. Confirm a `WARN`-level tracing event `write_tool_registration_after_boot` was emitted (check tracing subscriber capture or log output)
6. Confirm the invalidation map does NOT contain the entry (no write occurred)

**Expected Outcome:**

- `Err(SpecEngineError::WriteToolRegistrationAfterBoot)` (E-PLUGIN-020) returned
- WARN tracing event `write_tool_registration_after_boot` emitted
- Invalidation map unchanged (entry rejected, not written)
- Boot-step 7.5 is the only valid registration window (ADR-026 D7)

**Repos Tested:** prism-query (invalidation.rs register_write_tool + AtomicBool query-phase gate)

**BC Anchor:** BC-2.16.012 EC-016-012-005

**VP Traced:** VP-156 (related — register_write_tool contract surface per ADR-026 D7 v1.10)

---

**Note on VP-155 and HS-PREREQ-E-003:** VP-155 (CustomAdapter Absent from prism-spec-engine Public API — compile-fail perimeter) is a compile-time property, not a runtime scenario. HS-PREREQ-E-003 covers runtime dispatch behavior and WriteToolInvalidationMap extensibility. VP-155's runtime-equivalent coverage (confirming CustomAdapter types produce E0432 at compile time) is covered in HS-PREREQ-E-002-05. No additional VP-155 sub-scenario is needed in HS-PREREQ-E-003 — the compile-fail check is a CI gate, not a holdout runtime scenario.

---

## Validation Evidence Required

When this holdout scenario is evaluated, the evaluator must produce:

1. `SensorSpec` comparison output for all four initial sensors (byte-identical assertion result)
2. `grep -rn '"crowdstrike"\|"cyberint"...'` output from `spec_parser.rs` (must be empty in dispatch contexts)
3. `list_sensor_specs` tool response showing `hypothetical_sensor` with `status: loaded`
4. Test log for `register_write_tool` extensibility (invalidation fires on write after registration)

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.7 | FB46 | 2026-05-16 | product-owner | F-LP58-HIGH-002 closure: HS-003-05 Step 1 + Preconditions language canonicalized to require public-API `mark_query_phase_started()` invocation per AC-9 third-test gate (FB45 hardening); direct AtomicBool .store() explicitly forbidden. |
| 1.6 | FB37 | 2026-05-16 | product-owner | F-LP47-HIGH-001 HS-003-05 Preconditions AtomicBool set-time corrected from "set when query engine init completes at step 8" to canonical "set at step 8 start — as the first act of step 8, before QueryEngine construction proceeds, per ADR-026 D7"; simulating context updated from "post-boot context" to "post-step-8-start context". HS-003-05 Step 1 "simulating step 8 completion" corrected to "simulating post-step-8-start context (mark_query_phase_started() called)" per architect adjudication §1 note. Sibling-sweep with BC-2.16.012 EC-016-012-005 v1.16 + BC-2.16.002 row 33 v1.21. |
| 1.5 | prereq-e-fix-burst-12 | 2026-05-16 | product-owner | F-LP13-HIGH-003 Option A propagation — HS-003-03 fixture: `WriteToolInvalidationMap` entry gains `plugin_name: "plugin_a"` field. HS-003-04 fixture: entry_1 gains `plugin_name: "plugin_a"`, entry_2 gains `plugin_name: "plugin_b"` (exercises E-PLUGIN-012 `{plugin}`/`{conflicting_plugin}` placeholders per ADR-026 D7 v1.10); preconditions updated to name the two plugin values. HS-003-05 VP Traced footer: pin advanced ADR-026 D7 v1.9 → v1.10 per architect D-603. |
| 1.4 | prereq-e-fix-burst-10 | 2026-05-16 | product-owner | F-LP11-MED-001 — VP-156 bidirectional traceability symmetry restored: frontmatter `verification_properties: [VP-156]` added; HS-003-04 footer `**VP Traced:** VP-156 (Case 2 — duplicate name returns Err(DuplicateWriteToolRegistration))` added; HS-003-05 footer `**VP Traced:** VP-156 (related — register_write_tool contract surface per ADR-026 D7 v1.9)` added. RECURRING class — VP-154 closed FB1 (F-LP1-CRIT-001), VP-155 closed FB6 (F-LP6-HIGH-001), VP-156 third instance missed by passes 1-10; surfaced by pass-11 fresh-context. Sibling-class with HS-001/002 frontmatter+footer convention. |
| 1.3 | prereq-e-fix-burst-3 | 2026-05-15 | product-owner | F-LP3-LOW-001 closure: Added HS-PREREQ-E-003-04 (EC-016-012-004 duplicate-name: two plugins register same tool_name → second returns E-PLUGIN-012) and HS-PREREQ-E-003-05 (EC-016-012-005 after-boot: register_write_tool called after step 8 → returns E-PLUGIN-020 + WARN log). Both sub-scenarios include scenario description, preconditions, steps, expected outcome, validation evidence reference, and BC anchor. |
| 1.2 | prereq-e-fix-burst-2 | 2026-05-15 | product-owner | F-LP2-HIGH-002 closure (PO perimeter — 2 sites): Scenario Description body line ("closing TD-A-003") and HS-PREREQ-E-003-03 heading ("TD-A-003 Closure") both canonicalized to TD-S-PLUGIN-PREREQ-A-003. Changelog entry for v1.0 retains original TD-A-003 text as historical record (TD-VSDD-091 anti-volatile-pin; changelog is append-only). |
| 1.1 | S-PLUGIN-PREREQ-E-reconciliation | 2026-05-15 | product-owner | Q4 note: VP-155 is a compile-time property not a runtime scenario — confirmed HS-PREREQ-E-003 correctly does not cover it (VP-155 coverage added to HS-PREREQ-E-002-05 instead). Added VP-155 non-coverage rationale note in body. |
| 1.0 | S-PLUGIN-PREREQ-E-authoring | 2026-05-15 | product-owner | Initial draft. Three sub-scenarios: known-good SensorSpec parity, known-problematic novel sensor open-parse, and WriteToolInvalidationMap runtime extensibility (TD-A-003 closure). |
