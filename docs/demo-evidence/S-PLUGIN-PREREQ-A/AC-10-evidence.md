# AC-10 Evidence: prism-sensors integration test exercises AdapterRegistry insert+lookup with SensorId

## AC Text (verbatim)

> `prism-sensors` integration test `test_BC_2_01_013_004_adapter_registry_sensorid_insert_lookup`
> registers a mock adapter under `SensorId::from("crowdstrike")`, looks it up,
> asserts `Some`. Verifies cross-sensor isolation: lookup for an unregistered sensor returns `None`.

## Evidence Type

Test execution output + source excerpt.

## Test Execution

```
$ cargo nextest run --color=never -p prism-sensors \
    -E 'test(test_BC_2_01_013_004_adapter_registry_sensorid_insert_lookup)'

    Finished `test` profile [unoptimized + debuginfo] target(s) in 18.58s
────────────
 Nextest run ID 3d5f29df-1e3d-43a6-99d8-8dd34c77e1d8 with nextest profile: default
    Starting 1 test across 12 binaries (266 tests skipped)
        PASS [   0.027s] (1/1) prism-sensors tests::bc_2_01_013_sensorid::test_BC_2_01_013_004_adapter_registry_sensorid_insert_lookup
────────────
     Summary [   0.028s] 1 test run: 1 passed, 266 skipped
```

Test PASSES.

## Test Source

File: `/Users/jmagady/Dev/prism/.worktrees/S-PLUGIN-PREREQ-A/crates/prism-sensors/src/tests/bc_2_01_013_sensorid.rs:74`

```rust
/// BC-2.01.013 AC-4 + AC-10: AdapterRegistry insert + lookup with SensorId key.
///
/// AC-4: HashMap keyed by (OrgId, SensorId)
/// AC-10: register a mock adapter under SensorId::from("crowdstrike"), look it up,
///        assert Some. Verify cross-sensor isolation: lookup for "cyberint" returns None.
#[test]
fn test_BC_2_01_013_004_adapter_registry_sensorid_insert_lookup() {
    let org_id = OrgId::new();
    let mut registry = AdapterRegistry::new();

    // Register a stub adapter using the new SensorId-keyed API.
    registry.register(
        org_id,
        stub_adapter(SensorId::from("crowdstrike"), "crowdstrike"),
    );

    // Lookup via SensorId — now the native API.
    let result = registry.get(org_id, &SensorId::from("crowdstrike"));

    assert!(
        result.is_some(),
        "registry.get must return Some for a registered crowdstrike adapter (AC-4, AC-10)"
    );

    // Cross-sensor isolation: looking up "cyberint" must return None when only
    // "crowdstrike" was registered (BC-3.2.001 invariant 1).
    let no_result = registry.get(org_id, &SensorId::from("cyberint"));
    assert!(
        no_result.is_none(),
        "registry.get must return None for sensor not registered (cross-sensor isolation, AC-10)"
    );
}
```

## Cross-Sensor Isolation

The test explicitly verifies that `registry.get(org_id, &SensorId::from("cyberint"))` returns
`None` when only `"crowdstrike"` was registered. This validates BC-3.2.001 invariant 1:
`get(org_a, &SensorId("crowdstrike"))` never returns an adapter registered for a
different sensor under the same org.

## Verdict: SATISFIED

`test_BC_2_01_013_004_adapter_registry_sensorid_insert_lookup` passes in prism-sensors.
The test registers a mock adapter under `SensorId::from("crowdstrike")`, successfully
looks it up (asserts `Some`), and confirms cross-sensor isolation (lookup for
`"cyberint"` returns `None`). Located at
`crates/prism-sensors/src/tests/bc_2_01_013_sensorid.rs:74`.
