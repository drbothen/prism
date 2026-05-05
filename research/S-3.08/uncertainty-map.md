---
document_type: uncertainty-map
story_id: S-3.08
version: "1.0"
producer: dclaude-uncertainty-scanner
timestamp: "2026-05-04T00:00:00Z"
status: draft
---

# S-3.08 Uncertainty Map — Hidden Columns

## Summary verdict

**GREEN** — Mostly arrow `RecordBatch::project` mechanical work. Single
substantive concern is one `arrow 53` API call (`RecordBatch::project()`)
that should be sanity-checked against the latest 53.x.

## Findings

| Severity | Category | Finding | Recommended action |
|---|---|---|---|
| Suggestion | api-assumption | Line 215: `arrow 53 | RecordBatch::project() for column filtering`. The method exists but has been `&self -> Result<RecordBatch>` for a while. | LOW risk; confirm signature when implementing. |
| Suggestion | architecture-pattern | Lines 202–203 explicitly warn: "Do NOT use DataFusion column metadata or Arrow field metadata to store the hidden flag — DataFusion may not preserve arbitrary field metadata through projections." This is a known DataFusion limitation. | Architecture decision is sound; story stays out of metadata. No action needed. |
| Suggestion | version-pin | `arrow 53` no minor pin (line 215). | Match S-3.02 / S-3.07 pin. |

## Cross-references

- depends_on: S-3.02 only.
- No BCs cited (story has empty `behavioral_contracts:` and `anchor_bcs:` lists in frontmatter — verify this is intentional with orchestrator).

## RESEARCH-NEEDED queries

None at HIGH/MEDIUM level. LOW: confirm `arrow::record_batch::RecordBatch::project` signature in arrow 53.x latest.

