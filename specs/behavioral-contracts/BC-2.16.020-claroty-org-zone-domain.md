---
document_type: behavioral-contract
level: L3
version: "1.1"
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
input-hash: "2b1ff87"
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

# BC-2.16.020: Claroty xDome Organization Zone Domain — Zones and Zone Policies Queryable Surface with OCSF entity_management Mapping (No DTU)

## Description

Two `[[tables]]` blocks in `claroty.sensor.toml` — `claroty_organization_zones` and
`claroty_organization_zone_policies` — expose Claroty xDome network zone governance records as
queryable PrismQL tables. They form one behavioral contract (the **Zone Domain**) because a zone
and its associated policies constitute a single management contract: the zone defines which devices
belong to a network segment (via `device_conditions`), and zone policies govern what communications
are permitted or denied between zone pairs (via `applied_zone_pairs`). Both tables use `entity_management`
(class_uid 3004, existing arm) as their OCSF class. Under `ocsf_column_naming = true`, the
primary-key column (`zone_name` / `policy_name`) maps to the Tier-1 OCSF field `name` (Arrow: `name`,
REQUIRED), along with up to three additional Tier-1 mappings per table. The zones table carries 1
Json column (`device_conditions`); the zone_policies table carries 3 Json columns
(`communication_conditions`, `related_alerts_ids`, `applied_zone_pairs`). No DTU exists for either
endpoint; near-term tests run against the live monroe sensor only (D-2200 deferred DTU anchor).

## BC Structure Rationale — Domain-Pairing

This BC covers two tables (zones + zone_policies) rather than one. The domain-pairing is justified
because:

1. **Cohesive management contract:** A zone defines *what* devices are in a network segment;
   zone policies define *what communications are allowed/denied between zone pairs*. MSSP operators
   query them together (e.g., "what are the zones on this segment and what policies govern their
   traffic?"). Splitting them into two BCs would fragment a naturally cohesive behavioral surface.

2. **Structural symmetry with Firewall domain:** BC-2.16.021 mirrors this contract for the
   firewall group + firewall group policy pair. The two domains are architecturally identical
   (same OCSF class, same pagination, same Tier pattern) — domain-pairing in both BCs maintains
   a clean hierarchy.

3. **Burst-size discipline:** This F2 burst must produce ≤8 artifacts for G5's 4 tables. Two
   domain-paired BCs (zones + firewalls) achieve this without sacrificing contract granularity.

4. **Contrast with per-table pattern:** BC-2.16.018/019 (servers + server_interfaces) used
   separate BCs because those tables have different endpoints, different OCSF classes, and
   independent use cases. Here, zones and zone_policies share the same OCSF class, the same
   pagination pattern, and the same management domain — domain-pairing is semantically correct.

## Preconditions

- `claroty.sensor.toml` includes both the `claroty_organization_zones` and
  `claroty_organization_zone_policies` `[[tables]]` blocks as specified in S-CLAROTY-ORGPOLICY-001
- `ocsf_column_naming = true` is declared at the sensor level in `claroty.sensor.toml`
- The `entity_management` / class_uid 3004 arm exists in
  `prism-ocsf/src/class_selector.rs::select_by_class_name` (existing arm — no new arm required
  per spike findings §Overall Verdict)
- The Claroty bearer token credential is configured for the requesting client
- S-PLUGIN-PREREQ-A through S-PLUGIN-PREREQ-E have all merged (spec-engine pipeline active)
- The spec-engine's Json column serialization pipeline handles `column_type = "json"` columns
  by serializing nested arrays/objects into `raw_extensions` as JSON-typed values (existing
  behavior — no new mechanism required)

## Postconditions

### 1. TOML Table Contract — claroty_organization_zones

The `claroty_organization_zones` table MUST be declared in `claroty.sensor.toml` with:

```toml
[[tables]]
table_name = "claroty_organization_zones"
ocsf_class = "entity_management"   # class_uid 3004 (existing arm)
```

**Step definition:**

```toml
[[tables.steps]]
name = "fetch_organization_zones"
method = "POST"
path_template = "/api/v1/organization_zones/"
body_template = '{"fields": ["zone_name", "zone_description", "zone_source", "priority", \
  "enabled", "device_conditions", "attributed_devices", \
  "exportable_attributed_devices", "created_time", "last_update", "updated_by"]}'
response_path = "$.organization_zones"
variables_produced = []
[tables.steps.pagination]
type = "offset_limit"
page_size = 1000
```

**Envelope key:** `organization_zones` — confirmed in schema extract §organization_zones
(`envelope keys: count, organization_zones`). Response carries a `count` field; if `count` is
null or absent, pagination halts via empty-page check (EC-016-020-004).

### 2. TOML Table Contract — claroty_organization_zone_policies

The `claroty_organization_zone_policies` table MUST be declared in `claroty.sensor.toml` with:

```toml
[[tables]]
table_name = "claroty_organization_zone_policies"
ocsf_class = "entity_management"   # class_uid 3004 (existing arm; same as claroty_organization_zones)
```

**Step definition:**

```toml
[[tables.steps]]
name = "fetch_organization_zone_policies"
method = "POST"
path_template = "/api/v1/organization_zone_policies/"
body_template = '{"fields": ["policy_name", "policy_source", "policy_action", \
  "communication_conditions", "matching_devices", "should_generate_alerts", \
  "alert_use_case", "policy_notes", "related_alerts_ids", "applied_zone_pairs", \
  "created_time", "last_updated", "updated_by"]}'
response_path = "$.organization_zone_policies"
variables_produced = []
[tables.steps.pagination]
type = "offset_limit"
page_size = 1000
```

**Envelope key:** `organization_zone_policies` — confirmed in schema extract
§organization_zone_policies (`envelope keys: count, organization_zone_policies`).

**Datetime field name asymmetry (spec authoring note):** The zones table uses `last_update`
(no trailing 'd'); the zone_policies table uses `last_updated` (with trailing 'd'). Both are
confirmed in the schema extract OrganizationZones fields_enum and OrganizationZonePolicies
fields_enum respectively. Implementer MUST use the exact field name per-table.

### 3. Column Tier Classification — claroty_organization_zones (ADR-058)

Under `ocsf_column_naming = true`, columns for `claroty_organization_zones` are classified as:

**Tier-1 columns** (have `ocsf_field`; exposed as Arrow field name =
`ocsf_field_to_arrow_name(ocsf_field)`):

| Column (TOML name) | ColumnType | ocsf_field | Arrow Field Name | Options |
|--------------------|-----------|------------|-----------------|---------|
| `zone_name` | String | `name` | `name` | REQUIRED |
| `zone_description` | String | `comment` | `comment` | — |
| `enabled` | Boolean | `status_code` | `status_code` | — |
| `updated_by` | String | `actor.user.name` | `actor_user_name` | — |

**Tier-2 columns** (no `ocsf_field`; values aggregate into `raw_extensions` JSON object):

| Column (TOML name) | ColumnType | Notes |
|--------------------|-----------|-------|
| `zone_source` | String | "Custom", "Recommended", or other source tag |
| `priority` | Integer | Zone priority ordering — numeric comparison operators supported |
| `device_conditions` | **Json** | Array of device filter condition objects (determines zone membership) |
| `attributed_devices` | Integer | Count of devices currently matched by device_conditions |
| `exportable_attributed_devices` | Integer | Exportable subset of attributed_devices count |
| `created_time` | Datetime | ISO 8601; ADR-028 §D8-B implicit iso8601 default |
| `last_update` | Datetime | ISO 8601; field name is `last_update` (no trailing 'd') — see §PC2 note |

**Total declared columns (zones):** 11 (4 Tier-1, 7 Tier-2).  
All 11 fields are from the OrganizationZones fields_enum confirmed in schema extract
§OrganizationZones (field count: 11).  
**Json columns:** 1 (`device_conditions` — array of device filter objects per spike findings §Spike 3 §Table A).

### 4. Column Tier Classification — claroty_organization_zone_policies (ADR-058)

Under `ocsf_column_naming = true`, columns for `claroty_organization_zone_policies` are classified as:

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
| `communication_conditions` | **Json** | Array of src/dst zone condition objects (governs which zone pairs this policy matches) |
| `matching_devices` | Integer | Count of devices matching this policy |
| `should_generate_alerts` | Boolean | Whether Claroty generates alerts when policy triggers |
| `alert_use_case` | String | Alert category when triggered (e.g., "Unknown Communication") |
| `related_alerts_ids` | **Json** | Array of triggered alert IDs (integers or UUIDs) |
| `applied_zone_pairs` | **Json** | Array of `{src_zone, dst_zone}` pair objects indicating which zone pairs this policy covers |
| `created_time` | Datetime | ISO 8601; ADR-028 §D8-B implicit iso8601 default |
| `last_updated` | Datetime | ISO 8601; field name is `last_updated` (with trailing 'd') — see §PC2 note |

**Total declared columns (zone_policies):** 13 (4 Tier-1, 9 Tier-2).  
All 13 fields are from the OrganizationZonePolicies fields_enum confirmed in schema extract
§OrganizationZonePolicies (field count: 13).  
**Json columns:** 3 (`communication_conditions`, `related_alerts_ids`, `applied_zone_pairs`
per spike findings §Spike 3 §Table B).

### 5. Primary Keys and OCSF Mapping Rationale

**Primary key: `zone_name` (String, REQUIRED, single-column) for `claroty_organization_zones`**

`zone_name` uniquely identifies each network zone in the Claroty xDome instance. It maps to the
OCSF `entity_management` `name` field (Arrow: `name`), which is the canonical OCSF identifier
for the entity being managed. A network zone is precisely the kind of managed entity described by
the `entity_management` class (3004), and its deployment name is the natural OCSF `name`.

**Primary key: `policy_name` (String, REQUIRED, single-column) for `claroty_organization_zone_policies`**

`policy_name` uniquely identifies each zone communication policy. It maps to the same OCSF
`entity_management` `name` field (Arrow: `name`, REQUIRED) — the policy itself is the managed
entity being recorded in this context.

**OCSF Tier-1 mapping rationale (applies to both tables):**

- **`zone_name` / `policy_name` → `name` (Arrow: `name`):** The OCSF `entity_management` class
  represents management events on entities. The primary identifier of the entity being managed is
  `name`. A zone or zone policy's name is the natural identifier for the management record.

- **`zone_description` / `policy_notes` → `comment` (Arrow: `comment`):** OCSF `entity_management`
  carries `comment` for free-text analyst notes and descriptive context about the entity. These
  text fields are a direct semantic match.

- **`enabled` → `status_code` (Arrow: `status_code`):** OCSF `entity_management` carries
  `status_code` for the operational status of the entity. Zone active/inactive state maps
  directly. (Zone policies do not have an `enabled` field; this Tier-1 mapping applies to
  `claroty_organization_zones` only.)

- **`policy_action` → `activity_name` (Arrow: `activity_name`):** The `activity_name` OCSF field
  describes the management activity being recorded. A zone policy's Allow/Deny action is the
  management activity that governs the communication pair. (Zone groups do not have a
  `policy_action` equivalent; this mapping applies to `claroty_organization_zone_policies` only.)

- **`updated_by` → `actor.user.name` (Arrow: `actor_user_name`):** OCSF `entity_management` has
  an `actor` object capturing who performed the management action. `updated_by` (email or
  username of the last analyst to modify the entity) maps to `actor.user.name`.

**No new `class_selector` arm required:** Both tables use the existing `entity_management` arm
(class_uid 3004) in `prism-ocsf/src/class_selector.rs::select_by_class_name` per
xdome-endpoint-expansion-plan.md §Governing Directive.

### 6. Json Column Serialization Behavior

The four Json columns across both tables (`device_conditions`, `communication_conditions`,
`related_alerts_ids`, `applied_zone_pairs`) are nested arrays/objects from the Claroty API response.
The spec-engine's pipeline serializes them into `raw_extensions` as JSON-typed values when
`ocsf_column_naming = true` and `column_type = "json"` is declared. This is existing behavior —
no new mechanism is required. PrismQL operators can access them via:

```sql
SELECT raw_extensions FROM claroty.claroty_organization_zones LIMIT 5
-- raw_extensions.device_conditions is a JSON array value
```

An empty array `[]` for any Json column is serialized as `[]`, not null (EC-016-020-003).

### 7. SAP-2 DTU Parity Status

SAP-2 probe is **N/A** for both tables (no DTU exists for either endpoint per
xdome-endpoint-expansion-plan.md §Governing Directive and §Deferred DTU-Creation Stories).
The deferred DTU creation story is tracked as D-2200. Once the DTU story for the organization
zone domain executes, SAP-2 probe applies retroactively and this BC MUST be amended with:
- DTU route file references (`crates/prism-dtu-claroty/src/routes/organization_zones.rs` and
  `crates/prism-dtu-claroty/src/routes/organization_zone_policies.rs`)
- DTU types.rs field equivalencies for all contracted columns per table
- SAP-2 exclusion documentation for any deliberately excluded fields

Until the DTU story executes, near-term tests run against the live monroe sensor only (see
xdome-endpoint-expansion-plan.md §Per-Story Pipeline).

## Invariants

- DI-005: OCSF schema validity — `entity_management` class_uid 3004 is a valid OCSF class
- `zone_name` (String, REQUIRED) is the primary key for `claroty_organization_zones`; absent
  `zone_name` produces a null row per spec-engine REQUIRED semantics (not a hard error; subsequent
  rows continue)
- `policy_name` (String, REQUIRED) is the primary key for `claroty_organization_zone_policies`;
  same null-row REQUIRED semantics
- Tier-2 columns (all columns without `ocsf_field`) are NOT exposed as standalone Arrow columns;
  a PrismQL query referencing them by raw TOML name (e.g., `WHERE zone_source = 'Custom'`) MUST
  raise E-QUERY-038 with `available_columns` containing `raw_extensions`, `name`, `comment`,
  `status_code`, `actor_user_name` (zones) or `name`, `activity_name`, `comment`, `actor_user_name`
  (zone_policies), `class_uid`, `_sensor` — but NOT the raw Tier-2 column name
- All four Json columns (`device_conditions`, `communication_conditions`, `related_alerts_ids`,
  `applied_zone_pairs`) MUST be declared with `column_type = "json"` in the TOML spec; declaring
  them as `String` would cause the nested object/array to be serialized as a raw string token,
  not a JSON value — this is a P1 TOML authoring defect (spike findings §Nested-field classification
  principle)
- `last_update` (zones) and `last_updated` (zone_policies) are distinct field names (asymmetry
  confirmed in OrganizationZones vs OrganizationZonePolicies fields_enum); using the wrong name
  produces an empty column with no runtime error (the API simply omits the field from the response)
- Datetime fields use ADR-028 §D8-B implicit iso8601 default (`timestamp_formats` omitted →
  `effective_formats` returns `["iso8601"]`); null passthrough when field is absent

## Error Cases

| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-SENSOR-001` | Claroty API returns non-200 HTTP for POST /api/v1/organization_zones/ or POST /api/v1/organization_zone_policies/ | Structured error with sensor=claroty, status, body; partial results returned for previously fetched pages |
| `E-QUERY-038` | Query references `zone_source`, `priority`, `device_conditions`, `communication_conditions`, `policy_source`, `related_alerts_ids`, `applied_zone_pairs`, or any other Tier-2 column by its raw TOML name | Column-not-found at plan time; `available_columns` lists Tier-1 Arrow names, `raw_extensions`, `class_uid`, `_sensor` |
| `E-SPEC-018` | Datetime parse failure for `created_time`, `last_update`, or `last_updated` for a non-null non-ISO-8601 value | `E-SPEC-018 TimestampParseFailure` — null demoted with warning; row continues |

No new error codes are required for these tables. Json column serialization (nested arrays/objects)
uses existing spec-engine behavior and does not introduce new failure modes. All failure modes are
covered by existing codes.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-016-020-001 | Row in `claroty_organization_zones` missing `zone_name` field (REQUIRED) | Null row produced; no hard error; subsequent rows continue |
| EC-016-020-002 | Row in `claroty_organization_zone_policies` missing `policy_name` field (REQUIRED) | Null row produced; no hard error; subsequent rows continue |
| EC-016-020-003 | `device_conditions`, `communication_conditions`, `related_alerts_ids`, or `applied_zone_pairs` is an empty array `[]` | Serialized as `[]` JSON in `raw_extensions`; not null; no error |
| EC-016-020-004 | `count` is null or absent in the response envelope for either endpoint | Pagination continues via empty-page check (empty page → halt); not an error; same as EC-016-015-003 pattern |
| EC-016-020-005 | Query references Tier-2 column `zone_source` by raw name | E-QUERY-038; `available_columns` contains `raw_extensions`, `name`, `comment`, `status_code`, `actor_user_name` but NOT `zone_source` |
| EC-016-020-006 | Query references Tier-2 column `applied_zone_pairs` by raw name | E-QUERY-038; `available_columns` contains `raw_extensions`, `name`, `activity_name`, `comment`, `actor_user_name` but NOT `applied_zone_pairs` |
| EC-016-020-007 | `enabled` is null or absent in a zones row | Null `status_code` Arrow cell; not an error |
| EC-016-020-008 | `policy_action` is absent in a zone_policies row | Null `activity_name` Arrow cell; not an error |
| EC-016-020-009 | Implementer uses `last_updated` for the zones table instead of `last_update` | Column silently absent (API returns nothing for a non-existent field name); no runtime error but temporal data lost — caught by structural live-sensor test asserting `last_update` key present in `raw_extensions` |
| EC-016-020-010 | `device_conditions` is a JSON object (not array) in a given row | Serialized as JSON object in `raw_extensions.device_conditions`; spec-engine does not validate the nested structure beyond raw serialization; no error |

## Related BCs

- BC-2.16.013: Bundled Sensor Spec Authoring — parent spec for the Claroty sensor; this BC adds
  the `claroty_organization_zones` and `claroty_organization_zone_policies` tables to the Claroty
  sensor surface (depends on)
- BC-2.16.021: Claroty xDome Organization Firewall Domain — mirrors this BC for the
  `claroty_organization_firewall_groups` + `claroty_organization_firewall_policies` pair; both
  domains delivered in S-CLAROTY-ORGPOLICY-001, both use entity_management/3004 (sibling)
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
- `crates/prism-spec-engine/src/spec_driven_adapter.rs` — `pipeline_result_to_record_batch`
- `.factory/reference/api-specs/xdome_openapi_06.20.2026.json §/api/v1/organization_zones/` and
  `§/api/v1/organization_zone_policies/` — endpoint authority
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
| TV-BC-2.16.020-001 | `SELECT name FROM claroty.claroty_organization_zones LIMIT 5` against live monroe | Succeeds (no E-QUERY-038); rows have non-null `name` strings (zone names) |
| TV-BC-2.16.020-002 | `SELECT * FROM claroty.claroty_organization_zones LIMIT 1` | Wire JSON: `class_uid = 3004`; `name` present; `comment` present or null; `status_code` present or null; `actor_user_name` present or null; `raw_extensions` object present with `zone_source`, `priority`, `device_conditions` keys |
| TV-BC-2.16.020-003 | `SELECT zone_name FROM claroty.claroty_organization_zones LIMIT 1` | E-QUERY-038; `available_columns` contains `name`, `comment`, `status_code`, `actor_user_name`, `raw_extensions`; does NOT contain `zone_name` |
| TV-BC-2.16.020-004 | `SELECT device_conditions FROM claroty.claroty_organization_zones LIMIT 1` | E-QUERY-038; `available_columns` contains `raw_extensions` but NOT `device_conditions` |
| TV-BC-2.16.020-005 | `SELECT raw_extensions FROM claroty.claroty_organization_zones LIMIT 5` | Succeeds; `raw_extensions` JSON contains `device_conditions` key (value is a JSON array or `[]`) |
| TV-BC-2.16.020-006 | `SELECT name FROM claroty.claroty_organization_zone_policies LIMIT 5` | Succeeds; rows have non-null `name` strings (policy names) |
| TV-BC-2.16.020-007 | `SELECT * FROM claroty.claroty_organization_zone_policies LIMIT 1` | Wire JSON: `class_uid = 3004`; `name` present; `activity_name` present or null ("Allow"/"Deny"); `raw_extensions` with `communication_conditions`, `applied_zone_pairs`, `related_alerts_ids` keys |
| TV-BC-2.16.020-008 | `SELECT applied_zone_pairs FROM claroty.claroty_organization_zone_policies LIMIT 1` | E-QUERY-038; `available_columns` contains `raw_extensions` but NOT `applied_zone_pairs` |
| TV-BC-2.16.020-009 | Response envelope with null `count` for either endpoint | Pagination terminates on empty page; no error |

## Verification Properties

| VP | Verification Aspect |
|----|---------------------|
| (none) | No VP directly verifies this BC — structural tests cover via story RG list per S-CLAROTY-ORGPOLICY-001; holdout evaluator exercises live monroe surface via HS-028 |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-029 |
| Capability Anchor Justification | CAP-029 ("Config-Driven Sensor Adapters") per capabilities.md §CAP-029 — this BC specifies the TOML table contract for two Claroty xDome tables (`claroty_organization_zones` and `claroty_organization_zone_policies`), defining 11+13 columns (typed with ColumnOptions and OCSF mappings), multi-step fetch pipelines (POST-for-read, offset_limit pagination, `organization_zones` and `organization_zone_policies` envelope keys), Tier-1/Tier-2 OCSF column classification per ADR-058 (4 Tier-1 per table into name/comment/status_code/actor_user_name or name/activity_name/comment/actor_user_name; 7+9 Tier-2 into raw_extensions including Json columns), PK rationale (zone_name/policy_name → entity_management name REQUIRED), Json column typing decisions from spike findings §Spike 3, and SAP-2 N/A documentation (no DTU; D-2200 deferred DTU anchor). This is exactly what CAP-029 defines: sensor adapters defined in TOML spec files with tables, columns, pipelines, and pagination config. |
| L2 Invariants | DI-005 |
| Priority | P0 |
| Story | S-CLAROTY-ORGPOLICY-001 |
| DTU Status | NONE — no DTU exists for either endpoint; near-term tests against live monroe sensor only; DTU deferred to D-2200 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.1 | xdome-wave-c-remove-uncertainty | 2026-08-31 | research-agent | Remove-uncertainty pass (satisfies mandatory pre-delivery pass D-1110). Validated every TOML/API assumption against ground truth (endpoint-schema-extract.md OrganizationZones + OrganizationZonePolicies fields_enums; endpoint-spike-findings.md §Spike 3 Tables A/B; the xDome OpenAPI schema extract): all 11+13 `body_template` fields present in the respective field enums; endpoint paths (`/api/v1/organization_zones/`, `/api/v1/organization_zone_policies/`), envelope keys (`organization_zones`, `organization_zone_policies`), and `response_path` values confirmed; `entity_management`/3004 arm confirmed present in `class_selector.rs::select_by_class_name`; 4 Json columns confirmed against §Spike 3 (device_conditions ×1 zones; communication_conditions + related_alerts_ids + applied_zone_pairs zone_policies); `last_update` (zones) vs `last_updated` (zone_policies) datetime field-name asymmetry confirmed; `applied_zone_pairs` confirmed; omitted `timestamp_formats` (ADR-028 §D8-B implicit iso8601 default, SAP-2 datetime arm c) valid; SAP-2 N/A re-confirmed (no zone routes exist in prism-dtu-claroty); baseline Claroty table count confirmed = 4 committed tables (alerts, audit_logs, devices, device_alert_relations). CORRECTION: §Invariants zones `available_columns` enumeration was missing `actor_user_name`, inconsistent with §PC3, EC-016-020-005, TV-BC-2.16.020-002/003, and story AC-003 — added. BC-INDEX H1 title drift corrected (POLICY 7). input-hash refreshed (input files drifted since initial authoring). No content/mechanism defects found. |
| 1.0 | xdome-wave-c-f2-spec-evolution | 2026-08-24 | product-owner | Initial authoring — Claroty xDome Zone Domain (zones + zone_policies) queryable surface contract per xdome-endpoint-expansion-plan.md Wave C G5. Domain-pairing rationale documented (4 points). TOML table contracts for both tables with envelope keys and pagination. Column Tier classification: zones (11 cols: 4 Tier-1 [zone_name→name REQUIRED, zone_description→comment, enabled→status_code, updated_by→actor_user_name]; 7 Tier-2 including 1 Json: device_conditions); zone_policies (13 cols: 4 Tier-1 [policy_name→name REQUIRED, policy_action→activity_name, policy_notes→comment, updated_by→actor_user_name]; 9 Tier-2 including 3 Json: communication_conditions, related_alerts_ids, applied_zone_pairs). Datetime field name asymmetry noted (last_update vs last_updated). OCSF class: entity_management/3004 (existing arm). PK rationale for both tables. No new error codes. SAP-2 N/A (no DTU; D-2200 deferred DTU anchor). HS-028 holdout group registered with 4 P0 scenarios for S-CLAROTY-ORGPOLICY-001. |
