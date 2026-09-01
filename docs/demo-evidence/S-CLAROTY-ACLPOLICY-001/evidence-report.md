# Demo Evidence Report — S-CLAROTY-ACLPOLICY-001

**Story:** Claroty xDome ACL Policies Table — claroty_organization_acl_policies TOML block with
11-column Tier-1/Tier-2 spec, pagination type=none (non-paginated single-fetch), mandatory
policy_acl_syntax=Cisco dACL body field, live structural tests (Wave C G6)
**Story version:** v1.6
**Evidence date:** 2026-09-01
**Recorder:** demo-recorder
**Product type:** CLI (Rust workspace)
**Recording tools:** VHS 0.11.0 (terminal session recordings) + annotated product-schema transcript

---

## Live Validation Status

Per human-directed policy D-2410b + D-2410j (2026-09-01): live-tenant query result rows (real
tenant data) are NOT committed to the repository. The live monroe tenant currently has 0
organization ACL policies, so an empty-result envelope is the expected live output — no live
tenant row data would be present in recordings anyway.

**HS-029 holdout gate:** The story-level holdout gate (BLOCKING per story spec) was satisfied
by the holdout-evaluator prior to demo recording. The 3 hidden HS-029 scenarios covering
claroty_organization_acl_policies were evaluated against the built binary. This demo recording
phase follows holdout gate passage per the VSDD per-story pipeline SOP.

**Live test status:** RG-007 (`test_BC_2_16_022_claroty_org_acl_policies_live_wire_shape_class_uid_and_metadata_uid`)
and RG-010 (`test_BC_2_16_022_claroty_org_acl_policies_live_unbounded_select_no_pagination`) are
marked `#[ignore]` per SID-1 (require `CLAROTY_INSTANCE_URL` env var pointing to monroe). They are
not recorded here; their live behavior (class_uid=3004, metadata_uid UUID, no count column,
single-fetch success) is covered by the holdout gate passage plus the mock-based RG-012 wire-shape
gate that exercises the production `SpecDrivenSensorAdapter::fetch` path.

**Demo evidence scope:** Product schema artifacts (TOML column schema, E-QUERY-038 error messages)
and VHS recordings of `cargo nextest run` test suites using synthetic mock data only.

---

## Coverage Summary

All 11 acceptance criteria covered. AC-007 and AC-010 (live `#[ignore]` tests) are noted as
live-validated at holdout gate time; their non-live counterparts (RG-012 wire-shape and the
pagination-none unit test) are recorded. All 13 non-live Red Gate tests: PASS.

| AC | Red Gate(s) | Evidence Artifact | Evidence Type | Status |
|----|-------------|-------------------|---------------|--------|
| AC-001 | RG-001 | AC-001-002-003-004-toml-parse (.tape/.gif/.webm) + schema-describe.txt | VHS (spec-engine unit) + schema transcript | PASS |
| AC-002 | RG-002 | AC-001-002-003-004-toml-parse (.tape/.gif/.webm) | VHS (spec-engine unit — pagination-none no offset/limit) | PASS |
| AC-003 | RG-003, RG-013 | AC-001-002-003-004-toml-parse (.tape/.gif/.webm) | VHS (spec-engine unit — body_template policy_acl_syntax + filter_by) | PASS |
| AC-004 | RG-004 | AC-001-002-003-004-toml-parse (.tape/.gif/.webm) + schema-describe.txt | VHS (spec-engine unit — Tier-1/Tier-2 classification) + schema transcript | PASS |
| AC-005 | RG-005 | AC-005-006-error-paths (.tape/.gif/.webm) + error-paths.txt | VHS (prism-query E2E plan-time) + error transcript | PASS |
| AC-006 | RG-006 | AC-005-006-error-paths (.tape/.gif/.webm) + error-paths.txt | VHS (prism-query E2E plan-time WIRE-SHAPE) + error transcript | PASS |
| AC-007 | RG-007 (#[ignore]) | HS-029 holdout gate passage | Live-only test; validated at holdout gate (no live tenant data committed) | PASS (holdout) |
| AC-008 | RG-008, RG-012, sub-case | AC-008-009-011-unit-tests (.tape/.gif/.webm) | VHS (prism-bin mock + wire-shape production path) | PASS |
| AC-009 | RG-009 | AC-008-009-011-unit-tests (.tape/.gif/.webm) | VHS (prism-bin unit — REQUIRED absent→null row) | PASS |
| AC-010 | RG-010 (#[ignore]) | HS-029 holdout gate passage + RG-002 unit covers non-live path | Live-only test; holdout-validated; RG-002 gates pagination-none unit path | PASS (holdout + RG-002) |
| AC-011 | RG-011 | AC-008-009-011-unit-tests (.tape/.gif/.webm) | VHS (prism-bin unit — datetime null passthrough ADR-028 §D8-B) | PASS |

---

## Schema Transcript Evidence (product schema only)

`AC-001-004-schema-describe.txt` — product schema only (column names, types, OCSF field mappings
from `claroty.sensor.toml`), no tenant row data.

**What it proves:**

- `claroty_organization_acl_policies` is registered with `ocsf_class = "entity_management"`
  (class_uid 3004; existing arm in `class_selector.rs`).

- Table exposes 7 top-level Arrow columns:
  - `metadata_uid` (string, nullable=false) — Tier-1 REQUIRED, OCSF rename of `policy_id` via `metadata.uid`
  - `name` (string) — Tier-1, OCSF rename of `policy_name` via `name`
  - `actor_user_name` (string) — Tier-1, OCSF rename of `policy_updated_by` via `actor.user.name`
  - `comment` (string) — Tier-1, OCSF rename of `policy_notes` via `comment`
  - `raw_extensions` (json) — Tier-2 aggregate: policy_source, policy_acl_type, policy_acl,
    applied_models (json, NOT string), matching_devices, policy_creation_date, policy_last_updated
  - `class_uid` (integer, nullable=false)
  - `_sensor` (string, nullable=false)

- Pagination: `type = "none"` — non-paginated single-fetch; the only Claroty table without
  offset/limit. `PaginationConfig::None` confirmed via RG-002.

- Body template mandatory fields: `"policy_acl_syntax": "Cisco dACL"` (RG-003 PASS) and
  `"filter_by": {"field": "policy_id", "operation": "is_not_null"}` (RG-013 PASS, EC-016-022-011).

---

## Error-Paths Transcript Evidence

`AC-005-006-error-paths.txt` — product error messages only (E-QUERY-038 text, available_columns
lists). These are product-defined behaviors, not tenant data.

**What it proves:**

- `SELECT policy_source FROM claroty.claroty_organization_acl_policies LIMIT 1` → E-QUERY-038:
  ```
  column 'policy_source' not found in table 'claroty_organization_acl_policies';
  available: [_sensor, actor_user_name, class_uid, comment, metadata_uid, name, raw_extensions]
  ```
  Tier-2 column rejected; raw_extensions in available_columns; metadata_uid present (not policy_id).

- `SELECT policy_id FROM claroty.claroty_organization_acl_policies LIMIT 1` → E-QUERY-038:
  ```
  column 'policy_id' not found in table 'claroty_organization_acl_policies';
  available: [_sensor, actor_user_name, class_uid, comment, metadata_uid, name, raw_extensions]
  ```
  Raw TOML Tier-1 name 'policy_id' rejected; Arrow name 'metadata_uid' is in available_columns.
  Asymmetric rename: `policy_id` → `metadata.uid` → Arrow `metadata_uid`.

---

## VHS Recording Files

All VHS recordings run tests against the story worktree at
`/Users/jmagady/Dev/prism/.worktrees/S-CLAROTY-ACLPOLICY-001/`
using `cargo nextest`. Compilation artifacts are pre-warmed; actual test execution: < 1 second.

### AC-001-002-003-004-toml-parse

**Covers:** AC-001 (RG-001), AC-002 (RG-002), AC-003 (RG-003 + RG-013), AC-004 (RG-004)

**Command:** `cargo nextest run -p prism-spec-engine -E 'test(BC_2_16_022)' 2>&1 | tail -12`

**What it proves:**

- **AC-001 / RG-001:** `test_BC_2_16_022_claroty_org_acl_policies_toml_block_parses` PASS.
  `SpecLoader::parse` on `claroty.sensor.toml` returns `Ok(SensorSpec)`. 11 `ColumnSpec`
  entries for `claroty_organization_acl_policies`. `PaginationConfig::None` (not OffsetLimit).
  `response_path = "$.organization_acl_policies"` (no count field). Traces to BC-2.16.022 §PC1.

- **AC-002 / RG-002:** `test_BC_2_16_022_claroty_org_acl_policies_pagination_none_no_offset_limit`
  PASS. `pipeline.rs::build_request` for `PaginationConfig::None` builds POST body from
  `body_template` only — serialized body does NOT contain `"offset"` or `"limit"` keys.
  Traces to BC-2.16.022 §PC4 (Pagination-None Contract).

- **AC-003 / RG-003:** `test_BC_2_16_022_claroty_org_acl_policies_body_template_has_policy_acl_syntax`
  PASS. Deserializing `body_template` JSON confirms key `"policy_acl_syntax"` is present with
  value `"Cisco dACL"` (exact string, case-sensitive). Traces to BC-2.16.022 §PC1 §Invariants.

- **AC-003 / RG-013:** `test_BC_2_16_022_claroty_org_acl_policies_body_template_has_filter_by_policy_id_is_not_null`
  PASS. `body_template` contains `"filter_by"` with `field = "policy_id"` and
  `operation = "is_not_null"`. Gates EC-016-022-011: live API cross-field selector validator
  requires at least one of policy_id/policy_name/filter_by; omitting all three causes HTTP 422.
  Traces to BC-2.16.022 §Invariants, EC-016-022-011.

- **AC-004 / RG-004:** `test_BC_2_16_022_claroty_org_acl_policies_tier1_four_tier2_seven_correct_types`
  PASS. Exactly 4 columns have non-None `ocsf_field`: `policy_id→metadata.uid` (REQUIRED),
  `policy_name→name`, `policy_updated_by→actor.user.name`, `policy_notes→comment`. Exactly 7
  columns have `ocsf_field == None` (Tier-2 → aggregate into raw_extensions). `policy_id` carries
  `options = ["REQUIRED"]`. Traces to BC-2.16.022 §PC2.

**Test result:** 5/5 PASS (`prism-spec-engine::bc_2_16_022_test`)

### AC-005-006-error-paths

**Covers:** AC-005 (RG-005), AC-006 (RG-006)

**Command:** `cargo nextest run -p prism-query -E 'test(BC_2_16_022)' 2>&1 | tail -10`

**What it proves:**

- **AC-005 / RG-005:** `test_BC_2_16_022_claroty_org_acl_policies_policy_source_tier2_e_query_038`
  PASS (authoritative `prism-query` E2E via `QueryEngine::execute`). `SELECT policy_source`
  raises E-QUERY-038; `available_columns` set contains `raw_extensions`, `metadata_uid`, `name`,
  `actor_user_name`, `comment`, `class_uid`, `_sensor` but NOT `policy_source`.
  Traces to BC-2.16.022 §Invariants, EC-016-022-003.

- **AC-006 / RG-006:** `test_BC_2_16_022_claroty_org_acl_policies_policy_id_raw_name_not_projected_metadata_uid_is`
  PASS (authoritative `prism-query` E2E). `SELECT policy_id` raises E-QUERY-038; `available_columns`
  contains `metadata_uid` (the Arrow form of `policy_id→metadata.uid`) but NOT `policy_id`.
  Asymmetric rename confirmed: `policy_id` ≠ `metadata_uid` (not simple underscore substitution).
  Traces to BC-2.16.022 §Invariants, EC-016-022-004, TV-BC-2.16.022-003.

**Test result:** 2/2 PASS (`prism-query::bc_2_16_022_test`)

### AC-008-009-011-unit-tests

**Covers:** AC-008 (RG-008, RG-012, sub-case), AC-009 (RG-009), AC-011 (RG-011)

**Command:** `cargo nextest run -p prism-bin -E 'test(BC_2_16_022)' 2>&1 | tail -12`

**What it proves:**

- **AC-008 / RG-008:** `test_BC_2_16_022_applied_models_raw_extensions_json_array_not_string`
  PASS. Mock response with `applied_models: ["Siemens SIMATIC S7", "Rockwell"]` — deserialized
  `raw_extensions["applied_models"]` is a JSON array (not a string token `"[...]"`).
  Traces to BC-2.16.022 §PC5, EC-016-022-006.

- **AC-008 sub-case:** `test_BC_2_16_022_applied_models_empty_array_wire_shape` PASS. Empty
  `applied_models: []` serializes as JSON empty array `[]` not null (EC-016-022-005). Distinct
  from RG-008 (empty vs non-empty array sub-case). Traces to BC-2.16.022 §PC5, EC-016-022-005.

- **AC-008 / RG-012:** `test_BC_2_16_022_claroty_org_acl_policies_wire_shape_applied_models_json_array`
  PASS. Wire-shape gate via `SpecDrivenSensorAdapter::fetch` production path (authoritative per
  no-DTU ACL policies path, D-2200). Serialized JSON output confirms `applied_models` key is
  JSON-typed array in `raw_extensions`, NOT a standalone root key. SAP-2 N/A per D-2200.
  Traces to BC-2.16.022 §PC5, §Invariants (wire-shape discipline 2026-07-13).

- **AC-009 / RG-009:** `test_BC_2_16_022_null_metadata_uid_when_policy_id_absent` PASS.
  Row missing `policy_id` (REQUIRED field) produces a null row (null `metadata_uid` Arrow cell).
  No hard error raised. Second row with valid `policy_id` produces non-null `metadata_uid`.
  Traces to BC-2.16.022 §Invariants, EC-016-022-001.

- **AC-011 / RG-011:** `test_BC_2_16_022_datetime_fields_null_passthrough_in_raw_extensions`
  PASS. `policy_creation_date` and `policy_last_updated` declared `column_type = "datetime"` with
  no `timestamp_formats` key (ADR-028 §D8-B implicit iso8601 default). Row with valid ISO-8601
  datetimes → non-null Arrow cells. Row with null/absent datetime fields → null cells, no
  E-SPEC-018 raised (EC-016-022-010 null-passthrough pattern).
  Traces to BC-2.16.022 §Invariants, ADR-028 §D8-B.

- **SAP-2 N/A:** `test_BC_2_16_022_claroty_acl_policies_wire_shape_sap2_na_documented` PASS.
  Documents that SAP-2 DTU-parity probe is not applicable for `claroty_organization_acl_policies`
  per BC-2.16.022 §PC6 and D-2200 governing decision (no DTU exists for this table; DTU creation
  is a separate deferred story). Confirms test does NOT check
  `crates/prism-dtu-claroty/src/routes/` — no ACL policy route exists there and absence is expected.

**Test result:** 6/6 PASS (`prism-bin::bc_2_16_022_claroty_acl_policies_wire_shape` +
`prism-bin::spec_driven_adapter::tests`)

---

## BC Traceability

| Evidence Artifact | AC(s) | BC | EC |
|-------------------|----|----|----|
| AC-001-004-schema-describe.txt | AC-001, AC-004 | BC-2.16.022 §PC1, §PC2 | — |
| AC-001-002-003-004-toml-parse (.gif/.webm) | AC-001 (RG-001) | BC-2.16.022 §PC1 | — |
| AC-001-002-003-004-toml-parse (.gif/.webm) | AC-002 (RG-002) | BC-2.16.022 §PC4 | EC-016-022-007 |
| AC-001-002-003-004-toml-parse (.gif/.webm) | AC-003 (RG-003) | BC-2.16.022 §PC1 §Invariants | EC-016-022-002 |
| AC-001-002-003-004-toml-parse (.gif/.webm) | AC-003 (RG-013) | BC-2.16.022 §Invariants | EC-016-022-011 |
| AC-001-002-003-004-toml-parse (.gif/.webm) | AC-004 (RG-004) | BC-2.16.022 §PC2 | — |
| AC-005-006-error-paths.txt | AC-005 | BC-2.16.022 §Invariants | EC-016-022-003 |
| AC-005-006-error-paths.txt | AC-006 | BC-2.16.022 §Invariants | EC-016-022-004 |
| AC-005-006-error-paths (.gif/.webm) | AC-005 (RG-005) | BC-2.16.022 §Invariants | EC-016-022-003 |
| AC-005-006-error-paths (.gif/.webm) | AC-006 (RG-006) | BC-2.16.022 §Invariants | EC-016-022-004, TV-BC-2.16.022-003 |
| AC-008-009-011-unit-tests (.gif/.webm) | AC-008 (RG-008) | BC-2.16.022 §PC5 | EC-016-022-006 |
| AC-008-009-011-unit-tests (.gif/.webm) | AC-008 sub-case | BC-2.16.022 §PC5 | EC-016-022-005 |
| AC-008-009-011-unit-tests (.gif/.webm) | AC-008 (RG-012) | BC-2.16.022 §PC5, §Invariants | wire-shape discipline 2026-07-13 |
| AC-008-009-011-unit-tests (.gif/.webm) | AC-009 (RG-009) | BC-2.16.022 §Invariants | EC-016-022-001 |
| AC-008-009-011-unit-tests (.gif/.webm) | AC-011 (RG-011) | BC-2.16.022 §Invariants | EC-016-022-010, ADR-028 §D8-B |
| HS-029 holdout gate passage | AC-007, AC-010 | BC-2.16.022 §PC1/§PC2/§PC4/§PC5 | TV-BC-2.16.022-001, TV-BC-2.16.022-006 |
