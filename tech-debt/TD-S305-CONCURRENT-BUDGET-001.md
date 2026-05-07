# TD-S305-CONCURRENT-BUDGET-001: Concurrent byte-budget bypass test (SEC-NEW-002)

**Filed:** 2026-05-07
**Story:** S-3.05
**Adversary pass:** LOCAL pass-1 observation O-4
**Severity:** tech_debt
**Status:** open / deferred

## Finding

The byte-budget check in `QueryCache::put_with_ttl` uses a Relaxed load + separate
`fetch_add` outside the partition lock. Under concurrent insert load, N threads can
pass the budget check window before any increments are committed, causing transient
soft over-commitment by up to N × MAX_ENTRY_BYTES (tracked as SEC-NEW-002).

No regression test exercises this concurrent race.

## Why Deferred

Writing a deterministic concurrent race test requires either:
1. Moving the `fetch_add` inside the partition lock (requires refactor with performance
   tradeoffs), OR
2. Using a synchronization primitive (e.g., `Barrier`) to align N threads in the
   check-then-act window (non-trivial in unit test context).

The current design documents the trade-off: the budget is a resource-management heuristic,
not a hard security boundary. Transient over-commitment is bounded and recovers on
subsequent inserts.

## Resolution Path

If stricter budget enforcement is required (e.g., OOM risk analysis shows the window
matters), move the byte `fetch_add` inside `lock_partition_counts` to make the check
atomic with the reservation. Add a concurrent stress test using `std::sync::Barrier`.

## Related

- SEC-NEW-002 comment in `cache.rs` `put_with_ttl`
- MAX_ENTRY_BYTES = 5 MB per entry; worst-case over-commit = N × 5 MB
