---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "Alert & Case Management"
capability: "CAP-022"
---

# BC-2.14.001: `create_case` MCP Tool — Create Case from One or More Alerts

## Preconditions
- The `create_case` MCP tool is invoked with required parameters: `title` (string, 1-256 chars), `client_id` (the owning client)
- Optional parameters: `description` (string), `alert_ids` (array of alert UUIDs to link), `severity` (low/medium/high/critical, default inferred from highest-severity linked alert or "medium" if no alerts), `assignee` (analyst identifier)
- The `case.write` capability is allowed for the invoking context
- All referenced `alert_ids` exist in the alerts store and belong to the specified `client_id`

## Postconditions
- A new `Case` is created with:
  - `id`: UUID v7 (time-sortable)
  - `client_id`: owning client
  - `title`, `description`: from parameters
  - `status`: `New` (initial state)
  - `severity`: from parameter or inferred from linked alerts
  - `assignee`: from parameter or null
  - `source_alert_ids`: deduplicated list of linked alert IDs
  - `annotations`: empty
  - `timeline`: initial entry of type `Created` with actor, timestamp, and description
  - `created_at`, `updated_at`: current UTC timestamp
  - `closed_at`: null
  - `disposition`: null
- The case is persisted to the RocksDB `cases` domain (BC-2.14.009)
- An MCP notification is broadcast: `notifications/resources/updated` with `uri: "prism://cases"` (MCP-standard resource notification pattern, consistent with BC-2.13.005 alert notifications)
- An audit entry is emitted (DI-004)
- The `create_case` tool is gated by `case.write` capability and follows the hidden-tools pattern (BC-2.04.005)

## Invariants
- DI-004: Audit completeness
- DI-008: Client data separation -- a case belongs to exactly one client; linked alerts must belong to the same client
- Cases always start in `New` status

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-ALERT-001` | `alert_ids` contains an ID that does not exist | Structured error listing invalid IDs |
| `E-CASE-014` | `alert_ids` contains an alert belonging to a different client | Structured error: "Alert {id} belongs to client {other}, not {client_id}" |
| `E-FLAG-001` | `case.write` capability denied | Structured error (BC-2.04.015) |
| `E-CASE-015` | `title` is empty or exceeds 256 characters | Structured error with validation details |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-14-001 | Create case with 0 alert_ids | Valid; case created with empty alert list (manual investigation) |
| EC-14-002 | Create case with duplicate alert_ids | Deduplicated; each alert linked once |
| EC-14-003 | Alert already linked to another case | Alert can be linked to multiple cases; no uniqueness constraint on alert-to-case linkage |
| EC-14-004 | Create case with severity "critical" but all linked alerts are "low" | Explicit severity parameter takes precedence over inference |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-022 |
| L2 Invariants | DI-004, DI-008 |
| Priority | P0 |
