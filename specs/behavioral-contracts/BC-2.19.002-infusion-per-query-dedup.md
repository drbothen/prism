---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-16T12:00:00
phase: 2-patch
origin: greenfield
subsystem: "SS-19"
capability: "CAP-031"
lifecycle_status: active
introduced: cycle-1
---

# BC-2.19.002: Per-Query Dedup Cache — Unique Input Values Only, Not Per-Row

## Description

When an infusion UDF (e.g., `geoip_country(device_ip)`) is called during query execution,
Prism deduplicates enrichment calls at the unique-value level, not the per-row level.
If 10,000 events share 200 unique IP addresses, the infusion source is called exactly
200 times (not 10,000). The per-query dedup cache is a `HashMap` scoped to a single
query execution and is dropped when the query completes. This is INV-INFUSE-002.

## Preconditions

- A PrismQL query is executing that invokes an infusion UDF on a column
- Multiple rows in the query result set share the same input value (e.g., same IP address)
- The per-query dedup cache `HashMap<String, Option<serde_json::Value>>` is initialized
  at query start and keyed by `"{infusion_id}:{input_value}"`

## Postconditions

- For each unique input value encountered during query execution:
  - On FIRST occurrence: lookup in Tier 1 (dedup cache) → miss → lookup Tier 2 (LRU) →
    (miss) → lookup Tier 3 (RocksDB) → (miss) → call `InfusionSource::enrich_single(input)`
    → populate all tiers
  - On SUBSEQUENT occurrence (same query): dedup cache HIT → return cached value immediately;
    `InfusionSource::enrich_single` is NOT called again
- Total `InfusionSource::enrich_single` calls = number of unique input values, not total rows
- The per-query dedup cache is dropped (not persisted) when the query execution completes
- Tier 2 (in-memory LRU) and Tier 3 (RocksDB `infusion_cache` CF) are populated for
  values that reach the source

## Invariants

- INV-INFUSE-002: Per-query dedup operates on unique input values only — 10K events with 200 unique IPs = 200 source calls
- The per-query dedup cache MUST be allocated per-query and dropped after query completion
- The dedup cache MUST NOT be a shared in-memory structure — it is only safe within a
  single query execution context (DataFusion UDF execution is concurrent; per-query
  dedup must use appropriate synchronization or be Arrow-array-level)
- Cross-query sharing of dedup cache state is PROHIBITED (use Tier 2 LRU for cross-query caching)

## Error Cases

| Error | Condition | Behavior |
|-------|-----------|----------|
| — | `InfusionSource::enrich_single` returns `None` (no enrichment available) | `None` is cached in dedup map; UDF returns NULL to DataFusion; subsequent calls for same input return cached NULL |
| — | `InfusionSource::enrich_single` returns an error | Error logged; NULL returned to DataFusion; error NOT cached (next query will retry the lookup) |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-19-005 | 10K events, 200 unique IPs | 200 source calls exactly; verified by mocking `InfusionSource::enrich_single` call count |
| EC-19-006 | Single event (1 row, 1 unique input value) | 1 source call; dedup cache has 1 entry |
| EC-19-007 | 10K events, all unique IPs | 10K source calls; Tier 2 LRU eviction may occur if capacity exceeded |
| EC-19-008 | Same query executed twice in succession | First execution: N source calls; second execution: 0 source calls (all from Tier 2 LRU or Tier 3 RocksDB) |
| EC-19-009 | Concurrent queries with overlapping IP addresses | Each query has its own per-query dedup cache; both may call the source for the same IP (Tier 2 LRU prevents redundant source calls after first query populates it) |

## Related BCs

- BC-2.19.001 — Infusion Spec Loading (UDFs that this dedup governs)
- BC-2.19.005 — Infusion Three-Tier Cache (full cache stack: dedup → LRU → RocksDB → source)
- BC-2.11.005 — Ephemeral Materialization (the query execution context this dedup is scoped to)

## Architecture Anchors

- AD-020: Infusions — per-query dedup, three-tier caching
- `specs/architecture/infusions.md` — per-query dedup `HashMap`, Arrow array level dedup
- S-1.14 Task 6: `infusion/cache.rs` — Tier 1 per-query dedup

## Story Anchor

S-1.14 — prism-spec-engine: Infusion Spec Loading and UDF Registration (INV-INFUSE-002, AC-2)

## VP Anchors

Integration test: `tests/infusion_tests.rs` — "Verify per-query dedup: 3 rows with same IP → `InfusionSource::enrich_single` called once."

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-031 |
| Story Invariant | INV-INFUSE-002 |
| ADR | AD-020 |
| Story | S-1.14 |
| Priority | P0 |
