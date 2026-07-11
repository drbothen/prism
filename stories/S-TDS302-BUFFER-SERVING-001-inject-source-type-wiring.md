---
document_type: story
story_id: "S-TDS302-BUFFER-SERVING-001"
title: "Wire inject_source_type into run_materialization_pipeline — unfence BC-2.11.012 'buffered' contract"
wave: maintenance
epic_id: maintenance
priority: P2
status: draft
version: "0.1"
spec_version: "v0.1"
level: ops
producer: story-writer
timestamp: "2026-07-11"
modified: "2026-07-11"
input-hash: ""
inputs:
  - crates/prism-query/src/materialization.rs
  - crates/prism-query/src/virtual_fields.rs
  - crates/prism-query/src/types.rs
  - .factory/specs/behavioral-contracts/BC-2.11.012-virtual-fields.md
origin_finding: "TD-S302-005 (buffer-serving story queue); BC-2.11.012 v1.8 fence removal"
origin_cascade: "DEFECT-CSDEVICES-EMPTY-PIPELINE-001 LOCAL pass-20 F-CSD-P20-005; D-1669 (2026-07-10)"
cycle: "v1.0.0-greenfield"
phase: 3
tdd_mode: strict
track: "Platform Engineering"
subsystems: [SS-11]
crates_touched:
  - prism-query
target_module: "crates/prism-query/src/materialization.rs"
behavioral_contracts:
  - BC-2.11.012
# BC status: BC-2.11.012 v1.11 is active.
# BC-2.11.012 v1.8 added fence annotations on EC-11-032/033 ("buffered" value unreachable
# end-to-end; inject_source_type has zero production callers).
# This story delivers the wiring that makes EC-11-032/033 testable end-to-end within
# the pipeline's existing JSON layer, unfencing those ECs.
# No BC amendment is strictly required before dispatch — the story implements what
# BC-2.11.012 already specifies in AC-9/AC-10 and §inject_source_type semantics.
# A BC-2.11.012 version bump to remove the fence annotations is expected as a
# co-deliverable (implementer amends the fence comments in the same PR).
# AC↔BC bidirectional traces required before status=ready (S-7.01).
verification_properties: []
depends_on: []
blocks: []
points: 5
estimated_days: 1.5
risk: LOW
acceptance_criteria_count: 4
red_gate_tests: 4
estimated_passes: "2-3"
holdout_scenarios: []
assumption_validations: []
risk_mitigations: []
---

# S-TDS302-BUFFER-SERVING-001: Wire inject_source_type into run_materialization_pipeline — unfence BC-2.11.012 "buffered" contract

## §Origin — [tech-debt] TD-S302-005 + BC-2.11.012 v1.8 fence

**Cascade:** DEFECT-CSDEVICES-EMPTY-PIPELINE-001 LOCAL pass-20 F-CSD-P20-005; D-1669 (2026-07-10)
**Unblocked by:** PR #221 develop@5f1b5771 (2026-07-11)

BC-2.11.012 v1.8 (F-CSD-P20-005 adjudication) fenced the `"buffered"` contract:
> "EventStream buffer-serving path does NOT exist in production; inject_source_type fencing CORRECT;
> 'buffered' condition gated behind TD-S302-005; EC-11-032/033 annotated untestable-end-to-end
> pending TD-S302-005."

The fence was correct at the time: `inject_source_type` in
`crates/prism-query/src/materialization.rs` has zero production callers.
`inject_virtual_fields` in `crates/prism-query/src/virtual_fields.rs` unconditionally
injects `"live"` for `_source_type` regardless of whether rows came from a buffer.

This story closes TD-S302-005 by wiring `inject_source_type` into
`run_materialization_pipeline`. The deliverable is the **correct injection plumbing**,
not a full RocksDB EventBuffer integration (that requires a separate story once the
EventBufferStore infrastructure exists). The wiring must ensure:
1. `inject_source_type` is called on JSON rows before Arrow conversion for every fan-out target
2. `inject_virtual_fields` uses the derived `source_type` value rather than hardcoding `"live"`
3. EC-11-032/033 are testable via unit/integration tests exercising `inject_source_type`
   with `rows_from_buffer = true` descriptors through the pipeline

**Decision sites annotated with `TD-S302-005` comments (enumerate all TD-S302-005 cite
sites and update them in this story's PR):**
- `crates/prism-query/src/materialization.rs` module docstring (fence annotation, lines 10–12)
- `crates/prism-query/src/materialization.rs` `inject_source_type` doc comment (lines 73–79)
- `crates/prism-query/src/materialization.rs` `#[allow(clippy::ptr_arg)]` comment (line 88)
- `crates/prism-query/src/virtual_fields.rs` `inject_virtual_fields` doc comment (lines 73–79)
- `crates/prism-query/src/virtual_fields.rs` `source_type_array` construction (lines 109–110)
- `crates/prism-query/src/lib.rs` module-level comment (line 3)

All six sites must be updated to reflect the unfenced state — either removing the fence
comment or replacing it with a note pointing at the future EventBufferStore story
(`S-EVENTBUFFER-001` or equivalent to-be-named) for the `rows_from_buffer = true` path.

## Behavioral Contracts

| BC | Title | Version | Relevance |
|----|-------|---------|-----------|
| BC-2.11.012 | Virtual Fields in Queries — `_sensor`, `_client`, `_source_table`, `_source_type` | v1.11 | AC-9: EventStream rows from buffer → `"_source_type": "buffered"`. AC-10: PointInTime or EventStream cold-start → `"_source_type": "live"`. EC-11-032 and EC-11-033 are the edge cases gated behind TD-S302-005. This story delivers the wiring that unfences them. |

## Acceptance Criteria

### AC-001 — `inject_source_type` is called in `run_materialization_pipeline` for every per-target fan-out result
(traces to BC-2.11.012 v1.11 AC-9/AC-10: "inject_source_type sets `_source_type` on each row map based on `EventStream`/`PointInTime` delivery model and whether rows came from the buffer")

`run_materialization_pipeline` constructs a `SensorQueryDescriptor` for each fan-out
target and calls `inject_source_type` on the raw `Vec<serde_json::Value>` rows BEFORE
they are normalized to Arrow RecordBatches. The `SensorQueryDescriptor` is populated as:
- `table_name`: the fan-out `source_table` name
- `table_type`: derived from the `TableRegistry` or spec for the table (defaults to
  `TableType::PointInTime` when not specified; `EventStream` only for tables declared as
  such in the sensor TOML spec)
- `rows_from_buffer`: `false` for all current production paths (EventBuffer store is a
  future story); `true` when the future EventBufferStore integration sets it

A Red Gate test `test_BC_2_11_012_AC9_run_pipeline_inject_source_type_wiring` exercises
`run_materialization_pipeline` with a mock adapter that returns rows under a descriptor
with `table_type = EventStream, rows_from_buffer = true` and asserts that the resulting
RecordBatch `_source_type` column contains `"buffered"` values.

### AC-002 — `inject_virtual_fields` uses the `source_type` derived from the descriptor, not a hardcoded `"live"` constant
(traces to BC-2.11.012 v1.11 AC-10: "PointInTime table or EventStream cold-start → `_source_type = 'live'`"; and AC-9: "EventStream rows_from_buffer=true → `_source_type = 'buffered'`")

`inject_virtual_fields` in `crates/prism-query/src/virtual_fields.rs` is modified to
accept a `source_type: &str` parameter. The hardcoded `vec!["live"; num_rows]` is
replaced with `vec![source_type; num_rows]`. All call sites in
`run_materialization_pipeline` pass the `source_type` value derived from the
`SensorQueryDescriptor` constructed per target.

A Red Gate test `test_BC_2_11_012_AC9_inject_virtual_fields_accepts_buffered_source_type`
asserts that calling `inject_virtual_fields(batch, sensor, client, table, "buffered")`
(with the new signature) produces a RecordBatch where the `_source_type` column contains
`"buffered"` for every row.

### AC-003 — EC-11-032 is now testable: `_source_type = "buffered"` filter on PointInTime table returns empty result
(traces to BC-2.11.012 v1.11 EC-11-032: "`_source_type = 'buffered'` on a PointInTime-only table → no records match; value is always `'live'` for PointInTime tables")

After the wiring in AC-001 and AC-002, EC-11-032 is testable. A test
`test_BC_2_11_012_EC11_032_buffered_filter_on_point_in_time_returns_empty` constructs a
materialization scenario where the source table is `PointInTime`, the adapter returns
rows, and the query includes `WHERE _source_type = 'buffered'`. The result set must be
empty (all rows carry `"live"`, which does not match the `"buffered"` filter predicate).
The test annotation must NOT reference EC-11-032's prior fence comment; the EC-11-032
fence annotation in BC-2.11.012 is removed as part of this story.

### AC-004 — EC-11-033 is now testable: EventStream cold-start (rows_from_buffer=false) produces `_source_type = "live"`
(traces to BC-2.11.012 v1.11 EC-11-033: "`_source_type = 'buffered'` on EventStream table with empty buffer → no records match; cold-start falls back to live fetch")

A test `test_BC_2_11_012_EC11_033_eventstream_cold_start_produces_live_source_type`
constructs a materialization scenario where the source table has `table_type = EventStream`
but `rows_from_buffer = false` (cold-start live fallback). The result rows carry
`"_source_type": "live"`. The EC-11-033 fence annotation in BC-2.11.012 is removed as
part of this story.

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `inject_source_type` | `crates/prism-query/src/materialization.rs` | Pure (JSON mutation, no I/O) |
| `inject_virtual_fields` | `crates/prism-query/src/virtual_fields.rs` | Pure (Arrow RecordBatch construction) |
| `run_materialization_pipeline` | `crates/prism-query/src/materialization.rs` | Effectful (async, sensor I/O) |
| `SensorQueryDescriptor` | `crates/prism-query/src/types.rs` | Pure data |

Architecture section references:
- `architecture/module-decomposition.md` §SS-11 PrismQL Query Engine
- S-2.08 (inject_source_type authorship) and S-3.02 (run_materialization_pipeline authorship)

**Anchor justifications (POL-4/POL-5):**
- SS-11 owns this story's scope because `inject_source_type`, `inject_virtual_fields`, and `run_materialization_pipeline` are all SS-11 (PrismQL Query Engine) artifacts per ARCH-INDEX Subsystem Registry.
- No `depends_on` dependencies: this story is unblocked by PR #221 and has no other predecessor stories.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `rows_from_buffer = true` on a `PointInTime` table | AC-10 governs: PointInTime tables always yield `"live"`. The `inject_source_type` contract only sets `"buffered"` when BOTH `table_type == EventStream` AND `rows_from_buffer == true`. PointInTime with `rows_from_buffer = true` is a logical impossibility but must be handled gracefully: always `"live"`. |
| EC-002 | `inject_virtual_fields` called with `source_type = "buffered"` but rows contain a spoofed `_source_type` field | `remove_spoofed_virtual_columns` strips the spoofed field first; the correct `source_type` parameter value is injected. Spoofed fields cannot override the descriptor-derived value. |
| EC-003 | Empty `Vec<serde_json::Value>` passed to `inject_source_type` | No-op; `rows` is empty; no error. |
| EC-004 | Fan-out target with unknown `table_type` (not in `TableRegistry`) | Defaults to `PointInTime` → `"live"`. Conservative default is correct. |
| EC-005 | Future EventBufferStore story sets `rows_from_buffer = true` for a real EventStream table | `inject_source_type` stamps `"buffered"`; `inject_virtual_fields` injects `"buffered"` in the Arrow column. EC-11-032/033 end-to-end tests become gated on EventBuffer store readiness but the plumbing is correct. |

## Token Budget Estimate

| Item | Lines | Tokens (est.) |
|------|-------|--------------|
| Story spec (this file) | ~140 | ~1,960 |
| BC-2.11.012 (virtual fields BC, ~150 lines relevant section) | ~150 | ~2,100 |
| `crates/prism-query/src/materialization.rs` (inject_source_type + run_materialization_pipeline fan-out loop, ~200 lines) | ~200 | ~2,800 |
| `crates/prism-query/src/virtual_fields.rs` (inject_virtual_fields, ~150 lines) | ~150 | ~2,100 |
| `crates/prism-query/src/types.rs` (SensorQueryDescriptor, ~60 lines) | ~60 | ~840 |
| BC files (1 BC) | — | ~2,100 |
| **Total estimate** | | **~11,900 tokens** |

Fits within a 100k-token agent context window (~12%). No split required.

## Tasks

- [ ] Read `inject_source_type` in `crates/prism-query/src/materialization.rs` and confirm the `SensorQueryDescriptor.rows_from_buffer` semantic.
- [ ] Read `inject_virtual_fields` in `crates/prism-query/src/virtual_fields.rs` to understand the `remove_spoofed_virtual_columns` flow and the hardcoded `"live"` site.
- [ ] Read `run_materialization_pipeline`'s fan-out loop (the site where `inject_virtual_fields` is currently called) to understand where to insert the `inject_source_type` call.
- [ ] Write Red Gate test AC-001 (`test_BC_2_11_012_AC9_run_pipeline_inject_source_type_wiring`) — RED.
- [ ] Write Red Gate test AC-002 (`test_BC_2_11_012_AC9_inject_virtual_fields_accepts_buffered_source_type`) — RED.
- [ ] Write Red Gate test AC-003 (`test_BC_2_11_012_EC11_032_buffered_filter_on_point_in_time_returns_empty`) — RED.
- [ ] Write Red Gate test AC-004 (`test_BC_2_11_012_EC11_033_eventstream_cold_start_produces_live_source_type`) — RED.
- [ ] Change `inject_virtual_fields` signature to accept `source_type: &str` parameter (AC-002).
- [ ] Update all call sites of `inject_virtual_fields` in `run_materialization_pipeline` to pass the derived `source_type` value.
- [ ] Construct `SensorQueryDescriptor` per target in `run_materialization_pipeline` and call `inject_source_type` on JSON rows before OcsfNormalizer (AC-001).
- [ ] Remove or update all six TD-S302-005 fence comments from `materialization.rs`, `virtual_fields.rs`, and `lib.rs`.
- [ ] Run `just iter prism-query --no-fail-fast` to verify all tests GREEN.
- [ ] Run `just check` (full workspace) before declaring done.
- [ ] Report to orchestrator if the `table_type` derivation requires accessing `TableRegistry` inside the fan-out loop — this may need plumbing that currently isn't threaded through. See OQ-001.

## Previous Story Intelligence

**Prior story context:**
- `S-2.08` authored `inject_source_type` (pure function, unit-tested) and declared it ready to be wired.
- `S-3.02` authored `run_materialization_pipeline` but shipped the pipeline without calling `inject_source_type` — the TD-S302-005 fence was placed at that time with the intent to wire in a follow-up story.
- `DEFECT-CSDEVICES-EMPTY-PIPELINE-001` pass-20 (D-1669) adjudicated the fence: BC-2.11.012 v1.8 confirmed the fence is correct in the interim; EC-11-032/033 annotated untestable-end-to-end; the story was queued.
- The test file `crates/prism-query/src/tests/materialization_tests.rs` already has unit tests for `inject_source_type` at the function level (AC-9: `rows_from_buffer=true → "buffered"`, AC-10: `rows_from_buffer=false → "live"`, PointInTime always "live"). This story adds WIRING tests that exercise `inject_source_type` through the pipeline.

**Lesson from S-3.02:** The `inject_virtual_fields` spoofing guard (`remove_spoofed_virtual_columns`) strips ALL fields whose names match virtual field names. If `inject_source_type` stamps `"_source_type"` at the JSON layer and the JSON→Arrow normalization preserves it as a field, `remove_spoofed_virtual_columns` would strip it and `inject_virtual_fields` would re-inject "live". To avoid this, the architecture should NOT rely on passing `_source_type` through the JSON/Arrow boundary. Instead, derive `source_type` from the `SensorQueryDescriptor` and pass it directly as a parameter to `inject_virtual_fields` (AC-002 approach).

## Architecture Compliance Rules

- **BC-2.11.012 v1.11 §Invariants:** The four virtual fields (`_sensor`, `_client`, `_source_table`, `_source_type`) must be present in every sensor table RecordBatch. `inject_virtual_fields` remains the authoritative injection point; `inject_source_type` provides the `source_type` VALUE, not a second injection mechanism.
- **BC-2.11.012 v1.11 EC-002 (spoofing guard):** Sensor data cannot inject a real `_source_type` value. The spoofing guard in `remove_spoofed_virtual_columns` MUST remain in place — only the descriptor-derived value may appear in the final RecordBatch.
- **Fence removal discipline (TD-VSDD-091):** When removing fence comments, do NOT leave stale line-number references. Replace fence comments with forward-looking references to `S-EVENTBUFFER-001` (or the as-yet-unnamed EventBufferStore story) for the `rows_from_buffer = true` production path. Use function-name anchors, not file/line cites.
- **No `println!`:** Use `tracing::*!` for any diagnostic output inside the materialization pipeline.
- **`inject_virtual_fields` signature change is a breaking change within `prism-query`:** Run `grep -rn "inject_virtual_fields" crates/` after the change and update all call sites. The function is `pub(crate)` — no external callers, but intra-crate call sites must be updated (TD-VSDD-060 sibling-site sweep).

## Library & Framework Requirements

| Library | Version | Notes |
|---------|---------|-------|
| `prism-core` | workspace-pinned | `TableType` enum (`PointInTime`, `EventStream`) — used in `SensorQueryDescriptor` |
| `nextest` | workspace-pinned | `just iter prism-query` for fast inner loop |

No new external dependencies required.

**Forbidden dependencies:** `prism-query` must not gain a dependency on
`prism-dtu-harness` or any buffer-store crate as part of this story. The
`SensorQueryDescriptor.rows_from_buffer` flag is set to `false` for all current
production paths; the flag contract is already in place in `types.rs`.

## File Structure Requirements

| File | Action | Notes |
|------|--------|-------|
| `crates/prism-query/src/materialization.rs` | Modify | Wire `inject_source_type` call in fan-out loop; remove/update TD-S302-005 fence annotations |
| `crates/prism-query/src/virtual_fields.rs` | Modify | Add `source_type: &str` parameter to `inject_virtual_fields`; replace hardcoded `"live"` with `source_type` parameter; update TD-S302-005 fence comment |
| `crates/prism-query/src/lib.rs` | Modify | Remove TD-S302-005 fence annotation in module-level comment |
| `crates/prism-query/src/tests/materialization_tests.rs` | Modify | Add 4 Red Gate tests (AC-001 through AC-004) |
| `.factory/specs/behavioral-contracts/BC-2.11.012-virtual-fields.md` | Modify | Remove EC-11-032/EC-11-033 fence annotations; bump version to reflect fence removal |

No new files. No Cargo.toml changes.

## Open Questions

### OQ-001 — `table_type` derivation per fan-out target (potential plumbing gap)
To construct `SensorQueryDescriptor` for each fan-out target, `run_materialization_pipeline`
needs to know the `TableType` (PointInTime vs EventStream) for each source table. This
information exists in the `TableRegistry` (`mat_ctx.table_registry`) if the spec declares
a `table_type` per table. The implementer must confirm whether:
a) `TableRegistry` already exposes `table_type` per table name, OR
b) The current `TableRegistry` does not track `table_type` and additional plumbing is required

If (b), the implementer should either:
- Add a `table_type_for_table(table_name: &str) -> TableType` method to `TableRegistry` (small in-scope addition), OR
- Default all tables to `PointInTime` and note the EventStream table_type gap in the story PR as a follow-up item

For the purpose of this story, `rows_from_buffer = false` regardless of `table_type`,
so the default-to-PointInTime approach is safe for production. The correct `table_type`
is needed only to correctly stamp future EventStream buffered rows.

### OQ-002 — BC-2.11.012 fence annotation removal scope
The BC-2.11.012 fence on EC-11-032/033 was added at v1.8. Removing the fence requires
a BC version bump (v1.11 → v1.12 or equivalent). The implementer should either:
a) Make the BC amendment inline as part of this story's PR (product-owner will review),
   OR
b) Flag the BC amendment to the orchestrator for product-owner routing before merge

Per CLAUDE.md Canonical Principle Rule 4, AI agents discovering a gap fix it in scope.
The BC fence removal is in-scope for the implementing story PR (the BC was written to
be unfenced when the story delivers).
