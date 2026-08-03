---
document_type: story
story_id: "S-CROWDSTRIKE-ALERTS-V2-MIGRATION-001"
title: "CrowdStrike Detects API EOL — migrate alerts to /alerts/queries + /entities/alerts/v2"
wave: "A"
epic_id: wave-a-sensor-fidelity
priority: P0
status: draft
version: "0.1"
severity: CRIT
level: ops
producer: story-writer
timestamp: "2026-08-03"
modified: "2026-08-03"
inputs:
  - .factory/planning/findings-remediation-2026-07-20/triage-capture.md
  - .factory/specs/architecture/decisions/ADR-053-wave-a-sensor-fidelity-remediation-openapi-grounding-armis-token-exchange-cyberint-dual-surface.md
origin_finding: "S-CROWDSTRIKE-ALERTS-V2-MIGRATION-001 (D-1889 triage 2026-07-20, sensor CRIT bucket)"
cycle: "v1.0.0-greenfield"
phase: 3
track: "Platform Engineering"
behavioral_contracts: []
# BC status: pending PO authorship
# S-7.01 gate: behavioral_contracts: [] — status MUST remain draft until a product-owner
# authors and anchors a BC with canonical BC-S.SS.NNN ID for this story.
# Architect adjudication is required FIRST — see ## Gate.
# ADR-028-governed grounding of Alerts-v2 endpoints (no-OpenAPI Confirmed-tier per ADR-053 §D1)
# must precede any BC authorship or implementation.
verification_properties: []
depends_on: []
blocks: []
points: 0
risk: CRIT
holdout_scenarios: []
assumption_validations: []
risk_mitigations: []
---

# S-CROWDSTRIKE-ALERTS-V2-MIGRATION-001: CrowdStrike Detects API EOL — migrate alerts to /alerts/queries + /entities/alerts/v2

## Problem

`crates/prism-sensors/specs/crowdstrike.sensor.toml` declares a detections table grounded on
the CrowdStrike Detects v1 API. The CrowdStrike Detects API reached end-of-life (EOL)
**2025-09-30** — the EOL date is already past. Live tenants receive errors or empty responses
on the v1 detection endpoints; the alerts table is non-functional against any live tenant.

The replacement API surface is:
- `GET /alerts/queries/alerts/v2` — returns alert IDs (query endpoint)
- `POST /entities/alerts/v2` — bulk entity fetch by `composite_id`

The Alerts v2 API uses `composite_id` as the primary keying field (not `detection_id` from
the retired v1 Detects API). Fetching alert entities requires a two-step fan-out: first fetch
IDs from the query endpoint, then batch-fetch entities by `composite_id`. The current
CrowdStrike DTU (`crates/prism-dtu-crowdstrike/`) does not have Alerts-v2 routes and must be
updated to match the new spec after the architect grounds the endpoints.

A secondary issue: the `probe_table` configuration in the current spec references detection
endpoints that no longer exist. This `probe_table` poisoning causes the spec-engine to report
health-probe failures on every CrowdStrike tenant, masking real health issues unrelated to
the alerts surface.

**Credential scope prerequisite (ADR-053 §D1 note):** The CrowdStrike API credential must
carry the `Alerts:READ` OAuth2 scope. A credential provisioned with only `Detects:READ` (the
legacy scope for the retired v1 API) will fail at the Alerts-v2 endpoints with a permission
error. Verification of credential scope is a prerequisite before implementation can proceed;
this must be captured as an acceptance criterion.

## Origin

**Triage date:** 2026-07-20
**Source:** D-1889 live-demo triage, sensor CRIT bucket
**Triage capture:** `.factory/planning/findings-remediation-2026-07-20/triage-capture.md`
**EOL event:** CrowdStrike Detects API EOL 2025-09-30 — already past; no countdown remaining.

## Authority

| Artifact | Verbatim Status | Relevant Clause |
|----------|-----------------|-----------------|
| ADR-053 (Wave-A Sensor Fidelity Remediation) | `status: accepted` | §D1 — OpenAPI grounding order: vendor OpenAPI → spec → DTU; CrowdStrike falls under no-OpenAPI governance (Confirmed-tier web-corroborated findings); §D1 §"CrowdStrike Alerts-v2 credential scope prerequisite" — `Alerts:READ` scope required |
| ADR-028 (Sensor Spec Grounding) | `status: accepted` | §D9 — DTU-EXT-NNN blocker pattern for Partial-tier claims; §D3/§D4/§D6–§D13 authoritative for new endpoint spec authoring |
| BC-2.16.013 (Bundled sensor spec DTU parity) | `status: active` | Governs DTU fidelity requirement for all bundled sensors including CrowdStrike; DTU routes must match spec endpoints after migration |
| BC-2.02.003 (CrowdStrike field mapping) | `status: active` | Field mapping contract for CrowdStrike alerts; must be updated when detection field names change from v1 to v2 schema (composite_id keying, response field differences) |
| BC-2.01.005 (CrowdStrike OAuth2 two-step fetch) | `status: active` | Two-step fetch pattern (query → entity batch); the composite_id fan-out pattern for Alerts v2 is architecturally consistent with this BC's fan-out model; architect must confirm whether the existing model covers this or a new mechanism is required |

**FINDING-A (architect-routed, MEDIUM):** No governing BC or ADR exists yet specifying the
Alerts-v2 endpoint schema, `composite_id` keying, fan-out page size, and corrected probe_table
for CrowdStrike Alerts v2. The architect must ground the new API surface under ADR-053 §D1
no-OpenAPI Confirmed-tier governance before the product-owner can author BCs.

## Gate

**Architect adjudication required.** The migration introduces decisions that require
architectural grounding before BC authorship or implementation:

1. Ground `GET /alerts/queries/alerts/v2` and `POST /entities/alerts/v2` under ADR-053 §D1
   no-OpenAPI Confirmed-tier rules (web-corroborated documentation from independent
   production connectors)
2. Adjudicate how `composite_id` maps to the prism `SensorSpec` table key model
3. Confirm the two-step fan-out (IDs → entity batch) fits BC-2.01.005 or requires a new BC
4. Specify the corrected `probe_table` endpoint for the new Alerts-v2 surface
5. Confirm page size and rate-limit parameters for both the query and entity-fetch endpoints

After architect adjudication, product-owner authors or amends BCs, and story-writer populates
ACs, RG list, BC-5.38.001 density check, and `tdd_mode`.

## Routing

1. **Architect** — ground Alerts-v2 endpoints per ADR-053 §D1 no-OpenAPI Confirmed-tier
   rules; adjudicate `composite_id` keying model; confirm fan-out fits BC-2.01.005 or
   requires new BC; specify corrected `probe_table`; confirm page-size parameters
2. **Product-owner** — amend BC-2.02.003 (field mapping for v2 schema, composite_id
   keying); amend or extend BC-2.01.005 (fan-out pattern if needed); author new BCs per
   architect adjudication outcome
3. **Story-writer** — populate ACs, RG-001..RG-NNN list, BC-5.38.001 density check from
   adjudicated BCs; set `tdd_mode`
4. **Implementer** — migrate `crowdstrike.sensor.toml` to Alerts-v2 endpoints; update DTU
   Alerts-v2 routes; fix probe_table; verify `Alerts:READ` credential scope before proceeding

## Scope — NOT YET SPECIFIED

Acceptance criteria, Red Gate test enumeration (RG-001..RG-NNN), BC-5.38.001 density check,
`tdd_mode` declaration, task decomposition, story-point estimate, and token budget are deferred
to the architect (endpoint grounding, composite_id model) and product-owner (BC authorship or
amendment). This stub registers the defect and EOL context so it is trackable before
adjudication begins.

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 0.1 | 2026-08-03 | story-writer | Initial registration stub from D-1889 triage (sensor CRIT); CrowdStrike Detects API EOL 2025-09-30 already past; architect gate required for ADR-028-governed grounding of Alerts-v2 endpoints under ADR-053 §D1 no-OpenAPI rules; composite_id keying, fan-out, and probe_table poison fix noted |
