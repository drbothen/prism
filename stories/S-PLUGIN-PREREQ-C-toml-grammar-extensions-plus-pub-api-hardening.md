---
document_type: story
story_id: S-PLUGIN-PREREQ-C
title: "TOML Grammar Extensions + Pub-API Hardening — page_size, JSONPath Bracket/Wildcard, Proptest, Escape Mechanism, #[non_exhaustive] Audit, Cross-Newtype Audit, SensorIdValidationError Re-export"
wave: 0
epic_id: PLUGIN-MIGRATION-001
priority: P0
status: ready
# BC status: behavioral_contracts populated — BC-2.16.002 (pipeline execution; surface extended
#            by AC-1/AC-2) and BC-2.01.013 (datasource-trait adapter pattern; pub-API hardening
#            for spec-engine TOML types is load-bearing for the spec-driven auth surface).
#            Both BCs are active. No new BCs introduced by this story — BC amendment in the
#            same commit is the IMPLEMENTER's obligation for AC-2 (new event_type site for
#            bounds-check error if emitted; see PG-LP11-001 SOP call-out in story body).
behavioral_contracts:
  - BC-2.16.002
  - BC-2.01.013
depends_on:
  - S-PLUGIN-PREREQ-A  # merged PR #142 develop@ae7e26c8 — SensorId(Arc<str>) open newtype
  - S-PLUGIN-PREREQ-B  # merged PR #143 develop@ae7e26c8 — Real PipelineExecutor + AuthProvider
blocks:
  - PLUGIN-MIGRATION-001-A  # Delete 4 Named Auth Modules — gated on pub-API stability (#[non_exhaustive])
  - PLUGIN-MIGRATION-001-B  # Convert sensor-name dispatch sites — requires stable SensorId pub-API
  - PLUGIN-MIGRATION-001-C  # SpecDrivenMapper — requires extended TOML grammar (ocsf_field column mapping)
  - PLUGIN-MIGRATION-001-D  # Author 4 Production TOML Sensor Specs — requires page_size + JSONPath extensions
points: 8
estimated_days: 3
risk: MEDIUM
tdd_mode: strict
crates_touched: [prism-spec-engine, prism-core]
target_module: prism-spec-engine
# Subsystem anchor justifications:
#   SS-16 owns the primary scope of this story: prism-spec-engine's pipeline.rs,
#   spec_parser.rs, and interpolator.rs are all within the spec-engine subsystem per
#   ARCH-INDEX Subsystem Registry. TOML grammar extensions (AC-1/2/3/4/5) all land there.
#   SS-01 is included because AC-6 (cross-newtype audit in prism-core) and AC-7
#   (SensorIdValidationError re-export in prism-core lib.rs) directly modify the core-types
#   subsystem (SS-01). AC-5's compile-fail test also exercises the prism-core consumer surface.
subsystems: [SS-16, SS-01]
# Capability anchors: CAP-029 (spec-driven sensor fetch pipeline, primary); CAP-001 (sensor
# identifier and type system — impacted by AC-6/7 prism-core pub-API hardening)
capabilities: [CAP-029, CAP-001]
version: "1.2"
level: "L4"
producer: story-writer
timestamp: "2026-05-12T00:00:00Z"
updated: "2026-05-12"
input-hash: "6954524"
traces_to: []
cycle: "v1.0.0-greenfield"
phase: 3
verification_properties:
  - VP-PLUGIN-002  # PipelineExecutor returns non-empty records against wiremock DTU clone
  - VP-PLUGIN-005  # OAuth2 refresh-on-401 via declarative TOML retry policy
anchor_vps: [VP-PLUGIN-002, VP-PLUGIN-005]
anchor_bcs: [BC-2.16.002, BC-2.01.013]
anchor_capabilities: [CAP-029]
anchor_subsystem: [SS-16, SS-01]
assumption_validations: []
risk_mitigations: []
acceptance_criteria_count: 7
red_gate_tests: 0
estimated_passes: "8-12 LOCAL adversary passes"
inputs:
  - ".factory/specs/architecture/decisions/ADR-023-plugin-only-sensor-architecture.md"
  - ".factory/specs/behavioral-contracts/BC-2.16.002-multi-step-fetch-pipeline.md"
  - ".factory/specs/behavioral-contracts/BC-2.01.013-datasource-trait-adapter-pattern.md"
  - ".factory/stories/S-PLUGIN-PREREQ-A-sensorid-newtype.md"
  - ".factory/stories/S-PLUGIN-PREREQ-B-real-pipeline-executor.md"
  - ".factory/tech-debt-register.md"
  - ".factory/cycles/wave-4-operations/lessons.md"
# TD items resolved by this story
td_resolves:
  - TD-S-PLUGIN-PREREQ-B-001  # P2 — page_size on cursor pagination first-call
  - TD-S-PLUGIN-PREREQ-B-003  # P3 — JSONPath bracket notation + wildcard support
  - TD-S-PLUGIN-PREREQ-B-006  # P2 — proptest coverage for pure functions
  - TD-S-PLUGIN-PREREQ-B-008  # P3 — Interpolator escape mechanism for literal ${...}
  - TD-S-PLUGIN-PREREQ-B-016  # P2 — #[non_exhaustive] crate-wide audit for pub TOML-deserialized types
  - TD-S-PLUGIN-PREREQ-A-006  # P3 — cross-newtype pub-API validation-bypass audit
  - TD-S-PLUGIN-PREREQ-A-008  # P3 — SensorIdValidationError crate-root re-export
  # NOTE: TD-S-PLUGIN-PREREQ-A-007 (validate_sensor_id_string order reorder) is NOT in
  # td_resolves. Story-writer judgment: AC-8 is excluded. See AC-8 deferral note below.
---

# S-PLUGIN-PREREQ-C — TOML Grammar Extensions + Pub-API Hardening

## Narrative

As the Prism platform, I want the `prism-spec-engine` TOML grammar extended with
`page_size` on cursor pagination, bracket/wildcard JSONPath support, a template escape
mechanism, proptest coverage for pure pipeline functions, and `#[non_exhaustive]` applied
to all public TOML-deserialized types, AND the `prism-core` public API hardened with a
cross-newtype validation-bypass audit and `SensorIdValidationError` crate-root re-export,
so that the TOML spec surface is expressive enough for production CrowdStrike, Cyberint,
Claroty, and Armis sensor specs, the pub-API signals to external crates that spec-engine
types are explicitly non-final, and the story unblocks Wave 1 (PLUGIN-MIGRATION-001-A/B/C/D)
from dispatching.

---

## Goal Statement

PREREQ-C is the third and final keystone in the PLUGIN-MIGRATION Wave 0 prerequisite
sequence. PREREQ-A (merged PR #142) replaced the closed `SensorType` enum with
`SensorId(Arc<str>)`. PREREQ-B (merged PR #143) replaced the unconditional
`Ok(Vec::new())` stub with a real HTTP-capable `PipelineExecutor`. PREREQ-C closes the
carry-forward debt from both predecessors: five TD items from PREREQ-B's 16-pass
adversary cascade plus two from PREREQ-A's PR-LEVEL pass-4 observations. Together they
complete the contract surface that Wave 1 stories (PLUGIN-MIGRATION-001-A through -D)
depend on. Without PREREQ-C, 001-A cannot safely delete the four hardcoded auth modules
(the pub-API of `PaginationConfig`, `FetchStep`, and `SensorSpec` is not yet
`#[non_exhaustive]`-stable), 001-D cannot author production CrowdStrike TOML specs
(CrowdStrike GraphQL `first: N` requires `page_size` on every pagination call), and
001-B cannot convert dispatch sites to spec-catalog lookup without a stable
`SensorIdValidationError` crate-root surface.

---

## Behavioral Contracts

| BC ID | Title | Subsystem | Role in This Story |
|-------|-------|-----------|-------------------|
| BC-2.16.002 | Multi-Step Fetch Pipeline Execution — Sequential Steps with Variable Interpolation | SS-16 | Primary authority — AC-1 extends `PaginationConfig::CursorToken` with `page_size` (new field in the Structured Event Catalog-adjacent surface). AC-2 extends `extract_at_path` with bracket notation and wildcard support. Any new `tracing::*!(event_type = "...")` sites introduced by AC-2's bounds-check path MUST amend the Structured Event Catalog table per PG-LP11-001. |
| BC-2.01.013 | DataSource Trait: Spec-Driven Adapter Pattern | SS-01 | Secondary authority — AC-5 applies `#[non_exhaustive]` to all pub TOML-deserialized types, making the spec-driven adapter surface explicitly extensible. AC-6 and AC-7 harden the prism-core types consumed by spec-driven dispatch. |

---

## Token Budget Estimate

| Artifact | Estimated Tokens |
|----------|-----------------|
| This story spec | ~6,500 |
| `prism-spec-engine/src/pipeline.rs` (AC-1 page_size + AC-2 bracket/wildcard) | ~2,500 |
| `prism-spec-engine/src/spec_parser.rs` (AC-5 #[non_exhaustive] on 29 types — see CI EXPECTED=29) | ~2,000 |
| `prism-spec-engine/src/interpolator.rs` or sibling (AC-4 escape mechanism) | ~1,500 |
| `prism-spec-engine/tests/` (AC-3 proptest suite) | ~3,000 |
| `prism-spec-engine/tests/external-construction/` (AC-5 compile-fail test) | ~1,000 |
| `prism-core/src/lib.rs` (AC-7 re-export) + `prism-core/src/sensor_id.rs` (AC-6 audit) | ~1,500 |
| `prism-core/src/tenant.rs` and sibling newtypes (AC-6 audit sites) | ~1,500 |
| BC files (2 BCs: BC-2.16.002, BC-2.01.013) | ~4,000 |
| ADR-023 §§C2/D/E reference | ~2,000 |
| Total | ~26,000 |

Within the 30% context window budget (~40k tokens for a 128k-context agent). The budget
is tighter than PREREQ-B (~21k) primarily because AC-6 requires reading and auditing all
pub newtypes in `prism-core` — breadth, not depth.

---

## Acceptance Criteria

### AC-1 — page_size on cursor pagination first-call (resolves TD-S-PLUGIN-PREREQ-B-001 P2)

`PaginationConfig::CursorToken` gains a new field `page_size: Option<u32>`. The
`build_paged_url()` function in `crates/prism-spec-engine/src/pipeline.rs` (or its
inline pagination helper within that file) appends a `page_size` query parameter to BOTH
the first-call URL (no cursor yet) and all cursor-continuation URLs when `page_size` is
`Some(n)`. When `page_size` is `None`, no `page_size` query parameter is appended.

This closes the CrowdStrike GraphQL `first: N` real-world case: cursor APIs require the
page-size parameter on every request including the first.

**Red Gate test pointer:** Add or update the test function(s) in
`crates/prism-spec-engine/src/` (unit test module within `pipeline.rs` or equivalent
test file exercising `build_paged_url`) that assert the URL-building behavior:
- (a) A `PaginationConfig::CursorToken { page_size: Some(50) }` on a first call (no
  cursor value) produces a URL whose query string contains `page_size=50`.
- (b) A `PaginationConfig::CursorToken { page_size: Some(50) }` on a continuation call
  (with a cursor value) produces a URL whose query string contains both `page_size=50`
  and the cursor parameter.
- (c) A `PaginationConfig::CursorToken { page_size: None }` (or `PaginationConfig::CursorToken`
  without `page_size` field via `..Default::default()` or struct update) on any call
  produces a URL with no `page_size` query parameter.

Name the tests so they include the BC identifier: e.g.,
`test_BC_2_16_002_cursor_pagination_first_call_includes_page_size`,
`test_BC_2_16_002_cursor_pagination_continuation_includes_page_size`,
`test_BC_2_16_002_cursor_pagination_page_size_none_omitted`.

(traces to BC-2.16.002 postcondition: pagination within a step follows the sensor spec's
declared pagination config, iterating until the API returns an empty page or the cursor
is null)

---

### AC-2 — JSONPath bracket notation + wildcard support (resolves TD-S-PLUGIN-PREREQ-B-003 P3)

The `extract_at_path` function in `crates/prism-spec-engine/src/pipeline.rs` (or the
JSONPath extraction helper it delegates to) is extended beyond dot-notation to support:
- **Bracket indexing:** `$.x[0]` extracts the first element of array `x`.
- **Wildcard enumeration:** `$.x[*]` extracts all elements of array `x` and returns them
  as a `Vec<serde_json::Value>` (or the existing return type for array-valued paths).

**Implementation choice (deferred to implementer):** Extend the in-tree minimal JSON
Pointer implementation to cover RFC 6901 bracket notation, OR add a dependency on
`jsonpath-rust` (or `serde_json_path`) in `prism-spec-engine/Cargo.toml`. The choice
is implementation-local: the Red Gate tests specify behavior, not mechanism.

**Backward compatibility:** All existing dot-path tests (e.g., `$.resources`,
`$.data.items`) continue to pass unchanged.

**PG-LP11-001 ALERT:** If the bounds-check path for `$.x[99]` on a 3-element array
emits a new `tracing::*!(event_type = "...")` structured event (e.g.,
`array_index_out_of_bounds`), the implementer MUST amend BC-2.16.002's Structured Event
Catalog in the SAME atomic commit (one commit per TD-VSDD-053). See the PG-LP11-001
SOP section below.

**Red Gate test pointer:** Add test functions in the `extract_at_path` unit test module
(within `pipeline.rs` `#[cfg(test)]` block or a dedicated test file in
`crates/prism-spec-engine/src/`) asserting:
- (a) `$.devices[0].id` on `{"devices":[{"id":"A"},{"id":"B"}]}` extracts `"A"`.
- (b) `$.devices[*].id` on the same JSON enumerates `["A", "B"]` (or the equivalent
  multi-value representation in the existing return type).
- (c) Backward compat: `$.resources` on `{"resources":[{"id":1}]}` still extracts the
  array (existing behavior unchanged).
- (d) Bounds-checked: `$.x[99]` on `{"x":[1,2,3]}` returns a structured error (not a
  panic and not a silent `None`); the exact error variant must be an existing
  `SpecEngineError` variant or a new variant if required.

Name the tests to include the BC: e.g.,
`test_BC_2_16_002_extract_bracket_index`,
`test_BC_2_16_002_extract_wildcard_enumeration`,
`test_BC_2_16_002_extract_backward_compat_dot_path`,
`test_BC_2_16_002_extract_bracket_out_of_bounds_structured_error`.

(traces to BC-2.16.002 postcondition: variable interpolation uses `${step_name.field}`
where `field` is a JSONPath-like dot-notation path; this story extends that surface to
bracket notation and wildcards as declared in BC-2.16.002 postcondition variable
interpolation syntax block)

---

### AC-3 — Proptest fixtures for fan_out + extract + interpolate (resolves TD-S-PLUGIN-PREREQ-B-006 P2)

Proptest coverage is added for the following pure functions in
`crates/prism-spec-engine/src/`:
- `fan_out_batches` (in `pipeline.rs` or its module)
- `extract_at_path` (extended by AC-2)
- `Interpolator::interpolate` (in `interpolator.rs` or the struct containing
  the interpolation logic)
- `Interpolator::extract_references` (companion method enumerating template variables)

The proptest suite lives in `crates/prism-spec-engine/src/` or
`crates/prism-spec-engine/tests/` alongside existing tests. Use the `proptest!` macro.
Honor the project's `PROPTEST_CASES` setting: the Justfile sets `PROPTEST_CASES=32` for
the inner-loop `just iter` target (8× lower than the default 256 for speed); the full
`just check` gate runs at default.

**Required properties:**

- (a) **Totality for fan_out_batches** — for any non-empty input vector of length `n`
  and any `batch_size` of 1 to 500, the output batch count equals `(n + batch_size - 1)
  / batch_size` (ceiling division). No panic for any valid input.
- (b) **Batch-size bounds for fan_out_batches** — no output batch contains more than
  `batch_size` elements.
- (c) **Totality for extract_at_path** — for any JSON string and any path string,
  `extract_at_path` returns either `Ok(_)` or an `Err(SpecEngineError::...)` — never
  panics, never produces an `unwrap()` failure.
- (d) **Totality for Interpolator::interpolate** — for any template string and any
  variable map, `interpolate` returns either `Ok(String)` or a structured error variant
  — never panics on adversarial input.
- (e) **Round-trip for Interpolator::extract_references** — for any template string,
  every reference returned by `extract_references` can be matched by a subsequent
  call to `interpolate` against a map containing those keys without error.

**Red Gate test pointer:** The proptest test functions are the Red Gate anchors. Name
them to include the BC: e.g., `proptest_fan_out_batches_total_count`,
`proptest_fan_out_batches_max_batch_size`, `proptest_extract_at_path_totality`,
`proptest_interpolate_totality`, `proptest_extract_references_round_trip`.

(traces to BC-2.16.002 postcondition: fan-out results are concatenated into a single
result set; interpolation produces an HTTP request using path_template and body_template
interpolated against variables)

---

### AC-4 — `$${...}` literal escape mechanism (resolves TD-S-PLUGIN-PREREQ-B-008 P3)

The `Interpolator` in `crates/prism-spec-engine/src/interpolator.rs` (or the file
containing the interpolation regex and replacement logic) is updated to support a
double-dollar escape sequence: `$${...}` interpolates to the literal string `${...}`
without triggering variable substitution. This allows TOML spec authors to embed
documentation strings that contain template syntax (e.g., a body_template that
explains the format to the API without interpolating).

**Exact escape semantics (fix-burst-1 closure: context-free implementation):**
The escape is context-free — any `$$` pair collapses to `$` regardless of what follows.
To embed a literal `${var}` in the output, write `$${var}`: the `$$` collapses to `$`,
then the remaining `{var}` is NOT interpolated (no longer preceded by `$`).

- `$${var}` → literal `${var}` (no lookup in variable map).
- `${var}` → interpolated value of `var` (existing behavior, unchanged).
- `$$${var}` → literal `$` followed by interpolated value of `var` (double-dollar
  before the opening brace escapes exactly one dollar; the remaining `${var}` is live).

**Red Gate test pointer:** Add test functions in the `Interpolator` unit test block:
- (a) `test_BC_2_16_002_interpolator_escape_double_dollar` — assert `$${var}` with
  `var = "hello"` in scope produces the literal string `${var}`, NOT `hello`.
- (b) `test_BC_2_16_002_interpolator_live_reference_unaffected` — assert `${var}` with
  `var = "hello"` in scope still produces `hello` (backward compat).
- (c) `test_BC_2_16_002_interpolator_triple_dollar_escape` — assert `$$${var}` with
  `var = "hello"` produces `$hello` (one literal dollar + interpolated value).
- (d) *(optional — if proptest in scope)* A proptest property asserting that for any
  template string, applying escape + interpolate round-trips to the original for
  escaped sequences.

(traces to BC-2.16.002 postcondition: path_template and body_template are interpolated
against variables — escape mechanism is a grammar extension of the interpolation surface)

---

### AC-5 — `#[non_exhaustive]` audit of pub TOML-deserialized types (resolves TD-S-PLUGIN-PREREQ-B-016 P2)

`#[non_exhaustive]` is applied to ALL pub TOML-deserialized types in
`crates/prism-spec-engine/`. The full target set (from TD-S-PLUGIN-PREREQ-B-016, sourced
from pass-10 F-LP10-LOW-001) is:

**Audited types (29 total) — `#[non_exhaustive]` applied:**

#### `crates/prism-spec-engine/src/spec_parser.rs` (9 types)
| Type | Variant kind | Default impl |
|------|--------------|--------------|
| `CredentialRef` | struct | derived |
| `SensorSpec` | struct | explicit `impl Default` |
| `SensorTableDescriptor` | struct | explicit `impl Default` |
| `FetchStep` | struct | explicit `impl Default` |
| `ColumnSpec` | struct | explicit `impl Default` |
| `TableSpec` | struct | (existing `new_point_in_time`) |
| `PaginationConfig` | enum | N/A (enum) |
| `AuthType` | enum | N/A (enum) |
| `RateLimitHints` | struct | derived |

#### `crates/prism-spec-engine/src/write_endpoint.rs` (3 types)
| Type | Variant kind | Default impl |
|------|--------------|--------------|
| `BatchMode` | enum | N/A (enum) |
| `WriteStep` | struct | derived |
| `WriteEndpointSpec` | struct | explicit `impl Default` |

#### `crates/prism-spec-engine/src/infusion/mod.rs` (8 types)
| Type | Variant kind | Default impl |
|------|--------------|--------------|
| `InfusionType` | enum | N/A (enum) |
| `BuiltInSourceType` | enum | N/A (enum) |
| `InfusionSourceConfig` | struct | explicit `impl Default` |
| `CredentialRef` | struct | derived |
| `InfusionField` | struct | derived |
| `PipeStageConfig` | struct | derived |
| `PluginConfig` | struct | derived |
| `InfusionSpec` | struct | explicit `impl Default` |

#### `crates/prism-spec-engine/src/types.rs` (7 config-input types + 1)
| Type | Variant kind | Default impl |
|------|--------------|--------------|
| `SensorTableDescriptor` | struct | explicit `impl Default` |
| `CredentialRef` | struct | derived |
| `SensorSpec` | struct | explicit `impl Default` |
| `ColumnType` | enum | N/A (enum) |
| `ColumnDef` | struct | derived |
| `PaginationType` | enum | N/A (enum) |
| `SpecStatus` | enum | N/A (enum) |
| `ClientStatus` | enum | N/A (edge case — annotated for completeness; conceptually edge of config-input / wire boundary) |

#### `crates/prism-core/src/column.rs` (2 types)
| Type | Variant kind | Default impl |
|------|--------------|--------------|
| `ColumnType` | enum | N/A (enum) |
| `ColumnOptions` | struct | derived |

**Total: 29 types audited.** Source-of-truth: `.github/workflows/ci.yml` `EXPECTED=29` assertion in the `non-exhaustive-violation-compile-fail` job. Violator crate at `tests/external/non-exhaustive-violation/` exercises external struct-literal violations (18 E0639) + match-without-wildcard violations (11 E0004) = 29 total compile-fail errors.

**AC-5 audit scope:**

- **INCLUDED:** pub TOML-deserialized CONFIG-INPUT types in prism-spec-engine and prism-core (the 29 types enumerated above). These types are deserialized from `*.sensor.toml`, `*.infusion.toml`, and other configuration sources.
- **EXCLUDED:** pub Deserialize MCP-wire types (request DTOs, result types, status events). These types are stability-governed by the MCP protocol specification, not by Rust's `#[non_exhaustive]` forward-compat property. Documented in `crates/prism-spec-engine/src/types.rs` per the F-LP3-MED-002 adjudication.

The excluded MCP-wire types (11 total — for awareness, NOT audited by AC-5):
- `SensorSpecEntry`, `ConfigSnapshot`, `ValidationError`, `ModeChange`, `ReloadResult`, `ReloadStatus`, `ModifiedSpec`, `AddSensorSpecResult`, `ListSensorSpecsResult`, `AddSensorSpecArgs`, `ListSensorSpecsArgs`

Their stability is governed by the MCP protocol version; adding a variant requires an MCP version bump, not a Rust source-level annotation.

Each `#[non_exhaustive]` annotation MUST be accompanied by a doc-comment on the type
explaining why external struct-literal construction is not supported (e.g., "Fields may
be added in future releases without a semver bump; use the `Default` impl or builder
pattern."). Do NOT add `#[non_exhaustive]` to types that already have it (e.g.,
`FetchContext`, `PipelineResult`, `SpecEngineError` — these are already annotated per
pass-10 PREREQ-B audit).

**Compile-fail test (Red Gate anchor):** Add a compile-fail test under
`crates/prism-spec-engine/tests/` (suggest directory:
`crates/prism-spec-engine/tests/external-construction/`) that attempts struct-literal
construction of `SensorSpec` (or `FetchStep`) from outside the crate using all known
fields. With `#[non_exhaustive]` in place, this MUST fail to compile with the
`cannot create non-exhaustive struct with a struct expression` error. Use
`trybuild` or `compile_fail` doc-test convention. The compile-fail test is the Red Gate
anchor for this AC.

Name the test: `test_BC_2_01_013_non_exhaustive_sensor_spec_no_external_literal`.

(traces to BC-2.01.013 postcondition: the spec-driven adapter surface is explicitly
extensible — `#[non_exhaustive]` is the compile-time enforcement that external crates
cannot struct-literal-construct TOML spec types, ensuring forward compatibility with
field additions)

---

### AC-6 — Cross-newtype `*::new_unchecked` audit (resolves TD-S-PLUGIN-PREREQ-A-006 P3)

All pub-API validation-bypass constructors in `crates/prism-core/` are audited. The
known example from TD-S-PLUGIN-PREREQ-A-006 is `OrgSlug::new_unchecked` in
`crates/prism-core/src/tenant.rs`. The audit scope is: every pub constructor across all
newtypes in `prism-core` that bypasses validation (i.e., does not invoke the type's
canonical validation logic that the normal `TryFrom`/`new` path would invoke).

**For each identified `*::new_unchecked` (or equivalent):**
- (a) Add or update the doc-comment to clearly state the precondition that must hold
  for the caller (i.e., what invariant the caller is asserting by using
  `new_unchecked`).
- (b) Optionally restrict the visibility to `cfg(any(test, feature = "test-helpers"))`
  if the constructor has no legitimate production use (implementer's judgment; if
  `OrgSlug::new_unchecked` is called from production paths, document why; if it is
  test-only, gate it).

Note (fix-burst-1 closure): the production caller in `prism-query/src/materialization.rs`
has been migrated from `OrgSlug::new_unchecked()` to validated `OrgSlug::new()` with
explicit error handling. The `new_unchecked` constructor remains in the allowlist for
test fixtures only.

**Red Gate test (workspace grep regression):** Add a CI script or a Rust `#[test]` in
`prism-core` that asserts no new `new_unchecked` symbols exist in
`crates/prism-core/src/` beyond the audited set. The simplest approach: a doc-test or
unit test in `lib.rs` that includes a comment-inventory listing the known
`new_unchecked` sites; future additions will require updating the inventory, which is
a code-review-visible change.

Alternatively, add a script at `scripts/audit-new-unchecked.sh` (or as a Justfile
recipe) that greps for `new_unchecked` in `crates/prism-core/src/` and fails if the
count exceeds the known baseline, to be run in CI. Document the baseline count in the
script's header comment.

Name the Rust test (if chosen): `test_BC_2_01_013_new_unchecked_inventory_baseline`.

(traces to BC-2.01.013 postcondition: the spec-driven adapter surface enforces
validation-on-construct for sensor identifiers and org identifiers — `new_unchecked`
bypasses that enforcement and must be explicitly inventoried and justified)

---

### AC-7 — `SensorIdValidationError` crate-root re-export (resolves TD-S-PLUGIN-PREREQ-A-008 P3)

`SensorIdValidationError` is re-exported at the `prism_core` crate root, making it
accessible via `use prism_core::SensorIdValidationError;` instead of the currently
required `use prism_core::sensor_id::SensorIdValidationError;`.

The single-line change is in `crates/prism-core/src/lib.rs`: add
`pub use sensor_id::SensorIdValidationError;` alongside the existing
`pub use sensor_id::SensorId;` re-export (which is already at crate root per PREREQ-A).

**Red Gate test:** Add a doctest in `crates/prism-core/src/lib.rs` on the re-export
line that demonstrates `use prism_core::SensorIdValidationError;` compiles and the
error type can be matched. Example doctest form:

```rust
/// ```
/// use prism_core::SensorIdValidationError;
/// // SensorIdValidationError is accessible at crate root
/// let _: Option<SensorIdValidationError> = None;
/// ```
pub use sensor_id::SensorIdValidationError;
```

This doctest is the Red Gate anchor: `cargo test --doc -p prism-core` must pass.

(traces to BC-2.01.013 postcondition: the spec-driven sensor identifier surface exposes
a consistent error type at the crate root — ergonomic parity with `SensorId` which is
already re-exported at `prism_core::SensorId` per PREREQ-A)

---

## AC-8 Deferral: `validate_sensor_id_string` Order Reorder

**Story-writer judgment: AC-8 is excluded from this story.**

TD-S-PLUGIN-PREREQ-A-007 (reorder `validate_sensor_id_string` to length-check before
charset-check) is a P3 defense-in-depth micro-performance refactor. The risk it
addresses is latent only: at the current perimeter bound (`PRISM_MAX_QUERY_SIZE=65_536`),
a 64KB over-size input is rejected at the query-parser layer before reaching
`validate_sensor_id_string`. The reorder has value as a structural safeguard for future
entry points that bypass the parser bound, but bundling it here adds scope to a story
that already spans two crates and seven ACs. AC-8 will be included in the first
micro-cleanup maintenance story after PREREQ-C merges. Deferral is logged against
TD-S-PLUGIN-PREREQ-A-007, which remains open at P3.

---

## Tasks (MANDATORY)

The following sequence minimizes blast radius and front-loads API-stability changes:

**Step 1 — AC-5: `#[non_exhaustive]` audit first (pub-API stability gate)**
Apply `#[non_exhaustive]` to all pub TOML-deserialized config-input types in prism-spec-engine + prism-core (29 types total — see AC-5 body for enumeration; CI EXPECTED=29) before any other changes.
This ensures that AC-1's `page_size: Option<u32>` field addition to
`PaginationConfig::CursorToken` is immediately covered by the `#[non_exhaustive]` policy.
If AC-5 is applied after AC-1, the field addition risks slipping through without the
compile-fail regression test in place.

**Step 2 — AC-7 + AC-6: prism-core small touches**
Both are small (1 line for AC-7; grep + doc-comment + optional feature-gate for AC-6)
and touch a different crate from the remaining ACs. Completing them early reduces the
blast radius of any subsequent rebase if PREREQ-D lands concurrently.

**Step 3 — AC-1 + AC-4: pipeline.rs field additions (same file cluster)**
AC-1 adds `page_size: Option<u32>` to `PaginationConfig::CursorToken` and updates
`build_paged_url()`. AC-4 updates the `Interpolator` escape regex. Both are additive
changes to the `pipeline.rs` / `interpolator.rs` cluster and are straightforward to
combine into one PR commit.

**Step 4 — AC-2: JSONPath bracket/wildcard extension**
AC-2 is the most structurally complex change — bracket notation requires modifying the
path-traversal logic or introducing a dependency. Complete after AC-1/AC-4 so that the
type structure is stable before adding traversal complexity.

**Step 5 — AC-3: Proptest suite as capstone**
Add proptest coverage last after all production code changes are in place. The proptest
suite exercises the final state of all four functions and will immediately surface any
totality violations introduced by AC-1/AC-2/AC-4.

---

## Risks and Known Unknowns

### Risk 1 — JSONPath dependency decision (AC-2)
The implementer must choose between extending the in-tree JSON Pointer implementation
and adding a dependency on `jsonpath-rust` or `serde_json_path`. Tradeoffs:
- In-tree extension: no new dependency, but bracket parsing + wildcard enumeration
  requires non-trivial recursive logic and proptest surface to validate.
- External crate: lower implementer effort, but adds a Cargo dependency that requires
  `cargo deny` and `cargo audit` clearance. The adversary will check that the chosen
  crate is not flagged by `cargo deny` in CI.
The implementer should check `crates/prism-spec-engine/Cargo.toml` and `deny.toml`
before deciding. If adding a new dependency, pin the exact minor version.

### Risk 2 — `#[non_exhaustive]` blast radius across consumers (AC-5)
`SensorSpec`, `FetchStep`, and `PaginationConfig` may be struct-literal-constructed in
test code inside `prism-spec-engine` (intra-crate construction is unaffected by
`#[non_exhaustive]`). However, consumer test code in other crates (e.g., integration
tests in `prism-query`, `prism-bin`, or the compile-fail perimeter tests) that
struct-literal-construct these types will break to compile. The implementer should run
`cargo build --workspace` after applying AC-5 to discover all breakage sites before
writing the compile-fail test.

### Risk 3 — Proptest performance vs `PROPTEST_CASES=32` (AC-3)
The `just iter prism-spec-engine` recipe sets `PROPTEST_CASES=32` (8× lower than the
default 256). The proptest properties for `extract_at_path` with arbitrary JSON strings
may be slow to generate at full `PROPTEST_CASES=256`. The implementer should verify that
the full `just check` gate (which runs at default PROPTEST_CASES) completes within
acceptable time (< 120s warm for `prism-spec-engine`). If a property is too slow at
256 cases, add a per-property `max_shrink_iters` or `cases` override.

### Risk 4 — `OrgSlug::new_unchecked` production use (AC-6)
TD-S-PLUGIN-PREREQ-A-006 cites `tenant.rs` as the known site. Before feature-gating,
verify via `grep` that `new_unchecked` is not called from `prism-bin/src/boot.rs`,
`prism-query/src/`, or any non-test path in `prism-sensors/src/`. If production code
calls it, the feature-gate approach is blocked and the audit outcome is documentation
only (precondition doc-comment + inventory test).

---

## PG-LP11-001 SOP: Structured Event Catalog Discipline

**Source:** `.factory/cycles/wave-4-operations/lessons.md` Lesson 1 (codified
2026-05-11, fix-burst-12 closure of F-LP12-LOW-002).

**Operative rule (applicable to this story):** If the AC-2 implementation introduces a
new `tracing::*!(event_type = "...")` site in any `prism-spec-engine` source file
(specifically: `pipeline.rs`, `interpolator.rs`, `validation.rs`, or any new helper
file introduced by AC-2's bracket/wildcard implementation), the implementer MUST:

1. After making the code change, `git diff | grep 'event_type = "'` to find new emitters.
2. For each new emitter, identify the structured fields (beyond `event_type`) the macro
   includes.
3. Add a new row to BC-2.16.002's Structured Event Catalog table with:
   `event_type | level | function | fields | trigger condition`.
4. Bump BC-2.16.002's version number.
5. Update BC-INDEX with the new BC version.
6. Include the BC amendment in the SAME atomic commit as the code change (TD-VSDD-053).

**Example AC-2 candidate event (if implemented):** A bounds-check failure on
`$.x[99]` for a 3-element array might emit
`tracing::warn!(event_type = "array_index_out_of_bounds", path = %path, array_len = %len)`.
If so, a new catalog row is REQUIRED before the PR can merge.

**Current catalog state:** BC-2.16.002 v1.10 contains 16 rows (14 auth/pagination/fanout
events + 2 new rows: jsonpath_extraction_failed + jsonpath_size_cap_exceeded added in
fix-burst-1). Any PREREQ-C pass-2+ events expand this to 17+.

**Enforcement status (as of 2026-05-12):** Adversary review (LOCAL passes) is the sole
load-bearing enforcement layer. Layer 1 (implementer self-check) and Layer 2
(state-manager pre-commit grep) are not wired in engine prompts. Layer 4 (lefthook
automation TD-VSDD-093) is deferred. The adversary WILL grep for new `event_type = "`
literals in the diff and cross-reference against the BC catalog on each pass.

---

## Anti-Volatile-Pin Reminder (TD-VSDD-091)

Per TD-VSDD-091, the `validate-stable-anchors` hook blocks PRs and commits that cite
volatile line numbers (e.g., `pipeline.rs:143-174`, `sensor_id.rs:258-293`). This story
body contains NO line-number citations. All references use file path + function/symbol +
descriptive prose. When writing fix-burst notes during implementation, maintain this
discipline: identify code by file path + function name, never by line number.

---

## Previous Story Intelligence

### From PREREQ-A (merged PR #142, 12 LOCAL passes, 3/3 CLEAN)

- The `SensorId(Arc<str>)` open newtype is now at `prism_core::SensorId` (crate-root
  re-export). AC-7 uses the same pattern for `SensorIdValidationError`.
- `OrgSlug::new_unchecked` at `prism-core/src/tenant.rs` is the confirmed production
  validation-bypass entry point for AC-6. The PREREQ-A adversary found it at pass-7
  (F-LP7-LOW-002). The doc-comment says "MUST NOT be called from production code" but
  visibility is `pub`.
- Proptest was used in PREREQ-A for the validator-parity property (charset + length
  cross-crate round-trip). AC-3 follows the same pattern.
- The PREREQ-A adversary caught 11 findings across 12 passes. Trajectory: 11→7→5→3→2→
  1→1→1→0→0→0→0. Expect similar trajectory for PREREQ-C given comparable scope (7 ACs,
  two-crate blast radius).

### From PREREQ-B (merged PR #143, 16 LOCAL passes, 3/3 CLEAN)

- The most persistent finding class was **BC↔impl catalog drift** for
  `tracing::*!(event_type = ...)` sites (findings F-LP9-MED-001, F-LP11-MED-001,
  F-LP12-MED-001, F-LP13-MED-001 — four consecutive recurrences before closure). AC-2
  explicitly flags this risk; PG-LP11-001 is cited above.
- Fix-burst-1 corrected the `PaginationConfig::CursorToken` first-call page_size gap
  (filed as TD-S-PLUGIN-PREREQ-B-001). The fix was straightforward once the TD was
  written; the gap had persisted through 1 pass because cursor APIs were not tested
  end-to-end against a real CrowdStrike-schema wiremock stub.
- The `#[non_exhaustive]` discipline gap (filed as TD-S-PLUGIN-PREREQ-B-016 at pass-10)
  was caught late because the adversary focused on behavioral correctness before API
  stability. Apply AC-5 FIRST in the implementation sequence to avoid analogous drift.
- PREREQ-B took 16 LOCAL passes vs the 12 of PREREQ-A because it had a wider behavioral
  surface (9 ACs, HTTP client injection, fan-out, pagination, auth). PREREQ-C has 7 ACs
  but two-crate blast radius. Estimate: 8–12 LOCAL passes to 3/3 CLEAN.

---

## Architecture Compliance Rules

| Rule | Source | Enforcement |
|------|--------|-------------|
| `prism-spec-engine` MUST NOT depend on `prism-query` | ADR-023 §D dependency graph | Compile-fail: adding `prism-query` to `prism-spec-engine/Cargo.toml` breaks the workspace |
| `prism-core` MUST NOT depend on `prism-spec-engine` | Dependency inversion invariant (SS-01 is foundational) | `cargo build --workspace` fails if circular |
| All pub TOML-deserialized types in `prism-spec-engine` MUST carry `#[non_exhaustive]` | AC-5 post-merge invariant; ADR-023 §C2 forward-compatibility constraint | Compile-fail test enforces externally; adversary greps `spec_parser.rs` for unannotated pub structs/enums |
| New `tracing::*!(event_type = "...")` sites MUST have a BC-2.16.002 Structured Event Catalog row in the same commit | PG-LP11-001 (lessons.md Lesson 1) | Adversary grep + BC version check on each pass |
| No volatile line-number citations in story body or fix-burst notes | TD-VSDD-091 | `validate-stable-anchors` hook blocks PR |
| Single atomic commit per burst (story file + index update) | TD-VSDD-053 | Multi-commit chain detector in `.factory/hooks/` blocks dispatch |

**Forbidden dependencies:** `prism-spec-engine` MUST NOT gain a dependency on
`prism-query`, `prism-storage`, `prism-audit`, or any crate above it in the dependency
lattice. If AC-2 adds a JSONPath crate, it must be cleared by `cargo deny` and
`cargo audit`. If the build gains a forbidden dependency, the build MUST fail (enforced
by the existing workspace `deny.toml` and `cargo deny check`).

---

## Library & Framework Requirements (MANDATORY)

| Library | Version | Usage | Note |
|---------|---------|-------|------|
| `serde_json` | workspace pinned | `extract_at_path` traversal; proptest value generation | Already in `prism-spec-engine` prod deps |
| `proptest` | workspace pinned | AC-3 proptest suite | Check `prism-spec-engine/Cargo.toml` dev-deps; add if not present |
| `trybuild` | workspace pinned | AC-5 compile-fail test for `#[non_exhaustive]` | Check workspace dev-deps; add if not present |
| `jsonpath-rust` or `serde_json_path` | implementer's choice — pin exact minor | AC-2 JSONPath (OPTIONAL — only if in-tree extension is not chosen) | Must pass `cargo deny check` before merge |

Do NOT use library version numbers from training data. Check the actual pinned versions
in the workspace `Cargo.lock` and `Cargo.toml` before adding any new dependency.

---

## File Structure Requirements

| File | Action | AC |
|------|--------|----|
| `crates/prism-spec-engine/src/pipeline.rs` | Modify: add `page_size: Option<u32>` to `PaginationConfig::CursorToken`; update `build_paged_url`; extend `extract_at_path` | AC-1, AC-2 |
| `crates/prism-spec-engine/src/spec_parser.rs` | Modify: add `#[non_exhaustive]` to 29 pub TOML-deserialized config-input types across prism-spec-engine and prism-core (see AC-5 body table + CI EXPECTED=29) | AC-5 |
| `crates/prism-spec-engine/src/interpolator.rs` | Modify: update escape regex + replacement logic; or equivalent file containing `Interpolator` | AC-4 |
| `crates/prism-spec-engine/tests/proptest_pipeline.rs` | Create (or extend existing proptest file): add 5 proptest properties for AC-3 | AC-3 |
| `crates/prism-spec-engine/tests/external-construction/` | Create directory + `main.rs` or use trybuild fixture: compile-fail test for AC-5 | AC-5 |
| `crates/prism-core/src/lib.rs` | Modify: add `pub use sensor_id::SensorIdValidationError;` + doctest | AC-7 |
| `crates/prism-core/src/tenant.rs` | Modify: doc-comment update for `OrgSlug::new_unchecked`; optional feature-gate | AC-6 |
| `crates/prism-core/src/sensor_id.rs` | Modify (minor): verify existing pub constructor inventory for AC-6 audit | AC-6 |
| `scripts/audit-new-unchecked.sh` (or Justfile recipe) | Create (optional): workspace grep regression baseline for AC-6 | AC-6 |
| `crates/prism-spec-engine/Cargo.toml` | Modify (conditional): add `jsonpath-rust` or `serde_json_path` prod dep if in-tree extension is not chosen for AC-2; add `proptest` and/or `trybuild` to dev-deps if not present | AC-2, AC-3, AC-5 |

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `PaginationConfig::CursorToken` + `build_paged_url` | `crates/prism-spec-engine/src/pipeline.rs` | Pure |
| `extract_at_path` | `crates/prism-spec-engine/src/pipeline.rs` | Pure |
| `Interpolator::interpolate` + `extract_references` + escape regex | `crates/prism-spec-engine/src/interpolator.rs` | Pure |
| `fan_out_batches` | `crates/prism-spec-engine/src/pipeline.rs` | Pure |
| TOML-deserialized type definitions (`SensorSpec`, `FetchStep`, etc.) | `crates/prism-spec-engine/src/spec_parser.rs` | Pure |
| `SensorIdValidationError` re-export | `crates/prism-core/src/lib.rs` | Pure |
| `OrgSlug::new_unchecked` audit | `crates/prism-core/src/tenant.rs` | Pure |
| Proptest suite | `crates/prism-spec-engine/tests/` | Pure |
| Compile-fail test | `crates/prism-spec-engine/tests/` | Pure |

---

## Purity Classification

All components in this story are **Pure** (no I/O, no side effects, no async):

- `build_paged_url` — URL string construction from config fields; no network calls.
- `extract_at_path` — JSON value traversal via `serde_json`; no I/O.
- `Interpolator::interpolate` / `extract_references` — regex + string replacement; no I/O.
- `fan_out_batches` — vector chunking; pure mathematical operation.
- `#[non_exhaustive]` annotations — compile-time attribute; no runtime behavior.
- `SensorIdValidationError` re-export — type alias; no runtime behavior.
- `OrgSlug::new_unchecked` doc/visibility update — metadata change; no runtime behavior.

No **Effectful** components are introduced in this story. The HTTP pipeline execution
(effectful) lives in `PipelineExecutor::execute` which was delivered in PREREQ-B and is
not modified by this story.

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `page_size: Some(0)` passed to `build_paged_url` | Implementer's choice: either append `page_size=0` verbatim (leave validation to the API) OR return a structured `SpecEngineError::InvalidPaginationConfig`; must be consistent with existing validation patterns |
| EC-002 | `$.x[*]` on an object (not an array): `{"x":{"a":1}}` | Return a structured `SpecEngineError::...` (not a panic); consistent with the out-of-bounds case in AC-2(d) |
| EC-003 | `$${var}` where `var` is not in the variable map | Return the literal `${var}` string (escape fires before variable lookup; no error) |
| EC-004 | `fan_out_batches` called with `batch_size = 0` | Must not panic; either error or treat as batch_size=1 (document the behavior in a unit test) |
| EC-005 | `#[non_exhaustive]` on `AuthType` enum causes match exhaustiveness failure in external crate | External `match` on `AuthType` requires a `_ => {}` wildcard arm — this is correct and intended behavior; ensure AC-5 doc-comments explain this |
| EC-006 | `OrgSlug::new_unchecked` called from `prism-bin/src/boot.rs` production path | Feature-gate is NOT appropriate; add explicit doc-comment precondition instead; document in AC-6 audit output |

---

## Estimated Demo Evidence

Per POL-10 (demo evidence must be story-scoped):

Create `docs/demo-evidence/S-PLUGIN-PREREQ-C/INDEX.md` with the following evidence files:

| File | AC | Content |
|------|----|---------|
| `AC-1-page-size-url-params.md` | AC-1 | Test output showing `page_size=N` in first-call and cursor-continuation URLs; assertion failure output when `page_size: None` |
| `AC-2-jsonpath-bracket-wildcard.md` | AC-2 | Test output for `$.devices[0].id` and `$.devices[*].id` extraction; out-of-bounds structured error |
| `AC-3-proptest-pure-functions.md` | AC-3 | `cargo test -p prism-spec-engine proptest` output showing pass counts; PROPTEST_CASES value used |
| `AC-4-escape-mechanism.md` | AC-4 | Test output for `$${var}` → `${var}`, `${var}` → value, `$$${var}` → `$value` |
| `AC-5-non-exhaustive-audit.md` | AC-5 | `grep #\[non_exhaustive\]` output showing all 29 types annotated (matches CI EXPECTED=29 in ci.yml non-exhaustive-violation-compile-fail job); compile-fail test output showing expected build failure |
| `AC-6-new-unchecked-audit.md` | AC-6 | Output of `scripts/audit-new-unchecked.sh` (or equivalent grep); doc-comment diffs for each audited site |
| `AC-7-sensor-id-validation-error-reexport.md` | AC-7 | `cargo test --doc -p prism-core` output showing doctest pass; `cargo doc` output showing `SensorIdValidationError` at crate root |

---

## Estimated Adversary Pass Count

**Estimate: 8–12 LOCAL adversary passes to 3/3 CLEAN.**

Rationale:
- PREREQ-B took 16 passes (trajectory 20→0) from a higher initial finding count because
  it had a larger behavioral surface (9 ACs, full HTTP pipeline).
- PREREQ-A took 12 passes (trajectory 11→0) from a moderate initial count.
- PREREQ-C has 7 ACs across two crates. The primary risk dimension is the
  `#[non_exhaustive]` blast radius (AC-5): a broad change that can break external crates
  in unexpected ways and requires the compile-fail test to be correctly wired. JSONPath
  extension (AC-2) is another risk vector (implementation complexity, new dependency
  decision).
- Lower bound (8 passes): if AC-5 blast radius is contained within `prism-spec-engine`
  with no consumer breakage, and AC-2 uses the in-tree extension path cleanly.
- Upper bound (12 passes): if AC-2 requires a new crate dependency that triggers
  `cargo deny` findings, or if AC-5 breaks test code in `prism-query` or `prism-bin`
  that is currently doing struct-literal construction of spec-engine types.
- The BC↔catalog drift pattern (PG-LP11-001) is pre-mitigated by the explicit SOP
  callout above, so should not recur as a multi-pass source.

---

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.2 | 2026-05-12 | S-PLUGIN-PREREQ-C-fix-burst-3 | AC-5 narrative reconciliation: 4 "8 types" references replaced with "29 types"; full enumeration table added (29 types across 5 files); explicit AC-5 scope exclusion documented for 11 MCP-wire types (F-LP3-MED-001 + F-LP3-MED-002 adjudication). ClientStatus moved into types.rs sub-table as explicit row. No structural/AC/BC/frontmatter-list changes. Authority: ci.yml EXPECTED=29. |
| 1.1 | 2026-05-12 | state-manager | Narrative amendments for fix-burst-1 closure: AC-4 escape grammar described as context-free (matches implementation); AC-6 notes production caller migrated from new_unchecked to validated new(). No structural/AC/BC/frontmatter-list changes. Burst: S-PLUGIN-PREREQ-C-fix-burst-1. |
| 1.0 | 2026-05-12 | story-writer | Initial draft. 7 ACs from carry-forward TD audit (TD-S-PLUGIN-PREREQ-B-001/003/006/008/016 + TD-S-PLUGIN-PREREQ-A-006/008). AC-8 (TD-A-007 validate_sensor_id_string order reorder) deferred: P3, non-blocking at current perimeter, adds scope to a two-crate story; deferred to first micro-cleanup maintenance story. Estimate 8–12 LOCAL passes. Status: ready. |
