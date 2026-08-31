---
document_type: story
story_id: "DEFECT-PQL-SENSOR-QUALIFIED-TABLEREF-001"
title: "PrismQL sensor-qualified table reference `<sensor>.<table>` triggers misleading E-QUERY-037; bare name is canonical (post-v1 grammar reconciliation)"
severity: LOW
priority: P3
wave: post-v1
epic_id: engine-defects
status: draft
# BC status: pending PO + architect authorship at scheduling time. Do NOT promote to
# ready before a governing BC is authored and the architectural path (A or B) is
# adjudicated. behavioral_contracts: [] is intentional here; Spec-First Gate S-7.01 applies.
# # BC status: pending PO authorship
version: "0.1"
level: ops
producer: story-writer
timestamp: "2026-08-31"
modified: "2026-08-31"
origin_finding: "G2 live holdout evaluation — S-CLAROTY-OT-EVENTS-001 story-level holdout gate (HS-025 group), monroe tenant, 2026-08-31"
origin_cascade: "S-CLAROTY-OT-EVENTS-001 holdout gate (2026-08-31); pre-existing engine-wide behavior, NOT introduced by G2"
cycle: "v1.0.0-brownfield"
phase: 3
track: "Platform Engineering"
subsystems: [SS-11]
# Subsystem anchor justification:
#   SS-11 (Query Execution Engine) owns this story because the defect lives in the
#   PrismQL query planner — specifically the table-reference resolution path in
#   prism-query that handles FROM-clause source names. E-QUERY-037 is fired by the
#   plan-time table availability gate, which is exclusively SS-11 scope per ARCH-INDEX
#   Subsystem Registry. The grammar reconciliation (Path A) or error-message improvement
#   (Path B) is also a plan-gate concern owned by SS-11.
behavioral_contracts: []
# BC status: pending PO authorship
# Governing BC to be authored/confirmed by PO + architect at scheduling time.
# Anchor: E-QUERY-037 in `.factory/specs/prd-supplements/error-taxonomy.md` (plan-time
# table availability gate; `PrismError::TableNotAvailable`). No BC-S.SS.NNN ID assigned
# until PO authors or extends an existing BC to cover the `<sensor>.<table>` grammar
# surface. Do NOT use BC-TBD placeholders.
verification_properties: []
assumption_validations: []
risk_mitigations: []
depends_on: []
blocks: []
points: 0
# Points: 0 pending PO + architect scope adjudication. At scheduling time:
#   Path A (support qualified form as alias): estimated 5 pts (parser + planner + tests)
#   Path B (improved error message only):     estimated 2 pts (error construction + tests)
estimated_days: 0
# estimated_days: 0 pending adjudication; update at scheduling time
target_module: prism-query
# target_module: prism-query — the plan-time table availability gate lives in prism-query.
# Update if Path A requires grammar changes that also touch other crates.
traces_to: ""
# traces_to: no BC assigned yet; populate at scheduling time once governing BC is authored
inputs:
  - ".factory/specs/prd-supplements/error-taxonomy.md"
  - ".factory/holdout-scenarios/S-CLAROTY-OT-EVENTS-001-HS-001-ot-event-wire-shape-class-uid.md"
input-hash: "cf84c03"
# input-hash: computed from error-taxonomy.md + HS-001 holdout scenario
risk: LOW
tdd_mode: strict
# tdd_mode will be confirmed by PO at scheduling time. `strict` is the default per BC-8.30.001.
---

# DEFECT-PQL-SENSOR-QUALIFIED-TABLEREF-001: PrismQL sensor-qualified table reference `<sensor>.<table>` triggers misleading E-QUERY-037; bare name is canonical (post-v1 grammar reconciliation)

## Problem

On the live monroe tenant running the G2 binary (S-CLAROTY-OT-EVENTS-001), a
schema-qualified table reference of the form `<sensor>.<table>` — concretely
`SELECT ... FROM claroty.claroty_ot_activity_events` — returns error E-QUERY-037
with the message "sensor 'claroty' is not configured."

The bare table name `claroty_ot_activity_events` resolves correctly and returns data.
`prism_describe` PQL hints and the live-tenant runbook examples both use the bare name,
confirming that bare-name is the currently-working canonical query form.

The E-QUERY-037 message "sensor 'claroty' is not configured" is misleading: the claroty
sensor IS configured. The confusion arises because the SQL parser interprets
`claroty.claroty_ot_activity_events` as a schema-qualified reference where `claroty` is
the schema prefix. The planner's source-name extraction then resolves `claroty` (not
`claroty.claroty_ot_activity_events`) as the table source reference, attempts to look up
"claroty" as a registered table name, and fires E-QUERY-037 because there is no table
named "claroty" — only tables named `claroty_alerts`, `claroty_ot_activity_events`, etc.

This is a **pre-existing, engine-wide behavior** — it applies to any sensor whose name
appears as a schema prefix in a dotted FROM-clause reference. The defect is NOT
introduced by G2 and is NOT a G2 builder error.

**Why this matters:**

1. **Analyst UX:** The dotted `<sensor>.<table>` form is the intuitive form for analysts
   familiar with standard SQL schema.table syntax. SQL tools and LLM agents are trained
   to write fully-qualified references. Any analyst who writes the natural form gets a
   misleading error message that implies the sensor is not configured — a false
   diagnosis that wastes investigation time.

2. **Holdout scenario correctness:** The three G2 holdout scenarios
   (HS-OTEVTS-001-001, HS-OTEVTS-001-002, HS-OTEVTS-001-003) were authored using the
   dotted form `claroty.claroty_ot_activity_events` in their SQL query strings. These
   scenarios fail with E-QUERY-037 on any future re-execution. Correcting the HS query
   strings to the canonical bare form is an in-scope deliverable of this story.

3. **Engine-wide:** The same defect exists for every configured sensor. A query of the
   form `SELECT * FROM crowdstrike.crowdstrike_alerts` would trigger the same
   E-QUERY-037 on "sensor 'crowdstrike' is not configured" regardless of configuration.

## Two Resolution Paths (Architectural Decision Required)

The fix has two mutually exclusive paths. The architectural decision between them is
deferred to the architect + PO at scheduling time. Implementers MUST NOT choose a path
without the architect adjudication.

**Path A — Support sensor-qualified form as an alias (grammar extension):**
Extend the PrismQL FROM-clause table-reference resolution logic to accept
`<sensor>.<table>` as equivalent to the bare `<table>` form, provided that `<sensor>`
matches the sensor prefix embedded in `<table>`. PO must author or extend the relevant
BC (BC-2.11.001 plan-time source resolution or a new BC) to specify this alias rule.

**Path B — Replace misleading E-QUERY-037 with a clear diagnostic (error improvement):**
Preserve bare-name as the only canonical form. When the planner detects the
`<word>.<word>` dotted pattern and the full dotted string does NOT match a registered
table but `<second_word>` DOES match a registered table, emit a targeted diagnostic
such as: "sensor-qualified form is not supported; use the bare table name `<table>`."
Requires only error-construction changes in the plan-time gate path; minimal blast
radius; preserves existing grammar invariants.

## Origin

Surfaced during the story-level holdout gate for S-CLAROTY-OT-EVENTS-001 (HS-025
group), run against the live monroe Claroty xDome sensor on 2026-08-31. All three
holdout scenarios used the dotted form `claroty.claroty_ot_activity_events` in their
embedded SQL query strings and were blocked by E-QUERY-037.

**The G2 binary passed the holdout gate via the bare-name form** — the HS-001/002/003
SQL strings were patched to `claroty_ot_activity_events` (bare form) during that
session. This story captures the root cause for durable post-v1 resolution.

This defect was unregistered as a tracking artifact prior to this stub.
Human-directed: file as post-v1 follow-up; do NOT schedule into the v1 critical path.

## Authority

- **E-QUERY-037** in `.factory/specs/prd-supplements/error-taxonomy.md` — the
  plan-time table availability gate (`PrismError::TableNotAvailable`). This is the
  error whose message is misleading for the dotted-form case. The governing BC will
  anchor to this error code.
- **Governing BC:** to be authored or extended by PO + architect at scheduling time.
  No BC-S.SS.NNN ID is assigned in this draft. The story MUST NOT be promoted to
  `status: ready` until a governing BC is authored and its ID is set in
  `behavioral_contracts:` (Spec-First Gate S-7.01).
- **ADR anchor:** if Path A is chosen, an ADR amendment or new ADR section is required
  to canonicalize the `<sensor>.<table>` alias rule. Route to architect at scheduling.

## Routing

1. **Architect (at scheduling time):** Adjudicate Path A vs Path B.
2. **Product-owner (after architect decision):** Author the governing BC.
3. **Story-writer (after architect + PO):** Expand this stub to a full story spec with
   ACs, Red Gate list, density check, and all six context-engineering sections.
4. **Test-writer + implementer:** Standard TDD delivery.

SAP-3 standing probe applies to any pass touching prism-query grammar or plan gates:
every postcondition arm MUST be reachable end-to-end from a real PrismQL query string
(not only via synthetic AST injection).

## Narrative

As a security analyst or LLM agent writing PrismQL queries, I want the engine to either
accept or clearly guide me away from the sensor-qualified form `<sensor>.<table>`, so
that I do not receive a misleading "sensor not configured" error when the sensor is in
fact correctly configured and only the table reference form is wrong.

_Note: this narrative is a placeholder for the registration stub. The story-writer will
revise it to a precise user narrative once the architect adjudicates Path A vs Path B._

## Acceptance Criteria

> **NOT YET SPECIFIED — pending architect + PO adjudication.**
>
> AC list, BC traces, and Red Gate list (RG-001..RG-NNN) are to be authored by the
> product-owner after the architect has chosen Path A or Path B. ACs must conform to
> SAC-1 (enumerated Red Gate list in `RG-001..RG-NNN` format + BC-5.38.001 density check).
>
> **Confirmed in-scope AC (Path-independent):** Update the SQL query strings in the
> three G2 holdout scenario files from the dotted form to the bare canonical form:
>
> | Holdout Scenario File | Query Strings to Correct |
> |----------------------|--------------------------|
> | `S-CLAROTY-OT-EVENTS-001-HS-001-ot-event-wire-shape-class-uid.md` | `FROM claroty.claroty_ot_activity_events` → `FROM claroty_ot_activity_events` |
> | `S-CLAROTY-OT-EVENTS-001-HS-002-tier2-source-ip-not-standalone.md` | Both queries: `FROM claroty.claroty_ot_activity_events` → `FROM claroty_ot_activity_events` |
> | `S-CLAROTY-OT-EVENTS-001-HS-003-detection-time-time-column.md` | Both queries: `FROM claroty.claroty_ot_activity_events` → `FROM claroty_ot_activity_events` |
>
> These files carry `single_use: true` and `used: false`. The correction does NOT mark
> them used — it makes them executable with the correct query form.

## Architecture Mapping

> **NOT YET SPECIFIED — pending architect adjudication.**
>
> Preliminary architecture mapping (to be confirmed/extended at full-story-spec time):

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| Plan-time table availability gate | `prism-query` (plan gates / `resolve_source_refs`) | Pure |
| E-QUERY-037 error construction | `prism-core::error` (`PrismError::TableNotAvailable`) | Pure |
| Holdout scenario SQL strings | `.factory/holdout-scenarios/S-CLAROTY-OT-EVENTS-001-HS-00{1,2,3}*.md` | N/A (spec artifact) |

If Path A is chosen, additional components (PrismQL grammar, AST node, parser module
in `prism-query`) will be added to this table.

## Edge Cases

> **NOT YET SPECIFIED — pending architect adjudication and AC authorship.**
>
> Preliminary edge cases (to be refined at full-story-spec time):

| ID | Description | Expected Behavior (TBD per Path A/B) |
|----|-------------|---------------------------------------|
| EC-001 | `<sensor>.<table>` where sensor matches the prefix of table (e.g., `claroty.claroty_alerts`) | Path A: resolves correctly. Path B: targeted diagnostic guides to bare form. |
| EC-002 | `<sensor>.<table>` where sensor does NOT match the prefix of table (e.g., `claroty.crowdstrike_alerts`) | Path A: resolves to `crowdstrike_alerts` if registered (strip claroty. prefix), OR E-QUERY-037 for table not found. Adjudication required. Path B: generic E-QUERY-037 (no false-positive override). |
| EC-003 | `<word>.<table>` where `<word>` is not any configured sensor prefix (e.g., `schema.claroty_alerts`) | Path A: fall through to normal E-QUERY-037. Path B: no change — E-QUERY-037 fires correctly. |
| EC-004 | Bare table name continues to resolve correctly after any grammar change | Both paths: no regression to existing bare-name resolution. |
| EC-005 | Multi-tenant deployment: dotted form used with an org-scoped table | Path A: org-scoping invariants from E-QUERY-037 (ADR-039) must be preserved in the alias resolution path. |

## Purity Classification

| Component | Classification | Rationale |
|-----------|---------------|-----------|
| Table-reference resolution | Pure | No I/O; operates on parsed AST and in-memory `TableRegistry` |
| Error construction | Pure | Builds `PrismError::TableNotAvailable` from in-memory data |
| Holdout scenario update | N/A (spec file edit) | Not production code |

## Token Budget Estimate (MANDATORY)

> **Preliminary estimate — to be revised at full-story-spec time.**
>
> This is a registration stub. Token budget estimate for the implementing agent:

| Item | Estimated Tokens |
|------|-----------------|
| Story spec (this file, full version) | ~4,000 |
| BC files (0 BCs — pending) | 0 |
| Architecture sections (module-decomposition + dependency-graph excerpts) | ~3,000 |
| Relevant source files (prism-query plan gates, prism-core error.rs) | ~8,000 |
| Test files to add/modify | ~4,000 |
| Tool outputs (just check, cargo nextest) | ~2,000 |
| **Total estimate** | **~21,000** |

Token budget is well within the 20–30% context window limit for implementing agents.
This story does not require splitting.

## Tasks (MANDATORY)

> **NOT YET SPECIFIED — pending architect + PO adjudication and AC authorship.**
>
> Task list to be authored by story-writer at full-story-spec time after architect and
> PO have resolved Path A vs B and authored the governing BC.
>
> **Confirmed task (Path-independent, low-ceremony):**
> - [ ] Update the three holdout scenario files (HS-001/HS-002/HS-003) to replace
>       dotted table references with the canonical bare form. This task does NOT require
>       full story-delivery ceremony and may be dispatched to state-manager as a
>       records-tier micro-burst (TD-VSDD-096) independent of the engine fix timeline.

## Previous Story Intelligence (MANDATORY)

N/A — this is the first story in the engine-defects/PQL-grammar-qualified-tableref
sub-group. No predecessor stories have delivered in this exact scope.

**Related delivered stories for context:**
- `S-3.13-dynamic-table-availability.md` (merged): delivered E-QUERY-037 plan-time
  table availability gate. The defect originates in the source-name extraction logic
  introduced by S-3.13 when given a dotted FROM-clause reference.
- `S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001` (merged): delivered grammar improvements
  including `available_tables` and `did_you_mean` on E-QUERY-037 and E-QUERY-036.
  The did_you_mean heuristic on E-QUERY-037 will NOT suggest the correct table for
  the dotted-form case because `claroty` (the schema prefix) has no Levenshtein-close
  match to the full bare table name `claroty_ot_activity_events`.

## Architecture Compliance Rules (MANDATORY)

> **Preliminary compliance rules — to be confirmed/extended at full-story-spec time.**

1. **Pure-core boundary (ADR-022 §C):** any changes to plan-gate logic MUST stay in
   `prism-query` pure-core. No new I/O introduced in the table-availability gate path.

2. **E-QUERY-037 org-scoping invariant (ADR-039):** if Path A adds a
   `<sensor>.<table>` alias resolution, the alias lookup MUST use the same
   org-scoped `TableRegistry` filter as the existing E-QUERY-037 `available_sensors`
   and `available_tables` fields. Cross-org table aliases are forbidden (CWE-200).

3. **Error code stability:** the fix MUST NOT introduce a new error code for the
   sensor-qualified-form rejection path. Path B improvement is a message change within
   the existing `E-QUERY-037` contract, not a new code. Path A eliminates the error
   for the correctly-qualified case. Neither path requires a new `E-QUERY-NNN` entry.

4. **No `unwrap()` in plan-gate path (CLAUDE.md Conventions):** any new code in the
   plan-time gate must use `?` propagation or explicit `match`. No `unwrap()` or
   `expect()` in non-test code.

5. **BC-2.11.016 E-QUERY-038 gate ordering (error-taxonomy):** the E-QUERY-037 table
   check fires BEFORE E-QUERY-038 column check. Any change to the E-QUERY-037 gate
   must preserve this ordering invariant.

## Library & Framework Requirements (MANDATORY)

> Versions to be confirmed via `Cargo.lock` at full-story-spec time.
>
> Preliminary: no new library dependencies expected for Path B. Path A may require
> minor extensions to the existing `chumsky`-based PrismQL parser in `prism-query`.
>
> **Forbidden dependencies:** `prism-query` MUST NOT depend on `prism-mcp`, `prism-bin`,
> or `prism-sensors`. The existing dependency-graph.md §Dependency Rules forbid these.
> Any Path A grammar changes MUST stay within `prism-query` and `prism-core`; no new
> cross-crate dependencies are permitted without architect approval.

## File Structure Requirements (MANDATORY)

> **NOT YET SPECIFIED — pending architect adjudication.**
>
> Preliminary file structure (to be confirmed at full-story-spec time):

| File | Action | Purpose |
|------|--------|---------|
| `crates/prism-query/src/` (plan gate module) | Modify | Path A: alias resolution; Path B: E-QUERY-037 message improvement |
| `crates/prism-core/src/error.rs` | Possibly modify | Path B: update E-QUERY-037 message construction in `PrismError::TableNotAvailable` |
| `crates/prism-query/src/tests/` | Add | Red Gate tests for the chosen resolution path |
| `.factory/holdout-scenarios/S-CLAROTY-OT-EVENTS-001-HS-001-ot-event-wire-shape-class-uid.md` | Modify | Correct SQL from dotted to bare table name |
| `.factory/holdout-scenarios/S-CLAROTY-OT-EVENTS-001-HS-002-tier2-source-ip-not-standalone.md` | Modify | Correct SQL from dotted to bare table name |
| `.factory/holdout-scenarios/S-CLAROTY-OT-EVENTS-001-HS-003-detection-time-time-column.md` | Modify | Correct SQL from dotted to bare table name |

## §Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 0.1 | 2026-08-31 | story-writer | Initial registration stub — post-v1 follow-up for engine-wide PrismQL qualified table-reference grammar gap surfaced during G2 (S-CLAROTY-OT-EVENTS-001) live holdout gate on monroe tenant (2026-08-31). Human-directed: post-v1 deferral, do NOT schedule into v1 critical path. Holdout scenario correction (HS-001/002/003 SQL bare-name fix) documented as path-independent in-scope deliverable. |
