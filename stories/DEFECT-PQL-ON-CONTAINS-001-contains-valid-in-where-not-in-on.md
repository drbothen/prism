---
document_type: story
story_id: "DEFECT-PQL-ON-CONTAINS-001"
title: "CONTAINS operator valid in WHERE but rejected in ON clause — product-owner decision required"
severity: LOW
priority: P3
wave: tbd
epic_id: engine-defects
status: draft
version: "0.1"
level: ops
producer: story-writer
timestamp: "2026-08-03"
modified: "2026-08-03"
origin_finding: "F4 (D-1889 live-demo triage 2026-07-20)"
origin_cascade: "D-1889 live-demo triage (2026-07-20); decisions resolved at D-1943"
cycle: "v1.0.0-greenfield"
phase: 3
track: "Platform Engineering"
subsystems: [SS-11]
behavioral_contracts:
  - BC-2.11.003
verification_properties: []
depends_on: []
blocks: []
points: 0
risk: LOW
inputs:
  - .factory/planning/findings-remediation-2026-07-20/triage-capture.md
  - .factory/planning/findings-remediation-2026-07-20/findings/prism-pql-deficiencies.md
  - .factory/planning/findings-remediation-2026-07-20/findings/prism-pushdown-audit.md
---

# DEFECT-PQL-ON-CONTAINS-001: CONTAINS operator valid in WHERE but rejected in ON clause — product-owner decision required

## Problem

The `CONTAINS` operator is accepted in `WHERE` clause filter predicates but triggers a parse
error when used in `ON` clause predicates of JOIN expressions. The behavioral symmetry — or
intentional asymmetry — of `CONTAINS` across these two clause positions is not defined by the
current BC-2.11.003 §Invariants.

Whether `CONTAINS` should be a valid predicate operator in `ON` clauses is a product-owner
decision. Accepting it there is a grammar extension that must be reflected in BC-2.11.003.
Rejecting it intentionally (asymmetric design) is also a valid outcome but must likewise be
documented in BC-2.11.003 §Invariants to prevent this defect class from recurring.
Implementers MUST NOT assume the fix scope without a product-owner adjudication.

## Origin

Identified in the D-1889 live-demo triage (2026-07-20). Source documents:
`findings/prism-pql-deficiencies.md` (referenced in `triage-capture.md §source_readings`).
Finding ID: F4, severity LOW. Coverage NEW. Not wave-assigned in the triage wave schedule.

This defect was unregistered as a tracking artifact prior to this stub. It existed only in
`triage-capture.md` (status: OPEN at time of triage; decisions resolved at D-1943).

## Authority

- BC-2.11.003 §Postconditions / §Invariants (PrismQL SQL mode operator acceptance rules):
  governs which operators are valid in which clause positions. The BC must be amended to
  reflect the product-owner decision before implementation. Status: active.
- No ADR governs this specific defect.

## Routing

product-owner decision → implementer

Product-owner must adjudicate: (a) allow `CONTAINS` in `ON` (grammar extension, requires
BC-2.11.003 §Invariants amendment documenting the symmetric behavior); or (b) reject
`CONTAINS` in `ON` intentionally (requires BC-2.11.003 §Invariants amendment documenting
the asymmetry). After the product-owner authors the BC amendment, the implementer delivers
the fix (grammar change or explicit rejection gate + test).

## Scope — NOT YET SPECIFIED

Acceptance criteria, Red Gate test list (`RG-001..RG-NNN`), BC-5.38.001 density check, and
task decomposition are deferred to the routed specialist chain. This file is a registration
artifact — it makes the defect countable, discoverable, and assigned with a named owner.

Next step owner: **product-owner** (adjudicate `CONTAINS` in `ON`, author BC-2.11.003 amendment).
After BC amendment: **implementer** (TDD grammar fix or explicit rejection gate).

`tdd_mode` and enumerated Red Gate list MUST be set by product-owner when this story reaches
`status: ready`, per SAC-1.

## §Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 0.1 | 2026-08-03 | story-writer | Initial registration stub — defect identified in D-1889 live-demo triage (2026-07-20); unregistered prior to this artifact. |
