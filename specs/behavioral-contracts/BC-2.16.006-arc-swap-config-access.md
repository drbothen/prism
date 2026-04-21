---
document_type: behavioral-contract
level: L3
version: "1.2"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "SS-16"
capability: "CAP-030"
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
input-hash: "365fb25"
traces_to:
  - "CAP-030"
extracted_from: ".factory/specs/prd.md"
---

# BC-2.16.006: Arc-Swap Config Access on Hot Path — Lock-Free Reads for Query-Time Config Access

## Description

All query-time config reads use `arc_swap::ArcSwap::load()` for a lock-free atomic
read that returns a `Guard<Arc<ConfigSnapshot>>`. The guard holds a snapshot reference
for the entire query lifecycle; a config reload mid-query does not affect the in-flight
query — it continues using the snapshot captured at query start (DEC-037).

The swap operation (`store()`) is O(1) and is called only by `reload_config`. At most
two `ConfigSnapshot` instances exist simultaneously: the current one and the one being
replaced. `ConfigSnapshot` is immutable after construction — no interior mutability.

## Preconditions
- Prism has loaded a `ConfigSnapshot` at startup
- The `ConfigSnapshot` is stored in an `arc_swap::ArcSwap<ConfigSnapshot>` shared across all async tasks

## Postconditions
- All query-time config reads use `ArcSwap::load()` which returns an `arc_swap::Guard<Arc<ConfigSnapshot>>` — a lock-free atomic read
- No mutex, RwLock, or other blocking synchronization is on the query hot path for config access
- The `Guard` holds a reference to the `Arc<ConfigSnapshot>` that was current at the time of `load()` — subsequent swaps do not affect the guard's reference
- A query that begins execution with ConfigSnapshot v1 continues using v1 for its entire lifecycle, even if a reload swaps in v2 mid-query (DEC-037)
- The `arc_swap::ArcSwap::store()` method is used by `reload_config` (BC-2.16.005) to atomically replace the current snapshot — this is the only write path
- `store()` is called from the `reload_config` tool handler, which runs on the Tokio runtime — no blocking I/O occurs during the swap itself

## Performance Characteristics
- `ArcSwap::load()` is wait-free on x86_64 (single atomic load + reference count increment)
- No contention between concurrent query executions reading config
- The swap operation (`store()`) is O(1) and does not block readers
- Old `ConfigSnapshot` values are freed when the last `Guard` referencing them is dropped (automatic via `Arc` reference counting)

## Memory Management
- At most 2 `ConfigSnapshot` instances exist simultaneously: the current one and the one being replaced (while old guards are still held by in-flight queries)
- `ConfigSnapshot` is immutable after construction — no interior mutability
- Each `ConfigSnapshot` contains cloned data (not references to parsed TOML nodes) so the original file contents can be freed after parsing

## Invariants
- No blocking synchronization on the query hot path for config reads
- `ConfigSnapshot` is immutable after construction
- In-flight query snapshot is stable for the query's full lifecycle (DEC-037)
- Only `reload_config` calls `store()` — it is the sole write path

## Error Conditions
| Error | Condition | Behavior |
|-------|-----------|----------|
| (none) | `ArcSwap` operations are infallible | `load()` and `store()` have no error conditions |
| (panic) | `ArcSwap` is uninitialized (programming error) | Accessing an uninitialized ArcSwap panics — caught by tests; should never occur post-startup |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-037 | Reload mid-query | In-flight query continues with snapshot v1; next query uses v2 |
| DEC-039 | Reload changes credential references mid-query | In-flight query uses old credentials (from v1 snapshot); no disruption |
| Many concurrent queries | 100 concurrent query executions | All read config lock-free; no contention |
| Old snapshot cleanup | reload completes; last v1 guard dropped | v1 Arc freed automatically |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for full canonical vectors.

| Scenario | Input | Expected Output |
|----------|-------|-----------------|
| Happy path — hot read | concurrent query reads config | Lock-free load(); no blocking |
| Mid-query reload | reload occurs while query executing | Query uses original snapshot; completes successfully |
| Snapshot cleanup | reload + last guard dropped | Old snapshot memory freed; no leak |
| 100 concurrent reads | 100 tasks call load() simultaneously | All succeed; no contention; wait-free |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| (placeholder) | VP to be assigned — verify in-flight query snapshot stability under reload |
| (placeholder) | VP to be assigned — verify no blocking on query hot path |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-030 |
| L2 Invariants | -- |
| Related BCs | BC-2.16.005 (reload_config — sole write path), DEC-037, DEC-039 |
| Priority | P0 |

## Changelog
| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.2 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description; added ## Invariants; added ## Error Conditions (from inline Error Handling); converted ## Traces → ## Traceability table; added ## Canonical Test Vectors; added ## Verification Properties; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial draft (used ## Traces section) |
