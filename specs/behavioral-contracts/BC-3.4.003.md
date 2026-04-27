---
document_type: behavioral-contract
level: L3
version: "0.4"
status: PROPOSED
producer: product-owner
timestamp: 2026-04-27T00:00:00
phase: 3.A
inputs:
  - ".factory/specs/architecture/decisions/ADR-009-multi-tenant-data-generator.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "802850d"
traces_to: ".factory/specs/architecture/decisions/ADR-009-multi-tenant-data-generator.md"
origin: greenfield
extracted_from: null
subsystem: "SS-06"
capability: "CAP-039"
lifecycle_status: active
introduced: wave-3
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
bc_id: BC-3.4.003
title: Archetype catalog enumeration — 8 archetypes with defined baselines
wave: 3
phase: 3.A
date: 2026-04-27
authors: [product-owner]
related_decisions: [D-043, D-045]
related_adrs: [ADR-009]
inherits_from: null
superseded_by: null
---

# BC-3.4.003: Archetype Catalog Enumeration — 8 Archetypes with Defined Baselines

## Description

The archetype catalog is a fixed enumeration of 8 named deployment scenarios. Each archetype defines the semantic shape of the generated data: what records are present, what anomalies exist, what failure modes are active. At `scale = 1.0`, each archetype produces a defined baseline record count per sensor type. Archetypes are declared `#[non_exhaustive]` so adding new variants in future waves is backward-compatible. This BC defines the baseline counts, distinguishing characteristics, and observable behavior for each of the 8 archetypes.

## Preconditions

1. The archetype catalog is compiled into the `prism-dtu-common` crate as a `#[non_exhaustive]` enum.
2. `scale = 1.0` is used for all baseline count assertions (unless otherwise noted).
3. `time_anchor = DateTime::UNIX_EPOCH` is used for all time-dependent assertions in test vectors.
4. The generator has been called with a valid `(org_id, sensor_type, archetype, GenOpts)`.

## Postconditions

**Archetype baseline table (at `scale = 1.0`, any sensor type unless noted):**

| Archetype | Baseline Device/Asset Count | Alert/Detection Count | High-Severity Count | Notes |
|-----------|----------------------------|-----------------------|---------------------|-------|
| `HealthyOtEnvironment` | 50 | 5 | 0 | Stable OT network; no active threats |
| `CompromisedEndpoint` | 50 | 20 | 3 | Elevated alerts; containment state changes |
| `AuthOutage` | 20 | 2 | 0 | First API call returns HTTP 401; recovers after configurable delay (default: 2nd call succeeds) |
| `LargeScale` | 10,000 | 500 | 10 | Exercises pagination and memory budget |
| `PaginationEdgeCases` | `page_size × 3` | 0 | 0 | Exact multiples of page size; single-page and empty-final-page variants |
| `SchemaDrift` | 30 | 1 | 0 | 1 record has non-conformant field shape (missing required field or wrong type) |
| `HighChurn` | 200 | 10 | 1 | Devices appear/disappear between polling cycles; tombstone records present |
| `DormantTenant` | 0 | 0 | 0 | No data; simulates recently onboarded or offline tenant |

**Per-archetype behavioral specifications:**

**`HealthyOtEnvironment`**
- 50 devices, 5 alerts, 0 high-severity alerts at scale=1.0
- All devices have `status = "online"` or `status = "active"`
- No containment flags, no anomalous tag mutations
- All timestamps fall within `[time_anchor - 7d, time_anchor]`

**`CompromisedEndpoint`**
- 50 devices, 20 alerts, 3 high-severity alerts at scale=1.0
- At least 1 device has `containment_status = "contained"` (CrowdStrike) or equivalent
- At least 1 device has anomalous lateral-movement indicators in tags (Claroty) or status (Armis)
- High-severity alerts have `severity_id >= 4` (OCSF convention)

**`AuthOutage`**
- 20 devices at scale=1.0 (represents a smaller deployment for simplicity)
- The first simulated API call for this fixture returns an HTTP 401 response record
- Subsequent simulated calls return normal 200 responses (recovery behavior)
- The recovery delay is configurable via `GenOpts::overrides` using JSON Merge Patch: `{"auth_outage": {"recovery_after_calls": N}}`; default N=1

**`LargeScale`**
- 10,000 devices, 500 alerts, 10 high-severity at scale=1.0
- Devices are spread across at least 100 distinct subnets (to exercise query join conditions)
- Pagination cursors in `FixtureSet::cursors` represent a multi-page response at default page size

**`PaginationEdgeCases`**
- Device count is exactly `page_size × 3` where `page_size` is the sensor's default page size
- `FixtureSet::cursors` contains exactly 3 cursor values (one per page boundary)
- One page contains zero records (empty trailing page variant)
- Cursor values are at their maximum length (stress-tests cursor storage)

**`SchemaDrift`**
- 30 devices at scale=1.0
- Exactly 1 record has a non-conformant field shape: one of (a) a required field absent, (b) an integer field containing a string, or (c) an extra unknown field present
- `provenance.schema_valid = false` (see BC-3.4.002)
- The drifted record is always at a deterministic index (index 0) for stable test assertions

**`HighChurn`**
- 200 devices at scale=1.0
- At least 20 devices have tombstone records (device was present in a previous poll, now absent)
- `FixtureSet` contains both "current" records and "tombstone" records in `records`
- Tombstones are identifiable by `status = "tombstone"` or `deleted_at` field present

**`DormantTenant`**
- 0 devices, 0 alerts at any scale
- `FixtureSet::records` is an empty `Vec`
- `FixtureSet::cursors` is an empty `Vec`

**Scale behavior:**
- For all archetypes, `count_at_scale = floor(baseline_count * scale)`
- Minimum 0 records at any scale (scale cannot produce negative counts)
- Exception: `DormantTenant` is always 0 records regardless of scale

## Invariants

1. The 8 archetypes listed above are exhaustive for Wave 3; no other archetype string is valid.
2. `#[non_exhaustive]` on the `Archetype` enum means downstream `match` expressions require a wildcard arm; `all_archetypes()` helper provides exhaustive iteration for tests.
3. Baseline counts scale linearly with the `scale` parameter: `actual_count = floor(baseline * scale)`.
4. `SchemaDrift` MUST produce exactly 1 non-conformant record (not zero, not two) to ensure the drift detector fires exactly once per fixture.
5. `DormantTenant` MUST produce 0 records at all scale values; scale has no effect.
6. `AuthOutage` recovery delay (number of 401-class calls before recovery) MUST be overridable via `GenOpts::overrides` JSON Merge Patch.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-3.4.003-01 | `scale = 0.1` for `HealthyOtEnvironment` | `floor(50 * 0.1) = 5` devices, `floor(5 * 0.1) = 0` alerts |
| EC-3.4.003-02 | `scale = 10.0` for `HealthyOtEnvironment` | 500 devices, 50 alerts, 0 high-severity |
| EC-3.4.003-03 | `DormantTenant` at `scale = 100.0` | 0 records; scale has no effect |
| EC-3.4.003-04 | `SchemaDrift` schema validation | Exactly 1 record fails; `provenance.schema_valid = false` (see BC-3.4.002) |
| EC-3.4.003-05 | `AuthOutage` with default overrides | First simulated call returns 401; second returns 200 |
| EC-3.4.003-06 | `AuthOutage` with `overrides = {"auth_outage": {"recovery_after_calls": 3}}` | First 3 calls return 401; 4th returns 200 |
| EC-3.4.003-07 | `PaginationEdgeCases` — last page is empty | `FixtureSet::cursors` includes a cursor pointing to an empty page; empty page produces 0 records when fetched |
| EC-3.4.003-08 | `HighChurn` — tombstone records identified | All tombstone records have deterministic IDs following `dev-{org_slug}-{seed}-tomb-{n}` pattern |

## Canonical Test Vectors

| TV-ID | Input | Expected Output | Category |
|-------|-------|-----------------|----------|
| TV-3.4.003-01 | `generate(orgA, claroty, HealthyOtEnvironment, scale=1.0)` | `len(records) == 50` devices + 5 alerts; 0 high-severity | happy-path |
| TV-3.4.003-02 | `generate(orgA, claroty, CompromisedEndpoint, scale=1.0)` | `len(records) == 50` devices + 20 alerts; at least 3 `severity_id >= 4` | happy-path |
| TV-3.4.003-03 | `generate(orgA, claroty, AuthOutage, scale=1.0)` — first-call response | Response record has `status_code = 401` | happy-path |
| TV-3.4.003-04 | `generate(orgA, claroty, AuthOutage, scale=1.0)` — second-call response | Response record has `status_code = 200`; normal device records returned | happy-path |
| TV-3.4.003-05 | `generate(orgA, claroty, LargeScale, scale=1.0)` | `len(records) == 10,000` devices + 500 alerts; at least 100 distinct subnets | edge-case |
| TV-3.4.003-06 | `generate(orgA, claroty, SchemaDrift, scale=1.0)` | `records[0]` fails Claroty schema; `provenance.schema_valid == false`; remaining 29 records are valid | edge-case |
| TV-3.4.003-07 | `generate(orgA, claroty, DormantTenant, scale=1.0)` | `len(records) == 0`; `len(cursors) == 0` | edge-case |
| TV-3.4.003-08 | `generate(orgA, claroty, HighChurn, scale=1.0)` | `len(records) == 200`; at least 20 records have `status = "tombstone"` | happy-path |
| TV-3.4.003-09 | `generate(orgA, claroty, HealthyOtEnvironment, scale=0.1)` | `len(device_records) == 5`; `len(alert_records) == 0` (`floor(5 * 0.1) = 0`) | edge-case |
| TV-3.4.003-10 | `generate(orgA, claroty, DormantTenant, scale=100.0)` | `len(records) == 0`; scale has no effect for DormantTenant | edge-case |

## Verification Properties

| VP | Property | Proof Method |
|----|----------|--------------|
| VP-3.4.003-A | For each archetype at scale=1.0, `len(records)` equals the documented baseline (within sensor-type row count) | parameterized integration test over all 8 archetypes × 4 sensor types |
| VP-3.4.003-B | `floor(baseline * scale)` record count formula holds for all archetypes and scales in `[0.01, 100.0]` | proptest |
| VP-3.4.003-C | `DormantTenant` always produces 0 records for all scale values | proptest |
| VP-3.4.003-D | `SchemaDrift` always produces exactly 1 non-conformant record | unit test per sensor type |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-039 ("Multi-Tenant Fixture Generation") per capabilities.md §CAP-039 |
| Capability Anchor Justification | CAP-039 ("Multi-Tenant Fixture Generation") per capabilities.md §CAP-039 — this BC defines the "8-archetype catalog (`HealthyOtEnvironment`, `CompromisedEndpoint`, `AuthOutage`, `LargeScale`, `PaginationEdgeCases`, `SchemaDrift`, `HighChurn`, `DormantTenant`) covering the space of multi-tenant behavioral scenarios," which is exactly the archetype catalog enumerated in CAP-039. |
| L2 Domain Invariants | N/A (Wave 3 new capability; DI-NNN assignment pending domain-spec Wave 3 extension) |
| Architecture Module | SS-06 (Client Configuration) per ARCH-INDEX.md |
| Stories | S-3.7.01, S-3.7.02, S-3.7.03, S-3.7.04, S-3.7.05 |

## Related BCs

- BC-3.3.001 — depends on (archetype string is validated against this catalog at startup)
- BC-3.4.001 — depends on (archetype catalog assumes deterministic generation)
- BC-3.4.002 — composes with (SchemaDrift archetype behavior defined here; schema validation behavior in BC-3.4.002)
- BC-3.4.004 — composes with (org-tagged IDs apply to all archetype-generated records)

## Architecture Anchors

- ADR-009 §2.2 — Archetype Catalog enum definition with `#[non_exhaustive]`
- ADR-009 §8 open question 3 — per-archetype baseline counts (resolved here: HealthyOtEnvironment=50, etc.)
- ADR-009 §2.3 — `GenOpts::scale` semantics; `scale = 1.0` = archetype default

## Story Anchor

S-3.7.01, S-3.7.02, S-3.7.03, S-3.7.04, S-3.7.05

## VP Anchors

- VP-3.4.003-A — integration: baseline counts match spec for all archetype × sensor combinations
- VP-3.4.003-B — proptest: count = floor(baseline * scale) for all scales
- VP-3.4.003-C — proptest: DormantTenant always 0 records
- VP-3.4.003-D — unit: SchemaDrift always exactly 1 non-conformant record

## Open Questions

None. All open questions resolved.

- `PaginationEdgeCases` baseline count: **Resolved via D-055** — `PaginationEdgeCases` baseline = `default_page_size() × 3` per-sensor function, not a global constant.
- Armis/CrowdStrike schema availability: **Resolved via D-054** — Schema derivation is pre-story S-3.7.0 under E-3.7; blocks generator implementation for those 2 sensors, not BC authoring.

## BC Changelog

| Version | Change |
|---------|--------|
| v0.4 | M-003 (Pass 3): Stories field and Story Anchor resolved from TBD to S-3.7.01–S-3.7.05 per STORY-INDEX BC Traceability Matrix. |
| v0.3 | C-5 re-anchoring (2026-04-27): capability CAP-009 → CAP-039; Capability Anchor Justification updated to cite CAP-039 ("Multi-Tenant Fixture Generation") verbatim. Open Questions marked resolved. |
| v0.2 | Initial authoring from ADR-009. |
