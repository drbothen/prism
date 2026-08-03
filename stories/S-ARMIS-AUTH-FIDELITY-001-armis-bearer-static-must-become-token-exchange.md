---
document_type: story
story_id: "S-ARMIS-AUTH-FIDELITY-001"
title: "Armis auth fidelity — bearer_static must become token_exchange with POST /api/v1/access_token/"
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
  - .factory/specs/architecture/decisions/ADR-054-native-declarative-http-auth-acquisition.md
origin_finding: "S-ARMIS-AUTH-FIDELITY-001 (D-1889 triage 2026-07-20, sensor CRIT bucket)"
cycle: "v1.0.0-greenfield"
phase: 3
track: "Platform Engineering"
behavioral_contracts: []
# BC status: pending PO authorship
# S-7.01 gate: behavioral_contracts: [] — status MUST remain draft until a product-owner
# authors and anchors a BC with canonical BC-S.SS.NNN ID for this story.
# ADR-053 §D2 and ADR-054 (both status: accepted) ratify the token_exchange mechanism;
# BC-2.01.008 must be amended and a new BC covering DeclarativeHttpAuthProvider may be
# required before this story can transition to status: ready.
verification_properties: []
depends_on: [S-WAVE-A-ENGINE-001]
blocks: []
points: 0
risk: CRIT
holdout_scenarios: []
assumption_validations: []
risk_mitigations: []
---

# S-ARMIS-AUTH-FIDELITY-001: Armis auth fidelity — bearer_static must become token_exchange with POST /api/v1/access_token/

## Problem

`crates/prism-sensors/specs/armis.sensor.toml` declares `auth_type = "bearer_static"` and
carries a comment asserting this is the D-747 LOCKED value. That lock is now stale: ADR-053 §D2
(status: accepted, ratified 2026-07-22, D-1943) explicitly supersedes the D-747 Armis auth
decision and replaces it with `auth_type = "token_exchange"` via native
`DeclarativeHttpAuthProvider` per ADR-054.

The real Armis v1 API requires token-exchange authentication:

1. **Acquire:** POST form-encoded `secret_key=<long-lived credential>` to
   `POST /api/v1/access_token/`. Response:
   `{"success":true,"data":{"access_token":"<short-lived token>","expiration_utc":"<UTC timestamp>"}}`.
2. **Use:** Inject `Authorization: <raw_access_token>` with NO "Bearer" prefix on every
   subsequent request. Bearer-prefix injection causes HTTP 401 against live Armis v1 tenants.
3. **Refresh:** Re-POST the `secret_key` when `expiration_utc` is reached or on any 401.
   No refresh token; the long-lived `secret_key` is re-used for each exchange.

No `token_exchange` `AuthType` variant exists in the codebase today. ADR-054 specifies the
`DeclarativeHttpAuthProvider` design; the `token_exchange` variant and the `[auth_acquisition]`
TOML block are added by the Wave-A engine story (S-WAVE-A-ENGINE-001). This story
`depends_on: [S-WAVE-A-ENGINE-001]` — the engine story must land first.

The Armis DTU (`crates/prism-dtu-armis/`) has no `POST /api/v1/access_token/` route. The DTU
must be updated to match the corrected spec per ADR-053 §D1 (OpenAPI-grounded spec is the
authority; DTU follows spec).

The credential reference name changes from `bearer_token` to `secret_key` per ADR-053 §D2 and
ADR-032 naming conventions. BC-2.06.003 §Per-Sensor credential_refs rows, §Env-Var Derivation
worked examples, and §Canonical Test Vectors armis|bearer_token rows all require amendment per
the ADR-053 §D5 manifest before implementation begins.

## Origin

**Triage date:** 2026-07-20
**Source:** D-1889 live-demo triage, sensor CRIT bucket
**Triage capture:** `.factory/planning/findings-remediation-2026-07-20/triage-capture.md`
**Supersession authorization:** D-1889 (2026-07-20) — "Authorize full correction"; ADR-053
final approval gate PASSED 2026-07-22 (D-1943). ADR-053 supersedes ADR-028 LOCKED Armis
auth (D-747, `bearer_static`). The TOML comment asserting the D-747 lock is stale — ADR-053
§D2 is the governing authority from D-1943 forward.

## Authority

| Artifact | Verbatim Status | Relevant Clause |
|----------|-----------------|-----------------|
| ADR-053 (Wave-A Sensor Fidelity Remediation) | `status: accepted` | §D2 — Armis auth model: `token_exchange` via native `DeclarativeHttpAuthProvider`; `token_path = "/api/v1/access_token/"`, `header_scheme = "raw"`, credential ref name `secret_key`; supersedes ADR-028 LOCKED D-747 Armis `bearer_static` |
| ADR-054 (Native Declarative HTTP Auth Acquisition) | `status: accepted` | §D1 — `token_exchange` AuthType variant; §D3 — `[auth_acquisition]` TOML block fields; §D4 — `DeclarativeHttpAuthProvider::acquire_token()` implementation contract; §D8 — lazy-acquire VP (ZERO network calls during spec-load or boot) |
| ADR-028 (Sensor Spec Grounding) | `status: accepted` | §D1/§D2/§D5 superseded by ADR-053 for new spec authoring; §D3/§D4/§D6–§D13 remain authoritative |
| BC-2.01.008 (Armis bearer AQL) | `status: active` | Title and auth premise invalidated by ADR-053 §D2 — must be amended to reflect `token_exchange` + `header_scheme = "raw"` per ADR-053 §D5 manifest before this story reaches status: ready |
| BC-2.01.016 (Sensor auth open trait) | `status: active` | `acquire_token()` open-trait contract governing the new `DeclarativeHttpAuthProvider` implementation |
| BC-2.06.003 (Credential reference resolution) | `status: active` | §Per-Sensor credential_refs Armis row (`bearer_token` → `secret_key`); §Canonical Test Vectors armis|bearer_token rows require re-pointing to claroty|bearer_token per ADR-053 §D5 manifest option (i) |

**FINDING-A (architect-routed, MEDIUM):** BC-2.01.008 is `status: active` but its auth
premise is invalidated by ADR-053 §D2. Amendment to BC-2.01.008 and possibly a new BC
covering the `DeclarativeHttpAuthProvider` token-exchange lifecycle are required before this
story can reach status: ready. Route to product-owner for BC amendment after architect
confirms no additional ADR is required.

## Gate

**Human gate satisfied at the ADR level.** ADR-053 §D2 was accepted 2026-07-22 (D-1943)
under explicit human authorization D-1889 ("Authorize full correction"), which superseded the
D-747 lock. No additional human gate is required for implementation to proceed — the overturn
is ratified.

However, the following are required before this story reaches status: ready:
1. Product-owner amends BC-2.01.008 per ADR-053 §D5 manifest (auth premise → `token_exchange`)
2. Product-owner amends BC-2.06.003 per ADR-053 §D5 manifest (credential_refs Armis row,
   test vector re-pointing to claroty per option i)
3. Story-writer populates ACs, RG-001..RG-NNN list, BC-5.38.001 density check, and sets
   `tdd_mode`
4. S-WAVE-A-ENGINE-001 must land before this story merges (engine adds `token_exchange`
   variant, `[auth_acquisition]` block parsing, and `header_scheme` field)

## Routing

1. **Product-owner** — amend BC-2.01.008 (auth premise to `token_exchange`); amend
   BC-2.06.003 (credential_refs Armis row bearer_token → secret_key; §Canonical Test Vectors
   re-point armis rows to claroty per ADR-053 §D5 option i)
2. **Story-writer** — populate ACs, RG-001..RG-NNN list, BC-5.38.001 density check from
   amended BCs; set `tdd_mode`; confirm `depends_on: [S-WAVE-A-ENGINE-001]`
3. **Implementer** — migrate `armis.sensor.toml` to `auth_type = "token_exchange"`,
   `header_scheme = "raw"`, `[auth_acquisition]` block (token_path, credential_body_field,
   token_response_path, expiry_field, expiry_mode per ADR-053 §D2); re-clone Armis DTU with
   `POST /api/v1/access_token/` route; update BC-2.01.008 and BC-2.06.003 per §D5 manifest

## Scope — NOT YET SPECIFIED

Acceptance criteria, Red Gate test enumeration (RG-001..RG-NNN), BC-5.38.001 density check,
`tdd_mode` declaration, task decomposition, story-point estimate, and token budget are deferred
to product-owner (BC amendment) and story-writer (AC/RG decomposition). This stub registers
the defect and the ADR-053 §D2 overturn context so it is trackable before BC authorship begins.

## Changelog

| Version | Date | Author | Change |
|---------|------|--------|--------|
| 0.1 | 2026-08-03 | story-writer | Initial registration stub from D-1889 triage (sensor CRIT); records ADR-053 §D2 overturn of D-747 LOCKED Armis auth (ratified D-1943 2026-07-22); human gate satisfied at ADR level; BC-2.01.008 amendment and BC-2.06.003 amendment required before status: ready; depends_on S-WAVE-A-ENGINE-001 |
