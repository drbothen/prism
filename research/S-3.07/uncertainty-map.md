---
document_type: uncertainty-map
story_id: S-3.07
version: "1.0"
producer: dclaude-uncertainty-scanner
timestamp: "2026-05-04T00:00:00Z"
status: draft
---

# S-3.07 Uncertainty Map — Write Execution Pipeline

## Summary verdict

**RED** — Story explicitly self-flags a high-risk DataFusion API uncertainty
(line 478–481): "DataFusion's `insert_into()` signature changed in v53. Verify
… signature in DataFusion 53 before implementing." This is the most complex
cross-cutting story in W3 and depends on three subsystems each with their own
DataFusion / TableProvider integration assumptions.

## Findings

| Severity | Category | Finding | Recommended action |
|---|---|---|---|
| Critical | api-assumption | Lines 478–481: explicit self-flagged uncertainty about `TableProvider::insert_into()` signature change in DataFusion 53. Story estimated at 5 points and depends on this being correct. The trait now requires `InsertOp` enum (datafusion#10180-era change). | RESEARCH-NEEDED — BLOCKING: confirm exact `insert_into(&self, state: &dyn Session, input: Arc<dyn ExecutionPlan>, insert_op: InsertOp)` signature in 53.x; confirm whether `update`/`delete_from` exist as trait methods or require custom planner extension. |
| Critical | feature-claim | Lines 266–270: claims DataFusion `LogicalPlan` or `ExecutionPlan` carries enough info to extract a `WritePlan` and that WriteResult can be returned via `SendableRecordBatchStream`. DataFusion DML support has historically been incomplete (UPDATE/DELETE were not native trait methods until recently). | RESEARCH-NEEDED — BLOCKING: confirm DataFusion 53.x natively supports UPDATE/DELETE TableProvider methods, or whether Prism must implement these as a custom `ExtensionPlanner`/UDF. If the latter, story complexity grows materially. |
| Important | version-pin | Line 401: `datafusion 53.x` (no minor pin). | Match ADR-015 pin `=53.1.x`. |
| Important | architecture-pattern | Line 222: "Audit INTENT record (fail-closed)" → RocksDB via prism-audit. ADR-008 universal re-keying with `{org_id}:` prefix must thread through audit keys. | Confirm prism-audit AuditWriter API expects org_id-prefixed keys; this is W2 territory but W3 caller must populate. |
| Important | architecture-pattern | Cross-story coherence with S-3.06: this story consumes the parser AST from S-3.06. Story does not specify whether `WriteNode`/`DmlNode` enums are imported or duplicated. | Verify in implementation that one AST module is shared between parser and executor (single source of truth). |
| Suggestion | feature-claim | Lines 73–82 mention `dry_run.rs (Phase 4: WritePreview, ConfirmationToken gating)`. ConfirmationToken is BC-2.04.009 with 100-token cap. | Cross-check 100-token cap is still the BC-INDEX value; confirmed v4.32. |

## Cross-references

- **BLOCKING:** DataFusion 53 TableProvider write API.
- BCs BC-2.04.001/005/007/008 + BC-2.05.009 active.
- depends_on: S-3.06 (parser), S-3.02 (TableProvider base), S-1.08/09 (orgs), S-2.04 (feature flags), S-6.07 (write registry).
- ADR-008 org_id threading required.

## RESEARCH-NEEDED queries

1. "DataFusion 53.x TableProvider trait: exact signature of insert_into. Does the trait include update or delete_from methods natively, or are they extension-planner only? When was InsertOp enum introduced?"
2. "DataFusion 53.x: idiomatic pattern for implementing UPDATE and DELETE on a custom TableProvider. ExecutionPlan vs ExtensionPlanner approach."
3. "DataFusion 53.x SendableRecordBatchStream — return type for DML operations. Schema requirements for write result streams (count_inserted, count_updated, etc.)."

