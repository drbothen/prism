---
document_type: story
story_id: "DEFECT-PQL-SUBQUERY-FANOUT-001"
title: "cross-sensor WHERE IN(SELECT) silently returns 0 rows — recursive source extraction missing at materialization stage"
severity: CRIT
priority: P0
wave: c
epic_id: engine-defects
status: draft
version: "0.1"
level: ops
producer: story-writer
timestamp: "2026-08-03"
modified: "2026-08-03"
origin_finding: "F1 (D-1889 live-demo triage 2026-07-20)"
origin_cascade: "D-1889 live-demo triage (2026-07-20); decisions resolved at D-1943"
cycle: "v1.0.0-greenfield"
phase: 3
track: "Platform Engineering"
subsystems: [SS-11]
behavioral_contracts:
  - BC-2.11.005
verification_properties: []
depends_on: []
blocks: []
points: 0
risk: CRITICAL
inputs:
  - .factory/planning/findings-remediation-2026-07-20/triage-capture.md
  - .factory/planning/findings-remediation-2026-07-20/findings/prism-pql-deficiencies.md
  - .factory/planning/findings-remediation-2026-07-20/findings/prism-pushdown-audit.md
---

# DEFECT-PQL-SUBQUERY-FANOUT-001: cross-sensor WHERE IN(SELECT) silently returns 0 rows — recursive source extraction missing at materialization stage

## Problem

When a PrismQL query uses a cross-sensor `WHERE col IN (SELECT ... FROM sensor.table)` construct,
the outer query silently returns 0 rows with no error. Root cause: `extract_source_names` in
`prism-query::materialization` delegates to `extract_source_names_shallow`, which does not recurse
into subquery branches of the AST. At the source-extraction step inside `run_materialization_pipeline`,
the inner SELECT's source table names are absent from the fan-out set, so the materialization stage
fans out to no adapters for the subquery source tables. The materialized subquery result is empty,
and the outer query predicate matches nothing.

The function `extract_source_names_recursive` already exists in `prism-query::materialization` and
performs the correct AST traversal. The fix is to replace the `extract_source_names_shallow`
delegation at the call site inside `run_materialization_pipeline` with `extract_source_names_recursive`.
The exact call site and any guard conditions are to be confirmed by the implementer.

## Origin

Identified in the D-1889 live-demo triage (2026-07-20). Source documents:
`findings/prism-pql-deficiencies.md` and `findings/prism-pushdown-audit.md` (referenced in
`triage-capture.md §source_readings`). Finding ID: F1, severity CRIT. One of three engine
CRIT findings in the triage. Assigned to Wave C per triage wave schedule.

This defect was unregistered as a tracking artifact prior to this stub. It existed only in
`triage-capture.md` (status: OPEN at time of triage; decisions resolved at D-1943).

## Authority

- BC-2.11.005 §Preconditions / §Postconditions (ephemeral materialization): governs the
  materialization pipeline fan-out behavior and the requirement that all referenced source
  tables be resolved and queried. Status: active.
- No ADR governs this specific defect.

## Routing

product-owner → test-writer + implementer

SAP-3 standing probe applies: every BC-2.11.005 postcondition arm covered by the resulting
tests MUST be reachable end-to-end from the public MCP tool surface (real query string input),
not only via synthetic AST injection. Synthetic-AST tests count as defense-in-depth only.

## Scope — NOT YET SPECIFIED

Acceptance criteria, Red Gate test list (`RG-001..RG-NNN`), BC-5.38.001 density check, and
task decomposition are deferred to the routed specialist chain. This file is a registration
artifact — it makes the defect countable, discoverable, and assigned with a named owner.

Next step owner: **product-owner** (BC-2.11.005 amendment authoring and AC composition).
After AC authorship: **test-writer** (Red Gate stubs), then **implementer** (TDD fix delivery).

`tdd_mode` and enumerated Red Gate list MUST be set by product-owner when this story reaches
`status: ready`, per SAC-1.

## §Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 0.1 | 2026-08-03 | story-writer | Initial registration stub — defect identified in D-1889 live-demo triage (2026-07-20); unregistered prior to this artifact. |
