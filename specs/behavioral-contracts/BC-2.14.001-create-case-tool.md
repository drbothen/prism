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
input-hash: "1e29f9d"
traces_to:
  - "CAP-022"
extracted_from: ".factory/specs/prd.md"
---

# BC-2.14.001: `create_case` MCP Tool — Create Case from One or More Alerts

## Description

The `create_case` MCP tool provides analysts with a structured entry point for
opening investigations. It creates a new `Case` record in the New state, optionally
linking one or more existing alerts, inferring severity from the highest-severity
linked alert when not explicitly provided, and seeding an initial timeline entry.
All access is gated by the `case.write` capability; the tool follows the hidden-tools
pattern so it is not surfaced to contexts without write permission.

Every created case is persisted to the RocksDB `cases` domain and an audit entry is
emitted to satisfy DI-004. An MCP `notifications/resources/updated` notification is
broadcast so AI agents can refresh their view of the cases resource.

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

## Error Conditions
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

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for full canonical vectors.

| Scenario | Input | Expected Output |
|----------|-------|-----------------|
| Happy path — minimal | `title="Suspicious login", client_id="acme"` | Case created with status=New, severity=medium, empty alert list |
| Happy path — with alerts | `title="Incident", client_id="acme", alert_ids=["uuid-1","uuid-2"], severity="high"` | Case created, both alerts linked, severity=high |
| Duplicate alert_ids | `alert_ids=["uuid-1","uuid-1"]` | Case created with `source_alert_ids=["uuid-1"]` (deduplicated) |
| Cross-client alert | `client_id="acme", alert_ids=["alert-belonging-to-beta"]` | `E-CASE-014` structured error |
| capability denied | `case.write` not in allowed capabilities | `E-FLAG-001` structured error |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| (none) | case.write gate covered transitively by VP-002 (deny-by-default capability); audit-on-create ordering covered by VP-033; no pure-function invariant distinct from integration test. |

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
