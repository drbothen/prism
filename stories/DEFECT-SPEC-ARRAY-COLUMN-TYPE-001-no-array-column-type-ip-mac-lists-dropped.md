---
document_type: story
story_id: "DEFECT-SPEC-ARRAY-COLUMN-TYPE-001"
title: "No array ColumnType variant; ip_list/mac_list values dropped at normalization"
wave: TBD
epic_id: engine-defects
priority: P1
status: draft
version: "0.1"
severity: HIGH
level: engine
producer: story-writer
timestamp: "2026-08-03"
modified: "2026-08-03"
inputs:
  - .factory/planning/findings-remediation-2026-07-20/triage-capture.md
  - findings/prism-pql-deficiencies.md
  - findings/dtu-fidelity-gaps.md
origin_finding: "F8 = GAP-2c (D-1889 triage 2026-07-20)"
cycle: "v1.0.0-greenfield"
phase: 3
track: "Platform Engineering"
behavioral_contracts: []
# BC status: pending PO authorship
# F8 requires a NEW behavioral contract (no existing BC covers array ColumnType).
# S-7.01 gate: behavioral_contracts: [] — status MUST remain draft until a product-owner
# authors and anchors a BC with canonical BC-S.SS.NNN ID for this defect.
verification_properties: []
depends_on: []
blocks: []
points: 0
risk: HIGH
holdout_scenarios: []
assumption_validations: []
risk_mitigations: []
---

# DEFECT-SPEC-ARRAY-COLUMN-TYPE-001: No array ColumnType variant; ip_list/mac_list values dropped at normalization

## Problem

`prism_core::column::ColumnType` (the canonical sensor schema API per ADR-024) has
no `Array` or list variant. Sensor spec columns declared as array types (for example
`ip_list` and `mac_list` in CrowdStrike device data) are silently dropped during OCSF
normalization because the normalizer has no way to represent or forward list-valued
columns through the pipeline. This has confirmed real-client impact: cross-sensor
identity merge on device IP/MAC data fails because the lists never reach the query
layer.

There is no story or BC that covers array ColumnType creation. This was an implicit
promise in the sensor spec work (a "promised story") that was never registered as
a trackable artifact — a Canonical Principle Rule 3 violation flagged at D-1889 triage.

## Origin — D-1889 Triage (F8 = GAP-2c)

**Triage date:** 2026-07-20  
**Source findings:** `findings/prism-pql-deficiencies.md`, `findings/dtu-fidelity-gaps.md`  
**Triage capture:** `.factory/planning/findings-remediation-2026-07-20/triage-capture.md`
§Bucket-B table row F8, §DTU Fidelity + Scenario Enhancements §GAP-2c

`ColumnType` (verified at `crates/prism-core/src/column.rs` and referenced throughout
the codebase as the canonical sensor schema API) carries no `Array` variant. `ip_list`
and `mac_list` are representative real-client columns whose values are multi-valued and
cannot be expressed as any existing `ColumnType` variant (`String`, `Integer`, `Float`,
`Boolean`, `Datetime`, `Json`). The triage records confirmed real-client impact and
identifies this as GAP-2c / GAP-2b (degenerate IP/MAC + no MAC field; blocks
cross-sensor identity merge).

## Rule-3 Disclosure — Canonical Principle Rule 3 Violation

This defect was **flagged at D-1889 triage (2026-07-20) as a Canonical Principle Rule 3
violation**: an unregistered promised story. The array ColumnType requirement was
acknowledged implicitly during sensor spec authorship but was never registered as a
trackable story, BC, or tech-debt entry. The CLAUDE.md Canonical Principle Rule 3 states:
an AI-built defect that is deferred must have (a) explicit human direction, (b) a
concrete future dependency that makes deferral necessary, and (c) attachment to a
specific future story. None of the three were present.

This stub is the **remediating registration** that satisfies condition (c): the defect
now has a trackable story ID and is no longer silently lost. Conditions (a) and (b)
remain open — they require architect adjudication (is a new ColumnType variant the right
mechanism, or is `ColumnType::Json` with a schema hint sufficient?) and PO authorship
of a governing BC before this story can advance to `status: ready`.

## Authority

| Artifact | Verbatim Status | Relevant Clause |
|----------|-----------------|-----------------|
| BC-TBD (new BC required) | — pending authorship — | Governs array ColumnType semantics, normalization behavior, and list-valued column propagation through the pipeline |
| `prism_core::column::ColumnType` | — code artifact — | Canonical sensor schema API (ADR-024); current variants: `String / Integer / Float / Boolean / Datetime / Json`; `Array` variant absent |

No governing BC exists yet. A new BC must be authored by the product-owner to specify
array ColumnType semantics before ACs can be written. The architect must also determine
whether a new variant is added to `ColumnType` or whether an existing variant (e.g.
`Json`) is extended with type metadata.

## Routing

Route per triage: **architect + product-owner → story-writer → implementer**

1. **Architect adjudicates first**: determine the correct mechanism for array-typed
   columns in `ColumnType` — new `Array(Box<ColumnType>)` variant vs `Json` with
   schema annotation vs another approach; determine `#[non_exhaustive]` and
   compile-fail perimeter implications
2. Product-owner authors new BC with ID matching `BC-S.SS.NNN` pattern, covering
   array ColumnType semantics, normalization, and query-layer propagation
3. Story-writer decomposes ACs from the new BC
4. Implementer closes the gap under TDD

Wave assignment is TBD pending architect adjudication.

## Scope — NOT YET SPECIFIED

Acceptance criteria, Red Gate test enumeration (RG-001..RG-NNN), BC-5.38.001 density
check, `tdd_mode` declaration, task decomposition, and story-point estimate are deferred
to the architect (mechanism decision) and product-owner (BC authorship). This stub
registers the defect as a trackable artifact and documents the Rule-3 violation
remediation. No implementation guidance is authored here.

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 0.1 | 2026-08-03 | story-writer | Initial registration stub from D-1889 triage (F8 = GAP-2c); Rule-3 violation remediation; no ACs or implementation guidance |
