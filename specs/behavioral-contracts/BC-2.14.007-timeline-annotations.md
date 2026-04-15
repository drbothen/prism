---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "Case Management"
capability: "CAP-022"
---

# BC-2.14.007: Timeline Annotations — 5 Types: Note, StatusChange, AlertLink, EvidenceLink, OtImpact

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

## Error Cases
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

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-022 |
| L2 Invariants | DI-004 |
| Priority | P0 |
