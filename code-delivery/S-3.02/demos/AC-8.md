# S-3.02 AC-8 / VP-031 — Push-Down Property Test Corpus

**Story:** S-3.02 — prism-query: Query Tool and Materialization
**BC Anchor:** BC-2.11.007
**Verification Property:** VP-031
**Acceptance Criterion:** VP-031 proptest passes: REQUIRED columns are always classified as `PushDown` for all generated predicate trees.

---

## Test Names (all in `crates/prism-query/src/proofs/vp031_pushdown.rs`)

```
proofs::vp031_pushdown::kani_proofs::prop_required_columns_always_push_down
proofs::vp031_pushdown::kani_proofs::prop_no_predicate_silently_dropped
proofs::vp031_pushdown::kani_proofs::prop_post_filter_only_predicates_never_in_push_down
proofs::vp031_pushdown::kani_proofs::prop_classify_predicates_is_deterministic
```

Extended corpus (in `crates/prism-query/src/tests/bc_gap_fill_tests.rs`):
```
prop_BC_2_11_007_empty_predicate_list_both_empty
prop_BC_2_11_007_additional_columns_always_push_down
prop_BC_2_11_007_index_columns_always_push_down
prop_BC_2_11_007_default_columns_never_push_down
prop_BC_2_11_007_optimized_columns_never_push_down
prop_BC_2_11_007_mixed_predicates_split_correctly
```

## Terminal Output (10 proptest cases, PROPTEST_CASES=32)

```
running 10 tests
test proofs::vp031_pushdown::kani_proofs::prop_required_columns_always_push_down ... ok
test proofs::vp031_pushdown::kani_proofs::prop_classify_predicates_is_deterministic ... ok
test tests::bc_gap_fill_tests::bc_gap_fill::vp031_extended::prop_BC_2_11_007_empty_predicate_list_both_empty ... ok
test tests::bc_gap_fill_tests::bc_gap_fill::vp031_extended::prop_BC_2_11_007_additional_columns_always_push_down ... ok
test proofs::vp031_pushdown::kani_proofs::prop_post_filter_only_predicates_never_in_push_down ... ok
test tests::bc_gap_fill_tests::bc_gap_fill::vp031_extended::prop_BC_2_11_007_index_columns_always_push_down ... ok
test tests::bc_gap_fill_tests::bc_gap_fill::vp031_extended::prop_BC_2_11_007_default_columns_never_push_down ... ok
test proofs::vp031_pushdown::kani_proofs::prop_no_predicate_silently_dropped ... ok
test tests::bc_gap_fill_tests::bc_gap_fill::vp031_extended::prop_BC_2_11_007_optimized_columns_never_push_down ... ok
test tests::bc_gap_fill_tests::bc_gap_fill::vp031_extended::prop_BC_2_11_007_mixed_predicates_split_correctly ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 328 filtered out; finished in 0.00s
```

## Property Inventory (VP-031)

| Property | Invariant Tested |
|---|---|
| `prop_required_columns_always_push_down` | REQUIRED columns always in `push_down`, never `post_filter` |
| `prop_no_predicate_silently_dropped` | Every predicate appears in exactly one list (never dropped) |
| `prop_post_filter_only_predicates_never_in_push_down` | DEFAULT/OPTIMIZED columns always in `post_filter` |
| `prop_classify_predicates_is_deterministic` | Same inputs always produce same `PushDownPlan` |
| `prop_BC_2_11_007_empty_predicate_list_both_empty` | Empty WHERE clause → both lists empty |
| `prop_BC_2_11_007_additional_columns_always_push_down` | ADDITIONAL columns → `push_down` |
| `prop_BC_2_11_007_index_columns_always_push_down` | INDEX columns → `push_down` |
| `prop_BC_2_11_007_default_columns_never_push_down` | DEFAULT → `post_filter` only |
| `prop_BC_2_11_007_optimized_columns_never_push_down` | OPTIMIZED → `post_filter` only |
| `prop_BC_2_11_007_mixed_predicates_split_correctly` | Mixed predicate list splits correctly |

## Proptest Corpus Strategy

REQUIRED column names generated from:
```rust
prop_oneof![
    Just("customer_id"),   // CrowdStrike
    Just("device_id"),     // CrowdStrike
    Just("org_id"),        // Cyberint
    Just("site_id"),       // Claroty
    Just("organizationId"), // Armis
]
```

Each test runs 32 cases (`PROPTEST_CASES=32` for iteration speed; CI runs the default 256).

## Result

PASS — all 10 VP-031 property tests pass; `classify_predicates` satisfies all invariants across the full column option taxonomy.
