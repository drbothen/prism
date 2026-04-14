---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "Sensor Health"
capability: "CAP-008"
---

# BC-2.08.003: Rate Limit State Detection Per Sensor

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

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-008 |
| L2 Invariants | DI-008 |
| L2 Failure Modes | FM-008 |
| Priority | P1 |
