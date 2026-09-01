---
document_type: story
story_id: S-CLAROTY-ACLPOLICY-001
title: "Claroty xDome ACL Policies Table — claroty_organization_acl_policies TOML block with 11-column Tier-1/Tier-2 spec, pagination type=none (non-paginated single-fetch), mandatory policy_acl_syntax=Cisco dACL body field, live structural tests (Wave C G6)"
level: "L4"
wave: xdome-wave-c
epic_id: E-XDOME-EXPANSION
priority: P0
status: ready
# BC status: BC-2.16.022 v1.1 draft — pre-delivery remove-uncertainty pass complete 2026-08-31; promoted to ready (D-2385).
producer: story-writer
timestamp: "2026-08-24T00:00:00Z"
version: "1.5"
modified: "2026-09-01"
phase: 3
cycle: v1.0.0-brownfield
inputs:
  - ".factory/specs/behavioral-contracts/BC-2.16.022-claroty-org-acl-policies.md"
  - ".factory/objectives/xdome-endpoint-expansion-plan.md"
  - ".factory/objectives/xdome-v1-validation/endpoint-schema-extract.md"
  - ".factory/objectives/xdome-v1-validation/endpoint-spike-findings.md"
  - ".factory/specs/architecture/decisions/ADR-058-v1-column-naming-col-name-as-arrow-field-identifier.md"
  - "crates/prism-sensors/specs/claroty.sensor.toml"
input-hash: "11f1c65"
# input-hash: run `compute-input-hash <this-file> --update` after state-manager commits
traces_to: "BC-2.16.022"
points: 5
estimated_days: 1
tdd_mode: strict
subsystems: [SS-01, SS-16]
# Subsystem anchor justifications (ARCH-INDEX Subsystem Registry):
#   SS-01 (Sensor Adapters) owns this story's scope because
#     `crates/prism-sensors/specs/claroty.sensor.toml` — the TOML spec file being
#     modified — lives in the prism-sensors crate, which is listed under SS-01 per
#     ARCH-INDEX. The one new `[[tables]]` block (claroty_organization_acl_policies)
#     is a sensor-adapter configuration artifact, exactly the surface SS-01 governs.
#   SS-16 (Spec Engine) owns this story's scope because
#     `crates/prism-spec-engine/src/spec_parser.rs` must parse the new [[tables]]
#     block without validation error, and `crates/prism-spec-engine/src/pipeline.rs`
#     must route correctly on PaginationConfig::None (no offset/limit injection).
#     RG-001..RG-011 include spec-parser and pipeline unit tests that exercise SS-16
#     surfaces. SS-16 is the canonical owner of prism-spec-engine per ARCH-INDEX.
target_module: prism-sensors
crates_touched: [prism-sensors, prism-spec-engine, prism-bin, prism-query]
# crates_touched:
#   prism-sensors: claroty.sensor.toml — one new [[tables]] block
#   prism-spec-engine: RG unit tests (spec-parser, pipeline PaginationConfig::None path,
#                      column inspection); no production code changes expected
#   prism-bin: authoritative RG-005/RG-006 end-to-end E-QUERY-038 gates + RG-012 wire-shape
#   prism-query: plan-time E-QUERY-038 integration test path (via QueryEngine::execute in prism-bin)
capabilities:
  - CAP-029
behavioral_contracts:
  - BC-2.16.022
  # BC-2.16.022 v1.0 — Claroty xDome Organization ACL Policies: TOML table contract
  # for claroty_organization_acl_policies (§PC1: POST /api/v1/organization_acl_policies/,
  # response_path $.organization_acl_policies, pagination type=none, mandatory
  # policy_acl_syntax="Cisco dACL" body field, 11 cols); Tier-1/Tier-2 column
  # classifications (§PC2: 4 Tier-1 policy_id→metadata_uid REQUIRED, policy_name→name,
  # policy_updated_by→actor_user_name, policy_notes→comment; 7 Tier-2 into raw_extensions
  # incl. 1 Json applied_models); PK rationale §PC3; pagination-none vs offset_limit
  # differentiation §PC4; Json column serialization §PC5; SAP-2 N/A §PC6;
  # EC-016-022-001..010. All 11 ACs trace to this BC.
verification_properties: []
holdout_scenarios:
  - HS-029
# holdout_scenarios: HS-029 registered by PO at BC-2.16.022 §Changelog
# (3 P0 hidden scenarios covering claroty_organization_acl_policies live surface).
# Scenarios live under the holdout-scenarios directory that test-writer and implementer
# MUST NOT read (contamination control). The story-level holdout gate (human-approved
# 2026-07-13) is BLOCKING before demo recording / push to origin.
depends_on: []
# depends_on justification: No delivery-time scheduling dependency. The
# claroty_organization_acl_policies table is an independent POST-for-read query;
# it does not join to other xDome expansion tables in this first-cut spec.
# S-ADR058-OCSF-ROUTING-001 (which activated ocsf_column_naming=true) is already
# MERGED (PR #242, develop@3f1e66179). The entity_management arm (class_uid 3004)
# is already present in class_selector.rs (spike-findings §Overall Verdict).
# PaginationConfig::None is confirmed present in spec_parser.rs
# (spike-findings §Spike 4: "PaginationConfig::None in spec_parser.rs").
# No Wave A/B/C expansion blocks exist in the committed TOML at develop@3f1e66179.
blocks: []
acceptance_criteria_count: 11
risk: MEDIUM
# Risk justification:
#   No DTU exists; live Variant-1 tests are #[ignore]'d until live validation against
#   monroe. The pagination type=none contract is a structural novelty — the only
#   Claroty table without offset/limit injection. The mandatory policy_acl_syntax
#   field must appear in the body_template; omitting it causes an API 422.
#   SAP-2 DTU-parity probe is N/A per D-2200.
assumption_validations:
  - assumption: "Endpoint POST /api/v1/organization_acl_policies/; envelope key organization_acl_policies with NO count field; response_path $.organization_acl_policies"
    verdict: CONFIRMED
    source: "xdome_openapi_06.20.2026.json §GetOrganizationAclPoliciesResponse (sole required prop, no count); §paths /api/v1/organization_acl_policies/ (operationId ..._post); endpoint-schema-extract.md §organization_acl_policies"
  - assumption: "policy_acl_syntax is a REQUIRED request-level parameter (not a query field), valid ACLType values Cisco dACL/AireOS/ArubaOS-Switch/ArubaOS-CX; hardcoded Cisco dACL for v1"
    verdict: CONFIRMED
    source: "xdome_openapi_06.20.2026.json §GetOrganizationAclPoliciesRequest.required=[policy_acl_syntax] + §ACLType; policy_acl_syntax absent from OrganizationAclPolicy.fields_enum"
  - assumption: "11 body_template fields match the OrganizationAclPolicy fields_enum exactly (incl. order); all 11 response columns are anyOf/nullable"
    verdict: CONFIRMED
    source: "xdome_openapi_06.20.2026.json §OrganizationAclPolicy__fields_enum (11 entries) + §OrganizationAclPolicyResponseItem (11 anyOf/nullable props)"
  - assumption: "Column types: policy_id string/uuid→String REQUIRED, policy_name/policy_source/policy_acl/policy_acl_type/policy_updated_by/policy_notes→String, applied_models array-of-string→Json, matching_devices integer→Integer, policy_creation_date/policy_last_updated date-time→Datetime"
    verdict: CONFIRMED
    source: "xdome_openapi_06.20.2026.json §OrganizationAclPolicyResponseItem (per-field anyOf types; policy_id format:uuid; applied_models items:string array; dates format:date-time)"
  - assumption: "pagination type=none correct — request schema declares no offset/limit and additionalProperties=false; PaginationConfig::None present with serde tag=type snake_case; build_request skips offset/limit body injection when page_size=0"
    verdict: CONFIRMED
    source: "xdome_openapi_06.20.2026.json §GetOrganizationAclPoliciesRequest (additionalProperties:false, no offset/limit); prism-spec-engine/src/spec_parser.rs §PaginationConfig::None; prism-spec-engine/src/pipeline.rs §build_request (OffsetLimit+page_size>0 gate)"
  - assumption: "ocsf_field_to_arrow_name flattens dots to underscores (metadata.uid→metadata_uid, actor.user.name→actor_user_name); entity_management→class_uid 3004 arm exists and carries comment attr"
    verdict: CONFIRMED
    source: "prism-spec-engine/src/column_mapping.rs §ocsf_field_to_arrow_name; prism-ocsf/src/class_selector.rs §CLASS_UID_ENTITY_MANAGEMENT=3004 + entity_management arm (KF-01 comment attr)"
  - assumption: "Baseline is exactly 4 Claroty tables (alerts, audit_logs, devices, device_alert_relations); ocsf_column_naming=true at sensor level; SAP-2 N/A (no ACL DTU route)"
    verdict: CONFIRMED
    source: "crates/prism-sensors/specs/claroty.sensor.toml (4 table_name decls; ocsf_column_naming=true); crates/prism-dtu-claroty/src/routes/ has no organization_acl_policies route (D-2200)"
  - assumption: "Datetime fields use ADR-028 §D8-B implicit iso8601 default (timestamp_formats omitted → effective_formats returns [iso8601]) — SAP-2 datetime arm (c)"
    verdict: CONFIRMED
    source: "xdome_openapi_06.20.2026.json §OrganizationAclPolicyResponseItem policy_creation_date/policy_last_updated format:date-time (ISO-8601 examples); ADR-028 §D8-B"
risk_mitigations: []
---

# S-CLAROTY-ACLPOLICY-001: Claroty xDome ACL Policies Table — Non-Paginated Single-Fetch + Mandatory policy_acl_syntax

## Authority

**BC-2.16.022 §Postconditions §1 — TOML Table Contract** governs the exact
`[[tables]]` block: `table_name = "organization_acl_policies"` (bare name;
`{sensor_id}_{table_name}` = `claroty_organization_acl_policies` registered/queryable name),
`ocsf_class = "entity_management"`, step name `"fetch_organization_acl_policies"`,
`method = "POST"`, `path_template = "/api/v1/organization_acl_policies/"`,
`body_template` containing `"policy_acl_syntax": "Cisco dACL"` and all 11 contracted
fields (REQUIRED per OpenAPI schema — `required: ["policy_acl_syntax"]`),
`response_path = "$.organization_acl_policies"`, and `pagination.type = "none"`.

**BC-2.16.022 §Postconditions §2 — Column Tier Classification (ADR-058)**
governs the exact 4 Tier-1 and 7 Tier-2 column declarations under
`ocsf_column_naming = true`:

- Tier-1: `policy_id` → `metadata.uid` → Arrow `metadata_uid` (REQUIRED)
- Tier-1: `policy_name` → `name` → Arrow `name`
- Tier-1: `policy_updated_by` → `actor.user.name` → Arrow `actor_user_name`
- Tier-1: `policy_notes` → `comment` → Arrow `comment`
- Tier-2 (into `raw_extensions`): `policy_source` (String), `policy_acl_type` (String),
  `policy_acl` (String), `applied_models` (**Json** — array of device model strings),
  `matching_devices` (Integer), `policy_creation_date` (Datetime),
  `policy_last_updated` (Datetime)

**BC-2.16.022 §Postconditions §3 — Primary Key Rationale** establishes `policy_id`
(UUID-format, system-assigned, immutable) → `metadata.uid` as PK. `policy_name` is
human-editable and maps to `name` (Tier-1 display label, not the PK). The OCSF anchor
`metadata.uid` is the correct field for a stable record identifier.

**BC-2.16.022 §Postconditions §4 — Pagination-None Contract** establishes that
`PaginationConfig::None` means `pipeline.rs::build_request` MUST NOT inject `offset`/`limit`
into the POST body. The `GetOrganizationAclPoliciesRequest` schema has no `offset`/`limit`
fields — injecting them causes an API 422. The body is constructed from `body_template`
only. The response returns all ACL policies in a single HTTP response; no loop.

**BC-2.16.022 §Invariants** — critical mandatory invariants:
- `PaginationConfig::None` MUST NOT inject `offset`/`limit` (API 422 if violated)
- `policy_acl_syntax = "Cisco dACL"` MUST appear in `body_template` (API error if absent)
- Tier-2 columns NOT exposed as standalone Arrow columns (E-QUERY-038 at plan time)
- `applied_models` MUST be declared `column_type = "json"` (not `"string"`)
- `response_path = "$.organization_acl_policies"` (plural, no count field in envelope)

**ADR-058 §B2** — Tier-2 columns MUST aggregate into `raw_extensions` under
`ocsf_column_naming = true`. The `entity_management` OCSF class maps to class_uid 3004 —
the existing arm in `class_selector.rs::select_by_class_name` used without modification.

**ADR-058 §C** — `ocsf_field_to_arrow_name("actor.user.name")` = `"actor_user_name"` (dot
→ underscore flattening); `ocsf_field_to_arrow_name("metadata.uid")` = `"metadata_uid"`.

**spike-findings §Spike 4** — AUTHORITATIVE: pagination=none decision, mandatory
`policy_acl_syntax` handling, 11-column set with Tier classifications, PK decision
(policy_id→metadata.uid), body_template with all 11 fields listed. Also confirms:
`PaginationConfig::None` is present in `spec_parser.rs` (serde tag = "type", snake_case).

**S-ADR058-OCSF-ROUTING-001** (merged PR #242, develop@3f1e66179) activated
`ocsf_column_naming = true` at the sensor level in `claroty.sensor.toml`. The new table
inherits this setting automatically — no per-table flag needed.

---

## Narrative

As a SOC analyst querying Claroty xDome network ACL policy data via PrismQL,
I want a `claroty_organization_acl_policies` table with OCSF `entity_management` class
(class_uid 3004),
so that I can query xDome ACL policy records — with OCSF Tier-1 fields (`metadata_uid`,
`name`, `actor_user_name`, `comment`) for identity, actor, and analyst context, and
Tier-2 details (including `applied_models` as a Json array) available via `raw_extensions`
— enabling ACL governance queries without requiring pagination coordination, since the
Claroty API delivers all ACL policies in a single non-paginated response.

## Background

As of develop@3f1e66179 the committed `crates/prism-sensors/specs/claroty.sensor.toml`
contains exactly 4 tables — `alerts`, `audit_logs`, `devices`, `device_alert_relations`
(verified by direct inspection; exactly 4 `table_name =` declarations).

The Wave A/B/C sibling expansion stories (S-CLAROTY-VULNS-001, S-CLAROTY-OT-EVENTS-001,
S-CLAROTY-DEVVULNREL-001, S-CLAROTY-SERVERS-001, S-CLAROTY-ORGPOLICY-001) are
materialized draft (pending) — NONE merged, NONE implemented; their TOML blocks do NOT
exist in the committed TOML at this story's authoring time. The implementer MUST re-verify
the actual baseline table count at implementation time and treat the post-story total as
**baseline + 1** (5 if the baseline is still the 4-table develop@3f1e66179 set; more if
sibling expansion stories merge first per depends_on ordering).

This story delivers the Wave C G6 addition (one TOML block):

**`claroty_organization_acl_policies`** — 11 columns (4 Tier-1 + 7 Tier-2 incl. 1 Json:
`applied_models`). PK: `policy_id` REQUIRED → Arrow `metadata_uid`. response_path
`$.organization_acl_policies`. **KEY NOVELTY: `pagination.type = "none"` — the only
Claroty table (and the first prism sensor table of any kind encountered in Wave C) using
non-paginated single-fetch.** All other Claroty tables use `type = "offset_limit"` with
`page_size = 1000`.

**Mandatory body parameter:** `policy_acl_syntax = "Cisco dACL"` is a REQUIRED field in
the `GetOrganizationAclPoliciesRequest` OpenAPI schema (`required: ["policy_acl_syntax"]`).
It is NOT in the `fields_enum` — it controls the text format of the `policy_acl` response
field. For v1 it is hardcoded to `"Cisco dACL"` in `body_template`. Configurable
`policy_acl_syntax` per-table is deferred to a follow-up story (no story ID assigned at
authoring time; requires new TOML schema extensions beyond v1 scope — not a tech-debt-register
entry, but a feature extension). MSSP tenants needing ArubaOS or AireOS ACL format must
wait for the follow-up story.

**Live-test approach (per xdome-endpoint-expansion-plan.md §Per-Story Pipeline):**

- **Variant-1 (structural, required):** Live `#[ignore]`'d integration tests against the
  monroe sensor. Wire-level assertions on serialized JSON response (class_uid, field
  presence, `raw_extensions` keys, Json column types). Tests marked `#[ignore]` with
  comment: `// LIVE-MONROE-001: requires CLAROTY_INSTANCE_URL env var pointing to monroe;
  run manually or in live-validation CI job`.
- **Variant-2 (agent, optional):** PrismQL agent-level test. Deferred to live-validation
  milestone if not complete before holdout gate.
- **DTU note:** SAP-2 DTU-parity probe is **N/A** for this table per BC-2.16.022 §PC6
  and D-2200 governing decision (no DTU exists for `claroty_organization_acl_policies`;
  DTU creation is a separate deferred story tracked as D-2200). Do NOT run SAP-2 checks
  against `crates/prism-dtu-claroty/src/` — no ACL policy route exists there and its
  absence is expected.

**Story-level holdout gate:** After LOCAL 3-CLEAN adversary convergence and BEFORE demo
recording / push to origin, the holdout-evaluator runs HS-029 (hidden SINGLE-USE scenarios
authored by PO at remove-uncertainty time; stored under the holdout directory;
contamination-controlled — test-writer and implementer MUST NOT read the HS-029 scenario
files). The gate is BLOCKING: unsatisfied scenarios reset the LOCAL streak per BC-5.39.001.

## Behavioral Contracts

| BC | Title | Version | Role |
|----|-------|---------|------|
| BC-2.16.022 | Claroty xDome Organization ACL Policies — Non-Paginated Single-Page Fetch with Mandatory policy_acl_syntax and OCSF entity_management Mapping (No DTU) | v1.2 | §Postconditions §1 TOML contract (POST /api/v1/organization_acl_policies/, response_path $.organization_acl_policies, pagination type=none, mandatory policy_acl_syntax="Cisco dACL", 11 cols, NO count field in envelope); §Postconditions §2 Tier-1/Tier-2 column classification (4 Tier-1: policy_id→metadata.uid REQUIRED/metadata_uid, policy_name→name, policy_updated_by→actor_user_name, policy_notes→comment; 7 Tier-2 incl. 1 Json: applied_models); §Postconditions §3 PK rationale (policy_id UUID-format stable→metadata.uid; policy_name human-editable→name not PK); §Postconditions §4 Pagination-None vs Offset-Limit differentiation; §Postconditions §5 Json column serialization behavior (applied_models as JSON array not string; empty [] not null); §Postconditions §6 SAP-2 N/A; EC-016-022-001..010. All 11 ACs trace to this BC. |

## Acceptance Criteria

### AC-001: TOML block parses without validation error; 11 columns declared; pagination type=none declared; response_path=$.organization_acl_policies; policy_acl_syntax in body_template (traces to BC-2.16.022 postcondition 1 — TOML Table Contract)

`crates/prism-sensors/specs/claroty.sensor.toml` declares a `[[tables]]` block with
`table_name = "organization_acl_policies"` (bare name; `{sensor_id}_{table_name}` =
`claroty_organization_acl_policies` registered/queryable name), `ocsf_class = "entity_management"`,
a step named `"fetch_organization_acl_policies"` with `method = "POST"`,
`path_template = "/api/v1/organization_acl_policies/"`,
`response_path = "$.organization_acl_policies"`, `pagination.type = "none"` (NOT
`"offset_limit"`), and `body_template` containing `"policy_acl_syntax": "Cisco dACL"` and
all 11 contracted fields.

`SpecLoader::parse` on the modified TOML returns `Ok(SensorSpec)` without validation error.
The parsed spec reports 11 `ColumnSpec` entries for the `claroty_organization_acl_policies`
queryable table. The parsed `FetchStep` reports `PaginationConfig::None` (not OffsetLimit).

**Test:** `test_BC_2_16_022_claroty_org_acl_policies_toml_block_parses`

### AC-002: Pagination type=none — pipeline does NOT inject offset/limit into the POST body; unbounded SELECT succeeds without 422; no count column in wire output (traces to BC-2.16.022 postcondition 4 — Pagination-None Contract; BC-2.16.022 invariant — PaginationConfig::None MUST NOT inject offset/limit)

When `claroty_organization_acl_policies` is queried, the `pipeline.rs::build_request`
function routes on `PaginationConfig::None` and constructs the POST body from
`body_template` ONLY — no `"offset": N` or `"limit": N` fields are injected.

A unit test exercising the `build_request` code path for a `PaginationConfig::None` step
MUST assert that the serialized POST body does not contain the keys `"offset"` or `"limit"`.

The live Variant-1 test `SELECT * FROM claroty.claroty_organization_acl_policies` (no LIMIT)
MUST succeed (no API 422 error); the wire JSON MUST NOT contain a `count` column (the
envelope `$.organization_acl_policies` has no count field per BC-2.16.022 §PC1 response
envelope definition).

**Tests:**
- `test_BC_2_16_022_claroty_org_acl_policies_pagination_none_no_offset_limit` (unit — mock
  `build_request` path for `PaginationConfig::None`; assert POST body lacks `offset`/`limit`)
- `test_BC_2_16_022_claroty_org_acl_policies_live_unbounded_select_no_count_column`
  (`#[ignore]` — live Variant-1; `SELECT * FROM claroty.claroty_organization_acl_policies`
  with no LIMIT; assert no E-SENSOR-001 raised; assert wire JSON rows lack a `count` key)

### AC-003: policy_acl_syntax="Cisco dACL" present in body_template; TOML authoring test asserts the literal string is present (traces to BC-2.16.022 postcondition 1 — Mandatory request parameter; BC-2.16.022 invariant)

The `body_template` string in the TOML step MUST contain `"policy_acl_syntax": "Cisco dACL"`
as a literal key-value pair in the POST body JSON. This field is REQUIRED by the
`GetOrganizationAclPoliciesRequest` OpenAPI schema — omitting it causes the Claroty API to
return an error (EC-016-022-002).

A unit test parses the TOML block and deserializes the `body_template` JSON string. The
test asserts:
1. The key `"policy_acl_syntax"` is present in the `body_template` object
2. Its value is `"Cisco dACL"` (exact string — case-sensitive)

**Test:** `test_BC_2_16_022_claroty_org_acl_policies_body_template_has_policy_acl_syntax`
(unit — parse TOML; deserialize body_template; assert policy_acl_syntax key = "Cisco dACL")

### AC-004: Four Tier-1 columns declared with correct ocsf_field; Arrow names are metadata_uid (policy_id REQUIRED), name, actor_user_name, comment (traces to BC-2.16.022 postcondition 2 — Column Tier Classification)

The `[[tables.columns]]` blocks declare exactly:
- `policy_id`: `column_type = "string"`, `ocsf_field = "metadata.uid"`, `options = ["REQUIRED"]`
- `policy_name`: `column_type = "string"`, `ocsf_field = "name"`
- `policy_updated_by`: `column_type = "string"`, `ocsf_field = "actor.user.name"`
- `policy_notes`: `column_type = "string"`, `ocsf_field = "comment"`

Under `ocsf_column_naming = true`, Arrow names resolve to `metadata_uid`, `name`,
`actor_user_name`, `comment` respectively. Exactly 4 of 11 columns have a non-None
`ocsf_field`. Exactly 7 columns have no `ocsf_field` (aggregate into `raw_extensions`).

**Test:** `test_BC_2_16_022_claroty_org_acl_policies_tier1_four_tier2_seven_correct_types`
(unit — SpecLoader::parse; inspect ColumnSpec entries; assert exactly 4 have non-None
ocsf_field; assert ocsf_field strings match exactly; assert policy_id has REQUIRED option;
assert 7 columns have None ocsf_field)

### AC-005: Tier-2 column query raises E-QUERY-038; available_columns contains metadata_uid, name, actor_user_name, comment, raw_extensions but NOT raw Tier-2 name (traces to BC-2.16.022 invariant — Tier-2 not exposed as standalone Arrow column; EC-016-022-003)

A PrismQL query `SELECT policy_source FROM claroty.claroty_organization_acl_policies LIMIT 1`
raises E-QUERY-038 (column-not-found) at plan time. The error's `available_columns`
MUST contain `raw_extensions`, `metadata_uid`, `name`, `actor_user_name`, `comment`,
`class_uid`, `_sensor` and MUST NOT contain `policy_source` as a standalone column.

Same applies for any other Tier-2 column (`policy_acl_type`, `policy_acl`, `applied_models`,
`matching_devices`, `policy_creation_date`, `policy_last_updated`).

**Test:** `test_BC_2_16_022_claroty_org_acl_policies_policy_source_tier2_e_query_038`
(integration, plan-time — SELECT policy_source raises E-QUERY-038; assert available_columns
set)

### AC-006 (WIRE-SHAPE rename): SELECT policy_id (raw TOML name) raises E-QUERY-038; available_columns contains metadata_uid but NOT policy_id (traces to BC-2.16.022 invariant — raw Tier-1 TOML name rejected; Arrow name metadata_uid is the accepted form; TV-BC-2.16.022-003; EC-016-022-004)

A PrismQL query `SELECT policy_id FROM claroty.claroty_organization_acl_policies LIMIT 1`
raises E-QUERY-038 at plan time. The error's `available_columns` MUST contain `metadata_uid`
(the Arrow form of policy_id→metadata.uid) but MUST NOT contain `policy_id` as a standalone
column name.

This wire-shape rename assertion is structurally the same as the Tier-1 raw-name rejection
in BC-2.16.020 (zone_name→name), but with an asymmetric mapping: `policy_id` is the TOML
column name and `metadata_uid` is the Arrow field name — they differ by more than one
underscore substitution.

**Test:** `test_BC_2_16_022_claroty_org_acl_policies_policy_id_raw_name_not_projected_metadata_uid_is`
(integration, plan-time — SELECT policy_id raises E-QUERY-038; assert available_columns
has metadata_uid but NOT policy_id)

### AC-007 (WIRE-SHAPE): Live Variant-1 wire-shape — SELECT * LIMIT 1 serialized JSON contains class_uid=3004, metadata_uid present non-null (UUID), name present, raw_extensions present, applied_models as JSON array not string; policy_id NOT a standalone root key (traces to BC-2.16.022 postconditions 1/2/5; TV-BC-2.16.022-001)

Against the live monroe sensor, `SELECT * FROM claroty.claroty_organization_acl_policies LIMIT 1`
serialized JSON response (MCP-visible wire shape per 2026-07-13 wire-shape discipline):
1. `class_uid` key is present with value `3004`
2. `metadata_uid` key is present and non-null (UUID string format for ACL policy ID)
3. `name` key is present (non-null or null ACL policy name string)
4. `actor_user_name`, `comment` keys are present (non-null or null)
5. `raw_extensions` key is present as a JSON object (not null, not absent)
6. `raw_extensions` JSON object contains `applied_models` key; the value is a JSON array
   (NOT a JSON-stringified array — must be parseable as an array, not a `"[...]"` string)
7. `policy_id` does NOT appear as a standalone top-level key in the row (it is renamed
   to `metadata_uid`); `policy_source`, `policy_acl`, `matching_devices`, `policy_creation_date`,
   `policy_last_updated` do NOT appear as standalone top-level keys

**Test:** `test_BC_2_16_022_claroty_org_acl_policies_live_wire_shape_class_uid_and_metadata_uid`
(`#[ignore]` — requires `CLAROTY_INSTANCE_URL` env var pointing to monroe)

### AC-008: applied_models Json column serialized as JSON array in raw_extensions, NOT as stringified value; empty array serializes as [] not null (traces to BC-2.16.022 postcondition 5 — Json column serialization; EC-016-022-005/006)

The `applied_models` column is declared `column_type = "json"` in the TOML (not `"string"`).
When the spec-engine processes an ACL policy row, `applied_models` MUST be serialized into
`raw_extensions` as a JSON-typed value (an actual JSON array object). It MUST NOT be
serialized as a JSON string token (i.e., `"[\"model1\", \"model2\"]"` as a string value
is a P1 defect — same invariant as BC-2.16.020 §Invariants).

An empty `applied_models` array MUST serialize as `[]` JSON, not null (EC-016-022-005).

**Tests:**
- `test_BC_2_16_022_applied_models_raw_extensions_json_array_not_string` (unit — mock response
  containing a row with `applied_models: ["Siemens SIMATIC S7", "Rockwell"]`; assert deserialized
  `raw_extensions["applied_models"]` is a JSON array, not a string; covers the non-empty
  sub-case only)
- `test_BC_2_16_022_applied_models_empty_array_wire_shape` (integration in prism-bin
  §bc_2_16_022_claroty_acl_policies_wire_shape — mock response with `applied_models: []`; assert
  serialized as `[]` not null; EC-016-022-005 empty-array sub-case; distinct test from RG-008)

### AC-009: Missing REQUIRED policy_id field → null row; no hard error; subsequent rows unaffected (traces to BC-2.16.022 invariant — policy_id REQUIRED semantics; EC-016-022-001)

The `policy_id` column carries `options = ["REQUIRED"]` in the TOML. When the API response
contains an ACL policy row where `policy_id` is absent or null, the spec-engine produces a
null row (REQUIRED semantics) without raising a hard error. Subsequent rows continue normally.

**Test:** `test_BC_2_16_022_null_metadata_uid_when_policy_id_absent`
(unit — mock response containing one row missing `policy_id` and one row with valid `policy_id`;
assert first row is null; assert second row is non-null with metadata_uid present; no error raised)

### AC-010: Live SELECT * (no LIMIT) returns all ACL policies in single query; no pagination error; response set count is stable (no second-page loop) (traces to BC-2.16.022 postcondition 1 pagination contract; TV-BC-2.16.022-006)

Against the live monroe sensor, `SELECT * FROM claroty.claroty_organization_acl_policies`
(no LIMIT clause) MUST succeed — no E-SENSOR-001, no API 422. The result set is the complete
list of ACL policies from the single HTTP POST response. Running the same query twice returns
the same count of rows (confirming no second-page loop is occurring).

The absence of a `count` key in the serialized wire output rows is already asserted in AC-002.
This AC additionally asserts the query completes successfully without any pagination-related
error, confirming PaginationConfig::None takes the single-fetch path.

**Test:** `test_BC_2_16_022_claroty_org_acl_policies_live_unbounded_select_no_pagination`
(NOTE: this test is the same test function named in AC-002; one live Variant-1 test covers
both the pagination-none success path and the no-count-column assertion; counted as covering
both AC-002 and AC-010)

### AC-011: Datetime fields policy_creation_date and policy_last_updated use ADR-028 §D8-B implicit iso8601 default; null passthrough when field is absent (traces to BC-2.16.022 invariant — Datetime fields; ADR-028 §D8-B)

The `policy_creation_date` and `policy_last_updated` columns are declared
`column_type = "datetime"` with NO `timestamp_formats` key in the TOML (ADR-028 §D8-B
implicit iso8601 default — `effective_formats` returns `["iso8601"]` when the declared
chain is empty/absent). When a row contains a null or absent datetime field, the
spec-engine MUST pass through null without raising E-SPEC-018 (EC-016-022-010 pattern).

**Test:** `test_BC_2_16_022_datetime_fields_null_passthrough_in_raw_extensions`
(unit — mock response with one row containing ISO-8601 datetime strings for both fields;
assert Datetime cells are non-null; second row with both fields null/absent; assert Datetime
cells are null; no E-SPEC-018 raised)

## Red Gate Tests

| ID | Test name | Test type | What it gates |
|----|-----------|-----------|---------------|
| RG-001 | `test_BC_2_16_022_claroty_org_acl_policies_toml_block_parses` | Unit (SpecLoader::parse) | AC-001: TOML block parses Ok; 11 ColumnSpec entries; PaginationConfig::None; response_path $.organization_acl_policies; body_template present |
| RG-002 | `test_BC_2_16_022_claroty_org_acl_policies_pagination_none_no_offset_limit` | Unit (build_request mock) | AC-002: PaginationConfig::None path in pipeline.rs builds POST body without offset/limit fields |
| RG-003 | `test_BC_2_16_022_claroty_org_acl_policies_body_template_has_policy_acl_syntax` | Unit (SpecLoader::parse + body_template parse) | AC-003: body_template JSON contains key "policy_acl_syntax" with value "Cisco dACL" |
| RG-004 | `test_BC_2_16_022_claroty_org_acl_policies_tier1_four_tier2_seven_correct_types` | Unit (ColumnSpec inspection) | AC-004: exactly 4 Tier-1 (policy_id→metadata.uid REQUIRED; policy_name→name; policy_updated_by→actor.user.name; policy_notes→comment); 7 Tier-2 have None ocsf_field |
| RG-005 | `test_BC_2_16_022_claroty_org_acl_policies_policy_source_tier2_e_query_038` | Integration end-to-end (prism-query, via QueryEngine::execute — authoritative) | AC-005: SELECT policy_source raises E-QUERY-038; available_columns has raw_extensions, metadata_uid, name, actor_user_name, comment but NOT policy_source |
| RG-006 | `test_BC_2_16_022_claroty_org_acl_policies_policy_id_raw_name_not_projected_metadata_uid_is` | Integration end-to-end (prism-query, via QueryEngine::execute — authoritative) | AC-006 WIRE-SHAPE rename: SELECT policy_id raises E-QUERY-038; available_columns has metadata_uid but NOT policy_id |
| RG-007 | `test_BC_2_16_022_claroty_org_acl_policies_live_wire_shape_class_uid_and_metadata_uid` | Live Variant-1 (`#[ignore]`) | AC-007 WIRE-SHAPE: wire JSON class_uid=3004, metadata_uid non-null UUID, raw_extensions present, applied_models JSON array not stringified; policy_id NOT standalone root key |
| RG-008 | `test_BC_2_16_022_applied_models_raw_extensions_json_array_not_string` | Unit inline (prism-bin/src/spec_driven_adapter.rs) | AC-008 (non-empty sub-case): non-empty applied_models array (native JSON array, not stringified) in raw_extensions |
| (RG-008 sub-case) | `test_BC_2_16_022_applied_models_empty_array_wire_shape` | Integration (prism-bin §bc_2_16_022_claroty_acl_policies_wire_shape, MED-1 EC-016-022-005) | AC-008 (empty-array sub-case, EC-016-022-005): empty applied_models [] serializes as [] not null in wire output; distinct test from RG-008 |
| RG-009 | `test_BC_2_16_022_null_metadata_uid_when_policy_id_absent` | Unit inline (prism-bin/src/spec_driven_adapter.rs) | AC-009: policy_id absent → null row; no hard error; subsequent rows continue |
| RG-010 | `test_BC_2_16_022_claroty_org_acl_policies_live_unbounded_select_no_pagination` | Live Variant-1 (`#[ignore]`) | AC-002 + AC-010: SELECT * (no LIMIT) succeeds; no E-SENSOR-001; no count column in wire JSON; result stable across two runs |
| RG-011 | `test_BC_2_16_022_datetime_fields_null_passthrough_in_raw_extensions` | Unit inline (prism-bin/src/spec_driven_adapter.rs) | AC-011: policy_creation_date and policy_last_updated parse ISO-8601 correctly; absent fields produce null cell not E-SPEC-018 |
| RG-012 | `test_BC_2_16_022_claroty_org_acl_policies_wire_shape_applied_models_json_array` | Integration (prism-bin, wire-shape serialization assertion via SpecDrivenSensorAdapter::fetch — authoritative production path) | Wire-level assertion: `applied_models` Json column serializes as JSON-typed array (not string) in wire output; `applied_models` key absent from root-level wire envelope; fetch path is authoritative (no DTU for ACL policies per D-2200) |

**BC-5.38.001 density check:** 13 Red Gate tests / 11 acceptance criteria = 1.18 ≥ 0.5 threshold. PASS.
(Note: RG-010 covers two ACs — AC-002 and AC-010 — counted as 1 RGT per 1 distinct failing test function. RG-012 is a supplemental wire-shape gate — no separate AC assigned; counted toward density numerator only. RG-008 sub-case (`test_BC_2_16_022_applied_models_empty_array_wire_shape`) is a supplemental integration test covering EC-016-022-005 — counted toward density numerator only.)

## Architecture Mapping

| Component | Module | Pure/Effectful |
|-----------|--------|---------------|
| `claroty_organization_acl_policies` TOML block | `crates/prism-sensors/specs/claroty.sensor.toml` | Static data (TOML spec) |
| TOML parse validation | `crates/prism-spec-engine/src/spec_parser.rs §spec_parser` | Pure (TOML deserialization; no I/O) |
| Tier-1/Tier-2 Arrow schema computation | `crates/prism-spec-engine/src/column_mapping.rs §ocsf_field_to_arrow_name` | Pure (string transformation; deterministic) |
| `PaginationConfig::None` build_request path | `crates/prism-spec-engine/src/pipeline.rs §build_request` | Pure (body construction; no I/O; no offset/limit injection) |
| PaginationConfig::None single-fetch execution | `crates/prism-spec-engine/src/pipeline.rs §PipelineExecutor::execute` | Effectful (HTTP POST to xDome; single response; no loop) |
| Json column serialization into raw_extensions | `crates/prism-spec-engine/src/pipeline.rs §PipelineExecutor::execute` | Effectful (processes HTTP response; builds Arrow RecordBatch) |
| response_path extraction + null-passthrough | `crates/prism-bin/src/spec_driven_adapter.rs §pipeline_result_to_record_batch` (contains `build_column_array`) | Effectful (processes HTTP response; REQUIRED-field null rows; Tier-2 → raw_extensions) |
| `entity_management` class arm (shared) | `crates/prism-ocsf/src/class_selector.rs::select_by_class_name` | Pure (constant → u32 lookup; arm already exists; returns 3004) |

Architecture section references:
- `architecture/module-decomposition.md` §SS-01 Sensor Adapters (prism-sensors; claroty.sensor.toml)
- `architecture/module-decomposition.md` §SS-16 Spec Engine (prism-spec-engine; spec_parser, pipeline, column_mapping)
- ADR-058 §B2 (Tier-2 raw_extensions aggregation), §C (Arrow field naming; metadata.uid → metadata_uid; actor.user.name → actor_user_name), §D (ocsf_column_naming per-sensor flag)
- ADR-028 §D8-B (implicit iso8601 default when timestamp_formats omitted)

## Purity Classification

- **Pure functions (no I/O, deterministic):** `SpecLoader::parse` (TOML deserialization);
  `ocsf_field_to_arrow_name` (string → string); `select_by_class_name("entity_management")`
  (constant lookup → 3004); `build_request` for `PaginationConfig::None` (body_template
  construction only, no injection); RG-001/003/004/008/009/011 unit tests.
- **Effectful functions (I/O, network):** `PipelineExecutor::execute` for the ACL policies
  step (single HTTP POST to /api/v1/organization_acl_policies/); `pipeline_result_to_record_batch`
  (HTTP response → Arrow RecordBatch; Json column handling); RG-002 (mock — partially effectful
  boundary test); RG-007/010 live integration tests (require running monroe sensor).

---

## Edge Cases

| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-001 | Row in `claroty_organization_acl_policies` missing `policy_id` (REQUIRED) | Null row produced; no hard error; response continues (EC-016-022-001) |
| EC-002 | `body_template` omits `policy_acl_syntax` (TOML misconfiguration) | API returns 422 or 400; E-SENSOR-001 raised; entire query fails (single-fetch; no partial results) (EC-016-022-002) |
| EC-003 | Query references Tier-2 `policy_source` by raw name | E-QUERY-038; available_columns has metadata_uid, name, actor_user_name, comment, raw_extensions but NOT policy_source (EC-016-022-003) |
| EC-004 | Query references Tier-1 raw TOML name `policy_id` (not Arrow name `metadata_uid`) | E-QUERY-038; available_columns has metadata_uid but NOT policy_id (EC-016-022-004) |
| EC-005 | `applied_models` is an empty array `[]` | Serialized as `[]` JSON in raw_extensions; not null; no error (EC-016-022-005) |
| EC-006 | `applied_models` declared as `column_type = "string"` instead of `"json"` | Array serialized as raw string token in raw_extensions.applied_models — P1 TOML authoring defect; caught by structural test asserting JSON-typed value (EC-016-022-006) |
| EC-007 | `pagination.type` mistakenly set to `"offset_limit"` instead of `"none"` | Pipeline injects offset/limit into POST body; API returns 422 validation error; E-SENSOR-001 raised; entire query fails (EC-016-022-007) |
| EC-008 | `pagination` section omitted entirely from the step | Defaults to PaginationType::None via `.unwrap_or` in types.rs — behaviorally equivalent; NOT recommended (explicit declaration required per BC-2.16.022 §PC4 invariant) (EC-016-022-008) |
| EC-009 | `policy_acl_syntax = "ArubaOS-Switch"` used instead of `"Cisco dACL"` | API returns ACL text in ArubaOS-Switch format; valid API call but outside v1 contract scope (deferred configurability) (EC-016-022-009) |
| EC-010 | `policy_updated_by` absent in a row | Null `actor_user_name` Arrow cell; not an error (EC-016-022-010) |
| EC-011 | `policy_creation_date` or `policy_last_updated` contains non-ISO-8601 non-null value | E-SPEC-018 TimestampParseFailure; null demoted with warning; row continues |
| EC-012 | API returns non-200 HTTP for POST /api/v1/organization_acl_policies/ | E-SENSOR-001 with sensor=claroty, status, body; no partial results (single-fetch endpoint; no prior pages accumulated) |

## TOML Column-Block Specification

The complete `[[tables]]` block as specified by BC-2.16.022 §PC1/§PC2:

```toml
# Wave C G6 — claroty_organization_acl_policies
# POST /api/v1/organization_acl_policies/ → envelope key: organization_acl_policies (NO count field)
# OCSF class: entity_management (class_uid 3004; existing arm in class_selector.rs)
# PK: policy_id (String, REQUIRED, single-column) → Arrow metadata_uid (metadata.uid)
# PAGINATION: type = "none" — NON-PAGINATED SINGLE-FETCH (only Claroty table of this kind)
#   MUST NOT inject offset/limit into POST body (GetOrganizationAclPoliciesRequest has no these fields)
# MANDATORY body field: policy_acl_syntax = "Cisco dACL" (REQUIRED per OpenAPI schema)
# DTU status: NONE — SAP-2 probe N/A; near-term tests against live monroe only (D-2200 deferred)
[[tables]]
table_name = "organization_acl_policies"           # registered/queryable name = {sensor_id}_{table_name} = "claroty_organization_acl_policies"
ocsf_class = "entity_management"   # class_uid 3004 (existing arm; same as claroty_audit_logs)

# Tier-1: policy_id → metadata_uid (REQUIRED; UUID-format primary key; OCSF entity identifier)
[[tables.columns]]
name = "policy_id"
column_type = "string"
ocsf_field = "metadata.uid"
options = ["REQUIRED"]

# Tier-1: policy_name → name (human-readable display label of the ACL policy)
[[tables.columns]]
name = "policy_name"
column_type = "string"
ocsf_field = "name"

# Tier-1: policy_updated_by → actor_user_name (email/username of analyst who last modified)
[[tables.columns]]
name = "policy_updated_by"
column_type = "string"
ocsf_field = "actor.user.name"

# Tier-1: policy_notes → comment (free-text analyst notes for the ACL policy)
[[tables.columns]]
name = "policy_notes"
column_type = "string"
ocsf_field = "comment"

# Tier-2: "Custom" or system source tag for this policy
[[tables.columns]]
name = "policy_source"
column_type = "string"

# Tier-2: ACL syntax type returned by API (e.g., "Cisco dACL")
[[tables.columns]]
name = "policy_acl_type"
column_type = "string"

# Tier-2: raw multi-line ACL text in the requested syntax format (Cisco dACL for v1)
[[tables.columns]]
name = "policy_acl"
column_type = "string"

# Tier-2: array of device model strings to which this ACL applies — MUST be json, not string
[[tables.columns]]
name = "applied_models"
column_type = "json"

# Tier-2: count of devices currently matching this ACL policy
[[tables.columns]]
name = "matching_devices"
column_type = "integer"

# Tier-2: ACL policy creation timestamp; ISO 8601; ADR-028 §D8-B implicit iso8601 default
[[tables.columns]]
name = "policy_creation_date"
column_type = "datetime"

# Tier-2: last modification timestamp; ISO 8601; ADR-028 §D8-B implicit iso8601 default
[[tables.columns]]
name = "policy_last_updated"
column_type = "datetime"

[[tables.steps]]
name = "fetch_organization_acl_policies"
method = "POST"
path_template = "/api/v1/organization_acl_policies/"
body_template = '{"policy_acl_syntax": "Cisco dACL", "fields": ["policy_id", "policy_name", "policy_source", "applied_models", "matching_devices", "policy_acl_type", "policy_acl", "policy_creation_date", "policy_last_updated", "policy_updated_by", "policy_notes"]}'
response_path = "$.organization_acl_policies"
variables_produced = []

[tables.steps.pagination]
type = "none"
```

## Token Budget Estimate

| Item | Estimated tokens |
|------|-----------------|
| This story spec | ~8,000 |
| `crates/prism-sensors/specs/claroty.sensor.toml` (existing 4-table baseline; may be higher at implementation time if sibling expansion stories merge first per depends_on) | ~4,000 |
| BC-2.16.022 (full) | ~6,500 |
| ADR-058 §B2/§C/§D sections (Tier-1/Tier-2; metadata.uid → metadata_uid; actor.user.name → actor_user_name) | ~3,000 |
| ADR-028 §D8-B (implicit iso8601 default; datetime passthrough) | ~1,000 |
| `crates/prism-spec-engine/src/spec_parser.rs` (ColumnSpec + FetchStep + PaginationConfig::None variant) | ~3,500 |
| `crates/prism-spec-engine/src/pipeline.rs §build_request` (PaginationConfig::None path; no offset/limit injection) | ~2,500 |
| Test files (11 RGTs; 9 unit/integration + 2 live) | ~6,000 |
| spike-findings §Spike 4 (pagination=none decision, policy_acl_syntax mandatory, body_template) | ~1,500 |
| endpoint-schema-extract.md §organization_acl_policies section | ~1,000 |
| **Total estimate** | **~37,000 tokens** |

Well within 20-30% of a 200K window (~18%). If context is tight at implementation time,
load BC-2.16.022 first (it is the authoritative source for all column types, ocsf_field
values, body_template, and response_path), then load the existing `claroty.sensor.toml
§alerts` block as the TOML formatting reference, then load `spec_parser.rs §PaginationConfig`
for the None variant deserialization. Load BC-2.16.022 before writing tests — wire-shape
assertions must be derived from BC postconditions, not invented.

## Tasks

Tasks follow red-then-green ordering (SAC-1): ALL test-writing tasks MUST precede
ALL implementation tasks. The Red Gate check in Task 7 confirms all RGTs fail before
the TOML block and any production code changes are made.

- [ ] **Task 1 (Red Gate — test first):** Write RG-001 and RG-003:
  `test_BC_2_16_022_claroty_org_acl_policies_toml_block_parses` and
  `test_BC_2_16_022_claroty_org_acl_policies_body_template_has_policy_acl_syntax` in
  `crates/prism-spec-engine/src/spec_parser.rs #[cfg(test)] mod tests` (or a dedicated
  TOML fixture file if the existing pattern uses fixtures). Call `SpecLoader::parse` on
  `claroty.sensor.toml` (or inline fixture containing the new block). Assert `Ok(SensorSpec)`;
  11 ColumnSpec entries for `claroty_organization_acl_policies`; `PaginationConfig::None` in
  the parsed step; `body_template` deserialized JSON has `"policy_acl_syntax": "Cisco dACL"`.
  MUST fail before Task 8 (block not yet in TOML).

- [ ] **Task 2 (Red Gate — test first):** Write RG-004:
  `test_BC_2_16_022_claroty_org_acl_policies_tier1_four_tier2_seven_correct_types`.
  Assert exactly 4 ColumnSpec entries have non-None `ocsf_field`; assert the exact ocsf_field
  strings (`"metadata.uid"` with REQUIRED for `policy_id`, `"name"` for `policy_name`,
  `"actor.user.name"` for `policy_updated_by`, `"comment"` for `policy_notes`); assert 7
  columns have None ocsf_field. MUST fail before Task 8.

- [ ] **Task 3 (Red Gate — test first):** Write RG-002:
  `test_BC_2_16_022_claroty_org_acl_policies_pagination_none_no_offset_limit`.
  Exercise the `pipeline.rs::build_request` code path for a `PaginationConfig::None` step
  (unit test using a mock step with pagination=None and a body_template). Deserialize the
  returned POST body JSON. Assert the body does NOT contain `"offset"` or `"limit"` keys.
  Assert the body DOES contain `"policy_acl_syntax": "Cisco dACL"` and the `"fields"` array.
  MUST fail before Task 8 (if the None path doesn't exist yet) or before Task 9 (if the
  guard is absent).

- [ ] **Task 4 (Red Gate — test first):** Write RG-005 and RG-006:
  `test_BC_2_16_022_claroty_org_acl_policies_policy_source_tier2_e_query_038` and
  `test_BC_2_16_022_claroty_org_acl_policies_policy_id_raw_name_not_projected_metadata_uid_is`.
  Integration plan-time tests: `SELECT policy_source` raises E-QUERY-038 with correct
  `available_columns` set (contains `metadata_uid`, not `policy_source`);
  `SELECT policy_id` raises E-QUERY-038 with `available_columns` containing `metadata_uid`
  but NOT `policy_id`. MUST fail before Task 8.

- [ ] **Task 5 (Red Gate — test first):** Write RG-008, RG-009, RG-011:
  `test_BC_2_16_022_applied_models_raw_extensions_json_array_not_string`,
  `test_BC_2_16_022_null_metadata_uid_when_policy_id_absent`,
  `test_BC_2_16_022_datetime_fields_null_passthrough_in_raw_extensions`.
  Unit tests with mock responses:
  - applied_models mock: row with `applied_models: ["ModelA", "ModelB"]`; assert
    `raw_extensions["applied_models"]` is a JSON array (not string). Second mock:
    `applied_models: []`; assert serialized as `[]` not null.
  - required policy_id mock: row with missing `policy_id` + row with valid UUID `policy_id`;
    assert first row is null; second row has non-null `metadata_uid`.
  - datetime mock: row with valid ISO-8601 datetime strings → non-null Datetime cells;
    row with null/absent datetime fields → null cells; no E-SPEC-018.
  MUST fail before Task 8.

- [ ] **Task 6 (Red Gate — test first):** Write RG-007 and RG-010 (live `#[ignore]` tests):
  `test_BC_2_16_022_claroty_org_acl_policies_live_wire_shape_class_uid_and_metadata_uid`
  and `test_BC_2_16_022_claroty_org_acl_policies_live_unbounded_select_no_pagination`.
  Mark both `#[ignore]` with comment:
  `// LIVE-MONROE-001: requires CLAROTY_INSTANCE_URL env var pointing to monroe; run manually or in live-validation CI job`.
  Wire-level JSON assertions per AC-007 and AC-010 respectively.
  MUST fail (when run without `--ignored`, these are ignored — the "fail" counts as not-yet-green
  since #[ignore]'d tests do not pass without explicit `--ignored` flag).

- [ ] **Task 7 (Red Gate verification):** Run `cargo nextest run -p prism-sensors --no-fail-fast`
  (or `just iter prism-sensors`). Confirm RG-001..RG-006/RG-008/RG-009/RG-011 (9 non-ignored tests)
  ALL FAIL. Density: 9 failing + 2 ignored / 11 total = expected state. Proceed to Task 8 only
  after confirming all non-ignored RGTs are red.

- [ ] **Task 8 (Implement — TOML):** Add the `[[tables]]` block from §TOML Column-Block
  Specification to `crates/prism-sensors/specs/claroty.sensor.toml`. Insert after the last
  existing table block (or after any sibling expansion tables that have merged first per
  depends_on ordering — verify the actual TOML at implementation time). Verify TOML syntax
  with `toml_edit::parse` or `just check-fast` before proceeding.

- [ ] **Task 9 (Implement — pipeline guard, if absent):** Read
  `crates/prism-spec-engine/src/pipeline.rs §build_request`. Verify that the
  `PaginationConfig::None` arm explicitly skips offset/limit injection. If the guard is
  absent or falls through to an injection path, add the explicit None arm now. This is
  a production-grade requirement (BC-2.16.022 §Invariants): injecting offset/limit when
  `type = "none"` is declared causes an API 422. Run RG-002 after this task.

- [ ] **Task 10 (Green iteration):** Run `just iter prism-sensors` iteratively until
  RG-001/003/004/005/006/008/009/011 pass. Run `just iter prism-spec-engine` for
  pipeline-level tests (RG-002). Confirm density 9/9 non-ignored green.
  Run `just check` for full workspace gate before declaring local work done.

- [ ] **Task 11 (Holdout gate — BLOCKING):** Run story-level holdout gate (HS-029) per
  BC-5.39.001 BEFORE demo recording or push. Holdout-evaluator runs hidden HS-029 scenarios
  against the built binary (real MCP stdio + live monroe, wire-level assertions, scoped to
  claroty_organization_acl_policies surface). Any unsatisfied scenario routes as OBSERVED
  BEHAVIOR ONLY (never scenario text — contamination control) and resets the LOCAL
  streak per BC-5.39.001.

## Previous Story Intelligence

**S-CLAROTY-ORGPOLICY-001** (sibling, Wave C G5, materialized draft pending — NOT merged):
Delivered 4 TOML blocks (organization_zones, organization_zone_policies,
organization_firewall_groups, organization_firewall_policies) all using `type = "offset_limit"`
/ `page_size = 1000`. Key structural lessons carried forward:
- All four ORGPOLICY tables use `entity_management`/class_uid 3004 — confirmed correct arm
  exists in `class_selector.rs::select_by_class_name` per spike-findings §Overall Verdict.
  No new arm needed for ACL policies either.
- Json columns (`applied_models`) MUST be declared `column_type = "json"` — the `"string"`
  mistake causes array serialization as a raw string token (a P1 defect caught by RG-008).
- Tier-1 TOML column names ARE NOT the Arrow field names under `ocsf_column_naming = true`.
  `policy_id` is the TOML column name; `metadata_uid` is the Arrow name. `SELECT policy_id`
  raises E-QUERY-038 (AC-006 / RG-006).
- The ORGPOLICY story confirms `ocsf_column_naming = true` is set at sensor level (PR #242);
  the new ACL table inherits automatically.

**THIS STORY'S HEADLINE NOVELTY (distinct from ORGPOLICY):** `pagination.type = "none"`.
This is the FIRST Claroty table (and first prism sensor table in Wave C) without
offset/limit pagination. The `body_template` is the COMPLETE POST body — including the
mandatory `policy_acl_syntax` field. For offset_limit tables, `body_template` carries only
`fields` (pagination is injected separately). For this table, `body_template` is the entire
request body. The implementer MUST verify `pipeline.rs::build_request` has an explicit
`PaginationConfig::None` arm before assuming the table works correctly.

**S-CLAROTY-SERVERS-001** (sibling, Wave C G4, materialized draft pending — NOT merged):
Used `inventory_info`/5001 OCSF class arm (new arm). No lesson applies to this story
(ACL policies uses the existing `entity_management`/3004 arm, not the servers arm).

**N/A — first ACL policies story:** No predecessor story covers this specific table surface.

## Architecture Compliance Rules

Extracted from ADR-058 and ADR-028 (section-anchored cites per TD-VSDD-091):

1. **ADR-058 §B2 — Tier-2 raw_extensions aggregation:** Columns without `ocsf_field` MUST
   NOT be exposed as standalone Arrow columns when `ocsf_column_naming = true`. They
   aggregate into the `raw_extensions` JSON column. SELECT by raw Tier-2 name MUST raise
   E-QUERY-038 at plan time (not at execution time).

2. **ADR-058 §C — Arrow field naming for nested ocsf_field paths:** Dots become underscores:
   `"actor.user.name"` → `actor_user_name`; `"metadata.uid"` → `metadata_uid`. These are
   the canonical Arrow field names in the RecordBatch — not the raw TOML column names.

3. **ADR-058 §D — sensor-level ocsf_column_naming flag:** `ocsf_column_naming = true` is
   declared once at the sensor level in `claroty.sensor.toml` (set by S-ADR058-OCSF-ROUTING-001,
   merged PR #242). ALL tables in the claroty spec inherit this. No per-table override needed.

4. **ADR-028 §D8-B — implicit iso8601 default:** When `timestamp_formats` is absent from a
   Datetime column declaration, `effective_formats` returns `["iso8601"]`. This is correct and
   expected. Do NOT add `timestamp_formats = ["iso8601"]` to the column — omission is the
   intended canonical form per ADR-028 §D8-B backward compatibility.

5. **BC-2.16.022 §Invariants — PaginationConfig::None guard:** The `build_request` function
   in `pipeline.rs` MUST have a routing path that skips offset/limit injection when
   `PaginationConfig::None` is active. If this routing is absent, the offset/limit fields
   will be injected into the body and the Claroty API will return a 422 validation error.
   This is a CORRECTNESS invariant, not a style preference.

6. **BC-2.16.022 §Invariants — applied_models Json:** `applied_models` MUST be declared
   `column_type = "json"` (not `"string"`). The Claroty API returns this field as a JSON
   array. Declaring it as string serializes the array as a raw string token — a P1 defect.

7. **No new class_selector arm required:** The `entity_management` arm already exists in
   `prism-ocsf/src/class_selector.rs::select_by_class_name` (confirmed by spike-findings
   §Overall Verdict and used by BC-2.16.020/021). Adding a new arm would be a production
   code change outside this story's scope.

8. **Forbidden dependency rule:** `prism-sensors` MUST NOT gain a direct dependency on
   `prism-query` in this story (the TOML spec authoring is data, not query execution logic).
   Plan-time E-QUERY-038 verification tests run through `prism-query` (via
   QueryEngine::execute — the authoritative end-to-end surface per SAP-3 rule 3), NOT
   through `prism-spec-engine`.

## Library & Framework Requirements

No new dependencies are introduced by this story. All required mechanisms are already
present:

| Library/Crate | Version pin | Usage in this story |
|---------------|-------------|---------------------|
| `serde` | per `Cargo.toml` workspace pin | TOML deserialization in spec_parser |
| `toml` | per `Cargo.toml` workspace pin | claroty.sensor.toml parsing |
| `serde_json` | per `Cargo.toml` workspace pin | body_template JSON parse in tests; raw_extensions serialization |
| `prism-spec-engine` | workspace local | SpecLoader::parse, PipelineExecutor, column_mapping |
| `prism-ocsf` | workspace local | select_by_class_name("entity_management") → 3004 |
| `prism-sensors` | workspace local | claroty.sensor.toml (TOML spec authoring) |

**Rust toolchain:** per `rust-toolchain.toml` (stable, pinned). No nightly features.

## File Structure Requirements

| File | Action | Purpose |
|------|--------|---------|
| `crates/prism-sensors/specs/claroty.sensor.toml` | MODIFY — append 1 `[[tables]]` block | Add `organization_acl_policies` (bare name; registered/queryable `claroty_organization_acl_policies`) per §TOML Column-Block Specification |
| `crates/prism-spec-engine/src/spec_parser.rs` | MODIFY (add tests) | RG-001/003/004/011 unit tests in `#[cfg(test)] mod tests`; verify TOML parse with 11 ColumnSpec entries; verify PaginationConfig::None; verify body_template has policy_acl_syntax |
| `crates/prism-spec-engine/src/pipeline.rs` | MODIFY (add tests; possibly add guard) | RG-002 unit test for build_request PaginationConfig::None path; if guard is absent, add explicit None arm before offset/limit injection |
| `crates/prism-query/tests/bc_2_16_022_test.rs` | MODIFY (add integration tests) | RG-005/006 plan-time E-QUERY-038 integration tests — AUTHORITATIVE gate via QueryEngine::execute (per SAP-3 rule 3); SELECT policy_source raises E-QUERY-038 with correct available_columns (RG-005); SELECT policy_id raises E-QUERY-038 with available_columns containing metadata_uid but NOT policy_id (RG-006) |
| `crates/prism-sensors/tests/` or `crates/prism-spec-engine/tests/` | MODIFY (add live tests) | RG-007/010 live Variant-1 `#[ignore]`'d integration tests against live monroe; must carry the LIVE-MONROE-001 comment |
| `crates/prism-spec-engine/src/pipeline.rs` or `crates/prism-sensors/tests/` | MODIFY (add unit tests) | RG-008/009 mock response unit tests for applied_models Json serialization and REQUIRED policy_id semantics |
| `crates/prism-bin/tests/bc_2_16_022_claroty_acl_policies_wire_shape.rs` | CREATE | RG-012 prism-bin wire-shape test: SpecDrivenSensorAdapter::fetch path; serialized JSON assertion for applied_models Json column (JSON array, not string); wire-envelope shape (fetch path is authoritative; no DTU for ACL policies per D-2200) |
| `crates/prism-bin/Cargo.toml` | MODIFY | Add `[[test]]` entry for `bc_2_16_022_claroty_acl_policies_wire_shape` |

**File-count summary:** 1 TOML modification + test additions across 2–3 existing test files + 1 new prism-bin wire-shape test file + Cargo.toml update.

## Notes for Implementer

1. **Read BC-2.16.022 before writing any code.** The BC contains the authoritative
   body_template (including `policy_acl_syntax`) and the full 11-column field list. The
   body_template from BC-2.16.022 §PC1 is the canonical reference — do not use the
   abbreviated version from spike-findings §Spike 4 if there is a discrepancy.

2. **`pagination.type = "none"` is structurally distinct from the default pagination.**
   Read `pipeline.rs §build_request` to understand where offset/limit injection occurs,
   and verify the None arm exists BEFORE writing the TOML block. If the guard is absent
   and you write the TOML first, the integration tests may pass locally (if offset/limit
   injection doesn't cause a test failure because the live sensor happens to ignore extra
   fields), while the live API call would fail in production with a 422.

3. **`policy_acl_syntax` is NOT in the fields list.** The `"fields"` array in
   `body_template` contains the column field names. `policy_acl_syntax` is a separate
   request-level parameter that appears at the same JSON level as `"fields"`, not inside it.
   The body_template must be: `{"policy_acl_syntax": "Cisco dACL", "fields": [...]}`.

4. **Verify baseline table count at implementation time.** The baseline is 4 tables at
   develop@3f1e66179. If sibling expansion stories (VULNS-001, OT-EVENTS-001, DEVVULNREL-001,
   SERVERS-001, ORGPOLICY-001) have merged first, the baseline will be higher. The test for
   AC-001 (RG-001) may need to be written to assert the ACL policies table is PRESENT in the
   spec (not to assert a specific total table count) to avoid brittleness.

5. **`applied_models` is a `column_type = "json"` Tier-2 column.** When the Claroty API
   returns `"applied_models": ["Siemens SIMATIC S7", "Rockwell Allen-Bradley"]`, the
   spec-engine should serialize this as a JSON array value inside `raw_extensions`, not as a
   string. The test in RG-008 asserts `serde_json::Value::Array` not `serde_json::Value::String`.

6. **`metadata_uid` is the Arrow field name for `policy_id`.** Under `ocsf_column_naming = true`,
   `policy_id` (TOML column name) with `ocsf_field = "metadata.uid"` maps to Arrow field name
   `metadata_uid` (via `ocsf_field_to_arrow_name("metadata.uid")` = `"metadata_uid"`). This
   is a two-segment flattening: `metadata.uid` → `metadata_uid`. Test for this exact string.

7. **Holdout gate HS-029 is BLOCKING.** After LOCAL 3-CLEAN and before demo recording / push,
   the holdout-evaluator runs HS-029. The scenario files are HIDDEN from test-writer and
   implementer (contamination control). Do not attempt to read, locate, or infer the HS-029
   scenarios. Report the gate as PASS or FAIL based on the holdout-evaluator's verdict only.

8. **SAP-2 is explicitly N/A.** Do not run SAP-2 checks against `crates/prism-dtu-claroty/`.
   No `organization_acl_policies.rs` route exists in the DTU crate, and its absence is expected
   and documented. The DTU story for this endpoint is tracked as D-2200.

## Live-Test Approach

Per xdome-endpoint-expansion-plan.md §Per-Story Pipeline and BC-2.16.022 §Canonical Test Vectors:

**Variant-1 (structural, required before holdout gate):**
- `test_BC_2_16_022_claroty_org_acl_policies_live_wire_shape_class_uid_and_metadata_uid`
  (RG-007): `SELECT * FROM claroty.claroty_organization_acl_policies LIMIT 1`
  Wire assertions: `class_uid = 3004`; `metadata_uid` present non-null (UUID); `name` present;
  `raw_extensions` present as JSON object; `raw_extensions["applied_models"]` is a JSON array
  (not a JSON-stringified array); `policy_id` NOT a standalone root key.
  Corresponds to TV-BC-2.16.022-001.

- `test_BC_2_16_022_claroty_org_acl_policies_live_unbounded_select_no_pagination`
  (RG-010): `SELECT * FROM claroty.claroty_organization_acl_policies` (no LIMIT)
  Wire assertions: query completes without E-SENSOR-001; no `count` key in any row; running
  the query twice yields the same row count (no pagination loop occurring).
  Corresponds to TV-BC-2.16.022-006.

Both live tests marked `#[ignore]` with comment:
`// LIVE-MONROE-001: requires CLAROTY_INSTANCE_URL env var pointing to monroe; run manually or in live-validation CI job`

**Run command for live tests:**
```bash
CLAROTY_INSTANCE_URL=<monroe-url> cargo nextest run -p prism-sensors \
  --run-ignored ignored-only \
  -E 'test(claroty_org_acl_policies_live)'
```

**Variant-2 (agent, optional):** PrismQL agent-level test exercising the full LLM
reasoning path. Deferred to live-validation milestone if not complete before holdout gate.

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.5 | story-doc-consistency-g6-low001-obs001 | 2026-09-01 | story-writer | LOW-001: §File Structure Requirements RG-005/006 row corrected — stale `crates/prism-sensors/tests/ or prism-spec-engine/tests/` attribution and "defense-in-depth; authoritative in prism-bin" text removed; replaced with `crates/prism-query/tests/bc_2_16_022_test.rs` AUTHORITATIVE via QueryEngine::execute (per SAP-3 rule 3). OBS-001: §Red Gate Tests RG-008 scope narrowed to non-empty applied_models sub-case (native JSON array, not stringified); RG-008 sub-case row added for `test_BC_2_16_022_applied_models_empty_array_wire_shape` (EC-016-022-005, prism-bin §bc_2_16_022_claroty_acl_policies_wire_shape); §AC-008 §Test wording split into two named tests (non-empty unit test + empty-array integration test) to eliminate false implication that both sub-cases run in RG-008; density check updated 12→13 RGTs. |
| 1.4 | story-doc-governance-sweep-g3-g4-g5-g6 | 2026-08-31 | story-writer | FIX 1 (POL-39): Removed volatile BC-2.16.022 version pins from §Authority (§Postconditions §1..§4 + §Invariants headings) and §Token Budget row; §Behavioral Contracts table Version synced to v1.2 per POL-40 current-state pin. FIX 3 (§Red Gate Tests + §Architecture Compliance): Synced 8 RG rows to delivered test names and crate paths (RG-004/007/010 name rename; RG-005/006 prism-bin→prism-query + removed SAP-3 defense-in-depth text; RG-008/009/011 Unit inline prism-bin/src/spec_driven_adapter.rs); §AC §Tasks §Live-Test Approach test names updated in all occurrences via replace_all. Rule #8 corrected: E-QUERY-038 plan-time tests run through prism-query (authoritative per SAP-3 rule 3), NOT prism-spec-engine. |
| 1.3 | post-delivery-test-sync | 2026-08-31 | story-writer | FIX A: §TOML Column-Block Specification — 11 `column_name =` occurrences changed to `name =` (ColumnSpec deserializer field). FIX B: §Red Gate Tests RG-012 mechanism corrected from QueryEngine::execute → SpecDrivenSensorAdapter::fetch (authoritative production path; no DTU for ACL policies per D-2200). FIX C: §File Structure Requirements RG-012 CREATE entry description corrected to SpecDrivenSensorAdapter::fetch path with DTU-absence rationale. |
| 1.2 | g2-proven-spec-prose-corrections | 2026-08-31 | story-writer | G2-proven spec-prose corrections applied (mirrors S-CLAROTY-OT-EVENTS-001 v1.3 fixes). FIX 1 (MED-1 table_name): `[[tables]]` block now uses bare `table_name = "organization_acl_policies"`; §Authority, TOML block, and AC-001 updated; derivation note `{sensor_id}_{table_name}` = `claroty_organization_acl_policies` registered/queryable name added; SELECT examples and error prose retain prefixed queryable names unchanged. FIX 2 (MED-3 mechanism attribution): N/A — no ColumnMapper::map_record references in this story. FIX 3 (MED-4 prism-bin declaration): `crates_touched` adds `prism-bin` and `prism-query`; RG-005/RG-006 updated to prism-bin end-to-end authoritative (plan-time prism-sensors tests remain defense-in-depth per SAP-3 rule 3); RG-012 added (wire-shape serialization assertion in prism-bin for `applied_models` Json column); density check 11→12 RGTs / 11 ACs = 1.09 PASS; §File Structure Requirements adds CREATE `crates/prism-bin/tests/bc_2_16_022_claroty_acl_policies_wire_shape.rs` + MODIFY `crates/prism-bin/Cargo.toml`; TOML block description updated to bare name. FIX 4 (§Architecture Mapping path): `crates/prism-spec-engine/src/spec_driven_adapter.rs §pipeline_result_to_record_batch` corrected to `crates/prism-bin/src/spec_driven_adapter.rs §pipeline_result_to_record_batch` (contains `build_column_array`). |
| 1.1 | remove-uncertainty-aclpolicy-g6 | 2026-08-31 | research-agent | Remove-uncertainty pass (also satisfies mandatory pre-delivery pass per D-1110). Validated all API/technology assumptions against ground truth (`xdome_openapi_06.20.2026.json`, `endpoint-schema-extract.md §organization_acl_policies`, `endpoint-spike-findings.md §Spike 4`, `claroty.sensor.toml`, prism-spec-engine/prism-ocsf/prism-dtu-claroty source) — see populated `assumption_validations` frontmatter (8 assumptions, all CONFIRMED). Endpoint/envelope/response_path, required policy_acl_syntax, 11-field enum↔body_template parity (incl. order), all 11 column types, pagination-none mechanism (build_request page_size=0 skip; request additionalProperties=false), Arrow-name flattening, entity_management→3004 arm, 4-table baseline, ocsf_column_naming=true, SAP-2 N/A, and datetime implicit-iso8601 all confirmed. No corrections to this story (its §TOML Column-Block Specification body_template was already the valid single-line form). Companion correction landed in BC-2.16.022 v1.1: §PC1 body_template re-rendered from an invalid multi-line single-quoted TOML literal (backslash line-continuations, not legal TOML) to a valid single-line literal identical to this story's §TOML block; story Notes-for-Implementer #1 (BC §PC1 = canonical body_template reference) remains accurate post-fix. Status left draft (no change). BC dependency bumped to BC-2.16.022 v1.1. |
| 1.0 | xdome-wave-c-f3-story-materialization-g6 | 2026-08-24 | story-writer | Initial materialization — Wave C G6: claroty_organization_acl_policies TOML block (1 table, 11 columns: 4 Tier-1 [policy_id→metadata.uid REQUIRED/metadata_uid, policy_name→name, policy_updated_by→actor_user_name, policy_notes→comment] + 7 Tier-2 [policy_source/String, policy_acl_type/String, policy_acl/String, applied_models/Json, matching_devices/Integer, policy_creation_date/Datetime, policy_last_updated/Datetime]). KEY NOVELTY: pagination type=none (non-paginated single-fetch; no offset/limit injection); mandatory policy_acl_syntax="Cisco dACL" in body_template (REQUIRED per OpenAPI schema). OCSF entity_management/3004 (existing arm). 11 ACs; 11 RGTs; density 1.0. tdd_mode: strict. SAP-2 N/A (no DTU; D-2200 deferred). HS-029 holdout BLOCKING. depends_on: []. BC: BC-2.16.022 v1.0. |
