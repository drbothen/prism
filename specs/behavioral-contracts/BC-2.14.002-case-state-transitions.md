---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "SS-14"
capability: "CAP-022"
lifecycle_status: active
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "365fb25"
traces_to:
  - "CAP-022"
extracted_from: ".factory/specs/prd.md"
---

# BC-2.14.002: Case State Transitions — 5-State Machine with 12 Valid Transitions

## Description

Cases progress through a five-state lifecycle (New, Acknowledged, Investigating,
Resolved, Closed) governed by an exhaustive 12-transition state machine. The machine
enforces business rules around reopen semantics and disposition requirements while
preserving first-resolution timestamps for accurate MTTR computation. Every transition
generates a `StatusChange` timeline entry so the full investigation lifecycle is auditable.

Transitioning to Resolved requires a disposition to be set first (BC-2.14.006), ensuring
every resolved case carries a classification. Backward transitions to New or Acknowledged
are permanently prohibited; only Investigating is a valid reopen target, preserving the
conceptual separation between active work and terminal states.

## Preconditions
- A case exists in one of the 5 states: New, Acknowledged, Investigating, Resolved, Closed
- A state transition is requested via `update_case` (BC-2.14.003)

## Postconditions
- The state machine enforces exactly 12 valid transitions:

  **Forward (linear):**
  - New -> Acknowledged
  - Acknowledged -> Investigating
  - Investigating -> Resolved
  - Resolved -> Closed

  **Skip-ahead:**
  - New -> Investigating
  - New -> Resolved
  - New -> Closed
  - Acknowledged -> Resolved
  - Acknowledged -> Closed
  - Investigating -> Closed

  **Reopen:**
  - Resolved -> Investigating
  - Closed -> Investigating

- All other transitions are rejected with a structured error listing the current state and valid target states
- Self-transitions (e.g., New -> New) are rejected
- Backward transitions to New or Acknowledged are rejected (only "Investigating" is a valid reopen target)
- On transition to Resolved: `resolved_at` is set to current UTC timestamp (only if `resolved_at` is currently null — not overwritten on subsequent Resolved transitions after reopen cycles, preserving first resolution time)
- On transition to Closed: `closed_at` is set to current UTC timestamp
- On reopen (Resolved/Closed -> Investigating): `closed_at` is cleared to null; `resolved_at` is NOT cleared (preserves first resolution time for accurate MTTR)
- When a case transitions to Resolved status, all alerts linked to the case via `source_alert_ids` have their `resolved_at` field set to the current UTC timestamp. This enables the computed `status` field on Alert to reach `resolved`. If the case is later reopened (Resolved → Investigating), the linked alerts' `resolved_at` is cleared (set to null), returning their computed status to `acknowledged` or `open`.
- Every transition generates a `StatusChange` timeline entry with: previous status, new status, actor, timestamp

## Invariants
- The state machine is exhaustive: every (current_state, target_state) pair has a defined accept/reject outcome
- `resolved_at` is non-null if and only if the case has been resolved at least once (set on first Resolved transition, never cleared)
- `closed_at` is non-null if and only if status is Closed
- Transition to Resolved requires a disposition to be set (BC-2.14.006); transition is rejected if disposition is null

## Error Conditions
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-CASE-004` | Invalid transition (e.g., Closed -> New) | Structured error: "Cannot transition from {current} to {target}. Valid targets: [{list}]" |
| `E-CASE-005` | Self-transition (e.g., Investigating -> Investigating) | Structured error: "Case is already in {status} status" |
| `E-CASE-006` | Transition to Resolved without disposition set | Structured error: "Disposition is required before resolving a case. Set disposition via update_case first." |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-14-005 | New -> Closed (skip all intermediate states) | Valid; useful for false positive dismissal |
| EC-14-006 | Closed -> Investigating -> Resolved -> Closed (reopen cycle) | Valid; each transition logged in timeline |
| EC-14-007 | Rapid state transitions (3 transitions within 1 second) | All accepted; each generates a separate timeline entry with distinct timestamps (sub-second precision) |
| EC-14-008 | Concurrent transition requests for the same case | Serialized via RocksDB write; second request sees updated state and may fail if its transition is now invalid |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for full canonical vectors.

| Scenario | Input | Expected Output |
|----------|-------|-----------------|
| Happy path — forward linear | status=New, target=Acknowledged | Transition succeeds; timeline entry added |
| Happy path — skip-ahead | status=New, target=Closed | Transition succeeds |
| Reopen | status=Resolved, target=Investigating | Transition succeeds; closed_at cleared; resolved_at preserved |
| Invalid transition | status=Closed, target=New | `E-CASE-004` with valid targets list |
| Self-transition | status=Investigating, target=Investigating | `E-CASE-005` |
| Resolved without disposition | status=Investigating, target=Resolved, disposition=null | `E-CASE-006` |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| (placeholder) | VP to be assigned — verify all 12 valid transitions accept |
| (placeholder) | VP to be assigned — verify all invalid transitions reject with correct error |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-022 |
| L2 Invariants | DI-004, DI-025 |
| Priority | P0 |

## Changelog
| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial draft |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; renamed Error Cases → Error Conditions; added ## Changelog. |
