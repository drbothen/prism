---
document_type: validation-matrix
producer: product-owner
version: "0.2"
project: prism
timestamp: "2026-08-21"
develop_head: "362e4f85"
naming_regime: POST-ROUTING-001 (ocsf_column_naming=true)
routing_001_status: draft (in delivery — must merge before v1 release gate runs)
tables_in_scope: 4 (alerts, audit_logs, devices, device_alert_relations)
scope: Claroty xDome v1 release gate — full SOC-analyst path against real xDome tenant
note: >
  This is an OPEN, human-facing planning artifact. NOT holdout scenarios.
  Do NOT read .factory/holdout-scenarios/ when working with this file.
supplements:
  - .factory/objectives/xdome-v1-validation/soc-analyst-qa-catalog.md
traces_to:
  - .factory/objectives/xdome-v1-validation/feature-inventory.md
  - .factory/specs/behavioral-contracts/BC-2.16.003-column-to-ocsf-mapping.md
  - .factory/specs/behavioral-contracts/BC-2.16.002-multi-step-fetch-pipeline.md
  - .factory/stories/S-ADR058-OCSF-ROUTING-001-sensor-spec-ocsf-field-name-routing.md
  - crates/prism-sensors/specs/claroty.sensor.toml
---

# Claroty xDome v1 — RELEASE-GATE Live Validation Matrix

> **Naming regime:** All column names in this matrix reflect the **POST-ROUTING-001**
> Arrow field names (`ocsf_column_naming = true`). The matrix is intentionally written
> against the regime that ships — not the current `develop@362e4f85` interim state where
> Arrow names equal `col.name`. Running this matrix against a binary that has NOT merged
> `S-ADR058-OCSF-ROUTING-001` will produce false failures on Field Mapping and most Query
> checks. Gate 2 (Field Mapping) and Gate 3 (Queries) REQUIRE ROUTING-001 merged.

> **Credentials:** All live checks require a real Claroty xDome tenant with a valid
> bearer token stored via the credential CLI per AD-017. Credential values MUST NOT appear
> in AI context, logs, or any artifact. Use the opaque reference model exclusively.

> **DTU vs LIVE:** Each item is tagged `DTU` (testable against `prism-dtu-claroty`),
> `LIVE` (requires real xDome tenant), or `DTU+LIVE` (DTU for correctness, LIVE for
> compatibility confirmation). Items tagged `LIVE-REQUIRED` cannot be substituted with DTU.

---

## Preamble — POST-ROUTING-001 Arrow Field Name Reference

The table below is the canonical column mapping for each Claroty table **after
S-ADR058-OCSF-ROUTING-001 merges** (including KF-01..KF-12 TOML corrections). Use
this as ground truth for all Field Mapping and Query checks.

### claroty_alerts (ocsf_class: detection_finding, class_uid: 2004)

| TOML col.name | Arrow field name (POST-ROUTING-001) | Type | Tier | Note |
|---|---|---|---|---|
| `id` | `finding_info_uid` | String | Tier-1 | KF-03: was `finding.uid` |
| `status` | `status` | String | Tier-1 | unchanged |
| `detected_time` | `time` | Datetime | Tier-1 | unchanged |
| `updated_time` | `finding_info_modified_time` | Datetime | Tier-1 | KF-12: was `end_time` |
| `description` | `message` | String | Tier-1 | unchanged |
| `alert_name` | `finding_info_title` | String | Tier-1 | KF-04: was `finding.title` |
| `alert_type_name` | (→ raw_extensions) | — | Tier-2 | KF-09: ocsf_field removed |
| `category` | (→ raw_extensions) | — | Tier-2 | KF-08: ocsf_field removed |
| `devices_count` | (→ raw_extensions) | — | Tier-2 | KF-10: ocsf_field removed |
| `alert_class` | (→ raw_extensions) | — | Tier-2 | never had ocsf_field |
| `ot_devices_count` | (→ raw_extensions) | — | Tier-2 | never had ocsf_field |
| (synthesized) | `raw_extensions` | Json | — | single JSON blob per row |
| (synthesized) | `class_uid` | Integer | — | = 2004 |
| (synthesized) | `_sensor` | String | — | = "claroty" |

### claroty_audit_logs (ocsf_class: entity_management, class_uid: 3004)

| TOML col.name | Arrow field name (POST-ROUTING-001) | Type | Tier | Note |
|---|---|---|---|---|
| `action` | `activity_name` | String | Tier-1 | unchanged |
| `user_display_name` | `actor_user_name` | String | Tier-1 | unchanged |
| `timestamp` | `time` | Datetime | Tier-1 | INDEX — push-down eligible by both `timestamp` (col.name) and Arrow `time` (OQ-001 RESOLVED, ADR-058 §I6) |
| `details` | `message` | String | Tier-1 | unchanged |
| `username` | `actor_user_uid` | String | Tier-1 | unchanged |
| `note` | `comment` | String | Tier-1 | unchanged |
| `id` | `metadata_uid` | String | Tier-1 | KF-05 RESOLVED 2026-08-21: ocsf_field = "metadata.uid"; Tier-1 retention required; correlation-critical; RG-021 flip |
| `category` | (→ raw_extensions) | — | Tier-2 | KF-11: ocsf_field removed |
| (synthesized) | `raw_extensions` | Json | — | keys: "category" (id is now Tier-1 as metadata_uid) |
| (synthesized) | `class_uid` | Integer | — | = 3004 (KF-01: was 3001) |
| (synthesized) | `_sensor` | String | — | = "claroty" |

### claroty_devices (ocsf_class: inventory_info, class_uid: 5001)

| TOML col.name | Arrow field name (POST-ROUTING-001) | Type | Tier | Note |
|---|---|---|---|---|
| `uid` | `device_uid` | String | Tier-1 | unchanged |
| `asset_id` | `device_instance_uid` | String | Tier-1 | unchanged |
| `device_category` | `device_type` | String | Tier-1 | unchanged |
| `device_type` | `device_type_label` | String | Tier-1 | KF-06: was `device.type_name` |
| `risk_score` | `risk_score` | String | Tier-1 | unchanged |
| `retired` | `status_code` | Boolean | Tier-1 | unchanged |
| `device_name` | `device_name` | String | Tier-1 | unchanged |
| `os_category` | `device_os_name` | String | Tier-1 | unchanged |
| `ip_list` | (→ raw_extensions) | — | Tier-2 | JSON-list string in blob |
| `mac_list` | (→ raw_extensions) | — | Tier-2 | JSON-list string |
| `network_list` | (→ raw_extensions) | — | Tier-2 | JSON-list string |
| `vlan_list` | (→ raw_extensions) | — | Tier-2 | JSON-list string (integer elements stringified) |
| `purdue_level` | (→ raw_extensions) | — | Tier-2 | |
| `site_name` | (→ raw_extensions) | — | Tier-2 | |
| `device_subcategory` | (→ raw_extensions) | — | Tier-2 | |
| `device_type_family` | (→ raw_extensions) | — | Tier-2 | |
| `criticality` | (→ raw_extensions) | — | Tier-2 | |
| `is_online` | (→ raw_extensions) | — | Tier-2 | Boolean value in JSON |
| `manufacturer` | (→ raw_extensions) | — | Tier-2 | |
| `model` | (→ raw_extensions) | — | Tier-2 | |
| (synthesized) | `raw_extensions` | Json | — | all 12 Tier-2 keys |
| (synthesized) | `class_uid` | Integer | — | = 5001 (regression guard: NOT 0) |
| (synthesized) | `_sensor` | String | — | = "claroty" |

### claroty_device_alert_relations (ocsf_class: detection_finding, class_uid: 2004)

| TOML col.name | Arrow field name (POST-ROUTING-001) | Type | Tier | Note |
|---|---|---|---|---|
| `device_uid` | `device_uid` | String | Tier-1 | unchanged |
| `alert_id` | `finding_info_uid` | String | Tier-1 | KF-07: was `finding.uid` |
| `device_alert_detected_time` | `time` | Datetime | Tier-1 | unchanged |
| `device_risk_score` | `risk_score` | String | Tier-1 | unchanged |
| `alert_note` | `comment` | String | Tier-1 | unchanged |
| `device_alert_status` | `status` | String | Tier-1 | unchanged |
| `network_signature_severity` | (→ raw_extensions) | — | Tier-2 | |
| `network_signature_confidence` | (→ raw_extensions) | — | Tier-2 | |
| `malicious_ip_severity` | (→ raw_extensions) | — | Tier-2 | |
| `external_ip` | (→ raw_extensions) | — | Tier-2 | |
| (synthesized) | `raw_extensions` | Json | — | |
| (synthesized) | `class_uid` | Integer | — | = 2004 |
| (synthesized) | `_sensor` | String | — | = "claroty" |

---

## Gate 1: ONBOARDING

Items cover config/org setup, credential wiring, sensor reachability, and health check against the live tenant.

| ID | What to do | Exact expected result | How to verify | DTU/LIVE | Priority |
|---|---|---|---|---|---|
| OB-001 | Set `CLAROTY_INSTANCE_URL` env var to the xDome instance base URL. Store `bearer_token` for the Claroty org via `prism credential store` CLI. Boot prism. | `prism start` boots without error. Boot log shows Claroty adapter registered for the org. No credential value appears in any log line or MCP response. | Read boot log output; grep for `"claroty"` sensor registration line; grep logs for the actual credential value (must be absent). | LIVE-REQUIRED | P0 |
| OB-002 | Run `prism_describe` with the configured org/client_id. | Response includes all four tables: `claroty_alerts`, `claroty_audit_logs`, `claroty_devices`, `claroty_device_alert_relations`. No extra tables (vulnerabilities, tags are NOT surfaced). | `prism_describe` MCP call; assert JSON response `.tables[*].name` contains exactly those 4 prefixed names. | DTU+LIVE | P0 |
| OB-003 | Credential Tier-1 resolution: store token in env var `CLAROTY_BEARER_TOKEN` (or equivalent env-ref form). Verify prism resolves it at boot. | Prism resolves the credential from env without keyring access. Sensor health shows `auth_valid: true` against live xDome. | Check boot log for `env` resolution path; run `check_sensor_health`; confirm no keyring syscall on macOS (no Keychain prompt). | LIVE-REQUIRED | P1 |
| OB-004 | Credential Tier-3 path: configure org with `org_id` + `keyring` reference. Explicitly remove the keyring entry so it is unavailable. Attempt to boot. | Prism exits with `E-CRED-008` error (not silent fallthrough to empty credentials, not panic). Exit code is non-zero. | Boot output; assert `E-CRED-008` in stderr; assert no sensor registration for the org after boot fails. | DTU | P1 |
| OB-005 | Configure prism with an intentionally invalid or expired Claroty bearer token. Boot and run `check_sensor_health`. | Boot succeeds (credential resolution succeeds; token is opaque at boot). `check_sensor_health` returns `reachable: true`, `auth_valid: false`. No boot crash. Process stays running. | `check_sensor_health` MCP call; assert wire JSON `reachable == true`, `auth_valid == false`. | LIVE-REQUIRED | P0 |
| OB-006 | Trigger boot failure by corrupting the audit-init path (e.g., malformed audit log config). Separately, trigger credential-init failure (missing required credential). | Audit-init failure → `exit(4)`. Credential-init failure → `exit(5)`. Both exit codes match BC-2.08.001 / ADR-022 §A table. | Run prism binary with corrupted config; capture `$?`; assert correct exit codes. | DTU | P1 |
| OB-007 | With prism running and Claroty adapter registered, remove the Claroty sensor spec from the active config (empty snapshot) and trigger hot-reload via `reload_config` MCP call. Then restore the spec and reload again. | On removal: Claroty adapter deregistered; `prism_describe` no longer returns `claroty_*` tables. On restore: adapter re-registered; `prism_describe` returns all 4 tables again. Error surfaced if any — not silent. | `reload_config` MCP call; `prism_describe` before/after each reload; assert table count changes correctly. | DTU+LIVE | P1 |

**Gate 1 item count: 7**

---

## Gate 2: FIELD MAPPING

Items verify every column maps to the correct POST-ROUTING-001 Arrow field name and class_uid routes correctly. All assertions must be at the **serialized wire shape** level (Arrow JSON column names), not only pre-serialization Rust structs.

| ID | What to do | Exact expected result | How to verify | DTU/LIVE | Priority |
|---|---|---|---|---|---|
| FM-001 | Run `prism_describe claroty_alerts` (post-ROUTING-001 binary). | Returns exactly 6 Tier-1 `ColumnDescriptor` entries (`finding_info_uid`, `status`, `time`, `finding_info_modified_time`, `message`, `finding_info_title`) plus 1 `raw_extensions` descriptor with `col_type: Json`, `nullable: true`, and description enumerating source keys `alert_type_name`, `category`, `devices_count`, `alert_class`, `ot_devices_count`. **PLUS 2 synthesized descriptors (OQ-003 RESOLVED): `class_uid` (Integer, non-nullable) and `_sensor` (String, non-nullable), appended after the raw_extensions descriptor.** No individual descriptor for any Tier-2 col.name. | Assert serialized `prism_describe` JSON against the exact column list; assert absence of `alert_type_name`, `category`, `devices_count` as individual names; assert one `raw_extensions` entry; **assert `class_uid` descriptor with `col_type: Integer, nullable: false`; assert `_sensor` descriptor with `col_type: String, nullable: false`.** | DTU+LIVE | P0 |
| FM-002 | Run `prism_describe claroty_audit_logs`. | Returns **7** Tier-1 entries (`activity_name`, `actor_user_name`, `time`, `message`, `actor_user_uid`, `comment`, **`metadata_uid`** [KF-05 RESOLVED]) plus 1 `raw_extensions` entry with description enumerating `category` only (not `id` — now Tier-1 as `metadata_uid`). **PLUS 2 synthesized descriptors (OQ-003 RESOLVED): `class_uid` (Integer, non-nullable) and `_sensor` (String, non-nullable).** No individual `category` descriptor. | Same assertion pattern as FM-001; assert `metadata_uid` descriptor present with String type; assert `raw_extensions` description contains `"category"` but NOT `"id"`; **assert `class_uid` and `_sensor` synthesized descriptors.** | DTU+LIVE | P0 |
| FM-003 | Run `prism_describe claroty_devices`. | Returns 8 Tier-1 entries (`device_uid`, `device_instance_uid`, `device_type`, `device_type_label`, `risk_score`, `status_code`, `device_name`, `device_os_name`) plus 1 `raw_extensions` entry enumerating the 12 Tier-2 source keys. **PLUS 2 synthesized descriptors (OQ-003 RESOLVED): `class_uid` (Integer, non-nullable) and `_sensor` (String, non-nullable).** No individual descriptors for `ip_list`, `mac_list`, `purdue_level`, `is_online`, etc. | Same assertion pattern; **assert `class_uid` and `_sensor` synthesized descriptors.** | DTU+LIVE | P0 |
| FM-004 | Run `prism_describe claroty_device_alert_relations`. | Returns 6 Tier-1 entries (`device_uid`, `finding_info_uid`, `time`, `risk_score`, `comment`, `status`) plus 1 `raw_extensions` entry enumerating `network_signature_severity`, `network_signature_confidence`, `malicious_ip_severity`, `external_ip`. **PLUS 2 synthesized descriptors (OQ-003 RESOLVED): `class_uid` (Integer, non-nullable) and `_sensor` (String, non-nullable).** | Same assertion pattern; **assert `class_uid` and `_sensor` synthesized descriptors.** | DTU+LIVE | P0 |
| FM-005 | Execute `SELECT class_uid FROM claroty_alerts LIMIT 1` (or `FROM claroty_alerts \| limit 1`). | Wire JSON: `class_uid` column = `2004` (Int32). Not `0`. Not any other value. | Assert serialized Arrow JSON column value. | DTU+LIVE | P0 |
| FM-006 | Execute `SELECT class_uid FROM claroty_audit_logs LIMIT 1`. | Wire JSON: `class_uid` = `3004` (entity_management). NOT `3001` (the old `audit_activity` value). This is the KF-01 regression check. | Assert serialized column value explicitly asserts `!= 3001` AND `== 3004`. | DTU+LIVE | P0 |
| FM-007 | Execute `SELECT class_uid FROM claroty_devices LIMIT 1`. | Wire JSON: `class_uid` = `5001`. NOT `0` (which would be the BASE_EVENT fallback if the `inventory_info` arm is missing from `select_by_class_name`). This is the KF-02 regression guard (RG-017). | Assert `== 5001` AND `!= 0`. This is the explicit regression guard. | DTU+LIVE | P0 |
| FM-008 | Execute `SELECT class_uid FROM claroty_device_alert_relations LIMIT 1`. | Wire JSON: `class_uid` = `2004`. | Assert column value. | DTU+LIVE | P0 |
| FM-009 | Query `claroty_alerts` from a live tenant where `id` values are known to be returned as JSON integers (e.g., `id: 12345` in the API response). Inspect the `finding_info_uid` column in the result. | `finding_info_uid` column is String type; the integer ID is serialized as the string `"12345"` — not as an Integer column, not as null. (Polymorphic ID normalization per EC-016-013-004.) | Check Arrow schema field type is `Utf8`; assert value is the string-encoded integer. | LIVE-REQUIRED | P0 |
| FM-010 | Query `claroty_alerts` with a WHERE clause on `time` (datetime column). Execute against live tenant whose `detected_time` values are ISO-8601 strings. | Datetime values parse correctly via implicit `["iso8601"]` default (ADR-028 §D8-B). Rows return with populated `time` column; no `E-SPEC-018` timestamp-parse-failed errors in logs. | Check prism logs for timestamp parse errors (must be absent); assert rows returned with non-null `time`. | LIVE-REQUIRED | P1 |
| FM-011 | Materialize a `claroty_alerts` RecordBatch (any query returning rows). Inspect the serialized Arrow JSON. | (a) No first-class Arrow columns named `alert_type_name`, `category`, `devices_count`, `alert_class`, or `ot_devices_count`. (b) A `raw_extensions` column exists. (c) The JSON value of `raw_extensions` for each row contains keys `"alert_type_name"`, `"category"`, `"devices_count"`, `"alert_class"`, `"ot_devices_count"` with the corresponding vendor values preserved. | Wire-level assertion on serialized JSON (per CLAUDE.md §Conventions wire-shape assertion discipline). | DTU+LIVE | P0 |
| FM-012 | Materialize a `claroty_audit_logs` RecordBatch. Inspect the serialized Arrow JSON. | (a) First-class `metadata_uid` String column IS present (KF-05 RESOLVED; Tier-1); value equals the audit record ID string. (b) No first-class `category` column. (c) `raw_extensions` JSON blob contains key `"category"` with preserved vendor value; does NOT contain key `"id"` or `"metadata_uid"` (those are Tier-1, not Tier-2). (d) The audit record's ID is accessible directly via `SELECT metadata_uid FROM claroty_audit_logs`. | Wire-level assertion on serialized JSON; assert `metadata_uid` column present with String type; assert no `"metadata_uid"` key in `raw_extensions`; assert no `"id"` key in `raw_extensions`. | DTU+LIVE | P0 |
| FM-013 | Materialize a `claroty_devices` RecordBatch for a device known to have multiple IP addresses (e.g., from DTU fixture with `ip_list: ["10.0.1.1","10.0.1.2"]`). Inspect `raw_extensions`. | `raw_extensions["ip_list"]` = `"[\"10.0.1.1\",\"10.0.1.2\"]"` — compact JSON-list **string**, NOT a nested JSON array. Same for `mac_list`, `network_list`, `vlan_list` (EC-016-013-028 compact JSON-list string serialization). | Wire-level assertion: assert value IS a String, NOT a JSON array node; assert exact compact-string format. | DTU | P0 |
| FM-014 | Materialize a `claroty_device_alert_relations` RecordBatch for a row with `network_signature_severity` and `external_ip` populated. | (a) No first-class `network_signature_severity`, `network_signature_confidence`, `malicious_ip_severity`, `external_ip` columns. (b) These values present in `raw_extensions` JSON. | Wire-level assertion. | DTU+LIVE | P0 |
| FM-015 | Construct a test SensorSpec with `ocsf_column_naming = true` and two columns whose `ocsf_field` values flatten to the same Arrow name (e.g., `ocsf_field = "a.b_c"` and `ocsf_field = "a_b.c"` both → `a_b_c`). | `pipeline_result_to_record_batch` returns `Err(ArrowError::SchemaError(...))` — fail-closed. Not a silently wrong schema. | Unit test on `pipeline_result_to_record_batch`; assert `Err` variant. (RG-009 in story spec.) | DTU | P1 |
| FM-016 | Construct a SensorSpec with `ocsf_field = "class_uid"` (or `"raw_extensions"`, `"_sensor"`, `"category_uid"`) — fields that flatten to reserved synthesized names. | `pipeline_result_to_record_batch` returns `Err(ArrowError::SchemaError(...))` — fail-closed (EC-016-013-029, ADR-058 v2.26 §J2). | Unit test; assert `Err` on each reserved name. (RG-027 in story spec.) | DTU | P1 |
| FM-017 | Query `claroty_device_alert_relations` and inspect the `alert_id`-derived column. | Arrow column is named `finding_info_uid` (NOT `finding_uid` — the old pre-KF-07 form). Value for a known alert is the alert ID string. | `prism_describe` + actual query; assert column name; assert `finding_uid` column is ABSENT. (RG-020.) | DTU+LIVE | P0 |
| FM-018 | Verify `ocsf_field_to_arrow_name` crate location via Cargo.toml dependency graph. | Function resides in `prism-spec-engine::column_mapping`. `prism-mcp` imports it from `prism-spec-engine` (not from `prism-bin`). No `prism-mcp → prism-bin` dependency edge exists. | `grep -r "ocsf_field_to_arrow_name" crates/` confirms definition in `prism-spec-engine/src/column_mapping.rs`; `cargo tree -p prism-mcp` must NOT show a `prism-bin` dependency. | DTU | P0 |
| FM-019 | After ROUTING-001 merge, run the query `SELECT id FROM claroty_alerts LIMIT 1` (using the OLD pre-ROUTING-001 col.name). | Returns `E-QUERY-038` (ColumnNotFound) with a message naming `id` as invalid and listing `finding_info_uid` as the correct column. NOT a silent empty result set, NOT a query engine panic. | MCP `prism_query` call; assert error code `E-QUERY-038` in response JSON; assert suggestion field names correct column. | DTU+LIVE | P0 |
| FM-020 | Call `prism_describe` for each of the 4 Claroty tables and run each table's `example_query` verbatim. | Each `example_query` uses POST-ROUTING-001 Arrow column names (e.g., `finding_info_uid` not `id`), executes without parse error, and returns at least one row. | Run each `example_query` via `prism_query` MCP; assert no `E-QUERY-038` column errors; assert non-empty result. | DTU+LIVE | P1 |

**Gate 2 item count: 20**

---

## Gate 3: QUERIES

Items cover the representative PrismQL query catalog against all 4 Claroty tables. All column references use POST-ROUTING-001 Arrow names.

| ID | What to do | Exact expected result | How to verify | DTU/LIVE | Priority |
|---|---|---|---|---|---|
| QU-001 | `FROM claroty_alerts \| fields finding_info_uid, status, time, message \| limit 5` | Returns rows with exactly those 4 columns populated (no extra columns unless `class_uid`/`_sensor` are included by default). `finding_info_uid` is String. `time` is parseable as ISO-8601 datetime. | Inspect column names and types in Arrow result schema. | DTU+LIVE | P0 |
| QU-002 | `FROM claroty_alerts \| where status = 'Unresolved' \| limit 10` | Returns only rows where `status` equals `'Unresolved'` exactly (case-sensitive `=`). No rows with `status = 'Resolved'` in result. | Assert all returned rows have `status == 'Unresolved'`; cross-check with total count via a separate count query. | DTU+LIVE | P0 |
| QU-003 | `FROM claroty_alerts \| where time > '2024-01-01T00:00:00Z' \| where time < '2025-01-01T00:00:00Z' \| limit 20` | Returns only rows where `time` falls within the specified range. Rows outside the range absent. (Note: `time` is NOT an INDEX column for alerts — this is in-engine DataFusion filter, NOT a push-down.) | Assert all returned rows have `time` within the specified range. Verify no `filter_by` appears in alerts POST body (no push-down expected). | DTU+LIVE | P0 |
| QU-004 | `FROM claroty_alerts \| where status IEQ 'unresolved' \| limit 10` | Case-insensitive match: returns rows where `status` (exact stored case) matches `'Unresolved'` regardless of analyst-typed casing. Same row set as QU-002. | Compare row count / IDs against QU-002 result. | DTU+LIVE | P1 |
| QU-005 | `FROM claroty_audit_logs \| where activity_name IS NOT NULL \| order by time desc \| limit 20` | Returns 20 most recent audit log entries, ordered by `time` descending. `activity_name` is non-null for all returned rows. | Assert row order: each row's `time` >= next row's `time`. Assert `activity_name` non-null. | DTU+LIVE | P0 |
| QU-006 | `FROM claroty_alerts \| group by status \| count(*) as alert_count \| order by alert_count desc` | Returns one row per distinct `status` value with count. Typical statuses: `'Unresolved'`, `'Resolved'`. No casing fragmentation (`'unresolved'` vs `'Unresolved'` must not appear as separate buckets). | Assert GROUP BY result has at most a few distinct buckets; verify no casing-duplicate buckets for the same logical status. | DTU+LIVE | P1 |
| QU-007 | `FROM claroty_devices \| where device_type = 'OT Device' \| fields device_uid, device_type_label, risk_score, device_name \| limit 20` | Returns OT devices with the 4 requested fields. `device_type` (from `device_category`) is the category column; `device_type_label` (from `device_type`, post-KF-06) is the type-within-category column. Both are present and distinct. | Assert both columns present; assert no `device_type_name` column (old pre-KF-06 name absent). | DTU+LIVE | P0 |
| QU-008 | `FROM claroty_devices \| fields device_uid, raw_extensions \| limit 5` (raw_extensions Tier-2 access). Then parse `raw_extensions` JSON for key `"is_online"`. | `raw_extensions` column present as Json/String type. JSON blob parseable; key `"is_online"` present with boolean value. Verify Tier-2 data is accessible via SELECT/fields even without a direct WHERE predicate. | Assert `raw_extensions` column in result; parse JSON; assert `"is_online"` key present in at least one row. **Note on Tier-2 filtering:** `WHERE raw_extensions['is_online'] = false` is NOT supported until the `json_extract_string` DataFusion ScalarUDF story ships (S-JSON-EXTRACT-UDF-001, depends_on ROUTING-001; OQ-002 RESOLVED — gated on that story). SELECT access to raw_extensions is available in v1; WHERE predicates on Tier-2 keys require the UDF story. | DTU+LIVE | P1 |
| QU-009 | Configure a test scenario or use a live tenant with more than 1000 audit_log entries. Execute `FROM claroty_audit_logs \| limit 2000`. | All pages fetched via offset_limit POST-body pagination. Result contains 2000 rows (or all available if fewer). Pagination sequence: page 1 `offset=0 limit=1000`, page 2 `offset=1000 limit=1000`. | Assert total rows in result = min(2000, total_count). Check prism fetch logs for multiple POST requests to `/api/v1/audit_log/get` with incrementing offsets. | DTU+LIVE | P0 |
| QU-010 | Execute `FROM alerts \| limit 5` (no sensor prefix). | Returns `E-SENSOR-030` or `E-QUERY-037` (table not found / sensor prefix required). NOT silent empty result set, NOT a query that returns Claroty data. Error message hints that sensor-prefixed names are required. | Assert error code in MCP response. Assert response is not a 0-row success. | DTU | P0 |
| QU-011 | Execute `FROM claroty_alerts \| fields finding_info_uid, nonexistent_column \| limit 1`. | Returns `E-QUERY-038` (ColumnNotFound). Error message names `nonexistent_column` as invalid. Error message lists valid columns for `claroty_alerts` (at minimum `finding_info_uid`, `status`, `time`, `message`, `finding_info_title`, `finding_info_modified_time`, `raw_extensions`). | Assert `E-QUERY-038` in response; assert suggestion list contains correct POST-ROUTING-001 column names. | DTU+LIVE | P0 |
| QU-012 | Cross-table investigation: (1) `FROM claroty_alerts \| where status = 'Unresolved' \| fields finding_info_uid \| limit 5` to get alert IDs. (2) `FROM claroty_device_alert_relations \| where finding_info_uid = '<id-from-step-1>' \| fields device_uid, status \| limit 10` to find affected devices. (3) `FROM claroty_devices \| where device_uid = '<device_uid-from-step-2>' \| limit 1` for device profile. | Each query in the three-step pivot returns data. The `finding_info_uid` used in step 2 matches the `finding_info_uid` column name in `claroty_device_alert_relations` (KF-07 correction). The `device_uid` from step 2 matches the `device_uid` in `claroty_devices`. No `E-QUERY-038` column-not-found errors. | Manually execute the 3-step pivot; assert data flows correctly across all 3 tables using POST-ROUTING-001 column names. | DTU+LIVE | P0 |

**Gate 3 item count: 12**

---

## Gate 4: PUSH-DOWN

Items verify the `audit_logs` time-box push-down works against the live tenant, document the no-push-down tables, and close **ASM-CLAROTY-AUDITLOG-001**.

> **POST-ROUTING-001 naming impact on push-down — RESOLVED (OQ-001, ADR-058 §I6, ROUTING-001 RG-PD-001):**
> After ROUTING-001, `timestamp` (TOML col.name) becomes `time` (Arrow field name).
> Analyst queries use `WHERE time > 'T'`, not `WHERE timestamp > 'T'`. The
> `extract_time_window_from_ast` mechanism in `prism-query::pushdown` is specified
> (BC-2.16.003 EC-016-013-031) to register BOTH `timestamp` (col.name) AND `time`
> (Arrow name) as INDEX-eligible — both forms trigger push-down. PD-007 is the live
> confirmation item for this fix.

| ID | What to do | Exact expected result | How to verify | DTU/LIVE | Priority |
|---|---|---|---|---|---|
| PD-001 | Execute `FROM claroty_audit_logs \| limit 100` with NO time predicate. Inspect the POST body sent to `/api/v1/audit_log/get`. | POST body contains `"filter_by": {"field": "timestamp", "operation": "greater_or_equal", "value": "<now-7days-iso8601>"}` (EC-01-030: bounded default, never unbounded). Result is bounded to the last 7 days. No `E-QUERY-004` timeout. | Capture HTTP request body in prism fetch log (`tracing::debug!` at the POST step); assert `filter_by` key present; assert value is within 7 days of current time. | DTU+LIVE | P0 |
| PD-002 | Execute `FROM claroty_audit_logs \| where time > '2024-06-01T00:00:00Z' \| limit 100`. Inspect POST body. | POST body contains `"filter_by": {"field": "timestamp", "operation": "greater_or_equal", "value": "2024-06-01T00:00:00Z"}` — single `greater_or_equal` bound, no upper bound (EC-01-031). | Capture POST body; assert exact field/operation/value. | DTU+LIVE | P0 |
| PD-003 | Execute `FROM claroty_audit_logs \| where time < '2024-06-30T00:00:00Z' \| limit 100`. Inspect POST body. | POST body contains `"filter_by": {"field": "timestamp", "operation": "less_or_equal", "value": "2024-06-30T00:00:00Z"}`. NO synthetic lower bound added (EC-01-032). | Capture POST body; assert single `less_or_equal` predicate only; assert no `greater_or_equal` present. | DTU+LIVE | P0 |
| PD-004 | Execute `FROM claroty_audit_logs \| where time > '2024-06-01T00:00:00Z' \| where time < '2024-06-30T00:00:00Z' \| limit 100`. Inspect POST body. | POST body contains `"filter_by": {"operation": "and", "operands": [{"field": "timestamp", "operation": "greater_or_equal", "value": "2024-06-01T00:00:00Z"}, {"field": "timestamp", "operation": "less_or_equal", "value": "2024-06-30T00:00:00Z"}]}` (EC-01-033). Key MUST be `"operands"` NOT `"conditions"`. | Capture POST body; assert `"operation": "and"`, `"operands"` key (not `"conditions"`); assert both bounds present. | DTU+LIVE | P0 |
| PD-005 | **ASM-CLAROTY-AUDITLOG-001 LIVE CONFIRMATION.** Execute PD-001 through PD-004 against the **real xDome tenant** (not DTU). Confirm the live xDome API accepts the filter structure. | Real xDome returns HTTP 200 with non-empty results for each filter variant. No 400 (malformed filter), no 422 (validation error) from xDome. This is the one-line live-check that closes ASM-CLAROTY-AUDITLOG-001. | Execute all 4 time-filter variants against live xDome; assert HTTP 200 response; assert non-empty result or plausible empty (if date range has no events). Log confirmation: "ASM-CLAROTY-AUDITLOG-001: CONFIRMED / REFUTED" with the actual HTTP status and response shape. | LIVE-REQUIRED | P0 |
| PD-006 | Execute `FROM claroty_alerts \| where time > '2024-01-01T00:00:00Z' \| limit 50`. Inspect POST body to `/api/v1/alerts/`. | POST body to alerts endpoint does NOT contain any `filter_by` key. All filtering happens in-engine (DataFusion post-filter). Only `fields`, `offset`, `limit` appear in the body. | Capture POST body; assert no `filter_by` key; assert all returned rows satisfy the time predicate (DataFusion in-engine filter is correct). | DTU+LIVE | P0 |
| PD-007 | **[ROUTING-001 push-down live confirmation — RG-PD-001].** OQ-001 RESOLVED IN SCOPE (ADR-058 §I6, ROUTING-001 RG-PD-001): `extract_time_window_from_ast` registers BOTH `timestamp` (TOML col.name) AND `time` (Arrow field name) as INDEX-eligible. This item is now a **live confirmation** that the fix holds against the real xDome tenant. Execute `FROM claroty_audit_logs \| where time > '2024-01-01T00:00:00Z' \| limit 10` using `time` (POST-ROUTING-001 Arrow name). | Push-down fires: POST body contains `filter_by.operation = "greater_or_equal"`. This CONFIRMS the fix. If push-down does NOT fire against the live binary (after ROUTING-001 merge), it indicates the RG-PD-001 Red Gate test was insufficient — flag **BLOCKING** and escalate to implementer. | Capture POST body; assert `filter_by` present. Confirm "CONFIRMED" or flag "REGRESSION" with actual POST body snippet. | DTU+LIVE | P0 |
| PD-008 | Execute `FROM claroty_audit_logs \| where time < '2020-01-01T00:00:00Z' \| where time > '2024-01-01T00:00:00Z'` (inverted window: start > end). | Prism emits a `push_down.inverted_time_range` WARN log event. Both bounds still passed to xDome in the POST body. DataFusion in-engine post-filter backstop ensures zero rows returned (since the time range is logically impossible). No crash, no panic, no silent wrong data. | Check prism log for inverted-window WARN; assert result is 0 rows; assert POST body still contains both bounds. | DTU | P1 |

**Gate 4 item count: 8**

---

## Gate 5: SOC-Analyst Q&A Catalog

The heart of the release gate: 27 real analyst questions spanning all 4 tables and cross-table investigation flows, each with an exact PrismQL query, expected data, and pass criterion.

**See companion file:** `.factory/objectives/xdome-v1-validation/soc-analyst-qa-catalog.md`

**Item count: 27**

---

## Gate 6: STABILITY AND RESILIENCE

Items cover concurrent queries, fan-out bounds, memory/timeout budgets, error/health behavior, transport hardening, and graceful partial-failure.

| ID | What to do | Exact expected result | How to verify | DTU/LIVE | Priority |
|---|---|---|---|---|---|
| SR-001 | Issue 3 concurrent PrismQL queries against `claroty_devices`, `claroty_alerts`, and `claroty_audit_logs` simultaneously from the same prism server instance. | All 3 queries execute concurrently (no serialization by lock). `PrismServer::query` takes `&self`; ArcSwap lock-free config reads. All 3 return results without deadlock or panic. Wall-clock time for concurrent execution is NOT (query1 + query2 + query3) sequential. | Issue all 3 via separate MCP client connections or measure wall-clock elapsed; confirm no lock waits in prism logs; assert all 3 complete successfully. | DTU | P0 |
| SR-002 | Query a large `claroty_alerts` dataset (>5000 rows if available). Monitor memory usage during the query. | Memory per-query stays within 200MB budget (BC-2.01.002 / project memory project_memory_budget.md). All pages fetched via offset_limit pagination. No `E-QUERY-004` timeout on a dataset of typical MSSP tenant size. | Monitor process RSS during query; assert peak < 200MB per-query overhead. | DTU+LIVE | P1 |
| SR-003 | Simulate a slow xDome endpoint (artificially delay responses via DTU). Verify the 30s HTTP timeout fires. | After 30s, prism surfaces `E-SENSOR-002` (timeout error). No indefinite hang. Process continues accepting other queries. | DTU with artificial delay; assert `E-SENSOR-002` in MCP error response within 31s of query start. | DTU | P0 |
| SR-004 | Configure the live xDome sensor with an expired bearer token (or use an invalid token). Run `check_sensor_health`. | `check_sensor_health` returns: `reachable: true`, `auth_valid: false`, `http_status: 403` (or 401), `overall_status: "unhealthy"` or `"auth_invalid"`. NOT `Down`. | Assert `reachable == true`, `auth_valid == false`; assert `overall_status` is not `"healthy"`. | LIVE-REQUIRED | P0 |
| SR-005 | Simulate a Claroty API returning HTTP 5xx (503) via DTU. Run `check_sensor_health`. | `check_sensor_health` returns: `reachable: true`, `auth_valid: true`, `error: "service_unavailable"`, `overall_status: "degraded"`. NOT `Down`. Degraded NOT counted as healthy in the envelope `summary_counts`. | Assert `overall_status == "degraded"`, `reachable == true`, `auth_valid == true`, `error == "service_unavailable"`. | DTU | P0 |
| SR-006 | Completely stop the DTU clone (no TCP listener). Run `check_sensor_health` for Claroty. | `check_sensor_health` returns `reachable: false`, `overall_status: "down"`. `Down` is ONLY returned when no TCP/HTTP exchange was possible. | Assert `reachable == false`, `overall_status == "down"`. | DTU | P0 |
| SR-007 | Verify TLS transport: capture TLS handshake details when prism connects to the real xDome tenant. | TLS negotiation uses rustls (not native-tls). No macOS Keychain initialization delay. HTTP/2 negotiated where supported. `reqwest` client built with `default-features = false, features = ["rustls-tls"]` (ADR-050 D1/D2). | Check prism binary for native-tls dependency (`cargo tree -p prism-bin \| grep native-tls` must return empty); run against live tenant; confirm no 65s Keychain stall; check TLS negotiation logs for `h2` protocol. | LIVE-REQUIRED | P0 |
| SR-008 | Execute a query against `claroty_alerts` while simultaneously pulling a 5xx error from the Claroty DTU for `claroty_devices`. | The `claroty_alerts` query completes successfully. The `claroty_devices` fan-out returns a structured `SensorError::HttpError{status}` (`E-SENSOR-001`). The partial-failure propagates to the MCP caller — NOT silently swallowed as an empty Vec. The error envelope distinguishes `claroty_alerts` success from `claroty_devices` failure. | Assert alerts result is non-empty; assert devices fan-out error is present in the response envelope; assert no `Vec::new()` silent return for the failing leg (Standing Rule 3 §2). | DTU | P0 |
| SR-009 | Execute `FROM claroty_alerts \| limit 10` against the live tenant. Verify the HTTP request uses the correct trailing-slash path. | HTTP POST to `/api/v1/alerts/` (WITH trailing slash). HTTP POST to `/api/v1/audit_log/get` (WITHOUT trailing slash). Both accepted by live xDome. No 404 due to path mismatch. | Check prism fetch log for exact URL; assert trailing-slash format for alerts/devices/device_alert_relations; assert no trailing slash for audit_log/get. | LIVE-REQUIRED | P0 |
| SR-010 | Fan-out bounds: configure prism with 15 sensor adapters across multiple orgs (exceeding MAX_FANOUT_CONCURRENCY=10 per fan-out). Execute a cross-sensor query. | At most 10 concurrent fan-out tasks execute simultaneously (bounded semaphore). Additional tasks queue behind the semaphore. Total HTTP connections bounded by HTTP_SEMAPHORE_PERMITS=200. No deadlock, no unbounded goroutine spawning. | Check prism concurrency-architecture log events; assert concurrent fan-out count <= 10; assert no deadlock. | DTU | P1 |
| SR-011 | Execute a `claroty_devices` query where one step's pagination returns malformed JSON (DTU-injected error). | Prism returns a structured error to the MCP caller identifying the malformed response. NOT a panic. NOT a silent empty result masking the failure. Error message is actionable (identifies the sensor and step). | DTU with injected malformed response; assert structured error in MCP response; assert no server panic. | DTU | P1 |
| SR-012 | Verify `check_sensor_health` for Claroty uses the `probe_table = "devices"` endpoint (not a hardcoded table name). Inspect the LIMIT-0 probe POST request. | Health probe issues `POST /api/v1/devices/` with `{"fields": [...], "offset": 0, "limit": 0}`. NOT `/api/v1/alerts/` (or any other table). Response: HTTP 200 or valid error that provides auth signal. | Check prism health-probe fetch log; assert POST to the `probe_table = "devices"` path with limit=0. | DTU+LIVE | P1 |

**Gate 6 item count: 12**

---

## Priority-1 Live Risk Checks

These 5 items correspond directly to the top-5 risks identified in the feature inventory (§7 of `feature-inventory.md`). Each is a concrete LIVE verification that closes or bounds the risk.

| RISK-ID | Risk | Live Check | Pass Criterion | Escalation if Fails |
|---|---|---|---|---|
| RISK-1 | **ROUTING-001 not merged — query-breaking column rename.** The biggest v1 shape decision: all Claroty Arrow names flip from `col.name` to OCSF-flattened on merge. | Before release gate runs, confirm all 27 Red Gate tests (RG-001..RG-027) pass on the post-ROUTING-001 binary. Run `just iter prism-spec-engine` and `just iter prism-bin` and `just iter prism-mcp` against the merged binary. | Exit 0 on all three crate test runs. All 27 RG tests GREEN. | Block release. Root-cause the failing RG test. Implementer fixes in scope. |
| RISK-2 | **ASM-CLAROTY-AUDITLOG-001 unconfirmed.** `filter_by.field = "timestamp"` and operations `"greater_or_equal"` / `"less_or_equal"` are research-validated but not live-confirmed. If the real xDome API uses different field names or operation strings, audit_log push-down silently returns empty or wrong windows. | Execute PD-005 (Gate 4) against the real xDome tenant with a known date range that has audit events. Record the actual HTTP status and response shape. | HTTP 200; non-empty results for a known-populated date range; filter_by field/operation names accepted without 400/422. | If 400/422: xDome API uses different parameter names. Implementer must update `build_claroty_audit_filter_by` with the correct field/operation names from the live API error response. Block release until confirmed. |
| RISK-3 | **Tier-2 device columns provenance is OpenAPI-only.** The 12 Tier-2 device columns (`ip_list`, `mac_list`, `purdue_level`, `criticality`, etc.) were verified against xDome OpenAPI 2026-06-20, NOT against live API responses. Live xDome may return different field names or structures. | Execute QU-008 (Gate 3) against the live tenant. Also: query `claroty_devices` with `limit 5` and inspect raw_extensions JSON keys against the 12 expected Tier-2 keys. | All 12 expected Tier-2 keys present in live `raw_extensions` JSON; data types consistent with spec; no `E-SPEC-018` parse errors. | If keys differ: update TOML `col.name` values to match live field names. These are Tier-2 (raw_extensions) so no Arrow schema change required, but source_path extraction must reference correct JSON keys. |
| RISK-4 | **No push-down on alerts / devices / device_alert_relations — large-tenant risk.** These 3 tables require full-scan fan-out. Large xDome tenants with >10k devices or alerts risk `E-QUERY-004` timeout and 200MB memory pressure. | Execute `SELECT * FROM claroty_devices LIMIT 10000` (or equivalent) against the live tenant. Monitor query time and memory. If result_count > 5000: measure time and memory; verify 200MB budget; if timeout risk, document the actual record count. | Query completes within 30s for typical tenant size (<5000 devices). If timeout: document actual device count; escalate to architect for a scan-limit or count-cap mechanism before release. | If timeout on typical tenant: add scan-limit or circuit-breaker story to the release milestone. Do NOT silently timeout — make the failure explicit to the analyst (E-QUERY-004). |
| RISK-5 | **DTU-parity tests remain `#[ignore]`'d post-ROUTING-001.** The `S-ADR058-DTU-PARITY-MIGRATION-001` story is PARKED. Reference OCSF fixtures are not recorded. This means the parity tests (`prism-spec-engine/tests/parity/claroty.rs`) cannot validate that real Claroty data round-trips correctly through the OCSF normalization path. | Acknowledge gap explicitly: run all NON-ignored Claroty-path tests via `just iter prism-spec-engine` and `just iter prism-dtu-claroty`. Record the count of skipped parity tests. Confirm each `#[ignore]` has a documented gate condition and story anchor (SID-1 rule). | Non-ignored tests all pass. Each `#[ignore]`'d test has a comment citing blocking dependency (e.g., `// DTU-EXT-001: requires ROUTING-001 + fixture recording; un-gate in S-ADR058-DTU-PARITY-MIGRATION-001`). | If any `#[ignore]`'d test lacks a comment: add the comment per SID-1 before release. The parity gap itself is an accepted known limitation for v1; document it in the release notes. |

---

## Ignored-Test Un-Gate List

Tests that are currently `#[ignore]`'d and the conditions required to un-gate them for v1 or the next release cycle.

| IGN-ID | Test location | Ignore reason | Gate condition to un-ignore | Target story |
|---|---|---|---|---|
| IGN-001 | `prism-dtu-claroty/src/routes/audit_log.rs` — `test_..._pipeline_integration_ac_006` (approx §ac_006 region) | Requires `prism-bin` full-boot wiring with DTU; `todo!()` body | `S-DEMO-002` (full-boot wiring story) merged AND `todo!()` replaced with real test body | S-DEMO-002 |
| IGN-002 | `prism-spec-engine/tests/parity/claroty.rs` §117 — `test_BC_2_16_013_dtu_parity_claroty` | Requires DTU clone + recorded reference OCSF fixtures; blocked on ROUTING-001 + fixture recording pipeline | `S-ADR058-OCSF-ROUTING-001` merged + reference OCSF fixture for Claroty recorded via DTU | `S-ADR058-DTU-PARITY-MIGRATION-001` |
| IGN-003 | `prism-spec-engine/tests/parity/claroty.rs` §188 — second parity variant | Same gate as IGN-002 | Same as IGN-002 | `S-ADR058-DTU-PARITY-MIGRATION-001` |
| IGN-004 | `prism-spec-engine/src/pipeline.rs` — `test_BC_2_16_002_pagination_claroty_alerts_page_2_returns_data` | Requires DTU clone with 102-entry alerts fixture | DTU alerts fixture with >1000 entries recorded (per S-DEMO-CLAROTY-PAGINATION-001 gate) | `S-ADR058-DTU-PARITY-MIGRATION-001` or standalone |
| IGN-005 | `prism-bin/tests/e2e_smoke.rs` — 13 `#[ignore]` attrs (E2E-001) | Requires DTU server + prism binary running; ungated via `e2e` nextest profile | Run via `cargo nextest run --profile e2e` with DTU server started. Un-gate as part of CI `e2e` profile gate on every PR post-ROUTING-001 merge. | CI pipeline story |
| IGN-006 | `prism-bin/tests/e2e_multi_org.rs` — 10 `#[ignore]` attrs (E2E-MULTI-001) | Requires multi-org DTU; ungated via `e2e-multi-org` profile | Run via `cargo nextest run --profile e2e-multi-org` with multi-org DTU harness. | CI pipeline story |

**Total directly-Claroty ignored: 4 (IGN-001..004).
Cross-cutting E2E (including Claroty): 23 (IGN-005: 13 + IGN-006: 10).**

---

## Open Questions

These items could NOT be determined definitively from the specs and code examined. Each requires architect or implementer input before release gate sign-off.

| OQ-ID | Question | Why it matters | Owner | Blocking? |
|---|---|---|---|---|
| OQ-001 | **RESOLVED-IN-SCOPE (2026-08-21, ADR-058 §I6, ROUTING-001 RG-PD-001).** `extract_time_window_from_ast` will register BOTH `timestamp` (TOML col.name) AND `time` (Arrow field name, `ocsf_field_to_arrow_name("time")`) as INDEX-eligible for `audit_logs`. `WHERE time > 'T'` (POST-ROUTING-001 query) WILL trigger push-down. BC-2.16.003 EC-016-013-031 specifies the contract; story-writer leg of ROUTING-001 MUST add RG-PD-001. Live confirmation via PD-007. | The push-down fix is scoped into ROUTING-001. BC-2.16.003 §EC-016-013-031 is the governing contract. | Product-owner (RESOLVED) | RESOLVED-IN-SCOPE |
| OQ-002 | **RESOLVED (2026-08-21) — gated on `json_extract_string` DataFusion ScalarUDF story (S-JSON-EXTRACT-UDF-001, depends_on ROUTING-001, v1-chain delivery).** Human chose: Tier-2 fields WILL be filterable in v1 via a `json_extract_string(json_col, '$.path')` ScalarUDF registered in `build_session_context` / `prism-query::memory`. Until that story ships, WHERE predicates on `raw_extensions` keys will produce an error; SELECT access to `raw_extensions` is available immediately via ROUTING-001. Story-writer materializes `S-JSON-EXTRACT-UDF-001` after this burst. | OQ-002 is resolved as a v1-chain story gated on ROUTING-001. Tier-2 filtering will be available before live validation gate runs. Update QU-008 accordingly. | Product-owner (RESOLVED — gated) | RESOLVED-GATED |
| OQ-003 | **RESOLVED (2026-08-21, BC-2.16.003 §postconditions synthesized column discoverability, ADR-058 §G).** `prism_describe` with `ocsf_column_naming = true` MUST emit `ColumnDescriptor` entries for `class_uid` (Integer, non-nullable) and `_sensor` (String, non-nullable), appended after Tier-1 and Tier-2 descriptors. BC-2.16.003 §Postconditions §Interpretation A specifies the mandate; story-writer leg of ROUTING-001 adds the AC and Red Gate test. FM-001..FM-004 updated to assert these descriptors. | OQ-003 is resolved. BC-2.16.003 §Postconditions synthesized column discoverability is the governing spec. | Product-owner (RESOLVED) | RESOLVED |
| OQ-004 | **T13 demo runbook step 6.3 uses pre-ROUTING-001 column names.** Step 6.3 expects `claroty_audit_logs` to return columns `action`, `actor`, `id`, `resource`, `timestamp`. Post-ROUTING-001: `action` → `activity_name`, `id` → `metadata_uid` (KF-05 RESOLVED — Tier-1), `timestamp` → `time`. The columns `actor` and `resource` were already removed per LIVE-DRIFT-003. The runbook needs a targeted update to use POST-ROUTING-001 names before demo recording (T14). | Demo will fail at Step 6.3 if the runbook is run as-written post-ROUTING-001. | Product-owner (update T13-capstone-demo-runbook.md §6.3 post-ROUTING-001 merge) | YES — blocks T14 recording |
| OQ-005 | **RESOLVED (human-directed 2026-08-21).** `audit_logs.id` maps to `ocsf_field = "metadata.uid"` → Arrow `metadata_uid`, **Tier-1**. The audit record ID is a first-class Arrow column, directly queryable via `WHERE metadata_uid = '<id>'`. `raw_extensions` for `audit_logs` rows contains only `"category"` key. BC-2.16.003 EC-016-013-020 governs this mapping; FM-002 and FM-012 updated to reflect the correct Tier-1 state. TOML change and RG-021 flip delivered by S-ADR058-OCSF-ROUTING-001. | Audit record ID is now Tier-1 as `metadata_uid`. Correlation queries use `WHERE metadata_uid = '<id>'` directly. | Product-owner (RESOLVED) | RESOLVED |

---

## Item Count Summary

| Gate | Category | Count |
|---|---|---|
| Gate 1 | ONBOARDING | 7 |
| Gate 2 | FIELD MAPPING | 20 |
| Gate 3 | QUERIES | 12 |
| Gate 4 | PUSH-DOWN | 8 |
| Gate 5 | SOC-ANALYST Q&A (see companion) | 27 |
| Gate 6 | STABILITY / RESILIENCE | 12 |
| — | PRIORITY-1 RISK CHECKS | 5 |
| — | IGNORE-TEST UN-GATE LIST | 6 |
| — | OPEN QUESTIONS | 5 |
| **Total (excl. SOC Q&A)** | | **70** |
| **Total (incl. SOC Q&A)** | | **97** |
