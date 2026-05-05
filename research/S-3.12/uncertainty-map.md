---
document_type: uncertainty-map
story_id: S-3.12
version: "1.0"
producer: dclaude-uncertainty-scanner
timestamp: "2026-05-04T00:00:00Z"
status: draft
---

# S-3.12 Uncertainty Map — Column Pruning and Field Selection Push-Down

## Summary verdict

**YELLOW** — Single substantive concern: writing a custom DataFusion
`OptimizerRule` is a public-API extension surface that has changed across
DataFusion majors. The trait method names (`try_optimize` vs `rewrite`) and
signatures shifted between 30s and 50s.

## Findings

| Severity | Category | Finding | Recommended action |
|---|---|---|---|
| Important | api-assumption | Lines 191, 203: implements DataFusion `OptimizerRule` for column pruning. The trait surface has historically been: `try_optimize(&self, plan: &LogicalPlan, config: &OptimizerConfig) -> Result<Option<LogicalPlan>>` and was reorganized in later versions to use `rewrite()` with `Transformed<LogicalPlan>` / `TreeNode` patterns. | RESEARCH-NEEDED: confirm `OptimizerRule` trait shape in datafusion 53.x. Also confirm whether DataFusion 53.x already has built-in projection pushdown (`PushDownProjection`) such that this story is additive vs duplicative. |
| Important | feature-claim | Story claims pruning runs as a DataFusion optimizer rule (line 191) but elsewhere implies the AST walker (line 65, `column_usage.rs`) extracts `used_columns` per mode pre-DataFusion. Two pruning surfaces. | Clarify: is column-usage extraction done in PrismQL AST (pre-DataFusion) for sensor field selection push-down, AND a DataFusion OptimizerRule for post-fetch pruning? Or is one redundant? |
| Suggestion | feature-claim | Line 67: `sensor adapter changes (fields: Option<Vec<String>> param)` — relies on every sensor adapter accepting an optional `fields` projection list. | Confirm sensor specs all have `supports_field_selection` flag and consistent `field_selection_param` (line 68). |
| Suggestion | version-pin | No explicit datafusion version pinned in this story (no dependency table found). | Add explicit dep table; pin to S-3.02 baseline. |

## Cross-references

- depends_on: S-3.02 (TableProvider) + S-2.06 (sensor adapter base).
- frontmatter `behavioral_contracts: []` — verify with orchestrator.

## RESEARCH-NEEDED queries

1. "DataFusion 53.x OptimizerRule trait — current signature. Has `try_optimize` been replaced by `rewrite` with TreeNode in 53.x?"
2. "DataFusion 53.x built-in optimizer rules — does PushDownProjection exist by default? What is the rule's name and behavior with custom TableProviders?"

