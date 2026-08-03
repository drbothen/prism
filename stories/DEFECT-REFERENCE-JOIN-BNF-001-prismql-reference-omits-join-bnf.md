---
document_type: story
story_id: "DEFECT-REFERENCE-JOIN-BNF-001"
title: "prismql://reference omits JOIN BNF — implementation drift against BC-2.10.014 §Postconditions"
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
origin_finding: "F3 (D-1889 live-demo triage 2026-07-20)"
origin_cascade: "D-1889 live-demo triage (2026-07-20); decisions resolved at D-1943"
cycle: "v1.0.0-greenfield"
phase: 3
track: "Platform Engineering"
subsystems: [SS-10]
behavioral_contracts:
  - BC-2.10.014
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

# DEFECT-REFERENCE-JOIN-BNF-001: prismql://reference omits JOIN BNF — implementation drift against BC-2.10.014 §Postconditions

## Problem

The `prismql://reference` MCP static resource exposes the PrismQL grammar BNF to LLM agents.
The JOIN clause syntax is absent from the BNF content served at this resource. BC-2.10.014
§Postconditions specifies that the reference resource MUST include the complete PrismQL grammar,
including JOIN syntax. The current implementation serves a BNF that omits the JOIN clause.

This is implementation drift: the spec already requires JOIN BNF in the reference resource.
No new spec authoring or product-owner decision is needed — the implementer reads BC-2.10.014
§Postconditions for the authoritative BNF contract and adds the missing JOIN syntax to the
resource handler.

Note: the triage also notes a minor F3 co-finding (stale credential CLI help-text omitting
`--org-slug`). That item may be folded into this story's scope or handled as a separate LOW
story at Wave B/C backlog opening, per cross-check addendum item 1 in `triage-capture.md`.

## Origin

Identified in the D-1889 live-demo triage (2026-07-20). Source documents:
`findings/prism-pql-deficiencies.md` (referenced in `triage-capture.md §source_readings`).
Finding ID: F3, severity MED. Marked PARTIAL coverage — the reference resource exists but
is incomplete. Not wave-assigned in the triage wave schedule.

This defect was unregistered as a tracking artifact prior to this stub. It existed only in
`triage-capture.md` (status: OPEN at time of triage; decisions resolved at D-1943).

## Authority

- BC-2.10.014 §Postconditions (prismql://reference static resource content): governs the
  BNF content that the reference resource MUST serve, including the requirement for JOIN
  syntax. Status: active.
- No ADR governs this specific defect.

Note (POL-39 compliance): BC-2.10.014 is cited by contract ID and §-anchor only. No version
pin is included.

## Routing

implementer (direct dispatch — no product-owner adjudication required)

BC-2.10.014 §Postconditions already specifies the JOIN BNF requirement. The implementer reads
the BC, confirms the missing JOIN syntax, and adds it to the `prismql://reference` resource
handler. No BC amendment required.

## Scope — NOT YET SPECIFIED

Acceptance criteria, Red Gate test list (`RG-001..RG-NNN`), BC-5.38.001 density check, and
task decomposition are deferred to the routed specialist. This file is a registration artifact
— it makes the defect countable, discoverable, and assigned with a named owner.

Next step owner: **implementer** (reads BC-2.10.014 §Postconditions, writes failing test, adds
JOIN BNF to resource handler).

`tdd_mode` and enumerated Red Gate list MUST be set before this story reaches `status: ready`,
per SAC-1.

## §Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 0.1 | 2026-08-03 | story-writer | Initial registration stub — defect identified in D-1889 live-demo triage (2026-07-20); unregistered prior to this artifact. |
