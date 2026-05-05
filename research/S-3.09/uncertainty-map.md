---
document_type: uncertainty-map
story_id: S-3.09
version: "1.0"
producer: dclaude-uncertainty-scanner
timestamp: "2026-05-04T00:00:00Z"
status: draft
---

# S-3.09 Uncertainty Map — Query Performance Profiling

## Summary verdict

**YELLOW** — Single substantive uncertainty: `MemoryPool::reserved()` call
on DataFusion's pool API. This trait has had method renames across the 30→50
range and may differ between 53.0 and 53.x patches.

## Findings

| Severity | Category | Finding | Recommended action |
|---|---|---|---|
| Important | api-assumption | Lines 114, 229, 249: `GreedyMemoryPool` / `MemoryPool::reserved()` for peak memory tracking. The `MemoryPool` trait has methods like `reserved()`, `register_consumer()`, `try_grow()` — names have varied across DataFusion versions. | RESEARCH-NEEDED: confirm `MemoryPool::reserved() -> usize` is current API in datafusion 53.x. Some versions used `consumed()` instead. |
| Suggestion | feature-claim | Line 230: claim that `MemoryPool::reserved()` returns "DataFusion-tracked allocation, not process RSS" and complements the watchdog. Architecturally sound but exact metric semantics need confirmation. | RESEARCH-NEEDED: clarify whether `reserved()` is high-water-mark or current — story uses term `memory_peak_bytes`, implying high-water-mark. May require periodic sampling rather than single read. |
| Suggestion | version-pin | Line 249: `datafusion 53` no minor pin. | Match S-3.02 pin. |
| Tech Debt | feature-claim | Ring buffer for last 100 queries (line 67) — no specific concurrent data-structure cited. `parking_lot::RwLock<VecDeque>` is conventional but unpinned. | Decide library when implementing; `parking_lot` is fine if already a transitive dep. |

## Cross-references

- depends_on: S-3.02.
- blocks: S-3.10 (cost estimation reads from this).
- frontmatter `behavioral_contracts: []` — verify with orchestrator that no BC is required.

## RESEARCH-NEEDED queries

1. "DataFusion 53.x MemoryPool trait — current method names. Is `reserved()` still the high-water tracker, or is it `consumed()` / `current()`?"
2. "DataFusion 53.x: how to read peak memory consumption per query (single SessionContext) for diagnostics."

