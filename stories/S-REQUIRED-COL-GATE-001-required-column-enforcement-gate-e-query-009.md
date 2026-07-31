---
document_type: story
story_id: S-REQUIRED-COL-GATE-001
title: "prism-query: Required-Column Enforcement Gate — E-QUERY-009 at Query Plan Time (BC-2.11.007 §REQUIRED Column Runtime Mechanism / DI-021)"
# wave: not yet wave-scheduled; state-manager assigns wave at scheduling time
epic_id: E-QUERY
priority: P1
# Priority rationale: BC-2.11.007 §REQUIRED Column Runtime Mechanism is a P0 active contract
# (DI-021) whose plan-time E-QUERY-009 gate has ZERO wiring in the pre-fan-out execution path.
# The wave-A E-SPEC-029 backstop (required_filters on FetchStep §FetchStep, ADR-057 §D7) is
# the only active enforcement today; E-QUERY-009 at query plan time is an OPEN OBLIGATION
# recorded in ADR-057 §D7 Mechanism Layering. P1 because E-SPEC-029 covers the immediate
# wave-A safety need; the plan-time gate is required for full DI-021 compliance.
status: draft
# Spec-First Gate (S-7.01): behavioral_contracts is non-empty; status MUST remain draft until
# product-owner has authored holdout scenarios at materialization time (POL-35).
# BC-2.11.007 is v1.9 active — BCs are authored; holdout authorship is pending (see below).
version: "1.1"
level: "L3"
producer: story-writer
timestamp: "2026-07-31T00:00:00Z"
created: "2026-07-31"
modified: "2026-07-31"
phase: 3
tdd_mode: strict
subsystems: [SS-11]
# Subsystem anchor justification:
#   SS-11 (Query Execution) owns run_materialization_pipeline in prism-query/src/materialization.rs
#   — the pre-fan-out execution path where the E-QUERY-009 plan-time gate fires. It also owns
#   classify_predicates in prism-query/src/pushdown.rs, the classification function this story
#   wires into the pre-fan-out stage. Per ARCH-INDEX Subsystem Registry §SS-11.
target_module: prism-query
crates_touched: [prism-query, prism-core]
# crates_touched notes:
#   prism-query: materialization.rs (pre-fan-out gate placement), pushdown.rs (classify_predicates
#     wiring or dedicated enforce_required_columns helper)
#   prism-core: error.rs (conditional: verify PrismError variant for E-QUERY-009 exists; add if absent)
capabilities: [CAP-015]
behavioral_contracts:
  - BC-2.11.007  # Sensor Filter Push-Down — §REQUIRED Column Runtime Mechanism (DI-021
                 # enforcement: E-QUERY-009 before any API calls); §Error Cases (E-QUERY-009
                 # structured error contract); §Invariants INV-REQUIRED-SPECDRIVEN
# BC anchor justification:
#   BC-2.11.007 §REQUIRED Column Runtime Mechanism (## heading, verified) is the SOLE behavioral
#   contract for this story. It defines: (1) ColumnOptions::Required as SoT for REQUIRED
#   classification; (2) the ConfigSnapshot.sensor_specs[sensor_id] lookup path for ColumnSpec at
#   plan time; (3) the E-QUERY-009 error structure; (4) INV-REQUIRED-SPECDRIVEN invariant.
#   All four ACs trace to this BC — no other BC needed for this targeted gate story.
# BC bidirectional trace check (pre-ready gate):
#   BC-2.11.007 appears in body §Behavioral Contracts table (row below).
#   BC-2.11.007 is cited in AC-001, AC-002, AC-003, AC-004 trace lines.
verification_properties: [VP-031]
# VP-031: "Required Column Enforcement — Rejects Unconstrained Queries" (proptest).
# Anchor story is S-3.02; this story exercises VP-031 as a regression gate (AC-004).
# VP-031 proof_completed_date is null (S-3.02 partial-merge); if not yet implemented,
# AC-004/RG-004 implements it here.
assumption_validations: []
risk_mitigations:
  - "ADR-057 §D7 defense-in-depth MUST be preserved: E-SPEC-029 (required_filters on
    FetchStep §FetchStep — wave-A active, ADR-057 §D7) is NOT retired by this story. When the
    plan-time E-QUERY-009 gate lands, E-QUERY-009 fires first; E-SPEC-029 remains as the
    step-execution backstop. Do NOT remove or weaken required_filters logic while adding the
    plan-time gate."
  - "VP-031 regression: after wiring, run the VP-031 proptest. Any proptest failure indicates
    the wiring is incorrect — do not suppress the failure or comment out the test."
  - "resolved_spec_map None safe-default: if MaterializationContext.resolved_spec_map is None
    at the gate call site, the gate MUST NOT fire E-QUERY-009 — fall through to step-level
    enforcement only. Mirrors the ADR-033 T1 precedent for resolved_spec_map None handling."
  - "No hardcoded column-name list: the set of REQUIRED columns is determined exclusively by
    ColumnOptions::Required in the loaded ColumnSpec per INV-REQUIRED-SPECDRIVEN. Any hardcoded
    column name (e.g., 'device_id') in the gate logic is a defect."
depends_on:
  - S-3.02  # classify_predicates (prism-query/src/pushdown.rs) is defined in S-3.02.
            # Dependency anchor: classify_predicates §classify_predicates takes ColumnSpec
            # slices and already handles REQUIRED column classification; S-3.02 must have this
            # function available (not todo!()) before this story dispatches. S-3.02 is currently
            # partial-merge — implementer MUST verify classify_predicates is implemented in the
            # pushdown.rs component (not remaining as todo!()) before proceeding.
blocks: []
points: 5
# Points justification:
#   - Pre-fan-out REQUIRED column check placement in run_materialization_pipeline
#     (materialization.rs): 1.5 pts
#   - classify_predicates wiring — ColumnSpec lookup from resolved_spec_map per
#     BC-2.11.007 §REQUIRED Column Runtime Mechanism + enforce_required_columns helper: 1 pt
#   - E-QUERY-009 error construction: structured fields (sensor name, required_columns list,
#     example WHERE clause), wired to E-QUERY-009 error-taxonomy row: 1 pt
#   - Red Gate suite (4 tests): 1 pt
#   - ADR-057 §D7 defense-in-depth verification (E-SPEC-029 preserved): 0.5 pts
estimated_days: 2
risk: LOW
acceptance_criteria_count: 4
red_gate_tests: 4
estimated_passes: "2 LOCAL adversary passes"
holdout_scenarios: []
# POL-35: BC-2.11.007 is a non-empty behavioral_contracts entry — holdout scenarios are
# REQUIRED. `holdout_scenarios: []` is NOT compliant for a story with active BCs.
# Product-owner MUST author 2-4 HIDDEN, SINGLE-USE holdout scenarios at story materialization
# time (before status transitions to ready). Scenarios must NOT be shared with test-writer
# or implementer (contamination control per D-1715/D-1716 story-level holdout gate).
# Suggested scenario themes: (a) sensor with two REQUIRED columns, query satisfies only one;
# (b) multi-table query where one table has REQUIRED constraint and one does not;
# (c) E-QUERY-009 message contents and structured fields validation.
# # BC status: pending PO holdout authorship
inputs:
  - ".factory/specs/behavioral-contracts/BC-2.11.007-sensor-filter-push-down.md"
  - ".factory/specs/architecture/decisions/ADR-057-armis-activity-per-device-push-down-grammar.md"
  - ".factory/specs/architecture/decisions/ADR-033-push-down-time-window-extraction-strategy-pre-fan-out-heuristic.md"
  - ".factory/specs/prd-supplements/error-taxonomy.md"
  - "crates/prism-query/src/materialization.rs"
  - "crates/prism-query/src/pushdown.rs"
  - "crates/prism-core/src/error.rs"
  - "crates/prism-sensors/specs/armis.sensor.toml"
input-hash: null
traces_to: []
cycle: "v1.0.0-brownfield"
---

# S-REQUIRED-COL-GATE-001 — Required-Column Enforcement Gate (E-QUERY-009 at Query Plan Time)

**Story ID:** S-REQUIRED-COL-GATE-001
**Status:** draft
**Version:** v1.1
**Priority:** P1
**Points:** 5

---

## Authority

**BC-2.11.007 §REQUIRED Column Runtime Mechanism** is the behavioral contract governing
the DI-021 enforcement obligation this story implements: queries missing a REQUIRED column
constraint are rejected with E-QUERY-009 before any API calls. §Error Cases (E-QUERY-009
row) defines the structured error shape. §Invariants INV-REQUIRED-SPECDRIVEN establishes
that the REQUIRED column set is spec-driven only, with no hardcoded names.

Read BC-2.11.007 at:
`.factory/specs/behavioral-contracts/BC-2.11.007-sensor-filter-push-down.md`

**ADR-057 §D7 Mechanism Layering** is the architectural authority for the two-tier
defense-in-depth design this story participates in: the E-QUERY-009 plan-time gate
(Tier 1, this story) fires before the E-SPEC-029 step-execution gate (Tier 2,
`S-WAVE-A-ARMIS-ACTIVITY-001`). Neither tier retires the other. The tier responsibility
division — including the empty-value arm adjudication
(DRIFT-S-REQUIRED-COL-GATE-001-EMPTYVAL-001) — is documented in ADR-057 §D7.

Read ADR-057 at:
`.factory/specs/architecture/decisions/ADR-057-armis-activity-per-device-push-down-grammar.md`

---

## Origin

This story closes the OPEN OBLIGATION recorded in `ADR-057 §D7 Mechanism Layering` (Mechanism
A column, Status "OPEN OBLIGATION — future story not yet created" as of 2026-07-31). It is the
real story anchor replacing the wave-granularity deferral language in `S-DEMO-QUERY-PUSHDOWN-001`
(Out-of-Scope table) and `S-QUERY-SCOPE-PARAMS-001` (Out of Scope item 1).

**ADR-033 T2 adjudication (story-writer, FB104):** This story covers the REQUIRED-column
plan-time enforcement gate only. ADR-033 T2 describes FULL per-sensor post-resolution
`classify_predicates` integration via fan-out orchestration restructure — a separate concern.
ADR-057 §D7 characterized T2 as a prerequisite, but ground truth in ADR-033 §Decision confirms
`MaterializationContext.resolved_spec_map` is already accessible at the pre-fan-out stage (T1
already reads ColumnSpec from it for datetime column lookup). Therefore the E-QUERY-009 gate
is self-sufficient: it reads per-sensor `ColumnSpec` from `resolved_spec_map` using the same
access pattern T1 established, without restructuring fan-out orchestration. Full T2 (broad
push-down optimization across non-REQUIRED dimensions) remains a separate future story not yet
scoped. **One story is correct here; no T2 prerequisite story is needed.**

---

## Narrative

As the Prism query engine, I want to reject queries that omit a REQUIRED column constraint for
their target sensor BEFORE any API calls are issued — with a structured E-QUERY-009 error naming
the sensor, the required columns, and an example WHERE clause — so that the DI-021 enforcement
contract in BC-2.11.007 §REQUIRED Column Runtime Mechanism is satisfied at query plan time and
analysts receive actionable guidance rather than silent empty results or step-level errors.

---

## Behavioral Contracts

| BC ID | Version at authoring | Title | Role |
|-------|---------------------|-------|------|
| BC-2.11.007 | v1.9 (active) | Sensor Filter Push-Down | Sole anchor. §REQUIRED Column Runtime Mechanism defines the DI-021 enforcement path: queries missing a REQUIRED column constraint are rejected with E-QUERY-009 before any API calls. §Error Cases E-QUERY-009 row defines the structured error shape. §Invariants INV-REQUIRED-SPECDRIVEN: REQUIRED column set is spec-driven only, no hardcoded names. |

---

## Acceptance Criteria

### AC-001: Missing REQUIRED column on `armis_device_activity` returns E-QUERY-009 before any API call

**Given:** A PrismQL query `FROM armis_device_activity` with NO `WHERE device_id = '...'`
predicate, and `armis.sensor.toml` declares `device_id` with `options = ["REQUIRED"]` on the
`device_activity` table's column definition.

**When:** The query passes through `run_materialization_pipeline`'s plan-time REQUIRED-column
gate.

**Then:**
(a) `E-QUERY-009` is returned before any HTTP request is issued to the Armis sensor or DTU.
(b) The error's structured fields include: the sensor name (`"armis"`), the missing required
    column name (`"device_id"`), and example WHERE clause syntax (`WHERE device_id = '...'`).
(c) The error code in the Display string matches the `error-taxonomy.md §QUERY table` E-QUERY-009
    row's Message Format: `"Required column constraint violation for armis: columns [device_id]
    must be constrained in WHERE clause"`.
(d) `E-QUERY-009` fires at plan time (from the materialization gate), NOT as the step-level
    `E-SPEC-029` (which fires later in `execute_impl §execute_impl`). Both codes are correct at
    their respective layers; this AC verifies the plan-time layer fires first.

*(traces to BC-2.11.007 §REQUIRED Column Runtime Mechanism — enforcement: rejected with E-QUERY-009 before any API calls; §Error Cases E-QUERY-009 row — structured error fields)*

### AC-002: Query with `WHERE device_id = 'abc'` passes the enforcement gate

**Given:** A PrismQL query `FROM armis_device_activity WHERE device_id = 'abc'`, with
`armis.sensor.toml` declaring `device_id` with `options = ["REQUIRED"]`.

**When:** The query passes through `run_materialization_pipeline`'s plan-time REQUIRED-column
gate.

**Then:**
(a) No E-QUERY-009 error is returned by the gate; the query proceeds past the plan-time check.
(b) The gate correctly recognizes that the `device_id` REQUIRED column IS constrained (equality
    predicate `=`) and allows execution to continue to fan-out.
(c) The step-level E-SPEC-029 gate (in `execute_impl §execute_impl`) remains reachable as
    defense-in-depth; AC-002 does not test step-level behavior.

*(traces to BC-2.11.007 §REQUIRED Column Runtime Mechanism — positive path: query proceeds when REQUIRED constraint is present; §Invariants INV-REQUIRED-SPECDRIVEN — classification is spec-driven)*

### AC-003: Sensor with no REQUIRED columns never triggers E-QUERY-009 regardless of WHERE contents

**Given:** A sensor spec where NO `[[tables.columns]]` entry carries `options = ["REQUIRED"]`
(e.g., a test fixture sensor spec with all columns as INDEX/DEFAULT only), and any PrismQL
query against that sensor — including queries with no WHERE clause at all.

**When:** The query passes through the plan-time REQUIRED-column gate.

**Then:**
(a) E-QUERY-009 is NEVER returned, regardless of what (if any) predicates the query contains.
(b) The gate performs a spec-driven check: it reads REQUIRED columns from the ColumnSpec via
    `ColumnOptions::Required` entries in the loaded spec — not from a hardcoded column name list.
(c) An empty REQUIRED columns set (spec declares none) results in the gate being a no-op; no
    penalty for sensors that make all columns optional.

*(traces to BC-2.11.007 §Invariants INV-REQUIRED-SPECDRIVEN — "determined exclusively by ColumnOptions::Required entries in the loaded ColumnSpec, not by any hardcoded name list"; §REQUIRED Column Runtime Mechanism — spec-driven lookup via ConfigSnapshot.sensor_specs[sensor_id])*

### AC-004: VP-031 proptest passes after wiring — REQUIRED columns always produce Push classification in classify_predicates

**Given:** The `classify_predicates §classify_predicates` function in
`crates/prism-query/src/pushdown.rs` is called with any ColumnSpec where one or more columns
carry `ColumnOptions::Required`, and a WHERE predicate set that does NOT constrain any REQUIRED
column.

**When:** The VP-031 proptest (`vp031_pushdown.rs` proptest or equivalent) runs against the
wired code.

**Then:**
(a) `classify_predicates` returns a classification consistent with the REQUIRED column set for
    the query (REQUIRED column present without constraint → the plan should reflect an error
    or push-down obligation).
(b) The VP-031 proptest passes: for any REQUIRED column from any sensor spec,
    `classify_predicates` classifies the column as `PushDown` (not `PostFilter`) per the VP-031
    property statement.
(c) If the VP-031 proptest is not yet implemented (S-3.02 partial-merge), this AC includes
    implementing or verifying it as part of this story's deliverables.

*(traces to BC-2.11.007 §Verification Properties — VP-031 proptest property: REQUIRED columns always classified Push; §REQUIRED Column Runtime Mechanism — classify_predicates §classify_predicates is the classification function)*

---

## Red Gate Test Plan (SAC-1 — RG-001..RG-004)

| # | Test name | AC | Kind | File |
|---|-----------|----|----- |------|
| RG-001 | `test_armis_device_activity_missing_required_device_id_returns_e_query_009` | AC-001 | Integration (prism-query + armis.sensor.toml fixture; asserts E-QUERY-009 error code and message shape before any DTU request) | `crates/prism-query/src/tests/test_bc_2_11_007_required_gate.rs` |
| RG-002 | `test_armis_device_activity_with_device_id_filter_passes_enforcement_gate` | AC-002 | Integration (prism-query + armis.sensor.toml fixture; asserts no E-QUERY-009 from gate; execution proceeds) | `crates/prism-query/src/tests/test_bc_2_11_007_required_gate.rs` |
| RG-003 | `test_sensor_with_no_required_columns_never_triggers_e_query_009` | AC-003 | Unit (synthetic ColumnSpec with no REQUIRED entries; asserts gate is no-op for any predicate set) | `crates/prism-query/src/tests/test_bc_2_11_007_required_gate.rs` |
| RG-004 | `test_required_column_always_produces_push_down_in_classify_predicates` | AC-004 | Proptest (VP-031, `crates/prism-query/src/proofs/vp031_pushdown.rs`; property: for any REQUIRED column, classify_predicates returns Push) | `crates/prism-query/src/proofs/vp031_pushdown.rs` |

**BC-5.38.001 density check:** 4 Red Gate tests for 4 ACs = 1.0 ratio (≥ 0.5 required). PASS.

**Red-then-green task ordering (SAC-1 §3):** All test-authoring tasks (Tasks 1-4) MUST be
dispatched and completed before any implementation tasks (Tasks 5-7). The test-writer writes
all four Red Gate tests first; each must FAIL (red) before the implementer begins. Task ordering
in the §Tasks section below reflects this constraint — test tasks are numbered before
implementation tasks.

---

## Token Budget Estimate

| Input | Est. tokens |
|-------|------------|
| This story spec | ~6k |
| BC files (1 BC): BC-2.11.007 §REQUIRED Column Runtime Mechanism + §Error Cases + §Invariants (targeted) | ~4k |
| ADR-057 §D7 (Mechanism Layering — targeted read) | ~4k |
| ADR-033 §Decision (T1 access pattern reference — targeted) | ~2k |
| error-taxonomy.md E-QUERY-009 row (targeted) | ~1k |
| prism-query/src/pushdown.rs (classify_predicates signature + REQUIRED classification logic) | ~8k |
| prism-query/src/materialization.rs (run_materialization_pipeline + resolved_spec_map access pattern) | ~10k |
| prism-core/src/error.rs (E-QUERY-009 variant — conditional, targeted) | ~3k |
| crates/prism-sensors/specs/armis.sensor.toml (device_activity REQUIRED column declaration) | ~3k |
| New test files + proptest file | ~10k |
| Tool outputs (just iter, rg sweeps) | ~8k |
| **Total** | **~59k (~30% of 200k window — at the boundary; implementer MUST use targeted reads, not full-file reads, for materialization.rs and pushdown.rs to stay within budget)** |

---

## Tasks

### Phase A — Red Gate (test-writer dispatched FIRST; all tasks below must be RED before Phase B)

1. **RG-001 — Write `test_armis_device_activity_missing_required_device_id_returns_e_query_009`**
   Read `armis.sensor.toml` to confirm `device_id` with `options = ["REQUIRED"]` on `device_activity`.
   Construct a query `FROM armis_device_activity` with no WHERE predicate and exercise the
   plan-time gate (call path: `run_materialization_pipeline` or a direct call to the gate helper).
   Assert the error is `E-QUERY-009` with the structured fields from the error-taxonomy.md row.
   This test MUST FAIL before Phase B (no gate exists yet). Location: `test_bc_2_11_007_required_gate.rs`.

2. **RG-002 — Write `test_armis_device_activity_with_device_id_filter_passes_enforcement_gate`**
   Same setup as RG-001 with `WHERE device_id = 'abc'` added. Assert no E-QUERY-009 from the gate;
   execution proceeds past plan-time check. This test MUST FAIL before Phase B (no gate to pass
   through). Location: `test_bc_2_11_007_required_gate.rs`.

3. **RG-003 — Write `test_sensor_with_no_required_columns_never_triggers_e_query_009`**
   Construct a synthetic ColumnSpec with zero REQUIRED columns (all INDEX/DEFAULT). Assert the gate
   does not fire for any predicate combination including empty WHERE. MUST FAIL before Phase B.

4. **RG-004 — Write or verify `test_required_column_always_produces_push_down_in_classify_predicates`**
   If VP-031 proptest in `crates/prism-query/src/proofs/vp031_pushdown.rs` is already implemented
   (S-3.02 partial-merge component check), verify it is red (fails) if gate is absent. If not yet
   implemented, write the proptest here: property = for any REQUIRED column in any ColumnSpec,
   `classify_predicates` returns `Push` (not `PostFilter`). MUST FAIL before Phase B.

### Phase B — Implementation (implementer dispatched after all 4 RGs are RED)

5. **Verify classify_predicates availability (pre-condition check)**
   Read `crates/prism-query/src/pushdown.rs` and confirm `classify_predicates §classify_predicates`
   is implemented (not todo!()). If still todo!(), STOP and escalate to orchestrator — S-3.02's
   pushdown.rs component must land before this story can proceed.

6. **Implement pre-fan-out REQUIRED column gate in `run_materialization_pipeline`**
   In `crates/prism-query/src/materialization.rs`, before the fan-out loop, add a gate that:
   (a) Iterates the query's resolved sensor targets.
   (b) For each target, retrieves the sensor's `ColumnSpec` from
       `MaterializationContext.resolved_spec_map` (same access pattern as ADR-033 T1 datetime
       column lookup — see `extract_time_window_from_ast §extract_time_window_from_ast`).
   (c) Filters columns where `c.options.contains(&ColumnOptions::Required)`.
   (d) If REQUIRED columns are found, checks that the query's WHERE predicates constrain at
       least one REQUIRED column (equality predicate).
   (e) If none constrained, returns `PrismError` with E-QUERY-009 structured fields: sensor
       name, required_columns list, example WHERE clause.
   If `resolved_spec_map` is None, the gate is a no-op (safe default per risk_mitigations).
   Consider a helper function `check_required_column_constraints(predicates, column_specs,
   sensor_id)` in `pushdown.rs` to keep `materialization.rs` clean.

7. **Verify E-QUERY-009 PrismError variant in `prism-core/src/error.rs`**
   Read `error.rs` to confirm a `PrismError` variant emitting E-QUERY-009 with the
   `error-taxonomy.md §QUERY table` E-QUERY-009 Message Format exists. If absent, add the variant
   with structured fields: `sensor: String, required_columns: Vec<String>`. Display MUST match
   the taxonomy Message Format exactly: `"Required column constraint violation for {sensor}:
   columns [{required_columns}] must be constrained in WHERE clause"`. If adding a new variant,
   check `tests/external/non-exhaustive-violation/` EXPECTED_SYMBOLS list for any needed update
   (only if the PrismError type itself changes structure — verify before touching EXPECTED_SYMBOLS).

8. **Verify E-SPEC-029 defense-in-depth is intact**
   After implementing the plan-time gate, confirm that the step-level `required_filters` gate
   in `execute_impl §execute_impl` (ADR-057 §D7 Mechanism B, wave-A) is unchanged. The two
   mechanisms coexist: plan-time fires first, step-level is the backstop.

9. **Run Red Gate suite**
   `cargo nextest run -p prism-query -E 'test(test_bc_2_11_007_required_gate)'` — all 3 must go
   green. Run `cargo nextest run -p prism-query -E 'test(vp031_pushdown)'` or equivalent
   proptest run — must go green.

---

## Previous Story Intelligence

- **S-DEMO-QUERY-PUSHDOWN-001 (merged):** ADR-033 T1 established the access pattern for
  `resolved_spec_map` at the pre-fan-out stage. T1 reads per-sensor `ColumnSpec` from
  `resolved_spec_map` for datetime column type lookup — this story reuses the same access
  pattern for REQUIRED column lookup. Lesson: `resolved_spec_map` IS available pre-fan-out;
  the ADR-033 Context statement "per-sensor ColumnSpec is not yet resolved" refers to
  per-target resolution within the fan-out loop, not to the map itself.
  
- **S-3.02 (partial-merge):** `classify_predicates §classify_predicates` in
  `prism-query/src/pushdown.rs` handles REQUIRED classification. VP-031 proptest scoped to
  S-3.02. Verify both are out of partial-merge (not todo!()) before dispatching this story.
  
- **S-WAVE-A-ARMIS-ACTIVITY-001 (draft, blocked on holdout):** The wave-A story delivers
  `required_filters = ["device_id"]` + E-SPEC-029 (Mechanism B). This story delivers the
  plan-time E-QUERY-009 layer (Mechanism A). Both stories target `armis_device_activity`; the
  test names in this story focus on plan-time behavior, not step-execution behavior.
  
- **ADR-057 §D7 Mechanism Layering:** The layering table contracts precedence: E-QUERY-009 fires
  first (plan time); E-SPEC-029 fires as backstop (step execution). This precedence is NORMATIVE
  — do not implement the gate in a way that skips E-QUERY-009 and falls through to E-SPEC-029
  for queries that lack a REQUIRED WHERE predicate.

---

## Architecture Compliance Rules

Extracted from ADR-057 §D7, ADR-033 §Decision, and BC-2.11.007 §REQUIRED Column Runtime Mechanism:

1. **E-SPEC-029 (required_filters / Mechanism B) is NOT retired.** When the plan-time
   E-QUERY-009 gate ships, E-SPEC-029 in `execute_impl §execute_impl` remains as
   defense-in-depth per ADR-057 §D7 Mechanism Layering. Do NOT remove `required_filters`
   logic during this story's implementation. E-QUERY-009 + E-SPEC-029 coexist; the plan-time
   gate fires first, the step-level gate catches any bypass.

2. **Spec-driven, not hardcoded.** Per INV-REQUIRED-SPECDRIVEN (BC-2.11.007 §Invariants), the
   REQUIRED column set for any sensor+table is determined exclusively by `ColumnOptions::Required`
   in the loaded `ColumnSpec`. Any hardcoded column name (`"device_id"`, `"customer_id"`,
   `"organizationId"`) in the gate logic is a defect.

3. **resolved_spec_map access pattern.** Use
   `MaterializationContext.resolved_spec_map[sensor_id]` (same pattern as
   `extract_time_window_from_ast §extract_time_window_from_ast` in ADR-033 T1) to obtain
   ColumnSpec at the pre-fan-out stage. No fan-out orchestration restructuring is needed or
   permitted (that is ADR-033 T2's scope, not this story).

4. **Gate fires BEFORE fan-out.** The E-QUERY-009 check must execute before any HTTP request
   is dispatched — specifically before `seed_missing_query_filter_vars §seed_missing_query_filter_vars`
   runs on any step. The check is logically part of query plan construction, not execution.

5. **SAP-1 compliance.** If any new `tracing::*!(event_type=…)` emission is added, a same-commit
   BC-2.16.002 catalog row is required. Expected: zero new emissions for this gate story.

6. **Error code.** The error variant MUST display `"E-QUERY-009: ..."` per `error-taxonomy.md §QUERY
   table` E-QUERY-009 Message Format. Do NOT invent a new error code; E-QUERY-009 is registered.

---

## Library & Framework Requirements

| Dependency | Version source | Use |
|------------|---------------|-----|
| prism-core (workspace) | workspace path | ColumnOptions::Required, PrismError (E-QUERY-009 variant) |
| prism-spec-engine (workspace) | workspace path | ColumnSpec (from spec_parser.rs) — read-only; no new spec-engine dependency |
| proptest | workspace pin (S-3.02 baseline) | VP-031 proptest in vp031_pushdown.rs |

No new external dependencies. All versions from workspace `Cargo.toml` — never from training data.

---

## File Structure Requirements

| File | Action | Content |
|------|--------|---------|
| `crates/prism-query/src/materialization.rs` | MODIFY | Add pre-fan-out REQUIRED column check call in `run_materialization_pipeline §run_materialization_pipeline`, before the fan-out loop. Gate: iterate targets, look up ColumnSpec from resolved_spec_map, check REQUIRED constraints. |
| `crates/prism-query/src/pushdown.rs` | MODIFY | Add `check_required_column_constraints(predicates, column_specs, sensor_id)` helper (or extend existing API); isolates the check logic for unit-testability. |
| `crates/prism-core/src/error.rs` | MODIFY (conditional) | Verify or add PrismError variant for E-QUERY-009 with structured fields `sensor: String, required_columns: Vec<String>`. Display MUST match error-taxonomy.md §QUERY E-QUERY-009 Message Format verbatim. |
| `crates/prism-query/src/tests/test_bc_2_11_007_required_gate.rs` | CREATE | Red Gate tests RG-001, RG-002, RG-003. |
| `crates/prism-query/src/proofs/vp031_pushdown.rs` | CREATE or VERIFY | VP-031 proptest (RG-004): for any REQUIRED column, classify_predicates returns Push. If S-3.02 already implemented this file, verify it is passing; if not, implement it here. |

**Forbidden file paths:** No edits to any DTU crate (`crates/prism-dtu-*`). No edits to any
sensor TOML spec except for test fixtures. No edits to `.factory/` spec files (implementer
cannot author specs per routing policy).

---

## Forbidden Dependencies

- `prism-query` must NOT gain a dependency on any `prism-dtu-*` crate for the gate logic.
  The gate is pure: it reads ConfigSnapshot (sensor specs) and predicate AST only — no I/O.
  If the build graph gains a `prism-dtu-*` edge from `prism-query` via this story, the build
  MUST fail review.
- `prism-core` must NOT depend on `prism-query` or `prism-mcp` (existing layering invariant).
  Any new PrismError variant for E-QUERY-009 lives in prism-core, not in prism-query.

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `resolved_spec_map` is None at gate call time | Gate is a no-op; no E-QUERY-009 fired; fall through to step-level E-SPEC-029 defense. Mirrors ADR-033 T1 safe-default precedent. |
| EC-002 | Query against sensor with two REQUIRED columns; WHERE constrains only one | E-QUERY-009 fired, naming both required columns. The gate requires ALL REQUIRED columns to be constrained (DI-021: "queries cannot execute without it"). |
| EC-003 | Query against sensor with zero REQUIRED columns (all INDEX/DEFAULT) | Gate is a no-op; no E-QUERY-009. Pure spec-driven: INV-REQUIRED-SPECDRIVEN. |
| EC-004 | Multi-sensor fan-out query; Sensor A has REQUIRED column unconstrained; Sensor B has none | E-QUERY-009 fired for Sensor A only; the error names Sensor A and its required column. Sensor B proceeds normally. |
| EC-005 | REQUIRED column constrained with inequality predicate (e.g., `device_id > 'abc'`) | Implementer must decide: BC-2.11.007 says "constrained in WHERE clause" without specifying operator; equality is the safe assumption. If inequality doesn't satisfy the gate, E-QUERY-009 fires with a message directing user to use `=`. Route decision to product-owner if ambiguous. |
| EC-006 | E-QUERY-009 fired; downstream E-SPEC-029 path never reached | Correct behavior: E-QUERY-009 fires at plan time; E-SPEC-029 is unreachable for this code path. Both remain in the codebase; no removal. |
| EC-007 | REQUIRED column constrained with an empty-string value (e.g. `WHERE device_id = ''`) | Tier 1 gate (this story) **passes**: `classify_predicates §classify_predicates` checks predicate presence only — the predicate's RHS value is not inspected at plan time. `WHERE device_id = ''` is structurally present and indistinguishable from `WHERE device_id = 'abc'` at the plan-time layer; no E-QUERY-009 fires. Tier 2 (`required_filters` gate in `execute_impl §execute_impl`, E-SPEC-029, ADR-057 §D7 Rule 2) catches the present-but-empty arm before any HTTP request is issued. Coverage: `S-WAVE-A-ARMIS-ACTIVITY-001` AC-009 / RG-009. Expected behavior by design — this story owes no AC or RG test for the empty-value arm. |

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `check_required_column_constraints` | prism-query `pushdown.rs` | Pure (ColumnSpec + predicates in → Result out) |
| Plan-time gate placement | prism-query `materialization.rs` `run_materialization_pipeline §run_materialization_pipeline` | Pure (reads resolved_spec_map snapshot) |
| E-QUERY-009 error variant | prism-core `error.rs` | Pure (data type) |
| VP-031 proptest | prism-query `src/proofs/vp031_pushdown.rs` | Pure (property test) |

---

## Out of Scope

1. **ADR-033 T2 full restructure** (per-sensor `classify_predicates` post-resolution integration
   via fan-out orchestration restructuring): NOT this story's scope. The REQUIRED column gate
   uses `resolved_spec_map` pre-fan-out (consistent with T1) — no fan-out restructuring needed.
   Full T2 (broadening push-down optimization to all non-REQUIRED push-down dimensions in a
   per-sensor post-resolution way) is a separate future story not yet scoped. When that story
   is authored, it may supersede ADR-033 with a new ADR (per ADR-033 §Alternatives Considered).

2. **Retiring E-SPEC-029 (required_filters):** NOT this story's scope. E-SPEC-029 is active
   wave-A and remains as defense-in-depth per ADR-057 §D7 Mechanism Layering.

3. **REQUIRED column enforcement for write operations:** Write path (S-3.07, S-3.06) is
   separate; this gate is query-path only.

4. **E-QUERY-002 or other error code reuse:** No collision with existing codes. E-QUERY-009 is
   registered in error-taxonomy.md for this exact condition.

---

## Story Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 1.1 | 2026-07-31 | story-writer | FB111 — (1) EC-007 added to Edge Cases table: REQUIRED column constrained with empty-string value (`WHERE device_id = ''`). Architect adjudication DRIFT-S-REQUIRED-COL-GATE-001-EMPTYVAL-001 (ADR-057 §D7 v1.4): Tier 1 gate passes by design — `classify_predicates §classify_predicates` is predicate-presence, value-agnostic; `S-WAVE-A-ARMIS-ACTIVITY-001` AC-009 / RG-009 covers Tier 2 (E-SPEC-029 Rule 2 present-but-empty arm). No AC or RG test owed by this story for this case; AC count and SAC-1 density (4/4 = 1.0) UNCHANGED. (2) `## Authority` section added citing BC-2.11.007 §REQUIRED Column Runtime Mechanism (DI-021 enforcement path) and ADR-057 §D7 Mechanism Layering (tier division). Both anchors verified as real `##` headings on disk (BC-2.11.007 `## REQUIRED Column Runtime Mechanism`, ADR-057 `## D7 — Required-Filter Gate Mechanism`). ADR-057 `anchor_stories` can now be updated by architect to include this story per SAC-2. (3) `version:` frontmatter bumped 1.0 → 1.1; `modified: "2026-07-31"` added to frontmatter. POL-29: (9a) 237 S-*.md stories; 11 have `## Authority`; 226 lack it — corpus pattern, not a two-file gap; `S-DEMO-QUERY-PUSHDOWN-001` also lacks `## Authority` (separately routed per dispatch); (9b) EC-007 content downstream of ADR-057 §D7 "Tier responsibility division — empty-value arm" block (v1.4 source); both agree; no other artifact transcribes EC-007 verbatim; (9c) EC-007 contains no `MUST`; no new unanchored MUSTs introduced. |
| 1.0 | 2026-07-31 | story-writer | Authored per FB104 (close unanchored-deferral chain around plan-time REQUIRED-column enforcement, ADR-057 §D7 open obligation). Adjudication: one story (not two) — the enforcement gate is self-sufficient via resolved_spec_map pre-fan-out access (T1 pattern); ADR-033 T2 fan-out restructuring is a separate future concern not required here. S-3.02 as depends_on (classify_predicates availability). 4 ACs, 4 Red Gate tests, BC-5.38.001 density 1.0. POL-35 holdout note. SAC-1 compliant. POL-39 compliant (no version pins). POL-29 9c: all MUSTs anchored to AC/RG in this story. Re-anchors wave-granularity deferral in S-DEMO-QUERY-PUSHDOWN-001 and S-QUERY-SCOPE-PARAMS-001 to this story ID. |
