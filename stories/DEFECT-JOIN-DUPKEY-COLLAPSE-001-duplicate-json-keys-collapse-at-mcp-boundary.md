---
document_type: story
story_id: "DEFECT-JOIN-DUPKEY-COLLAPSE-001"
title: "duplicate JSON keys collapse at MCP serialization boundary — JOIN results silently drop fields"
severity: MED
priority: P2
wave: tbd
epic_id: engine-defects
status: draft
version: "0.1"
level: ops
producer: story-writer
timestamp: "2026-08-03"
modified: "2026-08-03"
origin_finding: "F2 (D-1889 live-demo triage 2026-07-20)"
origin_cascade: "D-1889 live-demo triage (2026-07-20); decisions resolved at D-1943"
cycle: "v1.0.0-greenfield"
phase: 3
track: "Platform Engineering"
subsystems: [SS-11]
behavioral_contracts:
  - BC-2.11.001
verification_properties: []
depends_on: []
blocks: []
points: 0
risk: MEDIUM
inputs:
  - .factory/planning/findings-remediation-2026-07-20/triage-capture.md
  - .factory/planning/findings-remediation-2026-07-20/findings/prism-pql-deficiencies.md
  - .factory/planning/findings-remediation-2026-07-20/findings/prism-pushdown-audit.md
---

# DEFECT-JOIN-DUPKEY-COLLAPSE-001: duplicate JSON keys collapse at MCP serialization boundary — JOIN results silently drop fields

## Problem

When a PrismQL JOIN produces a result row in which two source tables contribute columns with
the same key name (for example, both `crowdstrike.detections` and `claroty.devices` expose a
column named `id`), the JSON serialization at the MCP tool response boundary collapses the
duplicate key. One value is silently discarded. The consumer LLM agent receives a row with
fewer fields than the query schema declares, with no error or warning emitted.

This is a wire-shape serialization defect at the MCP boundary. SID-2 discipline applies: any
test covering this defect MUST assert on the full serialized JSON wire output (the exact bytes
the LLM agent receives), not only on pre-serialization Rust structures. Wire-shape assertion
at the serialized output level is the contract surface.

## Origin

Identified in the D-1889 live-demo triage (2026-07-20). Source documents:
`findings/prism-pql-deficiencies.md` (referenced in `triage-capture.md §source_readings`).
Finding ID: F2, severity MED. Not wave-assigned in the triage wave schedule.

This defect was unregistered as a tracking artifact prior to this stub. It existed only in
`triage-capture.md` (status: OPEN at time of triage; decisions resolved at D-1943).

## Authority

- BC-2.11.001 §Postconditions (MCP query tool wire-shape invariants): governs the serialized
  JSON shape of rows returned by the `prism_query` MCP tool, including field completeness.
  Status: active.
- No ADR governs this specific defect.

## Routing

product-owner → implementer

Product-owner must author the AC with an explicit wire-shape assertion contract (SID-2:
the AC must specify the expected serialized JSON field set, including disambiguation of
same-name columns from different JOIN sources). Implementer delivers the fix and
wire-shape tests.

## Scope — NOT YET SPECIFIED

Acceptance criteria, Red Gate test list (`RG-001..RG-NNN`), BC-5.38.001 density check, and
task decomposition are deferred to the routed specialist chain. This file is a registration
artifact — it makes the defect countable, discoverable, and assigned with a named owner.

Next step owner: **product-owner** (AC authoring with wire-shape contract).
After AC authorship: **implementer** (TDD fix delivery with SID-2 wire-shape assertions).

`tdd_mode` and enumerated Red Gate list MUST be set by product-owner when this story reaches
`status: ready`, per SAC-1.

## §Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 0.1 | 2026-08-03 | story-writer | Initial registration stub — defect identified in D-1889 live-demo triage (2026-07-20); unregistered prior to this artifact. |
