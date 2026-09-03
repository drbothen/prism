---
document_type: story
story_id: DEFECT-CLAROTY-SORTBY-DETERMINISM-001
title: "Claroty xDome sort_by determinism — add explicit sort_by arrays to 7 tables to fix offset-pagination instability under non-unique API default sorts"
level: "L4"
wave: claroty-sortby-fix
epic_id: E-XDOME-DEFECTS
priority: P1
status: ready
# BC status: All 5 BCs active (promoted per POL-14 from originating stories). Per S-7.01
# gate: behavioral_contracts is non-empty and all entries match canonical BC-S.SS.NNN
# pattern — eligible for status: ready after remove-uncertainty pass.
producer: story-writer
timestamp: "2026-09-02T00:00:00Z"
version: "1.2"
modified: "2026-09-02"
phase: 3
cycle: v1.0.0-brownfield
inputs:
  - ".factory/analysis/claroty-sortby-design-2026-09-02.md"
  - ".factory/specs/behavioral-contracts/BC-2.16.015-claroty-vulnerabilities-table.md"
  - ".factory/specs/behavioral-contracts/BC-2.16.013-bundled-sensor-spec-dtu-parity.md"
  - ".factory/specs/behavioral-contracts/BC-2.16.019-claroty-server-interfaces-table.md"
  - ".factory/specs/behavioral-contracts/BC-2.16.020-claroty-org-zone-domain.md"
  - ".factory/specs/behavioral-contracts/BC-2.16.021-claroty-org-firewall-domain.md"
  - "crates/prism-sensors/specs/claroty.sensor.toml"
input-hash: "f7eaf92"
traces_to:
  - "BC-2.16.015"
  - "BC-2.16.013"
  - "BC-2.16.019"
  - "BC-2.16.020"
  - "BC-2.16.021"
points: 3
estimated_days: 0.5
tdd_mode: strict
subsystems: [SS-01]
# Subsystem anchor justifications (ARCH-INDEX Subsystem Registry):
#   SS-01 (Sensor Adapters) owns this story's scope because the ONLY production
#   change is 7 `body_template` string edits in
#   `crates/prism-sensors/specs/claroty.sensor.toml` — the Claroty sensor TOML spec
#   file. All affected tables (vulnerabilities, audit_logs, server_interfaces,
#   organization_zones, zone_policies, firewall_groups, firewall_policies) are
#   governed by SS-01 per ARCH-INDEX. The spec-engine (SS-16) is invoked only in
#   tests (via SpecLoader::parse); no prism-spec-engine production code changes are
#   required.
target_module: prism-sensors
crates_touched: [prism-sensors]
# crates_touched: prism-sensors only (claroty.sensor.toml + tests).
# prism-spec-engine is NOT modified; SpecLoader::parse is called from tests but
# the crate itself has no changed files. No prism-bin changes required since all
# RG tests are spec-parse unit tests that do not exercise the full query pipeline.
capabilities:
  - CAP-029
behavioral_contracts:
  - BC-2.16.015
  # BC-2.16.015 v2.1 — Claroty xDome Vulnerability Findings Table: §Postconditions §1
  # sort-by postcondition (DEFECT-CLAROTY-SORTBY-DETERMINISM-001); EC-016-015-009
  # (offset pagination determinism; accepted residual for identical-score + identical-name
  # bounded tie case); anchors RG-001 + RG-008.
  - BC-2.16.013
  # BC-2.16.013 v1.44 — Bundled Sensor Spec Authoring: §Postconditions §1 audit_logs
  # sort-by postcondition (D-002); EC-016-013-011 (offset pagination determinism);
  # anchors RG-002 + RG-009.
  - BC-2.16.019
  # BC-2.16.019 v1.3 — Claroty Server Interfaces Table: §Postconditions §1 sort-by
  # postcondition (D-003); EC-016-019-007 (offset pagination determinism);
  # anchors RG-003 + RG-010.
  - BC-2.16.020
  # BC-2.16.020 v1.3 — Claroty Org Zone Domain: §Postconditions §1 zones sort-by
  # postcondition (D-004) EC-016-020-011; §Postconditions §2 zone_policies sort-by
  # postcondition (D-005) EC-016-020-012; anchors RG-004 + RG-005.
  - BC-2.16.021
  # BC-2.16.021 v1.3 — Claroty Org Firewall Domain: §Postconditions §1 firewall_groups
  # sort-by postcondition (D-006) EC-016-021-011; §Postconditions §2 firewall_policies
  # sort-by postcondition (D-007) EC-016-021-012; anchors RG-006 + RG-007.
verification_properties: []
holdout_scenarios: []
# holdout_scenarios: PO authors 2–4 hidden SINGLE-USE scenarios during remove-uncertainty
# pass. Stored under the holdout-scenarios directory that test-writer and implementer
# MUST NOT read (contamination control). Story-level holdout gate (human-approved
# 2026-07-13) is BLOCKING before demo/push.
depends_on: []
# depends_on justification: All 7 affected tables have their originating stories
# already merged to develop (S-CLAROTY-VULNS-001 PR #245, S-CLAROTY-AUDITLOG-*
# merged, S-CLAROTY-SERVERS-001 PR #248, S-CLAROTY-ORGPOLICY-001 PR #249,
# S-CLAROTY-ACLPOLICY-001 PR #250). No delivery-time scheduling dependency remains.
blocks: []
acceptance_criteria_count: 7
severity: MED
# Severity rationale: 7 tables have non-unique default sort orders making offset
# pagination non-deterministic (records may duplicate or skip across page boundaries).
# Practical blast radius is low for short-window audit queries and low-volume tables,
# but HIGH for vulnerabilities when the full set exceeds 10,000 records (DI-019 cap
# interacts with non-deterministic sort). Audit A/B consistency failures are latent.
risk: LOW
# Risk rationale: The change is additive (sort_by is an optional API parameter that
# cannot cause a non-200 response for enum-validated tables). audit_logs id tiebreaker
# carries MEDIUM risk (SortClause has no field enum; id may be rejected or silently
# ignored). Fallback protocol documented in BC-2.16.013 and AC-002.
assumption_validations: []
risk_mitigations: []
origin_finding: "D-001..D-007 in .factory/analysis/endpoint-conformance-audit-2026-09-02.md"
origin_cascade: "Human-directed 2026-09-02 — deterministic sort_by for all Claroty xDome paginated tables"
---

# DEFECT-CLAROTY-SORTBY-DETERMINISM-001: Claroty xDome sort_by Determinism — 7 Tables

## Authority

**BC-2.16.015 §Postconditions §1 sort-by postcondition** governs the exact
`sort_by` array for `claroty_vulnerabilities`:
`[{"field":"adjusted_vulnerability_score","order":"desc"},{"field":"name","order":"asc"}]`.
Both fields are confirmed members of `Vulnerability__sortable_fields_enum` (xDome OpenAPI
`ValidatingSortClause__6`). DI-019 truncation rationale: `adjusted_vulnerability_score desc`
ensures highest-risk records survive the 10K cap. The `name asc` secondary sort (mapping to
`finding_info.title`, `options=["REQUIRED"]` for presence, NOT a uniqueness constraint) is the
**best-available sortable tiebreaker** from `Vulnerability__sortable_fields_enum`; the opaque
record PK `id` is NOT a member of that enum and cannot serve as a sortable tiebreaker.
**Accepted residual non-determinism (symmetric with BC-2.16.013 §Sort-by postcondition):**
where two rows share BOTH the same `adjusted_vulnerability_score` AND the same `name`, page-
boundary ordering between them is not guaranteed. This residual is bounded — ties require
identical score AND identical title — and is further mitigated by risk-primary ordering and
the DI-019 10K cap. Accepted, not a defect. Anchor: EC-016-015-009; RG-001; RG-008.

**BC-2.16.013 §Postconditions §1 (audit_logs sort-by postcondition)** governs
`claroty_audit_logs`: preferred body_template is
`'{"filter_by": ${query.filter._claroty_audit_filter_by}, "sort_by": [{"field":"timestamp","order":"asc"},{"field":"id","order":"asc"}]}'`.
**`id` tiebreaker caveat:** `audit_logs` uses the generic `SortClause` schema (free-form
string field, no enum validation). The live-validation gate MUST assert OBSERVED ORDERING
DETERMINISM — a 2xx response alone is NOT evidence that `id` is honored. If `id` is
rejected (4xx) OR silently ignored (compound sort indistinguishable from timestamp-only),
the fallback is `[{"field":"timestamp","order":"asc"}]` alone — the implementer MUST use
the fallback and amend BC-2.16.013 and `claroty.sensor.toml` in the same burst; on
fallback adoption, BOTH RG-002 (parameterized to accept either form) AND RG-009 must be
updated. **Residual non-determinism (accepted if fallback adopted):** `audit_logs` has no
unique sortable tiebreaker in its documented field set (`category`, `action`,
`user_display_name`, `note`, `timestamp`, `details`); 7-day window bounds blast radius.
Anchor: EC-016-013-011; RG-002; RG-009.

**BC-2.16.019 §Postconditions §1 sort-by postcondition** governs
`claroty_server_interfaces`:
`[{"field":"server_name","order":"asc"},{"field":"interface_name","order":"asc"}]`.
Both fields are confirmed members of `ServerInterfaces__sortable_fields_enum` (10 values).
The composite `(server_name, interface_name)` is the unique PK per BC-2.16.019 §Postconditions
§3 — total sort order guaranteed. Anchor: EC-016-019-007; RG-003; RG-010.

**BC-2.16.020 §Postconditions §1 sort-by postcondition** governs
`claroty_organization_zones`: `[{"field":"zone_name","order":"asc"}]`.
`zone_name` is the REQUIRED PK (confirmed in `OrganizationZones__sortable_fields_enum`).
Anchor: EC-016-020-011; RG-004.

**BC-2.16.020 §Postconditions §2 sort-by postcondition** governs
`claroty_organization_zone_policies`: `[{"field":"policy_name","order":"asc"}]`.
`policy_name` is the REQUIRED PK (confirmed in `OrganizationZonePolicies__sortable_fields_enum`).
Anchor: EC-016-020-012; RG-005.

**BC-2.16.021 §Postconditions §1 sort-by postcondition** governs
`claroty_organization_firewall_groups`: `[{"field":"firewall_group_name","order":"asc"}]`.
`firewall_group_name` is the REQUIRED PK (confirmed in `OrganizationFirewallGroups__sortable_fields_enum`).
Anchor: EC-016-021-011; RG-006.

**BC-2.16.021 §Postconditions §2 sort-by postcondition** governs
`claroty_organization_firewall_policies`: `[{"field":"policy_name","order":"asc"}]`.
`policy_name` is the REQUIRED PK (confirmed in `OrganizationFirewallGroupPolicies__sortable_fields_enum`).
Anchor: EC-016-021-012; RG-007.

**ORDER BY push-down out of scope:** `build_request` in `prism-spec-engine::pipeline` injects
only `offset` and `limit` automatically. The `sort_by` must appear as a literal JSON key
in each `body_template`. PrismQL `ORDER BY` push-down to the API is deferred to
`TD-SENSOR-SORTBY-PUSHDOWN-001`.

---

## Narrative

As a Prism operator querying Claroty xDome tables via PrismQL,
I want every paginated Claroty table to include an explicit `sort_by` array in its
API request body,
so that offset pagination is deterministic across page boundaries (no duplicate or
skipped records) regardless of the xDome API default sort order.

## Background

The Claroty xDome spec-driven adapter uses offset/limit pagination for all tables.
Without an explicit `sort_by` clause, the API sorts by its own internal default —
which for several tables uses a non-unique field (e.g., `published_date` for
vulnerabilities, `priority` for zones). When two records share the same default-sort
field value, their relative order is undefined; different API calls may return them
in different positions. This causes duplicate or skipped records across offset page
boundaries.

The conformance audit (`.factory/analysis/endpoint-conformance-audit-2026-09-02.md`,
D-001..D-007) identified 7 tables with this defect:

| Audit ID | Table | Non-unique API default | Proposed sort_by |
|----------|-------|----------------------|-----------------|
| D-001 | vulnerabilities | `published_date desc` | `adjusted_vulnerability_score desc` + `name asc` |
| D-002 | audit_logs | undefined (generic SortClause) | `timestamp asc` + `id asc` (id UNVERIFIED — see AC-002) |
| D-003 | server_interfaces | `server_name asc` (partial — non-unique alone) | `server_name asc` + `interface_name asc` |
| D-004 | organization_zones | `priority asc` (non-unique) | `zone_name asc` |
| D-005 | organization_zone_policies | `matching_devices asc` (non-unique) | `policy_name asc` |
| D-006 | organization_firewall_groups | `priority asc` (non-unique) | `firewall_group_name asc` |
| D-007 | organization_firewall_policies | `matching_devices asc` (non-unique) | `policy_name asc` |

**The fix is confined to `claroty.sensor.toml`** — 7 `body_template` string edits. No
spec-engine production code changes are required. The BC amendments (product-owner
leg) are already complete (BC-2.16.015, BC-2.16.013, BC-2.16.019, BC-2.16.020,
BC-2.16.021, all active 2026-09-02).

**Live validation note:** The `audit_logs` `id` tiebreaker is UNVERIFIED at live-API level
(the `audit_logs` endpoint uses the generic `SortClause` with no field enum, unlike the
other 6 tables). If the live xDome API rejects `id` (4xx) OR silently ignores it
(compound sort indistinguishable from timestamp-only), the implementer MUST fall back to
`[{"field":"timestamp","order":"asc"}]` alone, amend BC-2.16.013 and
`claroty.sensor.toml` in the same burst, and update both RG-002 and RG-009 (see AC-002
and Task 7).

---

## Behavioral Contracts

| BC | Title | Version | Role in this story |
|----|-------|---------|-------------------|
| BC-2.16.015 | Claroty xDome Vulnerability Findings Table — Queryable Surface and OCSF vulnerability_finding Mapping | v2.1 | §Postconditions §1 sort-by postcondition: `[adjusted_vulnerability_score desc, name asc]`; `name` is best-available tiebreaker (not unique); EC-016-015-009 pagination determinism with accepted residual for identical-score + identical-name bounded tie case; anchors RG-001 + RG-008 |
| BC-2.16.013 | Bundled Sensor Spec Authoring and DTU-Parity Verification — 4 Initial Sensors | v1.44 | §Postconditions §1 `audit_logs` sort-by postcondition (amended v1.44): preferred `[timestamp asc, id asc]`; fallback `[timestamp asc]` if `id` rejected (4xx) OR silently ignored; residual non-determinism accepted; EC-016-013-011 pagination determinism; both RG-002 (parameterized) + RG-009 must be updated on fallback adoption |
| BC-2.16.019 | Claroty Server Interfaces Table | v1.3 | §Postconditions §1 sort-by postcondition: `[server_name asc, interface_name asc]`; EC-016-019-007 pagination determinism (composite PK guarantee); anchors RG-003 + RG-010 |
| BC-2.16.020 | Claroty Org Zone Domain | v1.3 | §Postconditions §1 zones sort-by postcondition: `[zone_name asc]`; EC-016-020-011; §Postconditions §2 zone_policies sort-by: `[policy_name asc]`; EC-016-020-012; anchors RG-004 + RG-005 |
| BC-2.16.021 | Claroty Org Firewall Domain | v1.3 | §Postconditions §1 firewall_groups sort-by: `[firewall_group_name asc]`; EC-016-021-011; §Postconditions §2 firewall_policies sort-by: `[policy_name asc]`; EC-016-021-012; anchors RG-006 + RG-007 |

---

## Acceptance Criteria

### AC-001: `claroty_vulnerabilities` body_template contains the deterministic sort_by array (traces to BC-2.16.015 postcondition 1 sort-by postcondition; EC-016-015-009)

The `fetch_vulnerabilities` step in `claroty.sensor.toml` MUST have a `body_template`
that contains the JSON key `"sort_by"` with value
`[{"field":"adjusted_vulnerability_score","order":"desc"},{"field":"name","order":"asc"}]`
as a literal string embedded in the template.

The `sort_by` array MUST appear alongside the existing `"fields"` array — it is
additive; the fields projection is unchanged (18 fields per BC-2.16.015 §Postconditions §1).

The `name` tiebreaker (mapping to `finding_info.title`, presence-required but NOT provably
unique) is the **best-available sortable tiebreaker** — the opaque PK `id` is NOT a member
of `Vulnerability__sortable_fields_enum` and cannot be used. **Accepted residual
non-determinism (symmetric with the `audit_logs` accepted residual in BC-2.16.013
§Sort-by postcondition):** where two rows share BOTH the same `adjusted_vulnerability_score`
AND the same `name` (title), page-boundary ordering between them is not guaranteed. This
residual is bounded (ties require identical score AND identical title) and is further mitigated
by risk-primary ordering and the DI-019 10K cap. Accepted per BC-2.16.015 §Postconditions §1,
not a defect (EC-016-015-009).

**Tests:** RG-001 (`test_rg_vulnerabilities_sort_by_in_request_body`),
RG-008 (`test_rg_vulnerabilities_sort_by_tiebreaker_is_best_available_field`).

### AC-002: `claroty_audit_logs` body_template contains sort_by alongside filter_by, with id tiebreaker and documented fallback (traces to BC-2.16.013 postcondition 1 audit_logs sort-by postcondition; EC-016-013-011)

The `fetch_audit_logs` step in `claroty.sensor.toml` MUST have a `body_template`
that contains BOTH `"filter_by"` AND `"sort_by"` keys. The `"filter_by"` key MUST
NOT be removed or replaced by `"sort_by"` — the template extends the existing bounded
push-down template.

The preferred `sort_by` value is
`[{"field":"timestamp","order":"asc"},{"field":"id","order":"asc"}]`.

**`id` tiebreaker live-validation obligation (OBSERVED ORDERING DETERMINISM required):**
Before pushing the PR, the implementer MUST verify ordering determinism directly —
e.g., stable, gap/dup-free row ordering across a page boundary, or that the compound
form produces pagination stability demonstrably distinct from the timestamp-only form.
A 2xx HTTP response alone is NOT evidence that `id` is honored; the live xDome API
uses `SortClause` (free-form, no field enum), meaning it will return HTTP 200 whether
`id` is honored or silently ignored.

If live validation demonstrates `id` is honored (compound sort creates a total order
distinct from timestamp-only) → keep `[{"field":"timestamp","order":"asc"},
{"field":"id","order":"asc"}]` as the TOML `body_template` sort form.

If `id` is rejected (4xx) OR silently ignored (compound sort indistinguishable from
timestamp-only) → the implementer MUST:
1. Change the template to the fallback: `[{"field":"timestamp","order":"asc"}]`
2. Amend BC-2.16.013 and `claroty.sensor.toml` in the same burst documenting the outcome
3. Update BOTH RG-002 (parameterized to accept either the compound or the fallback form)
   AND RG-009 (to assert the fallback form as the confirmed canonical form)

**Residual non-determinism (accepted if fallback adopted):** `audit_logs` exposes no
unique sortable tiebreaker in its documented `SortClause`/`GetAuditLogParameters` field
set (`category`, `action`, `user_display_name`, `note`, `timestamp`, `details`), so
full total-ordering is not achievable via the API contract; the 7-day time-window filter
bounds the practical blast radius; this residual is accepted, not a defect.

**Tests:** RG-002 (`test_rg_audit_logs_sort_by_in_request_body`),
RG-009 (`test_rg_audit_logs_sort_by_id_tiebreaker_or_fallback`).

### AC-003: `claroty_server_interfaces` body_template contains composite sort_by (server_name asc, interface_name asc) (traces to BC-2.16.019 postcondition 1 sort-by postcondition; EC-016-019-007)

The `fetch_server_interfaces` step in `claroty.sensor.toml` MUST have a `body_template`
that contains `"sort_by": [{"field":"server_name","order":"asc"},{"field":"interface_name","order":"asc"}]`
as a literal string embedded in the template.

Both `server_name` and `interface_name` are confirmed members of
`ServerInterfaces__sortable_fields_enum` (10 values). The composite
`(server_name, interface_name)` is the unique PK per BC-2.16.019 §Postconditions §3,
making the sort order total and guaranteeing deterministic page boundaries.

The `sort_by` array MUST contain exactly 2 elements in this order:
`server_name asc` first, `interface_name asc` second.

**Tests:** RG-003 (`test_rg_server_interfaces_sort_by_in_request_body`),
RG-010 (`test_rg_server_interfaces_composite_key_both_present`).

### AC-004: `claroty_organization_zones` body_template contains sort_by `[zone_name asc]` (traces to BC-2.16.020 postcondition 1 sort-by postcondition; EC-016-020-011)

The `fetch_organization_zones` step in `claroty.sensor.toml` MUST have a `body_template`
that contains `"sort_by": [{"field":"zone_name","order":"asc"}]` as a literal string
embedded in the template.

`zone_name` is confirmed in `OrganizationZones__sortable_fields_enum` (10 values) and
is the REQUIRED PK for `claroty_organization_zones` per BC-2.16.020 §Postconditions §3
— single-field sort is sufficient for total order.

**Test:** RG-004 (`test_rg_organization_zones_sort_by_in_request_body`).

### AC-005: `claroty_organization_zone_policies` body_template contains sort_by `[policy_name asc]` (traces to BC-2.16.020 postcondition 2 sort-by postcondition; EC-016-020-012)

The `fetch_organization_zone_policies` step in `claroty.sensor.toml` MUST have a
`body_template` that contains `"sort_by": [{"field":"policy_name","order":"asc"}]`
as a literal string embedded in the template.

`policy_name` is confirmed in `OrganizationZonePolicies__sortable_fields_enum` (11 values)
and is the REQUIRED PK for `claroty_organization_zone_policies` per BC-2.16.020
§Postconditions §3.

**Test:** RG-005 (`test_rg_organization_zone_policies_sort_by_in_request_body`).

### AC-006: `claroty_organization_firewall_groups` body_template contains sort_by `[firewall_group_name asc]` (traces to BC-2.16.021 postcondition 1 sort-by postcondition; EC-016-021-011)

The `fetch_organization_firewall_groups` step in `claroty.sensor.toml` MUST have a
`body_template` that contains
`"sort_by": [{"field":"firewall_group_name","order":"asc"}]` as a literal string
embedded in the template.

`firewall_group_name` is confirmed in `OrganizationFirewallGroups__sortable_fields_enum`
(10 values) and is the REQUIRED PK for `claroty_organization_firewall_groups` per
BC-2.16.021 §Postconditions §3.

**Test:** RG-006 (`test_rg_organization_firewall_groups_sort_by_in_request_body`).

### AC-007: `claroty_organization_firewall_policies` body_template contains sort_by `[policy_name asc]` (traces to BC-2.16.021 postcondition 2 sort-by postcondition; EC-016-021-012)

The `fetch_organization_firewall_policies` step in `claroty.sensor.toml` MUST have a
`body_template` that contains `"sort_by": [{"field":"policy_name","order":"asc"}]`
as a literal string embedded in the template.

`policy_name` is confirmed in
`OrganizationFirewallGroupPolicies__sortable_fields_enum` (11 values) and is the
REQUIRED PK for `claroty_organization_firewall_policies` per BC-2.16.021
§Postconditions §3.

**Test:** RG-007 (`test_rg_organization_firewall_policies_sort_by_in_request_body`).

---

## Red Gate Tests

| ID | Test name | Test type | What it gates |
|----|-----------|-----------|---------------|
| RG-001 | `test_rg_vulnerabilities_sort_by_in_request_body` | Unit — `SpecLoader::parse` on `claroty.sensor.toml`; assert `body_template` string for `fetch_vulnerabilities` contains `"sort_by"` key with `adjusted_vulnerability_score` desc and `name` asc | AC-001: vulnerabilities body_template emits the contracted sort_by array (traces to BC-2.16.015 postcondition 1 + EC-016-015-009) |
| RG-002 | `test_rg_audit_logs_sort_by_in_request_body` | Unit — `SpecLoader::parse`; assert `body_template` for `fetch_audit_logs` contains `"sort_by"` key in either the preferred compound form (`timestamp asc` + `id asc`) or the timestamp-only fallback form — parameterized to accept either after live-validation outcome | AC-002: audit_logs body_template contains sort_by (traces to BC-2.16.013 postcondition 1 + EC-016-013-011); coupled to fallback adoption alongside RG-009 |
| RG-003 | `test_rg_server_interfaces_sort_by_in_request_body` | Unit — `SpecLoader::parse`; assert `body_template` for `fetch_server_interfaces` contains `"sort_by"` key | AC-003: server_interfaces sort_by present (traces to BC-2.16.019 postcondition 1 + EC-016-019-007) |
| RG-004 | `test_rg_organization_zones_sort_by_in_request_body` | Unit — `SpecLoader::parse`; assert `body_template` for `fetch_organization_zones` contains `"sort_by"` with `zone_name asc` | AC-004: organization_zones sort_by (traces to BC-2.16.020 postcondition 1 + EC-016-020-011) |
| RG-005 | `test_rg_organization_zone_policies_sort_by_in_request_body` | Unit — `SpecLoader::parse`; assert `body_template` for `fetch_organization_zone_policies` contains `"sort_by"` with `policy_name asc` | AC-005: zone_policies sort_by (traces to BC-2.16.020 postcondition 2 + EC-016-020-012) |
| RG-006 | `test_rg_organization_firewall_groups_sort_by_in_request_body` | Unit — `SpecLoader::parse`; assert `body_template` for `fetch_organization_firewall_groups` contains `"sort_by"` with `firewall_group_name asc` | AC-006: firewall_groups sort_by (traces to BC-2.16.021 postcondition 1 + EC-016-021-011) |
| RG-007 | `test_rg_organization_firewall_policies_sort_by_in_request_body` | Unit — `SpecLoader::parse`; assert `body_template` for `fetch_organization_firewall_policies` contains `"sort_by"` with `policy_name asc` | AC-007: firewall_policies sort_by (traces to BC-2.16.021 postcondition 2 + EC-016-021-012) |
| RG-008 | `test_rg_vulnerabilities_sort_by_tiebreaker_is_best_available_field` | Unit — parse spec; extract `body_template` for `fetch_vulnerabilities`; parse the JSON embedded literal; assert `sort_by` array last element has `"field": "name"` and `"order": "asc"`; assert `name` is a member of `Vulnerability__sortable_fields_enum` (hardcode known values) | AC-001 structural: `name` (`finding_info.title`) is the best-available sortable tiebreaker — `id` not in `Vulnerability__sortable_fields_enum`; accepted residual non-determinism for identical-score + identical-name bounded tie case documented (traces to BC-2.16.015 §Postconditions §1 + EC-016-015-009) |
| RG-009 | `test_rg_audit_logs_sort_by_id_tiebreaker_or_fallback` | Unit — parse spec; assert `body_template` for `fetch_audit_logs` contains BOTH `"filter_by"` AND `"sort_by"` substrings (coexistence); AND assert `sort_by` array contains `"timestamp"` entry; AND assert (if the preferred form): `"id"` entry present or (if the fallback form is used after live-validation confirms silent-ignore or 4xx rejection): `"id"` absent but `"timestamp"` remains the only sort field | AC-002: filter_by preserved (not replaced by sort_by) AND id-tiebreaker or timestamp-only fallback (traces to BC-2.16.013 postcondition 1 audit_logs sort-by + EC-016-013-011) |
| RG-010 | `test_rg_server_interfaces_composite_key_both_present` | Unit — parse spec; extract `body_template` for `fetch_server_interfaces`; assert `sort_by` array contains exactly 2 elements: element 0 has `"field": "server_name"` and element 1 has `"field": "interface_name"`, both with `"order": "asc"` | AC-003 structural: composite key `(server_name, interface_name)` both present in correct order — uniqueness guarantee for total sort order (traces to BC-2.16.019 postcondition 1 + EC-016-019-007) |

**BC-5.38.001 density check:** 10 Red Gate tests (RG-001 through RG-010) /
7 acceptance criteria = 1.43 RGTs per AC. PASS (≥ 0.5 required). The 3 extra tests
(RG-008, RG-009, RG-010) are structural assertions that verify tiebreaker field presence
and coexistence invariants beyond the basic body_template presence checks.

**RG-number reconciliation note:** The design doc (`.factory/analysis/claroty-sortby-design-2026-09-02.md` §5)
assigned RG-009 = `test_rg_server_interfaces_composite_key_both_present` and
RG-010 = `test_rg_audit_logs_body_template_contains_sort_by_and_filter_by`. The
product-owner's BC amendments (BC-2.16.013 §Sort-by postcondition changelog) explicitly anchor RG-009
as `test_rg_audit_logs_sort_by_id_tiebreaker_or_fallback`. This story follows the
BC anchor (BCs supersede the design doc as a more recent authoritative artifact per
CLAUDE.md §Source-of-Truth Precedence). The user's story-materialization directive
(2026-09-02) confirms: RG-009 = audit_logs id-tiebreaker/coexistence; RG-010 =
server_interfaces composite-key structural assertion.

---

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `claroty.sensor.toml` — 7 `body_template` string edits | `crates/prism-sensors/specs/claroty.sensor.toml` | Static data (TOML spec) — no code change |
| TOML spec parse validation | `crates/prism-spec-engine/src/spec_parser.rs §spec_parser` | Pure (TOML deserialization; no I/O) — invoked only in tests |
| OffsetLimit POST-body injection | `crates/prism-spec-engine/src/pipeline.rs §PipelineExecutor::execute` | Effectful — unchanged; `sort_by` travels as a literal in the TOML string, not via injection |

Architecture section references:
- `architecture/module-decomposition.md` §SS-01 Sensor Adapters (`prism-sensors`; `claroty.sensor.toml`)
- `architecture/module-decomposition.md` §SS-16 Spec Engine (`prism-spec-engine`; `spec_parser`)
- `.factory/analysis/claroty-sortby-design-2026-09-02.md` §2 (per-table design decisions and OpenAPI sort enum evidence)
- BC-2.16.002 `pipeline.rs::build_request` injects only `offset` and `limit`; `sort_by` must appear as a literal in `body_template` (not auto-injected)

## Purity Classification

- **Pure (no I/O, deterministic):** `SpecLoader::parse` (TOML deserialization); all 10 RG tests are
  spec-parse unit tests that read `claroty.sensor.toml` from disk and make string assertions;
  no network, no DTU, no external services required.
- **Effectful (I/O, network):** The `body_template` literal change propagates to
  `PipelineExecutor::execute` at runtime (HTTP POST body to xDome), but the spec-engine
  production code itself is not modified. No effectful test paths in this story.

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | `audit_logs` `id` sort field rejected (4xx) OR silently ignored by xDome live API (HTTP 200 but compound sort indistinguishable from timestamp-only) | Implementer MUST fall back to `[{"field":"timestamp","order":"asc"}]` alone; amend BC-2.16.013 and `claroty.sensor.toml` in the same burst; update BOTH RG-002 (parameterized to accept either form) AND RG-009 to assert the fallback form; residual non-determinism accepted (no unique tiebreaker in documented SortClause field set — `category`, `action`, `user_display_name`, `note`, `timestamp`, `details`; 7-day window bounds blast radius) (traces to BC-2.16.013 §Sort-by postcondition §id tiebreaker caveat + EC-016-013-011 fallback clause) |
| EC-002 | `sort_by` key accidentally replaces `filter_by` in audit_logs body_template | `build_claroty_audit_filter_by` value is never injected; xDome returns all audit events without the 7-day time window; silent data inflation. RG-009 explicitly asserts BOTH keys are present to guard against this. |
| EC-003 | `sort_by` inserted inside the `fields` array literal (wrong placement) | JSON parse error at the `build_request` stage (body_template is not valid JSON after variable substitution). RG-001..007 parse the body_template string to detect malformed JSON structure. |
| EC-004 | `vulnerabilities` `adjusted_vulnerability_score` field name misspelled in `sort_by` | xDome returns 400 (ValidatingSortClause enum validation failure). RG-001 asserts the exact field name string. |
| EC-005 | ORDER BY push-down (future: `TD-SENSOR-SORTBY-PUSHDOWN-001`) conflicts with the embedded `sort_by` literal | Explicitly OUT OF SCOPE for this story. When push-down is implemented, the body_template `sort_by` literal will need to be replaced with a variable reference. Document in the tech debt tracker at that time. |

---

## Token Budget Estimate

| Item | Estimated tokens |
|------|-----------------|
| This story spec | ~6,000 |
| `crates/prism-sensors/specs/claroty.sensor.toml` (all 7 affected tables) | ~7,000 |
| BC files (5 BCs: BC-2.16.015, BC-2.16.013, BC-2.16.019, BC-2.16.020, BC-2.16.021) | ~15,000 |
| Design doc (`.factory/analysis/claroty-sortby-design-2026-09-02.md`) | ~5,000 |
| `crates/prism-spec-engine/src/spec_parser.rs` (SpecLoader, FetchStep, body_template parsing section) | ~3,000 |
| Test file `defect_claroty_sortby_determinism_001.rs` (10 RGTs) | ~4,000 |
| **Total estimate** | **~40,000 tokens** |

Well within 20-30% of a 200K window. If context is tight, read only the affected
`[[tables.steps]]` blocks in `claroty.sensor.toml` (7 sections of ~15 lines each)
rather than the full file.

---

## Tasks

**SAC-1 mandate: test-authoring tasks MUST precede implementation tasks (red-then-green).**
All RG-001..RG-010 tests MUST be written and verified to FAIL before any `claroty.sensor.toml`
edit is made. The TOML edit is the implementation step that turns them green.

- [ ] **Task 1 (Red Gate — write failing tests first): Write RG-001..RG-007** in
  `crates/prism-sensors/tests/defect_claroty_sortby_determinism_001.rs`.
  Each test: load `claroty.sensor.toml` via `SpecLoader::parse`; find the named table
  by `table_name`; find the step by `name`; assert the `body_template` string contains
  the expected `"sort_by"` substring (e.g., `assert!(body_template.contains("\"sort_by\""))`).
  Also assert the specific field name is present (e.g.,
  `assert!(body_template.contains("adjusted_vulnerability_score"))` for RG-001).
  All 7 MUST FAIL before Task 5 (TOML edit not yet made; current `body_template` strings
  have no `sort_by` key). Confirm each test is genuinely failing with a meaningful
  assertion error (not skipping or panicking).

- [ ] **Task 2 (Red Gate — write failing tests first): Write RG-008** in the same test file.
  `test_rg_vulnerabilities_sort_by_tiebreaker_is_best_available_field`: parse the
  `body_template` for `fetch_vulnerabilities`; extract the JSON substring between
  `"sort_by":` and the closing `]`; parse it as `Vec<serde_json::Value>`; assert the last
  element has `"field": "name"` and `"order": "asc"`; assert `name` is a member of
  `Vulnerability__sortable_fields_enum` (hardcode known values). MUST FAIL before Task 5.

- [ ] **Task 3 (Red Gate — write failing tests first): Write RG-009 and RG-010** in the
  same test file.

  **RG-009** (`test_rg_audit_logs_sort_by_id_tiebreaker_or_fallback`): parse
  `body_template` for `fetch_audit_logs`; assert it contains BOTH the substring
  `"filter_by"` AND the substring `"sort_by"` (coexistence guard); assert it contains
  `"timestamp"` in the sort_by value; assert it contains either `"id"` (preferred) or
  exclusively `"timestamp"` (fallback — only valid after live-validation confirms silent-
  ignore or 4xx rejection). MUST FAIL before Task 5.

  **RG-010** (`test_rg_server_interfaces_composite_key_both_present`): parse
  `body_template` for `fetch_server_interfaces`; extract and parse the `sort_by` JSON
  array; assert exactly 2 elements present; assert element 0 has `"field": "server_name"`
  and element 1 has `"field": "interface_name"`, both with `"order": "asc"`.
  MUST FAIL before Task 5.

- [ ] **Task 4 (Red Gate verification): Run all 10 tests and confirm all FAIL.**
  `cargo nextest run -p prism-sensors -E 'test(rg_)' --no-fail-fast`
  Expected: all 10 tests fail with assertion errors (missing `"sort_by"` substring).
  If any test PASSES at this point, the TOML already has the correct value (unexpected
  — investigate before proceeding).

- [ ] **Task 5 (Implementation — TOML edit): Add `sort_by` to all 7 `body_template` strings**
  in `crates/prism-sensors/specs/claroty.sensor.toml`.

  For each table's `[[tables.steps]]` block, extend the `body_template` JSON object
  to include `"sort_by": [...]` **AFTER** the existing `"fields"` array (or after
  `"filter_by"` for audit_logs). The exact JSON values are:

  - **vulnerabilities** (`fetch_vulnerabilities`): append
    `"sort_by": [{"field":"adjusted_vulnerability_score","order":"desc"},{"field":"name","order":"asc"}]`
    to the existing body_template (inside the JSON object literal, after the `"fields"` array).

  - **audit_logs** (`fetch_audit_logs`): change
    `'{"filter_by": ${query.filter._claroty_audit_filter_by}}'` to
    `'{"filter_by": ${query.filter._claroty_audit_filter_by}, "sort_by": [{"field":"timestamp","order":"asc"},{"field":"id","order":"asc"}]}'`
    (PRESERVE `filter_by` — do NOT replace it).

  - **server_interfaces** (`fetch_server_interfaces`): append
    `"sort_by": [{"field":"server_name","order":"asc"},{"field":"interface_name","order":"asc"}]`
    after the `"fields"` array.

  - **organization_zones** (`fetch_organization_zones`): append
    `"sort_by": [{"field":"zone_name","order":"asc"}]` after the `"fields"` array.

  - **organization_zone_policies** (`fetch_organization_zone_policies`): append
    `"sort_by": [{"field":"policy_name","order":"asc"}]` after the `"fields"` array.

  - **organization_firewall_groups** (`fetch_organization_firewall_groups`): append
    `"sort_by": [{"field":"firewall_group_name","order":"asc"}]` after the `"fields"` array.

  - **organization_firewall_policies** (`fetch_organization_firewall_policies`): append
    `"sort_by": [{"field":"policy_name","order":"asc"}]` after the `"fields"` array.

  After each edit, verify `SpecLoader::parse` returns `Ok` on the modified TOML.

- [ ] **Task 6 (Red Gate verification — all 10 tests turn GREEN):**
  `just iter prism-sensors`
  All 10 RG tests MUST now pass. Confirm no existing prism-sensors tests regressed.

- [ ] **Task 7 (audit_logs id live-validation — BLOCKING before push):**
  If live access to the monroe xDome instance is available, validate OBSERVED ORDERING
  DETERMINISM — NOT merely the HTTP response code. A 200 response does NOT confirm
  `id` is honored because `SortClause` performs no server-side field-name validation
  (the `audit_logs` endpoint returns HTTP 200 whether `id` is honored or silently ignored).
  Run a page-boundary check:
  `curl -X POST <CLAROTY_INSTANCE_URL>/api/v1/audit_log/get \
    -H "Authorization: Bearer <token>" \
    -H "Content-Type: application/json" \
    -d '{"sort_by": [{"field":"timestamp","order":"asc"},{"field":"id","order":"asc"}], "filter_by": {"field": "timestamp", "operation": "greater_or_equal", "value": "<now-7d ISO-8601>"}, "limit": 10, "offset": 0}'`
  Then repeat with `"offset": 10`. Verify: the row at position 10 in the first page
  equals the row at position 0 in the second page, confirming stable ordering across
  the page boundary. Alternatively, compare the compound-sort result against a
  timestamp-only result to confirm distinct ordering.
  - If `id` is honored (observed stable ordering distinct from timestamp-only): the
    preferred compound form is confirmed. RG-009 passes as-is. RG-002 passes as-is.
  - If `id` is rejected (4xx) OR silently ignored (compound sort indistinguishable
    from timestamp-only): change audit_logs `sort_by` to
    `[{"field":"timestamp","order":"asc"}]` alone, amend BC-2.16.013 and
    `claroty.sensor.toml` in the same burst, and update BOTH RG-002 (parameterized
    to accept either form) AND RG-009 (to assert the fallback form as canonical).
    Document the outcome in the PR description. **Residual non-determinism is
    accepted** — `audit_logs` has no unique sortable tiebreaker in its documented
    field set (`category`, `action`, `user_display_name`, `note`, `timestamp`,
    `details`); the 7-day window bounds the blast radius.
  If live access is unavailable at implementation time: proceed with the preferred form,
  flag the unverified risk explicitly in the PR description, and mark for validation
  in the story-level holdout scenarios.

- [ ] **Task 8 (SAP-1 self-check):**
  Confirm no new `tracing::*!(event_type = ...)` emissions are added (TOML-only change).
  If any tracing emission appears in test helpers, add a BC-2.16.002 catalog row per PG-LP11-001.

- [ ] **Task 9 (SAP-2 self-check — N/A for this story):**
  This story modifies `claroty.sensor.toml` by adding `sort_by` to body_templates.
  `sort_by` is a request parameter; it does not appear in the DTU response struct.
  SAP-2 (DTU↔TOML schema parity) does not apply to request-body fields. No DTU
  changes required. Record: SAP-2 status = N/A (request-body only; no response-shape impact).

- [ ] **Task 10 (Final gate):**
  `just check` (full workspace).
  Confirm all 10 RG tests pass. Confirm no new `unwrap()`/`expect()` on `Result`
  in production code (none should exist — this is a TOML-only change + tests).
  Confirm `claroty.sensor.toml` still parses as valid TOML via `SpecLoader::parse`
  (RG-001..007 provide this). After `just check` passes, proceed to story-level
  holdout gate (BLOCKING before push to origin).

---

## Previous Story Intelligence

1. **S-CLAROTY-AUDITLOG-TIMEBOX-001 (merged):** Established the bounded `filter_by`
   push-down for `audit_logs`. The current `body_template` is
   `'{"filter_by": ${query.filter._claroty_audit_filter_by}}'`. Task 5 MUST extend
   this template, not replace it. The `filter_by` injection is what gives `audit_logs`
   its 7-day default window — removing it would cause unbounded queries.

2. **S-CLAROTY-ORGPOLICY-001 (merged PR #249):** Established `organization_zones`,
   `zone_policies`, `firewall_groups`, and `firewall_policies` body_templates. These
   currently have `"fields": [...]` projections. The `sort_by` addition is additive.
   Read the current TOML blocks carefully before editing to identify the exact
   `body_template` string boundaries.

3. **S-CLAROTY-SERVERS-001 (merged PR #248):** Established `server_interfaces`
   body_template with the 10-field `"fields"` projection. Note the composite PK
   convention — `interface_name` is Tier-2 (raw_extensions) but is a PK element.
   RG-010 verifies both fields are present in the `sort_by` array.

4. **S-CLAROTY-VULNS-001 (merged PR #245):** Established `vulnerabilities`
   body_template with 18-field projection. The `body_template` is a single-line
   TOML string (TOML single-quoted literals do not support backslash line-continuation).
   When adding `sort_by`, ensure the extended string remains a single-line value.

5. **TOML body_template syntax guidance:** All Claroty `body_template` values are
   TOML single-quoted strings (`'...'`). JSON is embedded as a literal string.
   The spec-engine treats the entire string as a template for variable interpolation
   (only `${query.filter.KEY}` patterns are substituted; literal JSON is passed as-is).
   Do NOT use TOML multiline string syntax (`'''...'''`); keep each body_template on
   one line per the established convention.

---

## Architecture Compliance Rules

From `architecture/module-decomposition.md` §SS-01 Sensor Adapters:
- `claroty.sensor.toml` is the authoritative sensor spec. The `[[tables.steps]]`
  `body_template` field is a TOML single-quoted string containing a JSON literal with
  variable placeholders. Edits to `body_template` are data changes, not code changes;
  no spec-engine production code files require modification.
- `SpecLoader::parse` validates TOML structure and field types but does NOT validate
  JSON structure inside `body_template` strings — JSON validation happens at request
  time in `PipelineExecutor::execute`. Therefore, a syntactically invalid JSON
  `body_template` would not be caught by `SpecLoader::parse` alone; RG tests MUST
  parse the embedded JSON to detect structural errors.

From `architecture/module-decomposition.md` §SS-16 Spec Engine:
- `pipeline.rs::build_request` injects ONLY `offset` and `limit` (confirmed in
  `.factory/analysis/claroty-sortby-design-2026-09-02.md §Implementation path`).
  `sort_by` is NOT auto-injected. The literal JSON must appear verbatim in
  `body_template`.

From ADR-028 §D8 (TOML spec grounding):
- `body_template` changes are spec-level changes (not code-level). They do not require
  an ADR or architect adjudication. They are covered by the owning BC's postcondition.

From the `endpoint-conformance-audit` and design doc:
- The `sort_by` field is optional in the xDome OpenAPI for all 7 tables. The API will
  accept the additional JSON key without error (for the 6 `ValidatingSortClause` tables).
  For `audit_logs` (`SortClause`), the field value is a free string with no server-side
  enum validation — `id` is unverified (see AC-002).

---

## Library & Framework Requirements

| Library | Version | Source |
|---------|---------|--------|
| `prism-spec-engine` | workspace path | `SpecLoader::parse` — called from test file to validate TOML and inspect `body_template` strings |
| `serde_json` | per workspace Cargo.toml | Used in RG-008/009/010 to parse the embedded JSON sort_by arrays from body_template strings |
| `tokio` | per workspace Cargo.toml | NOT required — all RG tests are synchronous spec-parse unit tests; no async needed |

Do NOT add new Cargo.toml production dependencies. The test file requires only
`prism-spec-engine` (for `SpecLoader`) and `serde_json` (for JSON parsing in RG-008/009/010)
— both are already workspace dependencies.

---

## File Structure Requirements

| Action | File path | Notes |
|--------|-----------|-------|
| MODIFY | `crates/prism-sensors/specs/claroty.sensor.toml` | Add `"sort_by": [...]` to `body_template` strings for: `fetch_vulnerabilities`, `fetch_audit_logs`, `fetch_server_interfaces`, `fetch_organization_zones`, `fetch_organization_zone_policies`, `fetch_organization_firewall_groups`, `fetch_organization_firewall_policies` — 7 edits total |
| CREATE | `crates/prism-sensors/tests/defect_claroty_sortby_determinism_001.rs` | Contains RG-001..RG-010: spec-parse unit tests asserting sort_by presence and structure in the 7 modified body_templates |
| POSSIBLY MODIFY | `crates/prism-sensors/Cargo.toml` | Add `[dev-dependencies]` entry for `serde_json = { workspace = true }` if not already present (needed by RG-008/009/010 JSON parsing) |

Files that MUST NOT be modified:
- `crates/prism-spec-engine/src/pipeline.rs` — no production code change needed
- `crates/prism-spec-engine/src/spec_parser.rs` — no production code change needed
- Any `prism-dtu-claroty/` file — DTU routes are not sort-order-aware in their fixture responses; no DTU changes required
- BC files in `.factory/specs/behavioral-contracts/` — BC amendments are DONE (product-owner leg complete); story-writer and implementer MUST NOT modify BC bodies

## Forbidden Dependencies

`prism-sensors` MUST NOT gain any new production dependency on `prism-dtu-claroty`
(dev-dep only for future DTU-parity tests if needed). The test file MUST NOT import
`prism-dtu-claroty` (there is no DTU dependency in this story). `prism-spec-engine`
MUST NOT gain a new dependency on `prism-sensors` (direction is
prism-sensors → prism-spec-engine, not reverse).

---

## Notes for Implementer

1. **TOML single-line body_template requirement.** All Claroty body_template values
   are single-line TOML strings. Do NOT split across lines using TOML multiline
   literal syntax (`'''...'''`). The `body_template` must remain a single-line
   single-quoted string (`'...'`). Existing stories confirm this pattern.

2. **audit_logs template extension — filter_by must be preserved.** The `fetch_audit_logs`
   step currently has `body_template = '{"filter_by": ${query.filter._claroty_audit_filter_by}}'`.
   The new template MUST be additive: `'{"filter_by": ${query.filter._claroty_audit_filter_by}, "sort_by": [...]}'`.
   Replacing `filter_by` with `sort_by` would break the 7-day time-window enforcement
   (S-CLAROTY-AUDITLOG-TIMEBOX-001), which is a regression.

3. **Live validation for audit_logs `id` tiebreaker is BLOCKING before push.**
   Task 7 is a blocking gate. A 200 HTTP response does NOT confirm `id` is honored —
   `audit_logs` uses `SortClause` (no field enum), so xDome silently ignores unknown
   sort fields rather than returning 4xx. Validate OBSERVED ORDERING DETERMINISM
   (page-boundary check or compound-vs-timestamp comparison). If `id` is rejected (4xx)
   OR silently ignored (indistinguishable ordering), use the fallback form and amend
   BC-2.16.013, `claroty.sensor.toml`, RG-002, and RG-009 in the same burst.
   Do NOT push the PR without completing live validation OR explicitly flagging
   the unverified risk in the PR description for holdout evaluation.

4. **ORDER BY push-down is explicitly OUT OF SCOPE.** TD-SENSOR-SORTBY-PUSHDOWN-001
   tracks the future work of mapping PrismQL ORDER BY clauses to the API `sort_by`
   parameter. This story only adds the static literal `sort_by` to 7 body_templates.
   Do NOT attempt to wire dynamic ORDER BY push-down in this story.

5. **Test parsing of embedded JSON.** RG-008, RG-009, and RG-010 need to parse the
   JSON sort_by array from the body_template string. The approach:
   - Locate the `"sort_by":` substring in the body_template
   - Extract the JSON array value (from `[` to matching `]`)
   - Use `serde_json::from_str::<Vec<serde_json::Value>>(extracted)` to parse
   - Assert on the parsed structure
   This pattern handles the fact that sort_by is embedded in a larger JSON object
   as a literal string value in TOML.

---

## Changelog

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.2 | 2026-09-02 | story-writer | Aligned §Authority and AC-001 to BC-2.16.015 §Postconditions §1 amended to close adversary OBS-001 propagation: replaced "provably unique CVE ID" and "total order" claims for `name` tiebreaker with accurate best-available-tiebreaker + accepted-residual language (`name` maps to `finding_info.title`, presence-required but NOT unique; opaque PK `id` not in `Vulnerability__sortable_fields_enum`; accepted residual for identical-score + identical-name bounded tie case, symmetric with BC-2.16.013 §Sort-by postcondition, mitigated by risk-primary ordering and DI-019 10K cap). Updated Behavioral Contracts table BC-2.16.015 Version cell and Role description. Renamed RG-008 test to `test_rg_vulnerabilities_sort_by_tiebreaker_is_best_available_field`. POL-39 compliance: no vX.Y BC version tokens introduced in narrative prose. |
| 1.1 | 2026-09-02 | story-writer | Aligned AC-002, Task 7, and EC-001 to BC-2.16.013 §Sort-by postcondition amended per DEFECT-CLAROTY-SORTBY-DETERMINISM-001 OBS-1: live-validation gate updated from 2xx-check to OBSERVED ORDERING DETERMINISM; silent-ignore path added alongside 4xx-rejection path in decision logic; residual non-determinism documented. LOW-1 closure: added RG-002 coupling to fallback protocol throughout (AC-002, Task 7, EC-001, §Authority, RG-002 table row, §Background live validation note). Updated Behavioral Contracts table BC-2.16.013 row to v1.44. OBS-2 closure (POL-39 depin): removed vX.Y version tokens from all narrative-prose BC citations in story body — 30 instances depinned across §Authority, §Background, AC headers, and RG table; frontmatter POL-39-exempt and unchanged. |
| 1.0 | 2026-09-02 | story-writer | Initial story creation — 7 Claroty xDome sort_by determinism fixes, RG-001..RG-010 Red Gate enumeration, AC traceability to BC-2.16.015, BC-2.16.013, BC-2.16.019, BC-2.16.020, BC-2.16.021. |
