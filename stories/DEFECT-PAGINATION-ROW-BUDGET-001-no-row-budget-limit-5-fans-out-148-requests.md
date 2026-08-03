---
document_type: story
story_id: "DEFECT-PAGINATION-ROW-BUDGET-001"
title: "No row budget enforcement; LIMIT 5 fans out to 148 upstream pagination requests"
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
origin_finding: "F12 = G3 (D-1889 triage 2026-07-20)"
cycle: "v1.0.0-greenfield"
phase: 3
track: "Platform Engineering"
behavioral_contracts:
  - BC-2.07.001
  - BC-2.07.002
  - BC-2.11.006
# BC status:
#   BC-2.07.001 (Ephemeral Cursor Pagination): status: draft, lifecycle_status: active
#   BC-2.07.002 (Pagination Token Lifecycle): status: draft, lifecycle_status: active
#   BC-2.11.006 (Query Security Limits): status: draft, lifecycle_status: active
#
# FINDING-A (MEDIUM, architect-routed): All three anchor BCs carry status: draft while
# being cited as governing contracts. The architect or product-owner should confirm
# whether these BCs are ready to anchor ACs before story decomposition begins.
#
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

# DEFECT-PAGINATION-ROW-BUDGET-001: No row budget enforcement; LIMIT 5 fans out to 148 upstream pagination requests

## Problem

No row budget exists in the query execution layer. When a user issues a query with
`LIMIT 5`, the engine does not stop fetching upstream pages once 5 rows have been
accumulated; instead it fans out to all pagination pages across all sensors before
applying the limit post-fetch. Observed in the D-1889 live demo: a `LIMIT 5` query
produced 148 upstream API requests across sensors.

This violates BC-2.07.001 §Postconditions governing cursor pagination (ephemeral
cursor must be terminated when the row budget is satisfied), BC-2.07.002 §Postconditions
governing pagination token lifecycle (tokens must not be consumed past the row budget),
and BC-2.11.006 §Postconditions governing query security limits (the engine must enforce
row budgets to prevent unbounded upstream fan-out).

The practical consequence is that a small LIMIT can trigger hundreds of upstream API
requests, exhausting rate limits, slowing responses, and creating denial-of-service risk
against tenant sensor APIs.

## Origin — D-1889 Triage (F12 = G3)

**Triage date:** 2026-07-20  
**Source findings:** `findings/prism-pql-deficiencies.md`, `findings/prism-pushdown-audit.md`  
**Triage capture:** `.factory/planning/findings-remediation-2026-07-20/triage-capture.md`
§Bucket-B table row F12

The live demo observation (`limit 5` → 148 upstream requests) is the direct evidence.
The triage cross-references this as G3 in the pushdown audit. The related finding
DEFECT-QUERY-TIMEOUT-ORPHAN-SWEEP-001 (G4) is a separate defect covering the timeout
behavior when the fan-out is cancelled; row-budget enforcement and timeout orphan cleanup
are related but distinct problems assigned to different stories.

## Authority

| Artifact | Verbatim Status | Relevant Clause |
|----------|-----------------|-----------------|
| BC-2.07.001 (Ephemeral Cursor Pagination) | `status: draft` · `lifecycle_status: active` | §Postconditions — cursor must terminate when row budget is satisfied; **draft status is a FINDING-A (MEDIUM, architect-routed)** |
| BC-2.07.002 (Pagination Token Lifecycle) | `status: draft` · `lifecycle_status: active` | §Postconditions — pagination tokens must not be consumed past the row budget; **draft status is a FINDING-A (MEDIUM, architect-routed)** |
| BC-2.11.006 (Query Security Limits) | `status: draft` · `lifecycle_status: active` | §Postconditions — row-budget enforcement must prevent unbounded upstream fan-out; **draft status is a FINDING-A (MEDIUM, architect-routed)** |

All three anchor BCs are `status: draft`. The architect or product-owner must confirm
or advance these BCs before ACs can be anchored. This is a systemic concern: three
draft BCs co-governing a single CRIT/HIGH defect may indicate that the pagination and
security-limits contracts are insufficiently mature for Wave C dispatch.

No governing ADR has been identified for the row-budget mechanism beyond existing
pagination architecture. The architect may need to determine the correct row-budget
enforcement point (fetch pipeline, cursor manager, or query planner).

## Routing

Route per triage: **architect + product-owner → story-writer → implementer**

1. **Architect adjudicates**: determines the correct enforcement point for row budget
   (fetch pipeline early-exit vs cursor manager vs query planner push-down) and
   confirms whether the three draft BCs are architecturally correct as written
2. Product-owner advances or amends BC-2.07.001, BC-2.07.002, and BC-2.11.006 from
   `draft` to `active` with explicit row-budget postconditions
3. Story-writer decomposes ACs from the active BCs
4. Implementer closes the gap under TDD

Wave C assignment is contingent on at least BC-2.11.006 reaching `active` status.

## Scope — NOT YET SPECIFIED

Acceptance criteria, Red Gate test enumeration (RG-001..RG-NNN), BC-5.38.001 density
check, `tdd_mode` declaration, task decomposition, and story-point estimate are deferred
to the architect (enforcement-point decision) and product-owner (BC advancement). This
stub registers the defect as a trackable artifact; no implementation guidance is
authored here.

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 0.1 | 2026-08-03 | story-writer | Initial registration stub from D-1889 triage (F12 = G3); FINDING-A for three draft BC anchors; no ACs or implementation guidance |
