---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "SS-16"
capability: "CAP-029"
lifecycle_status: active
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.16.004: Rust Escape Hatch for Custom Adapters — Trait-Based Override When Config Is Insufficient

## Preconditions
- A sensor requires behavior that cannot be expressed in TOML spec files, such as:
  - Exotic authentication flows (multi-step OAuth2 with PKCE, SAML-based auth, mutual TLS)
  - Binary protocol handling (protobuf/gRPC responses, binary file downloads)
  - Complex response transformations (XML parsing, nested deduplication, polymorphic ID normalization)
  - Stateful pagination (pagination that requires maintaining state across requests beyond simple cursor tokens)

## Postconditions
- A `CustomAdapter` trait is defined in `prism-spec-engine` that allows Rust code to override any part of the spec-driven pipeline:
  ```
  trait CustomAdapter: Send + Sync {
      fn sensor_id(&self) -> &str;
      fn override_auth(&self, client_id: &TenantId) -> Option<Box<dyn SensorAuth>>;
      fn override_fetch(&self, table: &str, step: &FetchStep, context: &FetchContext) -> Option<Pin<Box<dyn Future<Output = Result<Vec<RecordBatch>>>>>>;
      fn transform_response(&self, table: &str, raw: &serde_json::Value) -> Option<serde_json::Value>;
  }
  ```
- Custom adapters are registered at startup via a `CustomAdapterRegistry`
- When a spec-driven table's fetch pipeline encounters a step, it checks the registry for an override:
  1. If `override_auth` returns `Some(auth)`, the custom auth replaces the spec-declared `auth_type` for that sensor
  2. If `override_fetch` returns `Some(future)`, the custom fetch replaces the spec-driven HTTP call for that specific step
  3. If `transform_response` returns `Some(value)`, the custom transform is applied to the raw response before the spec's `response_path` extraction
- All other spec-driven behavior (column mapping, OCSF normalization, pagination, rate limiting) continues to apply around the overridden component
- All sensors — including the four initial sensors (CrowdStrike, Cyberint, Claroty, Armis) — ship as TOML spec files and use the config-driven pipeline. The escape hatch exists for the ~20% of sensors requiring exotic behavior (binary protocols, complex streaming, multi-step OAuth with PKCE), not for the initial four
- Approximately 80% of REST API sensors are expected to be fully config-driven; approximately 20% will use the escape hatch for auth or response transformation. The initial four sensors demonstrate spec-system sufficiency

## Registration
- Custom adapters are registered in the `main.rs` startup sequence after config loading but before table registration
- Each custom adapter is associated with a `sensor_id` that matches a spec file's `sensor_id`
- A custom adapter without a matching spec file is a startup warning (the adapter is registered but has no tables)
- A spec file without a matching custom adapter uses the fully config-driven pipeline (no override)

## Error Handling
- Custom adapter panics are caught via `std::panic::catch_unwind` and converted to `E-SPEC-008` structured errors
- Custom adapter errors are reported in `sensor_errors` identically to spec-driven adapter errors

## Traces
- CAP-029 (Config-Driven Sensor Adapters)
- BC-2.01.013 (DataSource trait — the escape hatch complements, not replaces, the trait-based architecture)
