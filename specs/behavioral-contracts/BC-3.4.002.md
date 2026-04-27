---
document_type: behavioral-contract
level: L3
version: "0.3"
status: PROPOSED
producer: product-owner
timestamp: 2026-04-27T00:00:00
phase: 3.A
inputs:
  - ".factory/specs/architecture/decisions/ADR-009-multi-tenant-data-generator.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "802850d"
traces_to: ["CAP-009"]
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
bc_id: BC-3.4.002
title: Generator output schema-validates against canonical vendor API spec
wave: 3
phase: 3.A
date: 2026-04-27
authors: [product-owner]
related_decisions: [D-043, D-045]
related_adrs: [ADR-009]
inherits_from: null
superseded_by: null
---

# BC-3.4.002: Generator Output Schema-Validates Against Canonical Vendor API Spec

## Description

Every record in a `FixtureSet` generated for a given `sensor_type` must validate against the canonical OpenAPI schema for that sensor — except for the `SchemaDrift` archetype, which intentionally produces records that deviate from the spec to test OCSF normalization resilience. Schema validation runs automatically in test mode (`#[cfg(test)]`) using the `jsonschema` crate against vendored spec files. A generated fixture that fails schema validation (for any archetype other than `SchemaDrift`) is a bug in the generator, not a test failure to be suppressed.

## Preconditions

1. The generator has produced a `FixtureSet` for a given `(org_id, sensor_type, archetype, GenOpts)`.
2. The vendored schema files are present at their canonical paths:
   - Claroty: `.references/poller-bear/docs/specs.json`
   - Cyberint: `.references/poller-express/docs/specs/alert_api_specs.json`, `asm_assets_api_specs.json`, `cve_api_specs.json`, `ioc_api_specs.json`
   - Armis: Rust struct type shapes derived from `.references/poller-coaster/internal/` (translation to JSON Schema pending S-TBD)
   - CrowdStrike: Type shapes derived from `gofalcon` SDK types in `.references/poller-cobra/` and `crates/prism-dtu-crowdstrike/fixtures/`
3. The `jsonschema` crate is available as a dev-dependency in `prism-dtu-common`.
4. Schema validation runs only under `#[cfg(test)]` — it is not executed in production or benchmark builds.

## Postconditions

**For all archetypes except `SchemaDrift`:**

1. Every record in `FixtureSet::records` passes JSON Schema validation against the applicable vendor API spec for the given `sensor_type`.
2. If any record fails validation, the test panics with a message identifying: the sensor type, the archetype, the record index, and the specific schema validation error.
3. The `FixtureSet` is not returned to the caller until all records pass validation (validation is synchronous and blocking in test mode).

**For the `SchemaDrift` archetype specifically:**

1. At least one record in `FixtureSet::records` intentionally fails JSON Schema validation (this is the defining characteristic of the archetype).
2. The generator marks the `FixtureSet` with `provenance.schema_valid = false` to signal to test harnesses that schema drift is expected.
3. Tests consuming a `SchemaDrift` fixture MUST assert `provenance.schema_valid == false` rather than calling the generic schema validator — this prevents the schema validator from masking the intentional drift with a false pass.

**Schema source mapping:**

| Sensor Type | Schema Source | Validation Scope |
|-------------|---------------|-----------------|
| `claroty` | `.references/poller-bear/docs/specs.json` | Device, tag, and alert response shapes |
| `cyberint` | `.references/poller-express/docs/specs/*_api_specs.json` (4 files) | Alert, ASM asset, CVE, IOC response shapes |
| `armis` | Derived Rust→JSON Schema from `.references/poller-coaster/internal/` | Asset and alert response shapes |
| `crowdstrike` | Derived from `gofalcon` types + `crates/prism-dtu-crowdstrike/fixtures/` | Detection, device, containment response shapes |

## Invariants

1. Schema validation runs for ALL archetypes in test mode — including `SchemaDrift`, where failure is asserted rather than suppressed.
2. The `SchemaDrift` archetype is the ONLY exception to the "all records must validate" postcondition.
3. Vendored schema files are treated as read-only test artifacts; the generator never modifies them.
4. Schema validation is dev/test-only (`#[cfg(test)]`); it never runs in production binaries.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-3.4.002-01 | `SchemaDrift` archetype for Claroty | At least one record fails Claroty OpenAPI schema; `provenance.schema_valid = false` |
| EC-3.4.002-02 | `HealthyOtEnvironment` for Claroty | All 50 device records (at scale=1.0) pass Claroty OpenAPI schema |
| EC-3.4.002-03 | `LargeScale` archetype (10,000 records) | All records pass schema validation; validator does not time out or OOM |
| EC-3.4.002-04 | Generator bug introduces an invalid field into a non-SchemaDrift archetype | Test panics with record index and schema error; explicit failure, not silent pass |
| EC-3.4.002-05 | Vendored schema file missing at test time | Test fails at schema-loading step with a clear error naming the missing file path |
| EC-3.4.002-06 | `DormantTenant` archetype (0 records) | Empty `records` slice; schema validation trivially passes (no records to validate) |
| EC-3.4.002-07 | Cyberint — four separate API surface schemas | Each record is validated against the correct sub-spec for its API surface (alert vs. ASM vs. CVE vs. IOC) |

## Canonical Test Vectors

| TV-ID | Input | Expected Output | Category |
|-------|-------|-----------------|----------|
| TV-3.4.002-01 | `generate(orgA, claroty, HealthyOtEnvironment, GenOpts::default())` in `#[cfg(test)]` | All records pass `.references/poller-bear/docs/specs.json` validation; no panic | happy-path |
| TV-3.4.002-02 | `generate(orgA, claroty, SchemaDrift, GenOpts::default())` | At least one record fails schema validation; `provenance.schema_valid == false` | edge-case |
| TV-3.4.002-03 | `generate(orgA, crowdstrike, CompromisedEndpoint, GenOpts::default())` | All records pass CrowdStrike schema validation; `provenance.schema_valid == true` | happy-path |
| TV-3.4.002-04 | `generate(orgA, cyberint, HealthyOtEnvironment, GenOpts::default())` | Records for each Cyberint API surface pass the corresponding sub-spec | happy-path |
| TV-3.4.002-05 | `generate(orgA, claroty, LargeScale, GenOpts { scale: 1.0, .. })` (10,000 records) | All 10,000 records pass schema validation within test timeout | edge-case |
| TV-3.4.002-06 | `generate(orgA, claroty, DormantTenant, GenOpts::default())` | 0 records; schema validation returns immediately with no errors | edge-case |
| TV-3.4.002-07 | Manually inject an invalid record (e.g., missing required `device_id`) into a non-SchemaDrift fixture | Test panics; error message includes record index and missing field name | error |

## Verification Properties

| VP | Property | Proof Method |
|----|----------|--------------|
| VP-3.4.002-A | For all non-SchemaDrift archetypes, every generated record passes schema validation | integration test (parameterized over all sensor_type × archetype combinations) |
| VP-3.4.002-B | For SchemaDrift archetype, `provenance.schema_valid == false` and at least one record fails validation | unit test per sensor type |
| VP-3.4.002-C | Schema validation is absent from release build (`#[cfg(test)]` gate) | cargo build --release with `grep` CI check |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-039 ("Multi-Tenant Fixture Generation") per capabilities.md §CAP-039 |
| Capability Anchor Justification | CAP-039 ("Multi-Tenant Fixture Generation") per capabilities.md §CAP-039 — this BC specifies that "Generator output schema-validates against vendored OpenAPI specs in `#[cfg(test)]` mode," which is exactly the schema-validation behavior CAP-039 describes as part of the fixture generation capability. |
| L2 Domain Invariants | N/A (Wave 3 new capability; DI-NNN assignment pending domain-spec Wave 3 extension) |
| Architecture Module | SS-06 (Client Configuration) per ARCH-INDEX.md; schema sources in `.references/` |
| Stories | S-TBD (Phase 3.A implementation) |

## Related BCs

- BC-3.4.001 — depends on (schema validation assumes deterministic record generation)
- BC-3.4.003 — related to (SchemaDrift archetype defined in BC-3.4.003 catalog)

## Architecture Anchors

- ADR-009 §1.2 — Schema Sources table (Claroty, Cyberint, Armis, CrowdStrike spec locations)
- ADR-009 §3.3 — Threat: schema conformance regression; mitigation via `jsonschema` validation in `#[cfg(test)]`
- ADR-009 §2.2 — `SchemaDrift` archetype definition: "vendor API response deviates from schema"
- `.references/poller-bear/docs/specs.json` — Claroty OpenAPI schema (authoritative)
- `.references/poller-express/docs/specs/` — four Cyberint API surface schemas (authoritative)

## Story Anchor

S-TBD (Phase 3.A implementation)

## VP Anchors

- VP-3.4.002-A — integration: all non-SchemaDrift archetypes pass schema validation
- VP-3.4.002-B — unit: SchemaDrift produces schema_valid=false
- VP-3.4.002-C — CI: schema validation absent from release build

## BC Changelog

| Version | Change |
|---------|--------|
| v0.3 | C-5 re-anchoring (2026-04-27): capability CAP-009 → CAP-039; Capability Anchor Justification updated to cite CAP-039 ("Multi-Tenant Fixture Generation") verbatim. |
| v0.2 | Initial authoring from ADR-009. |
