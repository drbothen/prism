---
document_type: story
story_id: "DEFECT-CYBERINT-SPEC-FIDELITY-001"
title: "Cyberint spec fidelity — incidents phantom table, alerts endpoint remap, DTU re-clone, created_at to created_date, add ref_id"
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
origin_finding: "DEFECT-CYBERINT-SPEC-FIDELITY-001 (D-1889 triage 2026-07-20, sensor CRIT bucket)"
cycle: "v1.0.0-greenfield"
phase: 3
track: "Platform Engineering"
behavioral_contracts: []
# BC status: pending PO authorship
# S-7.01 gate: behavioral_contracts: [] — status MUST remain draft until a product-owner
# authors and anchors a BC with canonical BC-S.SS.NNN ID for this story.
# No human gate for the schema dimension. BC-2.02.004 amendment required before status: ready.
verification_properties: []
depends_on: [S-WAVE-A-ENGINE-001]
blocks: []
points: 0
risk: CRIT
holdout_scenarios: []
assumption_validations: []
risk_mitigations: []
---

# DEFECT-CYBERINT-SPEC-FIDELITY-001: Cyberint spec fidelity — incidents phantom table, alerts endpoint remap, DTU re-clone, created_at to created_date, add ref_id

## Problem

`crates/prism-sensors/specs/cyberint.sensor.toml` contains multiple schema fidelity defects
that originated from grounding against the DTU clone rather than the canonical Cyberint OpenAPI.
ADR-053 §D1 (status: accepted, ratified 2026-07-22, D-1943) reverses the grounding order:
spec grounds from vendor OpenAPI; DTU follows spec.

The canonical OpenAPI evidence for the Alerts surface is `cyberint_alerts_openapi_06.20.2026.json`
(title "Alerts", v1.0, server `/alert`, 11 paths).

### Defect 1: Phantom `incidents` table (CRIT)

The spec declares a table with `table_name = "incidents"` mapped to an endpoint that does not
exist in the Cyberint OpenAPI. Cyberint has no incidents object — everything is an alert.
The endpoint was fabricated by the DTU clone during original grounding. Querying the incidents
table against a live Cyberint tenant returns an error or empty response. The table entry must
be deleted entirely.

This was confirmed by D-1949 (2026-07-22), which retired the seeding story for this phantom
table citing "Cyberint API has no incidents object — phantom endpoint; everything is an alert"
per ADR-053 §Finding-1. D-1949 did not remove the table entry from the production TOML spec.

### Defect 2: Alerts endpoint remap required (CRIT)

The spec declares the alerts table using an incorrect HTTP method and extraction path. The
canonical Cyberint Alerts API per `cyberint_alerts_openapi_06.20.2026.json` is:

- **Method:** `POST` (not `GET`) to `/alert/api/v1/alerts`
- **Pagination:** `page` and `size` parameters (not cursor-based); maximum 100 results per page
- **Extraction path:** `$.alerts` (not `$.data`)
- **Server prefix:** `/alert` (base_url context for the Alerts surface)

These are C2-class mechanical corrections per ADR-053 §D1 §"Cyberint C2-class mechanical
fixes" and are in-scope for this story without additional architectural adjudication.

### Defect 3: DTU re-clone required (CRIT)

The Cyberint DTU (`crates/prism-dtu-cyberint/`) must be updated to match the corrected
alerts spec: `POST /alert/api/v1/alerts`, `page`/`size` pagination, `$.alerts` extraction,
and no incidents route. Per ADR-053 §D1, the DTU follows the spec; the corrected spec is the
authoritative source.

### Defect 4: Field renames — `created_at` to `created_date`, add `ref_id`

Per the Cyberint Alerts OpenAPI, the alert object uses `created_date` (not `created_at`) as
the timestamp field name. The `ref_id` field is present in the OpenAPI response schema and
must be added to the spec. DTU fixture data must be updated to match both corrections.

## Scope Separation: Schema Dimension vs. Auth Dimension

This story covers the **schema dimension only**: phantom incidents table deletion, alerts
endpoint remap (POST, `$.alerts`, page/size), DTU re-clone, and field renames/additions.

The **auth dimension** — Cyberint dual-surface split, `cyberint.sensor.toml` supersession and
deletion, creation of `cyberint-alerts.sensor.toml` and `cyberint-assets.sensor.toml`, and
`header_scheme = "cookie:access_token"` wiring — is handled exclusively by the sibling story
`ARCH-CYBERINT-AUTH-READJUDICATION-001`. These two dimensions must not be conflated in
implementation, review, or BC authorship.

Both stories declare `depends_on: [S-WAVE-A-ENGINE-001]` because the engine story adds the
`header_scheme` field to `SensorSpec`, which the new Cyberint spec files require. Co-land
sequencing: this story and `ARCH-CYBERINT-AUTH-READJUDICATION-001` must both land after
S-WAVE-A-ENGINE-001 merges, per ADR-053 §D2 backward-compatibility and co-land sequencing
requirements.

## Origin

**Triage date:** 2026-07-20
**Source:** D-1889 live-demo triage, sensor CRIT bucket
**Triage capture:** `.factory/planning/findings-remediation-2026-07-20/triage-capture.md`
**OpenAPI evidence:** `cyberint_alerts_openapi_06.20.2026.json` (title "Alerts", v1.0, server
`/alert`, 11 paths) — canonical per ADR-053 §D1 for the Alerts surface
**D-1949 confirmation (2026-07-22):** Retired `S-DEMO-CYBERINT-INCIDENTS-SEEDING-001` citing
"Cyberint API has no incidents object — phantom endpoint; everything is an alert"; TOML spec
incidents table entry was not removed in that burst.

## Authority

| Artifact | Verbatim Status | Relevant Clause |
|----------|-----------------|-----------------|
| ADR-053 (Wave-A Sensor Fidelity Remediation) | `status: accepted` | §D1 — OpenAPI grounding order (vendor OpenAPI → spec → DTU); §D1 §"Cyberint C2-class mechanical fixes" — POST method, `$.alerts` extraction, page/size pagination, `/alert` server prefix are in-scope C2 corrections; §D3-a — schema for the two new Cyberint spec files (auth dimension, sibling story scope) |
| BC-2.02.004 (Cyberint field mapping) | `status: active` | Field mapping contract for Cyberint alerts; must be amended to reflect `created_date` rename, `ref_id` addition, phantom `incidents` table removal, and corrected alerts endpoint schema before this story reaches status: ready |
| BC-2.16.013 (Bundled sensor spec DTU parity) | `status: active` | DTU must match spec endpoints and response shapes; DTU re-clone is required per this contract; DTU-EXT-001 and DTU-EXT-005 were already retired (D-1949); remaining parity gap is the corrected alerts endpoint and field names |

**FINDING-A (product-owner-routed, MEDIUM):** BC-2.02.004 is `status: active` but its field
mapping rows are grounded on the incorrect spec (GET endpoint, `$.data` extraction, `created_at`
field, no `ref_id`, phantom incidents rows). BC-2.02.004 must be amended from the Cyberint
Alerts OpenAPI before this story can reach status: ready.

## Gate

**No human gate for the schema dimension.** ADR-053 §D1 acceptance (D-1943, 2026-07-22)
authorizes C2-class mechanical corrections grounded in the canonical Cyberint Alerts OpenAPI.
All four defects listed above fall within the C2-class scope.

Required before this story reaches status: ready:
1. Product-owner amends BC-2.02.004 from `cyberint_alerts_openapi_06.20.2026.json` ground
   truth (created_date, ref_id, $.alerts, POST, page/size, no incidents rows)
2. Story-writer populates ACs, RG-001..RG-NNN list, BC-5.38.001 density check, and sets
   `tdd_mode`
3. S-WAVE-A-ENGINE-001 must land before this story merges (adds `header_scheme` to
   `SensorSpec`; the new Cyberint spec files require it)

The auth dimension (sibling story `ARCH-CYBERINT-AUTH-READJUDICATION-001`) is not a gate
for the schema dimension work. The two stories may proceed in parallel from their shared
`depends_on: [S-WAVE-A-ENGINE-001]` baseline.

## Routing

1. **Product-owner** — amend BC-2.02.004 from Cyberint Alerts OpenAPI (created_date, ref_id,
   $.alerts extraction, POST method, page/size pagination, phantom incidents table removal);
   confirm BC scope boundary with auth dimension (sibling story handles auth surface)
2. **Story-writer** — populate ACs, RG-001..RG-NNN list, BC-5.38.001 density check from
   amended BC-2.02.004; set `tdd_mode`; confirm `depends_on: [S-WAVE-A-ENGINE-001]`
3. **Implementer** — delete `incidents` table entry from `cyberint.sensor.toml`; remap
   alerts table to `POST /alert/api/v1/alerts`, `page`/`size` pagination, `$.alerts`
   extraction; rename `created_at` to `created_date`; add `ref_id`; re-clone Cyberint DTU
   to match corrected spec

## Scope — NOT YET SPECIFIED

Acceptance criteria, Red Gate test enumeration (RG-001..RG-NNN), BC-5.38.001 density check,
`tdd_mode` declaration, task decomposition, story-point estimate, and token budget are deferred
to product-owner (BC-2.02.004 amendment) and story-writer (AC/RG decomposition). This stub
registers the four schema defects and the OpenAPI ground-truth evidence so they are trackable
before BC authorship begins. The auth dimension is explicitly excluded from this stub's scope.

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 0.1 | 2026-08-03 | story-writer | Initial registration stub from D-1889 triage (sensor CRIT); schema dimension only — incidents phantom table (CRIT), alerts endpoint remap to POST /alert/api/v1/alerts with $.alerts and page/size (CRIT), DTU re-clone (CRIT), created_at to created_date and ref_id addition; auth dimension excluded (ARCH-CYBERINT-AUTH-READJUDICATION-001); no human gate; depends_on S-WAVE-A-ENGINE-001 |
