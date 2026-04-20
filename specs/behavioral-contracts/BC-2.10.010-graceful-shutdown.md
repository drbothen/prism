---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "365fb25"
traces_to: ["CAP-034"]
extracted_from: ".factory/specs/prd.md"
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

## Description

On SIGTERM, SIGINT, or stdin pipe closure, Prism enters a 5-step shutdown sequence: stop accepting new requests, cancel in-flight tokio tasks with 5s grace, flush pending state writes (RocksDB WAL, dirty bits, audit buffer), close HTTP client connections, then flush tracing subscribers and exit 0. If graceful shutdown does not complete within 5 seconds, force-exit with code 1. RocksDB WAL ensures atomicity of in-flight writes at process exit. All in-flight audit entries are flushed before exit per DI-004/DI-026.

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

## Canonical Test Vectors

| Input | Expected Output | Category |
|-------|----------------|----------|
| SIGTERM with no in-flight operations | Clean exit code 0; all audit entries flushed | happy-path |
| SIGTERM during active sensor API query | Query cancelled within 5s grace; partial results discarded; exit 0 | edge-case |
| Graceful shutdown exceeds 5s | Force-exit code 1; warning logged to stderr | error |
| Double SIGINT | Second SIGINT triggers immediate force-exit | edge-case |

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vector tables.

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-039 | Audit forward watermark: monotonically non-decreasing per destination across ACK, failure, and restart sequences | kani |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-034 |
| L2 Invariants | DI-004, DI-026 |
| L2 Failure Modes | FM-011 |
| Priority | P0 |

## Changelog
| Version | Date | Burst | Author | Change |
|---------|------|-------|--------|--------|
| 1.0 | 2026-04-14 | cycle-1 | product-owner | Initial draft |
| 1.1 | 2026-04-20 | pre-build-sweep | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
