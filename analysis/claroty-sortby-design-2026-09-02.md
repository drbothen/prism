# Claroty xDome `sort_by` Determinism Design Spec
**Date:** 2026-09-02  
**Author:** architect (design analysis only — no TOML/code changes)  
**Trigger:** Human-directed 2026-09-02 — add explicit, deterministic `sort_by` to the 7 Claroty xDome tables that currently paginate on a non-unique default sort (defects D-001..D-007 in `endpoint-conformance-audit-2026-09-02.md`).  
**Status:** Design phase. Pending human product checkpoint, then TDD implementation.

---

## 1. Evidence Base — OpenAPI Sort-Field Enums

All sort-field enums extracted from `xdome_openapi_06.20.2026.json` using `$ref` resolution. Schema type names are cited as they appear in the OpenAPI `components/schemas` section (per TD-VSDD-091: symbol anchors, no line numbers).

| Table | OpenAPI schema for sort_by items | sort_by field type |
|-------|----------------------------------|--------------------|
| vulnerabilities | `medigator__common__framework__virtual_tables__pagination__sort__ValidatingSortClause___locals____ValidatingSortClause__6` | `Vulnerability__sortable_fields_enum` (32 values — validated enum) |
| audit_logs | `SortClause` | `field: string` (NO enum — any string accepted; no validation at schema level) |
| server_interfaces | `ValidatingSortClause__10` | `ServerInterfaces__sortable_fields_enum` (10 values — validated enum) |
| organization_zones | `ValidatingSortClause__19` | `OrganizationZones__sortable_fields_enum` (10 values — validated enum) |
| organization_zone_policies | `ValidatingSortClause__20` | `OrganizationZonePolicies__sortable_fields_enum` (11 values — validated enum) |
| organization_firewall_groups | `ValidatingSortClause__18` | `OrganizationFirewallGroups__sortable_fields_enum` (10 values — validated enum) |
| organization_firewall_policies | `ValidatingSortClause__21` | `OrganizationFirewallGroupPolicies__sortable_fields_enum` (11 values — validated enum) |

**Key distinction:** Six of seven endpoints use `ValidatingSortClause` (server-side field validation against a declared enum). `audit_logs` uses the generic `SortClause` (field: string — no enum; the server accepts any string as a sort field).

---

## 2. Per-Table Design Decisions

### 2.1 Design Table

| Audit ID | Table | Endpoint | Current non-unique default | Proposed `sort_by` JSON array | All fields in contract sortable enum? | Determinism verdict | Relevance rationale |
|----------|-------|----------|---------------------------|-------------------------------|---------------------------------------|---------------------|---------------------|
| D-001 | vulnerabilities | POST /api/v1/vulnerabilities/ | `[{published_date, desc}]` | `[{"field":"adjusted_vulnerability_score","order":"desc"},{"field":"name","order":"asc"}]` | YES — `adjusted_vulnerability_score` in `Vulnerability__sortable_fields_enum`; `name` in enum | DETERMINISTIC — `name` is the CVE ID / unique vulnerability identifier | HIGH-VOLUME: monroe has >10K vulns; primary sort determines analyst view under DI-019 cap. **Recommendation: `adjusted_vulnerability_score desc` (see §3). Requires human confirmation.** |
| D-002 | audit_logs | POST /api/v1/audit_log/get | `[{timestamp, asc}]` | `[{"field":"timestamp","order":"asc"},{"field":"id","order":"asc"}]` | PARTIAL — `SortClause` has no enum; `id` is UNVERIFIED as sortable (see §2.2 Risk Note) | DETERMINISTIC if `id` is sortable (see §2.2) | MEDIUM-VOLUME: 7-day time-window filter limits most queries to <1K events; instability latent on high-volume tenants. Chronological order is operationally correct for audit trails. |
| D-003 | server_interfaces | POST /api/v1/server_interfaces/ | `[{server_name, asc}]` | `[{"field":"server_name","order":"asc"},{"field":"interface_name","order":"asc"}]` | YES — both `server_name` and `interface_name` in `ServerInterfaces__sortable_fields_enum` (all 10 values) | DETERMINISTIC — (`server_name`, `interface_name`) is the composite unique key per BC-2.16.019 §Postconditions | LOW-VOLUME: <1K interfaces typical; relevance is alphabetical by server then interface |
| D-004 | organization_zones | POST /api/v1/organization_zones/ | `[{priority, asc}]` | `[{"field":"zone_name","order":"asc"}]` | YES — `zone_name` in `OrganizationZones__sortable_fields_enum` (10 values) | DETERMINISTIC — `zone_name` is the unique PK per BC-2.16.020 §Postconditions; single field is sufficient | LOW-VOLUME: <100 zones typical; determinism alone suffices |
| D-005 | organization_zone_policies | POST /api/v1/organization_zone_policies/ | `[{matching_devices, asc}]` | `[{"field":"policy_name","order":"asc"}]` | YES — `policy_name` in `OrganizationZonePolicies__sortable_fields_enum` (11 values) | DETERMINISTIC — `policy_name` is the unique identifier per BC-2.16.020 §Postconditions | LOW-VOLUME: <100 policies typical; determinism alone suffices |
| D-006 | organization_firewall_groups | POST /api/v1/organization_fw_groups/ | `[{priority, asc}]` | `[{"field":"firewall_group_name","order":"asc"}]` | YES — `firewall_group_name` in `OrganizationFirewallGroups__sortable_fields_enum` (10 values) | DETERMINISTIC — `firewall_group_name` is the unique identifier per BC-2.16.021 §Postconditions | LOW-VOLUME: <100 groups typical; determinism alone suffices |
| D-007 | organization_firewall_policies | POST /api/v1/organization_fw_group_policies/ | `[{matching_devices, asc}]` | `[{"field":"policy_name","order":"asc"}]` | YES — `policy_name` in `OrganizationFirewallGroupPolicies__sortable_fields_enum` (11 values) | DETERMINISTIC — `policy_name` is the unique identifier per BC-2.16.021 §Postconditions | LOW-VOLUME: <100 policies typical; determinism alone suffices |

---

### 2.2 Risk Note — audit_logs `id` Tiebreaker Sortability (UNVERIFIED)

**Risk level:** MEDIUM (low practical blast radius given 7-day time-window filter).

**Evidence for `id`:** `ClarotyAuditLogEntry.id` (type `String`) is confirmed in the DTU response struct. The `id` field exists in the audit log response per the DTU fixture and BC-2.16.013 §Postconditions §1. The OpenAPI `GetAuditLogResponse` schema has `properties: {}` (empty — D-011) so no field-level contract exists.

**Evidence gap:** The `SortClause` schema (used by `audit_logs`) takes `field: string` with no enum validation. This contrasts with `ValidatingSortClause` (used by the other 6 endpoints) which validates against a declared enum. The API may silently accept `id` as a sort field (correct behavior), silently accept it but ignore it (wrong sort), or return a 400/422 (would break pagination).

**Decision:** Keep compound `[{timestamp, asc}, {id, asc}]` as the proposed value. `id` is the only viable unique tiebreaker. If the live xDome API rejects `id` as a sort field, the fallback is to accept `timestamp asc` alone (non-deterministic but low practical risk given the time-window filter). The implementation story's Red Gate tests must include a DTU parity test verifying the compound sort body is emitted; live-API verification is deferred to holdout scenarios.

**Fallback:** `[{"field":"timestamp","order":"asc"}]` — non-deterministic but equivalent to the current default; preserves the audit trail ordering that operators expect.

---

## 3. Product-Facing Choice — Vulnerabilities Primary Sort (REQUIRES HUMAN CONFIRMATION)

This is the only table with operationally significant primary-sort choices under the DI-019 10K record cap. Monroe has >10,000 vulnerabilities; every query truncates at 10K records, and the primary sort determines WHICH 10K analysts see.

### Candidates (all verified in `Vulnerability__sortable_fields_enum`):

| Option | Sort | Rationale | Tradeoffs |
|--------|------|-----------|-----------|
| **A (RECOMMENDED)** | `adjusted_vulnerability_score desc` | Claroty's own contextual risk score for the customer's environment — factors in CVSS score, known exploitability, AND asset criticality (medical/OT/IT device class). For an MSSP analyst, this surfaces the vulnerabilities that matter most TO THIS CUSTOMER, not just the worst CVEs globally. | Score is a float; two vulns can share the same score (rare but possible). `name asc` tiebreaker makes it deterministic. Score methodology may differ between xDome versions. |
| B | `cvss_v3_score desc` | Industry-standard CVSS v3 base score. Universal, comparable across tenants. | Does not factor in Claroty's asset context. Many critical vulns may have no exploitable devices at this site. |
| C | `epss_score desc` | EPSS (Exploit Prediction Scoring System from FIRST.org) — probability a CVE is exploited in the wild within 30 days. Highly regarded threat-intel-grounded metric. | Newer metric; some analysts unfamiliar. Only covers CVEs with exploitation data. |
| D | `published_date desc` | Most recently published vulnerabilities first — "what's new." | No risk prioritization. Analysts see newest vulns, not highest-risk vulns. Poor choice under 10K truncation. |

**RECOMMENDATION: Option A — `adjusted_vulnerability_score desc` with `name asc` tiebreaker.**

Full proposed array:
```json
[{"field": "adjusted_vulnerability_score", "order": "desc"}, {"field": "name", "order": "asc"}]
```

**Rationale:** An MSSP running prism for multiple tenants needs to surface the vulnerabilities that are highest-risk in each specific customer environment, not globally-worst CVEs that may not affect any of that customer's device types. `adjusted_vulnerability_score` is Claroty's contextual score and is purpose-built for this. The tiebreaker `name` (CVE ID — e.g., `CVE-2024-XXXXX`) is provably unique and in the 32-value sortable enum.

**If the human prefers Option B (CVSS v3):**
```json
[{"field": "cvss_v3_score", "order": "desc"}, {"field": "name", "order": "asc"}]
```

**HUMAN CONFIRMATION NEEDED BEFORE IMPLEMENTATION:** This is the one product-facing choice. Options A and B are both valid; the choice reflects product philosophy (customer-contextual risk vs. industry-standard score). All other 6 tables use unique name/id keys as their only sort and have no product-philosophy component.

---

## 4. BC Impact Analysis

### 4.1 Owning BCs per Table

| Table(s) | Owning BC | Current body_template postcondition |
|----------|-----------|-------------------------------------|
| vulnerabilities | **BC-2.16.015** | `fetch_vulnerabilities` body_template documents the 18-field `fields` array (§Postconditions §1). No `sort_by` field. |
| audit_logs | **BC-2.16.013** | `fetch_audit_logs` body_template is `'{"filter_by": ${query.filter._claroty_audit_filter_by}}'` (§Postconditions §1). No `sort_by` field. |
| server_interfaces | **BC-2.16.019** | `fetch_server_interfaces` body_template documents the 10-field `fields` array (§Postconditions §1). No `sort_by` field. |
| organization_zones + organization_zone_policies | **BC-2.16.020** | §Postconditions §1 (zones) and §2 (zone_policies) each document their body_template with a `fields` array. No `sort_by` field in either. |
| organization_firewall_groups + organization_firewall_policies | **BC-2.16.021** | §Postconditions §1 (firewall_groups) and §2 (firewall_policies) each document their body_template. No `sort_by` field in either. |

**BC-2.16.002** (Multi-Step Fetch Pipeline — OffsetLimit Pagination) describes the pipeline mechanics (offset/limit injection, short-page halt, DI-019 cap) but does NOT prescribe per-table sort ordering. It does not require amendment.

### 4.2 Amendment Scope Per BC

Each of the 5 owning BCs requires a **MINOR amendment** to:
1. Add the `sort_by` clause to the body_template literal in §Postconditions (the illustrative TOML block and its normative description)
2. Add a new EC or §Invariants entry for the deterministic pagination ordering guarantee: "offset pagination with the declared `sort_by` array is deterministic across page boundaries because the final sort key is the table's unique identifier"

**No new error codes are needed.** The `sort_by` field is optional in the OpenAPI schema (it has a default); adding an explicit value cannot produce a contract error. No new failure modes are introduced.

**Amendment is required** (not just a TOML change) because these BCs explicitly document the `body_template` value as a postcondition. Changing the body_template without amending the BC creates a TOML-vs-BC drift that would be caught by the adversary on the first pass.

### 4.3 BC Amendment Summary

| BC | Amendment type | §Postconditions change | New invariant/EC |
|----|---------------|----------------------|-----------------|
| BC-2.16.015 | body_template + invariant | Add `"sort_by": [<chosen option>, {"field":"name","order":"asc"}]` to §1 fetch_vulnerabilities body_template block | EC-016-015-NNN: "offset pagination on `claroty_vulnerabilities` is deterministic because `name` (CVE ID) uniquely identifies each row" |
| BC-2.16.013 | body_template + invariant | Add `"sort_by": [{"field":"timestamp","order":"asc"},{"field":"id","order":"asc"}]` to §1 fetch_audit_logs body_template block; note `id` UNVERIFIED risk | EC-016-013-NNN: "compound sort ({timestamp,asc},{id,asc}) provides near-deterministic offset pagination for audit_logs; `id` sortability is not schema-validated per SortClause (no enum)" |
| BC-2.16.019 | body_template + invariant | Add `"sort_by": [{"field":"server_name","order":"asc"},{"field":"interface_name","order":"asc"}]` to §1 body_template block | EC-016-019-NNN: "offset pagination on `claroty_server_interfaces` is deterministic because (server_name, interface_name) is the composite unique key" |
| BC-2.16.020 | body_template + invariant (×2) | §1 zones: add `"sort_by": [{"field":"zone_name","order":"asc"}]`; §2 zone_policies: add `"sort_by": [{"field":"policy_name","order":"asc"}]` | EC-016-020-NNN (zones): deterministic on zone_name unique PK; EC-016-020-NNN (zone_policies): deterministic on policy_name unique identifier |
| BC-2.16.021 | body_template + invariant (×2) | §1 firewall_groups: add `"sort_by": [{"field":"firewall_group_name","order":"asc"}]`; §2 firewall_policies: add `"sort_by": [{"field":"policy_name","order":"asc"}]` | EC-016-021-NNN (groups): deterministic on firewall_group_name; EC-016-021-NNN (policies): deterministic on policy_name |

---

## 5. Red Gate Test Shape (SAC-1 Enumerated RG List)

Per SAC-1: each story with `tdd_mode: strict` must carry an enumerated RG list before reaching `status: ready`. The tests must be written FIRST (failing), then the TOML change makes them pass.

**What must fail first (before any TOML edit):**

| RG | Test name pattern | Assert | Mechanism |
|----|-------------------|--------|-----------|
| RG-001 | `test_rg_vulnerabilities_sort_by_in_request_body` | POST body to DTU contains `"sort_by":[{"field":"adjusted_vulnerability_score","order":"desc"},{"field":"name","order":"asc"}]` | DTU echo of received body OR spec-engine build_request unit test asserting body_template JSON expansion |
| RG-002 | `test_rg_audit_logs_sort_by_in_request_body` | POST body to DTU contains `"sort_by":[{"field":"timestamp","order":"asc"},{"field":"id","order":"asc"}]` | Same mechanism |
| RG-003 | `test_rg_server_interfaces_sort_by_in_request_body` | POST body to DTU contains `"sort_by":[{"field":"server_name","order":"asc"},{"field":"interface_name","order":"asc"}]` | Same mechanism |
| RG-004 | `test_rg_organization_zones_sort_by_in_request_body` | POST body to DTU contains `"sort_by":[{"field":"zone_name","order":"asc"}]` | Same mechanism |
| RG-005 | `test_rg_organization_zone_policies_sort_by_in_request_body` | POST body to DTU contains `"sort_by":[{"field":"policy_name","order":"asc"}]` | Same mechanism |
| RG-006 | `test_rg_organization_firewall_groups_sort_by_in_request_body` | POST body to DTU contains `"sort_by":[{"field":"firewall_group_name","order":"asc"}]` | Same mechanism |
| RG-007 | `test_rg_organization_firewall_policies_sort_by_in_request_body` | POST body to DTU contains `"sort_by":[{"field":"policy_name","order":"asc"}]` | Same mechanism |
| RG-008 | `test_rg_vulnerabilities_sort_by_tiebreaker_is_unique_field` | `sort_by` array last element `field` value is `"name"` (in `Vulnerability__sortable_fields_enum`); asserts determinism property structurally | Parse the emitted sort_by JSON, assert final element is the unique key |
| RG-009 | `test_rg_server_interfaces_composite_key_both_present` | `sort_by` array contains both `server_name` and `interface_name` entries | Structural assertion on the 2-element array |
| RG-010 | `test_rg_audit_logs_body_template_contains_sort_by_and_filter_by` | The audit_logs body_template expansion produces a JSON object containing BOTH `"filter_by"` and `"sort_by"` keys | Guards against inadvertently replacing filter_by with sort_by; the new body_template must extend the existing one |

**Implementation path:** The TOML `body_template` strings must be updated to embed the `sort_by` JSON. Because `build_request` does NOT inject `sort_by` automatically (confirmed in the conformance audit appendix, `pipeline.rs::build_request` injects only `offset` and `limit`), the `sort_by` must appear as a literal JSON key in each `body_template`. For `audit_logs`, the current template is a JSON object reference (`'{"filter_by": ${query.filter._claroty_audit_filter_by}}'`); the new template must be `'{"filter_by": ${query.filter._claroty_audit_filter_by}, "sort_by": [{"field":"timestamp","order":"asc"},{"field":"id","order":"asc"}]}'`.

**BC-5.38.001 density check:** 10 Red Gate tests / 7 acceptance criteria (1 per table) ≈ 1.43 per AC. Acceptable; the extra tests cover structural tiebreaker properties (RG-008, RG-009, RG-010).

---

## 6. Risk Notes Summary

| Risk | Severity | Affected table | Mitigation |
|------|----------|---------------|------------|
| `id` sortability UNVERIFIED for audit_logs | MEDIUM | audit_logs | `SortClause` uses free string (no enum validation); `id` is a real response field but the API may silently ignore it as a sort key. Fallback is `timestamp asc` alone (pre-existing behavior; non-deterministic but low blast radius given 7-day window). Validate in holdout scenario. |
| Vulnerabilities primary sort is a product choice | MEDIUM | vulnerabilities | Recommend `adjusted_vulnerability_score desc` but requires human confirmation. Implementation blocked until this decision is made. |
| No sortable UNIQUE field with a single sort key on audit_logs | LOW | audit_logs | Only compound sort is viable; `id` risk above applies. No better alternative exists given the empty OpenAPI response schema. |
| Contract lacks `sort_by` sortable enum for audit_logs | LOW | audit_logs | Confirmed: `SortClause` has no field enum in the xDome OpenAPI. 6 other tables all use `ValidatingSortClause` with declared enums. This is a gap in the xDome API contract for audit_logs specifically. |

---

## 7. Scope

**What this story changes:**
- 7 `body_template` strings in `crates/prism-sensors/specs/claroty.sensor.toml` (add `sort_by` key to each)
- 5 BC amendments (BC-2.16.015, BC-2.16.013, BC-2.16.019, BC-2.16.020, BC-2.16.021) — body_template postcondition update + new determinism EC

**What this story does NOT change:**
- DTU routes (no DTU code change needed — the DTU accepts any well-formed POST body and is not sort-order-aware in its fixture response)
- Pipeline mechanics (build_request in prism-spec-engine continues to inject only offset/limit)
- BC-2.16.002 (multi-step pipeline contract)
- Any other Claroty tables (alerts, devices, device_alert_relations, ot_activity_events, device_vulnerability_relations, servers are already stable per the audit)

---

*Analysis complete: 2026-09-02. Static analysis only. No TOML or code edits made.*
