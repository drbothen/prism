---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "SS-12"
capability: "CAP-018"
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
input-hash: "c36ec87"
traces_to: ["CAP-018"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.12.005: Differential Result Computation — Hash Previous Results, Return Added/Removed

## Description

After each scheduled query execution, differential computation compares the current result set to the previous via SHA-256 fingerprinting. Records in the current set but not the previous are "added"; records in the previous but not the current are "removed" (if the schedule has `removed: true`). The first execution always emits everything as "added" (epoch 0 or snapshot mode). Schema changes between epochs fall back gracefully to snapshot mode with a warning. Empty diffs (no changes) still notify the detection engine for correlation window expiry. Every 100th execution performs a full row-by-row comparison to guard against hash drift.

## Preconditions
- A scheduled query execution has completed for a (schedule, client_id) pair
- The current result set is an array of OCSF-normalized records (Arrow RecordBatch)
- Previous results (if any) are available from the RocksDB `diff_results` domain, keyed by (schedule_name, client_id, epoch - 1)

## Postconditions
- If this is the first execution (epoch == 0) or `snapshot: true`: all current results are emitted as "added" with no "removed" entries
- If previous results exist and `snapshot: false`:
  - Each record is hashed (SHA-256 of all column values concatenated in schema order) to produce a fingerprint set
  - `added` = records in current fingerprint set but not in previous fingerprint set
  - `removed` = records in previous fingerprint set but not in current fingerprint set (only if `removed: true` on the schedule)
  - If both sets are identical (no added, no removed): a DiffResults with empty added and removed arrays is still emitted to the detection engine to allow correlation window expiry cleanup and sequence tracker maintenance. No MCP notification is sent (the agent does not need to know about unchanged data).
- The `DiffResults` structure contains: `added` (array of OCSF records), `removed` (array of OCSF records), `schedule_name`, `client_id`, `epoch`, `timestamp`, `query_execution_time_ms`
- Current result fingerprints are persisted to RocksDB as the new "previous" for the next epoch
- Differential results are persisted to the `diff_results` domain for retrieval via `get_diff_results` (BC-2.12.007)

## Invariants
- Record hashing is deterministic: the same record always produces the same fingerprint regardless of column iteration order (columns sorted by schema-defined order)
- SHA-256 collision probability is negligible (~2^-128). As a belt-and-suspenders measure, every 100th execution performs a full row-by-row comparison regardless of hash match, to detect any hash-based drift.
- No silent data loss: if differential computation fails (e.g., schema change between epochs), the full current result set is emitted as a snapshot with a warning annotation

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-DIFF-001` | No previous results for this schedule (first execution or history purged) | Emit all current results as "added"; this is the normal first-epoch path, not a true error |
| _Note_ | Previous result schema differs from current (schema evolution) | Normal restart condition: fallback to snapshot mode for this epoch; emit all current results as "added" with warning annotation; next epoch resumes differential. Not an error code — schema evolution is expected when queries or sensors change between epochs. |
| `E-STORE-003` | RocksDB read failure for previous fingerprints | Fallback to snapshot mode with warning; this is a storage-level error, not diff-specific |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-12-013 | Current results are empty but previous had records | All previous records emitted as "removed" (if `removed: true`) |
| EC-12-014 | Identical results across 10 consecutive epochs | No diff results emitted for any of those epochs; epoch counter still increments |
| EC-12-015 | Record contains `null` values in some columns | Null values are included in hash computation as a sentinel byte (0x00); two records differing only in null vs. non-null are distinct |

## Canonical Test Vectors

> See `.factory/specs/prd-supplements/test-vectors.md` for the canonical test vector tables.

| Input | Expected Output | Category |
|-------|----------------|----------|
| First execution (epoch 0) with 5 records | All 5 emitted as "added"; no "removed" | happy-path |
| Epoch N: 3 records; Epoch N+1: 2 original + 1 new | `added: [new_record]`, `removed: [dropped_record]` | happy-path |
| Epoch N and N+1 identical results | Empty diff; detection engine still notified | edge-case |
| Schema change between epochs | Fallback to snapshot mode; all current records as "added" with warning | edge-case |

## Verification Properties

| VP ID | Property | Proof Method |
|-------|----------|-------------|
| VP-019 | Diff computation: deterministic | proptest |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-018 |
| L2 Invariants | DI-008, DI-023 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial contract |
