---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-01"
capability: "CAP-001"
lifecycle_status: active
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

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 |
| L2 Invariants | DI-004 |
| Priority | P0 |
