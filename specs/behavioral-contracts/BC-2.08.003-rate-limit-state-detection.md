---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "[pending-recompute]"
traces_to: ["CAP-008"]
extracted_from: ".factory/specs/prd.md"
origin: greenfield
subsystem: "SS-08"
capability: "CAP-008"
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

# BC-2.08.003: Rate Limit State Detection Per Sensor

## Description

The health response includes a `rate_limit` object derived from observed HTTP response headers (`X-RateLimit-Remaining`, `Retry-After`) and internal 429-response tracking, scoped per `(client_id, sensor_id)` pair per DI-008. When no rate limit headers have been observed, `remaining_requests` is null. Active rate limiting triggers a suggestion to reduce query frequency or wait for reset.

## Preconditions
- A valid `client_id` and `sensor_id` are provided
- The sensor's internal rate tracker is initialized
- At least one query has been attempted against the sensor (rate state is populated)

## Postconditions
- The health response includes `rate_limit` object with: `is_rate_limited: bool`, `remaining_requests: Option<u32>`, `reset_at: Option<DateTime<Utc>>`, `retry_after_seconds: Option<u32>`
- Rate limit state is derived from the most recent HTTP response headers (`X-RateLimit-Remaining`, `Retry-After`) and internal tracking
- If no rate limit headers have been observed, `rate_limit.is_rate_limited: false` with `remaining_requests: null` (unknown)

## Invariants
- DI-008: Client data separation -- rate limit state is tracked per (client_id, sensor_id) pair

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| None | No queries have been made yet | `rate_limit.is_rate_limited: false`, `remaining_requests: null`, `status: "no_data"` |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-08-006 | Sensor API returned HTTP 429 within the last rate window | `is_rate_limited: true` with `retry_after_seconds` from the 429 response or default 30s |
| EC-08-007 | Rate limit headers not present in sensor responses | `remaining_requests: null`; rate limit state derived only from observed 429 responses |
| FM-008 | Active rate limiting persisting across multiple requests | Health report includes `suggestion: "Reduce query frequency or wait for rate window reset"` |

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| Sensor returned HTTP 429 in last rate window | `is_rate_limited: true`, `retry_after_seconds` set | error |
| No queries made yet | `is_rate_limited: false`, `remaining_requests: null`, `status: "no_data"` | edge-case |
| Sensor responses include no rate limit headers | `remaining_requests: null`; state from 429s only | edge-case |

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vector tables.

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| (no matching VP) | Rate limit state is scoped per (client_id, sensor_id) | integration test |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-008 |
| L2 Invariants | DI-008 |
| L2 Failure Modes | FM-008 |
| Priority | P1 |

## Changelog
| Version | Date | Burst | Author | Change |
|---------|------|-------|--------|--------|
| 1.0 | 2026-04-14 | cycle-1 | product-owner | Initial draft |
| 1.1 | 2026-04-20 | pre-build-sweep | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
