---
document_type: behavioral-contract
level: L3
bc_id: "BC-2.06.014"
version: "1.0"
status: draft
lifecycle_status: draft
producer: product-owner
timestamp: 2026-05-23T00:00:00Z
phase: 1a
origin: greenfield
subsystem: "SS-06"
capability: "CAP-009"
introduced: "2026-05-23"
modified: "2026-05-23"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
anchored_stories: [S-CONFIG-MULTI-TENANT-OVERRIDE-001]
verifying_vps: []
crates: [prism-spec-engine, prism-sensors]
inputs:
  - ".factory/specs/architecture/decisions/ADR-029-multi-tenant-sensor-endpoint-overrides.md"
  - ".factory/research/multi-tenant-sensor-endpoint-overrides-2026-05-23.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: ""
traces_to:
  - "CAP-009"
  - "ADR-029"
extracted_from: null
---

# BC-2.06.014: Instance Identity Resolution at Fanout — (org_id, sensor_id) Tuple Resolves to ResolvedSensorSpec

## Description

When the query engine dispatches a fan-out target for a query scoped to an `(org_id, sensor_id)`
pair, the fanout engine resolves the effective sensor configuration via a two-step lookup:
(1) `OrgRegistry.slug_for(org_id)` → `OrgSlug`, then (2) per-org resolved spec lookup at key
`(org_slug, sensor_id)` in the boot-time-constructed `ResolvedSensorSpec` map. If a
`ResolvedSensorSpec` exists for that pair, the fanout target uses the overlay-merged spec
(with per-org `base_url` and any scalar overrides). If no overlay was declared for the org
(SaaS sensor, or single-tenant deployment), the fanout target uses the TYPE spec as-is.
The `CredentialResolver` is not affected — credential lookup continues by `(org_id, sensor_id)`
independently of endpoint resolution. The `FanOutTarget` struct carries the `ResolvedSensorSpec`
reference so that the sensor adapter's HTTP client uses the correct `base_url` for each org.

## Preconditions

- Boot steps 3 and 4 have completed: `OrgRegistry` is populated; `ResolvedSensorSpec` map
  is indexed by `(org_slug, sensor_id)` in memory.
- A query has been dispatched to the fanout engine with a `FanOutTarget` that carries
  `org_id` and `sensor_id`.
- `OrgRegistry` can resolve `org_id` → `org_slug` (a lookup failure here means the org
  is not registered, which is a pre-boot-validation error caught by BC-2.06.015).

## Postconditions

### Case A: Overlay exists for `(org_slug, sensor_id)`

- `OrgRegistry.slug_for(org_id)` resolves to `OrgSlug` in O(1).
- `ResolvedSensorSpec` map lookup at `(org_slug, sensor_id)` returns the overlay-merged
  spec.
- The `FanOutTarget`'s effective `SensorSpec` has:
  - `base_url` from the overlay (per-org endpoint).
  - `rate_limit_hints` from the overlay if overridden, otherwise from the TYPE spec.
  - `[[tables]]` schema, `auth_type`, `sensor_id`, and `version` unchanged from TYPE spec.
- The sensor adapter's HTTP client uses the overlay `base_url` for this dispatch.
- Instance identity is logged as `instance_id = "{sensor_id}@{org_slug}"` in fanout span.

### Case B: No overlay exists for `(org_slug, sensor_id)` (SaaS sensor or single-tenant)

- `OrgRegistry.slug_for(org_id)` resolves to `OrgSlug` in O(1).
- `ResolvedSensorSpec` map lookup at `(org_slug, sensor_id)` returns `None`.
- The `FanOutTarget`'s effective `SensorSpec` is the TYPE spec as-is.
- `base_url` used is the TYPE spec default (e.g., `https://api.crowdstrike.com`).
- Instance identity is logged as `instance_id = "{sensor_id}"` (bare; no org suffix for
  single-instance SaaS sensors).

### Common postconditions (both cases)

- Credential lookup is independent: `CredentialResolver` continues to use `(org_id, sensor_id)`
  as its lookup key. The resolved `base_url` is NOT stored in the credential record.
- The fanout resolution is O(1) with no filesystem I/O on the hot path (map is in-memory,
  populated at boot).
- Fan-out results carry the org-scoped instance identity in their provenance metadata so that
  partial failure reporting (BC-2.01.010) attributes errors to the correct `instance_id`.

## Invariants

- INV-FANOUT-001: Credential resolution and endpoint resolution are independent. The credential
  store is keyed by `(org_id, sensor_id)`; the endpoint map is keyed by `(org_slug, sensor_id)`.
  No code path conflates credential identity with endpoint identity.
- INV-FANOUT-002: Endpoint resolution is O(1) on the hot path. No filesystem I/O occurs
  during fanout dispatch. The `ResolvedSensorSpec` map was populated at boot step 4.
- INV-FANOUT-003: The `[[tables]]` schema used at query time is always from the TYPE spec.
  The DataFusion query plan is derived from the TYPE-level schema; overlay scalars affect
  only the HTTP dispatch, not the schema.
- INV-FANOUT-004: Multiple concurrent fanout targets for different orgs with the same
  sensor type are independent. Resolving `(acme, armis)` does not affect `(contoso, armis)`.
- INV-FANOUT-005: `OrgSlug` resolution uses `OrgRegistry` (boot-time populated, read-only
  on hot path). No boot-bypass lookup is possible.

## Error Conditions

| Error | Condition | Behavior |
|-------|-----------|----------|
| `OrgRegistry` lookup failure for `org_id` | `org_id` not found in registry | Programming error (org was not registered at boot, which should have been caught by BC-2.06.015); returns `PrismError::Internal` at fanout time; partial failure in BC-2.01.010 |
| `base_url` from overlay is not a valid URL | Malformed URL in overlay | This is caught at boot validation time (BC-2.06.012); cannot reach fanout in a valid boot |

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-014-001 | Single-tenant deployment: no `customers/` overlays | All fanout targets use TYPE spec `base_url`; no `ResolvedSensorSpec` entries; behavior identical to pre-ADR-029 |
| EC-014-002 | Query targets `crowdstrike` (SaaS) for org `acme` | No overlay exists; TYPE spec `base_url = "https://api.crowdstrike.com"` used; instance logged as `crowdstrike` |
| EC-014-003 | Query targets `armis` for org `acme` with overlay | Overlay `base_url` used; instance logged as `armis@acme` |
| EC-014-004 | Cross-org fan-out (`client_id: null`): query spans `acme` and `contoso` | Two `FanOutTarget`s dispatched in parallel; each resolves its own `ResolvedSensorSpec`; `acme` uses `armis.acme-corp.io`; `contoso` uses `armis.contoso.com`; results merged by query engine |
| EC-014-005 | Config reload adds a new overlay for `globex/armis` | After `reload_config`, `ResolvedSensorSpec` map is rebuilt; subsequent fanouts for `(globex, armis)` use the new overlay `base_url` |
| EC-014-006 | Config reload removes overlay for `acme/armis` | After `reload_config`, `(acme, armis)` fallback to TYPE spec `base_url` |
| EC-014-007 | In-flight query during config reload | In-flight query uses the `ConfigSnapshot` captured at query start (per DI-039 snapshot semantics); endpoint resolution uses the snapshot's `ResolvedSensorSpec` map; reload does not affect in-flight queries |

## Canonical Test Vectors

| Scenario | Setup | Expected Dispatch |
|----------|-------|-------------------|
| On-prem sensor with overlay | `acme/armis.sensor.toml` with `base_url="https://armis.acme.io"` | HTTP dispatched to `https://armis.acme.io` |
| SaaS sensor, no overlay | No `acme/crowdstrike.sensor.toml` | HTTP dispatched to TYPE spec `base_url` (`https://api.crowdstrike.com`) |
| Two-org fanout | `acme/armis.sensor.toml` + `contoso/armis.sensor.toml` | Two dispatches: `armis.acme-corp.io` and `armis.contoso.com` in parallel |
| Single-tenant: no customers dir | `customers/` absent | All sensors use TYPE spec `base_url`; backwards-compatible |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| (none yet) | Integration test: `test_BC_2_06_014_overlay_base_url_used_at_dispatch` — mock HTTP capture verifies the correct per-org `base_url` is used for each fanout target |

## Related BCs

- BC-2.06.012 — Per-Tenant Overlay Loading and Merge Semantics (produces the `ResolvedSensorSpec` map consumed here)
- BC-2.06.015 — OrgRegistry Cross-Validation at Boot (ensures all org_ids used at fanout are registered)
- BC-2.01.010 — Partial Failure Handling (fanout errors are attributed per instance_id)
- BC-2.03.006 — Credential Resolution at Sensor Query Time (credential lookup is independent of this BC)
- BC-2.21.001 — OrgRegistry Initialization (OrgRegistry.slug_for() is the lookup called here)

## Architecture Anchors

- ADR-029 §Instance Identity Convention: `{sensor_type}@{org_slug}` pattern
- ADR-029 §At query time: "check for `customers/<org_slug>/<sensor_id>.sensor.toml`; if present, merge scalars from overlay onto TYPE spec"
- ADR-029 §Consequences: "FanOutTarget already carries `org_id`; fanout engine resolves `(org_id, sensor_id)` → `ResolvedSensorSpec` at dispatch time"
- ADR-022 §D: concurrency permit model (fanout uses existing 8-permit pool; resolution is O(1) and does not consume a permit)

## Story Anchor

S-CONFIG-MULTI-TENANT-OVERRIDE-001 (to-be-created)

## VP Anchors

(None yet — VP to be authored by test-writer alongside S-CONFIG-MULTI-TENANT-OVERRIDE-001)

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-009 |
| Capability Anchor Justification | CAP-009 ("Client Configuration") per capabilities.md §CAP-009 — this BC specifies how per-client (per-org) sensor configuration is resolved at query time: "Load and validate per-client sensor mappings" — the resolved sensor spec is the per-client mapping that routes an org's query to the correct sensor endpoint. |
| L2 Invariants | DI-008 (client data separation — each org_id resolves to its own `ResolvedSensorSpec`; no cross-org endpoint leakage) |
| L2 Entities | ResolvedSensorSpec, FanOutTarget, OrgRegistry, OrgSlug |
| Priority | P0 |
| ADR | ADR-029 (Multi-Tenant Sensor Endpoint Overrides) |
| Source-of-Truth Precedence | ADR-029 §At query time resolution algorithm is authoritative for the two-step lookup. |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | D-803 burst-3 | 2026-05-23 | product-owner | Initial draft per ADR-029 Burst 3 handoff |
