---
document_type: uncertainty-map
story_id: S-3.03
version: "1.0"
producer: dclaude-uncertainty-scanner
timestamp: "2026-05-04T00:00:00Z"
status: draft
---

# S-3.03 Uncertainty Map — Explain and Query Diagnostics

## Summary verdict

**YELLOW** — Inherits S-3.02's DataFusion 53.x uncertainty. Adds two specific
API claims about logical-plan generation against schema-only MemTables that
need verification.

## Findings

| Severity | Category | Finding | Recommended action |
|---|---|---|---|
| Important | api-assumption | Line 88: `SessionContext::create_logical_plan()` — this method has been renamed/removed across DataFusion versions. In modern 4x/5x it is `state().create_logical_plan()` or via `SessionState`. | RESEARCH-NEEDED: confirm correct API in 53.x. Story may need update to `SessionContext::state().create_logical_plan(sql)`. |
| Important | api-assumption | Line 243: `LogicalPlan::display_indent()` — display API has changed across DataFusion versions; some methods moved to `display_pg_json` / `display_indent_schema`. | RESEARCH-NEEDED: confirm `display_indent()` exists and produces the expected text format in 53.x. |
| Suggestion | feature-claim | Line 184: "schema-only MemTables" implies MemTable construction with empty `Vec<RecordBatch>` is valid for plan generation. | Verify MemTable accepts empty RecordBatch vector at construction (or use `EmptyTable` provider). |
| Suggestion | version-pin | Line 195: `datafusion 53` — same as S-3.02; no minor pin. | Match S-3.02 pin (`=53.x.y`). |

## Cross-references

- Inherits all S-3.02 DataFusion concerns.
- BC-2.11.010 active in BC-INDEX v4.32.

## RESEARCH-NEEDED queries

1. "DataFusion 53.x: correct API to obtain a LogicalPlan from a SQL string. Is `SessionContext::create_logical_plan` still public, or must we go through `state()`?"
2. "DataFusion 53.x LogicalPlan display methods — confirm `display_indent()` signature and output format."

