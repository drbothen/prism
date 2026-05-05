---
document_type: uncertainty-map
story_id: S-3.11
version: "1.0"
producer: dclaude-uncertainty-scanner
timestamp: "2026-05-04T00:00:00Z"
status: draft
---

# S-3.11 Uncertainty Map — In-Query Dedup Caching

## Summary verdict

**YELLOW** — Conceptually distinct from S-3.05 (in-query dedup vs cross-query
response cache), but the boundary should be confirmed. Caches `RecordBatch`
which has known schema-equality and clone-cost considerations.

## Findings

| Severity | Category | Finding | Recommended action |
|---|---|---|---|
| Important | architecture-pattern | Cross-story coherence with S-3.05: S-3.11 caches per-query (intra-query), keyed by push-down params (line 66 `cache_key.rs (deterministic hash of push_down params)`); S-3.05 caches across queries (inter-query) keyed by query hash + tenant. Both touch sensor fan-out path. | Verify both caches are architecturally distinct in implementation: S-3.11 instance lives inside a single QueryEngine::execute() invocation; S-3.05 instance is process-global. Confirm with implementer. |
| Important | feature-claim | Line 213: "no caching between clients (data isolation requirement)" — implies cache key includes client_id even within a single query. ADR-008 org_id threading. | Confirm cache_key includes both client_id AND org_id where multi-tenant single-query fan-out occurs. |
| Suggestion | api-assumption | Line 233: `arrow 53 | RecordBatch (cached type)`. Caching `RecordBatch` requires `Arc`-wrapping for cheap clones — Arrow already does this internally for buffers but `RecordBatch` itself is `Clone` and shares buffers. | LOW: confirm idiomatic pattern (likely `Arc<Vec<RecordBatch>>`). |
| Suggestion | version-pin | Line 233: `arrow 53` no minor pin. | Match S-3.02. |

## Cross-references

- depends_on: S-3.02.
- frontmatter `behavioral_contracts: []` — verify with orchestrator.
- Coherence with S-3.05 (Tier 3 parallel): both run in the 8-way parallel batch.

## RESEARCH-NEEDED queries

None at HIGH; one MEDIUM:

1. "Arrow 53.x RecordBatch: cost of `clone()` (shared buffers) vs explicit `Arc<RecordBatch>`. Best practice for caching multiple RecordBatches in Rust async code."

