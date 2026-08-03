---
document_type: story
story_id: "DEFECT-OCSF-STATUS-VOCAB-001"
title: "over-claimed OCSF enum-normalization contract — enum_value_map required for vocabulary enforcement"
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
origin_finding: "F5 (D-1889 live-demo triage 2026-07-20)"
origin_cascade: "D-1889 live-demo triage (2026-07-20); decisions resolved at D-1943"
cycle: "v1.0.0-greenfield"
phase: 3
track: "Platform Engineering"
subsystems: [SS-02]
behavioral_contracts:
  - BC-2.02.013
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

# DEFECT-OCSF-STATUS-VOCAB-001: over-claimed OCSF enum-normalization contract — enum_value_map required for vocabulary enforcement

## Problem

The implementation over-claims the OCSF enum-label normalization contract. It performs
case normalization on enum field values (for example, `status`, `severity`) but does not
constrain normalized output to the OCSF-defined vocabulary for those fields. Enum values
outside the OCSF-defined set pass through without validation or mapping to the canonical
OCSF enum label. The runtime therefore claims OCSF conformance for these fields while
delivering non-conforming values for inputs outside the expected vocabulary.

An `enum_value_map` lookup mechanism is needed to close this gap — analogous in purpose to
the runtime lookup governed by BC-2.02.010. The scope and design of this mechanism requires
architect adjudication before product-owner can author the corrective BC amendment.

BC-2.02.013 §Postconditions governs the canonical-case normalization behavior. ADR-047
§Decision (status: accepted) governs the adapter-boundary OCSF enum-label normalization
policy. Both are implicated in the fix.

## Origin

Identified in the D-1889 live-demo triage (2026-07-20). Source documents:
`findings/prism-pql-deficiencies.md` (referenced in `triage-capture.md §source_readings`).
Finding ID: F5, severity MED. Marked PARTIAL coverage — case normalization exists but the
vocabulary enforcement contract is over-claimed. Not wave-assigned in the triage wave schedule.

This defect was unregistered as a tracking artifact prior to this stub. It existed only in
`triage-capture.md` (status: OPEN at time of triage; decisions resolved at D-1943).

## Authority

- BC-2.02.013 §Postconditions (OCSF enum-label canonical-case normalization): governs the
  required behavior for enum field normalization at the adapter boundary. Status: active.
- ADR-047 §Decision (PrismQL case-sensitivity policy and adapter-boundary OCSF enum-label
  normalization): governs the normalization policy that the `enum_value_map` mechanism must
  implement. Status: **accepted**.

## Routing

architect + product-owner → story-writer → implementer

1. **Architect**: adjudicate the `enum_value_map` mechanism design — scope, crate placement,
   relationship to BC-2.02.010 runtime lookup, and whether an ADR amendment to ADR-047 is
   required.
2. **Product-owner**: author BC-2.02.013 corrective amendment based on architect adjudication.
3. **Story-writer**: decompose the amended spec into implementable stories.
4. **Implementer**: TDD delivery.

## Scope — NOT YET SPECIFIED

Acceptance criteria, Red Gate test list (`RG-001..RG-NNN`), BC-5.38.001 density check, and
task decomposition are deferred to the routed specialist chain. This file is a registration
artifact — it makes the defect countable, discoverable, and assigned with a named owner.

Next step owner: **architect** (mechanism adjudication; may require ADR-047 amendment).
After architect adjudication: **product-owner** (BC-2.02.013 amendment).
After BC amendment: **story-writer** (story decomposition), then **implementer**.

`tdd_mode` and enumerated Red Gate list MUST be set by product-owner when this story (or
its decomposed successors) reaches `status: ready`, per SAC-1.

## §Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 0.1 | 2026-08-03 | story-writer | Initial registration stub — defect identified in D-1889 live-demo triage (2026-07-20); unregistered prior to this artifact. |
