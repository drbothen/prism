---
document_type: behavioral-contract
level: L3
version: "1.0"
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
---

# BC-2.14.006: Disposition Assignment — Required on Resolved Transition

## Preconditions
- A case exists and a disposition is being set via `update_case` (BC-2.14.003)
- OR a case is being transitioned to `Resolved` status

## Postconditions
- Disposition is a tagged enum with four variants, each with optional per-variant metadata:
  - **TruePositive** `{ impact_level: String }` -- confirmed security incident; `impact_level` describes business impact (e.g., "data_exfiltration", "service_disruption", "compliance_violation")
  - **FalsePositive** `{ reason: String }` -- not a real incident; `reason` explains why (e.g., "legitimate_admin_activity", "test_traffic", "misconfigured_rule")
  - **Benign** `{ explanation: String }` -- real activity but not malicious; `explanation` describes context
  - **Inconclusive** -- insufficient evidence to determine; no additional metadata
- Disposition can be set at any time (independent of status transitions)
- Disposition can be changed (overwritten) at any time; the previous disposition is recorded in the timeline
- Disposition is **required** before transitioning to `Resolved` status; attempting to resolve without a disposition produces `E-CASE-006`
- Disposition is **not required** for `Closed` status (allowing administrative closure without classification)
- A `DispositionSet` timeline entry is generated with the disposition variant and metadata

## Invariants
- A resolved case always has a non-null disposition
- Disposition changes are tracked in the timeline (full audit trail of classification changes)
- Per-variant metadata fields are optional (can be set to empty string)

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-CASE-006` | Transition to Resolved without disposition | Structured error with guidance to set disposition first |
| `E-CASE-010` | Invalid disposition variant name | Structured error with valid variants |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-14-020 | Set disposition to TruePositive, then change to FalsePositive | Valid; both changes recorded in timeline; current disposition is FalsePositive |
| EC-14-021 | Set disposition on a Closed case | Valid; disposition can be set retroactively for reporting |
| EC-14-022 | Resolve with Inconclusive disposition (no metadata) | Valid; Inconclusive requires no additional fields |
| EC-14-023 | TruePositive with empty impact_level | Valid; metadata fields are optional |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-022 |
| L2 Invariants | DI-004 |
| Priority | P0 |
