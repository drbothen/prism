---
document_type: story
story_id: S-CLAROTY-ORGPOLICY-001
title: "Claroty xDome Org Policy Tables — 4 TOML [[tables]] blocks (claroty_organization_zones, claroty_organization_zone_policies, claroty_organization_firewall_groups, claroty_organization_firewall_policies), entity_management/3004, 8 Json cols, fw URL↔envelope-key asymmetry, live structural tests (Wave C G5)"
level: "L4"
wave: xdome-wave-c
epic_id: E-XDOME-EXPANSION
priority: P0
status: ready
# BC status: BC-2.16.020 v1.1 draft + BC-2.16.021 v1.1 draft — pre-delivery remove-uncertainty pass complete 2026-08-31; promoted to ready (D-2385).
producer: story-writer
timestamp: "2026-08-24T00:00:00Z"
version: "1.6"
modified: "2026-09-01"
phase: 3
cycle: v1.0.0-brownfield
inputs:
  - ".factory/specs/behavioral-contracts/BC-2.16.020-claroty-org-zone-domain.md"
  - ".factory/specs/behavioral-contracts/BC-2.16.021-claroty-org-firewall-domain.md"
  - ".factory/objectives/xdome-endpoint-expansion-plan.md"
  - ".factory/objectives/xdome-v1-validation/endpoint-schema-extract.md"
  - ".factory/objectives/xdome-v1-validation/endpoint-spike-findings.md"
  - ".factory/specs/architecture/decisions/ADR-058-v1-column-naming-col-name-as-arrow-field-identifier.md"
  - "crates/prism-sensors/specs/claroty.sensor.toml"
input-hash: "b68ddba"
# input-hash: run `compute-input-hash <this-file> --update` after state-manager commits
traces_to: "BC-2.16.020"
# traces_to covers primary BC (Zone Domain); BC-2.16.021 (Firewall Domain) is the companion BC; both wired via behavioral_contracts
points: 8
estimated_days: 2
tdd_mode: strict
subsystems: [SS-01, SS-16]
# Subsystem anchor justifications (ARCH-INDEX Subsystem Registry):
#   SS-01 (Sensor Adapters) owns this story's scope because
#     `crates/prism-sensors/specs/claroty.sensor.toml` — the TOML spec file being
#     modified — lives in the prism-sensors crate, which is listed under SS-01 per
#     ARCH-INDEX. All four `[[tables]]` blocks (organization_zones, organization_zone_policies,
#     organization_firewall_groups, organization_firewall_policies) are sensor-adapter
#     configuration artifacts, exactly the surface SS-01 governs.
#   SS-16 (Spec Engine) owns this story's scope because
#     `crates/prism-spec-engine/src/spec_parser.rs` must parse all four new [[tables]]
#     blocks without validation error. RG-001/RG-002/RG-009/RG-010/RG-015/RG-016/RG-021/RG-022
#     are spec-parser and ColumnSpec unit tests that exercise SS-16's deserialization and
#     column-mapping surfaces. SS-16 is the canonical owner of prism-spec-engine per
#     ARCH-INDEX Subsystem Registry.
target_module: prism-sensors
crates_touched: [prism-sensors, prism-bin]
# crates_touched:
#   prism-sensors: claroty.sensor.toml — four new [[tables]] blocks
#   prism-bin: authoritative RG-003/RG-012/RG-020/RG-024 end-to-end E-QUERY-038 gates + RG-027 wire-shape
capabilities:
  - CAP-029
behavioral_contracts:
  - BC-2.16.020
  # BC-2.16.020 v1.0 — Claroty xDome Organization Zone Domain: TOML table contracts
  # for claroty_organization_zones (§PC1) and claroty_organization_zone_policies (§PC2);
  # Tier-1/Tier-2 column classifications (§PC3 / §PC4); PK rationale for both tables (§PC5);
  # Json column serialization behavior (§PC6); SAP-2 N/A (§PC7);
  # EC-016-020-001..010 edge cases. ACs 001–014 trace to this BC.
  - BC-2.16.021
  # BC-2.16.021 v1.0 — Claroty xDome Organization Firewall Domain: TOML table contracts
  # for claroty_organization_firewall_groups (§PC1) and claroty_organization_firewall_policies (§PC2);
  # URL vs envelope key asymmetry invariant (fw_groups path uses _fw_ abbreviation; envelope uses
  # full organization_firewall_ spelling); Tier-1/Tier-2 column classifications (§PC3 / §PC4);
  # PK rationale (§PC5); Json column serialization (§PC6); SAP-2 N/A (§PC7);
  # EC-016-021-001..010 edge cases. ACs 015–026 trace to this BC.
verification_properties: []
holdout_scenarios:
  - HS-028
# holdout_scenarios: HS-028 registered by PO at BC-2.16.020 §Changelog and BC-2.16.021 §Changelog
# (4 P0 hidden scenarios covering all four tables from this story).
# Scenarios live under the holdout-scenarios directory that test-writer and implementer
# MUST NOT read (contamination control). The story-level holdout gate (human-approved
# 2026-07-13) is BLOCKING before demo recording / push to origin.
depends_on: []
# depends_on justification: No delivery-time scheduling dependency remains. The four
# org-policy tables are independent POST-for-read queries; they do not join to prior
# xDome tables in this first-cut spec. S-ADR058-OCSF-ROUTING-001 (which activated
# ocsf_column_naming=true) is already MERGED (PR #242, develop@3f1e66179). The entity_management
# arm (class_uid 3004) is already present in class_selector.rs (spike-findings §Overall Verdict).
# No Wave A/B/C VULNS/OT-EVENTS/DEVVULNREL/SERVERS blocks exist in the committed TOML at
# develop@3f1e66179; this story does not depend on them merging first.
blocks: []
acceptance_criteria_count: 26
risk: MEDIUM
# Risk justification:
#   All four tables have no DTU; live Variant-1 tests are #[ignore]'d until live validation
#   against monroe. The fw URL↔envelope-key asymmetry (BC-2.16.021 §Invariants) requires
#   careful spec authoring: the TOML path_template for firewall_groups uses /api/v1/organization_fw_groups/
#   (abbreviated _fw_) but the response_path must be $.organization_firewall_groups (full spelling).
#   Using the abbreviated key in response_path produces empty results silently (EC-016-021-006).
#   SAP-2 DTU-parity probe is N/A per D-2200 for all four tables.
assumption_validations: []
risk_mitigations: []
---

# S-CLAROTY-ORGPOLICY-001: Claroty xDome Org Policy Tables — 4 TOML Blocks + Live Structural Tests

## Authority

**BC-2.16.020 §Postconditions §1 — TOML Table Contract (claroty_organization_zones)**
governs the exact `[[tables]]` block: `table_name = "organization_zones"` (bare name;
`{sensor_id}_{table_name}` = `claroty_organization_zones` registered/queryable name),
`ocsf_class = "entity_management"`, step name `"fetch_organization_zones"`, `method = "POST"`,
`path_template = "/api/v1/organization_zones/"`, `response_path = "$.organization_zones"`,
pagination `type = "offset_limit"` / `page_size = 1000`, and the 11-field `body_template`.

**BC-2.16.020 §Postconditions §2 — TOML Table Contract (claroty_organization_zone_policies)**
governs the exact `[[tables]]` block: `table_name = "organization_zone_policies"` (bare name;
`{sensor_id}_{table_name}` = `claroty_organization_zone_policies` registered/queryable name),
`ocsf_class = "entity_management"`, step `"fetch_organization_zone_policies"`, `method = "POST"`,
`path_template = "/api/v1/organization_zone_policies/"`, `response_path = "$.organization_zone_policies"`,
pagination `type = "offset_limit"` / `page_size = 1000`, and the 13-field `body_template`.
**Datetime field name asymmetry (§PC2 note):** zones uses `last_update` (no trailing 'd');
zone_policies uses `last_updated` (with trailing 'd'). Using the wrong name silently
produces an empty column (EC-016-020-009).

**BC-2.16.020 §Postconditions §3 — Tier-1/Tier-2 Column Classification (claroty_organization_zones)**:
- Tier-1 (4 cols): `zone_name` (`ocsf_field = "name"` → Arrow `name`, REQUIRED),
  `zone_description` (`ocsf_field = "comment"` → Arrow `comment`),
  `enabled` (`ocsf_field = "status_code"` → Arrow `status_code`),
  `updated_by` (`ocsf_field = "actor.user.name"` → Arrow `actor_user_name`).
- Tier-2 (7 cols): all remaining columns, including 1 Json: `device_conditions`.

**BC-2.16.020 §Postconditions §4 — Tier-1/Tier-2 Column Classification (claroty_organization_zone_policies)**:
- Tier-1 (4 cols): `policy_name` (`ocsf_field = "name"` → Arrow `name`, REQUIRED),
  `policy_action` (`ocsf_field = "activity_name"` → Arrow `activity_name`),
  `policy_notes` (`ocsf_field = "comment"` → Arrow `comment`),
  `updated_by` (`ocsf_field = "actor.user.name"` → Arrow `actor_user_name`).
- Tier-2 (9 cols): all remaining columns, including 3 Json: `communication_conditions`,
  `related_alerts_ids`, `applied_zone_pairs`.

**BC-2.16.021 §Postconditions §1 — TOML Table Contract (claroty_organization_firewall_groups)**
governs the exact `[[tables]]` block: `table_name = "organization_firewall_groups"` (bare name;
`{sensor_id}_{table_name}` = `claroty_organization_firewall_groups` registered/queryable name),
`ocsf_class = "entity_management"`, step `"fetch_organization_firewall_groups"`, `method = "POST"`,
**`path_template = "/api/v1/organization_fw_groups/"`** (abbreviated `_fw_groups` in URL),
**`response_path = "$.organization_firewall_groups"`** (full `organization_firewall_` spelling in
envelope key — these are NOT the same string; mixing them causes silent empty-result defect per
BC-2.16.021 §Invariants and EC-016-021-006). Pagination `type = "offset_limit"` / `page_size = 1000`.

**BC-2.16.021 §Postconditions §2 — TOML Table Contract (claroty_organization_firewall_policies)**:
`path_template = "/api/v1/organization_fw_group_policies/"` (abbreviated),
`response_path = "$.organization_firewall_policies"` (full spelling). Same fw
URL↔envelope-key asymmetry applies; confirmed in schema extract §organization_firewall_policies.

**BC-2.16.021 §Postconditions §3 — Tier-1/Tier-2 Column Classification (claroty_organization_firewall_groups)**:
- Tier-1 (4 cols): `firewall_group_name` (`ocsf_field = "name"` → Arrow `name`, REQUIRED),
  `firewall_group_description` (`ocsf_field = "comment"` → Arrow `comment`),
  `enabled` (`ocsf_field = "status_code"` → Arrow `status_code`),
  `updated_by` (`ocsf_field = "actor.user.name"` → Arrow `actor_user_name`).
- Tier-2 (7 cols): all remaining columns, including 1 Json: `device_conditions`.

**BC-2.16.021 §Postconditions §4 — Tier-1/Tier-2 Column Classification (claroty_organization_firewall_policies)**:
- Tier-1 (4 cols): `policy_name` (`ocsf_field = "name"` → Arrow `name`, REQUIRED),
  `policy_action` (`ocsf_field = "activity_name"` → Arrow `activity_name`),
  `policy_notes` (`ocsf_field = "comment"` → Arrow `comment`),
  `updated_by` (`ocsf_field = "actor.user.name"` → Arrow `actor_user_name`).
- Tier-2 (9 cols): all remaining columns, including 3 Json: `communication_conditions`,
  `related_alerts_ids`, `applied_group_pairs` (**NOTE:** `applied_group_pairs` for firewall
  policies, NOT `applied_zone_pairs` — the zone-domain column name must NOT be used here;
  BC-2.16.021 §Invariants / EC-016-021-010).

**ADR-058 §B2** — Tier-2 columns (those without `ocsf_field`) MUST aggregate into `raw_extensions`
under `ocsf_column_naming = true`. The `entity_management` OCSF class maps to class_uid 3004 —
the existing arm in `class_selector.rs::select_by_class_name` used without modification.

**ADR-058 §C** — `ocsf_field_to_arrow_name("name")` = `"name"` (no change);
`ocsf_field_to_arrow_name("activity_name")` = `"activity_name"`;
`ocsf_field_to_arrow_name("actor.user.name")` = `"actor_user_name"` (dot → underscore flattening).

**spike-findings §Spike 3 §Nested-field classification principle** — all fields that are arrays-of-objects
or arrays-of-scalars MUST be declared `column_type = "json"`. Declaring them as `String` causes the
nested structure to be serialized as a raw string token, not a JSON value — this is a P1 TOML authoring
defect. The eight Json columns across the four tables (`device_conditions` ×2,
`communication_conditions` ×2, `related_alerts_ids` ×2, `applied_zone_pairs`, `applied_group_pairs`)
are authoritative from §Spike 3 Tables A, B, C, and D.

**spike-findings §Overall Verdict** confirms `"entity_management"` arm at class_uid 3004 exists
in `class_selector.rs::select_by_class_name`. No new arm required for any of the four tables.

**S-ADR058-OCSF-ROUTING-001** (merged PR #242, develop@3f1e66179) activated
`ocsf_column_naming = true` at the sensor level in `claroty.sensor.toml`. All four new tables
inherit this setting automatically — no per-table flag needed.

---

## Narrative

As a SOC analyst querying Claroty xDome network governance data via PrismQL,
I want `claroty_organization_zones`, `claroty_organization_zone_policies`,
`claroty_organization_firewall_groups`, and `claroty_organization_firewall_policies`
tables with OCSF `entity_management` class (class_uid 3004),
so that I can query xDome network zone definitions, zone communication policies,
firewall group definitions, and firewall group policies — with OCSF Tier-1 fields
(`name`, `activity_name`, `comment`, `status_code`, `actor_user_name`) for identity,
governance action, and analyst context, and Tier-2 details (including 8 nested Json columns)
available via `raw_extensions`, enabling org-policy governance queries and the
complete zone-firewall governance picture for MSSP security posture analysis.

## Background

As of develop@3f1e66179 the committed `crates/prism-sensors/specs/claroty.sensor.toml`
contains exactly 4 tables — `alerts`, `audit_logs`, `devices`, `device_alert_relations`
(verified by direct inspection of the TOML; exactly 4 `table_name =` declarations).
The Wave A/B/C sibling expansion stories (S-CLAROTY-VULNS-001, S-CLAROTY-OT-EVENTS-001,
S-CLAROTY-DEVVULNREL-001, S-CLAROTY-SERVERS-001) are materialized draft (pending) — NONE
merged, NONE implemented; their TOML blocks do NOT exist in the committed TOML at this story's
authoring time. The implementer MUST re-verify the actual baseline table count at implementation
time and treat the post-story total as **baseline + 4** (8 if the baseline is still the
4-table set at implementation time — more if sibling expansion stories merge first per
depends_on ordering). See §Notes for Implementer item 8 for the merge-status residual.

This story delivers the complete Wave C G5 addition (four TOML blocks):

1. **`claroty_organization_zones`** — 11 columns (4 Tier-1 + 7 Tier-2 incl. 1 Json:
   `device_conditions`). PK: `zone_name` REQUIRED → Arrow `name`. response_path
   `$.organization_zones`.

2. **`claroty_organization_zone_policies`** — 13 columns (4 Tier-1 + 9 Tier-2 incl. 3 Json:
   `communication_conditions`, `related_alerts_ids`, `applied_zone_pairs`). PK: `policy_name`
   REQUIRED → Arrow `name`. response_path `$.organization_zone_policies`. Field name:
   `last_updated` (with trailing 'd' — distinct from zones' `last_update`).

3. **`claroty_organization_firewall_groups`** — 11 columns (4 Tier-1 + 7 Tier-2 incl. 1 Json:
   `device_conditions`). PK: `firewall_group_name` REQUIRED → Arrow `name`. **URL:
   `/api/v1/organization_fw_groups/` (abbreviated); envelope: `$.organization_firewall_groups`
   (full spelling) — these differ and mixing them causes silent empty results.**

4. **`claroty_organization_firewall_policies`** — 13 columns (4 Tier-1 + 9 Tier-2 incl. 3 Json:
   `communication_conditions`, `related_alerts_ids`, `applied_group_pairs`). PK: `policy_name`
   REQUIRED → Arrow `name`. URL: `/api/v1/organization_fw_group_policies/`; envelope:
   `$.organization_firewall_policies`. Json column `applied_group_pairs` (NOT
   `applied_zone_pairs` — distinct from the zone_policies table).

5. **Tests** — TOML parse unit tests + live structural Variant-1 tests against monroe
   (wire-level JSON assertions) for all four tables.

**Live-test approach (per xdome-endpoint-expansion-plan.md §Per-Story Pipeline):**

- **Variant-1 (structural, required):** Live `#[ignore]`'d integration tests against the
  monroe sensor. Assertions are wire-level on the serialized JSON response (class_uid, field
  presence, raw_extensions keys, Json column types). Tests marked `#[ignore]` with comment:
  `// LIVE-MONROE-001: requires CLAROTY_INSTANCE_URL env var pointing to monroe; run manually or in live-validation CI job`.
- **Variant-2 (agent, optional):** PrismQL agent-level test exercising the full LLM agent
  reasoning path. Deferred to live-validation milestone if not complete before holdout gate.
- **DTU note:** SAP-2 DTU-parity probe is **N/A** for all four tables per BC-2.16.020 §PC7,
  BC-2.16.021 §PC7, and D-2200 governing decision (no DTU exists for any of the four
  org-policy endpoints; DTU creation is a separate deferred story). Do NOT run SAP-2 checks
  against `crates/prism-dtu-claroty/src/` — none of the four routes exist there and their
  absence is expected.

**fw URL↔envelope-key asymmetry note (critical):** `claroty_organization_firewall_groups` uses
`path_template = "/api/v1/organization_fw_groups/"` (URL abbreviates `_fw_groups`) but
`response_path = "$.organization_firewall_groups"` (envelope key spells out
`organization_firewall_groups`). These are NOT the same string. The same asymmetry applies
to `claroty_organization_firewall_policies` (URL: `organization_fw_group_policies`; envelope:
`organization_firewall_policies`). Using the abbreviated form in `response_path` produces
an empty result set with no runtime error — a silent data loss defect (EC-016-021-005/006).
This asymmetry is documented as a BC-2.16.021 §Invariant and is confirmed in schema extract
§organization_firewall_groups and §organization_firewall_policies.

**Story-level holdout gate:** After LOCAL 3-CLEAN adversary convergence and BEFORE demo
recording / push to origin, the holdout-evaluator runs HS-028 (4 hidden SINGLE-USE scenarios
authored by PO at remove-uncertainty time; stored under the holdout directory; contamination-
controlled — test-writer and implementer MUST NOT read the HS-028 scenario files). The gate
is BLOCKING: unsatisfied scenarios reset the LOCAL streak per BC-5.39.001.

## Behavioral Contracts

| BC | Title | Version | Role |
|----|-------|---------|------|
| BC-2.16.020 | Claroty xDome Organization Zone Domain — Zones and Zone Policies Queryable Surface with OCSF entity_management Mapping (No DTU) | v1.2 | §Postconditions §1 TOML contract (zones: POST /api/v1/organization_zones/, response_path $.organization_zones, 11 cols, offset_limit 1000); §Postconditions §2 TOML contract (zone_policies: POST /api/v1/organization_zone_policies/, response_path $.organization_zone_policies, 13 cols, last_updated field name asymmetry); §Postconditions §3 Tier-1/Tier-2 zones (4 Tier-1: name REQUIRED, comment, status_code, actor_user_name; 7 Tier-2 incl. 1 Json: device_conditions); §Postconditions §4 Tier-1/Tier-2 zone_policies (4 Tier-1: name REQUIRED, activity_name, comment, actor_user_name; 9 Tier-2 incl. 3 Json: communication_conditions, related_alerts_ids, applied_zone_pairs); §Postconditions §5 PK rationale; §Postconditions §6 Json column serialization behavior; §Postconditions §7 SAP-2 N/A; EC-016-020-001..010. ACs 001–014 trace to this BC. |
| BC-2.16.021 | Claroty xDome Organization Firewall Domain — Firewall Groups and Firewall Group Policies Queryable Surface with OCSF entity_management Mapping (No DTU) | v1.2 | §Postconditions §1 TOML contract (fw_groups: POST /api/v1/organization_fw_groups/ (abbreviated), response_path $.organization_firewall_groups (full spelling), 11 cols — URL≠envelope-key asymmetry CRITICAL); §Postconditions §2 TOML contract (fw_policies: POST /api/v1/organization_fw_group_policies/, response_path $.organization_firewall_policies, 13 cols, applied_group_pairs NOT applied_zone_pairs); §Postconditions §3 Tier-1/Tier-2 fw_groups (4 Tier-1: name REQUIRED, comment, status_code, actor_user_name; 7 Tier-2 incl. 1 Json: device_conditions); §Postconditions §4 Tier-1/Tier-2 fw_policies (4 Tier-1: name REQUIRED, activity_name, comment, actor_user_name; 9 Tier-2 incl. 3 Json: communication_conditions, related_alerts_ids, applied_group_pairs); §Postconditions §5 PK rationale; §Postconditions §6 Json column serialization; §Postconditions §7 SAP-2 N/A; EC-016-021-001..010. ACs 015–026 trace to this BC. |

## Acceptance Criteria

### — claroty_organization_zones (BC-2.16.020) —

### AC-001: TOML block parses without validation error; 11 columns declared; pagination offset_limit 1000; response_path $.organization_zones (traces to BC-2.16.020 postcondition 1 — TOML Table Contract)

`crates/prism-sensors/specs/claroty.sensor.toml` declares a `[[tables]]` block with
`table_name = "organization_zones"` (bare name; `{sensor_id}_{table_name}` = `claroty_organization_zones`
registered/queryable name), `ocsf_class = "entity_management"`,
a step named `"fetch_organization_zones"` with `method = "POST"`,
`path_template = "/api/v1/organization_zones/"`,
`response_path = "$.organization_zones"`, pagination `type = "offset_limit"` / `page_size = 1000`,
and `body_template` containing all 11 contracted fields.

`SpecLoader::parse` on the modified TOML returns `Ok(SensorSpec)` without validation error.
The parsed spec reports 11 `ColumnSpec` entries for the `claroty_organization_zones` queryable table.

**Test:** `test_BC_2_16_020_claroty_organization_zones_toml_block_parses`

### AC-002: Four Tier-1 columns declared with correct ocsf_field; Arrow names are `name` (REQUIRED), `comment`, `status_code`, `actor_user_name` (traces to BC-2.16.020 postcondition 3 — Tier-1 column classification)

The `[[tables.columns]]` blocks declare:
- `zone_name`: `column_type = "string"`, `ocsf_field = "name"`, `options = ["REQUIRED"]`
- `zone_description`: `column_type = "string"`, `ocsf_field = "comment"`
- `enabled`: `column_type = "boolean"`, `ocsf_field = "status_code"`
- `updated_by`: `column_type = "string"`, `ocsf_field = "actor.user.name"`

Under `ocsf_column_naming = true`, Arrow names resolve to `name`, `comment`, `status_code`,
`actor_user_name` respectively. Exactly 4 of 11 columns have a non-None `ocsf_field`.
Exactly 7 columns have no `ocsf_field` (aggregate into `raw_extensions`).

**Test:** `test_BC_2_16_020_claroty_organization_zones_tier1_columns_four_with_ocsf_field`

### AC-003: Tier-2 column query raises E-QUERY-038; `available_columns` contains `raw_extensions` not raw Tier-2 name (traces to BC-2.16.020 invariant — Tier-2 not exposed as standalone Arrow column; EC-016-020-005)

A PrismQL query `SELECT zone_source FROM claroty.claroty_organization_zones LIMIT 1`
raises E-QUERY-038 (column-not-found) at plan time. The error's `available_columns`
MUST contain `raw_extensions`, `name`, `comment`, `status_code`, `actor_user_name`,
`class_uid`, `_sensor` and MUST NOT contain `zone_source` as a standalone column.

Same applies for any other Tier-2 column (`priority`, `device_conditions`,
`attributed_devices`, `created_time`, `last_update`).

**Test:** `test_BC_2_16_020_claroty_organization_zones_e2e_e_query_038_tier2_column` (RG-003, prism-bin, via QueryEngine::execute — authoritative); sensor-side `test_BC_2_16_020_claroty_organization_zones_tier2_column_raises_e_query_038` is defense-in-depth per SAP-3 rule 3

### AC-004 (WIRE-SHAPE rename): SELECT zone_name (raw Tier-1 TOML name) raises E-QUERY-038; `available_columns` contains `name` but NOT `zone_name` (traces to BC-2.16.020 invariant — raw Tier-1 TOML name rejected; Arrow name `name` is the accepted form; TV-BC-2.16.020-003)

A PrismQL query `SELECT zone_name FROM claroty.claroty_organization_zones LIMIT 1` raises
E-QUERY-038 at plan time. The error's `available_columns` MUST contain `name`
(the Arrow form) but MUST NOT contain `zone_name` (the raw TOML column name).

**Test:** `test_BC_2_16_020_claroty_organization_zones_tier1_raw_toml_name_raises_e_query_038`

### AC-005 (WIRE-SHAPE): Live Variant-1 wire-shape — `SELECT * LIMIT 1` serialized JSON contains class_uid=3004, Tier-1 Arrow fields present, raw_extensions contains device_conditions as JSON array (traces to BC-2.16.020 postcondition 1 class_uid; postcondition 3 Tier-1 wire representation; postcondition 6 Json column; TV-BC-2.16.020-002)

Against the live monroe sensor, `SELECT * FROM claroty.claroty_organization_zones LIMIT 1`
serialized JSON response (MCP-visible wire shape per 2026-07-13 wire-shape discipline):
1. `class_uid` key is present with value `3004`
2. `name` key is present (non-null string — zone name)
3. `comment`, `status_code`, `actor_user_name` keys are present (non-null or null)
4. `raw_extensions` key is present as a JSON object (not null, not absent)
5. `raw_extensions` JSON object contains `device_conditions` key; the value is a JSON array
   (NOT a JSON-stringified array — the value must be parseable as an array, not a `"[...]"` string)
6. None of `zone_name`, `zone_description`, `enabled`, `zone_source`, `priority` etc. appear as
   standalone top-level keys in the row

**Test:** `test_BC_2_16_020_claroty_organization_zones_live_wire_shape_class_uid_and_tier1`
(`#[ignore]` — requires `CLAROTY_INSTANCE_URL` env var pointing to monroe)

### AC-006: device_conditions Json column serialized as JSON array in raw_extensions, NOT as stringified value (traces to BC-2.16.020 postcondition 6 — Json column serialization; spike-findings §Spike 3 §Nested-field classification principle; BC-2.16.020 invariant)

The `device_conditions` column is declared `column_type = "json"` in the TOML (not `"string"`).
When the spec-engine processes a zones row, `device_conditions` MUST be serialized into
`raw_extensions` as a JSON-typed value (an actual JSON array object). It MUST NOT be serialized
as a JSON string token (i.e., `"[{...}]"` as a string value is a defect).

An empty `device_conditions` array MUST serialize as `[]` JSON, not null (EC-016-020-003).

**Test:** `test_BC_2_16_020_claroty_organization_zones_device_conditions_json_not_string`
(unit test with mock response containing a row with `device_conditions: [{"key": "value"}]`;
assert deserialized `raw_extensions["device_conditions"]` is a JSON array, not a string)

### AC-007: Missing REQUIRED `zone_name` field → null row, no hard error, subsequent rows unaffected (traces to BC-2.16.020 invariant — zone_name MUST be present; EC-016-020-001)

The `zone_name` column carries `options = ["REQUIRED"]` in the TOML. When the API response
contains a zones row where `zone_name` is absent or null, the spec-engine produces a null
row (REQUIRED semantics) without raising a hard error. Subsequent rows continue normally.

**Test:** `test_BC_2_16_020_claroty_organization_zones_required_zone_name_absent_produces_null_row`
(unit test with mock response containing a row missing `zone_name`)

### AC-008: Nullable count envelope — empty-page halt triggers; no error when count is null (traces to BC-2.16.020 postcondition 1 pagination; EC-016-020-004)

When the `organization_zones` response envelope contains `count: null` or omits `count`,
the spec-engine pagination uses the empty-page check (halts when page is empty), not a
null-pointer dereference on `count`. No error raised.

**Test:** `test_BC_2_16_020_claroty_organization_zones_nullable_count_uses_empty_page_halt`
(unit test with mock response `{"organization_zones": [], "count": null}`)

### — claroty_organization_zone_policies (BC-2.16.020) —

### AC-009: TOML block parses; 13 columns declared; response_path $.organization_zone_policies; field name is `last_updated` (with trailing 'd') not `last_update` (traces to BC-2.16.020 postcondition 2 — TOML Table Contract; datetime field name asymmetry note)

`crates/prism-sensors/specs/claroty.sensor.toml` declares a `[[tables]]` block with
`table_name = "organization_zone_policies"` (bare name; `{sensor_id}_{table_name}` =
`claroty_organization_zone_policies` registered/queryable name), `ocsf_class = "entity_management"`,
step `"fetch_organization_zone_policies"`, `method = "POST"`,
`path_template = "/api/v1/organization_zone_policies/"`,
`response_path = "$.organization_zone_policies"`, pagination `type = "offset_limit"` /
`page_size = 1000`, and `body_template` containing all 13 contracted fields.

The `body_template` field list MUST include `"last_updated"` (with trailing 'd') for the
datetime field of zone_policies. It MUST NOT include `"last_update"` (which belongs to
the zones table). Using the wrong name silently omits temporal data from the response
(EC-016-020-009).

`SpecLoader::parse` returns `Ok(SensorSpec)` without validation error. 13 `ColumnSpec`
entries for the `claroty_organization_zone_policies` queryable table.

**Test:** `test_BC_2_16_020_claroty_organization_zone_policies_toml_block_parses`

### AC-010: Four Tier-1 columns declared; Arrow names `name` (REQUIRED), `activity_name`, `comment`, `actor_user_name` (traces to BC-2.16.020 postcondition 4 — Tier-1 column classification)

- `policy_name`: `column_type = "string"`, `ocsf_field = "name"`, `options = ["REQUIRED"]`
- `policy_action`: `column_type = "string"`, `ocsf_field = "activity_name"`
- `policy_notes`: `column_type = "string"`, `ocsf_field = "comment"`
- `updated_by`: `column_type = "string"`, `ocsf_field = "actor.user.name"`

Arrow names: `name`, `activity_name`, `comment`, `actor_user_name`. Exactly 4 of 13 columns
have a non-None `ocsf_field`. Exactly 9 columns have no `ocsf_field` (aggregate into
`raw_extensions`), including the 3 Json columns.

**Test:** `test_BC_2_16_020_claroty_organization_zone_policies_tier1_columns_four_with_ocsf_field`

### AC-011 (WIRE-SHAPE): Live Variant-1 wire-shape — `SELECT * LIMIT 1` serialized JSON contains class_uid=3004, `name`, `activity_name`; raw_extensions contains `communication_conditions`, `related_alerts_ids`, `applied_zone_pairs` as JSON arrays (traces to BC-2.16.020 postconditions 2/4/6; TV-BC-2.16.020-007)

Against the live monroe sensor, `SELECT * FROM claroty.claroty_organization_zone_policies LIMIT 1`
serialized JSON:
1. `class_uid` = 3004
2. `name` present (policy name string)
3. `activity_name` present or null ("Allow"/"Deny" / null)
4. `comment`, `actor_user_name` present or null
5. `raw_extensions` object present; contains `communication_conditions`, `related_alerts_ids`,
   `applied_zone_pairs` keys — each value is a JSON array (NOT a stringified array)
6. No standalone Tier-2 column names (`policy_source`, `matching_devices`, etc.) as root keys

**Test:** `test_BC_2_16_020_claroty_organization_zone_policies_live_wire_shape_class_uid_and_tier1`
(`#[ignore]` — requires `CLAROTY_INSTANCE_URL` env var)

### AC-012: SELECT applied_zone_pairs raises E-QUERY-038; `available_columns` contains `raw_extensions` not `applied_zone_pairs` (traces to BC-2.16.020 invariant; EC-016-020-006)

A PrismQL query `SELECT applied_zone_pairs FROM claroty.claroty_organization_zone_policies LIMIT 1`
raises E-QUERY-038 at plan time. The error's `available_columns` MUST contain `raw_extensions`,
`name`, `activity_name`, `comment`, `actor_user_name` but MUST NOT contain `applied_zone_pairs`.

**Test:** `test_BC_2_16_020_claroty_organization_zone_policies_e2e_e_query_038_tier2_column` (RG-012, prism-bin, via QueryEngine::execute — authoritative); sensor-side `test_BC_2_16_020_claroty_organization_zone_policies_applied_zone_pairs_raises_e_query_038` is defense-in-depth per SAP-3 rule 3

### AC-013: Missing REQUIRED `policy_name` → null row, no hard error (traces to BC-2.16.020 invariant; EC-016-020-002)

When a zone_policies response row has `policy_name` absent or null, the spec-engine produces a
null row (REQUIRED semantics) without a hard error. Subsequent rows continue normally.

**Test:** `test_BC_2_16_020_claroty_organization_zone_policies_required_policy_name_absent_produces_null_row`

### AC-014: Json columns communication_conditions, related_alerts_ids, applied_zone_pairs each serialized as JSON (not stringified) in raw_extensions (traces to BC-2.16.020 postcondition 6 — Json column serialization; BC-2.16.020 invariant; spike-findings §Spike 3 §Table B)

All three Json columns of zone_policies MUST be declared `column_type = "json"` (not `"string"`).
When the spec-engine processes a zone_policies row, each of `communication_conditions`,
`related_alerts_ids`, and `applied_zone_pairs` in `raw_extensions` MUST be a JSON-typed value
(actual array/object), not a JSON string encoding. An empty array MUST serialize as `[]`, not null.

**Test:** `test_BC_2_16_020_claroty_organization_zone_policies_json_columns_not_stringified`
(unit test with mock response containing `{"communication_conditions": [{"src": "A"}], "related_alerts_ids": [1, 2], "applied_zone_pairs": [{"src_zone": "Z1", "dst_zone": "Z2"}]}`; assert each `raw_extensions` value is a JSON array, not a string)

### — claroty_organization_firewall_groups (BC-2.16.021) —

### AC-015: TOML block parses; 11 columns; path_template=/api/v1/organization_fw_groups/ (abbreviated URL); response_path=$.organization_firewall_groups (full envelope key) (traces to BC-2.16.021 postcondition 1 — TOML Table Contract; URL vs envelope key asymmetry invariant)

`crates/prism-sensors/specs/claroty.sensor.toml` declares a `[[tables]]` block with
`table_name = "organization_firewall_groups"` (bare name; `{sensor_id}_{table_name}` =
`claroty_organization_firewall_groups` registered/queryable name), `ocsf_class = "entity_management"`,
step `"fetch_organization_firewall_groups"`, `method = "POST"`,
**`path_template = "/api/v1/organization_fw_groups/"`** (abbreviated — `_fw_groups` in path),
**`response_path = "$.organization_firewall_groups"`** (full `organization_firewall_groups`
spelling in envelope key — NOT `$.organization_fw_groups`),
pagination `type = "offset_limit"` / `page_size = 1000`, and `body_template` with all 11 fields.

The presence of `/organization_fw_groups/` in `path_template` AND `$.organization_firewall_groups`
in `response_path` in the SAME TOML block is NOT a contradiction — it is the documented
API asymmetry. Both strings MUST be present exactly as specified.

`SpecLoader::parse` returns `Ok(SensorSpec)`. 11 `ColumnSpec` entries for the
`claroty_organization_firewall_groups` queryable table.

**Test:** `test_BC_2_16_021_claroty_organization_firewall_groups_toml_block_parses`

### AC-016: Four Tier-1 columns declared; Arrow names `name` (REQUIRED), `comment`, `status_code`, `actor_user_name` (traces to BC-2.16.021 postcondition 3 — Tier-1 column classification)

- `firewall_group_name`: `column_type = "string"`, `ocsf_field = "name"`, `options = ["REQUIRED"]`
- `firewall_group_description`: `column_type = "string"`, `ocsf_field = "comment"`
- `enabled`: `column_type = "boolean"`, `ocsf_field = "status_code"`
- `updated_by`: `column_type = "string"`, `ocsf_field = "actor.user.name"`

Arrow names: `name`, `comment`, `status_code`, `actor_user_name`. Exactly 4 of 11 columns
have a non-None `ocsf_field`. Exactly 7 have no `ocsf_field` (aggregate into `raw_extensions`).

**Test:** `test_BC_2_16_021_claroty_organization_firewall_groups_tier1_columns_four_with_ocsf_field`

### AC-017 (fw URL↔envelope-key asymmetry): Live SELECT against claroty_organization_firewall_groups returns non-empty rows, confirming response_path=$.organization_firewall_groups extracts correctly from the abbreviated-URL response (traces to BC-2.16.021 invariant; EC-016-021-005/006)

Against the live monroe sensor, `SELECT name FROM claroty.claroty_organization_firewall_groups LIMIT 5`
returns at least 1 row with a non-null `name` string value. The non-empty result confirms
that `response_path = "$.organization_firewall_groups"` correctly extracts from the xDome
response envelope (which uses the full `organization_firewall_groups` key despite the URL
using the abbreviated `_fw_groups` path).

Using `response_path = "$.organization_fw_groups"` would produce an empty result with no
runtime error — the test's non-empty result is the evidence that the correct (full-spelling)
response_path is in effect.

**Test:** `test_BC_2_16_021_claroty_organization_firewall_groups_live_fw_asymmetry_nonempty_result`
(`#[ignore]` — requires `CLAROTY_INSTANCE_URL` env var)

### AC-018 (WIRE-SHAPE): Live SELECT * LIMIT 1 serialized JSON contains class_uid=3004, Tier-1 present, raw_extensions.device_conditions is JSON array (traces to BC-2.16.021 postconditions 1/3/6; TV-BC-2.16.021-002)

Against the live monroe sensor, `SELECT * FROM claroty.claroty_organization_firewall_groups LIMIT 1`
serialized JSON:
1. `class_uid` = 3004
2. `name` present (firewall group name string)
3. `comment`, `status_code`, `actor_user_name` present or null
4. `raw_extensions` object present; contains `device_conditions` key as a JSON array
   (NOT stringified)
5. No standalone raw TOML column names as root keys — `firewall_group_name` (Tier-1 raw form;
   only its Arrow name `name` appears), plus Tier-2 raw names (`firewall_group_source`, `priority`,
   `attributed_devices`, etc.)

**Test:** `test_BC_2_16_021_claroty_organization_firewall_groups_live_wire_shape_class_uid_and_tier1`
(`#[ignore]` — requires `CLAROTY_INSTANCE_URL` env var)

### AC-019: Missing REQUIRED `firewall_group_name` → null row, no hard error (traces to BC-2.16.021 invariant; EC-016-021-001)

When a firewall_groups response row has `firewall_group_name` absent or null, the spec-engine
produces a null row (REQUIRED semantics) without a hard error. Subsequent rows continue.

**Test:** `test_BC_2_16_021_claroty_organization_firewall_groups_required_fwgroupname_absent_produces_null_row`

### AC-020: SELECT firewall_group_source raises E-QUERY-038; `available_columns` contains `raw_extensions` not `firewall_group_source` (traces to BC-2.16.021 invariant — Tier-2 not exposed as standalone Arrow column)

A PrismQL query `SELECT firewall_group_source FROM claroty.claroty_organization_firewall_groups LIMIT 1`
raises E-QUERY-038 at plan time. The error's `available_columns` MUST contain `raw_extensions`,
`name`, `comment`, `status_code`, `actor_user_name` but MUST NOT contain `firewall_group_source`.

Same applies for `priority`, `device_conditions`, `attributed_devices`, etc.

**Test:** `test_BC_2_16_021_claroty_organization_firewall_groups_e2e_e_query_038_tier2_column` (RG-020, prism-bin, via QueryEngine::execute — authoritative); sensor-side `test_BC_2_16_021_claroty_organization_firewall_groups_tier2_column_raises_e_query_038` is defense-in-depth per SAP-3 rule 3

### — claroty_organization_firewall_policies (BC-2.16.021) —

### AC-021: TOML block parses; 13 columns; path_template=/api/v1/organization_fw_group_policies/ (abbreviated); response_path=$.organization_firewall_policies (full spelling) (traces to BC-2.16.021 postcondition 2 — TOML Table Contract; URL vs envelope key asymmetry)

`crates/prism-sensors/specs/claroty.sensor.toml` declares a `[[tables]]` block with
`table_name = "organization_firewall_policies"` (bare name; `{sensor_id}_{table_name}` =
`claroty_organization_firewall_policies` registered/queryable name), `ocsf_class = "entity_management"`,
step `"fetch_organization_firewall_policies"`, `method = "POST"`,
`path_template = "/api/v1/organization_fw_group_policies/"` (abbreviated URL),
`response_path = "$.organization_firewall_policies"` (full envelope key spelling),
pagination `type = "offset_limit"` / `page_size = 1000`, and `body_template` with all 13 fields.

`SpecLoader::parse` returns `Ok(SensorSpec)`. 13 `ColumnSpec` entries for the
`claroty_organization_firewall_policies` queryable table.

**Test:** `test_BC_2_16_021_claroty_organization_firewall_policies_toml_block_parses`

### AC-022: Four Tier-1 columns declared; Arrow names `name` (REQUIRED), `activity_name`, `comment`, `actor_user_name` (traces to BC-2.16.021 postcondition 4 — Tier-1 column classification)

- `policy_name`: `column_type = "string"`, `ocsf_field = "name"`, `options = ["REQUIRED"]`
- `policy_action`: `column_type = "string"`, `ocsf_field = "activity_name"`
- `policy_notes`: `column_type = "string"`, `ocsf_field = "comment"`
- `updated_by`: `column_type = "string"`, `ocsf_field = "actor.user.name"`

Arrow names: `name`, `activity_name`, `comment`, `actor_user_name`. Exactly 4 of 13 columns
have a non-None `ocsf_field`. Exactly 9 have no `ocsf_field` (aggregate into `raw_extensions`).

**Test:** `test_BC_2_16_021_claroty_organization_firewall_policies_tier1_columns_four_with_ocsf_field`

### AC-023 (WIRE-SHAPE): Live SELECT * LIMIT 1 serialized JSON contains class_uid=3004, `name`, `activity_name`; raw_extensions has communication_conditions, related_alerts_ids, applied_group_pairs (NOT applied_zone_pairs) as JSON arrays (traces to BC-2.16.021 postconditions 2/4/6; TV-BC-2.16.021-007)

Against the live monroe sensor, `SELECT * FROM claroty.claroty_organization_firewall_policies LIMIT 1`
serialized JSON:
1. `class_uid` = 3004
2. `name` present (firewall policy name)
3. `activity_name` present or null ("Allow"/"Deny")
4. `comment`, `actor_user_name` present or null
5. `raw_extensions` object present; contains `communication_conditions`, `related_alerts_ids`,
   `applied_group_pairs` keys — each a JSON array (NOT stringified)
6. `raw_extensions` MUST NOT contain `applied_zone_pairs` as a key — that column belongs to
   zone_policies, not firewall_policies
7. No standalone Tier-2 column names as root keys

**Test:** `test_BC_2_16_021_claroty_organization_firewall_policies_live_wire_shape_class_uid_and_tier1`
(`#[ignore]` — requires `CLAROTY_INSTANCE_URL` env var)

### AC-024: SELECT applied_group_pairs raises E-QUERY-038; `available_columns` contains `raw_extensions` not `applied_group_pairs`; confirms applied_group_pairs ≠ applied_zone_pairs (distinct columns in distinct tables) (traces to BC-2.16.021 invariant; EC-016-021-007/010)

A PrismQL query `SELECT applied_group_pairs FROM claroty.claroty_organization_firewall_policies LIMIT 1`
raises E-QUERY-038 at plan time. The error's `available_columns` MUST contain `raw_extensions`,
`name`, `activity_name`, `comment`, `actor_user_name` but MUST NOT contain `applied_group_pairs`.

Additionally, the TOML spec for `claroty_organization_firewall_policies` MUST declare
`name = "applied_group_pairs"` (not `"applied_zone_pairs"`). A TOML using the
zone-domain column name `applied_zone_pairs` in the firewall_policies block would silently
request the wrong field from the xDome API (EC-016-021-010).

**Test:** `test_BC_2_16_021_claroty_organization_firewall_policies_e2e_e_query_038_tier2_column` (RG-024, prism-bin, via QueryEngine::execute — authoritative); sensor-side `test_BC_2_16_021_claroty_organization_firewall_policies_applied_group_pairs_raises_e_query_038` is defense-in-depth per SAP-3 rule 3
(also verifies TOML has `applied_group_pairs` not `applied_zone_pairs` in the column block)

### AC-025: Missing REQUIRED `policy_name` → null row, no hard error; count null in firewall envelope → empty-page halt (traces to BC-2.16.021 invariant; EC-016-021-002; EC-016-021-004)

When a firewall_policies response row has `policy_name` absent or null, the spec-engine
produces a null row (REQUIRED semantics) without a hard error.

When the firewall_policies or firewall_groups response envelope contains `count: null` or
omits `count`, the spec-engine pagination uses the empty-page check — no error raised.
Consistent with the established pattern across all Claroty paginated endpoints.

**Tests:**
- `test_BC_2_16_021_claroty_organization_firewall_policies_required_policy_name_absent_produces_null_row`
- `test_BC_2_16_021_claroty_organization_firewall_policies_nullable_count_uses_empty_page_halt`
  (unit test with mock `{"organization_firewall_policies": [], "count": null}`)

### AC-026: Json columns communication_conditions, related_alerts_ids, applied_group_pairs each serialized as JSON (not stringified) in raw_extensions; applied_group_pairs contains {src_group, dst_group} objects — NOT {src_zone, dst_zone} (traces to BC-2.16.021 postcondition 6 — Json column serialization; BC-2.16.021 invariant; spike-findings §Spike 3 §Table D)

All three Json columns of firewall_policies MUST be declared `column_type = "json"` (not `"string"`).
Each MUST serialize in `raw_extensions` as a JSON-typed value (actual array/object), not a
JSON string encoding. `applied_group_pairs` specifically carries `{src_group, dst_group}` pair
objects per spike-findings §Table D — distinct from `applied_zone_pairs` (`{src_zone, dst_zone}`).

**Test:** `test_BC_2_16_021_claroty_organization_firewall_policies_json_columns_not_stringified`
(unit test with mock response with all three Json columns; assert each is a JSON array; assert
`applied_group_pairs` (not `applied_zone_pairs`) key is present in `raw_extensions`)

## Red Gate Tests

| ID | Test name | Test type | What it gates |
|----|-----------|-----------|---------------|
| RG-001 | `test_BC_2_16_020_claroty_organization_zones_toml_block_parses` | Unit (SpecLoader::parse) | AC-001: TOML block parses Ok; 11 ColumnSpec entries; pagination offset_limit 1000; response_path $.organization_zones |
| RG-002 | `test_BC_2_16_020_claroty_organization_zones_tier1_columns_four_with_ocsf_field` | Unit (ColumnSpec inspection) | AC-002: exactly 4 Tier-1 (zone_name→name REQUIRED; zone_description→comment; enabled→status_code; updated_by→actor.user.name); 7 Tier-2 have None ocsf_field |
| RG-003 | `test_BC_2_16_020_claroty_organization_zones_e2e_e_query_038_tier2_column` | Integration end-to-end (prism-bin, via QueryEngine::execute — authoritative; prism-sensors version is defense-in-depth per SAP-3 rule 3) | AC-003: SELECT zone_source raises E-QUERY-038; available_columns excludes zone_source; includes raw_extensions, name, comment, status_code, actor_user_name |
| RG-004 | `test_BC_2_16_020_claroty_organization_zones_tier1_raw_toml_name_raises_e_query_038` | Unit (ocsf_projected_column_names helper; defense-in-depth per SAP-3 rule 3) | AC-004 WIRE-SHAPE rename: SELECT zone_name raises E-QUERY-038; available_columns has `name` but NOT `zone_name`. Defense-in-depth — RG-003 (`test_BC_2_16_020_claroty_organization_zones_e2e_e_query_038_tier2_column`, prism-bin end-to-end) is the authoritative E-QUERY-038 gate that transitively covers the raw Tier-1 name case |
| RG-005 | `test_BC_2_16_020_claroty_organization_zones_live_wire_shape_class_uid_and_tier1` | Live Variant-1 (`#[ignore]`) | AC-005 WIRE-SHAPE: wire JSON class_uid=3004, name present, raw_extensions has device_conditions JSON array; no Tier-2 as standalone root keys |
| RG-006 | `test_BC_2_16_020_claroty_organization_zones_device_conditions_json_not_string` | Unit (mock response) | AC-006: device_conditions in raw_extensions is JSON array not stringified; empty array serializes as [] not null |
| RG-007 | `test_BC_2_16_020_claroty_organization_zones_required_zone_name_absent_produces_null_row` | Unit (mock response) | AC-007: zone_name absent → null row; no hard error; subsequent rows continue |
| RG-008 | `test_BC_2_16_020_claroty_organization_zones_nullable_count_uses_empty_page_halt` | Unit (mock response) | AC-008: count=null in organization_zones envelope → empty-page halt; no error; no null-ptr deref |
| RG-009 | `test_BC_2_16_020_claroty_organization_zone_policies_toml_block_parses` | Unit (SpecLoader::parse) | AC-009: TOML block parses Ok; 13 ColumnSpec entries; last_updated (WITH trailing 'd') in body_template; response_path $.organization_zone_policies |
| RG-010 | `test_BC_2_16_020_claroty_organization_zone_policies_tier1_columns_four_with_ocsf_field` | Unit (ColumnSpec inspection) | AC-010: exactly 4 Tier-1 (policy_name→name REQUIRED; policy_action→activity_name; policy_notes→comment; updated_by→actor.user.name); 9 Tier-2 have None |
| RG-011 | `test_BC_2_16_020_claroty_organization_zone_policies_live_wire_shape_class_uid_and_tier1` | Live Variant-1 (`#[ignore]`) | AC-011 WIRE-SHAPE: wire JSON class_uid=3004, name, activity_name present; raw_extensions has communication_conditions, related_alerts_ids, applied_zone_pairs as JSON arrays |
| RG-012 | `test_BC_2_16_020_claroty_organization_zone_policies_e2e_e_query_038_tier2_column` | Integration end-to-end (prism-bin, via QueryEngine::execute — authoritative; prism-sensors version is defense-in-depth per SAP-3 rule 3) | AC-012: SELECT applied_zone_pairs raises E-QUERY-038; available_columns excludes applied_zone_pairs; includes raw_extensions |
| RG-013 | `test_BC_2_16_020_claroty_organization_zone_policies_required_policy_name_absent_produces_null_row` | Unit (mock response) | AC-013: policy_name absent → null row; no hard error; subsequent rows continue |
| RG-014 | `test_BC_2_16_020_claroty_organization_zone_policies_json_columns_not_stringified` | Unit (mock response) | AC-014: communication_conditions, related_alerts_ids, applied_zone_pairs each serialized as JSON arrays in raw_extensions; not stringified |
| RG-015 | `test_BC_2_16_021_claroty_organization_firewall_groups_toml_block_parses` | Unit (SpecLoader::parse) | AC-015: TOML block parses Ok; 11 ColumnSpec entries; path_template=/api/v1/organization_fw_groups/ (abbreviated); response_path=$.organization_firewall_groups (full spelling) — both strings present in parsed spec |
| RG-016 | `test_BC_2_16_021_claroty_organization_firewall_groups_tier1_columns_four_with_ocsf_field` | Unit (ColumnSpec inspection) | AC-016: exactly 4 Tier-1 (firewall_group_name→name REQUIRED; firewall_group_description→comment; enabled→status_code; updated_by→actor.user.name); 7 Tier-2 have None |
| RG-017 | `test_BC_2_16_021_claroty_organization_firewall_groups_live_fw_asymmetry_nonempty_result` | Live Variant-1 (`#[ignore]`) | AC-017 fw asymmetry: SELECT name FROM claroty_organization_firewall_groups returns non-empty rows, confirming $.organization_firewall_groups response_path works against abbreviated-URL endpoint |
| RG-018 | `test_BC_2_16_021_claroty_organization_firewall_groups_live_wire_shape_class_uid_and_tier1` | Live Variant-1 (`#[ignore]`) | AC-018 WIRE-SHAPE: wire JSON class_uid=3004, name, raw_extensions.device_conditions JSON array; no Tier-2 as standalone root keys |
| RG-019 | `test_BC_2_16_021_claroty_organization_firewall_groups_required_fwgroupname_absent_produces_null_row` | Unit (mock response) | AC-019: firewall_group_name absent → null row; no hard error |
| RG-020 | `test_BC_2_16_021_claroty_organization_firewall_groups_e2e_e_query_038_tier2_column` | Integration end-to-end (prism-bin, via QueryEngine::execute — authoritative; prism-sensors version is defense-in-depth per SAP-3 rule 3) | AC-020: SELECT firewall_group_source raises E-QUERY-038; available_columns excludes firewall_group_source; includes raw_extensions, name, comment, status_code, actor_user_name |
| RG-021 | `test_BC_2_16_021_claroty_organization_firewall_policies_toml_block_parses` | Unit (SpecLoader::parse) | AC-021: TOML block parses Ok; 13 ColumnSpec entries; path_template=/api/v1/organization_fw_group_policies/ (abbreviated); response_path=$.organization_firewall_policies (full spelling); body_template has applied_group_pairs (NOT applied_zone_pairs) |
| RG-022 | `test_BC_2_16_021_claroty_organization_firewall_policies_tier1_columns_four_with_ocsf_field` | Unit (ColumnSpec inspection) | AC-022: exactly 4 Tier-1 (policy_name→name REQUIRED; policy_action→activity_name; policy_notes→comment; updated_by→actor.user.name); 9 Tier-2 have None |
| RG-023 | `test_BC_2_16_021_claroty_organization_firewall_policies_live_wire_shape_class_uid_and_tier1` | Live Variant-1 (`#[ignore]`) | AC-023 WIRE-SHAPE: wire JSON class_uid=3004, name, activity_name; raw_extensions has applied_group_pairs (NOT applied_zone_pairs) as JSON array; communication_conditions and related_alerts_ids also JSON arrays |
| RG-024 | `test_BC_2_16_021_claroty_organization_firewall_policies_e2e_e_query_038_tier2_column` | Integration end-to-end (prism-bin, via QueryEngine::execute — authoritative; prism-sensors version is defense-in-depth per SAP-3 rule 3) | AC-024: SELECT applied_group_pairs raises E-QUERY-038; also verifies TOML column block uses applied_group_pairs not applied_zone_pairs |
| RG-025 | `test_BC_2_16_021_claroty_organization_firewall_policies_required_policy_name_absent_produces_null_row` + `test_BC_2_16_021_claroty_organization_firewall_policies_nullable_count_uses_empty_page_halt` | Unit (mock response) — two sub-tests | AC-025: (1) policy_name absent → null row; (2) count=null in fw_policies envelope → empty-page halt |
| RG-026 | `test_BC_2_16_021_claroty_organization_firewall_policies_json_columns_not_stringified` | Unit (mock response) | AC-026: communication_conditions, related_alerts_ids, applied_group_pairs each serialized as JSON arrays; applied_group_pairs key (NOT applied_zone_pairs) present in raw_extensions |
| RG-027 | `test_BC_2_16_020_claroty_organization_zones_wire_shape_class_uid_3004_mock` | Integration (prism-bin, wire-shape via SpecDrivenSensorAdapter::fetch — authoritative path; no DTU per D-2200) | SAP4-020-Z-1: zones wire shape; class_uid=3004; name+comment+status_code+actor_user_name Tier-1 present; raw_extensions present; Tier-2 NOT as standalone root keys |
| RG-028 | `test_BC_2_16_020_claroty_organization_zones_wire_shape_serialized_json_null_not_absent` | Integration (prism-bin, wire-shape via SpecDrivenSensorAdapter::fetch — authoritative path) | SAP4-020-Z-2: zones null-not-absent wire discipline; absent optional fields produce explicit null cells in wire output |
| RG-029 | `test_BC_2_16_020_claroty_organization_zone_policies_wire_shape_class_uid_3004_mock` | Integration (prism-bin, wire-shape via SpecDrivenSensorAdapter::fetch — authoritative path; no DTU per D-2200) | SAP4-020-ZP-3: zone_policies wire shape; class_uid=3004; name+activity_name+comment+actor_user_name Tier-1 present; raw_extensions with communication_conditions/related_alerts_ids/applied_zone_pairs as JSON arrays |
| RG-030 | `test_BC_2_16_020_claroty_organization_zone_policies_wire_shape_serialized_json_null_not_absent` | Integration (prism-bin, wire-shape via SpecDrivenSensorAdapter::fetch — authoritative path) | SAP4-020-ZP-4: zone_policies null-not-absent wire discipline; Json columns explicit null when absent |
| RG-031 | `test_BC_2_16_021_claroty_organization_firewall_groups_wire_shape_class_uid_3004_mock` | Integration (prism-bin, wire-shape via SpecDrivenSensorAdapter::fetch — authoritative path; no DTU per D-2200) | SAP4-021-FG-1: firewall_groups wire shape; class_uid=3004; name+comment+status_code+actor_user_name Tier-1 present; raw_extensions present |
| RG-032 | `test_BC_2_16_021_claroty_organization_firewall_groups_wire_shape_serialized_json_null_not_absent` | Integration (prism-bin, wire-shape via SpecDrivenSensorAdapter::fetch — authoritative path) | SAP4-021-FG-2: firewall_groups null-not-absent wire discipline; device_conditions JSON column null-passthrough |
| RG-033 | `test_BC_2_16_021_claroty_organization_firewall_policies_wire_shape_class_uid_3004_mock` | Integration (prism-bin, wire-shape via SpecDrivenSensorAdapter::fetch — authoritative path; no DTU per D-2200) | SAP4-021-FP-3: firewall_policies wire shape; class_uid=3004; applied_group_pairs (NOT applied_zone_pairs) present in raw_extensions; communication_conditions and related_alerts_ids as JSON arrays |
| RG-034 | `test_BC_2_16_021_claroty_organization_firewall_policies_wire_shape_serialized_json_null_not_absent` | Integration (prism-bin, wire-shape via SpecDrivenSensorAdapter::fetch — authoritative path) | SAP4-021-FP-4: firewall_policies null-not-absent wire discipline; Json columns null-passthrough; applied_group_pairs (NOT applied_zone_pairs) key in null-explicit wire output |

**BC-5.38.001 density check:** 34 Red Gate tests / 26 acceptance criteria = 1.31 ≥ 0.5 threshold. PASS.
(Note: RG-025 gates two sub-tests under AC-025; counted as 1 RGT per 1 AC. RG-027..RG-034 are authoritative fetch-path wire-shape tests for all 4 org-policy tables via SpecDrivenSensorAdapter::fetch.)

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `claroty_organization_zones` TOML block | `crates/prism-sensors/specs/claroty.sensor.toml` | Static data (TOML spec) |
| `claroty_organization_zone_policies` TOML block | `crates/prism-sensors/specs/claroty.sensor.toml` | Static data (TOML spec) |
| `claroty_organization_firewall_groups` TOML block | `crates/prism-sensors/specs/claroty.sensor.toml` | Static data (TOML spec) |
| `claroty_organization_firewall_policies` TOML block | `crates/prism-sensors/specs/claroty.sensor.toml` | Static data (TOML spec) |
| TOML parse validation (all 4 tables) | `crates/prism-spec-engine/src/spec_parser.rs §spec_parser` | Pure (TOML deserialization; no I/O) |
| Tier-1/Tier-2 Arrow schema computation (all 4 tables) | `crates/prism-spec-engine/src/column_mapping.rs §ocsf_field_to_arrow_name` | Pure (string transformation; deterministic) |
| Json column serialization into raw_extensions (all 4 tables) | `crates/prism-spec-engine/src/pipeline.rs §PipelineExecutor::execute` | Effectful (processes HTTP response; builds Arrow RecordBatch) |
| OffsetLimit POST-body injection (all 4 tables) | `crates/prism-spec-engine/src/pipeline.rs §PipelineExecutor::execute` | Effectful (HTTP POST to xDome; merges offset/limit into body_template) |
| response_path extraction + null-passthrough (all 4 tables) | `crates/prism-bin/src/spec_driven_adapter.rs §pipeline_result_to_record_batch` (contains `build_column_array`) | Effectful (processes HTTP response; REQUIRED-field null rows; Tier-2 → raw_extensions) |
| `entity_management` class arm (shared by all 4 tables) | `crates/prism-ocsf/src/class_selector.rs::select_by_class_name` | Pure (constant → u32 lookup; arm already exists; returns 3004) |

Architecture section references:
- `architecture/module-decomposition.md` §SS-01 Sensor Adapters (prism-sensors; claroty.sensor.toml)
- `architecture/module-decomposition.md` §SS-16 Spec Engine (prism-spec-engine; spec_parser, pipeline, column_mapping)
- ADR-058 §B2 (Tier-2 raw_extensions aggregation), §C (Arrow field naming; actor.user.name → actor_user_name), §D (ocsf_column_naming per-sensor flag)

## Purity Classification

- **Pure functions (no I/O, deterministic):** `SpecLoader::parse` (TOML deserialization);
  `ocsf_field_to_arrow_name` (string → string); `select_by_class_name("entity_management")`
  (constant lookup, returns 3004); RG-001/002/009/010/015/016/021/022 TOML parse + column inspection.
- **Effectful functions (I/O, network):** `PipelineExecutor::execute` (HTTP POST to four
  org-policy endpoints; pagination loops); `pipeline_result_to_record_batch` (HTTP response to
  Arrow RecordBatch; Json column handling); RG-005/011/017/018/023 live integration tests
  (require running monroe sensor).

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Row in `claroty_organization_zones` missing `zone_name` (REQUIRED) | Null row; no hard error; pagination continues (EC-016-020-001) |
| EC-002 | `enabled` null or absent in a zones row | Null `status_code` Arrow cell; not an error (EC-016-020-007) |
| EC-003 | `device_conditions` is empty array `[]` in zones row | Serialized as `[]` JSON in raw_extensions; not null (EC-016-020-003) |
| EC-004 | `count` null or absent in organization_zones envelope | Empty-page halt; no null-deref (EC-016-020-004) |
| EC-005 | Query references Tier-2 `zone_source` by raw name on zones table | E-QUERY-038; available_columns has raw_extensions, name, comment, status_code, actor_user_name but NOT zone_source (EC-016-020-005) |
| EC-006 | Row in `claroty_organization_zone_policies` missing `policy_name` (REQUIRED) | Null row; no hard error (EC-016-020-002) |
| EC-007 | Query references Tier-2 `applied_zone_pairs` by raw name on zone_policies | E-QUERY-038; available_columns has raw_extensions, name, activity_name, comment, actor_user_name but NOT applied_zone_pairs (EC-016-020-006) |
| EC-008 | `policy_action` absent in a zone_policies row | Null `activity_name` Arrow cell; not an error (EC-016-020-008) |
| EC-009 | Implementer uses `last_update` for zone_policies body_template instead of `last_updated` | Column silently absent (API returns nothing for non-existent field name); structural live-sensor test catches it via last_updated key assertion (EC-016-020-009) |
| EC-010 | Row in `claroty_organization_firewall_groups` missing `firewall_group_name` (REQUIRED) | Null row; no hard error (EC-016-021-001) |
| EC-011 | `response_path` mistakenly uses `$.organization_fw_groups` (abbreviated) instead of `$.organization_firewall_groups` | Spec-engine extracts empty result; no runtime error — silent data loss; caught by structural live-sensor test asserting non-empty result (EC-016-021-006) |
| EC-012 | `path_template` mistakenly uses `/api/v1/organization_firewall_groups/` (full spelling) instead of `/api/v1/organization_fw_groups/` | API returns 404; E-SENSOR-001 raised (EC-016-021-005) |
| EC-013 | Query references `applied_group_pairs` by raw name on firewall_policies | E-QUERY-038; available_columns has raw_extensions, name, activity_name, comment, actor_user_name but NOT applied_group_pairs (EC-016-021-007) |
| EC-014 | `applied_zone_pairs` column name used in firewall_policies TOML instead of `applied_group_pairs` | TOML authoring defect; API returns nothing for non-existent field; structural test catches the missing key in raw_extensions (EC-016-021-010) |
| EC-015 | `count` null or absent in firewall_groups or firewall_policies envelope | Empty-page halt; no null-deref (EC-016-021-004) |
| EC-016 | Row missing `policy_name` in `claroty_organization_firewall_policies` | Null row; no hard error (EC-016-021-002) |
| EC-017 | API returns non-200 HTTP for any of the four POST endpoints | E-SENSOR-001 structured error with sensor=claroty, status, body; partial results from previously fetched pages returned |
| EC-018 | `device_conditions` is a JSON object (not array) in a given row | Serialized as JSON object in raw_extensions.device_conditions; spec-engine does not validate nested structure; no error (EC-016-020-010 pattern) |

## TOML Column-Block Specification

The complete `[[tables]]` blocks for all four tables as specified by BC-2.16.020 §PC1/§PC2
and BC-2.16.021 §PC1/§PC2:

```toml
# Wave C G5 — claroty_organization_zones
# POST /api/v1/organization_zones/ → envelope key: organization_zones (count, organization_zones)
# OCSF class: entity_management (class_uid 3004; existing arm in class_selector.rs)
# PK: zone_name (String, REQUIRED, single-column) → Arrow name
# DTU status: NONE — SAP-2 probe N/A; near-term tests against live monroe only (D-2200 deferred)
[[tables]]
table_name = "organization_zones"          # registered/queryable name = {sensor_id}_{table_name} = "claroty_organization_zones"
ocsf_class = "entity_management"   # class_uid 3004 (existing arm; same as claroty_audit_logs)

# Tier-1: zone_name → name (REQUIRED; primary key; OCSF entity_management name field)
[[tables.columns]]
name = "zone_name"
column_type = "string"
ocsf_field = "name"
options = ["REQUIRED"]

# Tier-1: zone_description → comment (free-text description of the zone)
[[tables.columns]]
name = "zone_description"
column_type = "string"
ocsf_field = "comment"

# Tier-1: enabled → status_code (zone active/inactive operational state)
[[tables.columns]]
name = "enabled"
column_type = "boolean"
ocsf_field = "status_code"

# Tier-1: updated_by → actor_user_name (email/username of analyst who last modified zone)
[[tables.columns]]
name = "updated_by"
column_type = "string"
ocsf_field = "actor.user.name"

# Tier-2: "Custom", "Recommended", or other zone source tag
[[tables.columns]]
name = "zone_source"
column_type = "string"

# Tier-2: zone priority order — numeric comparison operators supported
[[tables.columns]]
name = "priority"
column_type = "integer"

# Tier-2: array of device filter condition objects (determines zone membership) — MUST be json, not string
[[tables.columns]]
name = "device_conditions"
column_type = "json"

# Tier-2: count of devices currently matched by device_conditions
[[tables.columns]]
name = "attributed_devices"
column_type = "integer"

# Tier-2: exportable subset of attributed_devices count
[[tables.columns]]
name = "exportable_attributed_devices"
column_type = "integer"

# Tier-2: zone creation timestamp; ISO 8601; ADR-028 §D8-B implicit iso8601 default
[[tables.columns]]
name = "created_time"
column_type = "datetime"

# Tier-2: last modification timestamp; NOTE: last_update (NO trailing 'd') for zones table
# (zone_policies uses last_updated WITH trailing 'd' — per BC-2.16.020 §PC2 asymmetry note)
[[tables.columns]]
name = "last_update"
column_type = "datetime"

[[tables.steps]]
name = "fetch_organization_zones"
method = "POST"
path_template = "/api/v1/organization_zones/"
body_template = '{"fields": ["zone_name", "zone_description", "zone_source", "priority", "enabled", "device_conditions", "attributed_devices", "exportable_attributed_devices", "created_time", "last_update", "updated_by"]}'
response_path = "$.organization_zones"
variables_produced = []

[tables.steps.pagination]
type = "offset_limit"
page_size = 1000
```

```toml
# Wave C G5 — claroty_organization_zone_policies
# POST /api/v1/organization_zone_policies/ → envelope key: organization_zone_policies
# OCSF class: entity_management (class_uid 3004; existing arm; same as claroty_organization_zones)
# PK: policy_name (String, REQUIRED, single-column) → Arrow name
# DTU status: NONE — SAP-2 probe N/A; near-term tests against live monroe only (D-2200 deferred)
# NOTE: field name is last_updated (WITH trailing 'd') for zone_policies — different from zones' last_update
[[tables]]
table_name = "organization_zone_policies"  # registered/queryable name = {sensor_id}_{table_name} = "claroty_organization_zone_policies"
ocsf_class = "entity_management"   # class_uid 3004

# Tier-1: policy_name → name (REQUIRED; primary key)
[[tables.columns]]
name = "policy_name"
column_type = "string"
ocsf_field = "name"
options = ["REQUIRED"]

# Tier-1: policy_action → activity_name ("Allow" / "Deny")
[[tables.columns]]
name = "policy_action"
column_type = "string"
ocsf_field = "activity_name"

# Tier-1: policy_notes → comment (analyst notes for the policy)
[[tables.columns]]
name = "policy_notes"
column_type = "string"
ocsf_field = "comment"

# Tier-1: updated_by → actor_user_name (email/username of analyst who last modified policy)
[[tables.columns]]
name = "updated_by"
column_type = "string"
ocsf_field = "actor.user.name"

# Tier-2: "Custom", "Recommended", or other policy source tag
[[tables.columns]]
name = "policy_source"
column_type = "string"

# Tier-2: array of src/dst zone condition objects — MUST be json, not string
[[tables.columns]]
name = "communication_conditions"
column_type = "json"

# Tier-2: count of devices matching this policy
[[tables.columns]]
name = "matching_devices"
column_type = "integer"

# Tier-2: whether Claroty generates alerts when policy triggers
[[tables.columns]]
name = "should_generate_alerts"
column_type = "boolean"

# Tier-2: alert category when policy triggers (e.g. "Unknown Communication")
[[tables.columns]]
name = "alert_use_case"
column_type = "string"

# Tier-2: array of triggered alert IDs — MUST be json, not string
[[tables.columns]]
name = "related_alerts_ids"
column_type = "json"

# Tier-2: array of {src_zone, dst_zone} pair objects covered by this policy — MUST be json, not string
# NOTE: applied_zone_pairs (zone domain) — firewall_policies use applied_group_pairs instead
[[tables.columns]]
name = "applied_zone_pairs"
column_type = "json"

# Tier-2: policy creation timestamp; ISO 8601; ADR-028 §D8-B implicit iso8601 default
[[tables.columns]]
name = "created_time"
column_type = "datetime"

# Tier-2: last modification timestamp; NOTE: last_updated (WITH trailing 'd') for zone_policies
[[tables.columns]]
name = "last_updated"
column_type = "datetime"

[[tables.steps]]
name = "fetch_organization_zone_policies"
method = "POST"
path_template = "/api/v1/organization_zone_policies/"
body_template = '{"fields": ["policy_name", "policy_source", "policy_action", "communication_conditions", "matching_devices", "should_generate_alerts", "alert_use_case", "policy_notes", "related_alerts_ids", "applied_zone_pairs", "created_time", "last_updated", "updated_by"]}'
response_path = "$.organization_zone_policies"
variables_produced = []

[tables.steps.pagination]
type = "offset_limit"
page_size = 1000
```

```toml
# Wave C G5 — claroty_organization_firewall_groups
# CRITICAL: URL uses abbreviated _fw_groups; envelope key uses full organization_firewall_groups
# path_template = "/api/v1/organization_fw_groups/"   ← abbreviated
# response_path  = "$.organization_firewall_groups"   ← full spelling (NOT $.organization_fw_groups)
# Using $.organization_fw_groups in response_path causes silent empty results (EC-016-021-006)
# OCSF class: entity_management (class_uid 3004; existing arm)
# PK: firewall_group_name (String, REQUIRED, single-column) → Arrow name
# DTU status: NONE — SAP-2 probe N/A; near-term tests against live monroe only (D-2200 deferred)
[[tables]]
table_name = "organization_firewall_groups"        # registered/queryable name = {sensor_id}_{table_name} = "claroty_organization_firewall_groups"
ocsf_class = "entity_management"   # class_uid 3004

# Tier-1: firewall_group_name → name (REQUIRED; primary key)
[[tables.columns]]
name = "firewall_group_name"
column_type = "string"
ocsf_field = "name"
options = ["REQUIRED"]

# Tier-1: firewall_group_description → comment
[[tables.columns]]
name = "firewall_group_description"
column_type = "string"
ocsf_field = "comment"

# Tier-1: enabled → status_code (firewall group active/inactive)
[[tables.columns]]
name = "enabled"
column_type = "boolean"
ocsf_field = "status_code"

# Tier-1: updated_by → actor_user_name
[[tables.columns]]
name = "updated_by"
column_type = "string"
ocsf_field = "actor.user.name"

# Tier-2: "Custom", "Recommended", or other source tag
[[tables.columns]]
name = "firewall_group_source"
column_type = "string"

# Tier-2: firewall group priority order
[[tables.columns]]
name = "priority"
column_type = "integer"

# Tier-2: array of device filter condition objects — MUST be json, not string
[[tables.columns]]
name = "device_conditions"
column_type = "json"

# Tier-2: count of devices matched by device_conditions
[[tables.columns]]
name = "attributed_devices"
column_type = "integer"

# Tier-2: exportable subset of attributed_devices count
[[tables.columns]]
name = "exportable_attributed_devices"
column_type = "integer"

# Tier-2: group creation timestamp; ISO 8601; ADR-028 §D8-B implicit iso8601 default
[[tables.columns]]
name = "created_time"
column_type = "datetime"

# Tier-2: last modification timestamp; NOTE: last_update (NO trailing 'd') for fw_groups
[[tables.columns]]
name = "last_update"
column_type = "datetime"

[[tables.steps]]
name = "fetch_organization_firewall_groups"
method = "POST"
path_template = "/api/v1/organization_fw_groups/"
body_template = '{"fields": ["firewall_group_name", "firewall_group_description", "firewall_group_source", "priority", "enabled", "device_conditions", "attributed_devices", "exportable_attributed_devices", "created_time", "last_update", "updated_by"]}'
response_path = "$.organization_firewall_groups"
variables_produced = []

[tables.steps.pagination]
type = "offset_limit"
page_size = 1000
```

```toml
# Wave C G5 — claroty_organization_firewall_policies
# URL: /api/v1/organization_fw_group_policies/ (abbreviated _fw_group_policies)
# Envelope: $.organization_firewall_policies (full spelling) — same URL vs envelope asymmetry
# OCSF class: entity_management (class_uid 3004; same as claroty_organization_firewall_groups)
# PK: policy_name (String, REQUIRED, single-column) → Arrow name
# DTU status: NONE — SAP-2 probe N/A; near-term tests against live monroe only (D-2200 deferred)
# NOTE: Json column is applied_group_pairs (NOT applied_zone_pairs — that belongs to zone_policies)
[[tables]]
table_name = "organization_firewall_policies"      # registered/queryable name = {sensor_id}_{table_name} = "claroty_organization_firewall_policies"
ocsf_class = "entity_management"   # class_uid 3004

# Tier-1: policy_name → name (REQUIRED; primary key)
[[tables.columns]]
name = "policy_name"
column_type = "string"
ocsf_field = "name"
options = ["REQUIRED"]

# Tier-1: policy_action → activity_name ("Allow" / "Deny")
[[tables.columns]]
name = "policy_action"
column_type = "string"
ocsf_field = "activity_name"

# Tier-1: policy_notes → comment
[[tables.columns]]
name = "policy_notes"
column_type = "string"
ocsf_field = "comment"

# Tier-1: updated_by → actor_user_name
[[tables.columns]]
name = "updated_by"
column_type = "string"
ocsf_field = "actor.user.name"

# Tier-2: "Custom", "Recommended", or other source tag
[[tables.columns]]
name = "policy_source"
column_type = "string"

# Tier-2: array of src/dst firewall-group condition objects — MUST be json, not string
[[tables.columns]]
name = "communication_conditions"
column_type = "json"

# Tier-2: count of devices matching this firewall policy
[[tables.columns]]
name = "matching_devices"
column_type = "integer"

# Tier-2: whether Claroty generates alerts when this policy triggers
[[tables.columns]]
name = "should_generate_alerts"
column_type = "boolean"

# Tier-2: alert category when policy triggers
[[tables.columns]]
name = "alert_use_case"
column_type = "string"

# Tier-2: array of triggered alert IDs — MUST be json, not string
[[tables.columns]]
name = "related_alerts_ids"
column_type = "json"

# Tier-2: array of {src_group, dst_group} pair objects — MUST be json, not string
# NOTE: applied_group_pairs (firewall domain) — zone_policies use applied_zone_pairs instead
[[tables.columns]]
name = "applied_group_pairs"
column_type = "json"

# Tier-2: policy creation timestamp; ISO 8601; ADR-028 §D8-B implicit iso8601 default
[[tables.columns]]
name = "created_time"
column_type = "datetime"

# Tier-2: last modification timestamp; NOTE: last_updated (WITH trailing 'd') for fw_policies
[[tables.columns]]
name = "last_updated"
column_type = "datetime"

[[tables.steps]]
name = "fetch_organization_firewall_policies"
method = "POST"
path_template = "/api/v1/organization_fw_group_policies/"
body_template = '{"fields": ["policy_name", "policy_source", "policy_action", "communication_conditions", "matching_devices", "should_generate_alerts", "alert_use_case", "policy_notes", "related_alerts_ids", "applied_group_pairs", "created_time", "last_updated", "updated_by"]}'
response_path = "$.organization_firewall_policies"
variables_produced = []

[tables.steps.pagination]
type = "offset_limit"
page_size = 1000
```

## Token Budget Estimate

| Item | Estimated tokens |
|------|-----------------|
| This story spec | ~13,000 |
| `crates/prism-sensors/specs/claroty.sensor.toml` (existing 4-table baseline; may be higher at implementation time if sibling expansion stories merge first per depends_on) | ~7,500 |
| BC-2.16.020 (full) | ~9,000 |
| BC-2.16.021 (full) | ~9,000 |
| ADR-058 §B2/§C/§D sections (ocsf_column_naming flag mechanism; Tier-1/Tier-2; actor.user.name → actor_user_name) | ~4,000 |
| `crates/prism-spec-engine/src/spec_parser.rs` (ColumnSpec + FetchStep; json column_type handling) | ~3,000 |
| `crates/prism-spec-engine/src/column_mapping.rs` (ocsf_field_to_arrow_name) | ~1,500 |
| Test files (26 RGTs; 18 unit/integration + 8 live) | ~15,000 |
| endpoint-schema-extract.md §organization_zones/§organization_zone_policies/§organization_firewall_groups/§organization_firewall_policies sections | ~2,000 |
| spike-findings §Spike 3 §Tables A/B/C/D (Json column classification; Tier-1 OCSF mappings) | ~2,000 |
| **Total estimate** | **~66,000 tokens** |

Well within 20-30% of a 200K window. If context is tight at implementation time, load BC-2.16.020
and BC-2.16.021 first (they are the authoritative source for all column types, ocsf_field values,
and response_path strings), then load the existing `claroty.sensor.toml §alerts` block as the
TOML formatting reference, then load `spec_parser.rs §ColumnSpec` for the json column type
deserialization. Load both BC files before writing tests — wire-shape assertions must be
derived from BC postconditions.

## Tasks

- [ ] **Task 1 (Red Gate — test first):** Write RG-001 and RG-009: `test_BC_2_16_020_claroty_organization_zones_toml_block_parses` and `test_BC_2_16_020_claroty_organization_zone_policies_toml_block_parses` in `crates/prism-spec-engine/src/spec_parser.rs #[cfg(test)] mod tests` (or TOML fixture files). Call `SpecLoader::parse` on `claroty.sensor.toml` (or inline fixture containing the new blocks). Assert `Ok(SensorSpec)`; 11 ColumnSpec entries for zones; 13 for zone_policies; zone_policies `body_template` contains `"last_updated"` (with trailing 'd'). MUST fail before Task 9 (blocks not yet in TOML).

- [ ] **Task 2 (Red Gate — test first):** Write RG-015 and RG-021: `test_BC_2_16_021_claroty_organization_firewall_groups_toml_block_parses` and `test_BC_2_16_021_claroty_organization_firewall_policies_toml_block_parses`. For fw_groups: assert `path_template = "/api/v1/organization_fw_groups/"` AND `response_path = "$.organization_firewall_groups"` (both strings present in the parsed step — abbreviated URL, full envelope key). For fw_policies: assert `path_template = "/api/v1/organization_fw_group_policies/"` AND `response_path = "$.organization_firewall_policies"`; body_template contains `"applied_group_pairs"` (NOT `"applied_zone_pairs"`). MUST fail before Task 9.

- [ ] **Task 3 (Red Gate — test first):** Write RG-002, RG-010, RG-016, RG-022: Tier-1 column inspection tests for all four tables. For each table: assert exactly 4 `ColumnSpec` entries have non-None `ocsf_field`; verify the exact `ocsf_field` strings (`"name"` with REQUIRED for PK, `"activity_name"` for policy_action, `"comment"` for description/notes, `"actor.user.name"` for updated_by); assert remaining columns have None `ocsf_field`. All MUST fail before Task 9.

- [ ] **Task 4 (Red Gate — test first):** Write RG-006, RG-007, RG-008 (zones mock/unit), RG-013, RG-014 (zone_policies), RG-019, RG-025, RG-026 (fw mock/unit tests). These test: (a) Json columns not stringified (RG-006, RG-014, RG-026); (b) REQUIRED PK absent → null row (RG-007, RG-013, RG-019, one sub-test of RG-025); (c) count null → empty-page halt (RG-008, RG-025 second sub-test). Place in `crates/prism-sensors/tests/bc_2_16_020_claroty_org_zone_policy.rs` and `crates/prism-sensors/tests/bc_2_16_021_claroty_org_fw_policy.rs`. All MUST fail before Task 9.

- [ ] **Task 5 (Red Gate — test first):** Write RG-003, RG-004, RG-012, RG-020, RG-024 (plan-time validation tests). These test E-QUERY-038 on Tier-2 column names, Tier-1 raw TOML name rejection, and applied_group_pairs/applied_zone_pairs separation. Drive through the plan-time path. Assert E-QUERY-038 raised; assert `available_columns` memberships per each AC. All MUST fail before Task 9.

- [ ] **Task 6 (Red Gate — test first):** Write RG-005, RG-011, RG-017, RG-018, RG-023 — live Variant-1 `#[ignore]`'d integration tests. Each test includes comment: `// LIVE-MONROE-001: requires CLAROTY_INSTANCE_URL env var pointing to monroe; run manually or in live-validation CI job`. RG-017 is the fw-asymmetry confirmation test (SELECT name from fw_groups asserts non-empty result). RG-005/011/018/023 assert wire-shape per their ACs. All MUST fail (or be skipped/`#[ignore]`'d) before Task 9. Confirm that removing `#[ignore]` from RG-015/016 unit parse tests fails when TOML blocks are absent.

- [ ] **Task 7 (Pre-implementation fw-asymmetry check):** Before writing the TOML, confirm the fw URL vs envelope key asymmetry from the live monroe API (or OpenAPI spec). The schema extract §organization_firewall_groups states `path: /api/v1/organization_fw_groups/` and `envelope keys: count, organization_firewall_groups`. Confirm `response_path = "$.organization_firewall_groups"` extracts correctly from the live response. If the live API uses a different envelope key, update the TOML accordingly and route a spec amendment to product-owner (do NOT self-amend BC-2.16.021). Do not implement with an unconfirmed envelope key.

- [ ] **Task 8 (Pre-implementation Json column type check):** Confirm all 8 Json columns are typed correctly per spike-findings §Spike 3: `device_conditions` (×2), `communication_conditions` (×2), `related_alerts_ids` (×2), `applied_zone_pairs`, `applied_group_pairs`. None should be `column_type = "string"` — using String would serialize nested objects as raw string tokens. If any Json column type was uncertain from spike findings, verify against the live monroe API before writing the TOML.

- [ ] **Task 9 (Implementation — all four TOML blocks):** Add all four `[[tables]]` blocks to `crates/prism-sensors/specs/claroty.sensor.toml`. Follow the exact structures from §TOML Column-Block Specification above. Add blocks in order: `claroty_organization_zones`, then `claroty_organization_zone_policies`, then `claroty_organization_firewall_groups`, then `claroty_organization_firewall_policies`. Place all four after the current last `[[tables]]` block in the file (as of develop@3f1e66179 that is `device_alert_relations` — do NOT assume any expansion tables exist; re-verify the last block at implementation time). Include comments per the Wave C G5 and DTU-deferred conventions shown in §TOML Column-Block Specification.

  After editing: run `just iter prism-spec-engine` — RG-001, RG-002, RG-009, RG-010, RG-015, RG-016, RG-021, RG-022 MUST turn GREEN.

- [ ] **Task 10 (Implementation — verify parse + unit tests green):** Run `just iter prism-spec-engine --no-fail-fast`. Confirm all non-`#[ignore]` Red Gate tests (RG-001..RG-016, RG-019..RG-022, RG-024..RG-026 plus the non-live sub-test of RG-025) are GREEN. Confirm no existing tests regressed. Run `just iter prism-sensors` to confirm TOML is syntactically valid.

- [ ] **Task 11 (SAP-2 self-check — N/A documented for all four tables):** SAP-2 DTU-parity probe is N/A for all four tables per BC-2.16.020 §PC7, BC-2.16.021 §PC7, and D-2200 (no DTU exists for any of the four org-policy endpoints; none of these routes exist in `prism-dtu-claroty`). Record this explicitly in story comments. Do NOT create DTU routes in this story.

- [ ] **Task 12 (SAP-1 self-check):** Confirm no new `tracing::*!(event_type = ...)` emissions are added by this story (TOML-only change + unit tests). If any new emission appears during implementation, add a BC-2.16.002 catalog row per PG-LP11-001.

- [ ] **Task 13 (Final gate):** Run `just check` (full workspace). Confirm all non-`#[ignore]` Red Gate tests pass. Confirm no new `unwrap()`/`expect()` on `Result` in production code paths. Confirm `claroty.sensor.toml` gained exactly 4 new tables from this story on top of the branch-time baseline. The baseline was 4 tables as of develop@3f1e66179; if sibling expansion stories have merged before this story, the pre-story baseline will be higher — re-verify the table count before asserting a fixed total. After `just check` passes, hold for story-level holdout gate (HS-028) before pushing to origin.

## Previous Story Intelligence

1. **S-ADR058-OCSF-ROUTING-001 (merged PR #242):** Activated `ocsf_column_naming = true` at the
   sensor level in `claroty.sensor.toml`. The Tier-1/Tier-2 routing mechanism (ADR-058 §B2/§C)
   is already active for all Claroty tables. The `entity_management` / class_uid 3004 arm was
   confirmed existing in `class_selector.rs::select_by_class_name` (spike-findings §Overall
   Verdict). No new class_selector arm is needed for any of the four org-policy tables.

2. **S-CLAROTY-SERVERS-001 (Wave C G4 — materialized draft, pending):** Closest structural sibling.
   It adds two TOML blocks using the same `ocsf_column_naming = true` Tier-1/Tier-2 pattern.
   Key lessons: (a) the body_template must list ONLY fields in the API's `fields_enum` — all 11/13
   field names per table are confirmed in spike-findings §Spike 3 §Tables A/B/C/D; (b) Tier-1 raw
   TOML column names raise E-QUERY-038 (zone_name is rejected; `name` is the accepted Arrow form);
   (c) the existing `claroty.sensor.toml §alerts` block is the canonical TOML authoring pattern to
   mirror. NOTE: S-CLAROTY-SERVERS-001's TOML blocks are NOT present in the committed TOML at
   develop@3f1e66179 (this story has no depends_on on SERVERS-001).

3. **S-CLAROTY-VULNS-001, S-CLAROTY-OT-EVENTS-001, S-CLAROTY-DEVVULNREL-001 (Wave A/B — materialized draft, pending):**
   These are sibling expansion stories. NONE are merged, NONE are implemented; their TOML blocks
   do NOT exist in the committed TOML. This story's depends_on: [] is correct — the four org-policy
   tables are independent of Wave A/B tables. The implementer MUST re-verify the baseline table
   count at implementation time.

4. **S-ADR058-OCSF-COERCION-001 (merged PR #240):** Closed EC-016-013-007/008/009 (coercion path
   fixes). The org-policy tables include Datetime columns (`created_time`, `last_update`,
   `last_updated`). Verify that Datetime columns pass through the ADR-028 §D8-B implicit iso8601
   default without hitting the now-closed coercion bugs.

5. **S-DEMO-CLAROTY-TRAILING-SLASH-001 (merged):** Established that Claroty paths use trailing
   slash. All four `path_template` values in this story use trailing slash. The BC-specified paths
   are correct.

6. **Existing TOML pattern (claroty.sensor.toml §alerts):** The `alerts` table is the canonical
   TOML block pattern to mirror. Read the `alerts` block before authoring the four new blocks.

7. **Json column typing precedent (S-CLAROTY-DEVVULNREL-001, spike-findings §Nested-field
   classification principle):** Prior stories established that array-of-objects fields require
   `column_type = "json"`, not `"string"`. The eight Json columns in this story follow the same
   principle. Declaring them as `String` is a P1 TOML authoring defect per spike-findings.

8. **Wave A/B/C sibling stories are materialized drafts — not committed TOML.** Direct inspection
   of `crates/prism-sensors/specs/claroty.sensor.toml` at develop@3f1e66179 confirms 4 tables
   (`alerts`, `audit_logs`, `devices`, `device_alert_relations`); none of the xDome expansion tables
   exist in the committed TOML. This does NOT block this story. The implementer MUST re-verify the
   actual baseline table count at implementation time and treat the post-story total as baseline + 4.
   If Wave A/B/C tables are expected to land before this story, that is a scheduling question for
   the orchestrator/human to confirm.

## Architecture Compliance Rules

From `architecture/module-decomposition.md` §SS-16 Spec Engine:
- `spec_parser.rs §spec_parser` owns TOML deserialization; `ColumnSpec`, `FetchStep`,
  `PaginationConfig` are the canonical data structures. New `[[tables.columns]]` blocks with
  `column_type = "json"` must produce valid `ColumnSpec::Json` variants or `SpecParser` returns
  `Err(SpecEngineError::ConfigInvalid)`. Verify `column_type = "json"` is a recognized value in
  `ColumnType` before implementing.
- `ocsf_field_to_arrow_name` lives in `column_mapping.rs` (ADR-058 §I1). Do NOT re-implement
  the helper anywhere else. `ocsf_field_to_arrow_name("actor.user.name")` = `"actor_user_name"`.
- `PaginationConfig::OffsetLimit { page_size: 1000 }` is the correct deserialization target.

From ADR-058 §D (ocsf_column_naming flag mechanism):
- `ocsf_column_naming = true` is already declared at the sensor level in `claroty.sensor.toml`.
  New `[[tables]]` blocks inherit this setting automatically — no per-table flag needed.
- Per ADR-058 §B2: Tier-2 columns (those without `ocsf_field`) MUST aggregate into `raw_extensions`.

From BC-2.16.021 §Invariants — fw URL vs envelope key:
- `path_template` for `claroty_organization_firewall_groups` MUST use `/api/v1/organization_fw_groups/`
  (abbreviated) and `response_path` MUST use `$.organization_firewall_groups` (full spelling).
  These are NOT the same string. Mixing them produces silent data loss. This is a spec authoring
  invariant, not a runtime configuration choice.
- `path_template` for `claroty_organization_firewall_policies` MUST use
  `/api/v1/organization_fw_group_policies/` and `response_path` MUST use
  `$.organization_firewall_policies`. Same URL vs envelope key asymmetry.

From BC-2.16.021 §Invariants — `applied_group_pairs` column name:
- `claroty_organization_firewall_policies` MUST declare `name = "applied_group_pairs"`.
  Using `applied_zone_pairs` (the zone-domain column name) is a TOML authoring defect
  (EC-016-021-010) — the API returns nothing for an unknown field name, producing a silently empty
  column.

From xdome-endpoint-expansion-plan.md §Governing Directive:
- SAP-2 probe is N/A until DTU is created (D-2200). Do NOT run parity checks against
  `crates/prism-dtu-claroty/src/`. None of the four org-policy routes exist there.

## Library & Framework Requirements

| Library | Version | Source |
|---------|---------|--------|
| `prism-spec-engine` | workspace path | `SpecLoader::parse`, `ColumnSpec`, `FetchStep`, `PaginationConfig::OffsetLimit`; `ColumnType::Json` (json column_type) |
| `prism-ocsf` | workspace path | `class_selector.rs::select_by_class_name("entity_management")` → 3004 (existing arm — read only) |
| `serde_json` | per workspace Cargo.toml | Mock response construction in unit tests (Json column serialization assertions) |
| `tokio` | per workspace Cargo.toml | Async test runtime for live integration tests |

Do NOT add new Cargo.toml production dependencies. The TOML spec addition requires no new
crate imports in production code.

## File Structure Requirements

| Action | File path | Notes |
|--------|-----------|-------|
| MODIFY | `crates/prism-sensors/specs/claroty.sensor.toml` | Add FOUR `[[tables]]` blocks: `organization_zones` (bare; 11 cols), `organization_zone_policies` (bare; 13 cols), `organization_firewall_groups` (bare; 11 cols), `organization_firewall_policies` (bare; 13 cols) after the existing last table block |
| CREATE | `crates/prism-sensors/tests/bc_2_16_020_claroty_org_zone_policy.rs` | RG tests for zones + zone_policies (RG-001..RG-014 split between spec-parser and integration tests; `#[ignore]` live tests include `LIVE-MONROE-001` comment) |
| CREATE | `crates/prism-sensors/tests/bc_2_16_021_claroty_org_fw_policy.rs` | RG tests for fw_groups + fw_policies (RG-015..RG-026; `#[ignore]` live tests include `LIVE-MONROE-001` comment) |
| CREATE | `crates/prism-bin/tests/bc_2_16_020_claroty_org_zone_policy_wire_shape.rs` | Authoritative tests for BC-2.16.020: RG-027/028 zones wire-shape (class_uid=3004, null-not-absent), RG-029/030 zone_policies wire-shape, RG-003 E2E (zones E-QUERY-038 via execute), RG-012 E2E (zone_policies E-QUERY-038 via execute); SpecDrivenSensorAdapter::fetch for wire-shape tests |
| CREATE | `crates/prism-bin/tests/bc_2_16_021_claroty_org_fw_policy_wire_shape.rs` | Authoritative tests for BC-2.16.021: RG-031/032 firewall_groups wire-shape (class_uid=3004, null-not-absent), RG-033/034 firewall_policies wire-shape (applied_group_pairs not applied_zone_pairs), RG-020 E2E (fw_groups E-QUERY-038 via execute), RG-024 E2E (fw_policies E-QUERY-038 via execute) |
| MODIFY | `crates/prism-bin/Cargo.toml` | Add two `[[test]]` entries: `bc_2_16_020_claroty_org_zone_policy_wire_shape` and `bc_2_16_021_claroty_org_fw_policy_wire_shape` |

Files that MUST NOT be modified:
- `crates/prism-ocsf/src/class_selector.rs` — `entity_management` arm already exists; no changes
- `crates/prism-spec-engine/src/spec_parser.rs` — no production code changes needed; RG parse tests
  may add unit tests in-module if easier
- `crates/prism-dtu-claroty/` — read only (SAP-2 N/A; no DTU routes for these endpoints)
- `crates/prism-sensors/specs/claroty.sensor.toml` §existing tables — do not modify existing tables

## Forbidden Dependencies

`prism-sensors` MUST NOT gain any new production dependency on `prism-dtu-claroty` (SAP-2 N/A;
no DTU routes exist for these endpoints). `prism-spec-engine` MUST NOT gain a new dependency on
`prism-sensors` (direction is prism-sensors → prism-spec-engine, not reverse). If the build
gains a new dependency in either of these forbidden directions, the build MUST fail via
dependency-direction enforcement.

## Notes for Implementer

1. **Four TOML blocks, one file.** Add all four `[[tables]]` blocks in sequence:
   `claroty_organization_zones`, then `claroty_organization_zone_policies`, then
   `claroty_organization_firewall_groups`, then `claroty_organization_firewall_policies`.
   All four go in the same commit.

2. **SAP-2 DTU-parity probe is N/A for ALL four tables.** Do NOT run SAP-2 checks against
   `crates/prism-dtu-claroty/src/` — none of the four endpoints have registered routes there.
   The DTU creation stories are deferred per D-2200.

3. **The fw URL vs envelope key asymmetry is NOT an error.** The path
   `/api/v1/organization_fw_groups/` is the real OpenAPI route; `organization_firewall_groups`
   is the real response envelope key. Both MUST appear in the TOML spec — one in `path_template`,
   the other in `response_path`. Using the abbreviated key in `response_path` causes silent
   empty results, caught only by the live structural test (RG-017).

4. **`applied_group_pairs` vs `applied_zone_pairs` are table-scoped column names.** The zone_policies
   table uses `applied_zone_pairs`; the firewall_policies table uses `applied_group_pairs`. These
   are distinct column names for structurally similar Json arrays in different domain concepts.
   Do NOT use `applied_zone_pairs` in the firewall_policies block (EC-016-021-010).

5. **Datetime field name asymmetry.** The zones table and firewall_groups table use `last_update`
   (no trailing 'd'). The zone_policies table and firewall_policies table use `last_updated` (WITH
   trailing 'd'). This asymmetry is confirmed in both BCs and in the API's fields_enum responses.
   Using the wrong name per table silently omits temporal data (EC-016-020-009 / EC-016-021-009).

6. **All 8 Json columns MUST use `column_type = "json"`.** Using `"string"` for
   `device_conditions`, `communication_conditions`, `related_alerts_ids`, `applied_zone_pairs`,
   or `applied_group_pairs` is a P1 TOML authoring defect per spike-findings §Nested-field
   classification principle. The spec-engine serializes json columns as JSON-typed values;
   string-typed columns would embed the entire JSON array as a quoted string value.

7. **Live tests are `#[ignore]`'d.** Per SID-1 discipline: the non-`#[ignore]`'d TOML parse
   tests (RG-001, RG-002, RG-009, RG-010, RG-015, RG-016, RG-021, RG-022) provide unit-level
   coverage of the spec-parse path. The `#[ignore]`'d live tests (RG-005, RG-011, RG-017, RG-018,
   RG-023) exercise the full wire-shape path against the live monroe sensor.

8. **Holdout gate (HS-028) is BLOCKING.** After LOCAL adversary 3-CLEAN and BEFORE push to
   origin, the holdout-evaluator runs HS-028 (4 hidden scenarios). Do NOT read the HS-028
   scenario files — contamination control applies.

9. **Wave A/B/C sibling stories are materialized drafts — not committed TOML.** Direct inspection
   of `crates/prism-sensors/specs/claroty.sensor.toml` at develop@3f1e66179 confirms 4 tables
   only. The four new blocks from this story go after whatever the current last table block is
   at implementation time. Re-verify the baseline table count before asserting a post-story total.
   If Wave A/B/C tables have merged before this story, the baseline will be higher than 4 —
   use baseline + 4 as the expected post-story count.

---

## References

- BC-2.16.020 — §Postconditions §1 TOML contract (zones); §PC2 TOML contract (zone_policies; last_updated field name asymmetry); §PC3 Tier-1/Tier-2 zones; §PC4 Tier-1/Tier-2 zone_policies; §PC5 PK rationale; §PC6 Json column serialization; §PC7 SAP-2 N/A; EC-016-020-001..010
- BC-2.16.021 — §Postconditions §1 TOML contract (fw_groups; URL vs envelope key asymmetry critical); §PC2 TOML contract (fw_policies; applied_group_pairs NOT applied_zone_pairs); §PC3 Tier-1/Tier-2 fw_groups; §PC4 Tier-1/Tier-2 fw_policies; §PC5 PK rationale; §PC6 Json column serialization; §PC7 SAP-2 N/A; EC-016-021-001..010
- ADR-058 §B2 — Tier-2 columns aggregate into raw_extensions; §C — dot-to-underscore Arrow names (actor.user.name → actor_user_name); §D — per-sensor ocsf_column_naming flag
- spike-findings §Spike 3 §Tables A/B/C/D — Json column decisions (device_conditions, communication_conditions, related_alerts_ids, applied_zone_pairs, applied_group_pairs); Tier-1 OCSF mappings for all 4 tables — AUTHORITATIVE
- spike-findings §Overall Verdict — entity_management/3004 arm confirmed existing; no new arm required
- xdome-endpoint-expansion-plan.md §Gap Table G5 — Wave C scope authority (4 org-policy tables); §Per-Story Pipeline — no-DTU live test approach; §Governing Directive — DTU skip directive
- endpoint-schema-extract.md §organization_zones / §organization_zone_policies / §organization_firewall_groups / §organization_firewall_policies — envelope keys + URL confirmation; fw URL vs envelope key asymmetry confirmed
- `crates/prism-sensors/specs/claroty.sensor.toml §alerts` — canonical TOML block pattern to mirror
- S-ADR058-OCSF-ROUTING-001 (merged PR #242) — activated ocsf_column_naming=true; entity_management arm confirmed existing

---

## Changelog

| Version | Date | Author | Notes |
|---------|------|--------|-------|
| 1.6 | 2026-09-01 | story-writer | §Acceptance Criteria AC-003/AC-012/AC-020/AC-024 §Test citations corrected to cite authoritative prism-bin end-to-end tests (RG-003/RG-012/RG-020/RG-024, via QueryEngine::execute); sensor-side equivalents noted as defense-in-depth per SAP-3 rule 3. input-hash updated to b68ddba (BC input drift). No AC coverage or BC-trace changes. |
| 1.5 | 2026-08-31 | story-writer | §Red Gate Tests RG-004 type label corrected from "Integration (plan-time)" to "Unit (ocsf_projected_column_names helper; defense-in-depth per SAP-3 rule 3)"; What-it-gates note appended that RG-003 (`test_BC_2_16_020_claroty_organization_zones_e2e_e_query_038_tier2_column`, prism-bin end-to-end) is the authoritative E-QUERY-038 gate that transitively covers the raw Tier-1 name case — RG-004 is defense-in-depth per SAP-3 rule 3. No AC coverage or BC-trace changes. |
| 1.4 | 2026-08-31 | story-writer | FIX 1 (POL-39): Removed volatile BC version pins from §Authority (BC-2.16.020 §Postconditions §1..§4 + BC-2.16.021 §Postconditions §1..§4 headings), §Token Budget (both BC rows), and §References; §Behavioral Contracts table Version columns synced to v1.2 per POL-40 current-state pin. FIX 2 (crates_touched): Removed prism-spec-engine — delivered tests live in crates/prism-sensors/tests/ and crates/prism-bin/tests/ (no prism-spec-engine file modified per worktree verification). |
| 1.3 | 2026-08-31 | story-writer | FIX A: §TOML Column-Block Specification — 48 `column_name =` occurrences (11+13+11+13 across 4 tables) changed to `name =`; AC-024 body prose and §Architecture Compliance Rules updated accordingly. FIX B: §Red Gate Tests — RG-003/012/020/024 test names corrected to `…e2e_e_query_038_tier2_column` form (delivered ground truth); RG-027 (single combined wire-shape row) replaced with 8 authoritative fetch-path tests (RG-027..RG-034) across two delivered files (`bc_2_16_020_claroty_org_zone_policy_wire_shape.rs` + `bc_2_16_021_claroty_org_fw_policy_wire_shape.rs`); density updated 27→34 RGTs, ratio 1.04→1.31. FIX C: §File Structure Requirements prism-bin CREATE entry split from 1 combined file to 2 BC-scoped files; Cargo.toml MODIFY note updated to two `[[test]]` entries. |
| 1.2 | 2026-08-31 | story-writer | G2-proven spec-prose corrections applied (mirrors S-CLAROTY-OT-EVENTS-001 v1.3 fixes). FIX 1 (MED-1 table_name): all four `[[tables]]` blocks now use bare table_name (`organization_zones`, `organization_zone_policies`, `organization_firewall_groups`, `organization_firewall_policies`); §Authority, all four TOML blocks, and AC-001/AC-009/AC-015/AC-021 updated; derivation note `{sensor_id}_{table_name}` = registered/queryable name added throughout; SELECT examples and error prose retain prefixed queryable names unchanged. FIX 2 (MED-3 mechanism attribution): N/A — no ColumnMapper::map_record references in this story. FIX 3 (MED-4 prism-bin declaration): `crates_touched` adds `prism-bin`; RG-003/RG-012/RG-020/RG-024 updated to prism-bin end-to-end authoritative (plan-time prism-sensors tests remain defense-in-depth per SAP-3 rule 3); RG-027 added (wire-shape serialization assertion in prism-bin for Json columns); density check 26→27 RGTs / 26 ACs = 1.04 PASS; §File Structure Requirements adds CREATE `crates/prism-bin/tests/bc_2_16_020_021_claroty_org_policy_wire_shape.rs` + MODIFY `crates/prism-bin/Cargo.toml`; TOML block description updated to bare names. FIX 4 (§Architecture Mapping path): `crates/prism-spec-engine/src/spec_driven_adapter.rs §pipeline_result_to_record_batch` corrected to `crates/prism-bin/src/spec_driven_adapter.rs §pipeline_result_to_record_batch` (contains `build_column_array`). |
| 1.1 | 2026-08-31 | research-agent | Remove-uncertainty pass (also satisfies mandatory pre-delivery pass D-1110). Validated all technology/API assumptions in the story and both BCs against ground truth: endpoint-schema-extract.md field enums (OrganizationZones, OrganizationZonePolicies, OrganizationFirewallGroups, OrganizationFirewallGroupPolicies); endpoint-spike-findings.md §Spike 3 Tables A–D; the xDome OpenAPI schema extract; `class_selector.rs::select_by_class_name`; `crates/prism-dtu-claroty/src`. Findings: all 11/13/11/13 `body_template` fields present in their respective field enums; all four endpoint paths, envelope keys, and `response_path` values confirmed (fw URL↔envelope-key asymmetry verified for both firewall tables — abbreviated `_fw_` path vs full `organization_firewall_` envelope key); `entity_management`→class_uid 3004 arm confirmed present (no new arm required); all 8 Json columns confirmed against §Spike 3; per-table `last_update`/`last_updated` datetime field-name asymmetry confirmed; `applied_zone_pairs` (zone_policies) vs `applied_group_pairs` (firewall_policies) table-scoping confirmed; omitted `timestamp_formats` (ADR-028 §D8-B implicit iso8601 default, SAP-2 datetime arm c) valid; SAP-2 N/A confirmed (none of the four org-policy routes exist in prism-dtu-claroty); baseline table count confirmed = 4 committed Claroty tables (alerts, audit_logs, devices, device_alert_relations) at current develop HEAD. No corrections required to the story body — all 26 ACs, the RG list, §Architecture Mapping, and §TOML Column-Block Specification match ground truth exactly. Companion BCs corrected and bumped to v1.1 (BC-2.16.020/BC-2.16.021 §Invariants `available_columns` was missing `actor_user_name` for zones/firewall_groups; BC-INDEX H1 title drift corrected per POLICY 7). Story version bumped 1.0→1.1; status remains draft per dispatch. |
| 1.0 | 2026-08-24 | story-writer | Initial authoring — F3 story materialization for S-CLAROTY-ORGPOLICY-001 (Wave C G5). BC-2.16.020 v1.0 + BC-2.16.021 v1.0 traceability; 4 TOML table blocks: claroty_organization_zones (11 cols: 4 Tier-1 [zone_name→name REQUIRED, zone_description→comment, enabled→status_code, updated_by→actor_user_name]; 7 Tier-2 incl. 1 Json: device_conditions); claroty_organization_zone_policies (13 cols: 4 Tier-1 [policy_name→name REQUIRED, policy_action→activity_name, policy_notes→comment, updated_by→actor_user_name]; 9 Tier-2 incl. 3 Json: communication_conditions, related_alerts_ids, applied_zone_pairs; last_updated WITH trailing d); claroty_organization_firewall_groups (11 cols: 4 Tier-1 same structure; 7 Tier-2 incl. 1 Json: device_conditions; URL /api/v1/organization_fw_groups/ vs envelope $.organization_firewall_groups asymmetry documented); claroty_organization_firewall_policies (13 cols: 4 Tier-1 same structure; 9 Tier-2 incl. 3 Json: communication_conditions, related_alerts_ids, applied_group_pairs; NOT applied_zone_pairs); 8 Json columns total; 26 ACs; 26 RGTs; density 1.0; SAC-1 compliant; SAC-2 N/A (no ADR authored by this story); SAP-2 N/A per D-2200 for all 4 tables; live-test approach per xdome-endpoint-expansion-plan.md §Per-Story Pipeline; TOML column-block specs embedded per both BCs; HS-028 holdout gate BLOCKING; depends_on: []. |
