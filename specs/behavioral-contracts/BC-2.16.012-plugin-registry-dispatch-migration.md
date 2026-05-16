---
document_type: behavioral-contract
level: L3
version: "1.5"
status: draft
producer: product-owner
timestamp: 2026-05-15T00:00:00
phase: 1a
origin: greenfield
subsystem: "SS-16"
capability: "CAP-029"
lifecycle_status: draft
introduced: "2026-05-15"
modified: "2026-05-15"
deprecated: ~
deprecated_by: ~
replacement: ~
retired: ~
removed: ~
removal_reason: ~
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
  - ".factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md"
  - ".factory/specs/behavioral-contracts/BC-2.16.001-sensor-spec-file-loading.md"
input-hash: null
traces_to: ["CAP-029"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.16.012: PluginRegistry Dispatch in spec_parser.rs — Hardcoded Sensor Names Replaced with Registry Lookup

## Description

`crates/prism-spec-engine/src/spec_parser.rs` contains call sites that dispatch based on
hardcoded sensor name strings or `SensorType`-equivalent patterns to resolve sensor-specific
behavior (auth type, fetch pipeline, table schema). After PREREQ-A removed the `SensorType`
closed enum, residual dispatch may still use string literals like `"crowdstrike"` or
`"cyberint"` in match arms. This BC defines the contract: those dispatch sites are migrated
to `PluginRegistry` lookup so that spec-parser behavior is open-coded — an unknown sensor
name resolves the same way as a known one, via the registry, without any hardcoded branch
per sensor. The behavioral output (parsed `SensorSpec`) is identical for the four initial
sensors; the change is structural (open dispatch replaces closed match).

## Preconditions

- `S-PLUGIN-PREREQ-D` has merged: `PluginRegistry` (or `AdapterRegistry`) is wired into
  the runtime and accessible at the call sites within spec_parser's execution context.
- `S-PLUGIN-PREREQ-A` has merged: `SensorId(Arc<str>)` replaces `SensorType`; any
  `match sensor_type { SensorType::X => ... }` dispatch arms at these sites have already
  been converted to `match sensor_id.as_ref() { "x" => ... }` string-match arms.
- The spec_parser call sites are confirmed to be ZERO `CustomAdapter`/`CustomAdapterRegistry`
  references (verified by grep in ADR-023 §Architectural Constraints (C5 bullet) F-CRIT-NEW-001-PASS2-RESIDUAL): no
  `CustomAdapter` removal is needed from `spec_parser.rs` itself — only registry dispatch
  migration applies here.
- The `PluginRegistry` (or `AdapterRegistry`) provides a `lookup(sensor_id: &SensorId)`
  method that returns an `Option<Arc<dyn SensorAdapter>>` or equivalent plugin descriptor,
  enabling spec-parser logic to branch on registry presence rather than hardcoded name.

## Postconditions

- All dispatch call sites in `crates/prism-spec-engine/src/spec_parser.rs` that previously
  used hardcoded sensor names (string literal `match` arms for `"crowdstrike"`, `"cyberint"`,
  `"claroty"`, `"armis"`) to select sensor-specific parsing behavior are replaced with
  `PluginRegistry` lookup or a uniform code path that applies to any sensor string.
- The behavioral output for the four initial sensors is byte-identical to the pre-migration
  behavior: the same `SensorSpec`, `TableSpec`, `ColumnSpec`, and auth-type resolution
  results are produced when parsing the four built-in sensor TOML specs.
- An unknown sensor name (e.g., `"hypothetical_sensor"`) does NOT cause a parse-time error.
  If the sensor is not in the registry, the spec-parser falls through to the generic TOML
  parsing path and produces a `SensorSpec` with `registry_lookup_failed: false` (or
  equivalent flag); the tables are registered normally via DataFusion. The unknown sensor
  path was already open-coded via `SensorId`; this BC closes any residual hardcoded branch.
- `grep -rn '"crowdstrike"\|"cyberint"\|"claroty"\|"armis"' crates/prism-spec-engine/src/spec_parser.rs`
  returns ZERO matches in any dispatch-context code. Sensor name strings may still appear in
  comments, doc strings, or test fixture values — only production dispatch match arms are
  prohibited.
- The `WriteToolInvalidationMap` in `crates/prism-query/src/invalidation.rs` is migrated from
  a static `LazyLock<Vec<...>>` to a `RwLock<Vec<WriteToolInvalidationMap>>` (or equivalent
  runtime-extensible container) with a `register_write_tool(entry)` API, closing TD-S-PLUGIN-PREREQ-A-003.
  This enables plugin-registered write tools to participate in cache invalidation at runtime
  rather than requiring a compile-time list update. The registration API is called by
  `PluginRuntime` when a plugin declares write-tool capabilities in its manifest.

## Invariants

- **INV-SPEC-PARSER-OPEN-001:** After this story merges, `grep -rn "match.*sensor_id.*\"crowdstrike\"\|match.*sensor_id.*\"cyberint\"" crates/prism-spec-engine/src/spec_parser.rs` returns ZERO matches. The dispatch is open (registry or generic) not closed (hardcoded match).
- **INV-SPEC-PARSER-OPEN-002:** Behavioral equivalence for the four initial sensors: parsing `crowdstrike.sensor.toml`, `cyberint.sensor.toml`, `claroty.sensor.toml`, and `armis.sensor.toml` produces `SensorSpec` structs byte-identical to those produced by the pre-migration code. Verified by the behavioral-equivalence integration test (TV-BC-2.16.012-003).
- **INV-SPEC-PARSER-OPEN-003:** An unrecognized sensor name (not in the registry and not one of the four built-in names) is parsed generically: the spec file is parsed into a `SensorSpec` with whatever auth_type, tables, and columns the TOML declares. No hardcoded name-check gate rejects it.
- **INV-INVALIDATION-EXT-001 (TD-S-PLUGIN-PREREQ-A-003 closure):** After this story merges, `crates/prism-query/src/invalidation.rs` `WriteToolInvalidationMap` container is runtime-extensible: `register_write_tool(entry: WriteToolInvalidationMap)` is callable after startup. Plugins that register write tools in their manifest can invoke this API during boot-time plugin loading.

## Error Cases

| Error | Condition | Behavior |
|-------|-----------|----------|
| (none new) | PluginRegistry lookup for an unknown sensor returns `None` | Generic parsing path is used; no error. The spec file is parsed on its own declared fields. |
| `E-SPEC-001` | The sensor TOML spec file itself has a syntax error | Unchanged behavior; parse error surfaced as before, independent of registry dispatch |
| `E-SPEC-005` | The sensor TOML spec declares an unsupported `auth_type` string | Unchanged behavior; validation rejects invalid auth_type values at spec-load time |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-016-012-001 | `spec_parser.rs` has a match arm `"crowdstrike" => <crowdstrike-specific-logic>` that is actually behavioral, not just naming | That logic is generalized into the `SensorSpec` field model (e.g., added as a TOML field) or moved into the CrowdStrike sensor TOML spec. No sensor-specific Rust branch survives. |
| EC-016-012-002 | A new sensor TOML spec is added post-PREREQ-E (e.g., `hypothetical.sensor.toml`) | Parsed generically; tables registered; no hardcoded name check blocks it |
| EC-016-012-003 | PluginRegistry returns `Some(plugin)` for a sensor that also has a TOML spec | The registry-provided plugin adapter takes precedence for fetch behavior; the TOML spec provides table schema and column definitions (ADR-023 Rule 1 TOML as declarative baseline) |
| EC-016-012-004 | `register_write_tool` called by two plugins registering the same write tool name | Second registration is rejected with `Err(SpecEngineError::DuplicateWriteToolRegistration(tool_name))` per ADR-026 D7. Last-writer-wins is explicitly forbidden — see ADR-026 D7 rationale for production-grade default enforcement. |
| EC-016-012-005 | `register_write_tool` called from a non-boot context (after server is live, step 8+) | Rejected with `Err(SpecEngineError::WriteToolRegistrationAfterBoot)` per ADR-026 D7. An `AtomicBool` query-phase flag (set when query engine init completes at step 8) gates the write. A WARN-level tracing event is emitted. Plugin lifecycle must register write tools during boot step 7.5 only. Error code: E-PLUGIN-020. |

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.16.012-001 | `grep -rn '"crowdstrike"\|"cyberint"\|"claroty"\|"armis"' crates/prism-spec-engine/src/spec_parser.rs` after merge (in dispatch code) | Zero matches in match-arm dispatch contexts |
| TV-BC-2.16.012-002 | Parse a new `hypothetical.sensor.toml` with a novel sensor name not in any hardcoded list | Produces a valid `SensorSpec`; no parse-time error; tables registered in DataFusion catalog |
| TV-BC-2.16.012-003 | Parse `crowdstrike.sensor.toml` before and after migration; compare `SensorSpec` output | Byte-identical `SensorSpec` structs (behavioral equivalence test) |
| TV-BC-2.16.012-004 | `register_write_tool(WriteToolInvalidationMap { sensor_id: SensorId::from("custom_sensor"), tool_name: "write_custom_sensor_record", ... })` | Entry is present in the invalidation map on next `RwLock` read-guard acquisition; cache invalidation fires on a write to `custom_sensor` |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| VP-156 | WriteToolInvalidationMap registration uniqueness (proptest P1). Verifies EC-016-012-004 resolved behavior (error-on-duplicate, per ADR-026 D7 v1.6). Visibility guarantee is structural (RwLock contract + ADR-022 boot ordering) not proptest-verified — see VP-156 §Property Statement. Authored in prereq-e-fix-burst-1 (F-LP1-MED-003). Behavioral equivalence and open-dispatch invariants remain verified by integration test (TV-003) and grep gate (TV-001). |

## Related BCs

- BC-2.16.001 (Sensor Spec File Loading): parent contract; this BC augments the dispatch mechanism used when loading specs.
- BC-2.16.011 (CustomAdapter Rust Trait Retirement): sibling contract; the PluginRegistry becomes the sole extensibility mechanism after CustomAdapter is removed.
- BC-2.01.016 (SensorAuth Open Trait): sibling contract; the PluginRegistry dispatch enabled by open `SensorAuth` is the mechanism this BC exercises in spec_parser.
- BC-2.01.013 (DataSource Trait): grandparent contract; spec-parser migration feeds into the open DataSource dispatch chain.

## Architecture Anchors

- `crates/prism-spec-engine/src/spec_parser.rs` — primary migration target
- `crates/prism-query/src/invalidation.rs` — `WriteToolInvalidationMap` runtime-extensibility target (TD-S-PLUGIN-PREREQ-A-003)
- ADR-023 §Architectural Constraints (C5 bullet, Rule 5) scope note — confirms `spec_parser.rs` has zero `CustomAdapter` references; only open-dispatch migration applies

## Story Anchor

S-PLUGIN-PREREQ-E

## VP Anchors

- VP-156 (WriteToolInvalidationMap registration uniqueness: duplicate tool_name returns Err(DuplicateWriteToolRegistration); first registration persists unchanged; proptest P1; anchor: S-PLUGIN-PREREQ-E; authored in prereq-e-fix-burst-1 per F-LP1-MED-003 resolution; happens-before visibility is structural per RwLock contract + ADR-022 boot ordering, not proptest-verified)

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-029 |
| Capability Anchor Justification | CAP-029 ("Config-Driven Sensor Adapters") per capabilities.md §CAP-029. The spec_parser.rs dispatch migration directly implements the "runtime-interpreted" principle of CAP-029 — sensor name resolution must be open (registry-keyed) not closed (hardcoded match) so that dropping a new TOML spec file suffices to add a sensor without code changes. |
| L2 Invariants | DI-030 (every spec file validated at load and reload time — the open dispatch path must not bypass validation) |
| Related BCs | BC-2.16.001 (spec loading), BC-2.16.011 (CustomAdapter retirement), BC-2.01.016 (SensorAuth open trait) |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.5 | prereq-e-fix-burst-4 | 2026-05-15 | architect | F-LP4-HIGH-004 POL-23 sibling sweep: §Verification Properties VP-156 row — "ADR-026 D7 v1.5" pin updated to "ADR-026 D7 v1.6" (ADR-026 bumped in fix-burst-4 to add VP-156 to §VP Anchors). BC-2.16.012 v1.4→v1.5. |
| 1.4 | prereq-e-fix-burst-3 | 2026-05-15 | architect | F-LP3-HIGH-001 sibling-sweep (architect domain): §Verification Properties VP-156 row updated — "uniqueness + happens-before" → "uniqueness only"; stale "ADR-026 D7 v1.2" pin updated to v1.5. §VP Anchors VP-156 bullet updated — "uniqueness + happens-before" framing removed; structural visibility guarantee noted (RwLock contract + ADR-022 boot ordering). Aligns BC-2.16.012 with VP-156 v0.2 body (fix-burst-2) and ADR-026 current v1.5. |
| 1.3 | prereq-e-fix-burst-2 | 2026-05-15 | product-owner | F-LP2-HIGH-001 paper-fix correction: EC-016-012-004 body rewritten to specify `Err(SpecEngineError::DuplicateWriteToolRegistration(tool_name))` per ADR-026 D7. Prior v1.2 changelog claimed F-LP1-MED-002 closure but EC-016-012-004 body still read "Implementer chooses; last-writer-wins, OR an error is returned" — directly contradicting ADR-026 D7 v1.2 mandate. Body now reflects the production-grade default (error-on-duplicate; last-writer-wins explicitly forbidden). |
| 1.2 | prereq-e-fix-burst-1 | 2026-05-15 | architect | F-LP1-MED-003 resolution: §Verification Properties table updated from "(none in this story)" to VP-156; §VP Anchors updated to list VP-156. EC-016-012-004 duplicate-registration behavior now resolved to error-on-duplicate per ADR-026 D7 v1.2 (previously "implementer chooses"). BC-2.16.012 §Verification Properties coverage gap closed. |
| 1.2 | fix-burst-1 state-manager catch | 2026-05-15 | state-manager | (state-manager catch in fix-burst-1) F-LP1-HIGH-004 POL-20: introduced field canonicalized to ISO date 2026-05-15. Prior value `S-PLUGIN-PREREQ-E` was story-ID format; POL-20 requires `YYYY-MM-DD` for artifacts created outside greenfield cycles. |
| 1.1 | S-PLUGIN-PREREQ-E-fix-burst-1 | 2026-05-15 | product-owner | F-LP1-HIGH-003 closure: Three §C5 phantom-heading citations corrected per POL-21. F-LP1-MED-004 closure: Four TD-A-003 alias occurrences replaced with canonical TD-S-PLUGIN-PREREQ-A-003 (INV-INVALIDATION-EXT-001 label, §Postconditions, §Architecture Anchors, changelog). |
| 1.0 | S-PLUGIN-PREREQ-E-authoring | 2026-05-15 | product-owner | Initial draft. Operationalizes ADR-023 §Architectural Constraints (C5 bullet) spec_parser.rs call-site migration + closes TD-S-PLUGIN-PREREQ-A-003 WriteToolInvalidationMap extensibility. |
