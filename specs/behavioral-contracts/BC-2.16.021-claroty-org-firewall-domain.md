---
document_type: behavioral-contract
level: L3
version: "1.2"
status: draft
producer: product-owner
timestamp: 2026-08-24T00:00:00Z
phase: 3
origin: brownfield
subsystem: "SS-16"
capability: "CAP-029"
lifecycle_status: draft
inputs:
  - ".factory/objectives/xdome-endpoint-expansion-plan.md"
  - ".factory/objectives/xdome-v1-validation/endpoint-spike-findings.md"
  - ".factory/objectives/xdome-v1-validation/endpoint-schema-extract.md"
  - ".factory/specs/domain-spec/capabilities.md"
  - ".factory/specs/architecture/decisions/ADR-058-v1-column-naming-col-name-as-arrow-field-identifier.md"
  - "crates/prism-sensors/specs/claroty.sensor.toml"
input-hash: "5213907"
traces_to: ["CAP-029"]
extracted_from: ".factory/objectives/xdome-v1-validation/endpoint-spike-findings.md"
introduced: "2026-08-24"
modified: "2026-08-31"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.16.021: Claroty xDome Organization Firewall Domain — Firewall Groups and Firewall Group Policies Queryable Surface with OCSF entity_management Mapping (No DTU)

## Description

Two `[[tables]]` blocks in `claroty.sensor.toml` — `claroty_organization_firewall_groups` and
`claroty_organization_firewall_policies` — expose Claroty xDome network firewall governance records
as queryable PrismQL tables. They form one behavioral contract (the **Firewall Domain**) because a
firewall group and its associated policies constitute a single management contract: the firewall group
defines which devices are grouped under a shared firewall policy scope (via `device_conditions`), and
firewall group policies govern what communications are permitted or denied between firewall group pairs
(via `applied_group_pairs`). Both tables use `entity_management` (class_uid 3004, existing arm) as
their OCSF class. Under `ocsf_column_naming = true`, the primary-key column (`firewall_group_name` /
`policy_name`) maps to the Tier-1 OCSF field `name` (Arrow: `name`, REQUIRED), along with up to three
additional Tier-1 mappings per table. The firewall_groups table carries 1 Json column
(`device_conditions`); the firewall_policies table carries 3 Json columns
(`communication_conditions`, `related_alerts_ids`, `applied_group_pairs`). No DTU exists for either
endpoint; near-term tests run against the live monroe sensor only (D-2200 deferred DTU anchor).

## BC Structure Rationale — Domain-Pairing

This BC is the structural mirror of BC-2.16.020 (Zone Domain) for the firewall subsystem. Both
BCs use the same domain-pairing rationale: a firewall group and its policies form one cohesive
management contract, exactly as a zone and its policies do. For a full four-point rationale
(cohesive management contract, structural symmetry, burst-size discipline, contrast with per-table
pattern), see BC-2.16.020 §BC Structure Rationale — Domain-Pairing. That rationale applies
identically here with "zone" replaced by "firewall group".

**Firewall vs Zone semantic distinction (for MSSP context):** Claroty xDome distinguishes between
network zones (segment-based access policy using `applied_zone_pairs`) and firewall groups
(firewall-enforcement-point-based access policy using `applied_group_pairs`). The zone domain is
used for logical network segmentation; the firewall domain is used for enforcement-point-based
ACL governance. Despite their structural similarity, they are separate Claroty features surfaced
via distinct API endpoints (`/api/v1/organization_fw_groups/` vs `/api/v1/organization_zones/`),
justifying distinct BCs.

## Preconditions

- `claroty.sensor.toml` includes both the `claroty_organization_firewall_groups` and
  `claroty_organization_firewall_policies` `[[tables]]` blocks as specified in S-CLAROTY-ORGPOLICY-001
- `ocsf_column_naming = true` is declared at the sensor level in `claroty.sensor.toml`
- The `entity_management` / class_uid 3004 arm exists in
  `prism-ocsf/src/class_selector.rs::select_by_class_name` (existing arm — same arm as BC-2.16.020;
  no new arm required per spike findings §Overall Verdict)
- The Claroty bearer token credential is configured for the requesting client
- S-PLUGIN-PREREQ-A through S-PLUGIN-PREREQ-E have all merged (spec-engine pipeline active)
- The spec-engine's Json column serialization pipeline handles `column_type = "json"` columns
  by serializing nested arrays/objects into `raw_extensions` as JSON-typed values (existing
  behavior — no new mechanism required)

## Postconditions

### 1. TOML Table Contract — claroty_organization_firewall_groups

The `claroty_organization_firewall_groups` table MUST be declared in `claroty.sensor.toml` with:

```toml
[[tables]]
table_name = "organization_firewall_groups"  # bare name; TableRegistry derives the registered/queryable name as {sensor_id}_{table_name} = "claroty_organization_firewall_groups"
ocsf_class = "entity_management"   # class_uid 3004 (existing arm)
```

**Step definition:**

```toml
[[tables.steps]]
name = "fetch_organization_firewall_groups"
method = "POST"
path_template = "/api/v1/organization_fw_groups/"
body_template = '{"fields": ["firewall_group_name", "firewall_group_description", \
  "firewall_group_source", "priority", "enabled", "device_conditions", \
  "attributed_devices", "exportable_attributed_devices", \
  "created_time", "last_update", "updated_by"]}'
response_path = "$.organization_firewall_groups"
variables_produced = []
[tables.steps.pagination]
type = "offset_limit"
page_size = 1000
```

**URL vs envelope key asymmetry (critical):** The API path is `/api/v1/organization_fw_groups/`
(uses `_fw_groups` abbreviation) but the response envelope key is `organization_firewall_groups`
(spelled out). The `response_path` MUST use `$.organization_firewall_groups`. Using
`$.organization_fw_groups` will produce an empty result with no runtime error — a silent data
loss defect. This asymmetry is confirmed in schema extract §organization_firewall_groups
(`path: /api/v1/organization_fw_groups/`, `envelope keys: count, organization_firewall_groups`).

The `count` field is present in the envelope; if null or absent, pagination halts via empty-page
check (EC-016-021-004).

### 2. TOML Table Contract — claroty_organization_firewall_policies

The `claroty_organization_firewall_policies` table MUST be declared in `claroty.sensor.toml` with:

```toml
[[tables]]
table_name = "organization_firewall_policies"  # bare name; TableRegistry derives the registered/queryable name as {sensor_id}_{table_name} = "claroty_organization_firewall_policies"
ocsf_class = "entity_management"   # class_uid 3004 (existing arm; same as claroty_organization_firewall_groups)
```

**Step definition:**

```toml
[[tables.steps]]
name = "fetch_organization_firewall_policies"
method = "POST"
path_template = "/api/v1/organization_fw_group_policies/"
body_template = '{"fields": ["policy_name", "policy_source", "policy_action", \
  "communication_conditions", "matching_devices", "should_generate_alerts", \
  "alert_use_case", "policy_notes", "related_alerts_ids", "applied_group_pairs", \
  "created_time", "last_updated", "updated_by"]}'
response_path = "$.organization_firewall_policies"
variables_produced = []
[tables.steps.pagination]
type = "offset_limit"
page_size = 1000
```

**URL vs envelope key asymmetry (critical):** The API path is
`/api/v1/organization_fw_group_policies/` (uses `_fw_group_policies` abbreviation) but the
response envelope key is `organization_firewall_policies`. The `response_path` MUST use
`$.organization_firewall_policies`. Confirmed in schema extract §organization_firewall_policies
(`path: /api/v1/organization_fw_group_policies/`, `envelope keys: count, organization_firewall_policies`).

**Datetime field name asymmetry (spec authoring note):** The firewall_groups table uses
`last_update` (no trailing 'd'); the firewall_policies table uses `last_updated` (with trailing
'd'). Both confirmed in the schema extract OrganizationFirewallGroups fields_enum and
OrganizationFirewallGroupPolicies fields_enum respectively. This is the same asymmetry as the
Zone Domain (BC-2.16.020 §PC2 note). Implementer MUST use the exact field name per-table.

### 3. Column Tier Classification — claroty_organization_firewall_groups (ADR-058)

Under `ocsf_column_naming = true`, columns for `claroty_organization_firewall_groups` are classified as:

**Tier-1 columns** (have `ocsf_field`; exposed as Arrow field name =
`ocsf_field_to_arrow_name(ocsf_field)`):

| Column (TOML name) | ColumnType | ocsf_field | Arrow Field Name | Options |
|--------------------|-----------|------------|-----------------|---------|
| `firewall_group_name` | String | `name` | `name` | REQUIRED |
| `firewall_group_description` | String | `comment` | `comment` | — |
| `enabled` | Boolean | `status_code` | `status_code` | — |
| `updated_by` | String | `actor.user.name` | `actor_user_name` | — |

**Tier-2 columns** (no `ocsf_field`; values aggregate into `raw_extensions` JSON object):

| Column (TOML name) | ColumnType | Notes |
|--------------------|-----------|-------|
| `firewall_group_source` | String | "Custom", "Recommended", or other source tag |
| `priority` | Integer | Firewall group priority ordering — numeric comparison operators supported |
| `device_conditions` | **Json** | Array of device filter condition objects (determines firewall group membership) |
| `attributed_devices` | Integer | Count of devices currently matched by device_conditions |
| `exportable_attributed_devices` | Integer | Exportable subset of attributed_devices count |
| `created_time` | Datetime | ISO 8601; ADR-028 §D8-B implicit iso8601 default |
| `last_update` | Datetime | ISO 8601; field name is `last_update` (no trailing 'd') — see §PC2 note |

**Total declared columns (firewall_groups):** 11 (4 Tier-1, 7 Tier-2).  
All 11 fields are from the OrganizationFirewallGroups fields_enum confirmed in schema extract
§OrganizationFirewallGroups (field count: 11).  
**Json columns:** 1 (`device_conditions` — array of device filter objects per spike findings §Spike 3 §Table C).

### 4. Column Tier Classification — claroty_organization_firewall_policies (ADR-058)

Under `ocsf_column_naming = true`, columns for `claroty_organization_firewall_policies` are classified as:

**Tier-1 columns** (have `ocsf_field`; exposed as Arrow field name =
`ocsf_field_to_arrow_name(ocsf_field)`):

| Column (TOML name) | ColumnType | ocsf_field | Arrow Field Name | Options |
|--------------------|-----------|------------|-----------------|---------|
| `policy_name` | String | `name` | `name` | REQUIRED |
| `policy_action` | String | `activity_name` | `activity_name` | — |
| `policy_notes` | String | `comment` | `comment` | — |
| `updated_by` | String | `actor.user.name` | `actor_user_name` | — |

**Tier-2 columns** (no `ocsf_field`; values aggregate into `raw_extensions` JSON object):

| Column (TOML name) | ColumnType | Notes |
|--------------------|-----------|-------|
| `policy_source` | String | "Custom", "Recommended", or other source tag |
| `communication_conditions` | **Json** | Array of src/dst firewall-group condition objects (governs which group pairs this policy matches) |
| `matching_devices` | Integer | Count of devices matching this firewall policy |
| `should_generate_alerts` | Boolean | Whether Claroty generates alerts when this policy triggers |
| `alert_use_case` | String | Alert category when triggered |
| `related_alerts_ids` | **Json** | Array of triggered alert IDs (integers or UUIDs) |
| `applied_group_pairs` | **Json** | Array of `{src_group, dst_group}` pair objects indicating which firewall group pairs this policy covers |
| `created_time` | Datetime | ISO 8601; ADR-028 §D8-B implicit iso8601 default |
| `last_updated` | Datetime | ISO 8601; field name is `last_updated` (with trailing 'd') — see §PC2 note |

**Total declared columns (firewall_policies):** 13 (4 Tier-1, 9 Tier-2).  
All 13 fields are from the OrganizationFirewallGroupPolicies fields_enum confirmed in schema
extract §OrganizationFirewallGroupPolicies (field count: 13).  
**Json columns:** 3 (`communication_conditions`, `related_alerts_ids`, `applied_group_pairs`
per spike findings §Spike 3 §Table D).

### 5. Primary Keys and OCSF Mapping Rationale

**Primary key: `firewall_group_name` (String, REQUIRED, single-column) for `claroty_organization_firewall_groups`**

`firewall_group_name` uniquely identifies each firewall enforcement group in the Claroty xDome
instance. It maps to the OCSF `entity_management` `name` field (Arrow: `name`, REQUIRED), which
is the canonical OCSF identifier for the entity being managed. A firewall group is a managed entity
in the enforcement-point governance framework — its deployment name is the natural OCSF `name`.

**Primary key: `policy_name` (String, REQUIRED, single-column) for `claroty_organization_firewall_policies`**

`policy_name` uniquely identifies each firewall group communication policy. Maps to OCSF
`entity_management` `name` (Arrow: `name`, REQUIRED). Same mapping rationale as BC-2.16.020
§PC5 zone_policies.

**OCSF Tier-1 mapping rationale (applies to both tables):**

The four Tier-1 mappings are identical in structure to BC-2.16.020 §PC5 with the following
table-specific substitutions:

- **`firewall_group_name` / `policy_name` → `name` (Arrow: `name`):** OCSF `entity_management`
  primary entity identifier. A firewall group or policy name is the canonical identifier.

- **`firewall_group_description` / `policy_notes` → `comment` (Arrow: `comment`):** Free-text
  analyst notes and descriptions map to OCSF `comment`.

- **`enabled` → `status_code` (Arrow: `status_code`):** Firewall group active/inactive state.
  (Firewall_policies do not have `enabled`; this Tier-1 mapping applies to
  `claroty_organization_firewall_groups` only.)

- **`policy_action` → `activity_name` (Arrow: `activity_name`):** Allow/Deny firewall governance
  action. (Firewall_groups do not have `policy_action`; this mapping applies to
  `claroty_organization_firewall_policies` only.)

- **`updated_by` → `actor.user.name` (Arrow: `actor_user_name`):** Last analyst to modify
  the entity; maps to OCSF `actor.user.name`.

**No new `class_selector` arm required:** Both tables use the existing `entity_management` arm
(class_uid 3004) — the same arm already used by BC-2.16.020 and the existing `claroty_audit_logs`
table.

### 6. Json Column Serialization Behavior

The four Json columns across both tables (`device_conditions`, `communication_conditions`,
`related_alerts_ids`, `applied_group_pairs`) follow the same serialization behavior as the Zone
Domain (BC-2.16.020 §PC6). They are serialized into `raw_extensions` as JSON-typed values by the
existing spec-engine pipeline. Empty arrays serialize as `[]`, not null
(EC-016-021-003). Declaring these columns as `String` instead of `Json` is a P1 TOML authoring
defect (same invariant as BC-2.16.020).

**`applied_group_pairs` vs `applied_zone_pairs`:** These are structurally equivalent Json arrays
but refer to different domain concepts (firewall group pairs vs zone pairs). The column name
`applied_group_pairs` is the `claroty_organization_firewall_policies` table's equivalent of
`applied_zone_pairs` in `claroty_organization_zone_policies`. Both serialize identically; only
the semantic content differs (group names vs zone names in the pair objects).

### 7. SAP-2 DTU Parity Status

SAP-2 probe is **N/A** for both tables (no DTU exists for either endpoint per
xdome-endpoint-expansion-plan.md §Governing Directive and §Deferred DTU-Creation Stories).
The deferred DTU creation story is tracked as D-2200. Once the DTU story for the organization
firewall domain executes, SAP-2 probe applies retroactively and this BC MUST be amended with:
- DTU route file references (`crates/prism-dtu-claroty/src/routes/organization_fw_groups.rs` and
  `crates/prism-dtu-claroty/src/routes/organization_fw_group_policies.rs`)
- DTU types.rs field equivalencies for all contracted columns per table
- SAP-2 exclusion documentation for any deliberately excluded fields

Until the DTU story executes, near-term tests run against the live monroe sensor only.

## Invariants

- DI-005: OCSF schema validity — `entity_management` class_uid 3004 is a valid OCSF class
- `firewall_group_name` (String, REQUIRED) is the primary key for
  `claroty_organization_firewall_groups`; absent `firewall_group_name` produces a null row per
  spec-engine REQUIRED semantics (not a hard error)
- `policy_name` (String, REQUIRED) is the primary key for
  `claroty_organization_firewall_policies`; same null-row REQUIRED semantics
- Tier-2 columns (all columns without `ocsf_field`) are NOT exposed as standalone Arrow columns;
  a PrismQL query referencing them by raw TOML name MUST raise E-QUERY-038 with
  `available_columns` containing `raw_extensions`, `name`, `comment`, `status_code`,
  `actor_user_name` (firewall_groups) or `name`, `activity_name`, `comment`, `actor_user_name`
  (firewall_policies), `class_uid`, `_sensor` — but NOT the raw Tier-2 column name
- All four Json columns (`device_conditions`, `communication_conditions`, `related_alerts_ids`,
  `applied_group_pairs`) MUST be declared with `column_type = "json"` in the TOML spec; declaring
  them as `String` is a P1 TOML authoring defect (spike findings §Nested-field classification
  principle)
- The `path_template` for `claroty_organization_firewall_groups` MUST use
  `/api/v1/organization_fw_groups/` (abbreviated) and the `response_path` MUST use
  `$.organization_firewall_groups` (full spelling) — these are NOT the same string; mixing
  them produces a silent data loss (empty result with no error)
- The `path_template` for `claroty_organization_firewall_policies` MUST use
  `/api/v1/organization_fw_group_policies/` and the `response_path` MUST use
  `$.organization_firewall_policies` — same URL vs envelope key asymmetry
- `last_update` (firewall_groups) and `last_updated` (firewall_policies) are distinct field names
  (same asymmetry as BC-2.16.020); using the wrong name produces a silently empty column
- Datetime fields use ADR-028 §D8-B implicit iso8601 default; null passthrough when absent

## Error Cases

| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-SENSOR-001` | Claroty API returns non-200 HTTP for POST /api/v1/organization_fw_groups/ or POST /api/v1/organization_fw_group_policies/ | Structured error with sensor=claroty, status, body; partial results returned for previously fetched pages |
| `E-QUERY-038` | Query references `firewall_group_source`, `priority`, `device_conditions`, `communication_conditions`, `policy_source`, `related_alerts_ids`, `applied_group_pairs`, or any other Tier-2 column by its raw TOML name | Column-not-found at plan time; `available_columns` lists Tier-1 Arrow names, `raw_extensions`, `class_uid`, `_sensor` |
| `E-SPEC-018` | Datetime parse failure for `created_time`, `last_update`, or `last_updated` for a non-null non-ISO-8601 value | `E-SPEC-018 TimestampParseFailure` — null demoted with warning; row continues |

No new error codes are required. All failure modes are covered by existing codes. The URL vs
envelope key asymmetry for firewall paths is a spec-authoring concern (use correct
`response_path`), not a new runtime error mode.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-016-021-001 | Row in `claroty_organization_firewall_groups` missing `firewall_group_name` field (REQUIRED) | Null row produced; no hard error; subsequent rows continue |
| EC-016-021-002 | Row in `claroty_organization_firewall_policies` missing `policy_name` field (REQUIRED) | Null row produced; no hard error; subsequent rows continue |
| EC-016-021-003 | `device_conditions`, `communication_conditions`, `related_alerts_ids`, or `applied_group_pairs` is an empty array `[]` | Serialized as `[]` JSON in `raw_extensions`; not null; no error |
| EC-016-021-004 | `count` is null or absent in the response envelope for either endpoint | Pagination continues via empty-page check (empty page → halt); not an error |
| EC-016-021-005 | `path_template` mistakenly uses `/api/v1/organization_firewall_groups/` (full spelling) instead of `/api/v1/organization_fw_groups/` | API returns 404; `E-SENSOR-001` raised; the TOML spec path must match the actual OpenAPI route |
| EC-016-021-006 | `response_path` mistakenly uses `$.organization_fw_groups` (abbreviated) instead of `$.organization_firewall_groups` | Spec-engine receives empty extraction path result; produces empty result set with no error — silent data loss; caught by structural live-sensor test asserting non-empty result |
| EC-016-021-007 | Query references Tier-2 column `applied_group_pairs` by raw name | E-QUERY-038; `available_columns` contains `raw_extensions`, `name`, `activity_name`, `comment`, `actor_user_name` but NOT `applied_group_pairs` |
| EC-016-021-008 | `enabled` is null or absent in a firewall_groups row | Null `status_code` Arrow cell; not an error |
| EC-016-021-009 | Implementer uses `last_updated` for firewall_groups instead of `last_update` | Column silently absent; same as EC-016-020-009 |
| EC-016-021-010 | `applied_group_pairs` and `applied_zone_pairs` column names confused between tables | TOML spec authoring defect; the firewall_policies table MUST use `applied_group_pairs` (not `applied_zone_pairs`); using the wrong name produces an empty column |

## Related BCs

- BC-2.16.013: Bundled Sensor Spec Authoring — parent spec for the Claroty sensor; this BC adds
  the `claroty_organization_firewall_groups` and `claroty_organization_firewall_policies` tables
  to the Claroty sensor surface (depends on)
- BC-2.16.020: Claroty xDome Organization Zone Domain — structural sibling; same domain-pairing
  pattern, same OCSF class, same pagination, same Tier structure; delivered in the same story
  S-CLAROTY-ORGPOLICY-001 (sibling)
- BC-2.02.005: Claroty xDome Field Mapping to OCSF (9 Data Sources) — OCSF class mapping for
  all Claroty sources; `entity_management` class_uid 3004 covered (composes with)
- BC-2.01.007: Claroty Bearer Token Auth — auth mechanism unchanged; preconditions satisfied
  (depends on)

## Architecture Anchors

- `crates/prism-sensors/specs/claroty.sensor.toml` — TOML spec file authoring target
- `crates/prism-spec-engine/src/spec_parser.rs` — ColumnSpec (column_type Json), FetchStep
  deserialization
- `crates/prism-spec-engine/src/pipeline.rs` — OffsetLimit POST-body injection; `response_path`
  extraction; Json column serialization into `raw_extensions`
- `crates/prism-ocsf/src/class_selector.rs::select_by_class_name` — `"entity_management"` arm
  (existing; resolves to class_uid 3004)
- `crates/prism-bin/src/spec_driven_adapter.rs` — `pipeline_result_to_record_batch`
- `.factory/reference/api-specs/xdome_openapi_06.20.2026.json §/api/v1/organization_fw_groups/` and
  `§/api/v1/organization_fw_group_policies/` — endpoint authority (URL uses `_fw_` abbreviation;
  envelope keys use full `organization_firewall_` spelling)
- `.factory/objectives/xdome-endpoint-expansion-plan.md §Gap Table G5` — table scope authority
- `.factory/objectives/xdome-v1-validation/endpoint-spike-findings.md §Spike 3` — typing
  decisions (Json/Integer/Boolean/Datetime) and Tier-1 OCSF mappings — AUTHORITATIVE

## Story Anchor

S-CLAROTY-ORGPOLICY-001 (draft — Wave C)

## VP Anchors

(none — no formal verification properties defined; structural tests via story RG list per
S-CLAROTY-ORGPOLICY-001; holdout evaluator exercises live monroe surface via HS-028)

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.16.021-001 | `SELECT name FROM claroty.claroty_organization_firewall_groups LIMIT 5` against live monroe | Succeeds (no E-QUERY-038); rows have non-null `name` strings (firewall group names) |
| TV-BC-2.16.021-002 | `SELECT * FROM claroty.claroty_organization_firewall_groups LIMIT 1` | Wire JSON: `class_uid = 3004`; `name` present; `status_code` present or null; `raw_extensions` object present with `firewall_group_source`, `priority`, `device_conditions` keys |
| TV-BC-2.16.021-003 | `SELECT firewall_group_name FROM claroty.claroty_organization_firewall_groups LIMIT 1` | E-QUERY-038; `available_columns` contains `name`, `comment`, `status_code`, `actor_user_name`, `raw_extensions`; does NOT contain `firewall_group_name` |
| TV-BC-2.16.021-004 | `SELECT applied_group_pairs FROM claroty.claroty_organization_firewall_policies LIMIT 1` | E-QUERY-038; `available_columns` contains `raw_extensions` but NOT `applied_group_pairs` |
| TV-BC-2.16.021-005 | `SELECT raw_extensions FROM claroty.claroty_organization_firewall_policies LIMIT 5` | Succeeds; `raw_extensions` JSON contains `communication_conditions`, `related_alerts_ids`, `applied_group_pairs` keys (all Json columns aggregated) |
| TV-BC-2.16.021-006 | `SELECT name FROM claroty.claroty_organization_firewall_policies LIMIT 5` | Succeeds; rows have non-null `name` strings (policy names) |
| TV-BC-2.16.021-007 | `SELECT * FROM claroty.claroty_organization_firewall_policies LIMIT 1` | Wire JSON: `class_uid = 3004`; `name` present; `activity_name` present or null ("Allow"/"Deny"); `comment` present or null; `raw_extensions` with `communication_conditions`, `applied_group_pairs`, `related_alerts_ids` keys |
| TV-BC-2.16.021-008 | Response envelope with null `count` for either endpoint | Pagination terminates on empty page; no error |
| TV-BC-2.16.021-009 | `response_path = "$.organization_fw_groups"` (abbreviated, incorrect) | Empty result set; no runtime error — caught by structural test asserting non-empty result from live sensor |

## Verification Properties

| VP | Verification Aspect |
|----|---------------------|
| (none) | No VP directly verifies this BC — structural tests cover via story RG list per S-CLAROTY-ORGPOLICY-001; holdout evaluator exercises live monroe surface via HS-028 |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-029 |
| Capability Anchor Justification | CAP-029 ("Config-Driven Sensor Adapters") per capabilities.md §CAP-029 — this BC specifies the TOML table contract for two Claroty xDome tables (`claroty_organization_firewall_groups` and `claroty_organization_firewall_policies`), defining 11+13 columns (typed with ColumnOptions and OCSF mappings), multi-step fetch pipelines (POST-for-read, offset_limit pagination, `organization_firewall_groups` and `organization_firewall_policies` envelope keys), Tier-1/Tier-2 OCSF column classification per ADR-058 (4 Tier-1 per table into name/comment/status_code/actor_user_name or name/activity_name/comment/actor_user_name; 7+9 Tier-2 into raw_extensions including Json columns), PK rationale (firewall_group_name/policy_name → entity_management name REQUIRED), URL vs envelope key asymmetry documentation (path uses _fw_ abbreviation; envelope uses full spelling), Json column typing decisions from spike findings §Spike 3 Table C/D, and SAP-2 N/A documentation (no DTU; D-2200 deferred DTU anchor). This is exactly what CAP-029 defines: sensor adapters defined in TOML spec files with tables, columns, pipelines, and pagination config. |
| L2 Invariants | DI-005 |
| Priority | P0 |
| Story | S-CLAROTY-ORGPOLICY-001 |
| DTU Status | NONE — no DTU exists for either endpoint; near-term tests against live monroe sensor only; DTU deferred to D-2200 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.2 | g3-g4-g5-spec-prose-corrections | 2026-08-31 | product-owner | MED-1: §Postconditions §1 (firewall_groups) and §2 (firewall_policies) TOML bare table_names corrected: `"claroty_organization_firewall_groups"` → `"organization_firewall_groups"` and `"claroty_organization_firewall_policies"` → `"organization_firewall_policies"`; added derivation notes (`{sensor_id}_{table_name}` = registered/queryable name). Architecture anchor: §Architecture Anchors `spec_driven_adapter.rs` crate corrected `crates/prism-spec-engine` → `crates/prism-bin` (ground truth: `pipeline_result_to_record_batch` lives in `crates/prism-bin/src/spec_driven_adapter.rs`). FIX 2 not applicable — no `ColumnMapper::map_record` attribution present. |
| 1.1 | xdome-wave-c-remove-uncertainty | 2026-08-31 | research-agent | Remove-uncertainty pass (satisfies mandatory pre-delivery pass D-1110). Validated every TOML/API assumption against ground truth (endpoint-schema-extract.md OrganizationFirewallGroups + OrganizationFirewallGroupPolicies fields_enums; endpoint-spike-findings.md §Spike 3 Tables C/D; the xDome OpenAPI schema extract): all 11+13 `body_template` fields present in the respective field enums; endpoint paths, envelope keys, and `response_path` values confirmed — fw URL↔envelope-key asymmetry verified (`/api/v1/organization_fw_groups/` ↔ `$.organization_firewall_groups`; `/api/v1/organization_fw_group_policies/` ↔ `$.organization_firewall_policies`); `entity_management`/3004 arm confirmed present in `class_selector.rs::select_by_class_name`; 4 Json columns confirmed against §Spike 3 (device_conditions ×1 fw_groups; communication_conditions + related_alerts_ids + applied_group_pairs fw_policies); `last_update` vs `last_updated` datetime field-name asymmetry confirmed; `applied_group_pairs` (not `applied_zone_pairs`) confirmed; omitted `timestamp_formats` (ADR-028 §D8-B implicit iso8601 default, SAP-2 datetime arm c) valid; SAP-2 N/A re-confirmed (no fw routes exist in prism-dtu-claroty). CORRECTION: §Invariants firewall_groups `available_columns` enumeration was missing `actor_user_name`, inconsistent with §PC3, TV-BC-2.16.021-003, and story AC-020 — added. BC-INDEX H1 title drift corrected (POLICY 7). input-hash refreshed (input files drifted since initial authoring). No content/mechanism defects found. |
| 1.0 | xdome-wave-c-f2-spec-evolution | 2026-08-24 | product-owner | Initial authoring — Claroty xDome Firewall Domain (firewall_groups + firewall_policies) queryable surface contract per xdome-endpoint-expansion-plan.md Wave C G5. Structural mirror of BC-2.16.020 for the firewall subsystem. Domain-pairing rationale references BC-2.16.020. TOML table contracts for both tables with URL vs envelope key asymmetry documented (path `/api/v1/organization_fw_groups/` → envelope `$.organization_firewall_groups`; path `/api/v1/organization_fw_group_policies/` → envelope `$.organization_firewall_policies`). Column Tier classification: firewall_groups (11 cols: 4 Tier-1 [firewall_group_name→name REQUIRED, firewall_group_description→comment, enabled→status_code, updated_by→actor_user_name]; 7 Tier-2 including 1 Json: device_conditions); firewall_policies (13 cols: 4 Tier-1 [policy_name→name REQUIRED, policy_action→activity_name, policy_notes→comment, updated_by→actor_user_name]; 9 Tier-2 including 3 Json: communication_conditions, related_alerts_ids, applied_group_pairs). Datetime field name asymmetry noted (last_update vs last_updated — same as Zone Domain). OCSF class: entity_management/3004 (existing arm). No new error codes. SAP-2 N/A (no DTU; D-2200 deferred DTU anchor). HS-028 holdout group registered with 4 P0 scenarios for S-CLAROTY-ORGPOLICY-001. |
