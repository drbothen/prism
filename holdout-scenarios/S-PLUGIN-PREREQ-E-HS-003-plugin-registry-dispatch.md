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
version: "1.2"
status: draft
producer: product-owner
timestamp: 2026-05-15T00:00:00
phase: 4
inputs: []
input-hash: null
traces_to: "BC-2.16.012"
behavioral_contracts:
  - BC-2.16.012
  - BC-2.16.001
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

1. Construct a `WriteToolInvalidationMap` entry: `{ sensor_id: SensorId::from("custom_sensor"), tool_name: "write_custom_sensor_record", ... }`
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
| 1.2 | prereq-e-fix-burst-2 | 2026-05-15 | product-owner | F-LP2-HIGH-002 closure (PO perimeter — 2 sites): Scenario Description body line ("closing TD-A-003") and HS-PREREQ-E-003-03 heading ("TD-A-003 Closure") both canonicalized to TD-S-PLUGIN-PREREQ-A-003. Changelog entry for v1.0 retains original TD-A-003 text as historical record (TD-VSDD-091 anti-volatile-pin; changelog is append-only). |
| 1.1 | S-PLUGIN-PREREQ-E-reconciliation | 2026-05-15 | product-owner | Q4 note: VP-155 is a compile-time property not a runtime scenario — confirmed HS-PREREQ-E-003 correctly does not cover it (VP-155 coverage added to HS-PREREQ-E-002-05 instead). Added VP-155 non-coverage rationale note in body. |
| 1.0 | S-PLUGIN-PREREQ-E-authoring | 2026-05-15 | product-owner | Initial draft. Three sub-scenarios: known-good SensorSpec parity, known-problematic novel sensor open-parse, and WriteToolInvalidationMap runtime extensibility (TD-A-003 closure). |
