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
input-hash: "572c2a9"
traces_to:
  - "CAP-022"
extracted_from: ".factory/specs/prd.md"
---

# BC-2.14.003: `update_case` MCP Tool — Transition State, Set Disposition, Add Annotation

## Description

The `update_case` MCP tool is the primary mutation surface for case management. It
accepts a flexible set of optional update fields applied in a defined order: disposition
is set before status transitions so that a single call can satisfy the disposition
requirement for the Resolved transition. Each field change generates a corresponding
timeline entry, producing a continuous audit trail of all case mutations.

The tool is gated by the `case.write` capability and enforces client data separation.
All updates are persisted atomically to RocksDB, and an audit entry is emitted for
every invocation.

## Preconditions
- The `update_case` MCP tool is invoked with required parameters: `case_id` (UUID), `client_id`
- Optional parameters (at least one required): `status` (target state for transition), `disposition` (TruePositive/FalsePositive/Benign/Inconclusive with per-variant metadata), `severity` (update severity), `assignee` (update assignee), `annotation` (object with `type` and `content`), `link_alert_ids` (array of additional alert IDs to link)
- The `case.write` capability is allowed
- The case exists and belongs to the specified `client_id`

## Postconditions
- Each provided field is applied in order: disposition -> status -> severity -> assignee -> annotation -> link_alert_ids
- **Disposition update:** stored on the case; generates a `DispositionSet` timeline entry
- **Status transition:** validated against the state machine (BC-2.14.002); generates a `StatusChanged` timeline entry
- **Severity update:** generates a `PriorityChanged` timeline entry (using "priority" terminology for axiathon compatibility)
- **Assignee update:** generates a timeline entry with previous and new assignee
- **Annotation:** added to the case's annotations array; generates an `AnnotationAdded` timeline entry; annotation type must be one of the 5 valid types (BC-2.14.007)
- **Link alert IDs:** each alert is validated (exists, belongs to client), deduplicated against existing links, and added; generates an `AlertLinked` timeline entry per new alert
- `updated_at` is set to current UTC timestamp
- The updated case is persisted to RocksDB (BC-2.14.009)
- An audit entry is emitted (DI-004)
- The `update_case` tool is gated by `case.write` capability and follows the hidden-tools pattern (BC-2.04.005)

## Invariants
- DI-004: Audit completeness
- DI-008: Client data separation -- case must belong to the specified client_id
- Order of operations matters: disposition is set before status transition so that the Resolved transition's disposition requirement (BC-2.14.002) can be satisfied in a single call

## Error Conditions
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-CASE-001` | Case does not exist | Structured error |
| `E-CASE-008` | Case belongs to a different client than specified | Structured error |
| `E-CASE-004` | Invalid state transition | Structured error per BC-2.14.002 |
| `E-ALERT-001` | `link_alert_ids` contains invalid alert ID | Structured error; no partial updates (entire call fails) |
| `E-FLAG-001` | `case.write` capability denied | Structured error (BC-2.04.015) |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-14-009 | Set disposition to FalsePositive and transition to Resolved in one call | Disposition set first, then transition succeeds (disposition requirement satisfied) |
| EC-14-010 | Update with no optional parameters provided | Structured error: "At least one update field is required" |
| EC-14-011 | Set assignee to null (unassign) | Valid; generates timeline entry "Unassigned from {previous_assignee}" |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for full canonical vectors.

| Scenario | Input | Expected Output |
|----------|-------|-----------------|
| Happy path — disposition + resolve in one call | `disposition=FalsePositive, status=Resolved` | Disposition applied first; transition succeeds; two timeline entries added |
| Happy path — add annotation | `annotation={type="note", content="Investigating"}` | Annotation stored; AnnotationAdded timeline entry |
| No update fields | `case_id=X, client_id=Y` (no optional fields) | Structured error: at least one field required |
| Invalid alert link | `link_alert_ids=["nonexistent-uuid"]` | `E-ALERT-001`; no partial update |
| Wrong client | `case_id` belongs to different client | `E-CASE-008` |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| VP-052 | update_case: disposition applied before status transition — for any `CaseUpdateSpec` containing both `disposition: Some(d)` and `status: Some(Resolved)`, `apply_update_fields()` applies disposition before the status transition; a call with disposition=FalsePositive and status=Resolved succeeds when case has no prior disposition; the same call with status applied first would fail E-CASE-006. Method: Proptest. Priority: P0. |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-022 |
| L2 Invariants | DI-004, DI-008 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-74-fix | 2026-04-20 | product-owner | Resolved (placeholder) row in ## Verification Properties per pass-74 VP-TBD decision matrix extension. |
| 1.2 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; renamed Error Cases → Error Conditions; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial draft |
