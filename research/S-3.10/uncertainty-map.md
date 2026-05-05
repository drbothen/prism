---
document_type: uncertainty-map
story_id: S-3.10
version: "1.0"
producer: dclaude-uncertainty-scanner
timestamp: "2026-05-04T00:00:00Z"
status: draft
---

# S-3.10 Uncertainty Map — Cost Estimation with API Latency Awareness

## Summary verdict

**YELLOW** — Single architectural claim worth verifying: "Cost-based JOIN
ordering is a hint to DataFusion — Prism MUST NOT disable DataFusion's own
optimizer. Set the initial logical plan order based on cost estimates, then
let DataFusion optimize further." Whether DataFusion 53.x respects table-scan
ordering as a JOIN-ordering hint is a load-bearing assumption.

## Findings

| Severity | Category | Finding | Recommended action |
|---|---|---|---|
| Important | feature-claim | Lines 126, 224–226: claim that initial LogicalPlan table ordering is honored as a hint by DataFusion's join reorderer. Modern DataFusion uses statistics-driven join reordering (`OptimizerRule::join_selection`) which may override caller intent if statistics are absent. | RESEARCH-NEEDED: confirm DataFusion 53.x join-reorder rule behavior when no statistics are present (Prism does not pre-stat ephemeral MemTables). Determine whether ordering is preserved or overwritten. |
| Important | architecture-pattern | Lines 305–306: relies on DataFusion's hash join build-side selection ("smaller table loaded into memory"). DataFusion's choice of build-side is internal. | RESEARCH-NEEDED: confirm DataFusion 53.x exposes a way to inspect or hint hash-join build-side, or whether the cost estimator must accept that this is opaque. |
| Suggestion | feature-claim | Line 221: "RocksDB scan path is always ~5ms (local RocksDB scan)" — performance assumption baked into cost model. Could shift after Wave 2 hot-path changes. | Tag as observation, not pin; confirm via benchmark before relying on the constant. |
| Suggestion | version-pin | Line 245: `datafusion 53` no minor pin. | Match S-3.02. |

## Cross-references

- depends_on: S-3.09 (profiling) for histogram observations + S-3.02.
- frontmatter `behavioral_contracts: []` — verify with orchestrator.
- Sister-story: ensure S-3.09's `total_rows_fetched` and `actual_latency_ms` schema match what S-3.10's histogram consumes.

## RESEARCH-NEEDED queries

1. "DataFusion 53.x optimizer behavior on join reordering when no table statistics are provided. Does it preserve plan input order, or does it use a default heuristic?"
2. "DataFusion 53.x: how to disable individual optimizer rules or provide manual hints for join order. Is `disable_rule` still available?"

