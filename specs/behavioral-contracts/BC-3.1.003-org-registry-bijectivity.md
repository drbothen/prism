---
document_type: behavioral-contract
level: L3
version: "0.5"
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
bc_id: BC-3.1.003
title: OrgRegistry Maintains Strict Bijectivity at All Times
wave: 3
related_decisions: [D-041, D-045]
related_adrs: [ADR-006]
inherits_from: null
superseded_by: null
---

# BC-3.1.003: OrgRegistry Maintains Strict Bijectivity at All Times

## Description

At any instant, the `OrgRegistry` mapping is a strict bijection: no two `OrgSlug` values share the same `OrgId`, and no two `OrgId` values share the same `OrgSlug`. This invariant is enforced at registration time (BC-3.1.004), not lazily at lookup time. Slug rename is an atomic operation — the old slug is unmapped in the same operation that maps the new slug, so there is never an instant where both slugs map to the same `OrgId` or where neither slug maps to it.

## Preconditions

1. `OrgRegistry` has been initialized from `customers/*.toml`.
2. All registrations occurred via `OrgRegistry::register`, which enforces bijectivity (BC-3.1.004).
3. No external code modifies the internal BiMap directly (it is a private field, not pub).

## Postconditions

1. After any successful `register(slug, id)` call: `resolve(slug) == Some(id)` and `slug_for(id) == Some(slug)`.
2. After a slug rename (old-slug unmapped, new-slug mapped for the same id): `resolve(old-slug) == None`, `resolve(new-slug) == Some(id)`, `slug_for(id) == Some(new-slug)` — atomically.
3. The number of entries in the forward map always equals the number of entries in the reverse map.

## Invariants

1. `∀ slug, id: resolve(slug) == Some(id) ↔ slug_for(id) == Some(slug)` — holds at all times, not just after quiescence.
2. `∀ slug1 ≠ slug2: resolve(slug1) ≠ resolve(slug2)` — no two slugs share the same OrgId.
3. `∀ id1 ≠ id2: slug_for(id1) ≠ slug_for(id2)` — no two OrgIds share the same slug.
4. Rename atomicity: the BiMap implementation ensures no intermediate state where the old mapping exists simultaneously with the new mapping for the same id.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Register (slug-A, uuid-A); then register (slug-A, uuid-B) | Second registration errors (BC-3.1.004); registry unchanged; slug-A still maps to uuid-A |
| EC-002 | Register (slug-A, uuid-A); then register (slug-B, uuid-A) | Second registration errors (BC-3.1.004); registry unchanged; uuid-A still maps to slug-A |
| EC-003 | Rename slug-A to slug-A2 atomically | After rename: resolve(slug-A)=None, resolve(slug-A2)=Some(uuid-A), slug_for(uuid-A)=Some(slug-A2). No transient state where both exist. |
| EC-004 | Single-entry registry: register one pair, verify both directions | Bijectivity trivially holds; round-trip produces original values |
| EC-005 | Registry with 100 entries; verify forward-count == reverse-count | Both directions contain exactly 100 entries |

## Canonical Test Vectors

| TV-ID | Inputs | Expected Outputs | Notes |
|-------|--------|-----------------|-------|
| TV-3.1.003-01 | Register (slug-A, uuid-A); call resolve(slug-A) and slug_for(uuid-A) | resolve="Some(uuid-A)", slug_for="Some(slug-A)" | Basic bijectivity |
| TV-3.1.003-02 | Register (slug-A, uuid-A); register (slug-A, uuid-B) | Second register returns Err(SlugConflict); resolve(slug-A) still == Some(uuid-A) | Slug duplicate rejected |
| TV-3.1.003-03 | Register (slug-A, uuid-A); register (slug-B, uuid-A) | Second register returns Err(IdConflict); slug_for(uuid-A) still == Some(slug-A) | UUID duplicate rejected |
| TV-3.1.003-04 | Rename slug-A to slug-A2 for uuid-A (atomic remove+insert) | After operation: resolve(slug-A)=None, resolve(slug-A2)=Some(uuid-A), slug_for(uuid-A)=Some("slug-A2") | Atomic rename semantics |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-3.1.003-01 | Bijection invariant: forward-map size == reverse-map size after every operation | proptest (apply random sequence of valid operations; assert sizes equal) |
| VP-3.1.003-02 | No duplicate slug: two successful registrations with same slug is impossible | kani |
| VP-3.1.003-03 | No duplicate uuid: two successful registrations with same uuid is impossible | kani |
| VP-3.1.003-04 | Rename atomicity: no intermediate state observed by concurrent reader | proptest with concurrent access |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-038 ("Multi-Tenant Identity Model") per capabilities.md §CAP-038 |
| Capability Anchor Justification | CAP-038 ("Multi-Tenant Identity Model") per capabilities.md §CAP-038 — bijectivity is the defining structural invariant of `OrgRegistry`, which CAP-038 explicitly specifies: "The bijectivity invariant — no two slugs share a UUID, no two UUIDs share a slug — is enforced atomically at registration time." This BC specifies exactly that invariant. |
| L2 Domain Invariants | n/a (Wave 3 greenfield) |
| Architecture Module | `prism-core` or `prism-orgs` (ADR-006 §8 open question #5) |
| ADR Source | ADR-006 §2.2 (OrgRegistry BiMap), §3.3 (slug rename forensics), §3.4 (slug squatting) |
| Stories | S-3.1.03, S-3.3.02 |

## Related BCs

- BC-3.1.001 — depends on (resolution correctness relies on bijectivity holding)
- BC-3.1.004 — composes with (duplicate rejection is the enforcement mechanism for this invariant)
- BC-3.1.002 — depends on (audit denormalization relies on valid (org_id, org_slug) pairs)

## Architecture Anchors

- ADR-006 §2.2 — `OrgRegistry` struct; `BiMap<OrgSlug, OrgId>` internal representation
- ADR-006 §3.3 — slug rename forensics and atomicity requirement
- ADR-006 §3.4 — slug squatting / namespace collision threat

## Story Anchor

S-3.1.03, S-3.3.02

## VP Anchors

- VP-3.1.003-01 — bijection size invariant
- VP-3.1.003-02 — no duplicate slug
- VP-3.1.003-03 — no duplicate uuid
- VP-3.1.003-04 — rename atomicity

## Open Questions

- None. Bijectivity is a structural property of the BiMap data structure; it is enforced by construction, not by assertion.

## BC Changelog

| Version | Change |
|---------|--------|
| v0.5 | M-004 (pass-8-remediation): Title corrected to Title Case — "OrgRegistry Maintains Strict Bijectivity at All Times". Frontmatter `title:` and H1 updated; BC-INDEX entry updated in same pass. |
| v0.4 | M-003 (Pass 3): Stories field and Story Anchor resolved from TBD to S-3.1.03, S-3.3.02 per STORY-INDEX BC Traceability Matrix. |
| v0.3 | C-5 re-anchoring (2026-04-27): capability CAP-009 → CAP-038; Capability Anchor Justification updated to cite CAP-038 ("Multi-Tenant Identity Model") verbatim. |
| v0.2 | Initial authoring from ADR-006. |
