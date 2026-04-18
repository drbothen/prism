---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "Sensor Adapters"
capability: "CAP-001"
---

# BC-2.01.006: Cyberint Cookie-Based Authentication and Multi-Format Timestamp Parsing

## Preconditions
- Cyberint sensor is configured with an `access_token` credential
- The `access_token` is injected as a cookie via the Cookie RoundTripper middleware

## Postconditions
- All Cyberint API requests include the `access_token` cookie header
- Timestamps in Cyberint responses are parsed using the CyberintTime 4-format parser (ISO 8601, RFC 3339, Unix epoch seconds, Cyberint custom format)
- Cursor is a `(Timestamp, RecordID)` 2-tuple extracted from each alert/asset record

## Invariants
- DI-012: Sealed auth trait -- Cyberint cookie auth cannot be composed with other auth mechanisms

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::Sensor` | Cookie auth rejected (HTTP 401 or 403) | `category: "authentication"`, suggestion: "Verify Cyberint access_token in credential store; token may have expired" |
| `PrismError::Sensor` | Cyberint API returns HTTP 429 (rate limited) | Backoff with exponential retry (2s base, 30s max); if exhausted, return partial results with `truncation_reason: "rate_limited"` |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-015 | Timestamp in unexpected 5th format not covered by CyberintTime parser | Raw string preserved in `raw_extensions`; OCSF `time` field set to fetch timestamp as fallback; warning logged |
| EC-01-009 | Customer ID derived from API URL subdomain changes | Config validation at startup detects mismatch; existing cursor state is invalidated via fingerprint check |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 |
| L2 Invariants | DI-012 |
| Priority | P0 |
