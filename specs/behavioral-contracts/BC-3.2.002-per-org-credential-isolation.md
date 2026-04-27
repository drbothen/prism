---
document_type: behavioral-contract
level: L3
version: "0.2"
status: PROPOSED
producer: product-owner
timestamp: 2026-04-27T00:00:00
phase: 3.A
inputs: [.factory/specs/architecture/decisions/ADR-006-multi-tenant-dtu-topology.md]
input-hash: ""
traces_to: .factory/specs/architecture/decisions/ADR-006-multi-tenant-dtu-topology.md
origin: greenfield
extracted_from: null
subsystem: SS-03
capability: CAP-004
lifecycle_status: active
introduced: v3.0.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
bc_id: BC-3.2.002
title: Per-org credential isolation via OrgId-keyed namespace
wave: 3
related_decisions: [D-041, D-045]
related_adrs: [ADR-006]
inherits_from: null
superseded_by: null
---

# BC-3.2.002: Per-org credential isolation via OrgId-keyed namespace

## Description

Bearer tokens and API keys for `OrgId(A)`'s sensors are unreachable from a query context scoped to `OrgId(B)`. The credential store namespace key uses the `OrgId` UUID string (not the `OrgSlug`) as the org component, making it stable across renames and structurally opaque to AI observers. A lookup keyed by `(org_id_A, sensor_name, credential_name)` cannot return credentials stored under `(org_id_B, sensor_name, credential_name)` regardless of whether the slug strings match.

## Preconditions

1. The credential store namespace key format is `"{org_id_uuid}/{sensor}/{name}"` — i.e., `namespace_key` in `prism-credentials/src/namespace.rs` accepts `&OrgId` (UUID representation), not `&OrgSlug`.
2. The query context carries a resolved `OrgId` (obtained via `OrgRegistry::resolve`).
3. Credentials were stored under the correct `OrgId` at provisioning time (via `configure_credential_source` or equivalent).
4. No slug-keyed fallback lookup path exists in the credential store after the migration (ADR-006 §4 Step 3).

## Postconditions

1. `CredentialStore::get(org_id_A, sensor, name)` returns `Ok(credential)` if and only if a credential was stored under namespace `"{org_id_A_uuid}/{sensor}/{name}"`.
2. `CredentialStore::get(org_id_A, sensor, name)` returns `Err(NotFound)` if no credential is stored under that exact namespace key, regardless of whether org_id_B has a credential for the same sensor and name.
3. A slug rename (slug-A → slug-A2 for the same org_id) does not affect credential reachability: credentials stored before the rename remain accessible under the same `OrgId`.
4. Credential values never appear in error messages, logs, or MCP responses (AI-opaque credentials principle per `project_ai_opaque_credentials.md`).

## Invariants

1. The namespace key is always derived from `OrgId` (UUID), never from `OrgSlug` (string), after the ADR-006 Step 3 migration.
2. No credential lookup path accepts a raw string that could be confused with another org's UUID.
3. Credential isolation is at the namespace level: the keyring or file backend physically separates credentials by their namespace string prefix.
4. A credential cache (if present) is keyed by the same `(OrgId, sensor_name, credential_name)` tuple, never by slug.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | lookup(org_id_A, "claroty") where orgA has credentials | Ok(credential_for_A) |
| EC-002 | lookup(org_id_B, "claroty") where orgB has no credentials for claroty | Err(NotFound) |
| EC-003 | lookup(org_id_A, "armis") where orgA has claroty creds but not armis | Err(NotFound) for armis; claroty creds unaffected |
| EC-004 | Org renames from slug-A to slug-A2; lookup(org_id_A, "claroty") after rename | Ok(credential_for_A) — UUID-stable, rename has no effect |
| EC-005 | Two orgs accidentally use same slug at different times (sequential slug reuse) | Old org's creds are stored under old OrgId UUID; new org stored under new OrgId UUID; no collision |

## Canonical Test Vectors

| TV-ID | Inputs | Expected Outputs | Notes |
|-------|--------|-----------------|-------|
| TV-3.2.002-01 | Register cred (org_id_A, "claroty", "api_key"); get(org_id_A, "claroty", "api_key") | Ok(credential_A) | Happy path same-org retrieval |
| TV-3.2.002-02 | Register cred (org_id_A, "claroty", "api_key"); get(org_id_B, "claroty", "api_key") | Err(NotFound) | Cross-org credential isolation |
| TV-3.2.002-03 | Register cred (org_id_A, "claroty"); get(org_id_A, "armis") | Err(NotFound) — different sensor | Per-sensor isolation |
| TV-3.2.002-04 | Register cred under slug "acme-corp" (pre-migration); namespace_key uses UUID; rename slug to "acme-na"; get by org_id | Ok(credential) — UUID stable | Rename stability |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-3.2.002-01 | Cross-org lookup always returns NotFound: cred stored under org_id_A is not returned by get(org_id_B, ...) | proptest |
| VP-3.2.002-02 | Namespace key never contains slug string after migration: grep namespace_key call sites for OrgSlug usage | static analysis / CI lint |
| VP-3.2.002-03 | Rename does not invalidate credential: same org_id returns same cred before and after slug rename | integration test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-004 ("Credential Management") per capabilities.md §CAP-004 |
| Capability Anchor Justification | CAP-004 ("Credential Management") per capabilities.md §CAP-004 — this BC defines the isolation guarantee for credential access: "Credentials are isolated by client -- accessing Client A's CrowdStrike credentials while operating in Client B's context is a type error." This BC makes that guarantee concrete at the namespace key level using OrgId instead of the prior slug string. |
| L2 Domain Invariants | n/a (Wave 3 greenfield) |
| Architecture Module | `prism-credentials` (ADR-006 §4 Step 3) |
| ADR Source | ADR-006 §2.1 (OrgId for credential namespace), §3.2 (cross-tenant credential reachability threat) |
| Stories | S-3.1.04 |

## Related BCs

- BC-3.2.001 — composes with (data-layer isolation; credential isolation is the auth-layer companion)
- BC-3.2.003 — composes with (session token isolation builds on the same OrgId-keyed pattern)
- BC-3.1.001 — depends on (OrgId obtained via OrgRegistry::resolve before credential lookup)

## Architecture Anchors

- `crates/prism-credentials/src/namespace.rs:20` — `namespace_key` function; migration target from `&TenantId` to `&OrgId`
- `crates/prism-credentials/src/trait_.rs:27-66` — credential store trait methods; all async methods gain `&OrgId` parameter
- ADR-006 §3.2 — cross-tenant credential reachability threat model

## Story Anchor

S-3.1.04

## VP Anchors

- VP-3.2.002-01 — cross-org lookup returns NotFound
- VP-3.2.002-02 — namespace key never contains slug string post-migration
- VP-3.2.002-03 — rename does not invalidate credential

## Open Questions

- None. The migration is a mechanical change: `namespace_key(&TenantId)` → `namespace_key(&OrgId)`. The UUID string representation is the namespace prefix; no algorithmic change is required.
