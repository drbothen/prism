---
document_type: behavioral-contract
level: L3
version: "1.4"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-01"
capability: "CAP-001, CAP-002"
lifecycle_status: active
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "76729b7"
traces_to: ["CAP-001", "CAP-002"]
extracted_from: ".factory/specs/prd.md"
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.01.010: Partial Failure Handling for Paginated and Cross-Client Queries

## Description

When a sensor query (single-client or cross-client) encounters a failure after at least one successful page or client response, Prism returns the data already fetched rather than discarding it. The response is annotated with `truncated: true`, a `truncation_reason`, and (for cross-client queries) a `partial_failures` array listing each failed client. The cursor advances only to the last successfully delivered page, enabling safe resume on next invocation.

## Preconditions
- A sensor query (single-client or cross-client) is in progress
- At least one page or one client's query has succeeded before a failure occurs

## Postconditions
- Successfully fetched data is returned to the caller (not discarded)
- Response includes `truncated: true` when pagination was interrupted
- Response includes `truncation_reason` describing the failure (e.g., "sensor_unavailable", "rate_limited", "authentication_expired")
- For cross-client queries, `partial_failures` array lists each failed client with error category and suggestion
- Cursor advances only to the last successfully fetched and delivered page

## Invariants
- DI-001: Cursor advances only for successfully delivered pages (ephemeral in-memory cursor is not advanced beyond the last successful page)

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| N/A | HTTP 503 mid-pagination | Not a tool-level error; partial results returned with metadata |
| N/A | HTTP 429 after backoff exhaustion | Not a tool-level error; partial results returned with `truncation_reason: "rate_limited"` |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-001 | HTTP 503 after some pages fetched | Return fetched pages with `truncated: true` and `truncation_reason: "sensor_unavailable"`; cursor at last successful page |
| EC-01-014 | First page fails (no data fetched) | Empty results with full error in metadata; this is still not a tool-level error for cross-client queries |
| EC-01-015 | Network timeout during a single-client query | Return any fetched pages as partial; if no pages fetched, return structured error with timeout details and retry suggestion |

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.01.010-001 | 3-page query; HTTP 503 on page 3 | Pages 1-2 returned; `truncated: true`; `truncation_reason: "sensor_unavailable"`; cursor at end of page 2 |
| TV-BC-2.01.010-002 | Cross-client query; client B credentials expired | Client A results returned; `partial_failures` lists client B with `category: "authentication"` |
| TV-BC-2.01.010-003 | HTTP 429 after retry exhaustion | Partial results with `truncation_reason: "rate_limited"`; not a tool-level error |
| TV-BC-2.01.010-004 | First page fails immediately | Empty `events` array; error metadata populated; not a tool-level error for cross-client |

## Verification Properties

| VP | Verification Aspect |
|----|---------------------|
| (none) | No VP directly verifies this BC — see VP-INDEX.md for full map |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-001, CAP-002 |
| L2 Invariants | DI-001 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.4 | pass-72-fix | 2026-04-20 | product-owner | Reordered changelog rows to fully descending (CRIT-001 class scope expansion from pass-71 MED-002 fix). |
| 1.3 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.2 | pre-build-sweep | 2026-04-20 | product-owner | Normalized capability frontmatter from YAML array to string scalar per corpus convention (IMP-006). |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added inputs/input-hash/traces_to/extracted_from frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors; added ## Verification Properties; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract. |
