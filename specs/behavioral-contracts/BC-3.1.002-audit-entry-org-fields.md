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
subsystem: SS-05
capability: CAP-007
lifecycle_status: active
introduced: v3.0.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
bc_id: BC-3.1.002
title: Audit Entry Carries Both org_id and org_slug at Construction Time
wave: 3
related_decisions: [D-041, D-045]
related_adrs: [ADR-006]
inherits_from: null
superseded_by: null
---

# BC-3.1.002: Audit Entry Carries Both org_id and org_slug at Construction Time

## Description

Every `AuditEntry` produced by `emit_*` functions in `prism-audit` must carry two org-identity fields: `org_id: OrgId` (the stable UUID v7 canonical identifier) and `org_slug: OrgSlug` (the denormalized display name at the time of writing). The UUID is the primary key for forensic queries; the slug is denormalized for human readability without requiring a join at query time. Historical records retain the slug that was current when they were written, providing a complete rename history.

## Preconditions

1. The call context has a resolved `OrgId` (obtained via `OrgRegistry::resolve`) before any `emit_*` function is called.
2. The call context has the corresponding `OrgSlug` (obtained via `OrgRegistry::slug_for` or directly from the calling component's context).
3. Neither `org_id` nor `org_slug` is nullable on `AuditEntry`; both fields are required at construction time.
4. `OrgRegistry` is initialized and bijectivity holds (BC-3.1.003) so that `org_id` and `org_slug` form a valid pair at the instant of emission.

## Postconditions

1. The written `AuditEntry` contains `org_id: OrgId` equal to the OrgId in the call context.
2. The written `AuditEntry` contains `org_slug: OrgSlug` equal to the current slug for that OrgId at the time of emission.
3. Neither field is null, empty, or omitted in the serialized audit record.
4. A query filtering audit records by `OrgId` returns all records for that org regardless of what slug was current at write time.
5. Historical records preserve the slug that was active at write time (pre-rename entries show the old slug, post-rename entries show the new slug).

## Invariants

1. `org_id` is immutable across renames: the same UUID appears in all audit records for an org, regardless of how many times the slug changes.
2. `org_slug` is the slug current at write time — it is never retroactively updated when the org renames.
3. `AuditEntry` construction fails (compile error, not runtime panic) if either field is absent — both fields are required struct fields with no `Option` wrapper.
4. The pair `(org_id, org_slug)` in any audit record reflects a valid registered pair at the instant of emission (guaranteed by the call context having resolved both from `OrgRegistry`).

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Org renames from slug-A to slug-A2 between two audit events | Pre-rename record has org_slug=slug-A; post-rename record has org_slug=slug-A2; both have the same org_id. No retroactive update. |
| EC-002 | Query for org_id after rename | Returns all records (old and new slug) because the UUID is stable |
| EC-003 | Attempt to construct AuditEntry with null org_id | Compile-time error: field is non-optional in the struct definition |
| EC-004 | Attempt to construct AuditEntry with null org_slug | Compile-time error: field is non-optional in the struct definition |
| EC-005 | Two orgs with different UUIDs happen to be queried in the same audit window | Records are correctly separated by org_id; no commingling |

## Canonical Test Vectors

| TV-ID | Inputs | Expected Outputs | Notes |
|-------|--------|-----------------|-------|
| TV-3.1.002-01 | `emit_*(org_id=uuid-A, org_slug="acme-corp", ...)` | AuditEntry with org_id=uuid-A and org_slug="acme-corp" | Happy path — both fields present |
| TV-3.1.002-02 | Emit with uuid-A + slug "acme-corp"; rename to "acme-na"; emit again with uuid-A + slug "acme-na" | First entry: org_slug="acme-corp"; second entry: org_slug="acme-na"; both entries: org_id=uuid-A | Rename forensic trail |
| TV-3.1.002-03 | Query audit log by org_id=uuid-A after rename to "acme-na" | Returns both pre-rename and post-rename entries | UUID-stable query |
| TV-3.1.002-04 | Serialize AuditEntry to JSON; inspect fields | JSON contains "org_id": "...", "org_slug": "..." at top level; neither is null | Serialization shape |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-3.1.002-01 | Every AuditEntry has a non-null org_id and non-null org_slug | proptest (generate entries, assert both fields present) |
| VP-3.1.002-02 | `org_id` is stable across rename: two entries with the same UUID but different slugs are both returned by an org_id query | proptest |
| VP-3.1.002-03 | Denormalized slug matches OrgRegistry slug at time of emission | manual / integration test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-007 ("Audit Logging") per capabilities.md §CAP-007 |
| Capability Anchor Justification | CAP-007 ("Audit Logging") per capabilities.md §CAP-007 — this BC defines what org-identity fields every audit entry must carry, directly extending the audit completeness requirement. |
| L2 Domain Invariants | n/a (Wave 3 greenfield) |
| Architecture Module | `prism-audit` (ADR-006 §4 Step 5) |
| ADR Source | ADR-006 §2.3 (translation flow — audit row persists both fields), §3.3 (slug rename forensics) |
| Stories | S-3.1.07 |

## Related BCs

- BC-3.1.001 — depends on (`slug_for(org_id)` called to obtain the current slug before emission)
- BC-3.1.003 — composes with (bijectivity ensures the (org_id, org_slug) pair is valid at emission time)

## Architecture Anchors

- `crates/prism-audit/src/` — `AuditEntry` struct; `emit_*` functions (ADR-006 §4 Step 5)
- ADR-006 §2.3 — translation flow diagram showing audit row shape

## Story Anchor

S-3.1.07

## VP Anchors

- VP-3.1.002-01 — non-null org fields on every entry
- VP-3.1.002-02 — UUID stability across slug rename
- VP-3.1.002-03 — slug matches registry at emission time

## Open Questions

- None. Both fields are required non-optional struct members; no ambiguity in the contract.

## BC Changelog

| Version | Change |
|---------|--------|
| v0.3 | M-004 (pass-8-remediation): Title corrected to Title Case — "Audit Entry Carries Both org_id and org_slug at Construction Time". Frontmatter `title:` and H1 updated; BC-INDEX entry updated in same pass. |
| v0.2 | Initial authoring from ADR-006. |
