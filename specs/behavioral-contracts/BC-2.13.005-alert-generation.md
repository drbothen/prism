---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "Detection Engine"
capability: "CAP-020"
---

# BC-2.13.005: Alert Generation — Interpolate Template, Persist Alert, Broadcast via MCP Notification

## Preconditions
- A detection rule (single-event, correlation, or sequence) has matched, producing a match result with trigger event UIDs and optional extra variables (count, window, step data)

## Postconditions
- An `Alert` is created with:
  - `id`: UUID v7 (time-sortable)
  - `rule_id`, `rule_name`: from the matching rule's meta block
  - `severity`: from the rule's meta severity
  - `rule_type`: Single/Correlation/Sequence
  - `title`: interpolated from the rule's alert title template
  - `description`: interpolated from the rule's alert description template
  - `client_id`: from the triggering record's context
  - `trigger_event_uids`: event UIDs that caused the alert
  - `mitre_technique`: from rule meta `mitre` field (optional)
  - `created_at`: current UTC timestamp
- **Template interpolation** uses a four-level resolution chain:
  1. Extra variables (correlation-specific): `{count}`, `{window}`
  2. Step-scoped variables (sequence-specific): `{step_name.field}`, `{step_name.count}`
  3. Event field variables: `{src_endpoint.ip}`, `{user.name}` resolved via OCSF field paths
  4. Unresolved variables render as literal `{variable_name}` (no error, no silent empty string)
- The alert is persisted to the RocksDB `alerts` domain (BC-2.13.012)
- An MCP notification is broadcast: `notifications/resources/updated` with `uri: "prism://alerts/{alert_id}"` and the alert summary (id, rule_name, severity, client_id, title). This follows the MCP-standard resource notification pattern rather than a custom notification method.
- An audit entry is emitted for the alert generation event

## Invariants
- Every alert is persisted before the notification is broadcast (persist-then-notify ordering)
- Alert IDs are globally unique and time-sortable
- Template interpolation never fails: unresolved variables are rendered as literals

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-STORE-002` | RocksDB write failure during alert persistence | Alert is logged to stderr as fallback; notification is still broadcast with a `persistence_failed: true` flag |
| — | MCP notification broadcast has no subscribers | Notification dropped silently; alert is still persisted (not an error condition) |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-13-017 | Template references field not present in trigger event | Variable rendered as literal: `"Login from {src_endpoint.ip}"` if IP is missing |
| EC-13-018 | Correlation alert with 100+ trigger event UIDs | All UIDs included; no truncation |
| EC-13-019 | Same rule fires for 3 different clients in one scheduled execution | 3 separate alerts created, each with its own client_id |
| EC-13-020 | Alert severity is "critical" | Alert is persisted and broadcast identically to other severities; severity-based routing is the consumer's responsibility |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-020 |
| L2 Invariants | DI-004, DI-008 |
| Priority | P0 |
