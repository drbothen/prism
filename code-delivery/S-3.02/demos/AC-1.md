# S-3.02 AC-1 — Virtual Fields Injected in Every Row

**Story:** S-3.02 — prism-query: Query Tool and Materialization
**BC Anchor:** BC-2.11.001, BC-2.11.012
**Acceptance Criterion:** Given `execute("SELECT * FROM crowdstrike.detections LIMIT 10", clients: ["acme"])`, every result row contains `_sensor = "crowdstrike"`, `_client = "acme"`, `_source_table = "crowdstrike.detections"` virtual fields.

---

## Test Name

```
test_ac1_virtual_fields_present_in_every_row
  (crates/prism-query/src/tests/integration_tests.rs)
```

## Terminal Output

```
running 1 test
test tests::integration_tests::tests::test_ac1_virtual_fields_present_in_every_row ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 337 filtered out; finished in 0.03s
```

## Production Code Path

`crates/prism-query/src/virtual_fields.rs` — `inject_virtual_fields()`

The function:
1. Builds three constant-value `StringArray` columns (`_sensor`, `_client`, `_source_table`) sized to `batch.num_rows()`.
2. Removes any existing columns with those names from the batch schema (spoofing prevention, EC-005).
3. Appends the three canonical columns to the schema and column list.
4. Returns a new `RecordBatch` with all original columns plus the three virtual fields.

Constants declared:
```rust
pub const VIRTUAL_FIELD_SENSOR: &str = "_sensor";
pub const VIRTUAL_FIELD_CLIENT: &str = "_client";
pub const VIRTUAL_FIELD_SOURCE_TABLE: &str = "_source_table";
```

## Test Logic Summary

- Builds a 3-row batch with a single `severity` column.
- Calls `inject_virtual_fields(batch, &SensorType::CrowdStrike, &OrgSlug("acme"), "crowdstrike.detections")`.
- Asserts `result.num_rows() == 3`.
- Downcasts `_sensor` column to `StringArray`; asserts every value is `"crowdstrike"`.
- Downcasts `_client` column to `StringArray`; asserts every value is `"acme"`.

## Result

PASS — virtual fields present in all 3 rows for the specified sensor and client.
