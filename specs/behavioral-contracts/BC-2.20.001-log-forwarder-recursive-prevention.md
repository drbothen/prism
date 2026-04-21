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

# BC-2.20.001: Log Forwarder Recursive Prevention — Plugin host.log() Writes to Local Sink Only

## Description

When a WASM log-forwarding plugin invokes `host.log()` during a `forward-batch()` call,
those log entries must be written to the local stderr/file diagnostic sink only — they
must NOT be re-enqueued into any forwarder's delivery queue. This prevents an infinite
feedback loop where a delivery failure emits an error log that itself gets forwarded,
triggering another failure log, and so on. The no-recursive-forwarding guarantee applies
to all forwarder types (built-in and WASM plugin).

## Preconditions

- At least one external log destination is configured in `[[server.log_forward]]`
- The Prism diagnostic log forwarding subsystem is active
- The forwarder's delivery attempt for a batch produces a log entry (via the internal
  `tracing` subscriber or plugin `host.log()`)

## Postconditions

- Log entries produced DURING a `forward-batch()` invocation — including entries from:
  - Plugin `host.log()` calls inside the WASM sandbox
  - Internal forwarder error/warn entries (e.g., HTTP delivery failures)
  - Retry scheduling entries
  — are routed exclusively to the local stderr/rolling-file sink
- These in-flight-forwarding log entries are NEVER appended to any in-memory forwarder
  queue (including the queue for the same forwarder, any sibling forwarder, or a
  catch-all destination)
- After the `forward-batch()` call returns, normal routing resumes: new diagnostic entries
  from unrelated operations are enqueued per their destination's `min_level` filter
- No `forward-batch()` invocation triggers itself recursively (zero recursion depth on
  the forwarding path)

## Invariants

- The in-memory forwarder queue is never written to from within the forwarding code path
- Plugin `host.log()` calls never cause queue growth during forwarding
- This constraint applies regardless of the log level (even ERROR logs from a delivery
  failure are locally-only)

## Error Conditions

| Error | Condition | Behavior |
|-------|-----------|----------|
| Delivery failure (HTTP error, WASM trap, etc.) | Forwarder cannot deliver batch | Error logged to LOCAL sink only; batch retried per backoff schedule; queue not re-enqueued |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-20-001 | WASM plugin panics during `forward-batch()`; plugin `host.log()` emits error | Error written to stderr only; plugin trap is caught at wasmtime boundary; forwarder marks batch for retry |
| EC-20-002 | Built-in HTTP forwarder receives 503; forwarder logs retry WARN | WARN written to local sink only; entry not added to forwarder queue; retry scheduled via backoff |
| EC-20-003 | Two forwarders configured; one fails; its failure log is emitted | Failure log goes to local sink; the OTHER forwarder queue is also unaffected (no cross-forwarder contamination) |
| EC-20-004 | `host.log()` called 1000 times during a single `forward-batch()` invocation | All 1000 entries route to local sink; queue size unchanged; no memory accumulation from plugin verbosity |

## Canonical Test Vectors

| ID | Input | Expected Output | Notes |
|----|-------|----------------|-------|
| TV-20-001-recursive | WASM plugin configured as forwarder; plugin `host.log(error, "delivery failed")` inside `forward-batch()` | Log entry appears in stderr/file sink; forwarder in-memory queue size unchanged | AC-1 |
| TV-20-001-builtin | HTTP forwarder receives 503; internal WARN emitted by retry logic | WARN in stderr; queue size unchanged before and after | EC-20-002 |
| TV-20-001-cross | Two forwarders: A (WASM) fails and logs; B (splunk_hec) is healthy | A's failure log in stderr only; B's queue not modified; B continues forwarding normally | EC-20-003 |

## Verification Properties

| VP ID | Description | Verification Method |
|-------|-------------|---------------------|
| VP-TBD-20-001 | During any `forward-batch()` call, no write to any forwarder's in-memory queue occurs; queue size monotonically non-increasing during the call | Integration test (mock queue with write-count assertion) |

## Related BCs

- BC-2.20.002 — min_level filter (complementary: controls which entries enter the queue before forwarding begins)
- BC-2.20.003 — queue cap (governs overflow behavior; recursion prevention makes overflow less likely)
- BC-2.17.001 — Plugin Panic Isolation (WASM trap handling underpins EC-20-001)
- BC-2.17.002 — Plugin Sandbox (no direct syscalls; only host.log() path available)

## Architecture Anchors

- `specs/architecture/observability.md` §Forwarding Guarantees — "No recursive forwarding"
- `specs/architecture/observability.md` §Expandability via `.prx` Plugins — WIT `host.log()` interface

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
