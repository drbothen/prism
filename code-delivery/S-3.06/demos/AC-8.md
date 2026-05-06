# AC-8 — INSERT INTO SELECT

**Story:** S-3.06 v1.7 | **BC:** BC-2.11.004 | **Status:** PASS

## Criterion

Given `INSERT INTO crowdstrike_contained_hosts (device_id) SELECT device_id FROM
crowdstrike_hosts WHERE last_seen < 7d LIMIT 10`, when parsed in SQL mode, then
`DmlNode::InsertInto` is produced with `target_table = "crowdstrike_contained_hosts"`.

## Test Names

- Integration: `test_ac8_insert_into_select`
- Integration: `test_BC_2_11_004_dml_node_target_table_preserved_exactly`
- Integration: `test_BC_2_11_004_proptest_write_node_roundtrip`
- Unit (pub-crate): `test_BC_2_11_004_parse_sql_dml_delete_with_where`
- Unit (pub-crate): `test_BC_2_11_004_parse_sql_dml_delete_no_where_022`
- Unit (pub-crate): `test_BC_2_11_004_parse_sql_dml_update_prism_table_010`

## Evidence

### Integration test — primary assertion

```
test test_ac8_insert_into_select ... ok
```

Query: `INSERT INTO crowdstrike_contained_hosts (device_id) SELECT device_id FROM crowdstrike_hosts WHERE last_seen < 7d LIMIT 10`

Assertions:
```rust
match result.unwrap() {
    Ast::Sql(SqlStatement::Dml(node)) => {
        assert_eq!(node.operation, DmlOperation::InsertInto);
        assert_eq!(node.target_table, "crowdstrike_contained_hosts");
    }
    ...
}
```

### target_table preserved exactly

```
test test_BC_2_11_004_dml_node_target_table_preserved_exactly ... ok
```

Query: `INSERT INTO crowdstrike_contained_hosts (device_id) SELECT device_id FROM crowdstrike_hosts LIMIT 10`
Assertion: `node.target_table == "crowdstrike_contained_hosts"`

### Round-trip test (DELETE and UPDATE)

```
test test_BC_2_11_004_proptest_write_node_roundtrip ... ok
```

Verifies DELETE FROM crowdstrike_hosts WHERE id = 'x' produces Dml(Delete) and
UPDATE crowdstrike_hosts SET x = 'v' WHERE id = 'y' produces Dml(Update).

### AST type coverage

```rust
// write_ast.rs
pub enum DmlOperation { InsertInto, Update, Delete }
pub struct DmlNode {
    pub operation: DmlOperation,
    pub target_table: String,
    pub assignments: Vec<Assignment>,    // SET col=val pairs (UPDATE only)
    pub filter: Option<Expr>,            // WHERE clause (UPDATE/DELETE)
    pub source_select: Option<SqlQuery>, // SELECT (INSERT INTO only)
}
```

All three DML operations are covered across integration tests:
- `test_ac8_insert_into_select` — InsertInto
- `test_update_with_where` — Update
- `test_delete_from_with_where` — Delete

## Result

PASS — DmlNode::InsertInto produced with correct target_table; source_select carries
the bounded SELECT query.
