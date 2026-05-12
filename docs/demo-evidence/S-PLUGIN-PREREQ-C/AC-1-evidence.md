# AC-1 Evidence — page_size on Cursor Pagination

**Story:** S-PLUGIN-PREREQ-C v1.3
**Status: SATISFIED**
**Resolves:** TD-S-PLUGIN-PREREQ-B-001 (P2)
**BC anchor:** BC-2.16.002 postcondition — pagination within a step follows the sensor spec's declared
pagination config, iterating until the API returns an empty page or the cursor is null.

---

## AC Summary (quoted from story v1.3)

> `PaginationConfig::CursorToken` gains a new field `page_size: Option<u32>`. The
> `build_paged_url()` function appends a `page_size` query parameter to BOTH the first-call
> URL (no cursor yet) and all cursor-continuation URLs when `page_size` is `Some(n)`. When
> `page_size` is `None`, no `page_size` query parameter is appended.
>
> This closes the CrowdStrike GraphQL `first: N` real-world case: cursor APIs require the
> page-size parameter on every request including the first.

---

## Red Gate Tests

Three tests assert the URL-building behavior — all located in
`crates/prism-spec-engine/tests/ac_1_cursor_page_size_test.rs` (integration) and
duplicated as in-module unit tests in `crates/prism-spec-engine/src/pipeline.rs`:

| Test | Assertion |
|------|-----------|
| `test_BC_2_16_002_cursor_pagination_first_call_includes_page_size` | First call (no cursor): URL contains `page_size=50` |
| `test_BC_2_16_002_cursor_pagination_continuation_includes_page_size` | Continuation call (cursor present): URL contains both `page_size=50` and the cursor parameter |
| `test_BC_2_16_002_cursor_pagination_page_size_none_omitted` | `page_size: None`: URL contains no `page_size` query parameter |

---

## Real Test Output

```
$ cd /Users/jmagady/Dev/prism/.worktrees/S-PLUGIN-PREREQ-C && \
  cargo nextest run -p prism-spec-engine -E 'test(test_BC_2_16_002_cursor_pagination_)' --no-fail-fast

   Compiling prism-spec-engine v0.6.0 (crates/prism-spec-engine)
    Finished `test` profile [unoptimized + debuginfo] target(s) in 9.85s
────────────
 Nextest run ID 26e85f34-9f2d-4ce5-b3be-dc25f253ca5e with nextest profile: default
    Starting 6 tests across 18 binaries (321 tests skipped)
        PASS [   0.015s] (1/6) prism-spec-engine::ac_1_cursor_page_size_test test_BC_2_16_002_cursor_pagination_first_call_includes_page_size
        PASS [   0.016s] (2/6) prism-spec-engine::ac_1_cursor_page_size_test test_BC_2_16_002_cursor_pagination_continuation_includes_page_size
        PASS [   0.016s] (3/6) prism-spec-engine::ac_1_cursor_page_size_test test_BC_2_16_002_cursor_pagination_page_size_none_omitted
        PASS [   0.017s] (4/6) prism-spec-engine pipeline::cursor_page_size_tests::test_BC_2_16_002_cursor_pagination_page_size_none_omitted
        PASS [   0.018s] (5/6) prism-spec-engine pipeline::cursor_page_size_tests::test_BC_2_16_002_cursor_pagination_first_call_includes_page_size
        PASS [   0.018s] (6/6) prism-spec-engine pipeline::cursor_page_size_tests::test_BC_2_16_002_cursor_pagination_continuation_includes_page_size
────────────
     Summary [   0.018s] 6 tests run: 6 passed, 321 skipped
```

6 tests pass: 3 in the integration test file and 3 duplicated as in-module unit tests.

---

## Production Code Reference

**File:** `crates/prism-spec-engine/src/pipeline.rs`

The `build_paged_url_impl` function threads `page_size` into both execution paths:

- **First-call path:** When `PaginationConfig::CursorToken { page_size: Some(n), .. }` is present
  and no cursor exists yet, `build_paged_url_impl` appends `&page_size={n}` (or `?page_size={n}`
  for the first query parameter) to the base URL before the cursor parameter position.
- **Continuation path:** On subsequent calls (cursor non-empty), the same
  `PaginationConfig::CursorToken { page_size: ps_opt, .. }` destructure appends `page_size=n`
  alongside the cursor value. Both parameters appear in the query string.
- **None path:** When `page_size: None` matches, the parameter is entirely omitted — no key, no
  equals sign, no value. Backward compatibility is preserved for all pre-PREREQ-C sensor specs
  that use `CursorToken` without declaring `page_size`.

The public test-helper `build_paged_url_for_test` in the same file exposes `build_paged_url_impl`
under the `test-helpers` feature, allowing the integration tests in
`ac_1_cursor_page_size_test.rs` to exercise the function without spinning up a wiremock server.

**Context:** This change resolves the TD-S-PLUGIN-PREREQ-B-001 gap identified during PREREQ-B's
adversary cascade: the CrowdStrike GraphQL `first: N` pattern requires `page_size` on the very
first pagination call, not only on continuations. The TD noted that the PREREQ-B implementation
populated `page_size` only in the continuation branch, silently omitting it from the first call.

---

## Cross-References

- BC-2.16.002 v1.10 — postcondition: pagination config drives URL construction for all calls in
  the page sequence
- Fix-burst-1 closure (commit in feature/S-PLUGIN-PREREQ-C): initial `page_size` implementation
  plus Red Gate tests
- Adversary pass-1 finding that confirmed the first-call omission as the residual gap before
  fix-burst-1 landed
