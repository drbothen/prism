# AC-2 Evidence — JSONPath Bracket Notation + Wildcard + Bounds Check

**Story:** S-PLUGIN-PREREQ-C v1.3
**Status: SATISFIED**
**Resolves:** TD-S-PLUGIN-PREREQ-B-003 (P3)
**BC anchor:** BC-2.16.002 postcondition — variable interpolation uses `${step_name.field}` where
`field` is a JSONPath-like path; this story extends that surface to bracket notation and wildcards.

---

## AC Summary (quoted from story v1.3)

> The `extract_at_path` function is extended beyond dot-notation to support:
> - **Bracket indexing:** `$.x[0]` extracts the first element of array `x`.
> - **Wildcard enumeration:** `$.x[*]` extracts all elements of array `x` and returns them
>   as a `Vec<serde_json::Value>`.
>
> **Backward compatibility:** All existing dot-path tests continue to pass unchanged.
>
> **Bounds-check:** `$.x[99]` on a 3-element array returns a structured error (not a panic
> and not a silent `None`).

---

## Red Gate Tests

All four tests reside in the `pipeline::jsonpath_bracket_tests` in-module block within
`crates/prism-spec-engine/src/pipeline.rs`:

| Test | Assertion |
|------|-----------|
| `test_BC_2_16_002_extract_bracket_index` | `$.devices[0].id` on `[{"id":"A"},{"id":"B"}]` extracts `"A"` |
| `test_BC_2_16_002_extract_wildcard_enumeration` | `$.devices[*].id` on same input returns JSON array `["A","B"]` |
| `test_BC_2_16_002_extract_backward_compat_dot_path` | `$.resources` on `{"resources":[{"id":1}]}` still returns the full array |
| `test_BC_2_16_002_extract_bracket_out_of_bounds_structured_error` | `$.x[99]` on `{"x":[1,2,3]}` returns `Err(...)` — no panic |

---

## Real Test Output

```
$ cd /Users/jmagady/Dev/prism/.worktrees/S-PLUGIN-PREREQ-C && \
  cargo nextest run -p prism-spec-engine -E 'test(test_BC_2_16_002_extract_)' --no-fail-fast

    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.31s
────────────
 Nextest run ID 6def0c08-3458-4d6b-8a11-940c12d6bf83 with nextest profile: default
    Starting 4 tests across 18 binaries (323 tests skipped)
        PASS [   0.011s] (1/4) prism-spec-engine pipeline::jsonpath_bracket_tests::test_BC_2_16_002_extract_bracket_index
        PASS [   0.011s] (2/4) prism-spec-engine pipeline::jsonpath_bracket_tests::test_BC_2_16_002_extract_backward_compat_dot_path
        PASS [   0.011s] (3/4) prism-spec-engine pipeline::jsonpath_bracket_tests::test_BC_2_16_002_extract_wildcard_enumeration
        PASS [   0.011s] (4/4) prism-spec-engine pipeline::jsonpath_bracket_tests::test_BC_2_16_002_extract_bracket_out_of_bounds_structured_error
────────────
     Summary [   0.011s] 4 tests run: 4 passed, 323 skipped
```

All 4 tests pass in 11ms (in-module, pure computation — no I/O).

---

## Production Code Reference

**File:** `crates/prism-spec-engine/src/pipeline.rs`

The implementation uses an in-tree tokenizer rather than an external crate, keeping the
dependency surface clean for `cargo deny` and `cargo audit`.

**`extract_at_path`** is the public entry point. It tokenizes the path string and delegates
to `extract_with_tokens`, threading an `ExtractionContext` accumulator.

**`extract_with_tokens`** handles three token types:
- Plain key segments (dot-notation, RFC 6901 escaping for `~0`/`~1`)
- Bracket index tokens (e.g., `[0]`) — extracts a specific array element by 0-based index
- Wildcard tokens (`[*]`) — iterates all array elements and collects results into a JSON array

**`ExtractionContext`** is a resource-bounds accumulator threaded through recursive wildcard
calls. It tracks recursion depth and total elements produced to prevent:
- O(N^k) memory amplification from nested wildcards such as `$.a[*].b[*]`
- Stack overflow from deeply nested `[*]` selectors (depth cap enforced, HIGH-007 closure)

When a bracket index exceeds the array length, `extract_with_tokens` returns a structured
`Err(String)` error. The error includes the path expression and the actual array length,
allowing the caller to emit a structured `jsonpath_extraction_failed` event (catalogued in
BC-2.16.002 Structured Event Catalog row 15). A second catalog row — `jsonpath_size_cap_exceeded`
(row 16) — covers the `ExtractionContext` depth/element cap enforcement path.

Both catalog rows were added to BC-2.16.002 v1.10 in the same atomic commit as the code
change, satisfying PG-LP11-001 (BC↔catalog discipline from lessons.md Lesson 1).

---

## Cross-References

- BC-2.16.002 v1.10 Structured Event Catalog rows 15 (`jsonpath_extraction_failed`) and
  16 (`jsonpath_size_cap_exceeded`) — added in fix-burst-1 per PG-LP11-001 SOP
- HIGH-007 closure (fix-burst-2): `ExtractionContext` depth + element caps prevent adversarial
  wildcard memory amplification
- Adversary pass-1 finding (F-LP1-HIGH-007): flagged missing bounds on nested wildcards before
  `ExtractionContext` was introduced
