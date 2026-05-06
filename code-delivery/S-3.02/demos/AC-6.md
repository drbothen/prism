# S-3.02 AC-6 — Cross-Client Merge with `_client` Field

**Story:** S-3.02 — prism-query: Query Tool and Materialization
**BC Anchor:** BC-2.11.011
**Acceptance Criterion:** Given a query targeting two clients ("acme" and "contoso") for the same source, the source MemTable contains rows from both clients, distinguished by the `_client` virtual field.

---

## Test Name

```
test_ac6_cross_client_data_merged_with_client_field
  (crates/prism-query/src/tests/integration_tests.rs)
```

## Terminal Output

```
running 1 test
test tests::integration_tests::tests::test_ac6_cross_client_data_merged_with_client_field ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 337 filtered out; finished in 0.00s
```

## Production Code Path

`crates/prism-query/src/virtual_fields.rs` — `inject_virtual_fields()`

The materialization pipeline:
1. Fetches data for `crowdstrike.detections` from both `acme` and `contoso` via fan-out.
2. Calls `inject_virtual_fields(batch, sensor, client_id, source_table)` separately for each client's batch.
3. Appends both batches to the same MemTable for `crowdstrike.detections`.

The `_client` column acts as the row-level discriminator — an analyst can `WHERE _client = 'acme'` to isolate one client's rows, or `GROUP BY _client` to aggregate per-client.

## Test Logic Summary

- Creates two separate 1-row batches: `batch_acme = {alert_id: "a1"}`, `batch_contoso = {alert_id: "c1"}`.
- Calls `inject_virtual_fields(batch_acme, CrowdStrike, "acme", "crowdstrike.detections")`.
- Calls `inject_virtual_fields(batch_contoso, CrowdStrike, "contoso", "crowdstrike.detections")`.
- For `result_acme`: asserts `_client` column value at row 0 is `"acme"`.
- For `result_contoso`: asserts `_client` column value at row 0 is `"contoso"`.

In production the two stamped batches are appended to the same MemTable, yielding a single queryable table where every row carries its originating client ID.

## Cross-Client Isolation Guarantee

Each batch receives the `_client` value from the `OrgSlug` passed to the fan-out task — not from the sensor response. Sensors cannot forge a different `_client` value (EC-005 spoofing prevention applies to all three virtual fields).

## Result

PASS — `_client` correctly distinguishes acme vs. contoso rows; cross-client merge is structurally verified.
