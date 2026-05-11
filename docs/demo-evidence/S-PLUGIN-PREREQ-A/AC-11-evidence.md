# AC-11 Evidence: CustomAdapter trait NOT removed; CustomAdapterRegistry uses &str or SensorId (no SensorType)

## AC Text (verbatim)

> `CustomAdapter` trait is NOT removed (PREREQ-E story owns its retirement).
> `CustomAdapterRegistry::get` in `prism-spec-engine` uses `&str` (not `SensorType`).
> The trait body in `prism-sensors/src/adapter.rs` retains the `SensorAdapter`
> declaration (the dual-trait design is preserved).

## Evidence Type

Static code inspection of `prism-spec-engine/src/custom_adapter.rs` and absence of
`SensorType` from both custom adapter files.

## CustomAdapter Trait Preserved

The `CustomAdapter` trait is declared in:
`/Users/jmagady/Dev/prism/.worktrees/S-PLUGIN-PREREQ-A/crates/prism-spec-engine/src/custom_adapter.rs:25`

```rust
pub trait CustomAdapter: Send + Sync {
    fn sensor_id(&self) -> &str;
    fn override_auth(&self, client_id: &OrgSlug) -> Option<Box<dyn SensorAuth>>;
    fn override_fetch(&self, table: &str, step: &FetchStep, context: &FetchContext) -> Option<Vec<serde_json::Value>>;
    fn transform_response(&self, table: &str, raw: &serde_json::Value) -> Option<serde_json::Value>;
}
```

Trait body is intact — 4 methods, all present. NOT removed.

## CustomAdapterRegistry::get Uses &str

```rust
// custom_adapter.rs:102
pub fn get(&self, sensor_id: &str) -> Option<&dyn CustomAdapter> {
    self.adapters
        .iter()
        .find(|a| a.sensor_id() == sensor_id)
        .map(|a| a.as_ref())
}
```

Parameter type is `&str`. No `SensorType` in the signature.

## SensorType Absent from custom_adapter.rs

```
$ grep -n "SensorType" crates/prism-spec-engine/src/custom_adapter.rs
(no output — exit code 1)
```

Zero occurrences.

## SensorAdapter Trait Also Preserved in prism-sensors

The `SensorAdapter` trait in `crates/prism-sensors/src/adapter.rs:298` is a
distinct trait from `CustomAdapter`. It is preserved and unchanged (PREREQ-E
does not retire `SensorAdapter`). Its `sensor_type()` method now returns
`SensorId` per AC-3.

## PREREQ-E Boundary Respected

`CustomAdapter` trait retirement is assigned to story `S-PLUGIN-PREREQ-E`. That
story has not been executed. This story (S-PLUGIN-PREREQ-A) only replaces
the `SensorType` enum with `SensorId` — it does not touch `CustomAdapter`'s
existence.

## Verdict: SATISFIED

`CustomAdapter` trait is present and intact at `custom_adapter.rs:25`. Its registry's
`get()` method uses `&str` (not `SensorType`). Zero `SensorType` references exist in
`custom_adapter.rs`. The story boundary is respected: only PREREQ-E retires
`CustomAdapter`. The dual-trait design (`SensorAdapter` in prism-sensors + `CustomAdapter`
in prism-spec-engine) is preserved.
