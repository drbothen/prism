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
input-hash: "76729b7"
traces_to:
  - "CAP-022"
extracted_from: ".factory/specs/prd.md"
---

# BC-2.14.007: Timeline Annotations — 5 Types: Note, StatusChange, AlertLink, EvidenceLink, OtImpact

## Description

Case annotations form the investigation narrative. Five annotation types are defined:
three are user-created (note, evidence_link, ot_impact) and two are system-generated
(status_change, alert_link). System-generated annotations are created automatically by
state machine transitions and alert linking operations; they cannot be created via the
MCP tool by users. All annotations are append-only and immutable after creation,
preserving the integrity of the investigation audit trail.

The ot_impact type is a first-class annotation for MSSP environments managing OT/ICS
clients, capturing operational technology impact assessments directly in the case timeline.

## Preconditions
- A case exists and an annotation is being added via `update_case` (BC-2.14.003)
- The annotation has a `type` (one of 5 valid types) and `content` (string, 1-10000 chars)

## Postconditions
- **5 annotation types:**
  1. **note** -- free-form investigation notes; content is plain text or markdown
  2. **status_change** -- auto-generated when status transitions; content describes the transition (not user-created)
  3. **alert_link** -- auto-generated when alerts are linked; content contains alert summary (not user-created)
  4. **evidence_link** -- user-added reference to external evidence (e.g., URL to screenshot, ticket, artifact); content is a URI or description
  5. **ot_impact** -- OT/ICS-specific impact assessment for industrial clients; content describes operational technology impact (e.g., "PLC communication disrupted on line 3", "SCADA visibility lost for 15 minutes")
- Each annotation is stored with: `type`, `content`, `author` (analyst identifier or "system" for auto-generated), `timestamp` (UTC)
- User-created annotations are types: `note`, `evidence_link`, `ot_impact`
- System-generated annotations are types: `status_change`, `alert_link` (created automatically by state transitions and alert linking, never directly by users)
- An `AnnotationAdded` timeline entry is generated for each new annotation
- Annotations are append-only: once added, they cannot be edited or deleted (immutable audit trail)

## Invariants
- Annotations are immutable after creation
- Timeline entries are chronologically ordered and never reordered
- System-generated annotations cannot be created via the MCP tool (only via internal state transitions)

## Error Conditions
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-CASE-011` | Invalid annotation type | Structured error with valid types: note, evidence_link, ot_impact |
| `E-CASE-012` | Annotation content is empty or exceeds 10000 characters | Structured error with length constraint |
| `E-CASE-013` | User attempts to create a `status_change` or `alert_link` annotation | Structured error: "Annotation type '{type}' is system-generated and cannot be created manually" |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-14-024 | Case has 500 annotations | All stored and returned; no pagination within annotations |
| EC-14-025 | `ot_impact` annotation on a case for a non-OT client | Valid; annotation type is not restricted by client type |
| EC-14-026 | Two annotations added within the same millisecond | Both stored; ordering is deterministic (insertion order within the same timestamp) |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for full canonical vectors.

| Scenario | Input | Expected Output |
|----------|-------|-----------------|
| Happy path — note | `type=note, content="Investigating lateral movement"` | Annotation stored; AnnotationAdded timeline entry |
| Happy path — ot_impact | `type=ot_impact, content="PLC line 3 offline"` | Annotation stored; author=analyst |
| System type rejected | `type=status_change` (user-initiated) | `E-CASE-013` |
| Invalid type | `type=invalid_type` | `E-CASE-011` with valid types list |
| Content too long | content with 10001 chars | `E-CASE-012` |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| (none) | Annotation immutability is an append-only data structure property enforced by absence of mutation methods (code review); system-type rejection is a trivial 2-branch enum check covered by unit test; no pure-function invariant warrants a formal VP. |

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
