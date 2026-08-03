---
document_type: story
story_id: "DEFECT-PIPE-WHERE-PUSHDOWN-001"
title: "Ast::Pipe and Ast::Filter receive zero filter pushdown"
wave: "C"
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
  - findings/prism-pushdown-audit.md
origin_finding: "F7 = G1 (D-1889 triage 2026-07-20)"
cycle: "v1.0.0-greenfield"
phase: 3
track: "Platform Engineering"
behavioral_contracts:
  - BC-2.11.007
  - BC-2.11.020
# BC status:
#   BC-2.11.007 (Sensor Filter Push-Down): status: active, lifecycle_status: active
#   BC-2.11.020 (SQL-to-Pipe Composition — SqlPipe AST and Forbid-Both Dual-Limit): status: active, lifecycle_status: active
# S-7.01: behavioral_contracts non-empty; status may advance to ready after ACs are authored.
verification_properties: []
depends_on: []
blocks: []
points: 0
risk: HIGH
holdout_scenarios: []
assumption_validations: []
risk_mitigations: []
---

# DEFECT-PIPE-WHERE-PUSHDOWN-001: Ast::Pipe and Ast::Filter receive zero filter pushdown

## Problem

Queries parsed as `Ast::Pipe(_)` (pipe-mode: `FROM t | where …`) and `Ast::Filter(_)`
(filter-mode: bare predicate expressions) do not receive predicate pushdown to the sensor
fetch layer. Filters that should be pushed down as query parameters are instead evaluated
post-fetch, causing full-dataset upstream fan-out when a narrower fetch was possible.

This is a correctness gap against BC-2.11.007 §Postconditions which requires
filter pushdown across all supported query AST modes, and against BC-2.11.020 which
governs correct pipe-stage execution semantics that depend on pushdown propagating
through the pipe head.

## Origin — D-1889 Triage (F7 = G1)

**Triage date:** 2026-07-20  
**Source findings:** `findings/prism-pql-deficiencies.md`, `findings/prism-pushdown-audit.md`  
**Triage capture:** `.factory/planning/findings-remediation-2026-07-20/triage-capture.md`
§Bucket-B table row F7

The pushdown audit found that the pushdown logic covers the `Ast::Sql` arm of the AST
dispatch but the `Ast::Pipe` and `Ast::Filter` arms are not wired to the pushdown path.
Both `Ast::Pipe` (verified in `crates/prism-query/src/ast.rs` §`pub enum Ast`) and
`Ast::Filter` (same file) are live AST variants; the pushdown coverage gap is
confirmed by the audit source.

## Authority

| Artifact | Verbatim Status | Relevant Clause |
|----------|-----------------|-----------------|
| BC-2.11.007 (Sensor Filter Push-Down) | `status: active` · `lifecycle_status: active` | §Postconditions — filter extraction and pushdown for all supported query modes |
| BC-2.11.020 (SQL-to-Pipe Composition) | `status: active` · `lifecycle_status: active` | §Invariants — pipe-stage execution correctness depends on upstream pushdown |

No governing ADR has been identified for this defect. The product-owner may need to
amend BC-2.11.007 §Postconditions to explicitly enumerate `Ast::Pipe` and `Ast::Filter`
pushdown arms before ACs can be written.

## Routing

Route per triage: **product-owner → implementer**

1. Product-owner amends BC-2.11.007 to cover `Ast::Pipe` and `Ast::Filter` pushdown
   arms (if not already covered by existing language)
2. Story-writer decomposes ACs from the amended BC
3. Implementer closes the pushdown gap under TDD with SAP-3 e2e reachability discipline

No architect adjudication required before dispatch.

## Scope — NOT YET SPECIFIED

Acceptance criteria, Red Gate test enumeration (RG-001..RG-NNN), BC-5.38.001 density
check, `tdd_mode` declaration, task decomposition, and story-point estimate are deferred
to the product-owner (BC amendment) and story-writer (AC decomposition). This stub
registers the defect as a trackable artifact per the D-1889 triage; no implementation
guidance is authored here.

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 0.1 | 2026-08-03 | story-writer | Initial registration stub from D-1889 triage (F7 = G1); no ACs or implementation guidance |
