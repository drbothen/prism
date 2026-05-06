# AC-3 — Internal prism_* Table Write Protection (EC-11-061)

**Story:** S-3.06 v1.7 | **BC:** BC-2.11.004 | **Status:** PASS

## Criterion

Given `UPDATE prism_alerts SET status = 'resolved'`, when parsed, then `E-QUERY-010`
is returned with message "Internal Prism table 'prism_alerts' is write-protected" and
no DML node is produced.

## Test Names

- Integration: `test_ac3_internal_table_write_protected`
- Integration (gap-fill): `test_BC_2_11_004_internal_table_delete_prism_cases`
- Integration (gap-fill): `test_BC_2_11_004_internal_table_insert_prism_rules`
- Integration (gap-fill): `test_BC_2_11_004_internal_table_update_prism_schedules`
- Integration (gap-fill): `test_BC_2_11_004_internal_table_delete_prism_audit`
- Integration (gap-fill): `test_BC_2_11_004_internal_table_delete_prism_aliases`
- Integration (gap-fill): `test_BC_2_11_004_internal_table_unknown_prism_prefix`
- Integration (gap-fill): `test_BC_2_11_004_table_named_prism_no_underscore_is_allowed`
- Unit (pub-crate): `test_BC_2_11_004_is_internal_prism_table_prism_alerts`
- Unit (pub-crate): `test_BC_2_11_004_parse_sql_dml_update_prism_table_010`

## Evidence

### Integration test — primary assertion

```
test test_ac3_internal_table_write_protected ... ok
```

Query: `UPDATE prism_alerts SET status = 'resolved'`

Assertions:
- `result.is_err()`
- `result.unwrap_err()[0].message.contains("E-QUERY-010")`
- `result.unwrap_err()[0].message.contains("prism_alerts")`

### Coverage: all prism_* table patterns rejected

```
test test_BC_2_11_004_internal_table_delete_prism_cases ... ok
test test_BC_2_11_004_internal_table_insert_prism_rules ... ok
test test_BC_2_11_004_internal_table_update_prism_schedules ... ok
test test_BC_2_11_004_internal_table_delete_prism_audit ... ok
test test_BC_2_11_004_internal_table_delete_prism_aliases ... ok
test test_BC_2_11_004_internal_table_unknown_prism_prefix ... ok
test test_BC_2_11_004_table_named_prism_no_underscore_is_allowed ... ok
```

The last test confirms `DELETE FROM prism WHERE id = '1'` (no underscore suffix) does NOT
trigger E-QUERY-010 — the guard is prefix-based (`prism_`), not name-based (`prism`).

### Error message format (from error.rs constructor)

```
E-QUERY-010: Internal Prism table 'prism_alerts' is write-protected;
use the dedicated MCP tool for this operation
```

### Guard implementation (sql_parser.rs)

```rust
pub(crate) fn is_internal_prism_table(table_name: &str) -> bool {
    table_name.starts_with("prism_")
}
```

Check fires at parse time before any DmlNode is produced (architecture compliance rule:
"E-QUERY-010 MUST be emitted at parse time for SQL DML targeting prism_* tables").

## Result

PASS — E-QUERY-010 returned at parse time for all prism_* table write attempts.
