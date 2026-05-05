---
document_type: uncertainty-map
story_id: S-3.13
version: "1.0"
producer: dclaude-uncertainty-scanner
timestamp: "2026-05-04T00:00:00Z"
status: draft
---

# S-3.13 Uncertainty Map — Dynamic Table Availability

## Summary verdict

**YELLOW** — Story uses DataFusion's catalog/table-registration APIs which
are public but have non-trivial concurrency semantics (deregistration during
in-flight queries). The arc-swap config integration is sound but the
DataFusion side needs confirmation.

## Findings

| Severity | Category | Finding | Recommended action |
|---|---|---|---|
| Important | api-assumption | Lines 191, 203, 241: dynamic `SessionContext::register_table()` / `deregister_table()` during runtime. DataFusion's catalog APIs are publicly stable but the in-flight-query semantics (does deregistration affect already-planned queries?) are not strongly documented. | RESEARCH-NEEDED: confirm DataFusion 53.x `SessionContext::register_table` and `deregister_table` signatures and their thread-safety guarantees during concurrent query execution. |
| Important | architecture-pattern | Line 132: "Call `registered_tables()` at explain time (not cached in the plan)". Implies a fresh catalog snapshot per explain. Hot-reload (line 68) drives a config-swap → table-delta. | Verify atomicity: does the arc-swap snapshot of ConfigSnapshot align with a single `register/deregister` transaction on SessionContext? Race window risk. |
| Suggestion | feature-claim | Line 94: "{sensor_id}_{table_name}" naming convention. Naming choice rather than tech uncertainty, but DataFusion identifier rules (case-sensitivity, max length) constrain it. | Confirm DataFusion default identifier rules accept `{sensor_id}_{table_name}` (must avoid reserved characters and reserved keywords). |
| Suggestion | version-pin | Line 241: `datafusion 53` no minor pin. | Match S-3.02. |

## Cross-references

- depends_on: S-3.02 (catalog) + S-1.12 (config snapshot / hot-reload).
- frontmatter `behavioral_contracts: []` — verify with orchestrator.

## RESEARCH-NEEDED queries

1. "DataFusion 53.x SessionContext register_table / deregister_table — thread-safety guarantees with concurrent query execution. Are in-flight queries' table references invalidated mid-execution?"
2. "DataFusion 53.x catalog + schema provider trait. Custom CatalogProvider vs direct register_table approach for dynamic table availability."

