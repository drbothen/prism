# AC-9 Evidence: prism-core unit tests cover (a) equality/hash/Display roundtrip; (b) Borrow<str> lookup

## AC Text (verbatim)

> `prism-core` contains unit tests covering:
> (a) `From<&str>`, `Display` roundtrip; `PartialEq`, `Hash`, `Eq` content-based invariant.
> (b) `Borrow<str>` enabling `HashMap<SensorId, V>::get(&str)` without constructing a `SensorId`.

## Evidence Type

Test execution output + source excerpts for each of the 3 Red Gate unit tests.

## Test Execution

```
$ cargo nextest run --color=never -p prism-core \
    -E 'test(test_BC_2_01_013_001_sensorid_from_str_roundtrip) or
        test(test_BC_2_01_013_003_sensorid_hash_eq_invariant) or
        test(test_BC_2_01_013_004_sensor_id_borrow_str_lookup)'

    Finished `test` profile [unoptimized + debuginfo] target(s) in 14.98s
────────────
 Nextest run ID d0c80053-9189-42c8-8746-de530058d2c3 with nextest profile: default
    Starting 3 tests across 16 binaries (232 tests skipped)
        PASS [   0.011s] (1/3) prism-core sensor_id::tests::test_BC_2_01_013_004_sensor_id_borrow_str_lookup
        PASS [   0.011s] (2/3) prism-core sensor_id::tests::test_BC_2_01_013_003_sensorid_hash_eq_invariant
        PASS [   0.011s] (3/3) prism-core sensor_id::tests::test_BC_2_01_013_001_sensorid_from_str_roundtrip
────────────
     Summary [   0.015s] 3 tests run: 3 passed, 232 skipped
```

All 3 Red Gate tests PASS.

## Test Source Excerpts

### test_BC_2_01_013_001_sensorid_from_str_roundtrip (`sensor_id.rs:327`)

AC-9(a): From<&str> + Display roundtrip; HashSet containment (PartialEq + Hash).

```rust
#[test]
fn test_BC_2_01_013_001_sensorid_from_str_roundtrip() {
    let id = SensorId::from("crowdstrike");
    let displayed = format!("{id}");
    assert_eq!(
        displayed, "crowdstrike",
        "Display must reproduce the original string"
    );

    let mut set = HashSet::new();
    set.insert(SensorId::from("crowdstrike"));
    assert!(
        set.contains(&SensorId::from("crowdstrike")),
        "SensorId equality and hash must be content-based"
    );
}
```

### test_BC_2_01_013_003_sensorid_hash_eq_invariant (`sensor_id.rs:372`)

AC-9(a): Two independently-constructed SensorIds with the same string value are equal
and hash to the same bucket (content-based, not pointer-based).

```rust
#[test]
fn test_BC_2_01_013_003_sensorid_hash_eq_invariant() {
    let a = SensorId::from("crowdstrike");
    let b = SensorId::from("crowdstrike");

    assert_eq!(a, b, "two SensorIds from the same string must be equal");

    let mut map: HashMap<SensorId, u32> = HashMap::new();
    map.insert(a, 42);
    let retrieved = map.get(&b).copied();
    assert_eq!(
        retrieved,
        Some(42),
        "HashMap lookup via separately-constructed SensorId must find the inserted value"
    );
}
```

### test_BC_2_01_013_004_sensor_id_borrow_str_lookup (`sensor_id.rs:396`)

AC-9(b): `Borrow<str>` enables `HashMap<SensorId, V>::get("crowdstrike")` without
constructing a `SensorId`.

```rust
#[test]
fn test_BC_2_01_013_004_sensor_id_borrow_str_lookup() {
    let mut map: HashMap<SensorId, u32> = HashMap::new();
    map.insert(SensorId::from("armis"), 42);

    assert_eq!(
        map.get("armis").copied(),
        Some(42),
        "Borrow<str> impl must allow HashMap lookup via &str without constructing SensorId"
    );
    assert_eq!(
        map.get("crowdstrike"),
        None,
        "non-inserted key returns None"
    );
}
```

## Verdict: SATISFIED

All 3 Red Gate unit tests pass at HEAD `8b949bba`. Tests cover:
- AC-9(a): `From<&str>` + `Display` roundtrip, `HashSet` containment, content-based
  `PartialEq` + `Hash` invariant via separate-construction HashMap roundtrip.
- AC-9(b): `Borrow<str>` impl enabling `HashMap<SensorId, u32>::get(&str)` without
  constructing a `SensorId`. Non-inserted key correctly returns `None`.
