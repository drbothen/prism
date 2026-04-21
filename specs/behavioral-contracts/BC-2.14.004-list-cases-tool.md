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
input-hash: "3ff257e"
traces_to:
  - "CAP-022"
extracted_from: ".factory/specs/prd.md"
---

# BC-2.14.004: `list_cases` MCP Tool — Filter by Status, Client, Severity, Assignee

## Description

The `list_cases` MCP tool provides paginated, filterable access to case summaries
across one or all clients. Filters are combined with AND semantics, results are
sortable by multiple fields, and truncation metadata is included when the result set
exceeds the requested limit. The tool is read-only and always visible in `tools/list`
without capability gating.

All invocations emit an audit entry per DI-004, and cross-client listings include
per-case `client_id` provenance to satisfy DI-008.

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

## Error Conditions
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

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for full canonical vectors.

| Scenario | Input | Expected Output |
|----------|-------|-----------------|
| Happy path — no filters | `client_id="acme"` | All acme cases up to limit=25, sorted by updated_at desc |
| Filter by status | `status=["New","Investigating"], severity=["critical"]` | Only critical New/Investigating cases |
| Empty result | filters match nothing | Empty array, `total_available: 0` |
| Truncation | 200 cases exist, limit=25 | 25 cases returned, `is_truncated: true`, `total_available: 200` |
| Cross-client | `client_id: null` | Cases from all clients, each with client_id provenance |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| (none) | AND-filter semantics are trivial iterator conjunction; truncation metadata is integration behavior of the RocksDB scan wrapper; no pure-function invariant warrants a formal VP. |

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
