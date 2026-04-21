---
document_type: behavioral-contract
level: L3
version: "1.3"
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
input-hash: "8e43eb2"
traces_to:
  - "CAP-022"
extracted_from: ".factory/specs/prd.md"
---

# BC-2.14.006: Disposition Assignment — Required on Resolved Transition

## Description

Disposition is the analyst's formal classification of a case outcome, represented as
a tagged enum with four variants (TruePositive, FalsePositive, Benign, Inconclusive),
each carrying optional per-variant metadata. Disposition may be set or changed at any
time during the case lifecycle; it is required before a case can transition to Resolved,
ensuring every closed investigation carries an explicit outcome classification.

Every disposition change is recorded in the timeline, providing a full audit trail of
classification decisions even when analysts revise their assessment.

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

## Error Conditions
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

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for full canonical vectors.

| Scenario | Input | Expected Output |
|----------|-------|-----------------|
| Happy path — set TruePositive | `disposition={variant=TruePositive, impact_level="data_exfiltration"}` | Stored; DispositionSet timeline entry |
| Happy path — resolve with Inconclusive | disposition=Inconclusive, then status=Resolved | Both succeed in one call (disposition first) |
| Change disposition | TruePositive → FalsePositive | Both timeline entries present; current=FalsePositive |
| Resolve without disposition | status=Resolved, disposition=null | `E-CASE-006` |
| Invalid variant | `disposition={variant="Unknown"}` | `E-CASE-010` with valid variants list |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| VP-053 | Resolved case always has non-null disposition — for any `CaseRecord` produced by `advance_case_state(case, Resolved)`, `record.disposition.is_some()` holds; `advance_case_state` returns Err(E-CASE-006) when `case.disposition.is_none()`; no `CaseRecord` can have `status = Resolved` AND `disposition = None`. Method: Kani. Priority: P0. |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-022 |
| L2 Invariants | DI-004 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-74-fix | 2026-04-20 | product-owner | Resolved (placeholder) row in ## Verification Properties per pass-74 VP-TBD decision matrix extension. |
| 1.2 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; renamed Error Cases → Error Conditions; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial draft |
