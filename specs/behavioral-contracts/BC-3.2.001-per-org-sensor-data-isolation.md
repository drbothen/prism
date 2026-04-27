---
document_type: behavioral-contract
level: L3
version: "0.4"
status: PROPOSED
producer: product-owner
timestamp: 2026-04-27T00:00:00
phase: 3.A
inputs: [.factory/specs/architecture/decisions/ADR-006-multi-tenant-dtu-topology.md, .factory/specs/architecture/decisions/ADR-008-dtu-state-segregation.md]
input-hash: ""
traces_to: .factory/specs/architecture/decisions/ADR-008-dtu-state-segregation.md
origin: greenfield
extracted_from: null
subsystem: SS-01
capability: CAP-001
lifecycle_status: active
introduced: v3.0.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
bc_id: BC-3.2.001
title: Per-org sensor data isolation via composite HashMap key
wave: 3
related_decisions: [D-041, D-042, D-045]
related_adrs: [ADR-006, ADR-008]
inherits_from: null
superseded_by: null
---

# BC-3.2.001: Per-org sensor data isolation via composite HashMap key

## Description

A fetch or write call carrying `OrgId(A)` must not read or modify DTU state entries keyed under `OrgId(B)` for any `B ≠ A`. This isolation is structurally enforced by keying all mutable state stores in client-mode Security Telemetry DTU crates with a composite `(OrgId, String)` tuple rather than a bare `String`. Cross-tenant access requires explicitly constructing the wrong `OrgId`, which is impossible in a call context that only holds `OrgId(A)`. The query plan carries `OrgId` as a non-nullable constraint from PrismQL parse time through adapter dispatch.

## Preconditions

1. The DTU crate is a client-mode Security Telemetry type (`claroty`, `armis`, `crowdstrike`, `cyberint`).
2. All mutable state `HashMap` fields in the DTU state struct are keyed by `(OrgId, String)` (post ADR-008 migration).
3. The query plan carries a non-nullable `org_id: OrgId` constraint from parse time. Loss of this constraint is a compile error.
4. The adapter dispatch layer verifies that the query plan's `OrgId` matches the adapter instance's registered `OrgId` before invoking any DTU method (ADR-007 §2.2).

## Postconditions

1. `state.lookup(org_id_A, resource_id)` returns `None` when the entry was stored under `org_id_B`, even if `resource_id` is identical.
2. `state.write(org_id_A, resource_id, value)` does not modify any entry keyed under `(org_id_B, resource_id)`.
3. After storing device "dev-1" for orgA and device "dev-1" for orgB (different content), lookup("dev-1", orgA) returns orgA's content and lookup("dev-1", orgB) returns orgB's content — independently and correctly.
4. A lookup under an OrgId for which no state has ever been written returns the empty/default value for that store type (empty HashSet, None, etc.) — not an error.

## Invariants

1. The composite key `(OrgId, String)` is the exclusive keying scheme for all mutable state in client-mode DTU state structs. No bare-String keyed mutable store exists in these crates post-migration.
2. The isolation property is structural (type-enforced), not runtime-asserted: the type system prevents constructing `(OrgId(B), resource_id)` in a call context that only has `OrgId(A)`.
3. `DEFAULT_ORG_ID` test constant is `#[cfg(test)]` only and cannot appear in production code paths.
4. The query plan's `org_id` field is `OrgId` (not `Option<OrgId>`); absence of an org constraint is a type error.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Two orgs have devices with identical vendor-assigned IDs | Stored under (org_id_A, "dev-1") and (org_id_B, "dev-1") respectively; lookups return each org's content independently |
| EC-002 | Lookup for orgC that has no entries in a given DTU's state | Returns default (empty set / None); no error |
| EC-003 | Bug in dispatch layer passes wrong OrgId to adapter | Dispatch layer verifies OrgId match before invoking DTU method; mismatch is a fatal dispatch error |
| EC-004 | reset_for(org_id_A) called on a store with entries for both orgA and orgB | Removes all entries keyed (org_id_A, *); entries for org_id_B are unaffected |
| EC-005 | Shared-mode DTU (Slack, PagerDuty, Jira) with OrgId in payload only | These stores are NOT keyed by OrgId; cross-org isolation for shared-mode is not this contract's scope (see BC-3.2.004) |

## Canonical Test Vectors

| TV-ID | Inputs | Expected Outputs | Notes |
|-------|--------|-----------------|-------|
| TV-3.2.001-01 | Store tag {"malware"} for (org_id_A, "dev-1"); lookup (org_id_A, "dev-1") | {"malware"} | Same-org retrieval |
| TV-3.2.001-02 | Store tag {"malware"} for (org_id_A, "dev-1"); lookup (org_id_B, "dev-1") | empty set (None / default) | Cross-org isolation |
| TV-3.2.001-03 | Store tag {"tag-A"} for (org_id_A, "dev-1"); store tag {"tag-B"} for (org_id_B, "dev-1"); lookup both | (org_id_A,"dev-1")={"tag-A"}, (org_id_B,"dev-1")={"tag-B"} | Independent per-org state |
| TV-3.2.001-04 | Lookup (org_id_C, "dev-1") where orgC has no entries | empty / default | Missing org returns default |
| TV-3.2.001-05 | reset_for(org_id_A); lookup (org_id_A, "dev-1"); lookup (org_id_B, "dev-1") | orgA: empty; orgB: original content intact | Selective reset |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-3.2.001-01 | Cross-org lookup always returns default: write under org_id_A then lookup under org_id_B returns empty/None | proptest (adversarial org pairs, shared resource IDs) |
| VP-3.2.001-02 | Write under org_id_A does not modify any entry keyed under org_id_B | proptest (generate random ops; verify B entries unchanged after A writes) |
| VP-3.2.001-03 | OrgId-flipping mutation is killed: replacing org_id in lookup key with a different org's id returns wrong result | mutation testing (TD-DTU-MUTATE-COVERAGE-001) |
| VP-3.2.001-04 | reset_for(org_id_A) removes exactly the entries for org_id_A and no others | proptest |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 ("Sensor Adapter Layer (Internal)") per capabilities.md §CAP-001 |
| Capability Anchor Justification | CAP-001 ("Sensor Adapter Layer (Internal)") per capabilities.md §CAP-001 — this BC defines the isolation guarantee for adapter-layer state stores, which are the per-sensor per-client data stores that CAP-001 describes as "scoped exclusively to that client's sensor instance." |
| L2 Domain Invariants | n/a (Wave 3 greenfield) |
| Architecture Module | `prism-dtu-claroty`, `prism-dtu-armis`, `prism-dtu-crowdstrike`, `prism-dtu-cyberint` (ADR-008 §2.6) |
| ADR Source | ADR-006 §3.1 (cross-tenant data leakage threat), ADR-008 §2.1 (universal re-keying rule), §2.2 (lookup contract), §3.1 (collision threat) |
| Stories | S-3.1.06, S-3.2.01, S-3.2.02, S-3.2.03, S-3.2.04 |

## Related BCs

- BC-3.2.002 — composes with (credential isolation is the companion isolation property at the credential layer)
- BC-3.2.003 — composes with (session token isolation is the companion at the session layer)
- BC-3.1.001 — depends on (org_id obtained via OrgRegistry::resolve before dispatch)

## Architecture Anchors

- `crates/prism-dtu-claroty/src/state.rs:24` — `tag_store` migration target: `HashMap<String,_>` to `HashMap<(OrgId,String),_>`
- `crates/prism-dtu-armis/src/state.rs:72` — `tag_store` migration target
- `crates/prism-dtu-crowdstrike/src/state.rs:86,88` — `containment_store`, `detection_status_store` migration targets
- `crates/prism-dtu-cyberint/src/state.rs:52,56` — `alert_store`, `session_store` migration targets
- ADR-008 §2.1 — full crate-by-crate migration table

## Story Anchor

S-3.1.06, S-3.2.01, S-3.2.02, S-3.2.03, S-3.2.04

## VP Anchors

- VP-3.2.001-01 — cross-org lookup returns default
- VP-3.2.001-02 — write isolation (no cross-org modification)
- VP-3.2.001-03 — OrgId-flipping mutation killed
- VP-3.2.001-04 — reset_for selectivity

## Open Questions

None. All open questions resolved.

- CrowdStrike `session_registry` org-scoping: **Resolved via D-048** — CrowdStrike pagination session IDs are org-scoped at the query-engine layer (not clone state re-keying). The clone's `session_registry` (LruCache keyed by session ID string) is unchanged; the query engine generates session IDs with `OrgId` embedded, ensuring no cross-org collision. This BC's scope is confirmed to not require extension to cover that store.
- NVD/ThreatIntel enrichment DTU re-keying: **Resolved via D-049** — No re-keying required; these are read-only stores. `OrgId` threading is at the route handler level for audit attribution only.

## BC Changelog

| Version | Change |
|---------|--------|
| v0.4 | M-003 (Pass 3): Stories field and Story Anchor resolved from TBD to S-3.1.06, S-3.2.01, S-3.2.02, S-3.2.03, S-3.2.04 per STORY-INDEX BC Traceability Matrix. |
| v0.3 | C-1 sync (2026-04-27): Open Questions marked resolved per D-048 (CrowdStrike session_registry org-scoped at query-engine layer) and D-049 (NVD/ThreatIntel no re-keying). |
| v0.2 | Initial authoring from ADR-006, ADR-008. |
