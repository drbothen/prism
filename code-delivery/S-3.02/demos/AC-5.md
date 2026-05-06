# S-3.02 AC-5 — None Clients Fans Out to All

**Story:** S-3.02 — prism-query: Query Tool and Materialization
**BC Anchor:** BC-2.11.011
**Acceptance Criterion:** Given `execute(query, clients: None)`, all configured client IDs are included in the fan-out targets and results from all clients appear in the response.

---

## Test Name

```
test_ac5_none_clients_fans_out_to_all
  (crates/prism-query/src/tests/integration_tests.rs)
```

## Terminal Output

```
running 1 test
test tests::integration_tests::tests::test_ac5_none_clients_fans_out_to_all ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 337 filtered out; finished in 0.00s
```

## Production Code Path

`crates/prism-query/src/scoping.rs` — `resolve_clients()`

```rust
pub fn resolve_clients(
    clients: Option<Vec<OrgSlug>>,
    registry: &ClientRegistry,
) -> Result<Vec<OrgSlug>, PrismError> {
    match clients {
        None => Ok(registry.all_clients().to_vec()),   // fan-out to ALL
        Some(list) => {
            // validate each slug exists; return list
        }
    }
}
```

`ClientRegistry::all_clients()` returns the full slice of configured `OrgSlug` values. When `clients` is `None`, the resolver returns all of them without filtering.

## Test Logic Summary

- Creates `ClientRegistry::new(vec!["acme", "contoso", "globex"])` (3 clients).
- Calls `resolve_clients(None, &registry)`.
- Asserts `clients.len() == 3`.
- Asserts each of `"acme"`, `"contoso"`, `"globex"` is present in the result.

In the full pipeline, the returned `Vec<OrgSlug>` is passed to the fan-out stage, which spawns one tokio task per (client, source) pair.

## BC-2.11.011 Intersection Semantics

- Tool parameters define the outer boundary (None = "all configured clients").
- WHERE `_client = "..."` predicates narrow within that boundary.
- Predicates cannot widen scope — out-of-scope clients always produce empty results.

## Result

PASS — `clients: None` correctly resolves to all 3 configured clients.
