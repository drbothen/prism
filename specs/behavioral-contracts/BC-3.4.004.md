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
bc_id: BC-3.4.004
title: Org-tagged record IDs — every generated record carries an org-derived ID prefix
wave: 3
phase: 3.A
date: 2026-04-27
authors: [product-owner]
related_decisions: [D-043, D-045]
related_adrs: [ADR-009]
inherits_from: null
superseded_by: null
---

# BC-3.4.004: Org-Tagged Record IDs — Every Generated Record Carries an Org-Derived ID Prefix

## Description

Every record produced by the generator has a primary identifier derived from the `org_id` of the generating call. The ID format embeds the `org_slug` (resolved from `OrgRegistry`) and the `seed` so that cross-tenant data leakage is detectable by inspection: a test that queries org A and finds a record with an org B prefix in its ID is an unambiguous isolation failure without requiring a cryptographic proof. If slug resolution fails (org not registered), the generator substitutes the first 8 hex characters of the `org_id` UUID as the prefix.

## Preconditions

1. `OrgRegistry` is populated and contains an entry for the `org_id` being generated (normal startup case).
2. The `org_slug` for the `org_id` is available via `OrgRegistry::slug_for(org_id)`.
3. The generator has been called with a valid `(org_id, sensor_type, archetype, GenOpts { seed, .. })`.
4. The `seed` is a `u64` value.

## Postconditions

**Primary ID formats for generated records:**

| Record Type | ID Format | Example |
|-------------|-----------|---------|
| Device / asset | `dev-{org_slug}-{seed}-{index}` | `dev-acme-corp-42-0` |
| Alert / detection | `alert-{org_slug}-{seed}-{index}` | `alert-acme-corp-42-0` |
| Incident | `incident-{uuid8}-{seed}-{index}` where `uuid8` = first 8 chars of `org_id` UUID | `incident-01975e4e-42-0` |
| Tombstone record | `dev-{org_slug}-{seed}-tomb-{index}` | `dev-acme-corp-42-tomb-0` |

Where `{index}` is the zero-based record index within the generated `FixtureSet::records` slice.

**Postcondition guarantees:**

1. For every record in `generate(orgA, ...).records`, the primary ID field begins with `{prefix_A}` where `{prefix_A}` is derived solely from `orgA`'s slug and the call's `seed`.
2. For every record in `generate(orgB, ...).records` (where `orgB ≠ orgA`), the primary ID field begins with `{prefix_B}`, which is different from `{prefix_A}` as long as `orgA.slug ≠ orgB.slug`.
3. The ID sets of `generate(orgA, ...)` and `generate(orgB, ...)` are disjoint when `orgA.slug ≠ orgB.slug` (no shared IDs).
4. If `OrgRegistry::slug_for(org_id)` fails (org not registered at generation time), the generator uses `org_id[0..8]` hex as the prefix: `dev-{org_id_prefix}-{seed}-{index}`.

**Sensor-type-specific ID field names:**

| Sensor Type | Primary ID Field Name |
|-------------|----------------------|
| `claroty` | `device_id` for devices; `alert_id` for alerts |
| `armis` | `asset_id` for assets |
| `crowdstrike` | `device_id` for devices; `detection_id` for detections |
| `cyberint` | `alert_id` for alerts; `asset_id` for ASM assets |

## Invariants

1. Every record in every `FixtureSet` has a primary ID field that contains the org slug as a substring — no anonymous or shared IDs across orgs (ADR-009 §2.5, §3.1).
2. The ID prefix formula is applied consistently to ALL record types (devices, alerts, incidents, tombstones).
3. IDs are deterministic: given the same `(org_id, seed, index)`, the same ID is produced every time (composes with BC-3.4.001 determinism guarantee).
4. When slug resolution fails, the `org_id[0..8]` hex fallback prefix is still org-unique (UUID v7 uniqueness ensures distinct prefixes for distinct orgs with overwhelming probability).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-3.4.004-01 | `orgA` generates 50 devices; all device_ids start with `dev-{slugA}-` | Assertion passes; no ID uses slugB prefix |
| EC-3.4.004-02 | `orgB` generates 50 devices; all device_ids start with `dev-{slugB}-` | Assertion passes; no ID uses slugA prefix |
| EC-3.4.004-03 | orgA and orgB ID sets are compared; intersection is empty | Zero shared IDs (disjoint sets) |
| EC-3.4.004-04 | `OrgRegistry` does not contain the `org_id` at generation time | Fallback: prefix is `dev-{org_id[0..8]}-{seed}-{index}`; no panic |
| EC-3.4.004-05 | Two orgs with different slugs but same seed | IDs differ by slug prefix; still disjoint sets |
| EC-3.4.004-06 | Two orgs with same slug (should not happen — caught by BC-3.3.001 R-CUST-012) | Hypothetically: IDs collide; BC-3.3.001 prevents this at startup |
| EC-3.4.004-07 | Tombstone records | ID format `dev-{org_slug}-{seed}-tomb-{n}`; still org-tagged |
| EC-3.4.004-08 | `DormantTenant` archetype (0 records) | Empty records; ID constraint trivially satisfied |
| EC-3.4.004-09 | `seed = 0` | ID format `dev-{org_slug}-0-{index}`; valid, distinct from seed=1 IDs |

## Canonical Test Vectors

| TV-ID | Input | Expected Output | Category |
|-------|-------|-----------------|----------|
| TV-3.4.004-01 | `generate(orgA{slug="acme-corp"}, claroty, HealthyOtEnvironment, seed=42)` | All `device_id` values start with `"dev-acme-corp-42-"` | happy-path |
| TV-3.4.004-02 | `generate(orgB{slug="globex"}, claroty, HealthyOtEnvironment, seed=42)` | All `device_id` values start with `"dev-globex-42-"` | happy-path |
| TV-3.4.004-03 | orgA ID set ∩ orgB ID set (from TV-3.4.004-01 and TV-3.4.004-02) | Empty set; no shared IDs | edge-case |
| TV-3.4.004-04 | `generate(orgA, claroty, HealthyOtEnvironment, seed=42)` — first device ID | `"dev-acme-corp-42-0"` (index 0) | happy-path |
| TV-3.4.004-05 | `generate(orgA, claroty, HealthyOtEnvironment, seed=42)` — alert ID | `"alert-acme-corp-42-0"` (first alert, index 0) | happy-path |
| TV-3.4.004-06 | `generate` with `org_id` not in `OrgRegistry` | First device ID is `"dev-{org_id[0..8]}-42-0"`; no panic | edge-case |
| TV-3.4.004-07 | `generate(orgA, claroty, HighChurn, seed=42)` — tombstone record | Tombstone `device_id` = `"dev-acme-corp-42-tomb-0"` | edge-case |
| TV-3.4.004-08 | `generate(orgA, claroty, HealthyOtEnvironment, seed=1)` vs `generate(orgA, claroty, HealthyOtEnvironment, seed=2)` | `"dev-acme-corp-1-0"` vs `"dev-acme-corp-2-0"`; different seed → different IDs | edge-case |

## Verification Properties

| VP | Property | Proof Method |
|----|----------|--------------|
| VP-3.4.004-A | For all `(orgA, orgB)` where `orgA.slug ≠ orgB.slug`: `generate(orgA).records.ids ∩ generate(orgB).records.ids = ∅` | proptest with org slug generator |
| VP-3.4.004-B | For all records in `generate(orgX, ...).records`: primary ID contains `orgX.slug` as a substring | proptest |
| VP-3.4.004-C | When `OrgRegistry` lookup fails, generator does not panic and uses hex-prefix fallback | unit test with unregistered org_id |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-039 ("Multi-Tenant Fixture Generation") per capabilities.md §CAP-039 |
| Capability Anchor Justification | CAP-039 ("Multi-Tenant Fixture Generation") per capabilities.md §CAP-039 — this BC specifies that "Every generated record carries an org-tagged primary ID (`dev-{org_slug}-{seed}-{index}`) so cross-tenant data leakage is inspectably detectable," which is the exact org-tagged ID behavior CAP-039 defines as part of fixture generation. |
| L2 Domain Invariants | N/A (Wave 3 new capability; DI-NNN assignment pending domain-spec Wave 3 extension) |
| Architecture Module | SS-06 (Client Configuration) per ARCH-INDEX.md |
| Stories | S-TBD (Phase 3.A implementation) |

## Related BCs

- BC-3.4.001 — composes with (org-tagged IDs are deterministic per BC-3.4.001's guarantee)
- BC-3.4.003 — composes with (all archetype-generated records receive org-tagged IDs)
- BC-3.3.001 — depends on (R-CUST-012 prevents duplicate slugs that would cause ID collisions)

## Architecture Anchors

- ADR-009 §2.5 — Org-Tagged Record IDs: ID formats, slug resolution, hex fallback
- ADR-009 §3.1 — Threat: cross-tenant leakage via identical device IDs; mitigation via org-tagged IDs
- `crates/prism-dtu-claroty/src/state.rs:24` — `tag_store: Mutex<HashMap<String, HashSet<String>>>` — the store that org-tagged IDs protect against keying bugs in

## Story Anchor

S-TBD (Phase 3.A implementation)

## VP Anchors

- VP-3.4.004-A — proptest: orgA.ids ∩ orgB.ids = ∅ when slugs differ
- VP-3.4.004-B — proptest: every record ID contains org slug
- VP-3.4.004-C — unit: unregistered org uses hex-prefix fallback without panic

## Open Questions

None. All open questions resolved.

- ID prefix format: **Resolved via D-059** — Canonical format is slug-based: `"dev-{org_slug}-{seed}-{index}"` (e.g., `"dev-acme-corp-42-0"`). UUID-namespace prefix not implemented.

## BC Changelog

| Version | Change |
|---------|--------|
| v0.3 | C-5 re-anchoring (2026-04-27): capability CAP-009 → CAP-039; Capability Anchor Justification updated to cite CAP-039 ("Multi-Tenant Fixture Generation") verbatim. Open Questions marked resolved. |
| v0.2 | Initial authoring from ADR-009. |
