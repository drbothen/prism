# AC-7 — Unbounded DML Rejection (EC-11-062)

**Story:** S-3.06 v1.7 | **BC:** BC-2.11.004 | **Status:** PASS

## Criterion

Given `DELETE FROM armis_device_tags` with no WHERE clause, when parsed, then
`E-QUERY-022` is returned suggesting the analyst add a WHERE clause.

## Test Names

- Integration: `test_ac7_delete_without_where`
- Integration: `test_update_without_where`
- Integration: `test_BC_2_11_004_insert_select_without_limit_or_where_is_unbounded`
- Integration: `test_BC_2_11_004_insert_select_with_where_is_bounded`
- Integration: `test_BC_2_11_004_insert_select_with_limit_is_bounded`
- Unit (pub-crate): `test_BC_2_11_004_check_unbounded_write_delete_no_where`
- Unit (pub-crate): `test_BC_2_11_004_check_unbounded_write_update_no_where`
- Unit (pub-crate): `test_BC_2_11_004_check_unbounded_write_delete_with_where_is_safe`
- Unit (pub-crate): `test_BC_2_11_004_check_unbounded_write_insert_no_limit_no_where`
- Unit (pub-crate): `test_BC_2_11_004_check_unbounded_write_insert_with_limit_is_safe`
- Error constructor: `test_BC_2_11_004_error_022_message_contains_code_and_suggestion`

## Evidence

### Integration test — primary assertion

```
test test_ac7_delete_without_where ... ok
```

Query: `DELETE FROM armis_device_tags`

Assertions:
- `result.is_err()`
- `result.unwrap_err()[0].message.contains("E-QUERY-022")`

### UPDATE without WHERE also rejected

```
test test_update_without_where ... ok
```

Query: `UPDATE armis_devices SET status = 'resolved'`

### INSERT without LIMIT or WHERE rejected

```
test test_BC_2_11_004_insert_select_without_limit_or_where_is_unbounded ... ok
```

Query: `INSERT INTO armis_tags (id) SELECT id FROM events`

### Bounded operations accepted

```
test test_BC_2_11_004_insert_select_with_where_is_bounded ... ok
test test_BC_2_11_004_insert_select_with_limit_is_bounded ... ok
```

### Unit tests — check_unbounded_write directly

```
test test_BC_2_11_004_check_unbounded_write_delete_no_where ... ok
test test_BC_2_11_004_check_unbounded_write_update_no_where ... ok
test test_BC_2_11_004_check_unbounded_write_delete_with_where_is_safe ... ok
test test_BC_2_11_004_check_unbounded_write_insert_no_limit_no_where ... ok
test test_BC_2_11_004_check_unbounded_write_insert_with_limit_is_safe ... ok
```

### Error message format

```
E-QUERY-022: unbounded DELETE rejected — add a WHERE clause
(or LIMIT for INSERT...SELECT) to scope the operation,
or use explicit opt-in if provided by the sensor spec
```

### Guard implementation

```rust
pub(crate) fn check_unbounded_write(node: &DmlNode, offset: usize) -> Option<ParseError>
```

Called after DML parse succeeds but before returning `Ast::Sql(SqlStatement::Dml(node))`.

## Result

PASS — E-QUERY-022 returned for DELETE/UPDATE without WHERE and INSERT without LIMIT/WHERE.
