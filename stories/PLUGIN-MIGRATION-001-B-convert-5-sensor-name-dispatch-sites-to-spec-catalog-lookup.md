---
document_type: story
story_id: PLUGIN-MIGRATION-001-B
title: "prism-query: Convert 3 Sensor-Name Dispatch Sites to Spec-Catalog Lookup"
wave: 1
epic_id: PLUGIN-MIGRATION-001
priority: P0
status: ready
version: "v1.0"
level: "L4"
producer: story-writer
timestamp: "2026-05-26T00:00:00Z"
modified: "2026-05-26"
tdd_mode: strict
subsystems: [SS-07, SS-16]
# Subsystem anchor justifications:
#   SS-07 (Adapter Pagination & Response Cache; prism-query) owns all three dispatch sites
#   being converted: explain.rs cost heuristics, write_pipeline.rs compile-gate dispatch,
#   and invalidation.rs WRITE_TOOL_INVALIDATION_MAP — all reside in prism-query crate.
#   SS-16 (Spec Engine, prism-spec-engine) owns the WriteEndpointRegistry and
#   SensorSpec catalog that replace the hardcoded lookup, per BC-2.16.012 INV-SPEC-PARSER-OPEN-001
#   and INV-INVALIDATION-EXT-001.
crates_touched: [prism-query, prism-bin]
target_module: prism-query
capabilities: [CAP-029]
behavioral_contracts:
  - BC-2.16.012  # INV-SPEC-PARSER-OPEN-001: no hardcoded sensor-name match arms in dispatch
                 #   code; INV-INVALIDATION-EXT-001: WRITE_TOOL_INVALIDATION_MAP migrated to
                 #   DYNAMIC_WRITE_TOOLS via register_write_tool() at boot. Both invariants
                 #   targeted by this story's three conversion sites.
  - BC-2.01.013  # DataSource Trait / spec-driven adapter pattern: removing hardcoded sensor
                 #   name dispatch from prism-query advances the spec-driven open dispatch
                 #   mandate for the full stack (sensors → spec-engine → query).
verification_properties:
  - VP-156  # WriteToolInvalidationMap registration uniqueness (proptest P1). Verifies
            # that duplicate tool_name registration returns Err(DuplicateWriteToolRegistration).
            # Directly exercises the DYNAMIC_WRITE_TOOLS path that replaces WRITE_TOOL_INVALIDATION_MAP.
depends_on:
  - S-PLUGIN-PREREQ-A   # SensorId newtype — dispatch sites use SensorId::as_ref(); required
  - S-PLUGIN-PREREQ-C   # TOML grammar — WriteEndpointRegistry populated from sensor TOML specs
  - PLUGIN-MIGRATION-001-A  # Deletes prism-sensors auth modules; prism-query must not hold
                             # imports from deleted modules. 001-A merged PR #156 develop@948a709f.
blocks:
  - PLUGIN-MIGRATION-001-F  # test: Rewrite 10+ sensor-named test files; must see clean prism-query
                             # production code before test rewrite can proceed
# Dependency anchor justifications:
#   depends_on PLUGIN-MIGRATION-001-A: because 001-A rewrites init_registry_for_org to use
#     spec-catalog dispatch; 001-B's explain.rs and write_pipeline.rs sites must not retain
#     imports that reference symbols deleted by 001-A. Also, 001-A's spec-catalog dispatch
#     in prism-sensors proves the pattern that 001-B replicates in prism-query.
#   blocks PLUGIN-MIGRATION-001-F: because the test rewrite story converts sensor-named test
#     fixtures to spec-catalog patterns; the production conversion must land first so tests
#     can be anchored to the new code paths.
points: 3
# Points justification:
#   SITE-1 (explain.rs latency match): ~0.5 day — remove 4 match arms, replace with uniform
#     default `_ => 300` (correct open-dispatch behavior; 300ms is already the wildcard value).
#     Red Gate test: confirm unknown sensor gets 300ms. Simple change.
#   SITE-2 (write_pipeline.rs compile-gate match): ~1 day — replace 4 match arms + hardcoded
#     function imports with WriteEndpointRegistry::get() presence check. Red Gate test: unknown
#     sensor → CompileFeatureGate::Absent; known-but-unregistered sensor → Absent; registered
#     sensor → Present (mocked registry).
#   SITE-3 (invalidation.rs WRITE_TOOL_INVALIDATION_MAP migration): ~0.5 day — populate
#     DYNAMIC_WRITE_TOOLS at boot from WriteEndpointRegistry instead of LazyLock; remove or
#     hollow WRITE_TOOL_INVALIDATION_MAP static; add boot-time registration helper.
#   Total: 3 points (~1.5-2 days). Scope is conversion only; no new write tool logic added.
estimated_days: 2
risk: LOW
# Risk justification: All three conversion sites have a correct open-dispatch fallback (wildcard
# arm / Absent / empty) that already handles unknown sensors. The changes are structural
# simplifications, not behavioral changes for the four built-in sensors. Existing tests
# exercise both paths. The only new failure mode is if boot-time DYNAMIC_WRITE_TOOLS
# registration is incomplete, leaving write tool cache invalidation silently empty — mitigated
# by a Red Gate test asserting the boot-time registration path.
acceptance_criteria_count: 5
red_gate_tests: 3
estimated_passes: "2-3 LOCAL adversary passes"
holdout_scenarios: []
assumption_validations: []
risk_mitigations:
  - "SITE-3 boot-time gap: if DYNAMIC_WRITE_TOOLS is not populated before queries run, write
    tool cache invalidation silently skips. Mitigated by AC-003's Red Gate test asserting
    that register_write_tool() populates DYNAMIC_WRITE_TOOLS for each built-in write tool."
inputs:
  - "crates/prism-query/src/explain.rs"
  - "crates/prism-query/src/write_pipeline.rs"
  - "crates/prism-query/src/invalidation.rs"
  - "crates/prism-security/src/feature_flag.rs"
  - "crates/prism-spec-engine/src/write_endpoint.rs"
  - ".factory/specs/behavioral-contracts/BC-2.16.012-plugin-registry-dispatch-migration.md"
  - ".factory/specs/behavioral-contracts/BC-2.01.013-datasource-trait-adapter-pattern.md"
  - ".factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md"
  - ".factory/specs/architecture/decisions/ADR-027-custom-adapter-deprecation-removal.md"
  - ".factory/stories/PLUGIN-MIGRATION-001-A-delete-4-named-auth-modules-and-replace-init-registry-for-org.md"
input-hash: null
traces_to: []
cycle: "v1.0.0-greenfield"
phase: 3
---

# PLUGIN-MIGRATION-001-B: prism-query — Convert 3 Sensor-Name Dispatch Sites to Spec-Catalog Lookup

**Story ID:** PLUGIN-MIGRATION-001-B
**Status:** ready
**Version:** v1.0
**Wave:** 1 (all dependencies merged; PLUGIN-MIGRATION-001-A merged PR #156 develop@948a709f)

---

## §Origin

Registered in STORY-INDEX at D-334 (2026-05-10) as Wave 1 of the PLUGIN-MIGRATION saga,
dependent on PLUGIN-MIGRATION-001-A. PLUGIN-MIGRATION-001-A merged 2026-05-24 (PR #156,
develop@948a709f). All dependencies are now satisfied.

The STORY-INDEX entry originally said "5 dispatch sites" — a pre-code-read estimate. Actual
grep of `crates/prism-query/src/` confirms **3 production dispatch sites** with hardcoded
sensor name strings. The title in this story uses the corrected count. The 3-site count is
authoritative for implementation.

---

## Story-Level Goal

At merge, `crates/prism-query/` contains zero hardcoded sensor name `match` arms in dispatch
contexts. Three production sites are converted:

1. `explain.rs:1053-1059` — 4-arm latency heuristic match replaced with uniform open default.
2. `write_pipeline.rs:324-331` — 4-arm compile-gate match replaced with `WriteEndpointRegistry`
   presence check.
3. `invalidation.rs:252-310` — `WRITE_TOOL_INVALIDATION_MAP` static `LazyLock` with 8
   hardcoded entries replaced with a boot-time `register_write_tool()` call sequence that
   populates `DYNAMIC_WRITE_TOOLS` from the `WriteEndpointRegistry`.

The four built-in sensors continue to work identically after conversion; open dispatch
means an unknown or future plugin-registered sensor falls through cleanly without error.

---

## Narrative

As the Prism platform, I want all sensor-name dispatch match arms in `prism-query` converted
to spec-catalog lookup or open-dispatch defaults, so that adding a new sensor via TOML spec
automatically extends write gate checks, latency cost estimates, and cache invalidation
mappings without requiring a code change in prism-query.

---

## Functional Summary

1. **SITE-1 — `explain.rs:1053-1059` latency heuristic**

   Remove the 4-arm match:
   ```rust
   // BEFORE (hardcoded dispatch — INV-SPEC-PARSER-OPEN-001 violation):
   let latency_ms = match src.sensor_id.as_ref() {
       "crowdstrike" => 250,
       "cyberint" => 400,
       "claroty" => 350,
       "armis" => 300,
       _ => 300,
   };

   // AFTER (open dispatch — uniform default):
   let latency_ms = 300_u64;
   // TODO-S-3.10: replace with SensorSpec.latency_hint_ms when that field is added
   // to the SensorSpec schema (S-3.10 cost estimation story). Until then, 300ms is
   // the correct open-dispatch default (matches the prior wildcard arm exactly).
   ```

   The 4 hardcoded per-sensor values are heuristics only (the existing TODO comment on
   line 1050 of `explain.rs` acknowledges this and defers to a future metrics-backed
   implementation). The correct open-dispatch behavior is the `_ => 300` wildcard —
   which was already the fallback for unknown sensors. Collapsing to the uniform default
   satisfies INV-SPEC-PARSER-OPEN-001 without changing behavior for any sensor.

   Remove the now-unused imports of `crowdstrike_write_gate`, `cyberint_write_gate`, etc.
   from `explain.rs` (if present — implementer must verify). Only the latency match is
   in `explain.rs`; the feature-gate imports are in `write_pipeline.rs`.

2. **SITE-2 — `write_pipeline.rs:324-331` write gate dispatch**

   Replace the 4-arm compile-gate match with a `WriteEndpointRegistry` presence check:
   ```rust
   // BEFORE (hardcoded dispatch — INV-SPEC-PARSER-OPEN-001 violation):
   let compile_gate: CompileFeatureGate = match plan.sensor.as_str() {
       "crowdstrike" => crowdstrike_write_gate().into(),
       "cyberint" => cyberint_write_gate().into(),
       "claroty" => claroty_write_gate().into(),
       "armis" => armis_write_gate().into(),
       _ => CompileFeatureGate::Absent,
   };

   // AFTER (open dispatch — registry-driven):
   let compile_gate: CompileFeatureGate =
       if self.endpoint_registry.get(&plan.sensor, &plan.verb).is_some() {
           CompileFeatureGate::Present
       } else {
           CompileFeatureGate::Absent
       };
   ```

   **Rationale for this replacement:** `CompileFeatureGate::Present` means "this binary was
   compiled with write support for this sensor." The existing `WriteEndpointRegistry` already
   represents exactly this fact for the post-PLUGIN-MIGRATION world: a sensor has write
   capability if and only if its TOML spec declares `[[write_endpoints]]` sections and those
   specs are loaded at boot. The `{sensor}-write` Cargo feature flags are an _additional_
   safety gate layered on top; once all sensors are spec-driven, the registry presence IS the
   authoritative capability signal. The `_write` Cargo features remain active (they gate the
   prism-security and prism-sensors write codepaths), but the dispatch in prism-query's
   `write_pipeline.rs` should be driven by the spec registry, not per-sensor function calls.

   **Remove hardcoded function imports** from `write_pipeline.rs`:
   ```rust
   // REMOVE these imports:
   use prism_security::feature_flag::{
       armis_write_gate, claroty_write_gate, crowdstrike_write_gate, cyberint_write_gate,
       FeatureFlagEvaluator,
   };
   // KEEP only:
   use prism_security::feature_flag::FeatureFlagEvaluator;
   ```

   Verify `FeatureFlagEvaluator` is still needed after the import reduction; if not, remove
   it too. `CompileFeatureGate` stays (it's used in `phase2_safety_check`).

3. **SITE-3 — `invalidation.rs:252-310` WRITE_TOOL_INVALIDATION_MAP migration**

   The `WRITE_TOOL_INVALIDATION_MAP` static `LazyLock<Vec<WriteToolInvalidationMap>>` contains
   8 hardcoded entries naming 4 sensors. BC-2.16.012 INV-INVALIDATION-EXT-001 mandates that
   this map be replaced by boot-time `register_write_tool()` calls.

   **Step 1 — Add a boot-time registration helper:**
   ```rust
   /// Populate `DYNAMIC_WRITE_TOOLS` from the built-in static map.
   ///
   /// Called ONCE at boot, before `mark_query_phase_started()`. After this call,
   /// `WRITE_TOOL_INVALIDATION_MAP` is no longer consulted — all invalidation goes
   /// through `DYNAMIC_WRITE_TOOLS`.
   ///
   /// Idempotent if called on an already-populated registry (duplicate tool_name
   /// returns `Err(DuplicateWriteToolRegistration)` per BC-2.16.012 EC-016-012-004 —
   /// caller must ensure this is called exactly once per process).
   pub fn register_builtin_write_tools() -> Result<(), SpecEngineError> {
       for entry in WRITE_TOOL_INVALIDATION_MAP.iter() {
           register_write_tool(entry.clone())?;
       }
       Ok(())
   }
   ```

   **Step 2 — Mark `WRITE_TOOL_INVALIDATION_MAP` as the migration source:**

   Add a doc-comment to `WRITE_TOOL_INVALIDATION_MAP` marking it as transitional:
   ```rust
   /// Built-in write tool invalidation map for the four initial sensors.
   ///
   /// **Transitional:** This static list is the source for `register_builtin_write_tools()`
   /// which populates `DYNAMIC_WRITE_TOOLS` at boot. After PLUGIN-MIGRATION-001-F
   /// (test rewrite), this static will be emptied and `DYNAMIC_WRITE_TOOLS` will be
   /// the sole source. The static is retained in this story to avoid breaking the
   /// existing `invalidate_for_sensor` code path which currently iterates both.
   ```

   **Step 3 — Wire `register_builtin_write_tools()` in boot sequence:**

   The correct wiring site is `crates/prism-bin/src/boot.rs` step 7.5 (after plugin loading,
   before `mark_query_phase_started()` at step 8). Since `boot.rs` step 7.5 is currently a
   `todo!()` stub (S-WAVE5-PREP-01), the implementer must add a call to
   `prism_query::invalidation::register_builtin_write_tools()` in the step-7.5 block, or
   emit a scoped `todo!()` with the anchor `// PLUGIN-MIGRATION-001-B: register_builtin_write_tools()
   // must be called here at step 7.5 before mark_query_phase_started()`.

   Document the boot-wiring state in §Known Gaps if the step-7.5 stub prevents full wiring.

---

## Behavioral Contracts

| BC ID | Version | Title | Subsystem | Role in This Story |
|-------|---------|-------|-----------|-------------------|
| BC-2.16.012 | 1.33 | PluginRegistry Dispatch in spec_parser.rs | SS-16 | **Primary** — INV-SPEC-PARSER-OPEN-001 (no hardcoded match arms in dispatch code) is the direct target of SITE-1 and SITE-2. INV-INVALIDATION-EXT-001 (WRITE_TOOL_INVALIDATION_MAP → DYNAMIC_WRITE_TOOLS) is the target of SITE-3. Both invariants close upon this story's merge. |
| BC-2.01.013 | 1.7 | DataSource Trait — Spec-Driven Adapter Pattern | SS-01 | **Completion** — removing hardcoded sensor-name dispatch from prism-query advances the full-stack spec-driven open dispatch mandate; the query layer must not re-introduce closed dispatch after the sensor layer has been opened by 001-A. |

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec | ~7,000 |
| BC-2.16.012 v1.33 (primary BC, full read) | ~8,000 |
| BC-2.01.013 v1.7 (supporting BC, full read) | ~2,000 |
| ADR-023 v1.19 §Decision Rules 1-5 | ~4,000 |
| ADR-027 v1.9 §D5 (dispatch audit) | ~2,000 |
| `crates/prism-query/src/explain.rs` (partial: SITE-1 context ~100 lines) | ~3,000 |
| `crates/prism-query/src/write_pipeline.rs` (partial: SITE-2 context ~100 lines) | ~3,000 |
| `crates/prism-query/src/invalidation.rs` (partial: SITE-3 context ~150 lines) | ~4,000 |
| `crates/prism-security/src/feature_flag.rs` (partial: write gate functions ~80 lines) | ~2,000 |
| `crates/prism-spec-engine/src/write_endpoint.rs` (partial: registry API ~80 lines) | ~2,000 |
| PLUGIN-MIGRATION-001-A story (predecessor intelligence) | ~6,000 |
| **Total estimate** | **~43,000** |
| Agent context window (claude-sonnet-4-6) | ~200,000 |
| **% of context window** | **~22%** |

Within the 20–30% target. All source files are readable in targeted excerpts without context pressure.

---

## Acceptance Criteria

### AC-001: SITE-1 — `explain.rs` latency heuristic match removed (traces to BC-2.16.012 invariant INV-SPEC-PARSER-OPEN-001)

The 4-arm `match src.sensor_id.as_ref()` block at `crates/prism-query/src/explain.rs:1053-1059`
is removed. It is replaced with a uniform constant or variable assignment:

```rust
let latency_ms = 300_u64;
```

(Exact form — `300_u64`, `300u64`, or `let latency_ms: u64 = 300;` — is at implementer
discretion as long as the type matches `HashMap<String, u64>` insertion.)

**Verification:**

```bash
grep -n '"crowdstrike"\|"cyberint"\|"claroty"\|"armis"' crates/prism-query/src/explain.rs
```

Returns ZERO matches in non-comment lines. Sensor name strings may appear in doc comments
or test fixture values — only production `match` arms are prohibited.

A Red Gate test `test_BC_2_16_012_B_001_explain_unknown_sensor_latency_is_300` is added to
`crates/prism-query/src/tests/explain_tests.rs` (or an appropriate test file) asserting that
an `ExplainResult` for an unknown sensor returns `per_sensor_latency_ms` entry with value 300.

(traces to BC-2.16.012 invariant INV-SPEC-PARSER-OPEN-001 — no hardcoded sensor-name match arms
in dispatch contexts; unknown sensor handled uniformly)

---

### AC-002: SITE-2 — `write_pipeline.rs` compile-gate match replaced with registry dispatch (traces to BC-2.16.012 invariant INV-SPEC-PARSER-OPEN-001)

The 4-arm `match plan.sensor.as_str()` block at `crates/prism-query/src/write_pipeline.rs:324-331`
is replaced with a `WriteEndpointRegistry::get()` presence check as specified in §Functional
Summary SITE-2.

The following imports are removed from `write_pipeline.rs`:
```rust
// REMOVED:
armis_write_gate, claroty_write_gate, crowdstrike_write_gate, cyberint_write_gate,
```

If `FeatureFlagEvaluator` is no longer used after the import reduction, it is also removed.

**Verification:**

```bash
grep -n '"crowdstrike"\|"cyberint"\|"claroty"\|"armis"' crates/prism-query/src/write_pipeline.rs
```

Returns ZERO matches in non-comment, non-test-block production lines.

```bash
grep -n 'crowdstrike_write_gate\|cyberint_write_gate\|claroty_write_gate\|armis_write_gate' \
  crates/prism-query/src/write_pipeline.rs
```

Returns ZERO matches in production lines (test-only references in `#[cfg(test)]` blocks are
acceptable if they test the new registry-driven path via a mock registry).

A Red Gate test `test_BC_2_16_012_B_002_write_gate_absent_for_unregistered_sensor` is added
asserting that `WriteExecutor::execute()` for an unknown sensor name (not in the endpoint
registry) returns `CompileFeatureGate::Absent` semantics — i.e., the write is rejected with
the appropriate feature-gate error before reaching the HTTP dispatch.

(traces to BC-2.16.012 invariant INV-SPEC-PARSER-OPEN-001 — open dispatch; no hardcoded
sensor-name match arms; unknown sensor defaults to Absent write gate)

---

### AC-003: SITE-3 — `register_builtin_write_tools()` helper added and boot-wiring documented (traces to BC-2.16.012 invariant INV-INVALIDATION-EXT-001)

`crates/prism-query/src/invalidation.rs` exports `register_builtin_write_tools()` with the
signature and behavior specified in §Functional Summary SITE-3.

`WRITE_TOOL_INVALIDATION_MAP` doc-comment is updated to mark it as transitional per §Functional
Summary SITE-3.

A Red Gate test `test_BC_2_16_012_B_003_register_builtin_write_tools_populates_dynamic_registry`
is added asserting:
1. Before `register_builtin_write_tools()`, `DYNAMIC_WRITE_TOOLS` count is 0 (or baseline).
2. After `register_builtin_write_tools()`, `dynamic_write_tool_count()` equals 8 (the 8
   entries in `WRITE_TOOL_INVALIDATION_MAP`).
3. Each entry's `sensor_id` is one of the four built-in sensors.
4. A second call to `register_builtin_write_tools()` returns `Err(DuplicateWriteToolRegistration)`
   for the first duplicate tool_name encountered (per BC-2.16.012 EC-016-012-004).

**Boot-wiring:** `register_builtin_write_tools()` is called in `crates/prism-bin/src/boot.rs`
step 7.5 immediately after plugin loading and before `mark_query_phase_started()`. If step 7.5
is a `todo!()` stub, a justified `todo!()` with the anchor comment below is added:

```rust
// PLUGIN-MIGRATION-001-B: call register_builtin_write_tools() here at step 7.5
// before mark_query_phase_started() (BC-2.16.012 INV-INVALIDATION-EXT-001).
// TODO: prism_query::invalidation::register_builtin_write_tools()
//   .expect("boot: write tool registration failed");
```

The boot-wiring state (fully wired or `todo!()` stub) is documented in §Known Gaps.

(traces to BC-2.16.012 invariant INV-INVALIDATION-EXT-001 — WRITE_TOOL_INVALIDATION_MAP
source content is accessible via `register_builtin_write_tools()`; `DYNAMIC_WRITE_TOOLS`
is the runtime-extensible target; boot registration path is established)

---

### AC-004: No new prism-query crate dependencies introduced (traces to BC-2.16.012 postcondition — behavioral output byte-identical for four initial sensors)

`crates/prism-query/Cargo.toml` is NOT modified for runtime dependencies. The conversion
uses only APIs already available in prism-query's existing dependency graph:
- `prism-spec-engine::write_endpoint::WriteEndpointRegistry` — already a dep via `prism_spec_engine`
- `prism-query::invalidation::register_write_tool` — already in the same crate

**Amendment (PLUGIN-MIGRATION-001-B fix-burst, 2026-05-26):** The `[features]` section of
`Cargo.toml` was amended to restore empty feature declarations for
`crowdstrike-write`, `cyberint-write`, `claroty-write`, `armis-write`, and `all-write`.
These features were removed by PLUGIN-MIGRATION-001-A's cross-crate forwarding chain deletion,
which silently caused ~20 test functions gated with `#[cfg(feature = "crowdstrike-write")]`
etc. to be dropped from `--all-features` builds. Restoring them as empty declarations (no
cross-crate forwarding) preserves test coverage until PLUGIN-MIGRATION-001-F de-gates them.
No runtime dependencies are added; only the `[features]` stubs are restored.

**Verification:**

```bash
git diff develop..HEAD -- crates/prism-query/Cargo.toml
```

Returns only the `[features]` block additions (5 feature stubs). No `[dependencies]` or
`[dev-dependencies]` changes.

(traces to BC-2.16.012 postcondition — structural open-dispatch migration; no new runtime
dependencies; behavioral equivalence for four initial sensors preserved)

---

### AC-005: Workspace-wide `just check` GREEN (traces to BC-2.16.012 invariant INV-SPEC-PARSER-OPEN-001 — no compile regression)

`just check` (fmt + clippy + nextest + doctests + crate-layout) passes workspace-wide with
all pre-existing tests green. No tests that were passing before this story are made to fail.

Specifically:
- All `write_pipeline.rs` tests that exercised the old match-arm dispatch continue to pass
  (they test `WriteExecutor` via mock `WriteEndpointRegistry` — the registry-driven path is
  drop-in compatible with the mock).
- All `explain_tests.rs` tests that reference `per_sensor_latency_ms` continue to pass
  (they assert values, not the specific per-sensor breakdowns — the uniform 300ms default
  is a valid value for all four sensors since 300 was the prior wildcard value).
- VP-156 proptest (WriteToolInvalidationMap registration uniqueness) continues to pass.

(traces to BC-2.16.012 invariant INV-SPEC-PARSER-OPEN-001 — workspace compile-clean; open
dispatch regression-free)

---

## Tasks

Implementer: follow strict TDD discipline — write the Red Gate test first (Task 2 per site),
confirm it fails RED for the right reason, then apply the conversion (Task 3 per site) to
drive GREEN.

### Task 1: Read source files and BCs before writing any code

Read in full:
- `crates/prism-query/src/explain.rs` lines 1040–1080 (SITE-1 latency heuristic)
- `crates/prism-query/src/write_pipeline.rs` lines 20–35 (imports) and 310–335 (SITE-2 match)
- `crates/prism-query/src/invalidation.rs` lines 30–315 (SITE-3 static map and register API)
- `crates/prism-security/src/feature_flag.rs` lines 240–300 (write gate functions)
- `crates/prism-spec-engine/src/write_endpoint.rs` lines 238–360 (WriteEndpointRegistry API)
- BC-2.16.012 v1.33 (invariants INV-SPEC-PARSER-OPEN-001 and INV-INVALIDATION-EXT-001)
- BC-2.01.013 v1.7 (postconditions)

Confirm:
- The 4-arm latency match is at `explain.rs:1053-1059` (or the current line after 001-A diff)
- The 4-arm compile-gate match is at `write_pipeline.rs:324-331`
- `WRITE_TOOL_INVALIDATION_MAP` is at `invalidation.rs:252-310` and has 8 entries
- `register_write_tool()` and `DYNAMIC_WRITE_TOOLS` exist at `invalidation.rs:41-229`
- `WriteEndpointRegistry::get(&sensor, &verb)` is the registry lookup API

### Task 2: Write Red Gate tests (all 3, RED first)

In the appropriate test files, add:
- `test_BC_2_16_012_B_001_explain_unknown_sensor_latency_is_300` (AC-001)
- `test_BC_2_16_012_B_002_write_gate_absent_for_unregistered_sensor` (AC-002)
- `test_BC_2_16_012_B_003_register_builtin_write_tools_populates_dynamic_registry` (AC-003 — sub-parts 1-4)

Run per-site:
```bash
cargo nextest run -p prism-query -E 'test(test_BC_2_16_012_B)'
```
Confirm each test fails RED (either because the production code isn't changed yet, or because
a helper like `register_builtin_write_tools()` doesn't exist yet). Do not proceed to Task 3
until RED is confirmed for each.

### Task 3: SITE-1 — Remove latency heuristic match (GREEN)

Apply the `explain.rs` change per AC-001. Replace the 4-arm match with `let latency_ms = 300_u64;`.

```bash
cargo nextest run -p prism-query -E 'test(test_BC_2_16_012_B_001)'
```
Confirm GREEN. Run crate suite:
```bash
just iter prism-query
```
Confirm no regressions.

Verify zero matches:
```bash
grep -n '"crowdstrike"\|"cyberint"\|"claroty"\|"armis"' crates/prism-query/src/explain.rs
```

### Task 4: SITE-2 — Replace write gate dispatch (GREEN)

Apply the `write_pipeline.rs` change per AC-002. Replace the match with
`WriteEndpointRegistry::get()` presence check. Remove sensor-specific function imports.

```bash
cargo nextest run -p prism-query -E 'test(test_BC_2_16_012_B_002)'
```
Confirm GREEN. Run crate suite:
```bash
just iter prism-query
```
Confirm no regressions.

Verify zero matches:
```bash
grep -n '"crowdstrike"\|"cyberint"\|"claroty"\|"armis"\|crowdstrike_write_gate\|cyberint_write_gate\|claroty_write_gate\|armis_write_gate' \
  crates/prism-query/src/write_pipeline.rs
```

### Task 5: SITE-3 — Add `register_builtin_write_tools()` (GREEN)

Add the `register_builtin_write_tools()` function and update the `WRITE_TOOL_INVALIDATION_MAP`
doc-comment per AC-003. Wire to `prism-bin/src/boot.rs` step 7.5 or add the `todo!()` stub
with anchor comment.

```bash
cargo nextest run -p prism-query -E 'test(test_BC_2_16_012_B_003)'
```
Confirm all 4 sub-parts of AC-003 pass GREEN.

### Task 6: Final workspace gate

```bash
just check
```
Must pass GREEN. Resolve any clippy warnings. Confirm no regressions across all crates.

### Task 7: Verify dispatch-site invariants

Run the grep from BC-2.16.012 TV-BC-2.16.012-001 (adapted for prism-query):

```bash
grep -rn '"crowdstrike"\|"cyberint"\|"claroty"\|"armis"' \
  crates/prism-query/src/explain.rs \
  crates/prism-query/src/write_pipeline.rs \
  crates/prism-query/src/invalidation.rs \
  | grep -v '//\|/// \|#\[cfg(test'
```

Returns zero matches in production dispatch contexts.

---

## Previous Story Intelligence

PLUGIN-MIGRATION-001-A (direct predecessor, merged PR #156 develop@948a709f):

- `init_registry_for_org` in `prism-sensors` was rewritten to dispatch via spec-catalog instead
  of constructing hardcoded adapters. The pattern is: spec-catalog provides the sensor
  configuration; the registry is populated from loaded specs at boot time.
- Three auth modules (`claroty.rs`, `cyberint.rs`, `armis.rs`) were deleted. Any prism-query
  code that imported symbols from those modules must already have been cleaned up by 001-A.
  Verify that no lingering imports from those modules exist in prism-query at the start of
  this story.
- The `DYNAMIC_WRITE_TOOLS` RwLock and `register_write_tool()` API were added in
  S-PLUGIN-PREREQ-E (not 001-A), but are confirmed live as of develop@948a709f.
- ADR-028 §D10 co-merge contract: 001-A is now in production. The deploy dependency is
  resolved; no co-merge constraint applies to 001-B.
- ADR-028 §D4 forbids using deleted auth module symbols as ground-truth for any behavioral
  claim. 001-B does NOT touch those modules; all three conversion sites are entirely within
  prism-query.

---

## Architecture Compliance Rules

Extracted from `architecture/module-decomposition.md`, `ADR-023`, `ADR-027`:

| Rule | Source | Enforcement |
|------|--------|-------------|
| No hardcoded sensor-name match arms in dispatch-context code | ADR-023 Rule 2 / BC-2.16.012 INV-SPEC-PARSER-OPEN-001 | Task 7 grep; adversary grep check |
| `WRITE_TOOL_INVALIDATION_MAP` entries must be accessible via `register_write_tool()` at boot | BC-2.16.012 INV-INVALIDATION-EXT-001 | AC-003 Red Gate test |
| `WriteEndpointRegistry::get()` is the correct open-dispatch API for write gate determination | ADR-023 Rule 1 (TOML as declarative baseline) | AC-002; registry presence = write capability |
| No `unwrap()` / `expect()` in production code added by this story | CLAUDE.md §Conventions | Clippy + adversary |
| No `println!` in production code | CLAUDE.md §Conventions | Clippy `--deny warnings` |
| Boot wiring for `register_builtin_write_tools()` must precede `mark_query_phase_started()` | BC-2.16.012 EC-016-012-005 (AtomicBool query-phase flag) | AC-003 assertion + boot.rs ordering |

### Forbidden Dependencies

`prism-query` MUST NOT gain any new dependency in `Cargo.toml` through this story. All three
conversion sites use APIs already in prism-query's existing dependency graph:
- `WriteEndpointRegistry` is already accessible via the existing `prism-spec-engine` dep.
- `register_write_tool` / `DYNAMIC_WRITE_TOOLS` are in `prism-query` itself (`invalidation.rs`).
- No new cross-crate deps are required.

If any of the above turn out to require a new Cargo.toml dep, stop and escalate to the
architect — do not add an undeclared dependency without review.

---

## Library and Framework Requirements

| Library | Version | Pin Source |
|---------|---------|------------|
| `arrow` | per workspace `Cargo.toml` | workspace pin |
| `datafusion` | per workspace `Cargo.toml` | workspace pin |
| `tokio` | per workspace `Cargo.toml` | workspace pin |
| `prism-spec-engine` | workspace path dep | no version change |
| `prism-security` | workspace path dep | no version change |

Do NOT introduce new crate dependencies in `prism-query/Cargo.toml`. This story is a
conversion/simplification — it reduces prism-query's implicit sensor-specific coupling
to prism-security (fewer `{sensor}_write_gate` imports), net-zero on external deps.

---

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/prism-query/src/explain.rs` | MODIFY | Remove 4-arm latency match at line ~1053; replace with `let latency_ms = 300_u64;` |
| `crates/prism-query/src/write_pipeline.rs` | MODIFY | Remove 4-arm compile-gate match at line ~324; remove sensor-specific gate function imports; add registry-driven check |
| `crates/prism-query/src/invalidation.rs` | MODIFY | Add `pub fn register_builtin_write_tools()` helper; update `WRITE_TOOL_INVALIDATION_MAP` doc-comment |
| `crates/prism-bin/src/boot.rs` | MODIFY (minor) | Wire `register_builtin_write_tools()` at step 7.5, or add scoped `todo!()` with anchor comment |
| `crates/prism-query/src/tests/explain_tests.rs` | MODIFY | Add Red Gate test `test_BC_2_16_012_B_001_explain_unknown_sensor_latency_is_300` |
| `crates/prism-query/src/tests/` or inline `#[cfg(test)]` | MODIFY | Add Red Gate tests `test_BC_2_16_012_B_002` and `test_BC_2_16_012_B_003` |
| `crates/prism-query/Cargo.toml` | NO CHANGE | No new deps |
| `crates/prism-security/src/feature_flag.rs` | NO CHANGE | The write gate functions remain for use by other callers; only prism-query's dispatch match is removed |
| `crates/prism-spec-engine/src/write_endpoint.rs` | NO CHANGE | `WriteEndpointRegistry::get()` API already exists |

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `WriteEndpointRegistry` is empty at query time (specs not yet loaded at boot) | SITE-2 returns `CompileFeatureGate::Absent` for all sensors — write is rejected. This is safe fail-closed behavior; no panic, no silent success. |
| EC-002 | `register_builtin_write_tools()` is called twice (e.g., double-boot in tests) | Second call returns `Err(DuplicateWriteToolRegistration)` for the first duplicate tool_name per BC-2.16.012 EC-016-012-004. Caller must reset `DYNAMIC_WRITE_TOOLS` between calls in tests (use `clear_dynamic_write_tools()` if available). |
| EC-003 | An explain query targets a sensor not in the four built-in sensors (e.g., a future plugin sensor) | SITE-1 returns `latency_ms = 300` uniformly. No error. The `per_sensor_latency_ms` map entry is populated with 300. Correct open-dispatch behavior. |
| EC-004 | `FeatureFlagEvaluator` import is still used elsewhere in `write_pipeline.rs` after import reduction | Keep the import; do not break other callers. Implementer must grep the entire file before removing any import. |
| EC-005 | The explain test suite uses hardcoded latency value assertions for the four built-in sensors | Existing tests that assert `per_sensor_latency_ms["crowdstrike"] == 250` (or similar) will fail after this change — those assertions must be updated to assert `== 300`. This is a correct test update (the new uniform value IS 300). Do not suppress these failures; fix them. |
| EC-006 | `WRITE_TOOL_INVALIDATION_MAP` is read by `invalidate_for_sensor` before `register_builtin_write_tools()` runs | The static read is still present; no data is lost. The migration makes `DYNAMIC_WRITE_TOOLS` the authoritative path; the static remains as a transitional fallback. No data loss or silent skip. |

---

## §Known Gaps

| Gap ID | Scope | Description | Resolution Target |
|--------|-------|-------------|-------------------|
| GAP-001-B | L3 | `register_builtin_write_tools()` boot wiring in `prism-bin/src/boot.rs` step 7.5 may be a `todo!()` stub if step-7.5 infrastructure is not yet wired at implement time | PLUGIN-MIGRATION-001-F or S-WAVE5-PREP-01 (existing boot stub pattern) |
| GAP-002-B | L3 | `WRITE_TOOL_INVALIDATION_MAP` static is retained as a transitional fallback — `invalidate_for_sensor` still iterates both static and dynamic. Full removal of the static is deferred to after PLUGIN-MIGRATION-001-F when tests no longer reference it. | PLUGIN-MIGRATION-001-F |
| GAP-003-B | L3 | SITE-2 registry-driven gate uses `endpoint_registry.get()` presence only — it does not cross-check the `{sensor}-write` Cargo feature. For the four built-in sensors, the spec catalog and the Cargo feature gate are in sync; for a future plugin sensor, the registry presence is the correct capability signal. No action required until a non-built-in write-capable sensor ships. | Wave 5+ (first third-party write plugin) |

---

## Dependencies

### Satisfied (ALL MERGED)

| Dependency | PR / SHA | Notes |
|------------|----------|-------|
| S-PLUGIN-PREREQ-A | MERGED | SensorId newtype; dispatch sites use `SensorId::as_ref()` |
| S-PLUGIN-PREREQ-C | MERGED | TOML grammar extensions; WriteEndpointRegistry is populated from TOML specs |
| PLUGIN-MIGRATION-001-A | PR #156, develop@948a709f | Deleted auth modules; spec-catalog dispatch in prism-sensors. This story must not retain imports from deleted modules. |

### Pending (none)

All dependencies are satisfied. This story is unblocked as of develop@948a709f.

---

## §Source Citations

| Artifact | Version / SHA | Authoritative Symbols |
|----------|-------------|----------------------|
| `crates/prism-query/src/explain.rs` | develop@948a709f | latency heuristic match at line ~1053 (4 arms: crowdstrike→250, cyberint→400, claroty→350, armis→300, _→300) |
| `crates/prism-query/src/write_pipeline.rs` | develop@948a709f | compile-gate match at line ~324 (4 arms: crowdstrike→crowdstrike_write_gate(), etc.); imports of sensor-specific gate functions at line ~26 |
| `crates/prism-query/src/invalidation.rs` | develop@948a709f | `WRITE_TOOL_INVALIDATION_MAP` LazyLock at line ~252 (8 entries, 4 sensors); `DYNAMIC_WRITE_TOOLS` RwLock at line ~41; `register_write_tool()` at line ~153 |
| `crates/prism-security/src/feature_flag.rs` | develop@948a709f | `crowdstrike_write_gate()`, `cyberint_write_gate()`, `claroty_write_gate()`, `armis_write_gate()` at lines ~254-299 |
| `crates/prism-spec-engine/src/write_endpoint.rs` | develop@948a709f | `WriteEndpointRegistry::get(&sensor, &verb) -> Option<&WriteEndpointSpec>` at line ~303; `WriteEndpointRegistry::is_composite()` at line ~354 |
| BC-2.16.012 | v1.33 (2026-05-23) | INV-SPEC-PARSER-OPEN-001 (no hardcoded match arms); INV-INVALIDATION-EXT-001 (TD-S-PLUGIN-PREREQ-A-003 closure) |
| BC-2.01.013 | v1.7 (2026-05-22) | postconditions (spec-driven adapter pattern; no hardcoded dispatch) |
| ADR-023 | v1.19 | §Decision Rules 1-5; Rule 2 (open dispatch mandate); Rule 5 (CustomAdapter retirement) |
| ADR-027 | v1.9 | §D5 (hardcoded-sensor-string dispatch audit for spec_parser.rs + prism-query) |

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| Latency heuristic simplification (SITE-1) | `crates/prism-query/src/explain.rs` | Pure (cost estimation; no I/O) |
| Write gate dispatch conversion (SITE-2) | `crates/prism-query/src/write_pipeline.rs` | Pure (gate check; no I/O in Phase 2) |
| `register_builtin_write_tools()` helper | `crates/prism-query/src/invalidation.rs` | Effectful (RwLock write; called at boot only) |
| Boot wiring call | `crates/prism-bin/src/boot.rs` | Effectful (boot-time orchestration) |
| Red Gate tests | `crates/prism-query/src/tests/` | Pure (unit test assertions) |
