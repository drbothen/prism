---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "SS-15"
capability: "CAP-025"
lifecycle_status: active
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "47125c0"
traces_to:
  - "CAP-025"
extracted_from: ".factory/specs/prd.md"
---

# BC-2.15.004: Audit Buffer Overflow — Purge Oldest Entries When Exceeding 100K, Log Warning

## Description

When the audit buffer grows beyond its configured maximum (default 100K entries),
the buffer forwarder background task purges the oldest entries down to 90K to maintain
headroom. Purged entries are permanently lost — they were not successfully forwarded
to any sink — but the purge event itself is written as a special audit entry, creating
a meta-audit trail of data loss events. Newer entries are preserved because recent audit
data has higher operational value than aged un-forwarded entries.

The threshold is configurable to accommodate high-velocity environments, with a minimum
floor of 1,000 entries to prevent aggressive accidental configuration.

## Preconditions
- The RocksDB `audit_buffer` column family contains entries that have not yet been forwarded
- The entry count approaches or exceeds the configured maximum (default: 100,000 entries)

## Postconditions
- The buffer forwarder background task monitors buffer size during each forward cycle
- When the buffer exceeds 100K entries:
  1. A warning is emitted to stderr: `"Audit buffer overflow: {count} entries exceed 100K limit. Purging oldest {purge_count} entries."`
  2. The oldest entries (by timestamp key) are purged until the buffer is at 90K entries (10% headroom)
  3. A special audit entry is written recording the purge event: `{ event: "audit_buffer_purge", purged_count: N, oldest_purged_timestamp: T, newest_purged_timestamp: T }`
- The purge operation uses `remove_range` (BC-2.15.002) for efficient batch deletion
- Purged entries are lost -- they were not successfully forwarded to any sink
- The overflow threshold is configurable via `audit.buffer_max` in TOML (default 100,000; minimum 1,000)

## Invariants
- Buffer size is bounded: unbounded growth cannot exhaust disk space
- Purge preserves the newest entries (most recent audit data is higher value)
- The purge event itself is audit-logged (meta-audit)

## Error Conditions
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-AUDIT-004` | Purge operation fails (RocksDB error) | Critical warning; buffer continues growing; next purge cycle retries |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-15-014 | Buffer reaches 100K with all entries from the last 5 minutes (extremely high tool invocation rate) | Purge proceeds; warning indicates rapid accumulation suggesting Vector endpoint is down |
| EC-15-015 | `audit.buffer_max` set to 1000 (minimum) | Aggressive purging under normal load; warning recommends increasing the limit |
| EC-15-016 | Buffer at exactly 100K entries | Purge triggered on the next insert that would exceed the limit |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for full canonical vectors.

| Scenario | Input | Expected Output |
|----------|-------|-----------------|
| Happy path — overflow trigger | buffer at 100,001 entries | Purge oldest to 90K; warning emitted; purge-event audit entry written |
| Minimum buffer_max | `audit.buffer_max: 1000`, 1001 entries | Purge to 900; warning with recommendation to increase limit |
| Purge fails | RocksDB error during remove_range | `E-AUDIT-004`; buffer continues growing; retry next cycle |
| Purge event audited | overflow purge runs | Special purge audit entry present in buffer after purge |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| VP-056 | Audit buffer overflow purge: oldest entries deleted, newest preserved — for any audit buffer of N entries (N > threshold), `compute_purge_targets()` returns exactly the oldest `(N - floor(threshold * 0.9))` entries by timestamp key; the newest `floor(threshold * 0.9)` entries are never in the purge target set; a purge-event record is always included in the output. Method: Proptest. Priority: P1. |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-025 |
| L2 Invariants | DI-004 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-74-fix | 2026-04-20 | product-owner | Resolved (placeholder) row in ## Verification Properties per pass-74 VP-TBD decision matrix extension. |
| 1.2 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; renamed Error Cases → Error Conditions; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial draft |
