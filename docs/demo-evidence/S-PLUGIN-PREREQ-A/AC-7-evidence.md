# AC-7 Evidence: No CustomAdapter trait calls depend on closed-enum dispatch

## AC Text (verbatim)

> The `CustomAdapter` trait in `prism-sensors/src/adapter.rs` is preserved (PREREQ-E
> retires it). `CustomAdapterRegistry::get` in `prism-spec-engine` accepts `&str`,
> not `SensorType`. No CustomAdapter call site passes a `SensorType` value.

## Evidence Type

Static code inspection — CustomAdapter trait location, CustomAdapterRegistry::get
signature, and absence of SensorType in CustomAdapter call sites.

## CustomAdapter Trait in prism-spec-engine

Note: The `CustomAdapter` trait and `CustomAdapterRegistry` live in
`prism-spec-engine/src/custom_adapter.rs` (not prism-sensors/src/adapter.rs).
The `prism-sensors/src/adapter.rs` file defines the `SensorAdapter` trait (AC-3).
These are distinct traits serving different purposes.

### `CustomAdapter` trait declaration (`custom_adapter.rs:25`)

```rust
pub trait CustomAdapter: Send + Sync {
    /// The sensor_id this adapter handles. Must match a loaded spec file's sensor_id.
    fn sensor_id(&self) -> &str;

    fn override_auth(&self, client_id: &OrgSlug) -> Option<Box<dyn SensorAuth>>;
    fn override_fetch(
        &self, table: &str, step: &FetchStep, context: &FetchContext,
    ) -> Option<Vec<serde_json::Value>>;
    fn transform_response(&self, table: &str, raw: &serde_json::Value) -> Option<serde_json::Value>;
}
```

`fn sensor_id(&self) -> &str` — returns `&str`, not `SensorType`.

### `CustomAdapterRegistry::get` signature (`custom_adapter.rs:102`)

```rust
pub fn get(&self, sensor_id: &str) -> Option<&dyn CustomAdapter> {
    self.adapters
        .iter()
        .find(|a| a.sensor_id() == sensor_id)
        .map(|a| a.as_ref())
}
```

Accepts `&str` — not `&SensorType` or `SensorType`. Open string dispatch.

### `CustomAdapterRegistry::register` — duplicate check (`custom_adapter.rs:80`)

```rust
pub fn register(&mut self, adapter: Box<dyn CustomAdapter>) -> Result<(), PrismError> {
    let id = adapter.sensor_id().to_string();
    if self.adapters.iter().any(|a| a.sensor_id() == id) {
        return Err(...); // EC-003: adapter name must be unique
    }
    self.adapters.push(adapter);
    Ok(())
}
```

String-keyed — no `SensorType` involvement.

## SensorType Absence in custom_adapter.rs

```
$ grep -n "SensorType" crates/prism-spec-engine/src/custom_adapter.rs
(no output — exit code 1)
```

Zero occurrences of `SensorType` in the file.

## CustomAdapter Trait Preservation

The trait body is intact (not removed). PREREQ-E is the story that retires
`CustomAdapter` — it has not been executed. The trait remains fully functional
as specified by the story boundary constraint.

## Verdict: SATISFIED

`CustomAdapterRegistry::get` in `prism-spec-engine/src/custom_adapter.rs:102`
accepts `&str` — not `SensorType`. Zero `SensorType` references exist in
`custom_adapter.rs`. The `CustomAdapter` trait is preserved (not retired; PREREQ-E
owns retirement). No CustomAdapter call site depends on closed-enum dispatch.
