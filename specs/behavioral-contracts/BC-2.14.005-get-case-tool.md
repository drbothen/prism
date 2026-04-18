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

# BC-2.14.005: `get_case` MCP Tool — Full Case Detail with Timeline and Linked Alerts

## Preconditions
- The `get_case` MCP tool is invoked with required parameters: `case_id` (UUID), `client_id`

## Postconditions
- Returns the full case record including:
  - **Core fields:** `id`, `client_id`, `title`, `description`, `status`, `severity`, `assignee`, `disposition` (with per-variant metadata), `created_at`, `updated_at`, `closed_at`
  - **Linked alerts:** full array of `source_alert_ids` with summary for each alert (rule_name, severity, created_at, title)
  - **Timeline:** complete array of timeline entries in chronological order, each with: `event_type`, `description`, `actor`, `timestamp`
  - **Annotations:** complete array of annotations, each with: `type`, `content`, `author`, `timestamp`
  - **Metrics:** `mttd` (mean time to detect, if calculable), `mttr` (mean time to resolve, if case is resolved/closed), `time_in_current_status` (duration since last status change)
- The case must belong to the specified `client_id`
- An audit entry is emitted (DI-004)
- This is a read-only tool -- always visible in `tools/list`

## Invariants
- DI-004: Audit completeness
- DI-008: Client data separation -- case must belong to the specified client_id

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-CASE-001` | Case does not exist | Structured error |
| `E-CASE-008` | Case belongs to a different client than specified | Structured error |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-14-016 | Case with 100+ timeline entries | All entries returned; no pagination within a single case's timeline |
| EC-14-017 | Linked alert was deleted (orphaned reference) | Alert summary shows `deleted: true` with original alert_id; no error |
| EC-14-018 | Case in New status with no annotations, no linked alerts | All arrays empty; metrics show `mttd: null`, `mttr: null` |
| EC-14-019 | Resolved case | `mttr` calculated as `resolved_at - created_at`; `mttd` calculated if linked alerts have `created_at` earlier than case `created_at` |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-022 |
| L2 Invariants | DI-004, DI-008 |
| Priority | P0 |
