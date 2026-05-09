---
document_type: behavioral-contract
level: L3
version: "1.3"
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
input-hash: "335606b"
traces_to: ["CAP-035"]
extracted_from: ".factory/specs/architecture/observability.md"
---

# BC-2.20.003: Log Forwarder Queue Cap — Drop-Oldest on Overflow with Metric Emission

## Description

Each external log forwarder maintains a per-destination in-memory batch queue bounded
at `10 × batch_size` entries (default `batch_size = 100` → cap of 1,000 entries per
destination). This is a v1 best-effort delivery model: when the queue exceeds its cap,
the oldest entries are dropped to make room for new entries. A WARN-level log is emitted
to the LOCAL sink on every drop event. This prevents unbounded memory growth when a
destination is unreachable for an extended period while Prism continues operating.

## Preconditions

- An external log destination is configured with `batch_size = N` (default 100)
- The destination is temporarily unreachable or slow, causing the queue to grow
- New log entries continue to be generated at a rate that would exceed the queue cap

## Postconditions

- Each destination's in-memory queue is bounded at `10 × batch_size` entries at all times
- When a new entry would cause the queue to exceed the cap:
  - The OLDEST entry (front of queue) is removed (dropped-oldest policy)
  - The new entry is appended at the tail
  - A WARN entry is emitted to the LOCAL stderr/file sink: `"[log-forwarder/{name}] queue full (cap={cap}); dropping oldest entry — best-effort delivery"`
  - No error is returned to the caller; log emission continues normally
- Queue drops are reflected in per-destination drop metrics accessible via `watchdog_status`
- The queue drop WARN itself is NOT fed back into the forwarder queue (recursive prevention per BC-2.20.001)
- Delivery is best-effort: v1 does NOT provide a persistent/durable queue for diagnostic logs

## Invariants

- Queue size never exceeds `10 × batch_size` entries per destination
- Drop-oldest is the ONLY overflow strategy; no blocking, no back-pressure to callers
- Diagnostic log forwarding is independent of audit log durability (CAP-025): audit logs
  have RocksDB-backed at-least-once delivery; diagnostic forwarder does not

## Error Conditions

| Error | Condition | Behavior |
|-------|-----------|----------|
| Queue overflow | New entry would exceed `10 × batch_size` | Oldest entry dropped; WARN to local sink; new entry enqueued; no caller error |
| Destination unreachable for extended period | Queue fills repeatedly | Entries drop continuously; drop count accumulates in metrics; WARN emitted per drop |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-20-010 | Queue at exactly cap (`1000`); new entry arrives | Entry at position 0 dropped; new entry appended at position 1000; WARN emitted |
| EC-20-011 | Queue at cap; destination comes back online; flush occurs | Flush delivers entries 1..N currently in queue (not the dropped ones); after flush queue empties; new entries enqueue normally |
| EC-20-012 | `batch_size = 50` configured | Cap = 500 entries for that destination |
| EC-20-013 | Two destinations both reach cap simultaneously | Each destination independently drops oldest entries; no cross-contamination |
| EC-20-014 | Queue empty; destination unreachable | No drops yet; entries accumulate up to cap before drops begin |
| EC-20-015 | Queue drops 10,000 entries during a 1-hour outage | Each drop emits WARN to local sink; drop_count metric = 10,000; no memory growth beyond cap |

## Canonical Test Vectors

| ID | Input | Expected Output | Notes |
|----|-------|----------------|-------|
| TV-20-003-cap | Destination offline; 1001 entries emitted with `batch_size = 100` (cap = 1000) | Queue size = 1000; drop_count = 1; 1 WARN in stderr | EC-20-010 |
| TV-20-003-recovery | Queue at cap (1000 entries); destination comes back; batch flush | Entries delivered; queue drains; no new drops during flush | EC-20-011 |
| TV-20-003-custom | `batch_size = 50`; 501 entries emitted while offline | Queue size = 500; drop_count = 1; WARN emitted | EC-20-012 |
| TV-20-003-multi | Two destinations both at cap; 2 new entries | Each destination drops 1 oldest; each emits 1 WARN | EC-20-013 |

## Verification Properties

| VP ID | Description | Verification Method |
|-------|-------------|---------------------|
| VP-062 | Under any sequence of enqueue operations, queue.len() never exceeds `10 × batch_size`; every overflow enqueue triggers exactly one drop of the oldest entry (drop_count +1); pure BoundedQueue function | proptest (sequence of enqueue calls across varied batch_size values; WARN emission to local sink verified by integration test TV-20-003-cap) |

## Related BCs

- BC-2.20.001 — Recursive Prevention (WARN from overflow goes to local sink only)
- BC-2.20.002 — Min-Level Filter (filter applied before enqueue; reduces overflow pressure)
- BC-2.20.005 — Destination Isolation (cap enforced independently per destination)

## Architecture Anchors

- `specs/architecture/observability.md` §Forwarding Guarantees — "Per-forwarder in-memory batch queue is capped at 10 × batch_size (default 1,000 entries)"
- `specs/architecture/observability.md` §Forwarding Guarantees — "Best-effort delivery"

## Story Anchor

S-5.09 — prism-mcp: External Log Forwarding Subsystem

## VP Anchors

VP-062 — proptest: queue.len() bounded at 10 × batch_size; drop_count +1 per overflow enqueue (pure BoundedQueue); WARN emission verified by integration test TV-20-003-cap

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-035 (Diagnostic Log Forwarding) |
| ADR | observability.md §Forwarding Guarantees |
| Story | S-5.09 |
| Priority | P0 |
| L2 Invariants | DI-026 (Audit Buffer Durability) is the explicit contrast: audit logs MUST persist (RocksDB-backed, at-least-once) while the diagnostic forwarder queue MAY drop (best-effort, in-memory only). DI-018 (Cache Bounds) provides the closest structural analogy — both enforce a per-destination memory cap with eviction when the bound is exceeded. No DI directly covers the diagnostic forwarder bounded queue; the BC-level postconditions serve as the authoritative spec. |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-81 | 2026-04-21 | architect | F81-009: Resolved VP-TBD-20-003 → VP-062 (proptest, P1); updated VP table and VP Anchors. WARN emission remains integration-test only per effectful-shell pattern. |
| 1.2 | pass-81-remediation | 2026-04-21 | product-owner | F81-008: Added L2 Invariants row to Traceability (DI-026 as contrast, DI-018 as structural analogy; no direct DI). |
| 1.1 | pass-80-follow-on | 2026-04-21 | product-owner | Re-anchored CAP-025 → CAP-035 (business-analyst created CAP-035 post-hoc per pass-80 F80-002 follow-on); removed Capability Anchor Note; added capabilities.md to inputs |
| 1.0 | pass-80-remediation | 2026-04-21 | product-owner | Initial contract — F80-002 gap closure |
