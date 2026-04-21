---
document_type: behavioral-contract
level: L3
version: "1.2"
status: draft
producer: product-owner
timestamp: 2026-04-16T14:00:00
phase: 2-patch
origin: greenfield
subsystem: "SS-05"
capability: "CAP-007"
lifecycle_status: active
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "365fb25"
traces_to: ["CAP-007"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.05.011: Audit Forwarding — At-Least-Once Delivery to External Destinations (VP-039 monotonic watermark)

## Description

Audit entries stored in the `audit_buffer` RocksDB column family are forwarded to
external destinations (Vector, syslog, or webhook) as configured in `[audit.forward]`.
Delivery is at-least-once: every entry is guaranteed to reach every enabled destination
at least once. A per-destination watermark stored in RocksDB advances ONLY after the
destination ACKs delivery. Failed deliveries use exponential backoff (2 seconds base,
60 seconds maximum). Entries remain in `audit_buffer` until all destinations have ACKed.

If the `audit_buffer` CF reaches the 100K entry cap (BC-2.15.004), FIFO eviction
occurs as a safety last resort. This is the ONLY condition under which audit entries
may be lost without delivery. The eviction emits a CRITICAL log entry and is never
silent. This forwarding subsystem is entirely separate from diagnostic log forwarding
(observability.md §"External Log Forwarding") — diagnostic log forwarding is best-effort
and has no at-least-once guarantee.

## Preconditions

- `[audit.forward]` is configured with at least one destination (type, endpoint)
- The `audit_buffer` RocksDB column family is initialized (BC-2.15.001)
- At least one audit entry exists in `audit_buffer` beyond the current per-destination watermark
- The forwarding task is running as a background tokio task started at server startup

## Postconditions

### Successful forwarding

- Each audit entry at watermark position W is fetched from `audit_buffer` and transmitted
  to the configured destination
- On destination ACK:
  - The per-destination watermark in RocksDB advances from W to W+1
  - The entry remains in `audit_buffer` (it is not deleted until all destinations have
    advanced their watermark past it; cleanup is a separate GC pass)
- Entries are forwarded in insertion order (FIFO by audit entry ID)
- Forwarding is asynchronous relative to query execution — it does not block MCP tool
  responses

### Forwarding failure with backoff

- On transient failure (network error, HTTP 5xx, syslog connection refused):
  - The per-destination watermark does NOT advance
  - The next retry occurs after `min(base × 2^attempt, max)` seconds:
    attempt 0 = 2s, 1 = 4s, 2 = 8s, 3 = 16s, 4 = 32s, 5 = 60s (cap)
  - Retry state is held in memory (not separately persisted); on restart, forwarding
    resumes from the last RocksDB watermark position (at-least-once preserved)
- On permanent failure (HTTP 4xx, DNS resolution permanently fails):
  - A WARN-level log is emitted: `"Audit forward permanent failure for destination '{name}': {reason}"`
  - The watermark does NOT advance; the entry will NOT be retried for this session unless
    the destination config is corrected and `reload_config` is called
  - Error code `E-AUDIT-005` is emitted

### Buffer cap / FIFO eviction (safety last resort)

- If `audit_buffer` reaches 100K entries (BC-2.15.004) AND unACKed entries for the
  oldest destination prevent GC:
  - FIFO eviction proceeds (oldest entries deleted from `audit_buffer`)
  - A CRITICAL log entry is emitted: `"Audit buffer capacity exceeded: evicting N entries. At-least-once guarantee LOST for evicted entries. Check audit forward destination health."`
  - The watermark for the lagging destination is advanced past the evicted entries
    (effectively skipping them) to prevent the buffer from filling permanently

## Invariants

- **INV-AUDIT-FWD-001 (Monotonic Watermark):** The per-destination forward watermark in
  RocksDB is monotonically non-decreasing. It NEVER decrements and NEVER skips an entry
  that has not been ACKed (except FIFO eviction under buffer-full condition, which is
  the only documented exception). This property is proposed for formal verification
  (VP-039 — see VP Anchors)
- **INV-AUDIT-FWD-002 (Restart Durability):** On server restart, the forwarding task
  reads the per-destination watermark from RocksDB and resumes from that position.
  Entries already ACKed are not retransmitted. Entries not yet ACKed (between watermark
  and current `audit_buffer` tail) are retransmitted — at-least-once delivery is
  preserved across restarts
- **INV-AUDIT-FWD-003 (Separation from Diagnostic Forwarding):** Audit forwarding
  (`[audit.forward]`) and diagnostic log forwarding (`[[server.log_forward]]`) share no
  code paths, queues, or error handling. A failure in diagnostic log forwarding MUST NOT
  affect audit forwarding, and vice versa
- **INV-AUDIT-FWD-004 (No Silent Loss):** The ONLY condition under which an audit entry
  may be lost without delivery is the documented FIFO eviction path. Every other loss
  condition (network failure, config error, destination outage) preserves the entry in
  `audit_buffer` and retries

## Error Cases

| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-AUDIT-002` | Destination temporarily unreachable (network error, HTTP 5xx) | Exponential backoff retry; watermark does not advance; entry remains in buffer |
| `E-AUDIT-005` | Destination permanently unreachable (HTTP 4xx, invalid config) | WARN log; watermark does not advance; no retry for this session; entry preserved in buffer until config fix + reload |
| `E-AUDIT-006` | FIFO eviction triggered (buffer at 100K cap, lagging destination) | CRITICAL log; evicted entries' watermarks advanced; at-least-once guarantee lost only for evicted entries; all subsequent entries continue to be forwarded normally |
| `E-STORE-002` | RocksDB unavailable for watermark write | Log ERROR; watermark write retried on next forwarding cycle; entry may be re-delivered on next attempt (at-least-once preserved at cost of duplication) |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-05-020 | Two destinations configured; destination A is healthy, destination B is down | Destination A's watermark advances normally; destination B's watermark stalls; entries accumulate for destination B; buffer fills if outage is prolonged |
| EC-05-021 | Server restarts mid-forward (process killed after destination ACK but before watermark write) | On restart, the watermark reflects the last durably written position; the ACKed entry may be re-forwarded (at-least-once: harmless duplicate) |
| EC-05-022 | Server restarts mid-forward (process killed after watermark write but before next entry fetch) | On restart, watermark is consistent; next entry after watermark is forwarded first; no loss |
| EC-05-023 | `audit_buffer` has 50K entries, destination is down for 2 hours | Entries accumulate; backoff caps at 60s retry; no eviction until 100K; WARN logs emitted per-retry-attempt when buffer crosses 90K threshold |
| EC-05-024 | `reload_config` is called with updated `[audit.forward]` endpoint | New endpoint takes effect for subsequent forwarding cycles; existing watermark is retained; no entries are re-forwarded that already advanced the watermark |
| EC-05-025 | `[audit.forward]` is removed from config on reload | All forwarding stops; existing watermarks are preserved in RocksDB; if `[audit.forward]` is re-added, forwarding resumes from watermarks (no gap) |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vectors for BC-2.05.011.

| Scenario | Setup | Expected Behavior |
|----------|-------|-------------------|
| Normal forward with ACK | 1 destination, 10 entries in buffer | All 10 entries forwarded; watermark advances to 10; entries retained in buffer until GC |
| Transient failure, retry | Destination returns HTTP 503 on attempt 0 | Watermark does not advance; retry after 2s; subsequent success advances watermark |
| Backoff sequence | Destination down for 6 consecutive attempts | Retry delays: 2s, 4s, 8s, 16s, 32s, 60s (capped); no further increase |
| Restart mid-forward | Process killed after ACK but before watermark write | On restart, entry re-forwarded (harmless duplicate; at-least-once preserved) |
| FIFO eviction | Buffer at 100K entries, destination down | CRITICAL log emitted; evicted entries' watermarks advanced; subsequent entries forwarded normally |

## Related BCs

- BC-2.15.003 — Buffered Audit Log Persistence — RocksDB + Exponential Backoff
  (covers the `audit_buffer` CF that this BC drains)
- BC-2.15.004 — Audit Buffer Overflow — Purge Oldest at 100K Entries (the FIFO
  eviction that is the safety last resort for this BC)
- BC-2.05.007 — Audit Entries Are Compatible with the Vector Pipeline (format compatibility
  with the Vector destination type)
- BC-2.05.006 — Audit Entries Are Append-Only and Immutable (the entries this BC forwards)
- BC-2.18.001 — Alert Action At-Least-Once Delivery (analogous at-least-once pattern
  for the action delivery subsystem; different queue and different watermark)

## Architecture Anchors

- `specs/architecture/config-schema.md` §`[audit.forward]` — type, endpoint, format,
  retry_base_seconds, retry_max_seconds configuration fields
- `specs/architecture/actions.md` §"Forwarding Guarantees" — at-least-once vs
  best-effort comparison between audit and diagnostic forwarding
- `specs/architecture/observability.md` §"External Log Forwarding" — explicitly separate
  from audit forwarding (best-effort only)
- S-5.10 Task: `audit/forward.rs` — per-destination watermark tracking, exponential
  backoff loop, RocksDB watermark reads/writes

## Story Anchor

S-5.10 — Audit Forwarding

## VP Anchors

**VP-039 (proposed — Kani formal verification):** Watermark monotonicity for audit
forwarding. The Kani harness encodes the per-destination forwarding state machine and
proves: (1) the watermark position for a given destination is a monotonically
non-decreasing function over all possible event sequences (ACK, network failure, restart);
(2) the watermark never decrements; (3) the watermark never skips an unACKed entry
except through the documented FIFO eviction path. This VP is proposed here and should
be added to the VP catalog (VP-INDEX.md) under Story anchor S-5.10.

Story anchor for VP-039: S-5.10

## Verification Properties

- **VP-039** (Audit forward watermark: monotonically non-decreasing per destination across ACK, failure, and restart sequences) — Kani proof of INV-AUDIT-FWD-001. Story anchor: S-5.10.

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-007 |
| L2 Invariants | DI-026 |
| Story | S-5.10 |
| Priority | P0 |
| VP Proposal | VP-039 (Kani monotonic watermark) |
| Interface | config-schema.md §audit.forward |

## Changelog
| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.2 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added missing frontmatter fields (deprecated, deprecated_by, modified, removal_reason, removed, replacement, retired, inputs, input-hash, traces_to, extracted_from); added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref (VP-039); added ## Changelog. |
| 1.0 | 2-patch | 2026-04-16 | product-owner | Initial contract (phase 2-patch) |
