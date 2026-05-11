# AC-5 Evidence: All dispatch sites use open dispatch (no SensorType::X matches in production)

## AC Text (verbatim)

> All 7 dispatch sites previously using `match sensor_type { SensorType::X => ... }`
> have been rewritten to open `SensorId`-keyed dispatch. Zero `SensorType::` match
> arms remain in production source.

## Evidence Type

`grep` confirming zero `SensorType::` in production source, plus file:line excerpts
for each of the 7 dispatch sites.

## Grep Confirmation

```
$ grep -rn "SensorType::" crates/*/src/
(no output — exit code 1)
```

Zero `SensorType::` references anywhere in `crates/*/src/`. The Red Gate test
`test_BC_2_01_013_005_sensorid_lookup_at_virtual_fields_dispatch` passed (shown in AC-5 test).

## Per-Site Evidence

### Site 1: types.rs:79-88 — DELETED

The `SensorType` enum variant dispatch block has been deleted entirely as part of
the enum removal (AC-2). No replacement needed — open dispatch is implicit via
`SensorId` string equality.

### Site 2: virtual_fields.rs:163-166

```rust
// File: crates/prism-query/src/virtual_fields.rs:163-165
/// `SensorId` IS the string — delegates to `AsRef<str>`.
pub(crate) fn sensor_id_to_str(sensor: &SensorId) -> &str {
    sensor.as_ref()
}
```

Open dispatch: `sensor.as_ref()` returns the string directly — no enum match.

### Site 3: explain.rs:659-668

```rust
// File: crates/prism-query/src/explain.rs:659-668
fn sensor_id_from_source_ref(s: &SourceRef) -> Option<SensorId> {
    match &s.kind {
        SourceRefKind::External { sensor, .. } => {
            let lower = sensor.to_lowercase();
            SensorId::try_from_str(&lower).ok()
        }
```

Open dispatch: `try_from_str` accepts any valid lowercase string — no enum match.

### Site 4: explain.rs:1045-1053

```rust
// File: crates/prism-query/src/explain.rs:1045-1053
for src in &sensors_to_query {
    let sensor_key = src.sensor_id.to_string();
    let latency_ms = match src.sensor_id.as_ref() {
        "crowdstrike" => 250,
        "cyberint" => 400,
```

Open string match on `sensor_id.as_ref()` — not a `SensorType::` match. Unknown
sensors fall through to the default arm (open extensibility).

### Site 5: write_dispatch.rs:282-284

```rust
// File: crates/prism-query/src/write_dispatch.rs:282-284
let sensor_id = match SensorId::try_from_str(plan.sensor.as_str()) {
    Ok(id) => id,
    Err(e) => { /* E-QUERY-031 error path */ }
```

Open dispatch: `try_from_str` constructs any valid `SensorId` from user input —
no enum variant match. Dispatch then uses `adapter_registry.get(org_id, &sensor_id)`.

### Site 6: materialization.rs:779-794

```rust
// File: crates/prism-query/src/materialization.rs:779-794
fn resolve_org_id(
    client_id: &OrgSlug,
    sensor_id: SensorId,
    adapter_registry: &AdapterRegistry,
    org_registry: &Option<Arc<prism_core::OrgRegistry>>,
) -> OrgId {
    let adapters = adapter_registry.get_all_for_sensor(&sensor_id);
```

Open dispatch: `get_all_for_sensor(&sensor_id)` accepts any `SensorId` value —
no enum match.

### Site 7: invalidation.rs:57-99

```rust
// File: crates/prism-query/src/invalidation.rs:57-99
pub static WRITE_TOOL_INVALIDATION_MAP: LazyLock<Vec<WriteToolInvalidationMap>> =
    LazyLock::new(|| {
        vec![
            WriteToolInvalidationMap {
                tool_name: "crowdstrike_contain_host",
                source_ids: &["crowdstrike_hosts", "crowdstrike_detections"],
                sensor_id: SensorId::from("crowdstrike"),
            },
            // ... (4 sensors, 8 entries)
```

Open dispatch: `WriteToolInvalidationMap` carries a `SensorId` field. Lookup uses
`entry.sensor_id == *probe_sensor_id` — content-based equality, not enum variant.

## Verdict: SATISFIED

`grep -rn "SensorType::" crates/*/src/` returns zero matches. All 7 dispatch sites
have been migrated to open `SensorId`-keyed dispatch. The Red Gate test
`test_BC_2_01_013_005_sensorid_lookup_at_virtual_fields_dispatch` passed, validating
that the virtual_fields dispatch site correctly returns any `SensorId`'s string value
without a closed enum match.
