# Demo Evidence Report — S-CLAROTY-ORGPOLICY-001

**Story:** Claroty xDome Org Policy Tables — 4 TOML `[[tables]]` blocks (claroty_organization_zones, claroty_organization_zone_policies, claroty_organization_firewall_groups, claroty_organization_firewall_policies), entity_management/3004, 8 Json cols, fw URL↔envelope-key asymmetry, live structural tests (Wave C G5)
**Story version:** v1.6
**Evidence date:** 2026-09-01
**Recorder:** demo-recorder
**Product type:** CLI (Rust workspace) + live MCP (prism-live, monroe client)
**Recording tools:** VHS 0.11.0 (terminal session recordings) + annotated product-schema MCP transcript

---

## Live Validation Status

Live-tenant validation was performed against the monroe client using the deployed prism binary at
`/Users/jmagady/Dev/test-soc/bin/prism` (SHA-256 e7acb234...) with the G5 spec at
`/Users/jmagady/Dev/test-soc/.prism-live/specs/claroty.sensor.toml`. All 4 org-policy tables
returned live data confirming correct behavior (class_uid=3004, Tier-1 Arrow fields, raw_extensions
as JSON-serialized string, activity_name="Deny"/"Allow", fw URL/envelope asymmetry correct).
**HS-028 holdout gate: PASSED** prior to demo recording per the story-level holdout gate SOP.

Per human-directed policy (2026-09-01): live-tenant query result rows (real tenant data) are
NOT committed to the repository. The schema transcript (`AC-001-002-009-010-015-016-021-022-schema-describe.txt`)
contains product schema only (table/column names/types from prism_describe, not tenant rows).
The error-paths transcript (`AC-003-004-012-020-024-error-paths.txt`) contains product error messages
only (E-QUERY-038 error text, available_columns lists — these are product-defined, not tenant data).

**raw_extensions wire-shape note:** `raw_extensions` is emitted as a JSON-serialized STRING on the
wire (Arrow Utf8 type), not a native JSON object. This is the current observed behavior per ADR-058 §I2.
Transcript files describe what is actually emitted (a string that parses to an object). Tests assert
on the serialized string format. No claims are made about the intended final form.

---

## Coverage Summary

All 26 acceptance criteria covered across 4 tables (zones: AC-001..AC-008; zone_policies: AC-009..AC-014;
fw_groups: AC-015..AC-020; fw_policies: AC-021..AC-026). AC-005, AC-011, AC-017, AC-018, AC-023
(live wire shape and fw asymmetry) are validated by mock-based wire shape tests (prism-bin) plus
live validation that confirmed all behaviors match spec. Live-tenant query result rows are not
committed per policy — only the PASS outcome is recorded here.

| AC | Red Gate(s) | Evidence Artifact | Evidence Type | Status |
|----|-------------|-------------------|---------------|--------|
| AC-001 | RG-001 | AC-001-002-009-010-015-016-021-022-toml-parse (.tape/.gif/.webm) + schema-describe.txt | VHS (test suite) + schema transcript | PASS |
| AC-002 | RG-002 | AC-001-002-009-010-015-016-021-022-toml-parse (.tape/.gif/.webm) + schema-describe.txt | VHS (test suite) + schema transcript | PASS |
| AC-003 | RG-003 | AC-003-004-012-020-024-plan-gate-tests (.tape/.gif/.webm) + error-paths.txt | VHS (prism-bin E2E) + live error transcript | PASS |
| AC-004 | RG-004 | AC-003-004-012-020-024-plan-gate-tests (.tape/.gif/.webm) + error-paths.txt | VHS (plan-time) + live error transcript | PASS |
| AC-005 | RG-005 wire | AC-005-011-018-023-wire-shape-mock (.tape/.gif/.webm) | VHS (mock path); live validation PASS (not committed) | PASS |
| AC-006 | RG-006 | AC-006-007-008-013-014-019-025-026-unit-tests (.tape/.gif/.webm) | VHS (unit mock) | PASS |
| AC-007 | RG-007 | AC-006-007-008-013-014-019-025-026-unit-tests (.tape/.gif/.webm) | VHS (unit mock) | PASS |
| AC-008 | RG-008 | AC-006-007-008-013-014-019-025-026-unit-tests (.tape/.gif/.webm) | VHS (unit mock) | PASS |
| AC-009 | RG-009 | AC-001-002-009-010-015-016-021-022-toml-parse (.tape/.gif/.webm) + schema-describe.txt | VHS (test suite) + schema transcript | PASS |
| AC-010 | RG-010 | AC-001-002-009-010-015-016-021-022-toml-parse (.tape/.gif/.webm) + schema-describe.txt | VHS (test suite) + schema transcript | PASS |
| AC-011 | RG-011 wire | AC-005-011-018-023-wire-shape-mock (.tape/.gif/.webm) | VHS (mock path); live validation PASS (not committed) | PASS |
| AC-012 | RG-012 | AC-003-004-012-020-024-plan-gate-tests (.tape/.gif/.webm) + error-paths.txt | VHS (prism-bin E2E) + live error transcript | PASS |
| AC-013 | RG-013 | AC-006-007-008-013-014-019-025-026-unit-tests (.tape/.gif/.webm) | VHS (unit mock) | PASS |
| AC-014 | RG-014 | AC-006-007-008-013-014-019-025-026-unit-tests (.tape/.gif/.webm) | VHS (unit mock) | PASS |
| AC-015 | RG-015 | AC-001-002-009-010-015-016-021-022-toml-parse (.tape/.gif/.webm) + schema-describe.txt | VHS (test suite) + schema transcript | PASS |
| AC-016 | RG-016 | AC-001-002-009-010-015-016-021-022-toml-parse (.tape/.gif/.webm) + schema-describe.txt | VHS (test suite) + schema transcript | PASS |
| AC-017 | (live #[ignore]) | AC-003-004-012-020-024-error-paths.txt (fw asymmetry section) | Live error-paths transcript (non-empty result note; no row data) | PASS |
| AC-018 | RG-018 wire | AC-005-011-018-023-wire-shape-mock (.tape/.gif/.webm) | VHS (mock path); live validation PASS (not committed) | PASS |
| AC-019 | RG-019 | AC-006-007-008-013-014-019-025-026-unit-tests (.tape/.gif/.webm) | VHS (unit mock) | PASS |
| AC-020 | RG-020 | AC-003-004-012-020-024-plan-gate-tests (.tape/.gif/.webm) + error-paths.txt | VHS (prism-bin E2E) + live error transcript | PASS |
| AC-021 | RG-021 | AC-001-002-009-010-015-016-021-022-toml-parse (.tape/.gif/.webm) + schema-describe.txt | VHS (test suite) + schema transcript | PASS |
| AC-022 | RG-022 | AC-001-002-009-010-015-016-021-022-toml-parse (.tape/.gif/.webm) + schema-describe.txt | VHS (test suite) + schema transcript | PASS |
| AC-023 | RG-023 wire | AC-005-011-018-023-wire-shape-mock (.tape/.gif/.webm) | VHS (mock path); live validation PASS (not committed) | PASS |
| AC-024 | RG-024 | AC-003-004-012-020-024-plan-gate-tests (.tape/.gif/.webm) + error-paths.txt | VHS (prism-bin E2E) + live error transcript | PASS |
| AC-025 | RG-025 | AC-006-007-008-013-014-019-025-026-unit-tests (.tape/.gif/.webm) | VHS (unit mock) | PASS |
| AC-026 | RG-026 | AC-006-007-008-013-014-019-025-026-unit-tests (.tape/.gif/.webm) | VHS (unit mock) | PASS |

---

## Schema Transcript Evidence (prism_describe, client: monroe)

`AC-001-002-009-010-015-016-021-022-schema-describe.txt` — product schema only (column names/types),
no tenant row data.

**What it proves:**

- All 4 G5 tables registered in monroe schema with `description=entity_management` (OCSF class,
  maps to class_uid=3004 per the existing `entity_management` arm in `class_selector.rs`).
  13 total tables confirmed for monroe (4 G5 + 9 pre-existing).

- `claroty_organization_zones` exposes 7 Arrow columns:
  - `name` (string, description="name") — Tier-1, OCSF rename of `zone_name`, REQUIRED
  - `comment` (string, description="comment") — Tier-1, OCSF rename of `zone_description`
  - `status_code` (boolean, description="status_code") — Tier-1, OCSF rename of `enabled`
  - `actor_user_name` (string, description="actor.user.name") — Tier-1, OCSF rename of `updated_by`
  - `raw_extensions` (json) — Tier-2 aggregate: zone_source, priority, device_conditions,
    attributed_devices, exportable_attributed_devices, created_time, last_update (7 Tier-2)
  - `class_uid` (integer, nullable=false)
  - `_sensor` (string, nullable=false)

- `claroty_organization_zone_policies` exposes 7 Arrow columns:
  - `name` (string, description="name") — Tier-1, OCSF rename of `policy_name`, REQUIRED
  - `activity_name` (string, description="activity_name") — Tier-1, OCSF rename of `policy_action`
  - `comment` (string, description="comment") — Tier-1, OCSF rename of `policy_notes`
  - `actor_user_name` (string, description="actor.user.name") — Tier-1
  - `raw_extensions` (json) — Tier-2 aggregate: policy_source, communication_conditions,
    matching_devices, should_generate_alerts, alert_use_case, related_alerts_ids, applied_zone_pairs,
    created_time, last_updated (9 Tier-2 incl. 3 Json; `last_updated` with 'd' confirmed)
  - `class_uid` (integer, nullable=false)
  - `_sensor` (string, nullable=false)

- `claroty_organization_firewall_groups` exposes 7 Arrow columns:
  - Same Tier-1 structure as zones (name, comment, status_code, actor_user_name)
  - `raw_extensions` (json) — Tier-2 aggregate: firewall_group_source, priority, device_conditions,
    attributed_devices, exportable_attributed_devices, created_time, last_update (7 Tier-2)
  - `class_uid`, `_sensor`

- `claroty_organization_firewall_policies` exposes 7 Arrow columns:
  - Same Tier-1 structure as zone_policies (name, activity_name, comment, actor_user_name)
  - `raw_extensions` (json) — Tier-2 aggregate: policy_source, communication_conditions,
    matching_devices, should_generate_alerts, alert_use_case, related_alerts_ids, **applied_group_pairs**
    (NOT applied_zone_pairs — BC-2.16.021 §Invariants EC-016-021-010), created_time, last_updated (9 Tier-2)
  - `class_uid`, `_sensor`

---

## Error-Paths Transcript Evidence (E-QUERY-038, client: monroe)

`AC-003-004-012-020-024-error-paths.txt` — product error messages only (not tenant data).

**What it proves:**

- `SELECT zone_source FROM claroty_organization_zones LIMIT 1` → E-QUERY-038:
  ```
  column 'zone_source' not found in table 'claroty_organization_zones' for client 'monroe';
  available: [_sensor, actor_user_name, class_uid, comment, name, raw_extensions, status_code]
  ```
  Tier-2 column rejected; raw_extensions in available_columns.

- `SELECT zone_name FROM claroty_organization_zones LIMIT 1` → E-QUERY-038:
  ```
  column 'zone_name' not found; available: [_sensor, actor_user_name, class_uid, comment, name, raw_extensions, status_code]
  ```
  Raw TOML Tier-1 name rejected; Arrow name `name` is in available_columns.

- `SELECT applied_zone_pairs FROM claroty_organization_zone_policies LIMIT 1` → E-QUERY-038:
  ```
  column 'applied_zone_pairs' not found; available: [_sensor, activity_name, actor_user_name, class_uid, comment, name, raw_extensions]
  ```
  Json Tier-2 column rejected; raw_extensions present in available_columns.

- `SELECT firewall_group_source FROM claroty_organization_firewall_groups LIMIT 1` → E-QUERY-038:
  Tier-2 column rejected; raw_extensions in available_columns.

- `SELECT applied_group_pairs FROM claroty_organization_firewall_policies LIMIT 1` → E-QUERY-038:
  ```
  column 'applied_group_pairs' not found; available: [_sensor, activity_name, actor_user_name, class_uid, comment, name, raw_extensions]
  ```
  Json Tier-2 column rejected; confirms applied_group_pairs (NOT applied_zone_pairs) is Tier-2
  in firewall_policies.

- `SELECT name FROM claroty_organization_firewall_groups LIMIT 5` → 1 row returned, non-empty:
  Confirms response_path=$.organization_firewall_groups correctly extracts from the xDome response
  envelope despite the URL using abbreviated /organization_fw_groups/ path. URL↔envelope-key
  asymmetry (AC-017) is working correctly.

---

## VHS Recording Files

All VHS recordings run tests against the story worktree at
`/Users/jmagady/Dev/prism/.worktrees/S-CLAROTY-ORGPOLICY-001/`
using `cargo nextest`. Compilation artifacts are pre-warmed; expected runtime per tape: 30-90s.

### AC-001-002-009-010-015-016-021-022-toml-parse

**Covers:** AC-001 (RG-001), AC-002 (RG-002), AC-009 (RG-009), AC-010 (RG-010),
AC-015 (RG-015), AC-016 (RG-016), AC-021 (RG-021), AC-022 (RG-022)

**What it proves:**

- **AC-001 / RG-001:** `test_BC_2_16_020_claroty_organization_zones_toml_block_parses` PASS.
  `SpecLoader::parse` on `claroty.sensor.toml` returns `Ok(SensorSpec)`. 11 `ColumnSpec` entries
  for `claroty_organization_zones`. Pagination `offset_limit` / page_size 1000. Traces to
  BC-2.16.020 §PC1.

- **AC-002 / RG-002:** `test_BC_2_16_020_claroty_organization_zones_tier1_columns_four_with_ocsf_field`
  PASS. Exactly 4 columns have `ocsf_field == Some(_)`: zone_name→`"name"` (REQUIRED),
  zone_description→`"comment"`, enabled→`"status_code"`, updated_by→`"actor.user.name"`.
  Exactly 7 columns have `ocsf_field == None` (Tier-2). Traces to BC-2.16.020 §PC3.

- **AC-009 / RG-009:** `test_BC_2_16_020_claroty_organization_zone_policies_toml_block_parses` PASS.
  13 `ColumnSpec` entries for `claroty_organization_zone_policies`.
  `body_template` contains `"last_updated"` (with 'd', not `"last_update"`).
  Traces to BC-2.16.020 §PC2.

- **AC-010 / RG-010:** `test_BC_2_16_020_claroty_organization_zone_policies_tier1_columns_four_with_ocsf_field`
  PASS. Exactly 4 Tier-1 columns: policy_name→`"name"` (REQUIRED), policy_action→`"activity_name"`,
  policy_notes→`"comment"`, updated_by→`"actor.user.name"`. Exactly 9 Tier-2 (incl. 3 Json).
  Traces to BC-2.16.020 §PC4.

- **AC-015 / RG-015:** `test_BC_2_16_021_claroty_organization_firewall_groups_toml_block_parses` PASS.
  11 `ColumnSpec` entries. `path_template = "/api/v1/organization_fw_groups/"` (abbreviated URL).
  `response_path = "$.organization_firewall_groups"` (full spelling — NOT `$.organization_fw_groups`).
  Both strings present in same TOML block confirming the URL↔envelope asymmetry invariant.
  Traces to BC-2.16.021 §PC1.

- **AC-016 / RG-016:** `test_BC_2_16_021_claroty_organization_firewall_groups_tier1_columns_four_with_ocsf_field`
  PASS. Exactly 4 Tier-1 columns: firewall_group_name→`"name"` (REQUIRED),
  firewall_group_description→`"comment"`, enabled→`"status_code"`, updated_by→`"actor.user.name"`.
  Exactly 7 Tier-2. Traces to BC-2.16.021 §PC3.

- **AC-021 / RG-021:** `test_BC_2_16_021_claroty_organization_firewall_policies_toml_block_parses` PASS.
  13 `ColumnSpec` entries. `path_template = "/api/v1/organization_fw_group_policies/"` (abbreviated).
  `response_path = "$.organization_firewall_policies"` (full spelling). Same URL↔envelope asymmetry.
  Traces to BC-2.16.021 §PC2.

- **AC-022 / RG-022:** `test_BC_2_16_021_claroty_organization_firewall_policies_tier1_columns_four_with_ocsf_field`
  PASS. Exactly 4 Tier-1: policy_name→`"name"` (REQUIRED), policy_action→`"activity_name"`,
  policy_notes→`"comment"`, updated_by→`"actor.user.name"`. Exactly 9 Tier-2 (incl. applied_group_pairs
  NOT applied_zone_pairs). Traces to BC-2.16.021 §PC4.

### AC-003-004-012-020-024-plan-gate-tests

**Covers:** AC-003 (RG-003), AC-004 (RG-004), AC-012 (RG-012), AC-020 (RG-020), AC-024 (RG-024)

**What it proves:**

- **AC-003 / RG-003:** `test_BC_2_16_020_claroty_organization_zones_e2e_e_query_038_tier2_column`
  PASS (authoritative prism-bin E2E gate via `QueryEngine::execute`). E-QUERY-038 raised for
  Tier-2 column query; `available_columns` excludes Tier-2 name; includes `raw_extensions`,
  `name`, `comment`, `status_code`, `actor_user_name`, `class_uid`, `_sensor`.
  Traces to BC-2.16.020 §Invariants, EC-016-020-005.

- **AC-004 / RG-004:** `test_BC_2_16_020_claroty_organization_zones_tier1_raw_toml_name_raises_e_query_038`
  PASS. `SELECT zone_name` raises E-QUERY-038; available has `name` not `zone_name`.
  Traces to BC-2.16.020 §Invariants, TV-BC-2.16.020-003.

- **AC-012 / RG-012:** `test_BC_2_16_020_claroty_organization_zone_policies_e2e_e_query_038_tier2_column`
  PASS (authoritative prism-bin E2E). E-QUERY-038 for `SELECT applied_zone_pairs`;
  `available_columns` has `raw_extensions`, `name`, `activity_name`, `comment`, `actor_user_name`
  but NOT `applied_zone_pairs`. Traces to BC-2.16.020 §Invariants, EC-016-020-006.

- **AC-020 / RG-020:** `test_BC_2_16_021_claroty_organization_firewall_groups_e2e_e_query_038_tier2_column`
  PASS (authoritative prism-bin E2E). E-QUERY-038 for `SELECT firewall_group_source`;
  `available_columns` includes `raw_extensions` not `firewall_group_source`.
  Traces to BC-2.16.021 §Invariants.

- **AC-024 / RG-024:** `test_BC_2_16_021_claroty_organization_firewall_policies_e2e_e_query_038_tier2_column`
  PASS (authoritative prism-bin E2E). E-QUERY-038 for `SELECT applied_group_pairs`;
  confirms `applied_group_pairs` is Tier-2 (not a standalone Arrow column) and that the TOML
  declares `applied_group_pairs` (not `applied_zone_pairs`).
  Traces to BC-2.16.021 §Invariants, EC-016-021-007,010.

### AC-005-011-018-023-wire-shape-mock

**Covers:** AC-005 (RG-005 wire), AC-011 (RG-011 wire), AC-018 (RG-018 wire), AC-023 (RG-023 wire)

**What it proves:**

- **AC-005 / RG-005:** `test_BC_2_16_020_claroty_organization_zones_wire_shape_class_uid_3004_mock`
  PASS (SAP-4 production path via mock). Serialized JSON wire output contains `class_uid=3004`,
  `name` present, `comment` present, `status_code` present, `actor_user_name` present,
  `raw_extensions` present (JSON-serialized string containing Tier-2 keys). No standalone
  Tier-2 root keys. Traces to BC-2.16.020 §PC1 (class_uid), §PC3.
  Also: `test_BC_2_16_020_claroty_organization_zones_wire_shape_serialized_json_null_not_absent` PASS.
  Null columns present as explicit null values, not absent keys (wire-shape discipline 2026-07-13).

- **AC-011 / RG-011:** `test_BC_2_16_020_claroty_organization_zone_policies_wire_shape_class_uid_3004_mock`
  PASS. class_uid=3004, name, activity_name present; raw_extensions present (JSON-serialized string)
  containing communication_conditions, related_alerts_ids, applied_zone_pairs keys.
  Traces to BC-2.16.020 §PC2, §PC4, §PC6.

- **AC-018 / RG-018:** `test_BC_2_16_021_claroty_organization_firewall_groups_wire_shape_class_uid_3004_mock`
  PASS. class_uid=3004, name present; raw_extensions present (JSON-serialized string) containing
  device_conditions key (Json column). Traces to BC-2.16.021 §PC1, §PC3, §PC6.

- **AC-023 / RG-023:** `test_BC_2_16_021_claroty_organization_firewall_policies_wire_shape_class_uid_3004_mock`
  PASS. class_uid=3004, name, activity_name present; raw_extensions present (JSON-serialized string)
  containing applied_group_pairs key (NOT applied_zone_pairs — BC-2.16.021 §Invariants EC-016-021-010).
  Traces to BC-2.16.021 §PC2, §PC4, §PC6.

### AC-006-007-008-013-014-019-025-026-unit-tests

**Covers:** AC-006 (RG-006), AC-007 (RG-007), AC-008 (RG-008), AC-013 (RG-013),
AC-014 (RG-014), AC-019 (RG-019), AC-025 (RG-025a, RG-025b), AC-026 (RG-026)

**What it proves:**

- **AC-006 / RG-006:** `test_BC_2_16_020_claroty_organization_zones_device_conditions_json_not_string`
  PASS. `device_conditions` column_type=json; raw_extensions["device_conditions"] is a JSON array
  (not a string token). Empty array serializes as `[]` not null.
  Traces to BC-2.16.020 §PC6, spike-findings §Spike 3 §Nested-field classification principle.

- **AC-007 / RG-007:** `test_BC_2_16_020_claroty_organization_zones_required_zone_name_absent_produces_null_row`
  PASS. Row missing zone_name → null row (REQUIRED semantics); no hard error; subsequent rows
  continue. Traces to BC-2.16.020 §Invariants, EC-016-020-001.

- **AC-008 / RG-008:** `test_BC_2_16_020_claroty_organization_zones_nullable_count_uses_empty_page_halt`
  PASS. `count=null` in organization_zones envelope → empty-page halt; no null-ptr dereference.
  Traces to BC-2.16.020 §PC1 pagination note, EC-016-020-004.

- **AC-013 / RG-013:** `test_BC_2_16_020_claroty_organization_zone_policies_required_policy_name_absent_produces_null_row`
  PASS. Row missing policy_name → null row; no hard error.
  Traces to BC-2.16.020 §Invariants, EC-016-020-002.

- **AC-014 / RG-014:** `test_BC_2_16_020_claroty_organization_zone_policies_json_columns_not_stringified`
  PASS. All three Json columns (communication_conditions, related_alerts_ids, applied_zone_pairs)
  serialize as JSON arrays in raw_extensions, not string-encoded tokens.
  Traces to BC-2.16.020 §PC6, spike-findings §Spike 3 §Table B.

- **AC-019 / RG-019:** `test_BC_2_16_021_claroty_organization_firewall_groups_required_fwgroupname_absent_produces_null_row`
  PASS. Row missing firewall_group_name → null row; no hard error.
  Traces to BC-2.16.021 §Invariants, EC-016-021-001.

- **AC-025 / RG-025a:** `test_BC_2_16_021_claroty_organization_firewall_policies_required_policy_name_absent_produces_null_row`
  PASS. Row missing policy_name → null row; no hard error.
  Traces to BC-2.16.021 §Invariants, EC-016-021-002.

- **AC-025 / RG-025b:** `test_BC_2_16_021_claroty_organization_firewall_policies_nullable_count_uses_empty_page_halt`
  PASS. `count=null` in fw_policies/fw_groups envelope → empty-page halt; no error.
  Traces to BC-2.16.021 §PC2 pagination, EC-016-021-004.

- **AC-026 / RG-026:** `test_BC_2_16_021_claroty_organization_firewall_policies_json_columns_not_stringified`
  PASS. All three Json columns (communication_conditions, related_alerts_ids, applied_group_pairs)
  serialize as JSON arrays in raw_extensions. `applied_group_pairs` key is present (NOT
  `applied_zone_pairs` — confirms distinct column name per BC-2.16.021 §Invariants EC-016-021-010).
  Traces to BC-2.16.021 §PC6, spike-findings §Spike 3 §Table D.

---

## BC Traceability

| Evidence Artifact | AC(s) | BC | EC |
|-------------------|----|----|----|
| AC-001-002-009-010-015-016-021-022-schema-describe.txt | AC-001, AC-002 | BC-2.16.020 §PC1, §PC3 | — |
| AC-001-002-009-010-015-016-021-022-schema-describe.txt | AC-009, AC-010 | BC-2.16.020 §PC2, §PC4 | — |
| AC-001-002-009-010-015-016-021-022-schema-describe.txt | AC-015, AC-016 | BC-2.16.021 §PC1, §PC3 | — |
| AC-001-002-009-010-015-016-021-022-schema-describe.txt | AC-021, AC-022 | BC-2.16.021 §PC2, §PC4 | EC-016-021-010 |
| AC-001-002-009-010-015-016-021-022-toml-parse (.gif/.webm) | AC-001 (RG-001) | BC-2.16.020 §PC1 | — |
| AC-001-002-009-010-015-016-021-022-toml-parse (.gif/.webm) | AC-002 (RG-002) | BC-2.16.020 §PC3 | — |
| AC-001-002-009-010-015-016-021-022-toml-parse (.gif/.webm) | AC-009 (RG-009) | BC-2.16.020 §PC2 | EC-016-020-009 |
| AC-001-002-009-010-015-016-021-022-toml-parse (.gif/.webm) | AC-010 (RG-010) | BC-2.16.020 §PC4 | — |
| AC-001-002-009-010-015-016-021-022-toml-parse (.gif/.webm) | AC-015 (RG-015) | BC-2.16.021 §PC1 | — |
| AC-001-002-009-010-015-016-021-022-toml-parse (.gif/.webm) | AC-016 (RG-016) | BC-2.16.021 §PC3 | — |
| AC-001-002-009-010-015-016-021-022-toml-parse (.gif/.webm) | AC-021 (RG-021) | BC-2.16.021 §PC2 | — |
| AC-001-002-009-010-015-016-021-022-toml-parse (.gif/.webm) | AC-022 (RG-022) | BC-2.16.021 §PC4 | EC-016-021-010 |
| AC-003-004-012-020-024-error-paths.txt | AC-003, AC-004 | BC-2.16.020 §Invariants | EC-016-020-005 |
| AC-003-004-012-020-024-error-paths.txt | AC-012 | BC-2.16.020 §Invariants | EC-016-020-006 |
| AC-003-004-012-020-024-error-paths.txt | AC-017 | BC-2.16.021 §Invariants | EC-016-021-005,006 |
| AC-003-004-012-020-024-error-paths.txt | AC-020 | BC-2.16.021 §Invariants | EC-016-021-005 |
| AC-003-004-012-020-024-error-paths.txt | AC-024 | BC-2.16.021 §Invariants | EC-016-021-007,010 |
| AC-003-004-012-020-024-plan-gate-tests (.gif/.webm) | AC-003 (RG-003) | BC-2.16.020 §Invariants | EC-016-020-005 |
| AC-003-004-012-020-024-plan-gate-tests (.gif/.webm) | AC-004 (RG-004) | BC-2.16.020 §Invariants | TV-BC-2.16.020-003 |
| AC-003-004-012-020-024-plan-gate-tests (.gif/.webm) | AC-012 (RG-012) | BC-2.16.020 §Invariants | EC-016-020-006 |
| AC-003-004-012-020-024-plan-gate-tests (.gif/.webm) | AC-020 (RG-020) | BC-2.16.021 §Invariants | — |
| AC-003-004-012-020-024-plan-gate-tests (.gif/.webm) | AC-024 (RG-024) | BC-2.16.021 §Invariants | EC-016-021-007,010 |
| AC-005-011-018-023-wire-shape-mock (.gif/.webm) | AC-005 (RG-005) | BC-2.16.020 §PC1, §PC3 | TV-BC-2.16.020-002 |
| AC-005-011-018-023-wire-shape-mock (.gif/.webm) | AC-011 (RG-011) | BC-2.16.020 §PC2, §PC4, §PC6 | TV-BC-2.16.020-007 |
| AC-005-011-018-023-wire-shape-mock (.gif/.webm) | AC-018 (RG-018) | BC-2.16.021 §PC1, §PC3, §PC6 | TV-BC-2.16.021-002 |
| AC-005-011-018-023-wire-shape-mock (.gif/.webm) | AC-023 (RG-023) | BC-2.16.021 §PC2, §PC4, §PC6 | TV-BC-2.16.021-007, EC-016-021-010 |
| AC-006-007-008-013-014-019-025-026-unit-tests (.gif/.webm) | AC-006 (RG-006) | BC-2.16.020 §PC6 | EC-016-020-003 |
| AC-006-007-008-013-014-019-025-026-unit-tests (.gif/.webm) | AC-007 (RG-007) | BC-2.16.020 §Invariants | EC-016-020-001 |
| AC-006-007-008-013-014-019-025-026-unit-tests (.gif/.webm) | AC-008 (RG-008) | BC-2.16.020 §PC1 | EC-016-020-004 |
| AC-006-007-008-013-014-019-025-026-unit-tests (.gif/.webm) | AC-013 (RG-013) | BC-2.16.020 §Invariants | EC-016-020-002 |
| AC-006-007-008-013-014-019-025-026-unit-tests (.gif/.webm) | AC-014 (RG-014) | BC-2.16.020 §PC6 | spike-findings §Spike 3 §Table B |
| AC-006-007-008-013-014-019-025-026-unit-tests (.gif/.webm) | AC-019 (RG-019) | BC-2.16.021 §Invariants | EC-016-021-001 |
| AC-006-007-008-013-014-019-025-026-unit-tests (.gif/.webm) | AC-025 (RG-025a, RG-025b) | BC-2.16.021 §Invariants, §PC2 | EC-016-021-002,004 |
| AC-006-007-008-013-014-019-025-026-unit-tests (.gif/.webm) | AC-026 (RG-026) | BC-2.16.021 §PC6 | EC-016-021-010, spike-findings §Spike 3 §Table D |
