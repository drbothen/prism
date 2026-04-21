---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-01"
capability: "CAP-001"
lifecycle_status: active
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "8e43eb2"
traces_to: ["CAP-001"]
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

# BC-2.01.014: Exponential Backoff and Retry for Transient Sensor API Errors

## Description

When a sensor API call returns a transient error (HTTP 429, 500, 502, 503, 504, or network timeout), the adapter retries with exponential backoff (2s base, 30s max). HTTP 429 responses with a `Retry-After` header override the computed backoff interval. If max retries are exhausted, the adapter returns partial results annotated with `truncated: true` rather than raising a fatal error. Non-transient errors (HTTP 400, 404) are never retried.

## Preconditions
- A sensor API call returns a transient error (HTTP 429, 500, 502, 503, 504, or network timeout)
- The backoff configuration is: 2s base, 30s max, configurable max retries (0 = unlimited)

## Postconditions
- The failed request is retried with exponential backoff (2s, 4s, 8s, 16s, 30s, 30s, ...)
- HTTP 429 responses respect the `Retry-After` header if present (overrides computed backoff)
- If max retries are exhausted, partial results are returned (not a fatal error)
- Each retry attempt is logged with retry count and backoff duration

## Invariants
- DI-004: Audit completeness -- retries are logged as part of the tool invocation's audit trail

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Partial result | All retries exhausted for a transient error | Return fetched data with `truncated: true`; the specific HTTP status and retry count included in metadata |
| `PrismError::Sensor` | Non-transient error (HTTP 400, 404) | No retry; immediate structured error return |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-001 | HTTP 503 after some pages already fetched | Retry the failed page; if retries exhausted, return previously fetched pages as partial result |
| EC-01-022 | HTTP 429 with `Retry-After: 120` (2 minutes) | Respect the header; wait 120s before retry; log the wait duration |
| EC-01-023 | Network TCP connection reset mid-response | Treated as transient; full page re-requested on retry |

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.01.014-001 | HTTP 503 on page 2 of 3; retries succeed on second attempt | Page 2 fetched after 1 retry; all 3 pages returned; retry logged |
| TV-BC-2.01.014-002 | HTTP 429 with `Retry-After: 10` | 10s wait before retry (header overrides computed backoff); attempt logged |
| TV-BC-2.01.014-003 | HTTP 429 with `Retry-After: 120` | 120s wait; log records the wait duration |
| TV-BC-2.01.014-004 | Max retries exhausted on HTTP 503 | Partial results returned with `truncated: true`; HTTP status and retry count in metadata |
| TV-BC-2.01.014-005 | HTTP 404 (non-transient) | No retry; immediate `PrismError::Sensor` |

## Verification Properties

| VP | Verification Aspect |
|----|---------------------|
| (none) | No VP directly verifies this BC — see VP-INDEX.md for full map |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 |
| L2 Invariants | DI-004 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added inputs/input-hash/traces_to/extracted_from frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors; added ## Verification Properties; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract. |
