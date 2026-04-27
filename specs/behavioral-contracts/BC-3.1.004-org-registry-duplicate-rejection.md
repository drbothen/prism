---
document_type: behavioral-contract
level: L3
version: "0.3"
status: PROPOSED
producer: product-owner
timestamp: 2026-04-27T00:00:00
phase: 3.A
inputs: [.factory/specs/architecture/decisions/ADR-006-multi-tenant-dtu-topology.md]
input-hash: ""
traces_to: .factory/specs/architecture/decisions/ADR-006-multi-tenant-dtu-topology.md
origin: greenfield
extracted_from: null
subsystem: SS-06
capability: CAP-038
lifecycle_status: active
introduced: v3.0.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
bc_id: BC-3.1.004
title: OrgRegistry rejects duplicate slugs and UUIDs at registration
wave: 3
related_decisions: [D-041, D-045]
related_adrs: [ADR-006]
inherits_from: null
superseded_by: null
---

# BC-3.1.004: OrgRegistry rejects duplicate slugs and UUIDs at registration

## Description

`OrgRegistry::register(slug, id)` returns a structured `RegistrationError` if the slug is already bound to a different `OrgId`, or if the `OrgId` is already bound to a different slug. The error identifies both the conflicting slug and the conflicting UUID so the operator can resolve the config file without guessing. The registry state is unchanged after a rejected registration — no partial state is applied.

## Preconditions

1. `OrgRegistry` is not yet fully initialized (registrations happen during startup, before MCP bind).
2. The incoming `(slug, id)` pair is syntactically valid: slug matches `^[a-zA-Z0-9_-]{1,64}$`, id is UUID v7.
3. Existing entries in the registry have all passed prior registrations without error.

## Postconditions

1. If `register(slug, id)` succeeds: bijectivity holds with the new pair added (BC-3.1.003).
2. If `register(slug, id)` returns `Err(RegistrationError::SlugConflict)`: the registry contains zero entries for the attempted `slug`/`id` combination that was rejected; the pre-call state is fully preserved.
3. If `register(slug, id)` returns `Err(RegistrationError::IdConflict)`: same — the pre-call state is fully preserved.
4. The error value includes: the conflicting slug strings, the conflicting UUID values, and an operator-actionable message identifying which `customers/*.toml` file to fix.
5. The process startup sequence treats any `RegistrationError` as fatal: Prism does not bind the MCP transport until all registrations succeed.

## Invariants

1. `register` is atomic: it either fully applies the new (slug, id) pair or leaves the registry entirely unchanged.
2. No silent last-write-wins: re-registering an existing slug with a new UUID is always an error, never a silent override.
3. Error messages identify both sides of the conflict (existing entry + attempted entry) so the operator can resolve without additional lookups.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Register (slug-A, uuid-A); register (slug-A, uuid-B) | Returns Err(SlugConflict { slug: slug-A, existing_id: uuid-A, attempted_id: uuid-B }); registry unchanged |
| EC-002 | Register (slug-A, uuid-A); register (slug-B, uuid-A) | Returns Err(IdConflict { id: uuid-A, existing_slug: slug-A, attempted_slug: slug-B }); registry unchanged |
| EC-003 | Register (slug-A, uuid-A); register (slug-A, uuid-A) (exact duplicate) | Returns `Ok` — exact duplicate re-registration is idempotent (same slug and same UUID). Only true bijectivity violations (same slug different UUID, or same UUID different slug) produce `RegistrationError` (D-050). |
| EC-004 | All customer configs conflict; none register successfully | Process logs all RegistrationErrors in one pass (multi-error reporting), then aborts startup |
| EC-005 | Register valid pair after a rejected registration attempt | Registration proceeds normally; rejected attempt left no trace |

## Canonical Test Vectors

| TV-ID | Inputs | Expected Outputs | Notes |
|-------|--------|-----------------|-------|
| TV-3.1.004-01 | register("acme-corp", uuid-A) succeeds; then register("acme-corp", uuid-B) | Err(SlugConflict); error mentions "acme-corp", uuid-A, uuid-B | Duplicate slug caught |
| TV-3.1.004-02 | register("acme-corp", uuid-A) succeeds; then register("beta-inc", uuid-A) | Err(IdConflict); error mentions uuid-A, "acme-corp", "beta-inc" | Duplicate UUID caught |
| TV-3.1.004-03 | register("acme-corp", uuid-A) rejected (conflict); then resolve("acme-corp") | Some(uuid-A) — pre-rejection state preserved | No partial state on error |
| TV-3.1.004-04 | Two conflicting customer TOMLs at startup | Process logs both errors and refuses to start; MCP transport never bound | Fatal startup enforcement |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-3.1.004-01 | Registry size is unchanged after any Err return from register | proptest |
| VP-3.1.004-02 | Err(SlugConflict) message contains both the existing UUID and the attempted UUID | unit test (inspect error fields) |
| VP-3.1.004-03 | Err(IdConflict) message contains both the existing slug and the attempted slug | unit test (inspect error fields) |
| VP-3.1.004-04 | After N successful registrations and one rejected registration, resolve produces correct results for all N successful pairs | proptest |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-038 ("Multi-Tenant Identity Model") per capabilities.md §CAP-038 |
| Capability Anchor Justification | CAP-038 ("Multi-Tenant Identity Model") per capabilities.md §CAP-038 — duplicate rejection at registration time is the enforcement mechanism for the bijectivity invariant that CAP-038 defines: "`OrgRegistry::register` returns `Ok` for idempotent exact-duplicate re-registration and `RegistrationError` for true bijectivity violations." This BC specifies the exact rejection semantics and error payloads for those violations. |
| L2 Domain Invariants | n/a (Wave 3 greenfield) |
| Architecture Module | `prism-core` or `prism-orgs` (ADR-006 §8 open question #5) |
| ADR Source | ADR-006 §2.2 (register method), §3.4 (slug squatting / namespace collision threat) |
| Stories | TBD (filled by story-writer) |

## Related BCs

- BC-3.1.003 — composes with (this BC is the enforcement mechanism for the bijectivity invariant)
- BC-3.1.001 — composes with (resolution correctness depends on no conflicts existing in the registry)

## Architecture Anchors

- ADR-006 §2.2 — `OrgRegistry::register` signature and `RegistrationError` enum
- ADR-006 §3.4 — slug squatting / namespace collision threat model

## Story Anchor

TBD — implementing story to be assigned by story-writer (Epic E-3.1 Step 1)

## VP Anchors

- VP-3.1.004-01 — no partial state on error
- VP-3.1.004-02 — SlugConflict error contains both UUIDs
- VP-3.1.004-03 — IdConflict error contains both slugs
- VP-3.1.004-04 — successful registrations unaffected by subsequent rejections

## Open Questions

None. All open questions resolved.

- EC-003 (exact duplicate behavior): **Resolved via D-050** — Exact same `(slug, uuid)` pair re-registration returns `Ok` (idempotent). Only true bijectivity violations produce `RegistrationError`. EC-003 updated to reflect this.

## BC Changelog

| Version | Change |
|---------|--------|
| v0.3 | C-1 sync (2026-04-27): EC-003 reframed to reflect D-050 resolution — exact duplicate returns `Ok` (idempotent); only true `(OrgId, OrgSlug)` collisions error. Open Questions marked resolved. |
| v0.2 | Initial authoring from ADR-006. |
