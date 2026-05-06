# AC-6 — Pipe Write with Literal Args

**Story:** S-3.06 v1.7 | **BC:** BC-2.11.004 | **Status:** PASS

## Criterion

Given `FROM crowdstrike_hosts | where zone = "OT" | tag key="review" value="pending"`,
when parsed, then `WriteNode.args` contains
`[WriteArg { key: "key", value: "review" }, WriteArg { key: "value", value: "pending" }]`.

## Test Names

- Integration: `test_ac6_pipe_write_with_args` (no-panic, public API)
- Unit (pub-crate): `test_BC_2_11_004_parse_pipe_with_write_multiple_args`
- Unit (pub-crate): `test_BC_2_11_004_write_arg_integer_literal`
- Unit (pub-crate): `test_BC_2_11_004_write_arg_boolean_literal`

## Evidence

### Unit test — full WriteArg assertion

```
test test_BC_2_11_004_parse_pipe_with_write_multiple_args ... ok
```

Query: `FROM crowdstrike_hosts | tag key="review" value="pending"`
Registry: `["tag"]`

Assertions:
- `write.verb == "tag"`
- `write.args.len() == 2`
- Args contain `WriteArg { key: "key", value: Literal::String("review") }`
- Args contain `WriteArg { key: "value", value: Literal::String("pending") }`

### Integer and boolean literal args

```
test test_BC_2_11_004_write_arg_integer_literal ... ok    # FROM hosts | tag priority=42
test test_BC_2_11_004_write_arg_boolean_literal ... ok    # FROM hosts | tag critical=true
```

Supports string, integer, and boolean literal values in write args.

### AST types

```rust
// write_ast.rs
pub struct WriteArg {
    pub key: String,
    pub value: Literal,  // Literal::String | Literal::Int | Literal::Bool | ...
}
```

## Result

PASS — WriteNode.args correctly populated with key=value pairs from pipe write stage.
