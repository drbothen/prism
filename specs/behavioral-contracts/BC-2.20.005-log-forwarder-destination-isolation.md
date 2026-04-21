---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-04-21T00:00:00Z
phase: 2-patch
origin: greenfield
subsystem: "SS-20"
capability: "CAP-035"
lifecycle_status: active
introduced: cycle-1-pass-80
modified: 2026-04-21
deprecated: ~
deprecated_by: ~
replacement: ~
retired: ~
removed: ~
removal_reason: ~
inputs:
  - ".factory/specs/architecture/observability.md"
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "[md5]"
traces_to: ["CAP-035"]
extracted_from: ".factory/specs/architecture/observability.md"
---

# BC-2.20.005: Log Forwarder Destination Isolation — Single Failed Destination Must Not Block Others

## Description

When multiple external log forwarding destinations are configured, each destination
operates independently with its own in-memory queue, delivery task, error state, and
retry schedule. A delivery failure, connection timeout, authentication error, or WASM
panic for destination A must not affect the delivery of log entries to destination B.
This isolation ensures that an unreachable Datadog endpoint does not prevent Splunk
from receiving its entries, and that a misconfigured webhook does not starve a healthy
OTLP collector.

## Preconditions

- Two or more `[[server.log_forward]]` destinations are configured
- At least one destination experiences a delivery failure

## Postconditions

- Each destination has an independent delivery task (separate async Tokio task per
  destination), independent in-memory queue, and independent retry backoff state
- A delivery failure for destination A:
  - Does NOT delay enqueue of entries for destination B
  - Does NOT delay delivery attempts for destination B
  - Does NOT modify destination B's queue, error count, or backoff state
  - Does NOT cause destination B to skip entries
- Per-destination error state includes: consecutive_failures count, last_error, backoff_until timestamp
- Quarantine: after 10 consecutive delivery failures for destination A, A is quarantined
  for a configurable cool-down period (default 1 hour) before retry resumes
- While destination A is quarantined:
  - New entries for A are still enqueued (subject to queue cap per BC-2.20.003)
  - Delivery attempts for A are suspended until `backoff_until` passes
  - Destination B is completely unaffected
- Maximum 5 concurrent active (non-quarantined) forwarders per observability.md constraint

## Invariants

- Isolation is enforced at the task level (separate Tokio tasks, no shared state between
  delivery tasks)
- Destination A's consecutive_failures counter does not affect any other destination's counter
- Quarantine state is local to each destination struct

## Error Conditions

| Error | Condition | Behavior |
|-------|-----------|----------|
| Destination A: 10 consecutive failures | Quarantine threshold reached | A quarantined for `cool_down` (default 3600s); WARN to local sink; B unaffected |
| Destination A: delivery task panics | Tokio task panic | Task is restarted with backoff; B's task continues uninterrupted |
| Destination A: WASM plugin `forward-batch()` trap | wasmtime error boundary | Error caught; A retries; B not involved |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-20-020 | 5 destinations configured; 1 goes offline | 4 continue delivering; 1 accumulates drops per queue cap; system not degraded overall |
| EC-20-021 | Destination A quarantined; new entries still arrive | A's queue continues to accumulate (subject to cap); when A comes out of quarantine, it delivers from its current queue position |
| EC-20-022 | Destinations A and B both healthy; A takes 5s per batch (slow network); B is fast | A and B deliver at their own rates; B is not blocked waiting for A's 5s batches |
| EC-20-023 | Only 1 destination configured; it fails | Standard retry/quarantine for that destination; no isolation boundary issue (single-destination baseline) |
| EC-20-024 | 6th destination configured (exceeds 5-destination cap) | Config validation error at load time; 6th destination not registered; first 5 unaffected |

## Canonical Test Vectors

| ID | Input | Expected Output | Notes |
|----|-------|----------------|-------|
| TV-20-005-isolation | Destinations: `datadog` (HTTP 503 loop), `splunk` (healthy); 50 entries emitted | `splunk` delivers all 50; `datadog` retries and accumulates; no cross-contamination | Core isolation |
| TV-20-005-quarantine | `datadog` fails 10 consecutive batches | `datadog` quarantined; WARN emitted; `splunk` delivery unaffected | Quarantine threshold |
| TV-20-005-concurrent | 5 destinations; all healthy; 100 entries emitted | All 5 enqueue and deliver 100 entries independently | 5-destination maximum |
| TV-20-005-cap | Destinations A and B at cap simultaneously | Each independently drops oldest entry; each emits own WARN; no shared drop logic | EC-20-020 + BC-2.20.003 |

## Verification Properties

| VP ID | Description | Verification Method |
|-------|-------------|---------------------|
| VP-TBD-20-005 | Under concurrent operation of N destinations where M < N fail, the remaining N-M destinations' delivery counts are unaffected by the M failures | Integration test (mock HTTP receiver; inject failures on destination A; assert destination B delivery_count == expected) |

## Related BCs

- BC-2.20.001 — Recursive Prevention (per-destination local sink routing)
- BC-2.20.003 — Queue Cap (each destination has its own bounded queue)
- BC-2.20.004 — Credential Resolution (each destination resolves independently)

## Architecture Anchors

- `specs/architecture/observability.md` §Forwarding Guarantees — "Maximum 5 concurrent forwarders"
- `specs/architecture/observability.md` §Built-in Forwarder Types — independent forwarder type table
- `specs/architecture/observability.md` §External Log Forwarding — per-destination `[[server.log_forward]]` config

## Story Anchor

S-5.09 — prism-mcp: External Log Forwarding Subsystem

## VP Anchors

TBD — integration test in `tests/log_forwarding_tests.rs`

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-035 (Diagnostic Log Forwarding) |
| ADR | observability.md §Forwarding Guarantees |
| Story | S-5.09 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.1 | pass-80-follow-on | 2026-04-21 | product-owner | Re-anchored CAP-025 → CAP-035 (business-analyst created CAP-035 post-hoc per pass-80 F80-002 follow-on); removed Capability Anchor Note; added capabilities.md to inputs |
| 1.0 | pass-80-remediation | 2026-04-21 | product-owner | Initial contract — F80-002 gap closure |
