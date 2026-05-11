# AC-3 Evidence: SensorAdapter::sensor_type trait returns SensorId

## AC Text (verbatim)

> The `SensorAdapter` trait method `sensor_type(&self) -> SensorId` is defined in
> `prism-sensors`. All 4 concrete adapter impls return `SensorId::from("<name>")`.

## Evidence Type

Static code inspection of trait definition and 4 concrete adapter implementations.

## Trait Definition

File: `/Users/jmagady/Dev/prism/.worktrees/S-PLUGIN-PREREQ-A/crates/prism-sensors/src/adapter.rs:311`

```rust
fn sensor_type(&self) -> SensorId;
```

Full doc-comment on lines 299-311:

> Returns the sensor id this adapter handles.
> Used by `AdapterRegistry` to key the adapter lookup table.
> Returns an open `SensorId` string key (ADR-023 §C1, BC-2.01.013).
> Method name preserved post-rename (S-PLUGIN-PREREQ-A story task 5). The
> return type is `SensorId` (open newtype, ADR-023 Rule 1) replacing the historical
> closed `SensorType` enum.

## Concrete Adapter Implementations

### CrowdStrike (`crates/prism-sensors/src/auth/crowdstrike.rs:377`)

```rust
fn sensor_type(&self) -> SensorId {
    SensorId::from("crowdstrike")
}
```

### Cyberint (`crates/prism-sensors/src/auth/cyberint.rs:254`)

```rust
fn sensor_type(&self) -> SensorId {
    SensorId::from("cyberint")
}
```

### Claroty (`crates/prism-sensors/src/auth/claroty.rs:247`)

```rust
fn sensor_type(&self) -> SensorId {
    SensorId::from("claroty")
}
```

### Armis (`crates/prism-sensors/src/auth/armis.rs:558`)

```rust
fn sensor_type(&self) -> SensorId {
    SensorId::from("armis")
}
```

All 4 concrete adapters return `SensorId::from("<lowercase-sensor-name>")` as
specified by the method's doc-comment convention.

## Verdict: SATISFIED

The `SensorAdapter` trait declares `fn sensor_type(&self) -> SensorId` at
`adapter.rs:311`. All 4 concrete adapter impls (`crowdstrike`, `cyberint`,
`claroty`, `armis`) return `SensorId::from("<name>")`. No adapter returns
a `SensorType` enum variant.
