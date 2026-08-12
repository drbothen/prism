---
document_type: story
story_id: S-ADR058-DTU-PARITY-MIGRATION-001
title: "DTU Parity Test Migration for ADR-058 Stage 2 OCSF Field-Path Routing"
wave: "post-demo"
epic_id: EPIC-OCSF-ROUTING
priority: P2
status: draft
version: "1.0"
acceptance_criteria_count: 6
red_gate_tests: 6
level: "L4"
producer: story-writer
timestamp: "2026-08-12T00:00:00Z"
modified: "2026-08-12"
tdd_mode: strict
subsystems: [SS-07, SS-12]
# Subsystem anchor justifications:
#   SS-07 (Spec Engine) owns prism-spec-engine::column_mapping — ColumnMapper::map_record is the
#     Stage 2 translation boundary that this story's tests verify via downstream schema assertions.
#   SS-12 (Sensor Adapters / DTU) owns the four DTU crates whose parity tests are migrated.
crates_touched: [prism-dtu-claroty, prism-dtu-crowdstrike, prism-dtu-armis, prism-dtu-cyberint, prism-bin]
target_module: "prism-dtu-claroty, prism-dtu-crowdstrike, prism-dtu-armis, prism-dtu-cyberint"
capabilities: [CAP-029]
behavioral_contracts:
  - BC-2.16.003
  # BC-2.16.003 v1.4, status: draft (observed 2026-08-12, modified 2026-08-11 — in flux).
  # Governs mapped_fields keyed by ocsf_field path strings — the target schema naming
  # convention this story migrates parity tests to. Read before dispatching to ready.
  - BC-2.01.013
  # BC-2.01.013 v1.16, status: active (observed 2026-08-12).
  # EC-01-025 records the ColumnMapper wiring gap as NON-CONFORMANT.
  # AC-006 closes EC-01-025 after Stage 2 ships.
verification_properties: []
holdout_scenarios: []
depends_on: []
# DEPENDENCY ANCHOR MISSING — this story depends on the ADR-058 Stage 2 OCSF field-path
# routing story. That story does not yet exist. Wire depends_on once a story ID is created.
# Do NOT invent a placeholder ID — per Canonical Principle Rule 3, only real story IDs.
blocks: []
points: 5
# Points justification (lower bound if generators need no output change):
#   - Migrate parity test assertions in 4 DTU crates (ocsf_field column names): ~2 pts
#   - Migrate integration tests in prism-bin to use ocsf_field column names: ~1 pt
#   - Verify BC-2.01.013 EC-01-025 conformance update (product-owner step): ~0.5 pts
#   - Red Gate test authorship (6 tests): ~1.5 pts
#   Total: 5 pts (~1.5 days focused TDD work, assuming generators don't need output changes)
#   If generators need output changes (architect decision pending): bump to 8 pts.
estimated_days: 2
risk: MEDIUM
# Risk justification:
#   MEDIUM because the exact scope is not yet settled — the architect must reconcile ADR-058
#   §C2 (which says generators need to change) against the story-writer finding (generators
#   don't need to change if Stage 2 uses ColumnMapper::map_record). Scope uncertainty makes
#   estimation less reliable. Parity test migration itself is LOW risk (established pattern).
assumption_validations: []
risk_mitigations: []
phase: 3
cycle: "v1.0.0-brownfield"
inputs:
  - ".factory/specs/behavioral-contracts/BC-2.16.003-column-to-ocsf-mapping.md"
  - ".factory/specs/behavioral-contracts/BC-2.01.013-datasource-trait-adapter-pattern.md"
  - ".factory/specs/architecture/decisions/ADR-058-v1-column-naming-col-name-as-arrow-field-identifier.md"
  - "crates/prism-spec-engine/src/column_mapping.rs"
  - "crates/prism-bin/src/spec_driven_adapter.rs"
input-hash: "ffbaf9f"
traces_to:
  - "BC-2.16.003"
  - "BC-2.01.013"
estimated_passes: "tbd"
tags:
  - ocsf-routing
  - dtu-parity
  - adr-058
  - stage2
  - deferral-record
  - human-directed-2026-08-12
---

# S-ADR058-DTU-PARITY-MIGRATION-001: DTU Parity Test Migration for ADR-058 Stage 2 OCSF Field-Path Routing

## Authority

**ADR-058: v1 Column Naming — col.name as Arrow Field Identifier; ocsf_field as Semantic
Metadata.** Version `1.0`, **status: accepted** (observed on disk 2026-08-12, authored
2026-08-11). The human noted ADR-058 is being amended concurrently by the architect — the
implementer MUST re-read ADR-058 at dispatch time. Governing sections: §B (decision),
§C2 (Stage 2 prerequisites including DTU generator scope), §D3 (Stage 2 prerequisite
checklist). Path: `.factory/specs/architecture/decisions/ADR-058-v1-column-naming-col-name-as-arrow-field-identifier.md`.

**BC-2.16.003: Column-to-OCSF Mapping at Query Time.** Version `1.4`, **status: draft**
(observed 2026-08-12, modified 2026-08-11 — in flux at story authoring time). The primary
behavioral authority for the ocsf_field mapping convention. The postconditions govern
`mapped_fields` keyed by `ocsf_field` path strings — the schema naming target this story
migrates parity tests to. Path:
`.factory/specs/behavioral-contracts/BC-2.16.003-column-to-ocsf-mapping.md`.

**BC-2.01.013: DataSource Trait Adapter Pattern.** Version `1.16`, **status: active**
(observed 2026-08-12). EC-01-025 records "ColumnMapper step is missing" as NON-CONFORMANT
under the OCSF Conformance Clause. AC-006 of this story closes EC-01-025 once Stage 2 lands.
Path: `.factory/specs/behavioral-contracts/BC-2.01.013-datasource-trait-adapter-pattern.md`.

---

## Deferral Provenance

Human-directed deferral, 2026-08-12.

The full OCSF field-path routing work (ADR-058 Stage 2 / Interpretation A) is being developed
and validated against the live Claroty sensor. The DTU parity migration is explicitly NOT on
the critical path for that v1 Claroty live work. The human directed creation of this story so
the deferred work is a tracked artifact rather than an informal note, per Canonical Principle
Rule 3, which requires a deferral to carry:

1. Explicit human direction — satisfied (2026-08-12 directive).
2. A concrete dependency — satisfied (ADR-058 Stage 2 routing story must land first).
3. Attachment to a specific story so it cannot be lost — partially satisfied (THIS story is
   the anchor; the upstream Stage 2 routing story does not yet exist and cannot be referenced
   until story-writer creates it).

## Narrative

As a test infrastructure maintainer, I want the four DTU parity test suites
(`prism-dtu-claroty`, `prism-dtu-crowdstrike`, `prism-dtu-armis`, `prism-dtu-cyberint`) to
assert Arrow schema field names using `ocsf_field` values rather than `col.name` values, so
that DTU-based integration tests exercise the same OCSF-keyed column surface that production
queries will use after the Stage 2 OCSF routing story ships.

## Background

### Scope Finding (human-directed 2026-08-12)

The human directed this story to investigate the disputed architect claim before scoping. The
finding follows. An earlier architect report asserted: "all four DTU generators produce records
keyed by `col.name`. All must update to `ocsf_field` keys."

Evidence read from the `fix/claroty-live-api-fidelity` worktree:

`ColumnMapper::map_record` in `prism-spec-engine::column_mapping` reads raw records by
`col.name` (or `source_path`) and emits to `mapped_fields` keyed by `ocsf_field` path
strings. This is the designed translation boundary.

`build_column_array` in `prism-bin::spec_driven_adapter` reads raw records by `col.name`
(or `source_path`) and builds Arrow arrays. The Arrow schema uses `Field::new(&col.name, ...)`
— `col.name` is the current Arrow field identifier. `ColumnMapper::map_record` is NOT called
in the production path; it fires only in tests.

`pipeline_result_to_record_batch` in `prism-bin::spec_driven_adapter` builds the Arrow schema
from `col.name` values and does not call `ColumnMapper::map_record`.

**Finding: if Stage 2 is implemented by wiring `ColumnMapper::map_record` into
`pipeline_result_to_record_batch` (the design-consistent path), DTU generators do NOT need to
change their output format.** Generators produce records keyed by API field names. ColumnMapper
reads those by `col.name` / `source_path` and emits to `ocsf_field` keys. What changes is:
(a) parity tests that currently assert `col.name`-named Arrow schema fields must migrate to
`ocsf_field` names; (b) integration tests querying columns by name must use `ocsf_field`.

**The architect claim is partially correct (parity tests and integration tests definitely
change) but may over-scope generators.** ADR-058 §C2 states "All 4 DTU generators updated to
produce ocsf_field-keyed records" — this would require generators to duplicate the ColumnMapper
mapping knowledge and is inconsistent with the ColumnMapper design. **The architect must
reconcile ADR-058 §C2 before this story reaches `status: ready`.** If the reconciliation finds
generators DO need to change, ACs AC-007..AC-010 must be added (one per generator).

This story ACs are scoped to the minimum-certain work: parity tests and integration tests.

### Prerequisites (must be resolved before status: ready)

1. **Quoting convention ADR**: the product-owner must decide how `ocsf_field` dot-path names
   are expressed as PrismQL column identifiers (underscore-flattened, e.g. `finding_uid`, vs.
   double-quoted dotted path, e.g. `"finding.uid"`). Recorded in ADR-058 §C2 as Stage 2
   precondition.
2. **Stage 2 routing story**: a story ID must exist for the OCSF field-path routing
   implementation. Wire `depends_on:` once that story is created.
3. **Architect reconciliation on generator scope**: architect must reconcile ADR-058 §C2
   with the ColumnMapper design (see §Background §Scope Finding above). If generators need
   to change, add ACs AC-007..AC-010.

## Behavioral Contracts

| BC | Version | Status | Relevance |
|----|---------|--------|-----------|
| BC-2.16.003 | v1.4 | draft | Postconditions govern `mapped_fields` keyed by `ocsf_field` path strings — the target schema naming convention this story migrates parity tests to (traces to postcondition: Arrow schema field names equal ocsf_field values) |
| BC-2.01.013 | v1.16 | active | EC-01-025 NON-CONFORMANT annotation for the ColumnMapper wiring gap; AC-006 closes it post Stage 2 |

## Red Gate Tests (SAC-1 — tdd_mode: strict)

Note: specific test names below use `ocsf_field` as conceptual anchors. The exact names depend
on the quoting convention ADR decision (see §Background §Prerequisites). The test-writer must
finalize names once that decision is made. All six tests must be RED before T-08 begins.

- **RG-001:** `test_claroty_dtu_alerts_arrow_schema_field_names_equal_ocsf_field_values` —
  fails until `pipeline_result_to_record_batch` uses `ocsf_field` for claroty alerts table
  schema field names (AC-001)
- **RG-002:** `test_claroty_dtu_devices_arrow_schema_field_names_equal_ocsf_field_values` —
  fails until claroty devices table parity migration is complete (AC-001)
- **RG-003:** `test_crowdstrike_dtu_devices_arrow_schema_field_names_equal_ocsf_field_values` —
  fails until crowdstrike devices parity migration is complete (AC-002)
- **RG-004:** `test_crowdstrike_dtu_alerts_arrow_schema_field_names_equal_ocsf_field_values` —
  fails until crowdstrike alerts parity migration is complete (AC-002)
- **RG-005:** `test_armis_dtu_devices_arrow_schema_field_names_equal_ocsf_field_values` —
  fails until armis devices parity migration is complete (AC-003)
- **RG-006:** `test_cyberint_dtu_incidents_arrow_schema_field_names_equal_ocsf_field_values` —
  fails until cyberint incidents parity migration is complete (AC-004)

### BC-5.38.001 Density Check

Red Gate test count: 6 (RG-001..RG-006). Acceptance criteria directly driven by Red Gate
tests: 4 (AC-001..AC-004, one per sensor). AC-005 (integration tests) and AC-006 (conformance
update) may require additional RGTs once Stage 2 routing scope is finalized. Current density:
6 RGTs / 6 ACs = 1.0 ≥ 0.5 (compliant with BC-5.38.001). Density check will be revised when
the quoting convention ADR and Stage 2 routing story are in place.

## Acceptance Criteria

### AC-001: Claroty DTU parity tests use ocsf_field column names
Arrow RecordBatch schema field names asserted in parity tests for `prism-dtu-claroty` match
the `ocsf_field` values declared in `claroty.sensor.toml` for all tables (alerts, devices,
audit_log, device_alert_relations — whichever tables exist at implementation time).
(traces to BC-2.16.003 postcondition: `mapped_fields` keyed by `ocsf_field` path strings)

### AC-002: CrowdStrike DTU parity tests use ocsf_field column names
Arrow RecordBatch schema field names asserted in parity tests for `prism-dtu-crowdstrike`
match the `ocsf_field` values declared in the CrowdStrike sensor TOML spec for all tables.
(traces to BC-2.16.003 postcondition: `mapped_fields` keyed by `ocsf_field` path strings)

### AC-003: Armis DTU parity tests use ocsf_field column names
Arrow RecordBatch schema field names asserted in parity tests for `prism-dtu-armis` match
the `ocsf_field` values declared in the Armis sensor TOML spec for all tables.
(traces to BC-2.16.003 postcondition: `mapped_fields` keyed by `ocsf_field` path strings)

### AC-004: Cyberint DTU parity tests use ocsf_field column names
Arrow RecordBatch schema field names asserted in parity tests for `prism-dtu-cyberint` match
the `ocsf_field` values declared in the Cyberint sensor TOML spec for all tables.
(traces to BC-2.16.003 postcondition: `mapped_fields` keyed by `ocsf_field` path strings)

### AC-005: Integration tests in prism-bin migrated to ocsf_field column names
All integration tests in `prism-bin` that resolve sensor columns by name use `ocsf_field`
values for column lookups after Stage 2 routing lands. Column references in PrismQL strings
use the post-quoting-convention form (subject to the quoting convention ADR — see §Prerequisites).
(traces to BC-2.01.013 postcondition 1: every spec-declared column survives into the
RecordBatch with the correct type; after Stage 2, "column name" = ocsf_field value)

### AC-006: BC-2.01.013 EC-01-025 updated to CONFORMANT
The product-owner updates BC-2.01.013 EC-01-025 annotation from NON-CONFORMANT to CONFORMANT
after this story and the Stage 2 routing story both merge. The implementer confirms the
annotation is updated before closing this story.
(traces to BC-2.01.013 EC-01-025: ColumnMapper wiring gap closed after Stage 2)

## Architecture Mapping

| Component | Module | Pure/Effectful | Scope |
|-----------|--------|---------------|-------|
| DTU parity test suites | `crates/prism-dtu-{claroty,crowdstrike,armis,cyberint}/tests/` | Pure (test assertions) | Modified by this story — assertions migrate from col.name to ocsf_field names |
| `pipeline_result_to_record_batch` | `prism-bin::spec_driven_adapter` | Effectful (Arrow I/O) | Modified by Stage 2 routing story (not this story); this story writes tests that verify its output |
| `build_column_array` | `prism-bin::spec_driven_adapter` | Pure (data transformation) | Field name sourcing changes in Stage 2 routing story |
| `ColumnMapper::map_record` | `prism-spec-engine::column_mapping` | Pure | Called in Stage 2 production path; currently test-only |
| Integration tests | `crates/prism-bin/tests/` | Pure (test assertions) | Modified by this story — column name references migrated |

Architecture section files: `architecture/module-decomposition.md` (SS-07, SS-12),
`architecture/dependency-graph.md`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | A TOML column has no `ocsf_field` declared | Under Stage 2, column stays in `raw_extensions` blob; no Arrow schema field created for it. Parity test must NOT assert a schema field for this column. |
| EC-002 | Claroty `audit_log` table has different `ocsf_field` values from `alerts` and `devices` | Each table's parity test must use that table's own TOML `ocsf_field` declarations, not a shared list |
| EC-003 | A TOML `ocsf_field` value contains a dot (e.g., `"finding.uid"`) | The quoting convention ADR governs how this becomes a PrismQL / Arrow field name; the parity test must use the post-convention form (flattened or quoted), consistent with PrismQL query syntax |
| EC-004 | DTU generator produces a record key matching `col.name` but not `ocsf_field` | Under Stage 2 via ColumnMapper path: ColumnMapper reads by `col.name`, emits by `ocsf_field` — generator output is correct input. Parity test asserts the Arrow schema uses `ocsf_field`, which ColumnMapper ensures. |
| EC-005 | Stage 2 routing story is reverted or found non-viable | This story is blocked again; `status: draft` must be preserved; no code under this story should be merged without the Stage 2 routing story |
| EC-006 | Architect decides generators DO need output changes (see §Background §Scope Finding) | ACs AC-007..AC-010 must be added; points bump to 8; RG-007..RG-010 must be enumerated per SAC-1 |

## Token Budget Estimate

| Source | Estimated tokens |
|--------|-----------------|
| This story spec | ~4k |
| `spec_driven_adapter.rs` (pipeline_result_to_record_batch + build_column_array) | ~12k |
| `column_mapping.rs` (ColumnMapper::map_record) | ~3k |
| 4× DTU parity test files (~400 lines each) | ~6k |
| 4× sensor TOML specs (ocsf_field declarations) | ~4k |
| BC-2.16.003 + BC-2.01.013 | ~4k |
| ADR-058 | ~3k |
| **Total** | **~36k** |

36k tokens is well within a 200k agent context window (~18%). This story does NOT need splitting.

## Tasks

### Phase A: Prerequisites (human + architect + product-owner — NOT implementer)

- T-PREREQ-01: Quoting convention ADR decision (product-owner + architect) — must complete
  before T-01
- T-PREREQ-02: Stage 2 routing story created (story-writer) — wire `depends_on:` once ID exists
- T-PREREQ-03: Architect reconciles ADR-058 §C2 generator scope (architect amends ADR-058
  or confirms generators need no change) — must complete before T-01

### Phase B: Red Gate (test-writer dispatched BEFORE implementer — red-then-green)

- T-01: Read all 4 sensor TOML specs and extract `ocsf_field` values per table column (grounding)
- T-02: Write RG-001 — `test_claroty_dtu_alerts_arrow_schema_field_names_equal_ocsf_field_values` (failing)
- T-03: Write RG-002 — `test_claroty_dtu_devices_arrow_schema_field_names_equal_ocsf_field_values` (failing)
- T-04: Write RG-003 — `test_crowdstrike_dtu_devices_arrow_schema_field_names_equal_ocsf_field_values` (failing)
- T-05: Write RG-004 — `test_crowdstrike_dtu_alerts_arrow_schema_field_names_equal_ocsf_field_values` (failing)
- T-06: Write RG-005 — `test_armis_dtu_devices_arrow_schema_field_names_equal_ocsf_field_values` (failing)
- T-07: Write RG-006 — `test_cyberint_dtu_incidents_arrow_schema_field_names_equal_ocsf_field_values` (failing)
- T-GATE: Confirm all 6 Red Gate tests are failing (BC-5.38.001 density check ≥ 0.5)

### Phase C: Implementation (implementer dispatched after Red Gate)

- T-08: Migrate parity test assertions in `prism-dtu-claroty` from `col.name` to `ocsf_field`
  schema names for all tables (AC-001; makes RG-001 and RG-002 green)
- T-09: Migrate parity test assertions in `prism-dtu-crowdstrike` for all tables
  (AC-002; makes RG-003 and RG-004 green)
- T-10: Migrate parity test assertions in `prism-dtu-armis` for all tables
  (AC-003; makes RG-005 green)
- T-11: Migrate parity test assertions in `prism-dtu-cyberint` for all tables
  (AC-004; makes RG-006 green)
- T-12 (conditional): If T-PREREQ-03 concludes generators need output changes — update DTU
  generator record builders to emit `ocsf_field`-keyed records; otherwise skip
- T-13: Migrate integration tests in `prism-bin` that query sensor columns by name to use
  `ocsf_field` column identifiers per quoting convention ADR (AC-005)
- T-14: Run `just check` — all 6 RGTs must pass; no regressions in workspace
- T-15: Confirm with product-owner that BC-2.01.013 EC-01-025 annotation is updated from
  NON-CONFORMANT to CONFORMANT (AC-006 — product-owner step, not implementer)

## Previous Story Intelligence

This is a deferral record story authored before the upstream Stage 2 routing story exists.
No predecessor story in this epic has been implemented. This is the first story in EPIC-OCSF-ROUTING.

The Claroty live-API work (branch `fix/claroty-live-api-fidelity`, worktree `CLAROTY-LIVE`)
is Stage 1 (ADR-058 Interpretation B — coercion integration in `build_column_array` without
Arrow schema change). Stage 1 is INDEPENDENT of this story; Stage 1 landing does NOT unblock
this story. Only the Stage 2 routing story (not yet created) unblocks this.

Key precedent: `ColumnMapper::coerce_value` has the String-type-first rule (LIVE-DRIFT-003)
that normalizes integer inputs on String columns. The implementer must verify this rule
continues to fire correctly after Stage 2 wires `ColumnMapper::map_record` into production.

## Purity Classification

| Component | Classification | Rationale |
|-----------|---------------|-----------|
| DTU parity test functions | Pure | Test assertions are pure functions over in-memory data structures; no I/O |
| `ColumnMapper::map_record` | Pure | Takes `&Value` + `&TableSpec`, returns `MappingResult`; no I/O, no mutation |
| `build_column_array` (post-Stage-2) | Pure (data transformation) | Takes `&[Value]` + `&ColumnSpec`, returns `Arc<dyn Array>`; deterministic, no I/O |
| `pipeline_result_to_record_batch` | Effectful | Calls Arrow `RecordBatch::try_new` which performs schema validation (Arrow error path); I/O boundary is the RecordBatch materialization |

## Architecture Compliance Rules

From `architecture/module-decomposition.md`, ADR-023, and ADR-058:

1. `prism-sensors` MUST NOT import `prism-spec-engine` — parity test migration MUST NOT
   introduce this import. DTU crates (`prism-dtu-*`) are separate from `prism-sensors` and may
   import `prism-spec-engine` for schema assertion helpers if needed.
2. `pipeline_result_to_record_batch` lives in `prism-bin::spec_driven_adapter` — all Arrow
   schema field name logic is owned here. No other crate may control the field naming convention.
3. `ColumnMapper::map_record` lives in `prism-spec-engine` — Stage 2 production wiring calls
   it FROM `prism-bin` (one-way; ADR-023 §D3 crate boundary).
4. ADR-058 §C3 invariant: `ColumnMapper::map_record` remains in `prism-spec-engine` as the
   pure-core implementation. The Stage 2 routing story wires it; this story tests it post-wiring.
5. ADR-058 §D3 prerequisite: no code may hardcode dot-path column names without the quoting
   convention ADR decision. Parity tests must use the convention-compliant form.

## Library & Framework Requirements

No new dependencies are anticipated for parity test migration. The following workspace-pinned
versions apply (implementer MUST verify versions against `Cargo.toml` at dispatch time — do
NOT use versions from training data; workspace pins are the single source of truth):

| Library | Role | Constraint |
|---------|------|-----------|
| `arrow` | Arrow schema + RecordBatch assertions in parity tests | Workspace-pinned version in root `Cargo.toml` |
| `serde_json` | JSON record construction in test fixtures | Workspace-pinned version |

New `Cargo.toml` dependency entries added to any crate in this story MUST follow:
`default-features = false, features = ["rustls-tls"]` for `reqwest` (ADR-050).
No new `reqwest` dependencies are anticipated; this note is a standing compliance reminder.

## File Structure Requirements

Files to MODIFY only — do NOT create new files unless no existing test file exists:

| File | Action |
|------|--------|
| `crates/prism-dtu-claroty/tests/` (actual file TBD at dispatch) | Modify assertions: col.name → ocsf_field names |
| `crates/prism-dtu-crowdstrike/tests/` (actual file TBD at dispatch) | Modify assertions: col.name → ocsf_field names |
| `crates/prism-dtu-armis/tests/` (actual file TBD at dispatch) | Modify assertions: col.name → ocsf_field names |
| `crates/prism-dtu-cyberint/tests/` (actual file TBD at dispatch) | Modify assertions: col.name → ocsf_field names |
| `crates/prism-bin/tests/` (actual file TBD at dispatch) | Modify column name refs in integration tests |

The implementer must determine actual test file names at dispatch time via:
`find crates/prism-dtu-*/tests crates/prism-bin/tests -name "*.rs" -type f`

Do NOT modify: `column_mapping.rs`, `spec_driven_adapter.rs` (Stage 2 routing story scope),
any sensor TOML spec file (ocsf_field declarations are inputs), any BC or ADR file
(product-owner / architect scope).

## Forbidden Dependencies

Build-time enforcement rules:
- `prism-sensors` MUST NOT gain a dependency on `prism-spec-engine` — if `cargo tree -p prism-sensors` shows `prism-spec-engine` after this story, the story has introduced a forbidden import.
- `prism-dtu-*` crates MUST NOT depend on `prism-query` — DTU crates are test infrastructure and must not pull in the query engine.

## Notes for Implementer

At dispatch time, this story requires all §Background §Prerequisites to be complete. If
they are not, stop and report to the orchestrator — do NOT proceed with implementation
under uncertainty.

The SAC-1 Red Gate test names above contain `ocsf_field` as a conceptual placeholder. At
dispatch time, read the quoting convention ADR (a new ADR or BC amendment — see §Prerequisites)
and use the post-convention form. For example, if the convention is underscore-flattening,
`finding.uid` becomes `finding_uid` and the test name becomes
`test_claroty_dtu_alerts_arrow_schema_field_names_use_flattened_ocsf_paths`.

The scope finding in §Background states generators likely do NOT need to change. However,
the architect's ADR-058 §C2 says otherwise. Do NOT resolve this disagreement by assumption —
route to the orchestrator if the reconciliation (T-PREREQ-03) has not happened.

## References

- ADR-058 §C2 — Stage 2 prerequisites including DTU generator scope statement
- ADR-058 §D3 — Stage 2 prerequisite checklist
- BC-2.16.003 — Column-to-OCSF Mapping postconditions (ocsf_field key contract)
- BC-2.01.013 EC-01-025 — NON-CONFORMANT annotation to be closed by this story

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.0 | 2026-08-12 | story-writer | Initial deferral record — human-directed 2026-08-12. Establishes provenance, scope finding (generators vs. parity-tests-only dispute with architect), AC list, SAC-1 Red Gate list with density check, prerequisite checklist, and forbidden dependencies. ADR-058 v1.0 (accepted), BC-2.16.003 v1.4 (draft), BC-2.01.013 v1.16 (active) at authoring time. |
