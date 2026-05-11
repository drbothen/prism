# AC-4 Evidence: AdapterRegistry keyed by (OrgId, SensorId); get accepts &SensorId

## AC Text (verbatim)

> `AdapterRegistry` is keyed by `(OrgId, SensorId)`. The `get()` method accepts
> `&SensorId` (not `SensorType`). `get_all_for_sensor()` also accepts `&SensorId`.

## Evidence Type

Static code inspection of `/Users/jmagady/Dev/prism/.worktrees/S-PLUGIN-PREREQ-A/crates/prism-sensors/src/registry.rs`

## HashMap Declaration (`registry.rs:40`)

```rust
pub struct AdapterRegistry {
    /// Internal store keyed by `(OrgId, SensorId)` composite.
    adapters: HashMap<(OrgId, SensorId), Arc<dyn SensorAdapter>>,
}
```

Key type is `(OrgId, SensorId)` — confirmed composite key.

## `register()` Signature (`registry.rs:63`)

```rust
pub fn register(&mut self, org_id: OrgId, adapter: Arc<dyn SensorAdapter>) {
    let sensor_id = adapter.sensor_type();
    self.adapters.insert((org_id, sensor_id), adapter);
}
```

Inserts under the composite `(org_id, sensor_id)` key obtained from
`adapter.sensor_type()` (which returns `SensorId`).

## `get()` Signature (`registry.rs:81`)

```rust
pub fn get(&self, org_id: OrgId, sensor_id: &SensorId) -> Option<Arc<dyn SensorAdapter>> {
    self.adapters.get(&(org_id, sensor_id.clone())).cloned()
}
```

Accepts `&SensorId` — no `SensorType` parameter anywhere in the signature.

## `get_all_for_sensor()` Signature (`registry.rs:96`)

```rust
pub fn get_all_for_sensor(&self, sensor_id: &SensorId) -> Vec<(OrgId, Arc<dyn SensorAdapter>)> {
    self.adapters
        .iter()
        .filter(|((_, sid), _)| sid == sensor_id)
        .map(|((org_id, _), adapter)| (*org_id, Arc::clone(adapter)))
        .collect()
}
```

Also accepts `&SensorId`. Open filtering — any `SensorId` value works.

## Verdict: SATISFIED

`AdapterRegistry` uses a `HashMap<(OrgId, SensorId), Arc<dyn SensorAdapter>>` key.
Both `get()` and `get_all_for_sensor()` accept `&SensorId`. No `SensorType`
appears anywhere in `registry.rs`. The BC-3.2.001 multi-tenant isolation invariant
is preserved via the composite key.
