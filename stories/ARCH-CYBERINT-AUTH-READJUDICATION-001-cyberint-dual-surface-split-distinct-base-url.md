---
document_type: story
story_id: "ARCH-CYBERINT-AUTH-READJUDICATION-001"
title: "Cyberint dual-surface split: distinct base_url and auth for Alerts and Assets"
wave: "A"
epic_id: wave-a-sensor-fidelity
priority: P0
status: draft
version: "0.1"
severity: HIGH
level: ops
producer: story-writer
timestamp: "2026-08-03"
modified: "2026-08-03"
inputs:
  - .factory/planning/findings-remediation-2026-07-20/triage-capture.md
  - .factory/specs/architecture/decisions/ADR-053-wave-a-sensor-fidelity-remediation-openapi-grounding-armis-token-exchange-cyberint-dual-surface.md
origin_finding: "ARCH-CYBERINT-AUTH-READJUDICATION-001 (D-1889 triage 2026-07-20)"
origin_cascade: "D-1889 live-demo triage 2026-07-20"
cycle: "v1.0.0-greenfield"
phase: 3
track: "Platform Engineering"
behavioral_contracts:
  - BC-2.01.006
  - BC-2.01.017
  - BC-2.06.003
# BC status (verbatim reads):
#   BC-2.01.006: status: active, lifecycle_status: active
#   BC-2.01.017: status: active, lifecycle_status: active
#   BC-2.06.003: status: active, lifecycle_status: active
# Per S-7.01 gate: behavioral_contracts is non-empty, so draft status is valid.
# BC↔AC bidirectional traces are deferred to specification time (no ACs authored here).
verification_properties: []
depends_on: []
blocks: []
points: 0
risk: HIGH
holdout_scenarios: []
assumption_validations: []
risk_mitigations: []
# tdd_mode: NOT SET — tdd_mode and Red Gate enumeration (RG-001..RG-NNN) are
# deferred to specification time per SAC-1. Setting tdd_mode on a registration
# stub with no acceptance criteria would create an immediate SAC-1 violation.
subsystems: [SS-01, SS-06, SS-16, SS-17]
---

# ARCH-CYBERINT-AUTH-READJUDICATION-001: Cyberint dual-surface split — distinct base_url and auth for Alerts and Assets

## Problem

The current `cyberint.sensor.toml` declares a single sensor definition with one `base_url`
and `auth_type = "cookie_roundtrip"`. The comment in that file asserts
`"auth_type = 'cookie_roundtrip' is the D-747 LOCKED value"` — a lock that ADR-053 has
since overturned. That comment is stale.

The Cyberint API exposes two distinct surfaces:

| Surface | Server prefix | Use case |
|---------|--------------|----------|
| Alerts  | `/alert` | Alert detection, IOC enrichment |
| Assets  | `/asset-configuration` | ASM inventory |

A single TOML sensor definition cannot faithfully represent both surfaces because they
have different `base_url` values. The prism-spec-engine does not support per-table
`base_url` overrides (deferred to Wave-B per ADR-053 §D3-b). The resolution is two
separate TOML sensor specs: `cyberint-alerts.sensor.toml` and `cyberint-assets.sensor.toml`.

**Auth model (CONFIRMED by ADR-053 §D3-c):**

Both surfaces use `Cookie: access_token=<static token>` injection on every request. The
credential is a portal-generated static API key — there is NO login round-trip, NO session
exchange, and NO token expiry. Both surfaces are `auth_type = "cookie_roundtrip"` with
`header_scheme = "cookie:access_token"`. This is the `StaticCookieAuthProvider` pattern
per BC-2.01.017 §P2.

The `X-Api-Key` hypothesis raised during initial triage (v0.1 D3-c of ADR-053) was
**REJECTED**. The precondition that had blocked spec authoring is **RESOLVED** per
ADR-053 §D3-c research confirmation. The header scheme is `cookie:access_token` for both
surfaces.

The existing `cyberint.sensor.toml` is superseded and deleted by the remediation story
(per ADR-053 §D3-a). The credential ref name changes from `api_key` (generic placeholder)
to `access_token` (the wire cookie name), with corresponding env-var changes from
`PRISM_CLIENTS_{ID}_SENSORS_CYBERINT_API_KEY` to per-surface names
`PRISM_CLIENTS_{ID}_SENSORS_CYBERINT_ALERTS_ACCESS_TOKEN` and
`PRISM_CLIENTS_{ID}_SENSORS_CYBERINT_ASSETS_ACCESS_TOKEN`.

**Stale TOML comment:** `cyberint.sensor.toml` contains the comment
`"auth_type = 'cookie_roundtrip' is the D-747 LOCKED value"`. The D-747 lock on the
combined Cyberint single-surface spec was overturned by ADR-053 §D3 under D-1889 human
authorization. That comment is a stale reference to a lock that no longer holds. It
will be moot when `cyberint.sensor.toml` is deleted by the remediation story.

## Origin

D-1889 live-demo triage 2026-07-20, registered at D-2094 engine-defect batch. Source:
`.factory/planning/findings-remediation-2026-07-20/triage-capture.md`.

This item is counted among the eight CRITICALs surfaced at D-1889, specifically as the
arch-human-gate item requiring decision on Cyberint auth before spec authoring could
proceed. The architectural gate (ADR-053 §D3 human authorization) was PASSED 2026-07-22
(D-1943). Spec authoring may now proceed.

## Authority

| Artifact | Verbatim status | Governing clause |
|----------|-----------------|-----------------|
| ADR-053 (Wave-A Sensor Fidelity Remediation) | `status: accepted` | §D3 — Cyberint dual-surface schema decision, auth confirmation, and deletion of combined spec. §D3-c — `X-Api-Key` hypothesis REJECTED; `cookie:access_token` CONFIRMED for Alerts. §D3-d — Assets auth unchanged and confirmed. Human authorization: D-1889/D-1943 |
| BC-2.01.006 (Cyberint Cookie Auth) | `status: active`, `lifecycle_status: active` | Per ADR-053 §D5, BC-2.01.006 scope is narrowed to the Assets surface only after the split. A new Cyberint Alerts BC must be authored covering the `/alert` surface. BC authorship is in scope for the product-owner leg of the remediation story |
| BC-2.01.017 (Static Cookie Auth Provider — No Login Roundtrip) | `status: active`, `lifecycle_status: active` | §P2 — `StaticCookieAuthProvider` contract: injects static `Cookie: {name}={token}`; `acquire_token()` reads credential at request time (not construction); makes NO HTTP request; no login step; no session exchange; no token expiry. Both Cyberint surfaces conform to this pattern |
| BC-2.06.003 (Credential Reference Resolution) | `status: active`, `lifecycle_status: active` | Governs four-tier per-client credential resolution chain. Env-var names change from `CYBERINT_API_KEY` to `CYBERINT_ALERTS_ACCESS_TOKEN` and `CYBERINT_ASSETS_ACCESS_TOKEN` per ADR-032 per-client format |

## Gate

**Architectural gate (ADR level): PASSED.** ADR-053 §D3 was accepted 2026-07-22 (D-1943)
under D-1889 human authorization. This overturns LOCKED decision #4 (D-747: combined
Cyberint `cookie_roundtrip` spec). The gate authorizing spec authoring, BC amendment, and
story decomposition has cleared.

**Story co-land sequencing gate (ADR-053 §D2):** `S-WAVE-A-CYBERINT-SPEC-001` (Cyberint
spec migration story) MUST NOT merge before `S-WAVE-A-ENGINE-001` (standalone Wave-A engine
story adding `header_scheme` to `SensorSpec` and switching `build_request()` dispatch) has
landed. Both new Cyberint spec files declare `header_scheme = "cookie:access_token"` and
require the engine change to pass spec-load Rule 9.

**Spec-first gate (S-7.01):** `behavioral_contracts` is non-empty (`BC-2.01.006`,
`BC-2.01.017`, `BC-2.06.003`), so `status: draft` is valid. Before transitioning to
`status: ready`, the product-owner must author the new Cyberint Alerts BC (scoped to the
`/alert` surface) and all ACs must carry `(traces to BC-S.SS.NNN ...)` bidirectional
citations per BC array propagation policy.

## Routing

Route: **story-writer** (decompose ACs from ADR-053 §D3 and the Alerts BC once authored)
→ **product-owner** (amend BC-2.01.006 to scope it to the Assets surface; author a new BC
for the Cyberint Alerts `/alert` surface) → **implementer**

The architectural decision is already made (ADR-053 §D3 accepted). No further architect
adjudication is required before decomposition begins. The co-land sequencing dependency on
`S-WAVE-A-ENGINE-001` must be recorded in `depends_on` when ACs are authored.

## Scope — NOT YET SPECIFIED

Acceptance criteria, Red Gate test enumeration (RG-001..RG-NNN), BC-5.38.001 density
check, task decomposition, architecture mapping, and story-point estimate are deferred to
specification time. This stub registers the defect as a trackable artifact and records
the ADR-053 §D3 ratification of `cookie:access_token` for Cyberint Alerts (rejecting the
`X-Api-Key` hypothesis) and the stale TOML comment finding.

`tdd_mode` and Red Gate enumeration will be set when acceptance criteria are authored.

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 0.1 | 2026-08-03 | story-writer | Initial registration stub from D-1889 triage; records ADR-053 §D3 ratification (cookie:access_token confirmed, X-Api-Key hypothesis rejected); notes stale cyberint.sensor.toml comment; no ACs or implementation guidance |
