---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-10"
capability: "CAP-034"
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

# BC-2.10.010: Graceful Shutdown on SIGTERM/SIGINT

## Preconditions
- Prism is running and potentially has in-flight operations (sensor API queries, state writes)
- A shutdown signal is received: SIGTERM, SIGINT, or stdin pipe closure (client disconnect)

## Postconditions
- On signal receipt, Prism enters shutdown mode:
  1. Stop accepting new MCP requests
  2. Cancel in-flight tokio tasks (sensor API queries) with a 5-second grace period
  3. Flush pending state writes (flush RocksDB WAL, complete in-flight RocksDB writes, clear dirty bits for completed operations, flush audit buffer)
  4. Close HTTP client connections to sensor APIs
  5. Flush tracing/audit log subscribers
  6. Exit with code 0 (clean shutdown) or 1 (forced after timeout)
- If graceful shutdown does not complete within 5 seconds, force-exit with code 1
- No new sensor API requests are initiated after shutdown signal
- State integrity is maintained: RocksDB WAL ensures atomicity of in-flight writes

## Invariants
- DI-004: Audit completeness -- all in-flight audit entries are flushed before shutdown completes
- DI-026: Audit buffer durability -- audit buffer entries are flushed to RocksDB WAL before process exit

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Timeout | Graceful shutdown exceeds 5 seconds | Force-exit with code 1; log warning to stderr |
| I/O error | State flush fails during shutdown | Log error to stderr; exit with code 1; RocksDB WAL ensures recovery of committed writes on next startup |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| FM-011 | Client disconnects mid-query (stdin pipe broken) | Same shutdown sequence triggered by pipe closure detection |
| EC-10-018 | SIGTERM received while no operations are in-flight | Immediate clean exit (code 0) after flushing log subscribers |
| EC-10-019 | Double SIGINT (user presses Ctrl-C twice) | Second signal triggers immediate force-exit (standard Unix behavior) |
| EC-10-020 | State write in progress when shutdown begins | Write is allowed to complete (within 5s grace period); in-flight RocksDB write completes, ephemeral cursor state is discarded on shutdown |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-034 |
| L2 Invariants | DI-004, DI-026 |
| L2 Failure Modes | FM-011 |
| Priority | P0 |
