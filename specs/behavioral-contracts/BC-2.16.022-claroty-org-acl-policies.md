---
document_type: behavioral-contract
level: L3
version: "1.0"
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
input-hash: "a4f5f7b"
traces_to: ["CAP-029"]
extracted_from: ".factory/objectives/xdome-v1-validation/endpoint-spike-findings.md"
introduced: "2026-08-24"
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.16.022: Claroty xDome Organization ACL Policies — Non-Paginated Single-Page Fetch with Mandatory policy_acl_syntax and OCSF entity_management Mapping (No DTU)

## Description

One `[[tables]]` block in `claroty.sensor.toml` — `claroty_organization_acl_policies` — exposes
Claroty xDome ACL policy records as a queryable PrismQL table. This table has a structurally
atypical fetch contract: it uses `PaginationConfig::None` (`type = "none"` in TOML) rather than
the `offset_limit` pagination used by all other Claroty tables. The endpoint returns all ACL
policies in a single HTTP response with no `count` field in the envelope. Additionally, the POST
body requires a mandatory `policy_acl_syntax` request parameter (not a query field — it controls
the format of the `policy_acl` response field); this parameter is hardcoded to `"Cisco dACL"` in
v1. The table uses `entity_management` (class_uid 3004, existing arm) as its OCSF class. Under
`ocsf_column_naming = true`, four Tier-1 OCSF mappings are declared (including `policy_id →
metadata.uid` as the REQUIRED primary key), with the remaining seven columns aggregated into
`raw_extensions` as Tier-2 — including one Json column (`applied_models`, an array of device
model strings). No DTU exists for this endpoint; near-term tests run against the live monroe
sensor only (D-2200 deferred DTU anchor).

## Preconditions

- `claroty.sensor.toml` includes the `claroty_organization_acl_policies` `[[tables]]` block as
  specified in S-CLAROTY-ACLPOLICY-001
- `ocsf_column_naming = true` is declared at the sensor level in `claroty.sensor.toml`
- The `entity_management` / class_uid 3004 arm exists in
  `prism-ocsf/src/class_selector.rs::select_by_class_name` (existing arm — the same arm used by
  BC-2.16.020 [zones], BC-2.16.021 [firewall groups], and `claroty_audit_logs`; no new arm
  required per spike findings §Overall Verdict)
- The Claroty bearer token credential is configured for the requesting client
- S-PLUGIN-PREREQ-A through S-PLUGIN-PREREQ-E have all merged (spec-engine pipeline active)
- `PaginationConfig::None` is a valid variant in `prism-spec-engine/src/spec_parser.rs`
  (confirmed in spike findings §Spike 4: "PaginationConfig::None in spec_parser.rs, decorated
  with `#[serde(tag = "type", rename_all = "snake_case")]`")
- The spec-engine pipeline does NOT inject `offset`/`limit` into POST bodies when
  `PaginationConfig::None` is declared — the `build_request` function in `pipeline.rs` routes
  on `PaginationConfig` variant before constructing the request body

## Postconditions

### 1. TOML Table Contract — claroty_organization_acl_policies

The `claroty_organization_acl_policies` table MUST be declared in `claroty.sensor.toml` with:

```toml
[[tables]]
table_name = "claroty_organization_acl_policies"
ocsf_class = "entity_management"   # class_uid 3004 (existing arm)
```

**Step definition:**

```toml
[[tables.steps]]
name = "fetch_organization_acl_policies"
method = "POST"
path_template = "/api/v1/organization_acl_policies/"
body_template = '{"policy_acl_syntax": "Cisco dACL", "fields": ["policy_id", "policy_name", \
  "policy_source", "applied_models", "matching_devices", "policy_acl_type", "policy_acl", \
  "policy_creation_date", "policy_last_updated", "policy_updated_by", "policy_notes"]}'
response_path = "$.organization_acl_policies"
variables_produced = []
[tables.steps.pagination]
type = "none"
```

**Pagination contract (critical — key novelty):** `type = "none"` explicitly declared. This is
the only Claroty table using non-paginated fetch. The pipeline MUST NOT inject `offset`/`limit`
into the POST body — these fields are absent from the `GetOrganizationAclPoliciesRequest` schema
(confirmed in spike findings §Spike 4). Injecting `offset`/`limit` would cause an API validation
error or unexpected behavior. The API returns the complete ACL policy list in a single response.

**Response envelope:** `$.organization_acl_policies` — key `organization_acl_policies` confirmed
in schema extract §organization_acl_policies (`envelope keys: organization_acl_policies` — **NO
count field**). The absence of `count` is confirmed and expected. Pagination halt-on-empty is not
applicable for `PaginationConfig::None`; the single response is the complete result set.

**Mandatory request parameter — policy_acl_syntax:** The `policy_acl_syntax` field in the POST
body is a REQUIRED parameter in the `GetOrganizationAclPoliciesRequest` OpenAPI schema
(`required: ["policy_acl_syntax"]`). It is NOT in the fields_enum — it is a request-level
parameter that controls the text format of the `policy_acl` response field. Valid values:
"Cisco dACL", "AireOS", "ArubaOS-Switch", "ArubaOS-CX". For v1, it is hardcoded to
`"Cisco dACL"` (the API example default and most common MSSP network equipment syntax). The
`policy_acl` column in the query results will contain ACL text in Cisco dACL format.

**v1 deferral note:** Configurable `policy_acl_syntax` per-table is deferred to a follow-up
story. Making it configurable would require new TOML schema extensions beyond v1 scope. MSSP
tenants needing ArubaOS or AireOS ACL format must wait for the follow-up story. The deferred
scope is NOT a tech-debt-register entry (not a fix but a feature extension).

### 2. Column Tier Classification (ADR-058)

Under `ocsf_column_naming = true`, columns for `claroty_organization_acl_policies` are
classified as:

**Tier-1 columns** (have `ocsf_field`; exposed as Arrow field name =
`ocsf_field_to_arrow_name(ocsf_field)`):

| Column (TOML name) | ColumnType | ocsf_field | Arrow Field Name | Options |
|--------------------|-----------|------------|-----------------|---------|
| `policy_id` | String | `metadata.uid` | `metadata_uid` | REQUIRED |
| `policy_name` | String | `name` | `name` | — |
| `policy_updated_by` | String | `actor.user.name` | `actor_user_name` | — |
| `policy_notes` | String | `comment` | `comment` | — |

**Tier-2 columns** (no `ocsf_field`; values aggregate into `raw_extensions` JSON object):

| Column (TOML name) | ColumnType | Notes |
|--------------------|-----------|-------|
| `policy_source` | String | "Custom" or system source tag |
| `policy_acl_type` | String | ACL syntax type returned by API (e.g., "Cisco dACL") |
| `policy_acl` | String | Raw multi-line ACL text in the requested syntax format |
| `applied_models` | **Json** | Array of device model strings to which this ACL applies |
| `matching_devices` | Integer | Count of devices currently matching this ACL policy |
| `policy_creation_date` | Datetime | ISO 8601; ADR-028 §D8-B implicit iso8601 default |
| `policy_last_updated` | Datetime | ISO 8601; ADR-028 §D8-B implicit iso8601 default |

**Total declared columns:** 11 (4 Tier-1, 7 Tier-2).
All 11 fields are from the OrganizationAclPolicy fields_enum confirmed in schema extract
§OrganizationAclPolicy (field count: 11) and the OrganizationAclPolicyResponseItem concrete
schema (11 anyOf/nullable fields, all anyOf [type, null]).
**Json columns:** 1 (`applied_models` — array of device model strings per spike findings
§Spike 4 body_template note).

### 3. Primary Key and OCSF Mapping Rationale

**Primary key: `policy_id` (String, REQUIRED, single-column)**

`policy_id` is declared as `anyOf: [uuid, null]` in the `OrganizationAclPolicyResponseItem`
concrete schema — a UUID-format identifier that serves as the stable, system-assigned identity
for each ACL policy record. It maps to the OCSF `entity_management` `metadata.uid` field
(Arrow: `metadata_uid`), which is the canonical OCSF anchor for a unique record identifier
within the metadata object.

**Decision rationale (policy_id over policy_name):**

1. `policy_id` is a UUID-format system identifier — immutable, non-human-editable, and
   guaranteed unique per Claroty's internal assignment. Human-readable policy names can be
   renamed by administrators.
2. OCSF `metadata.uid` is the correct semantic anchor for a stable record identifier, while
   `name` carries the human-readable policy label.
3. The OrganizationAclPolicyResponseItem schema declares `policy_id` as `anyOf: [uuid, null]` —
   UUID-format fields in Claroty's schema are system identifiers.
4. The REQUIRED option on `policy_id` (with null-row semantics for absent values) follows the
   same pattern as zone/firewall primary keys in BC-2.16.020/021.

**OCSF Tier-1 mapping rationale:**

- **`policy_id` → `metadata.uid` (Arrow: `metadata_uid`, REQUIRED):** The stable UUID primary
  key of each ACL policy record maps to OCSF `metadata.uid` — the standard OCSF field for
  a unique record identifier in the metadata object. A UUID PK is semantically distinct from
  `name` (which carries the human-readable label) and `comment` (analyst notes).

- **`policy_name` → `name` (Arrow: `name`):** OCSF `entity_management` carries `name` for
  the human-readable identifier of the managed entity. The policy name is the display label
  for the ACL policy.

- **`policy_updated_by` → `actor.user.name` (Arrow: `actor_user_name`):** OCSF
  `entity_management` has an `actor` object capturing who performed the management action.
  `policy_updated_by` (email or username of the last analyst to modify the ACL policy) maps
  to `actor.user.name`.

- **`policy_notes` → `comment` (Arrow: `comment`):** OCSF `entity_management` carries
  `comment` for free-text analyst notes and descriptive context. Policy notes map directly.

**No new `class_selector` arm required:** The existing `entity_management` arm (class_uid 3004)
in `prism-ocsf/src/class_selector.rs::select_by_class_name` is reused. Same arm as
BC-2.16.020, BC-2.16.021, and `claroty_audit_logs`. Zero new class_selector arms per
xdome-endpoint-expansion-plan.md §Governing Directive and spike findings §Overall Verdict.

### 4. Pagination-None Contract vs Offset-Limit Pattern (ADR Differentiation)

This table is **the only Claroty table** (and the first prism sensor table of any kind encountered
in Wave C) to use `PaginationConfig::None`. All other Claroty tables use:

```toml
[tables.steps.pagination]
type = "offset_limit"
page_size = 1000
```

The ACL policies endpoint declares:
```toml
[tables.steps.pagination]
type = "none"
```

**Behavioral difference at `pipeline.rs::build_request`:**
- `offset_limit`: injects `"offset": N, "limit": 1000` into the POST body and loops until the
  response returns an empty page or a page count less than the page size
- `none`: constructs the POST body from `body_template` only (no offset/limit injection); issues
  a single HTTP POST; returns the entire response as one result set; no loop

**body_template responsibility shift:** Because there is no pagination loop, the `body_template`
field carries ALL body parameters — including the mandatory `policy_acl_syntax`. For
`offset_limit` tables, `body_template` only carries `fields` (pagination parameters are
injected by the pipeline). For `none` tables, `body_template` is the complete POST body.

**Response count field:** Standard tables carry `{"<key>": [...], "count": N}`. This table
returns `{"organization_acl_policies": [...]}` with NO `count` field. The pipeline's
empty-page-check is not invoked for `PaginationConfig::None` — the response IS the complete
result.

### 5. Json Column Serialization Behavior

The `applied_models` Json column is a nested array of device model strings in the Claroty API
response. The spec-engine's pipeline serializes it into `raw_extensions` as a JSON-typed value
when `ocsf_column_naming = true` and `column_type = "json"` is declared. This is existing
behavior — no new mechanism is required.

An empty array `[]` for `applied_models` is serialized as `[]`, not null
(EC-016-022-005).

Declaring `applied_models` as `String` instead of `Json` would cause the array to be serialized
as a raw string token (e.g., `"[\"model1\", \"model2\"]"`) — a P1 TOML authoring defect
following the same invariant as BC-2.16.020 §Invariants.

### 6. SAP-2 DTU Parity Status

SAP-2 probe is **N/A** (no DTU exists for this endpoint per
xdome-endpoint-expansion-plan.md §Governing Directive and §Deferred DTU-Creation Stories).
The deferred DTU creation story is tracked as D-2200. Once the DTU story for
`claroty_organization_acl_policies` executes, SAP-2 probe applies retroactively and this BC
MUST be amended with:
- DTU route file reference (`crates/prism-dtu-claroty/src/routes/organization_acl_policies.rs`)
- DTU types.rs field equivalencies for all 11 contracted columns
- SAP-2 exclusion documentation for any deliberately excluded fields
- Verification that `policy_acl_syntax = "Cisco dACL"` is correctly reflected in the DTU
  static fixture path

Until the DTU story executes, near-term tests run against the live monroe sensor only.

## Invariants

- DI-005: OCSF schema validity — `entity_management` class_uid 3004 is a valid OCSF class
- `policy_id` (String, REQUIRED) is the primary key for `claroty_organization_acl_policies`;
  absent `policy_id` produces a null row per spec-engine REQUIRED semantics (not a hard error;
  subsequent rows continue)
- `PaginationConfig::None` MUST NOT result in `offset`/`limit` field injection in the POST
  body; any implementation that injects these fields when `type = "none"` is declared produces
  an API validation error (the `GetOrganizationAclPoliciesRequest` schema does not accept
  `offset`/`limit`)
- The `policy_acl_syntax` field MUST appear in the `body_template` and MUST be hardcoded to
  `"Cisco dACL"` for v1; omitting `policy_acl_syntax` from the POST body causes the Claroty
  API to return an error (it is in `required: [...]` in the OpenAPI schema)
- Tier-2 columns (all columns without `ocsf_field`) are NOT exposed as standalone Arrow columns;
  a PrismQL query referencing them by raw TOML name MUST raise E-QUERY-038 with
  `available_columns` containing `raw_extensions`, `metadata_uid`, `name`, `actor_user_name`,
  `comment`, `class_uid`, `_sensor` — but NOT the raw Tier-2 column name
- `applied_models` MUST be declared with `column_type = "json"` in the TOML spec; declaring it
  as `String` would cause the array to be serialized as a raw string token — a P1 TOML
  authoring defect (spike findings §Spike 4 body_template note, spike findings §Spike 3
  §Nested-field classification principle)
- The response envelope key is `organization_acl_policies` (not `organization_acl_policy`);
  the `response_path` MUST use `$.organization_acl_policies`
- Datetime fields (`policy_creation_date`, `policy_last_updated`) use ADR-028 §D8-B implicit
  iso8601 default (`timestamp_formats` omitted → `effective_formats` returns `["iso8601"]`);
  null passthrough when field is absent

## Error Cases

| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-SENSOR-001` | Claroty API returns non-200 HTTP for POST /api/v1/organization_acl_policies/ | Structured error with sensor=claroty, status, body; no partial results (single-fetch endpoint, no prior pages) |
| `E-SENSOR-001` | `policy_acl_syntax` field missing from POST body (body_template misconfigured) | API returns 422 or 400 validation error; `E-SENSOR-001` raised with API error body |
| `E-QUERY-038` | Query references `policy_source`, `policy_acl_type`, `policy_acl`, `applied_models`, `matching_devices`, `policy_creation_date`, `policy_last_updated`, or any other Tier-2 column by its raw TOML name | Column-not-found at plan time; `available_columns` lists `metadata_uid`, `name`, `actor_user_name`, `comment`, `raw_extensions`, `class_uid`, `_sensor` |
| `E-SPEC-018` | Datetime parse failure for `policy_creation_date` or `policy_last_updated` for a non-null non-ISO-8601 value | `E-SPEC-018 TimestampParseFailure` — null demoted with warning; row continues |

**No new error codes are required.** The `PaginationConfig::None` path does not introduce a
new distinct failure mode relative to `offset_limit`. The `E-SENSOR-001` code covers HTTP
errors from this endpoint (including the API's 422 response when `policy_acl_syntax` is
missing). The absence of a `count` field in the response is expected and is not an error
condition — the pipeline does not attempt to read `count` for `PaginationConfig::None`.

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-016-022-001 | Row in `claroty_organization_acl_policies` missing `policy_id` field (REQUIRED) | Null row produced; no hard error; subsequent rows continue |
| EC-016-022-002 | `body_template` omits `policy_acl_syntax` (misconfiguration) | API returns validation error; `E-SENSOR-001` raised; entire query fails (no partial results — single-fetch) |
| EC-016-022-003 | Query references Tier-2 column `policy_source` by raw TOML name | E-QUERY-038; `available_columns` contains `metadata_uid`, `name`, `actor_user_name`, `comment`, `raw_extensions` but NOT `policy_source` |
| EC-016-022-004 | Query references Tier-2 column `policy_id` by raw TOML name (TOML column name, NOT the Arrow field name) | E-QUERY-038; `available_columns` contains `metadata_uid` (the Arrow field name for policy_id→metadata.uid) but NOT `policy_id` as a standalone column |
| EC-016-022-005 | `applied_models` is an empty array `[]` | Serialized as `[]` JSON in `raw_extensions`; not null; no error |
| EC-016-022-006 | `applied_models` declared as `column_type = "string"` instead of `"json"` | Array serialized as raw string token (e.g., `"[\"model1\"]"`) in `raw_extensions.applied_models` — P1 TOML authoring defect; caught by structural test asserting JSON-typed value |
| EC-016-022-007 | `pagination.type` mistakenly set to `"offset_limit"` instead of `"none"` | Pipeline injects `offset`/`limit` into POST body; API returns 422 validation error; `E-SENSOR-001` raised; entire query fails |
| EC-016-022-008 | `pagination` section omitted entirely from the step | `PaginationConfig` defaults to `PaginationType::None` via `.unwrap_or(PaginationType::None)` in types.rs — behaviorally equivalent to `type = "none"` declared explicitly; NOT recommended (explicit declaration is required per §PC4 invariant for clarity) |
| EC-016-022-009 | `policy_acl_syntax = "ArubaOS-Switch"` used instead of `"Cisco dACL"` | API returns ACL text in ArubaOS-Switch format in the `policy_acl` field; `policy_acl_type` confirms the syntax; valid API call but outside v1 contract scope (deferred configurability) |
| EC-016-022-010 | `policy_updated_by` absent in a row | Null `actor_user_name` Arrow cell; not an error |

## Related BCs

- BC-2.16.013: Bundled Sensor Spec Authoring — parent spec for the Claroty sensor; this BC adds
  the `claroty_organization_acl_policies` table to the Claroty sensor surface (depends on)
- BC-2.16.020: Claroty xDome Organization Zone Domain — structural sibling in the organization
  policy group; both use entity_management/3004 and the same Tier-1 pattern; Zone Domain uses
  offset_limit pagination while this BC uses none (sibling, contrasted on pagination)
- BC-2.16.021: Claroty xDome Organization Firewall Domain — structural sibling; same OCSF class
  and Tier pattern; Firewall Domain uses offset_limit pagination (sibling, contrasted on
  pagination)
- BC-2.02.005: Claroty xDome Field Mapping to OCSF (9 Data Sources) — OCSF class mapping for
  all Claroty sources; `entity_management` class_uid 3004 covered (composes with)
- BC-2.01.007: Claroty Bearer Token Auth — auth mechanism unchanged; preconditions satisfied
  (depends on)

## Architecture Anchors

- `crates/prism-sensors/specs/claroty.sensor.toml` — TOML spec file authoring target
- `crates/prism-spec-engine/src/spec_parser.rs` — `PaginationConfig::None` variant;
  `ColumnSpec` (column_type Json); `FetchStep` deserialization
- `crates/prism-spec-engine/src/pipeline.rs` — `build_request`: `PaginationConfig::None` path
  (no offset/limit injection); `response_path` extraction; Json column serialization into
  `raw_extensions`
- `crates/prism-ocsf/src/class_selector.rs::select_by_class_name` — `"entity_management"` arm
  (existing; resolves to class_uid 3004)
- `crates/prism-spec-engine/src/spec_driven_adapter.rs` — `pipeline_result_to_record_batch`
- `.factory/reference/api-specs/xdome_openapi_06.20.2026.json §/api/v1/organization_acl_policies/` — endpoint authority; `GetOrganizationAclPoliciesRequest` schema (required: ["policy_acl_syntax"]); `OrganizationAclPolicyResponseItem` concrete schema (11 fields, all anyOf/nullable)
- `.factory/objectives/xdome-endpoint-expansion-plan.md §Gap Table G6` — table scope authority
- `.factory/objectives/xdome-v1-validation/endpoint-spike-findings.md §Spike 4` — AUTHORITATIVE: pagination=none decision, policy_acl_syntax mandatory handling, 11-column set with Tier classifications, PK decision (policy_id→metadata.uid), body_template

## Story Anchor

S-CLAROTY-ACLPOLICY-001 (draft — Wave C)

## VP Anchors

(none — no formal verification properties defined; structural tests via story RG list per
S-CLAROTY-ACLPOLICY-001; holdout evaluator exercises live monroe surface via HS-029)

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.16.022-001 | `SELECT * FROM claroty.claroty_organization_acl_policies LIMIT 1` against live monroe | Succeeds (no error); wire JSON has `class_uid = 3004`; `metadata_uid` column present non-null (UUID string); `name` present; `raw_extensions` JSON object present with at least one ACL Tier-2 key; `policy_id` NOT a standalone Arrow column |
| TV-BC-2.16.022-002 | `SELECT metadata_uid FROM claroty.claroty_organization_acl_policies LIMIT 5` | Succeeds (no E-QUERY-038); rows have non-null `metadata_uid` UUID strings |
| TV-BC-2.16.022-003 | `SELECT policy_id FROM claroty.claroty_organization_acl_policies LIMIT 1` | E-QUERY-038; `available_columns` contains `metadata_uid`, `name`, `actor_user_name`, `comment`, `raw_extensions`; does NOT contain `policy_id` as standalone column |
| TV-BC-2.16.022-004 | `SELECT policy_source FROM claroty.claroty_organization_acl_policies LIMIT 1` | E-QUERY-038; `available_columns` contains `raw_extensions` but NOT `policy_source` |
| TV-BC-2.16.022-005 | `SELECT raw_extensions FROM claroty.claroty_organization_acl_policies LIMIT 5` | Succeeds; `raw_extensions` JSON contains at least `policy_source`, `applied_models` keys; `applied_models` value is a JSON array (not a quoted string) |
| TV-BC-2.16.022-006 | `SELECT * FROM claroty.claroty_organization_acl_policies` (no LIMIT) against live monroe | Succeeds; returns all ACL policies in a single query without pagination; no `count` column in wire output; result set may be larger than 1000 rows if more than 1000 ACL policies exist |
| TV-BC-2.16.022-007 | Wire output of `SELECT *` with `pagination.type = "offset_limit"` (misconfiguration test) | API returns 422 or 400; `E-SENSOR-001` raised (offset/limit injected into request body — API rejects) |

## Verification Properties

| VP | Verification Aspect |
|----|---------------------|
| (none) | No VP directly verifies this BC — structural tests cover via story RG list per S-CLAROTY-ACLPOLICY-001; holdout evaluator exercises live monroe surface via HS-029 |

## Traceability

| Field | Value |
|-------|-------|
| L2 Capability | CAP-029 |
| Capability Anchor Justification | CAP-029 ("Config-Driven Sensor Adapters") per capabilities.md §CAP-029 — this BC specifies the TOML table contract for the `claroty_organization_acl_policies` table, defining 11 columns (typed with ColumnOptions and OCSF mappings), a non-paginated single-page fetch pipeline (`type = "none"` pagination, POST /api/v1/organization_acl_policies/ with mandatory `policy_acl_syntax = "Cisco dACL"` hardcoded in body_template, envelope key `$.organization_acl_policies`, NO count field), Tier-1/Tier-2 OCSF column classification per ADR-058 (4 Tier-1: policy_id→metadata.uid REQUIRED/metadata_uid, policy_name→name, policy_updated_by→actor_user_name, policy_notes→comment; 7 Tier-2 into raw_extensions including 1 Json: applied_models), PK decision rationale (policy_id UUID→metadata.uid vs policy_name: UUID PK is stable; metadata.uid is the correct OCSF anchor), and SAP-2 N/A documentation (no DTU; D-2200 deferred DTU anchor). This is exactly what CAP-029 defines: sensor adapters defined in TOML spec files with tables, columns, pipelines, and pagination config. |
| L2 Invariants | DI-005 |
| Priority | P0 |
| Story | S-CLAROTY-ACLPOLICY-001 |
| DTU Status | NONE — no DTU exists for this endpoint; near-term tests against live monroe sensor only; DTU deferred to D-2200 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.0 | xdome-wave-c-f2-spec-evolution-g6 | 2026-08-24 | product-owner | Initial authoring — Claroty xDome Organization ACL Policies queryable surface contract per xdome-endpoint-expansion-plan.md Wave C G6. KEY NOVELTY: `type = "none"` pagination (only Claroty table with non-paginated fetch; all others use offset_limit/1000); mandatory `policy_acl_syntax = "Cisco dACL"` hardcoded in body_template (REQUIRED API field, not in fields_enum, hardcoded v1, follow-up story deferred); response envelope has NO count field. 11 columns from OrganizationAclPolicyResponseItem (all anyOf/nullable): 4 Tier-1 (policy_id→metadata.uid REQUIRED/metadata_uid, policy_name→name, policy_updated_by→actor_user_name, policy_notes→comment) + 7 Tier-2 (policy_source/String, policy_acl_type/String, policy_acl/String, applied_models/Json, matching_devices/Integer, policy_creation_date/Datetime, policy_last_updated/Datetime). PK: policy_id (UUID-format, stable system ID) → metadata.uid; policy_name→name (human-readable Tier-1, not PK). OCSF: entity_management/3004 (existing arm; same as BC-2.16.020/021). No new error codes (PaginationConfig::None no-count path covered by existing E-SENSOR-001/E-QUERY-038/E-SPEC-018). No new class_selector arm. SAP-2 N/A (no DTU; D-2200 deferred DTU anchor). HS-029 holdout group registered with 3 P0 scenarios for S-CLAROTY-ACLPOLICY-001. |
