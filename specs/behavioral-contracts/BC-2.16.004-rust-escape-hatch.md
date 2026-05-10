---
document_type: behavioral-contract
level: L3
version: "1.3"
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
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "76729b7"
traces_to:
  - "CAP-029"
extracted_from: ".factory/specs/prd.md"
scheduled_amendment_in: ADR-023
amendment_lifecycle: pending
---

# BC-2.16.004: Rust Escape Hatch for Custom Adapters — Trait-Based Override When Config Is Insufficient

## Description

Approximately 20% of REST API sensors require behavior that cannot be expressed in TOML
spec files: exotic auth flows, binary protocols, complex response transformations, or
stateful pagination. For these cases, a `CustomAdapter` trait allows Rust code to
override any part of the spec-driven pipeline while leaving the remaining spec-driven
behavior intact. All four initial sensors (CrowdStrike, Cyberint, Claroty, Armis) ship
as pure TOML specs, validating that the config-driven path covers the common case.

Custom adapters are registered in the startup sequence before table registration and
associated with a `sensor_id` that matches a spec file. A spec without a matching
adapter uses the fully config-driven pipeline. Panics in custom adapter code are caught
via `catch_unwind` and converted to structured errors.

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
- All sensors — including the four initial sensors (CrowdStrike, Cyberint, Claroty, Armis) — ship as TOML spec files and use the config-driven pipeline. The escape hatch exists for the ~20% of sensors requiring exotic behavior.
- Approximately 80% of REST API sensors are expected to be fully config-driven; approximately 20% will use the escape hatch for auth or response transformation.

## Registration
- Custom adapters are registered in the `main.rs` startup sequence after config loading but before table registration
- Each custom adapter is associated with a `sensor_id` that matches a spec file's `sensor_id`
- A custom adapter without a matching spec file is a startup warning (the adapter is registered but has no tables)
- A spec file without a matching custom adapter uses the fully config-driven pipeline (no override)

## Invariants
- A spec file without a matching custom adapter always uses the fully config-driven pipeline
- Custom adapter panics are caught and converted to structured errors (never crash the process)
- The four initial sensors (CrowdStrike, Cyberint, Claroty, Armis) do NOT use the escape hatch — they are pure TOML specs

## Error Conditions
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-SPEC-008` | Custom adapter panic | Caught via `std::panic::catch_unwind`; converted to `E-SPEC-008` structured error |
| (warning) | Custom adapter registered without matching spec file | Startup warning logged; adapter registered but has no effect |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| Adapter with no spec | registered adapter has no matching sensor_id in specs | Warning logged; adapter ignored |
| Spec with no adapter | spec has no registered custom adapter | Fully config-driven pipeline used; no error |
| override_auth returns None | custom adapter declines auth override | Spec-declared auth_type used |
| override_fetch returns None | custom adapter declines fetch override | Spec-driven HTTP call proceeds normally |
| Custom adapter panic | override_fetch panics | Caught; `E-SPEC-008` returned to caller; other queries unaffected |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for full canonical vectors.

| Scenario | Input | Expected Output |
|----------|-------|-----------------|
| Happy path — no adapter | spec-only sensor queried | Fully config-driven; no adapter involvement |
| Adapter overrides auth | `override_auth` returns custom SensorAuth | Custom auth used; spec pipeline continues |
| Adapter overrides fetch | `override_fetch` returns custom RecordBatch | Custom fetch result used; OCSF mapping continues |
| Adapter panic | `override_fetch` panics | `E-SPEC-008`; process not crashed |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| (none) | Panic isolation via catch_unwind is a construction-time guarantee; integration test with panicking fixture is correct vehicle; initial-sensor TOML-only property is a code review invariant; no pure-function formal VP. |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-029 |
| L2 Invariants | -- |
| Related BCs | BC-2.01.013 (DataSource trait), BC-2.16.001 (spec loading) |
| Priority | P1 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-74-fix | 2026-04-20 | product-owner | Resolved (placeholder) row in ## Verification Properties per pass-74 VP-TBD decision matrix extension. |
| 1.2 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description; added ## Invariants; added ## Error Conditions (from inline Error Handling); converted ## Traces → ## Traceability table; added ## Canonical Test Vectors; added ## Verification Properties; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial draft (used ## Traces section) |
