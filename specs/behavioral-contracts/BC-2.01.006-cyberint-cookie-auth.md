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
input-hash: "248b3b0"
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

# BC-2.01.006: Cyberint Cookie-Based Authentication and Multi-Format Timestamp Parsing

## Description

The Cyberint adapter authenticates via an `access_token` cookie injected by a Cookie RoundTripper middleware. Because Cyberint responses use inconsistent timestamp formats, the adapter employs a 4-format CyberintTime parser (ISO 8601, RFC 3339, Unix epoch seconds, Cyberint custom format) and maintains a `(Timestamp, RecordID)` 2-tuple cursor. Timestamps that cannot be parsed through any format fall back to the fetch timestamp, with the raw string preserved in `raw_extensions`.

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

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.01.006-001 | Valid access_token cookie; standard ISO 8601 timestamp in response | Record parsed; `(Timestamp, RecordID)` cursor set; alert returned |
| TV-BC-2.01.006-002 | Timestamp in Cyberint custom format (4th format) | CyberintTime parser succeeds on 4th attempt; timestamp correctly extracted |
| TV-BC-2.01.006-003 | Timestamp in unknown 5th format (DEC-015) | Parse fails; fallback to fetch timestamp; raw string in `raw_extensions`; warning logged |
| TV-BC-2.01.006-004 | HTTP 401 cookie rejection | `PrismError::Sensor` with `category: "authentication"` and token refresh suggestion |
| TV-BC-2.01.006-005 | HTTP 429 rate limit | Exponential backoff; partial results with `truncation_reason: "rate_limited"` if retries exhausted |

## Verification Properties

| VP | Verification Aspect |
|----|---------------------|
| (none) | No VP directly verifies this BC — see VP-INDEX.md for full map |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 |
| L2 Invariants | DI-012 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added inputs/input-hash/traces_to/extracted_from frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors; added ## Verification Properties; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract. |
