---
document_type: verification-property
level: L4
version: "1.1"
status: draft
producer: architect
timestamp: 2026-04-19T00:00:00
phase: 1c
inputs: [VP-INDEX.md, S-3.02-query-materialization.md]
input-hash: "ac35fdd"
traces_to: architecture/verification-architecture.md
source_bc: BC-2.11.011
module: prism-query
priority: P0
proof_method: proptest
verification_method: proptest
feasibility: medium
verification_lock: false
proof_completed_date: null
proof_file_hash: null
lifecycle_status: active
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
withdrawn: null
withdrawal_reason: null
removed: null
removal_reason: null
---

# VP-031: Required Column Enforcement — Rejects Unconstrained Queries

## Property Statement

For every planned query, if the query touches a table whose schema declares one or
more required columns (e.g. time bounds, tenant scope) and the query does not place
a binding predicate on each required column, the planner returns
`Err(PlanError::MissingRequiredColumnConstraint)` before materialization.
Unconstrained queries on guarded tables are never executed.

## Source Contract

- **Anchor Story:** `S-3.02-query-materialization.md`
- **Source BC:** BC-2.11.011 — Required Column Enforcement
- **Module:** prism-query
- **Category:** Security

## Proof Method

| Method | Tool | Bounded? | Coverage |
|--------|------|----------|----------|
| proptest | proptest (latest) | No — random query plans | Satisfied + unsatisfied constraint cases |

## Proof Harness Skeleton

```rust
// [TODO: harness skeleton — author during Phase 5 formal-verify]
// Method: proptest
// Target: prism_query::planner::plan
//
// Sketch: generate arbitrary plans over tables with known required columns;
// assert planner returns Err when any required column lacks a binding predicate,
// and Ok otherwise.
```

## Feasibility Assessment

| Factor | Assessment | Notes |
|--------|------------|-------|
| Bounded inputs? | No | Size-bounded plan generator |
| Tool support? | Full | proptest |
| Execution time budget | <60 seconds for 10k cases | Planner checks are fast |
| Assumptions required | Per-table required-column metadata stable during test | Fixture |

## Lifecycle

| Event | Date | Actor |
|-------|------|-------|
| introduced | 2026-04-14 | architect |

## Changelog

| Version | Burst | Date | Author | Notes |
|---------|-------|------|--------|-------|
| 1.1 | pre-build-sweep | 2026-04-20 | architect | Template-compliance sweep: added priority frontmatter (from VP-INDEX v1.5); added verification_method alias (proof_method retained for backward compat). |
