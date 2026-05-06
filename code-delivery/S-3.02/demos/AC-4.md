# S-3.02 AC-4 — REQUIRED Column Push-Down (BC-2.11.007 v1.4)

**Story:** S-3.02 — prism-query: Query Tool and Materialization
**BC Anchor:** BC-2.11.007
**Acceptance Criterion:** Given a WHERE clause with `severity_id >= 3` where `severity_id` is a REQUIRED column on the CrowdStrike source, `severity_id >= 3` appears in `PushDownPlan.push_down` and is passed as a filter to the sensor adapter fetch call.

---

## Test Name

```
test_ac4_required_column_push_down
  (crates/prism-query/src/tests/integration_tests.rs)
```

## Terminal Output

```
running 1 test
test tests::integration_tests::tests::test_ac4_required_column_push_down ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 337 filtered out; finished in 0.00s
```

## Production Code Path

`crates/prism-query/src/pushdown.rs` — `classify_predicates()`

Classification logic:
```rust
pub fn classify_predicates(where_clause: &[Expr], columns: &[ColumnSpec]) -> PushDownPlan {
    // For each expr: extract column name → look up ColumnOptions in spec
    // Required | Index | Additional → push_down
    // Optimized | Default           → post_filter
}
```

Column option priority order (highest to lowest):
```
Required > Index > Additional > Optimized > Default
```

`ColumnOptions::Required` maps to `ColumnPushDownOption::Required`, which routes to `plan.push_down`.

## Test Logic Summary

- Declares a `ColumnSpec { name: "severity_id", options: [ColumnOptions::Required] }`.
- Constructs `Expr::Compare { lhs: Field("severity_id"), op: Ge, rhs: Literal(3) }`.
- Calls `classify_predicates(&[expr], &columns)`.
- Asserts `plan.push_down.len() == 1` and `plan.push_down[0].column_name == "severity_id"`.
- Asserts `plan.post_filter.len() == 0`.

The `Predicate` in `push_down` carries the original `Expr` and is passed to `fan_out()` as `QueryParams.filters` in the full pipeline.

## Push-Down Taxonomy (BC-2.11.007)

| Column Class | Push-Down Behavior |
|---|---|
| REQUIRED | Always push down (mandatory API param) |
| INDEX | Push down (native API filter) |
| ADDITIONAL | Push down (supplemental filter) |
| OPTIMIZED | Post-filter only (Prism-local) |
| DEFAULT | Post-filter only (no push-down) |

## Result

PASS — `severity_id` (REQUIRED) correctly classified as push-down; post-filter list is empty.
