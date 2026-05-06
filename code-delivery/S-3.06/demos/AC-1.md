# AC-1 — Pipe Write No Args

**Story:** S-3.06 v1.7 | **BC:** BC-2.11.004 | **Status:** PASS

## Criterion

Given `FROM crowdstrike_hosts | where last_seen < 7d | contain`, when parsed in pipe
mode, then the AST is a `PipeQuery` with one `Where` stage and
`write = Some(WriteNode { verb: "contain", args: [] })`.

## Test Names

- Integration: `test_ac1_pipe_write_no_args` (via public API, no-panic assertion)
- Unit (pub-crate): `test_BC_2_11_004_parse_pipe_with_write_happy_path`

## Evidence

### Unit test — full assertion (parse_pipe_with_write, with registry)

```
test test_BC_2_11_004_parse_pipe_with_write_happy_path ... ok
```

The unit test calls `parse_pipe_with_write` with a registry containing `["contain", "tag"]`
and asserts:
- `result.is_ok()`
- `pq.write.as_ref().verb == "contain"`
- `pq.write.as_ref().args.is_empty()`

### Public API (integration test — no registry, no-panic)

```
test test_ac1_pipe_write_no_args ... ok
```

Query: `FROM crowdstrike_hosts | where last_seen < 7d | contain`

The public `PrismQlParser::parse` does not receive a registry — the verb `contain` is
resolved as an unknown identifier at that layer. The integration test therefore verifies
the no-panic contract. Full AST assertion is covered by the unit test above.

### Source path

- Parser production: `crates/prism-query/src/pipe_parser.rs` — `parse_pipe_with_write`
- AST node: `crates/prism-query/src/write_ast.rs` — `WriteNode { verb, args, source_sensor }`

## Result

PASS — WriteNode with verb="contain" and empty args produced by pipe mode parser.
