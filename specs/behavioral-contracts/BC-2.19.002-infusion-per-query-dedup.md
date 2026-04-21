---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-16T12:00:00
phase: 2-patch
origin: greenfield
subsystem: "SS-19"
capability: "CAP-031"
lifecycle_status: active
introduced: cycle-1
modified: 2026-04-20
deprecated: ~
deprecated_by: ~
replacement: ~
retired: ~
removed: ~
removal_reason: ~
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "3eb97f3"
traces_to: ["CAP-031"]
extracted_from: ".factory/specs/prd.md"
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

## Error Conditions

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

## Canonical Test Vectors

| ID | Input | Expected Output | Notes |
|----|-------|----------------|-------|
| TV-19-002-happy | 3 rows with same IP; mock `enrich_single` | `enrich_single` called exactly once | AC-2 |
| TV-19-002-10k | 10K events; 200 unique IPs | Exactly 200 `enrich_single` calls | EC-19-005 |
| TV-19-002-allunique | 10K events; all unique IPs | 10K calls; LRU eviction may occur | EC-19-007 |
| TV-19-002-repeat | Same query twice; Tier 2 LRU populated | Second execution: 0 `enrich_single` calls | EC-19-008 |

## Verification Properties

| VP ID | Description | Verification Method |
|-------|-------------|---------------------|
| VP-049 | For any input sequence of N values containing K distinct values (1 ≤ K ≤ N ≤ 10000), the per-query dedup cache results in exactly K calls to `InfusionSource::enrich_single`; the dedup cache contains exactly K entries after processing | Proptest |
| (none) | Per-query cache dropped after query completion — integration behavior verified by unit test with memory tracking in tests/infusion_tests.rs | — |

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

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Resolved VP-TBD placeholder per decision matrix (ADD-VP-049); normalized changelog schema to canonical 5-col form. |
| 1.1 | Wave-6-pre-build-sweep | 2026-04-20 | product-owner | Added frontmatter (inputs, input-hash, traces_to, extracted_from, lifecycle fields); added Error Conditions (from inline entries), Canonical Test Vectors, Verification Properties, Changelog |
| 1.0 | Phase-2 | 2026-04-16 | product-owner | Initial contract |
