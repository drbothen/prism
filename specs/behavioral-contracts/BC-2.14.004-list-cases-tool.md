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

# BC-2.14.004: `list_cases` MCP Tool — Filter by Status, Client, Severity, Assignee

## Preconditions
- The `list_cases` MCP tool is invoked
- Optional parameters: `client_id` (filter to one client or null for all), `status` (filter to one or more statuses), `severity` (filter to one or more severities), `assignee` (filter to assigned analyst), `limit` (max results, default 25, max 100), `sort_by` (one of: `created_at`, `updated_at`, `severity`; default `updated_at`), `sort_order` (`asc` or `desc`, default `desc`)

## Postconditions
- Returns an array of case summaries, each containing: `id`, `client_id`, `title`, `status`, `severity`, `assignee`, `disposition` (if set), `source_alert_count` (number of linked alerts), `annotation_count`, `created_at`, `updated_at`, `closed_at`
- Filters are combined with AND semantics: `status: ["New", "Investigating"]` AND `severity: ["critical"]` returns only critical cases in New or Investigating status
- If `client_id` is null: cases from all configured clients are returned, each with `client_id` provenance
- Results are sorted by the specified field and order
- If more cases exist than `limit`, response includes `is_truncated: true` and `total_available`
- An audit entry is emitted (DI-004)
- This is a read-only tool -- always visible in `tools/list`

## Invariants
- DI-004: Audit completeness
- DI-008: Client data separation -- cross-client listing includes per-case `client_id`

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-MCP-004` | `client_id` is not a valid configured client | Structured error |
| `E-CASE-009` | Invalid `status` or `severity` value | Structured error with valid enum values |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-14-012 | No cases exist | Empty array, not an error |
| EC-14-013 | Filter produces 0 matching cases | Empty array with `total_available: 0` |
| EC-14-014 | `client_id: null` with 50 clients, 1000 total cases | Returns up to `limit` cases across all clients, sorted as specified |
| EC-14-015 | Sort by `severity` | Ordered: critical > high > medium > low (desc) or low > medium > high > critical (asc) |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-022 |
| L2 Invariants | DI-004, DI-008 |
| Priority | P0 |
