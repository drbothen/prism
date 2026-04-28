---
document_type: behavioral-contract
level: L3
version: "0.5"
status: PROPOSED
producer: product-owner
timestamp: 2026-04-27T00:00:00
phase: 3.A
inputs: [.factory/specs/architecture/decisions/ADR-006-multi-tenant-dtu-topology.md, .factory/specs/architecture/decisions/ADR-007-configurable-dtu-mode.md]
input-hash: "aba9c59"
traces_to: .factory/specs/architecture/decisions/ADR-007-configurable-dtu-mode.md
origin: greenfield
extracted_from: null
subsystem: SS-01
capability: CAP-040
lifecycle_status: active
introduced: v3.0.0
modified: []
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
bc_id: BC-3.2.004
title: Shared-Mode DTU Tags OrgId in Payload Body Not in Routing Headers
wave: 3
related_decisions: [D-042, D-045]
related_adrs: [ADR-006, ADR-007]
inherits_from: null
superseded_by: null
---

# BC-3.2.004: Shared-Mode DTU Tags OrgId in Payload Body Not in Routing Headers

## Description

Shared-infrastructure DTUs (Slack, PagerDuty, Jira, NVD, ThreatIntel) operate as single MSSP-wide instances. When dispatching a payload on behalf of an org, the adapter must include the `OrgId` as a structured field in the upstream API payload body (e.g., a Slack Block Kit context block, a PagerDuty custom_details field, a Jira issue field). The `OrgId` must not be placed in HTTP headers, URL path segments, or URL query parameters that would be visible to third-party observers of the upstream service. The UUID form of `OrgId` (not the slug) is used in the payload body for AI-opacity. Mode metadata must not appear in analyst-facing query results.

## Preconditions

1. The DTU is configured as `mode = "shared"` in the customer TOML config (ADR-007 §2.1: Slack, PagerDuty, Jira, NVD, ThreatIntel default to shared).
2. The call context provides an `OrgId` at payload-construction time (threaded through from the dispatch layer per ADR-007 §2.6 Step 3).
3. The upstream API payload format supports a structured field for metadata (e.g., Slack Block Kit context blocks, PagerDuty `custom_details`, Jira `labels` or custom fields).
4. The MSSP webhook URL, API key, or routing token is MSSP-owned infrastructure not visible to end-customer Slack/Jira users.

## Postconditions

1. The outgoing upstream API payload contains the `OrgId` UUID string in a designated metadata field (e.g., `"org_id": "<uuid>"` in a JSON body or structured field).
2. The `OrgId` does not appear in: HTTP request URL path segments, URL query parameters, or `X-` HTTP headers that would be forwarded to or visible by third-party upstream service users.
3. Response routing from the shared DTU back to the caller preserves the `OrgId` boundary: responses destined for OrgId(A) are not surfaced to OrgId(B) listeners.
4. Concurrent sends from OrgId(A) and OrgId(B) to the same shared DTU are correctly distinguished: each payload contains its sender's OrgId.
5. Mode metadata (that a DTU is shared) does not appear in OCSF-normalized event records or analyst-facing query results.

## Invariants

1. The `OrgId` UUID form (not `OrgSlug`) is used in shared-mode payload bodies to maintain AI-opacity: the UUID is opaque to LLM context window observers.
2. The shared DTU instance does not maintain per-org routing tables: it routes by the `mode = "shared"` declaration, tagging payloads with OrgId for attribution only.
3. The shared DTU does not provide cross-org data isolation at the storage or query layer: isolation for shared adapters relies on the upstream service's access control (e.g., separate Slack channels, separate Jira project keys).
4. OrgId tagging is the sole mechanism for forensic attribution of shared-mode actions to their originating org.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Concurrent Slack payloads from orgA and orgB | Both payloads contain their respective OrgId UUID in the message body; Slack delivery is sequential or concurrent per HTTP client; no commingling of body content |
| EC-002 | NVD/ThreatIntel enrichment (read-only, no outgoing payload) | No OrgId tagging required in the upstream CVE/IOC query; OrgId attribution is handled at the query-engine audit layer upstream of the adapter call |
| EC-003 | Analyst query returns results from a shared-mode adapter | Results do not include mode metadata or org routing metadata; only the OCSF-normalized event fields are returned |
| EC-004 | Jira ticket created on behalf of orgA; orgB's Jira user views the ticket | Ticket contains org_id in a designated field (e.g., a label "org_id:<uuid>"); the UUID is opaque to Jira users without system-level access |
| EC-005 | Shared Slack DTU delivers notification with OrgId in webhook URL token | VIOLATION: webhook URL token is MSSP-controlled and must not carry OrgId; OrgId belongs in the message body only |

## Canonical Test Vectors

| TV-ID | Inputs | Expected Outputs | Notes |
|-------|--------|-----------------|-------|
| TV-3.2.004-01 | orgA sends incident to shared Slack DTU | Captured payload contains "org_id": "<uuid-A>" in message body; HTTP request URL has no OrgId component | OrgId in body, not URL |
| TV-3.2.004-02 | orgB sends incident to shared Slack DTU concurrently with orgA | orgA payload has uuid-A, orgB payload has uuid-B; both captured independently | Concurrent sends distinguished |
| TV-3.2.004-03 | orgA sends; verify orgB listener sees nothing | orgB's response channel receives no notification from orgA's send | Response routing isolation |
| TV-3.2.004-04 | PagerDuty DTU payload for orgA | Payload JSON contains "org_id" field in custom_details; dedup_key is MSSP-scoped (not org-scoped) | PagerDuty custom_details shape |
| TV-3.2.004-05 | Analyst queries sensor data; checks result fields | Result rows contain OCSF fields only; no "mode", "shared", "org_routing" fields in result | Mode metadata excluded from results |

## Verification Properties

| VP-NNN | Property | Proof Method |
|--------|----------|-------------|
| VP-3.2.004-01 | OrgId appears in payload body: every captured shared-mode payload JSON contains "org_id" key | unit test (inspect captured DTU state after send) |
| VP-3.2.004-02 | OrgId absent from HTTP routing fields: request URL and headers contain no org_id or org_slug component | unit test (inspect HTTP request shape in DTU clone) |
| VP-3.2.004-03 | Concurrent sends produce independent payloads: two orgs' payloads have distinct org_id values | proptest |
| VP-3.2.004-04 | Mode metadata absent from query results: result rows from shared-mode DTU contain no mode field | integration test |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-040 ("Multi-Tenant Adapter Dispatch Mode") per capabilities.md §CAP-040 |
| Capability Anchor Justification | CAP-040 ("Multi-Tenant Adapter Dispatch Mode") per capabilities.md §CAP-040 — this BC specifies the shared-mode payload tagging rule that CAP-040 defines: "Shared-mode adapters tag every upstream API payload with the `OrgId` UUID in the payload body (not in URL paths or headers)." This is precisely the postcondition defined in this BC. |
| L2 Domain Invariants | n/a (Wave 3 greenfield) |
| Architecture Module | `prism-dtu-slack`, `prism-dtu-pagerduty`, `prism-dtu-jira` (ADR-007 §2.6 Step 3) |
| ADR Source | ADR-006 §3.5 (shared-infrastructure DTU privacy threat), ADR-007 §2.2 (shared mode semantics), §3.2 (shared-mode payload leakage threat) |
| Stories | S-3.1.06, S-3.2.05, S-3.2.06, S-3.2.07, S-3.4.05 |

## Related BCs

- BC-3.2.005 — composes with (mode is deployment-time config that determines whether this BC applies)
- BC-3.2.001 — related to (data isolation; shared-mode does not use per-org state keys — this BC is the shared-mode complement)

## Architecture Anchors

- `crates/prism-dtu-slack/src/state.rs:153` — `capture_payload(payload: Value)` call site; OrgId to be embedded in payload construction (ADR-007 §2.6 Step 3)
- `crates/prism-dtu-pagerduty/src/state.rs:91` — incident_registry; dedup_key remains MSSP-scoped (not re-keyed)
- ADR-007 §3.2 — shared-mode payload leakage threat model
- ADR-006 §3.5 — privacy in shared-infrastructure DTU

## Story Anchor

S-3.1.06, S-3.2.05, S-3.2.06, S-3.2.07, S-3.4.05

## VP Anchors

- VP-3.2.004-01 — OrgId in payload body
- VP-3.2.004-02 — OrgId absent from HTTP routing fields
- VP-3.2.004-03 — concurrent sends produce independent payloads
- VP-3.2.004-04 — mode metadata absent from query results

## Open Questions

None. All open questions resolved.

- NVD/ThreatIntel OrgId threading: **Resolved via D-049** — NVD and ThreatIntel DTUs accept `OrgId` optionally at the route handler level for audit attribution only (not for routing or payload tagging). No outgoing payload tagging required. EC-002 exemption confirmed valid.

## BC Changelog

| Version | Change |
|---------|--------|
| v0.5 | M-004 (pass-8-remediation): Title corrected to Title Case — "Shared-Mode DTU Tags OrgId in Payload Body Not in Routing Headers". Frontmatter `title:` and H1 updated; BC-INDEX entry updated in same pass. |
| v0.4 | M-003 (Pass 3): Stories field and Story Anchor resolved from TBD to S-3.1.06, S-3.2.05, S-3.2.06, S-3.2.07, S-3.4.05 per STORY-INDEX BC Traceability Matrix. |
| v0.3 | C-5 re-anchoring (2026-04-27): capability CAP-009 → CAP-040; Capability Anchor Justification updated to cite CAP-040 ("Multi-Tenant Adapter Dispatch Mode") verbatim. Open Questions resolved per D-049. |
| v0.2 | Initial authoring from ADR-006, ADR-007. |
