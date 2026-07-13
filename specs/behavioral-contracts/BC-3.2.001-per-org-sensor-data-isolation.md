---
document_type: behavioral-contract
level: L3
version: "0.10"
status: active
producer: product-owner
timestamp: 2026-04-27T00:00:00
phase: 3.A
inputs: [.factory/specs/architecture/decisions/ADR-006-multi-tenant-dtu-topology.md, .factory/specs/architecture/decisions/ADR-008-dtu-state-segregation.md]
input-hash: "6a21b7f"
traces_to: .factory/specs/architecture/decisions/ADR-008-dtu-state-segregation.md
origin: greenfield
extracted_from: null
subsystem: SS-01
capability: CAP-001
lifecycle_status: active
introduced: cycle-3
modified: "2026-07-13"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
bc_id: BC-3.2.001
title: Per-Org Sensor Data Isolation via Composite HashMap Key
wave: 3
related_decisions: [D-041, D-042, D-045]
related_adrs: [ADR-006, ADR-008]
inherits_from: null
superseded_by: null
---

# BC-3.2.001: Per-Org Sensor Data Isolation via Composite HashMap Key

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
5. **Cross-org query isolation (observable error):** When a `query` MCP tool call is explicitly scoped to org-A (via the `clients` parameter) and the requested sensor is registered in `AdapterRegistry` under a different org but NOT under org-A's `OrgId`, the response MUST be an error envelope containing E-QUERY-032 ("Sensor '{sensor_id}' is not registered for org '{org_slug}'"), NOT a successful empty-result envelope and NOT an opaque MCP `-32000 INTERNAL_ERROR` response (caller-visible message: `"Internal error"`; suggestion: `"See audit log for details."` per BC-2.10.007 message/suggestion split; formerly formatted as `"Internal error; see audit log"` before DEFECT-MCP-ROWSHAPE-NULLS-001 [H8b] 2026-07-13). Zero data rows are returned. No data from any other org's adapter is included. **Rationale:** Swallowing `AdapterNotFound` into `sensor_errors` (the partial-failure path) produces an empty SUCCESS envelope, which is observationally indistinguishable from a successful query that returned no matching rows. AC-012 of S-DEMO-002 and the isolation guarantee of this BC require the condition to be **observably** an error so that a test (and an LLM agent) can distinguish "sensor not available for this org" from "sensor available but returned no data". The surfaced E-QUERY-032 error carries no credential signal and does not violate AD-017 (credential opacity). See error-taxonomy.md §E-QUERY-032 for full surfacing rationale.

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
| EC-006 | Explicit `clients` scope query to org-A for a sensor only registered under org-B | Returns E-QUERY-032 error envelope; zero rows; no data from org-B leaked (postcondition 5) |
| EC-007 | Explicit `clients` scope query to org-A for a sensor registered for BOTH org-A and org-B | Dispatches to org-A's adapter only; returns org-A's data; org-B's adapter is not invoked |

## Canonical Test Vectors

| TV-ID | Inputs | Expected Outputs | Notes |
|-------|--------|-----------------|-------|
| TV-3.2.001-01 | Store tag {"malware"} for (org_id_A, "dev-1"); lookup (org_id_A, "dev-1") | {"malware"} | Same-org retrieval |
| TV-3.2.001-02 | Store tag {"malware"} for (org_id_A, "dev-1"); lookup (org_id_B, "dev-1") | empty set (None / default) | Cross-org isolation |
| TV-3.2.001-03 | Store tag {"tag-A"} for (org_id_A, "dev-1"); store tag {"tag-B"} for (org_id_B, "dev-1"); lookup both | (org_id_A,"dev-1")={"tag-A"}, (org_id_B,"dev-1")={"tag-B"} | Independent per-org state |
| TV-3.2.001-04 | Lookup (org_id_C, "dev-1") where orgC has no entries | empty / default | Missing org returns default |
| TV-3.2.001-05 | reset_for(org_id_A); lookup (org_id_A, "dev-1"); lookup (org_id_B, "dev-1") | orgA: empty; orgB: original content intact | Selective reset |
| TV-3.2.001-06 | MCP query: `FROM claroty_alerts LIMIT 5`, `clients: ["demo-org-a"]`; demo-org-a has CrowdStrike+Armis only; claroty is registered under demo-org-b | Error envelope with code E-QUERY-032, message contains "claroty" and "demo-org-a"; zero rows returned; no Claroty data included | Cross-org isolation observable error (postcondition 5) |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-3.2.001-01 | Cross-org lookup always returns default: write under org_id_A then lookup under org_id_B returns empty/None | proptest (adversarial org pairs, shared resource IDs) |
| VP-3.2.001-02 | Write under org_id_A does not modify any entry keyed under org_id_B | proptest (generate random ops; verify B entries unchanged after A writes) |
| VP-3.2.001-03 | OrgId-flipping mutation is killed: replacing org_id in lookup key with a different org's id returns wrong result | mutation testing (TD-DTU-MUTATE-COVERAGE-001) |
| VP-3.2.001-04 | reset_for(org_id_A) removes exactly the entries for org_id_A and no others | proptest |

### Intentional no-VP: Postcondition 5 (E-QUERY-032 observable error)

**Architect determination (2026-06-02, `vp_index_is_vp_catalog_source_of_truth` evaluation):** No VP-3.2.001-05 is added for postcondition 5. Rationale:

Postcondition 5 — when an explicit `clients`-scoped MCP query requests a sensor not registered for that org, the response must be an E-QUERY-032 error envelope, not a silent empty-success — is a **behavioral/integration property**, not a formally verifiable pure-core property.

The enforcement site is `resolve_source_refs` in `prism-query::materialization`, an `async fn` that consults `Arc<AdapterRegistry>` and `Option<Arc<OrgRegistry>>`. The observable outcome is a JSON-RPC error envelope at the MCP protocol layer. Neither Kani (no async/Arc support) nor proptest (would require constructing full registry state and be indistinguishable from an integration test) can add proof value beyond what an integration test already provides.

**Existing VP-077..080** cover the structurally equivalent pure-core property (HashMap composite-key isolation) with proptest, which is formally verifiable because the isolation is a deterministic, side-effect-free function of the key structure.

**Adequate coverage via tests (mandatory):**
- **AC-012 Red Gate (S-DEMO-002):** subprocess integration test — MCP query with `clients: ["demo-org-a"]` for a sensor registered only under demo-org-b must return a JSON-RPC error envelope with code -32602 and message containing `"E-QUERY-032"`, `"claroty"`, and `"demo-org-a"`.
- **SID-1 unit test (implementer obligation):** a non-`#[ignore]`'d unit test in `crates/prism-query/src/tests/` must drive `resolve_source_refs` with an explicit `clients` list, a populated `AdapterRegistry` where `get(org_id, sensor_id)` returns `None`, and assert the returned `PrismError` carries the E-QUERY-032 message. This test provides fast inner-loop coverage without subprocess overhead.

No VP-INDEX update is required — this decision is intentional and documented here at the BC's verification section, which is the authoritative traceability artifact for BC-level verification scope.

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

- `crates/prism-dtu-claroty/src/state.rs` — `ClarotyState::tag_store` migration target: `HashMap<String,_>` to `HashMap<(OrgId,String),_>`
- `crates/prism-dtu-armis/src/state.rs` — `ArmisState::tag_store` migration target
- `crates/prism-dtu-crowdstrike/src/state.rs` — `CrowdstrikeState::containment_store`, `CrowdstrikeState::detection_status_store` migration targets
- `crates/prism-dtu-cyberint/src/state.rs` — `CyberintState::alert_store`, `CyberintState::session_store` migration targets
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
| v0.10 | **POL-25/POL-29 sweep — postcondition 5 internal-error message updated to BC-2.10.007 split contract (2026-07-13).** Prior wording said the response must NOT be a redacted `"Internal error; see audit log"`, which pinned the RETIRED all-in-one caller-visible message. DEFECT-MCP-ROWSHAPE-NULLS-001 [H8b] changed the E-INT-001 MCP surface to `"Internal error"` (message) + `"See audit log for details."` (suggestion field) per BC-2.10.007 message/suggestion split. Postcondition 5 updated to reference the new format and note the historical transition. Contract semantics unchanged: the response must be E-QUERY-032, NOT an opaque MCP `-32000 INTERNAL_ERROR` response of any format. Frontmatter `modified: [] → "2026-07-13"`. |
| v0.9 | **[reconstructed-tombstone]** D-987 (2026-06-04) POL-14 auto-promotion: `status:` field synced draft→active when anchor story S-DEMO-002 merged PR #171 develop@fdd12251 2026-06-04. `lifecycle_status` was already `active` prior to this row; only the legacy `status:` field was synced. No contract content change. Recorded in BC-INDEX v5.79. |
| v0.8 | Architect VP catalog evaluation (2026-06-02, `vp_index_is_vp_catalog_source_of_truth`): Added §Verification Properties "Intentional no-VP: Postcondition 5" rationale block. Decision (A): no VP-3.2.001-05 authored. Postcondition 5 is a behavioral/integration property (enforcement site is `async fn resolve_source_refs` with Arc-wrapped registry state); Kani and proptest cannot add proof value beyond the mandatory AC-012 Red Gate integration test (S-DEMO-002) and SID-1 unit test. VP-INDEX.md unchanged. |
| v0.7 | S-DEMO-002-spec-evolution-CRIT-001 (2026-06-02): Added postcondition 5 (cross-org query isolation — observable error). When a `query` MCP tool call scoped to org-A via `clients` requests a sensor not registered for org-A's OrgId, the response MUST be an E-QUERY-032 error envelope (not a silent empty-success envelope). Added EC-006 (cross-org query returns E-QUERY-032) and EC-007 (same-org query when both orgs have sensor). Added TV-3.2.001-06 (cross-org isolation MCP test vector). Closes S-DEMO-002 LOCAL adversarial CRIT-001: AC-012 matcher contract updated to assert E-QUERY-032 surface (not redacted E-SENSOR-010). Companion changes: error-taxonomy.md v1.58 (E-QUERY-032 definition) + story v1.6 (AC-012 matcher + AC-007 scan-target decision + FSR .config fix). |
| v0.6 | D-468 (2026-05-13): TD-VSDD-091 cleanup — line-number anchors in Architecture Anchors section converted to symbol-name form (`ClarotyState::tag_store`, `ArmisState::tag_store`, `CrowdstrikeState::containment_store`/`detection_status_store`, `CyberintState::alert_store`/`session_store`). POL-20 migration: `introduced: v3.0.0` → `introduced: cycle-3`. |
| v0.5 | M-004 (pass-8-remediation): Title corrected to Title Case — "Per-Org Sensor Data Isolation via Composite HashMap Key". Frontmatter `title:` and H1 updated; BC-INDEX entry updated in same pass. |
| v0.4 | M-003 (Pass 3): Stories field and Story Anchor resolved from TBD to S-3.1.06, S-3.2.01, S-3.2.02, S-3.2.03, S-3.2.04 per STORY-INDEX BC Traceability Matrix. |
| v0.3 | C-1 sync (2026-04-27): Open Questions marked resolved per D-048 (CrowdStrike session_registry org-scoped at query-engine layer) and D-049 (NVD/ThreatIntel no re-keying). |
| v0.2 | Initial authoring from ADR-006, ADR-008. |
