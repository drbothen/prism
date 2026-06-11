---
document_type: behavioral-contract
level: L3
version: "1.4"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "SS-15"
capability: "CAP-024"
lifecycle_status: active
introduced: cycle-1
modified: "2026-06-10"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "2580252"
traces_to:
  - "CAP-024"
extracted_from: ".factory/specs/prd.md"
---

# BC-2.15.005: Crash Recovery Dirty Bits — Set Before Operation, Clear After, Detect on Restart

## Description

Before executing any query, Prism sets a dirty bit in RocksDB with sync semantics
that records the query hash, source (ad-hoc or scheduled), and start time. The dirty
bit is cleared on every orderly exit — successful completion OR structured-error
termination — so only genuine process death leaves a surviving marker. On startup,
surviving dirty bits indicate queries that were in-flight when the process crashed.

If a query hash has crashed three or more consecutive times (consecutive_crashes >=3),
it is added to the watchdog denylist for 24 hours to prevent crash loops. Dirty bits
cover query execution only; detection state and case updates rely on RocksDB WriteBatch
atomicity instead, making dirty bits redundant for those domains.

## Preconditions
- The RocksDB `dirty_bits` column family is initialized (BC-2.15.001)
- A query (ad-hoc or scheduled) is about to execute

## Postconditions
- **Before query execution:** a dirty bit is set in RocksDB with `sync: true`: key = `{query_hash}`, value = serialized `DirtyBitEntry { query_hash, query_source: AdHoc | Scheduled { schedule_name, client_id }, started_at, consecutive_crashes }` (matching data-layer.md DirtyBitEntry struct)
- **On every ORDERLY query exit** — successful completion OR structured-error termination (any error returned through the normal error path, including watchdog terminations: query timeout `E-QUERY-004`, process-RSS kill `E-WATCHDOG-002`, pool trip `E-WATCHDOG-001`, record cap `E-QUERY-005`) — the dirty bit is cleared (key deleted). Only genuine process death leaves a surviving marker. **Rationale (double-count hazard):** if markers persisted on structured-error exits, the startup recovery scan would re-count those terminations as crashes — a single watchdog termination would feed the shared consecutive-failure counter twice (once as `timeout`/`rss_kill` at termination time per BC-2.15.007, again as `crash` on the next restart), breaking BC-2.15.008 v1.4 postcondition 2's single-streak semantics (watchdog terminations and crashes share ONE consecutive counter)
- **On startup:** all remaining dirty bits are scanned from the `dirty_bits` column family (after schema version check per data-layer.md startup protocol)
- For each dirty bit found on startup:
  - `consecutive_crashes` is incremented
  - If `consecutive_crashes >= 3`: query_hash added to watchdog denylist (86400s expiry)
  - If source is `Scheduled`: log WARN ("schedule interrupted, will retry on next tick")
  - If source is `AdHoc`: log WARN only (ad-hoc queries are not retried)
  - After processing, dirty bit is cleared
- **Scope:** Dirty bits cover query execution only (ad-hoc + scheduled). Detection evaluation and case updates do not use dirty bits — detection state (correlation windows, sequence trackers) IS persisted to RocksDB, but the acceptable failure mode on crash is window reset (E-DETECT-004/006: "correlation windows reset to empty; warning logged; detection resumes from clean state"), making dirty bits redundant for detection. Case updates use RocksDB WriteBatch atomicity (no dirty bit needed).

## Invariants
- Dirty bit is always set before the operation begins and cleared on every orderly exit (successful completion or structured-error termination); only process death leaves a marker
- Dirty bit write is a single RocksDB `put` (not part of the operation's WriteBatch) to ensure it's visible even if the operation's batch fails
- Startup recovery is idempotent: running recovery twice produces the same result

## Error Conditions
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-STORE-009` | Dirty bit write fails | Query is aborted (fail-closed). Without a dirty bit, a crashing query cannot be denylisted on restart — the entire crash recovery safety mechanism is bypassed. The query is rejected with E-STORE-009 and the analyst is advised to investigate storage health. |
| `E-STORE-010` | Recovery action fails on startup | Warning logged; dirty bit is NOT cleared; recovery retried on next startup |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-15-017 | Clean shutdown (SIGTERM) | All in-flight operations complete; all dirty bits are cleared; startup finds zero dirty bits |
| EC-15-018 | Crash during dirty bit write itself | Dirty bit may or may not be set (RocksDB atomic write); at worst, one operation goes undetected |
| EC-15-019 | 100 dirty bits on startup (many concurrent operations crashed) | All processed sequentially; recovery may take several seconds |
| EC-15-020 | Dirty bit from a previous Prism version with unknown operation type | Warning logged; dirty bit cleared; no recovery attempted |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for full canonical vectors.

| Scenario | Input | Expected Output |
|----------|-------|-----------------|
| Happy path — successful query | query executes and completes | Dirty bit set before; cleared after; no bits on next startup |
| Orderly error exit — watchdog termination | query terminated with `E-QUERY-004` (or any structured error) | Dirty bit cleared on the orderly error exit; no marker survives; next startup does NOT increment consecutive_crashes (no double-count vs BC-2.15.008 streak) |
| Crash detection — single crash | 1 dirty bit on startup | consecutive_crashes=1; warn logged; bit cleared; no denylisting |
| Crash detection — 3rd crash | dirty bit with consecutive_crashes=2 on startup | consecutive_crashes=3; query hash denylisted 86400s |
| Dirty bit write failure | RocksDB unavailable before query | Query aborted with E-STORE-009 |
| Clean shutdown | SIGTERM received | All bits cleared; clean startup |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| VP-057 | Crash recovery: denylist triggered at consecutive_crashes >= 3 — `advance_crash_counter(entry)` returns `RecoveryAction::Denylist { expiry_seconds: 86400 }` if and only if `entry.consecutive_crashes + 1 >= 3`; for all other values it returns `RecoveryAction::Warn`; the threshold is exactly 3 (not 2 or 4). Method: Kani. Priority: P0. |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-024 |
| L2 Invariants | -- |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.4 | S-WATCHDOG-WIRING-001 v1.1 AC-014 wording-sharpen flag (QRY cascade P3-01 realignment, story-writer fix-burst 2026-06-10 — review-2026-06-10 PO micro-burst) | 2026-06-10 | product-owner | Postcondition 2 sharpened from "After successful query execution: the dirty bit is cleared" to the ratified orderly-completion semantics (per the story's AC-014 reading, flagged to product-owner in story v1.1): the dirty bit is cleared on EVERY orderly exit — successful completion OR structured-error termination (including watchdog terminations `E-QUERY-004`/`E-WATCHDOG-002`, pool trip `E-WATCHDOG-001`, record cap `E-QUERY-005`) — so only genuine process death leaves a surviving marker. Double-count rationale recorded inline: markers persisting on structured-error exits would make the startup recovery scan re-count watchdog terminations as crashes, feeding the shared consecutive-failure counter twice (once as `timeout`/`rss_kill` at termination time per BC-2.15.007, again as `crash` on restart) — breaking BC-2.15.008 v1.4 postcondition 2's single-streak semantics. Wording sharpen, NOT a semantic change: "successful" never licensed leaving markers on structured errors; the BC now states it explicitly. Sibling sweep within this BC: Description ¶1 + Invariant 1 aligned to the same orderly-exit wording; 1 canonical vector added (orderly error exit clears the bit, no restart increment; append-only). No H1 retitle; no edge-case-ID, error-condition, VP, or lifecycle/status changes. |
| 1.3 | pass-74-fix | 2026-04-20 | product-owner | Resolved (placeholder) row in ## Verification Properties per pass-74 VP-TBD decision matrix extension. |
| 1.2 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; renamed Error Cases → Error Conditions; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial draft |
