---
document_type: story
story_id: S-CLAROTY-ACLPOLICY-DTU-001
title: "Claroty DTU — ACL Policies Route (POST /api/v1/organization_acl_policies/, non-paginated, Wave C G6 DTU parity)"
level: "L4"
wave: post-v1-dtu-parity
epic_id: EPIC-OCSF-ROUTING
# DTU-parity anchor: S-ADR058-DTU-PARITY-MIGRATION-001 governs post-v1 DTU-parity batch scheduling.
priority: P2
status: draft
# BC status: pending PO authorship — behavioral_contracts must be non-empty before status: ready.
version: "1.0"
producer: story-writer
timestamp: "2026-08-24T00:00:00Z"
modified: "2026-08-24"
phase: 3
cycle: v1.0.0-brownfield
behavioral_contracts: []
# BC status: pending PO authorship
verification_properties: []
holdout_scenarios: []
points: 5
# 5 pts — single small endpoint; novelty is the non-paginated contract + mandatory
# policy_acl_syntax request field (mirrors what S-CLAROTY-ACLPOLICY-001 built in TOML).
estimated_days: 1
crates_touched: [prism-dtu-claroty]
target_module: "prism-dtu-claroty"
subsystems: [SS-12]
tdd_mode: facade
# facade: DTU clone routes are structural fakes (mock server pattern).
assumption_validations: []
risk_mitigations: []
traces_to: []
inputs: []
input-hash: "[pending-recompute]"
# inputs list and input-hash computed at scheduling time when the story is materialized.
depends_on:
  - S-CLAROTY-ACLPOLICY-001
# Dependency justification: S-CLAROTY-ACLPOLICY-001 defines the claroty_organization_acl_policies TOML
# block (11 cols, pagination type=none, body_template with mandatory policy_acl_syntax, BC-2.16.022).
# DTU cannot be built before the canonical column set + non-paginated contract are finalized and
# live-validated.
blocks: []
gap: G6
dtu_fidelity: L3
---

# S-CLAROTY-ACLPOLICY-DTU-001: Claroty DTU — ACL Policies Route (G6)

**STATUS: DRAFT STUB — Deferred post-v1. Execute with DTU-parity batch (D-2200 / D-2264).**

RG list and full ACs will be authored at scheduling time when this story is dispatched.

---

## Scope

Build the `prism-dtu-claroty` behavioral clone route for
`POST /api/v1/organization_acl_policies/`, replicating the real xDome non-paginated response
shape (envelope `$.organization_acl_policies`, concrete `OrganizationAclPolicyResponseItem`
11-field schema per `endpoint-schema-extract.md §organization_acl_policies`).

**Current state:** No `/api/v1/organization_acl_policies/` route exists in
`crates/prism-dtu-claroty/src/routes/`. The table was added by `S-CLAROTY-ACLPOLICY-001` (G6,
Wave C) with `SAP-2 N/A` until this story ships.

**KEY NOVELTY — non-paginated contract:** This is the only Claroty table that does NOT use
`offset_limit` pagination. The DTU route MUST replicate this correctly:
- Request body contains mandatory `policy_acl_syntax` field (not an optional projection field);
  DTU accepts `"Cisco dACL"` and any other valid syntax value
- Response envelope is `{"organization_acl_policies": [...]}` — NO `count` field
- DTU returns the full fixture list in a single response regardless of request parameters
- No `offset`/`limit` handling required

**Route contract:** POST, `pagination type = none` (no offset/limit injection). OCSF class
`entity_management` (class_uid 3004, existing arm).

11 fields (from OrganizationAclPolicyResponseItem concrete schema): `policy_id`, `policy_name`,
`policy_source`, `applied_models`, `matching_devices`, `policy_acl_type`, `policy_acl`,
`policy_creation_date`, `policy_last_updated`, `policy_updated_by`, `policy_notes`.
Note: `applied_models` is a Json array field.

---

## Deferral

**DEFERRED post-v1 per D-2200 / D-2264.** Full materialization (RG list, ACs, fixture data
including the `policy_acl` text and `applied_models` Json array, SAP-2 parity alignment) occurs
when `S-ADR058-DTU-PARITY-MIGRATION-001` DTU-parity batch is scheduled. SAP-2 probe status
changes from N/A to APPLICABLE once this story ships.

---

## Authority

- `BC-2.16.022` — Claroty ACL Policies Table behavioral contract (parity target: non-paginated
  route contract + response shape)
- `endpoint-schema-extract.md §organization_acl_policies` — concrete OrganizationAclPolicyResponseItem
  schema with 11 fields (`.factory/objectives/xdome-v1-validation/endpoint-schema-extract.md`)
- `endpoint-spike-findings.md §Spike-4 — ACL Pagination Anomaly` — non-paginated contract
  rationale and `policy_acl_syntax` mandatory field decision
  (`.factory/objectives/xdome-v1-validation/endpoint-spike-findings.md`)
- `S-ADR058-DTU-PARITY-MIGRATION-001` — DTU-parity governing story for scheduling this batch
- `xdome-endpoint-expansion-plan.md §Deferred DTU-Creation Stories`
  (`.factory/objectives/xdome-endpoint-expansion-plan.md`)

Section cites only — no line numbers per TD-VSDD-091.

---

## Narrative

As a test infrastructure consumer, I want a behavioral clone of the Claroty xDome ACL Policies
endpoint (`POST /api/v1/organization_acl_policies/`) at fidelity level L3, so that integration
tests and DTU-based holdout evaluation can run for `claroty_organization_acl_policies` without
hitting the live sensor — and critically, so the non-paginated contract (no `count` field, no
offset/limit injection) is validated against realistic fixture data.

---

## Acceptance Criteria

Acceptance criteria will be authored at scheduling time when BCs are assigned. At minimum, ACs
will cover:

- Clone implements `POST /api/v1/organization_acl_policies/` with `pagination type = none`
- Response envelope is `{"organization_acl_policies": [...]}` — NO `count` field
- The route does NOT inject `offset`/`limit` regardless of request parameters
- All 11 OrganizationAclPolicyResponseItem fields present in each fixture item (SAP-2 parity)
- `applied_models` is serialized as a JSON array
- Clone accepts any valid `policy_acl_syntax` value in the request body

(RG list + AC traces authored at scheduling time per SAC-1.)

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| organization_acl_policies route | `crates/prism-dtu-claroty/src/routes/organization_acl_policies.rs` | Effectful (HTTP server route) |
| OrganizationAclPolicyItem type | `crates/prism-dtu-claroty/src/types.rs` | Pure (data struct) |

Subsystem: SS-12 (Sensor Adapters / DTU).

---

## Edge Cases

Edge cases enumerated at scheduling time. Likely candidates:

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Missing `policy_acl_syntax` in request body | 400 (required field per OpenAPI schema) |
| EC-002 | `policy_acl_syntax = "ArubaOS-CX"` (non-default value) | 200 with same fixture data |
| EC-003 | Empty fixture | `{"organization_acl_policies": []}` — NO `count` key in response |

---

## Purity Classification

- Route handler: Effectful (HTTP I/O)
- Response struct construction: Pure
- `applied_models` Json field serialization: Pure (serde_json::Value)

---

## Token Budget Estimate (MANDATORY)

Deferred stub — budget estimated at scheduling time.

| Source | Estimated tokens |
|--------|-----------------|
| This story spec | ~3,500 |
| BC-2.16.022 | ~2,000 |
| Existing DTU route exemplars (2 routes) | ~4,000 |
| endpoint-schema-extract.md §organization_acl_policies | ~1,000 |
| endpoint-spike-findings.md §Spike-4 | ~1,500 |
| Total estimate | ~12,000 |

Within 20-30% of agent context window. No split required.

---

## Tasks (MANDATORY)

Tasks enumerated at scheduling time. High-level sequence:

1. Add `OrganizationAclPolicyItem` (11 fields, `applied_models: serde_json::Value`) to
   `crates/prism-dtu-claroty/src/types.rs`
2. Create `crates/prism-dtu-claroty/src/routes/organization_acl_policies.rs` — POST handler
   that returns fixture without offset/limit handling; no `count` field in response
3. Register route in `crates/prism-dtu-claroty/src/routes/mod.rs`
4. Add fixture data with `policy_acl` text, `applied_models` as JSON array
5. Write SAP-2 parity test (verify all 11 fields present; verify no `count` key in response)
6. Run `just iter prism-dtu-claroty`

---

## Previous Story Intelligence (MANDATORY)

N/A — first stub for this table in the DTU-parity batch.

At scheduling time: review `S-CLAROTY-ACLPOLICY-001` merge notes for the finalized TOML spec
and any live-validation findings about the `policy_acl_syntax` field behavior. The non-paginated
pattern is unique among all Claroty DTU routes — reference `spec_parser.rs`
`PaginationConfig::None` variant for how the live sensor adapter handles it.

---

## Architecture Compliance Rules (MANDATORY)

At scheduling time, extract from architecture docs and ADRs. Key rules:

- DTU routes in `crates/prism-dtu-claroty/src/routes/<table>.rs`
- Response types in `crates/prism-dtu-claroty/src/types.rs`
- All public response types require `#[non_exhaustive]`
- Non-paginated route: MUST NOT include `count` in response envelope; MUST NOT process
  `offset`/`limit` from request body
- `applied_models` Json field: MUST serialize as JSON array (not scalar string)
- ADR-058 OCSF routing active post-ROUTING-001

---

## Library & Framework Requirements (MANDATORY)

Library versions confirmed at scheduling time from `dependency-graph.md`.

| Library | Version pin | Usage |
|---------|------------|-------|
| `axum` | workspace | HTTP route handler |
| `serde` / `serde_json` | workspace | Response serialization; Json fields |

---

## File Structure Requirements (MANDATORY)

| Action | File | Notes |
|--------|------|-------|
| CREATE | `crates/prism-dtu-claroty/src/routes/organization_acl_policies.rs` | POST handler + fixture; no count field; no offset/limit |
| MODIFY | `crates/prism-dtu-claroty/src/routes/mod.rs` | Register route |
| MODIFY | `crates/prism-dtu-claroty/src/types.rs` | Add OrganizationAclPolicyItem struct |
