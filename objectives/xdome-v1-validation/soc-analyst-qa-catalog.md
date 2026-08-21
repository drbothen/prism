---
document_type: soc-qa-catalog
producer: product-owner
version: "0.2"
project: prism
timestamp: "2026-08-21"
develop_head: "362e4f85"
naming_regime: POST-ROUTING-001 (ocsf_column_naming=true)
parent: .factory/objectives/xdome-v1-validation/live-validation-matrix.md
scope: Gate 5 companion — 27 real SOC-analyst questions, Claroty xDome v1
note: >
  OPEN human-facing planning artifact. NOT holdout scenarios.
  Do NOT read .factory/holdout-scenarios/ when working with this file.
traces_to:
  - .factory/objectives/xdome-v1-validation/live-validation-matrix.md
  - .factory/stories/S-ADR058-OCSF-ROUTING-001-sensor-spec-ocsf-field-name-routing.md
  - crates/prism-sensors/specs/claroty.sensor.toml
---

# Claroty xDome v1 — SOC-Analyst Q&A Catalog (Gate 5)

This catalog models real security analyst workflow questions for the Claroty xDome sensor.
Each entry pairs an analyst intent with the exact PrismQL query using POST-ROUTING-001
column names, the expected data returned, and a testable pass criterion.

> **Naming regime:** All queries use POST-ROUTING-001 Arrow field names. ROUTING-001 must
> be merged before running against a live or DTU binary.
>
> **PrismQL pipe operator in table cells:** All `|` characters inside table cells are
> written as `\|` per Markdown table convention.

---

## Distribution by Table

| Table | QA entries |
|---|---|
| claroty_alerts | QA-001..QA-007 (7 entries) |
| claroty_audit_logs | QA-008..QA-014 (7 entries) |
| claroty_devices | QA-015..QA-021 (7 entries) |
| claroty_device_alert_relations | QA-022..QA-024 (3 entries) |
| Cross-table flows | QA-025..QA-027 (3 entries) |
| **Total** | **27** |

---

## claroty_alerts (QA-001..QA-007)

| QA-ID | Analyst Question | Exact Query | Expected Data Returned | Pass Criterion | DTU/LIVE | Priority |
|---|---|---|---|---|---|---|
| QA-001 | What are the 10 most recent unresolved alerts? | `FROM claroty_alerts \| where status = 'Unresolved' \| order by time desc \| limit 10` | 10 rows with `status = 'Unresolved'`, ordered by `time` descending. Fields: `finding_info_uid`, `finding_info_title`, `status`, `time`, `message`. | All rows have `status == 'Unresolved'`; rows are in descending `time` order; no `E-QUERY-038`. | DTU+LIVE | P0 |
| QA-002 | How many total alerts exist, broken down by status? | `FROM claroty_alerts \| group by status \| count(*) as total \| order by total desc` | One row per distinct `status` value (e.g., `Unresolved`, `Resolved`) with count. | `status` column uses POST-ROUTING-001 name; GROUP BY executes; no casing-duplicate buckets for the same logical status. | DTU+LIVE | P0 |
| QA-003 | Show me all alerts with title 'Unauthorized Device Connectivity'. | `FROM claroty_alerts \| where finding_info_title = 'Unauthorized Device Connectivity' \| limit 50` | Rows where `finding_info_title` equals the specified value exactly (case-sensitive). Column is `finding_info_title`, NOT the old `alert_name` (KF-04 correction). | Column name is `finding_info_title`; query executes without `E-QUERY-038`; no `alert_name` column in schema. | DTU+LIVE | P0 |
| QA-004 | What alert categories and types are present in our environment? (Tier-2 field inspection) | `FROM claroty_alerts \| fields raw_extensions \| limit 100` then parse JSON for keys `"category"` and `"alert_type_name"`. | `raw_extensions` column present for each row as a Json/String type. JSON blob contains keys `"category"` and `"alert_type_name"` with vendor values (KF-08/KF-09 moved these to Tier-2; no first-class columns for them). | `raw_extensions` present; JSON parseable; keys `"category"` and `"alert_type_name"` appear; no first-class `category` or `alert_type_name` column in schema. | DTU+LIVE | P1 |
| QA-005 | Show me alerts updated in the last 48 hours. | `FROM claroty_alerts \| where finding_info_modified_time > '2026-08-19T00:00:00Z' \| order by finding_info_modified_time desc \| limit 20` | Rows where `finding_info_modified_time` (KF-12: was `end_time` pre-ROUTING-001) is within the last 48h. Field is `finding_info_modified_time`. | Column is `finding_info_modified_time` (not `updated_time` or `end_time`); datetime filtering works correctly. | DTU+LIVE | P0 |
| QA-006 | Find a specific alert by its ID to examine full details. | `FROM claroty_alerts \| where finding_info_uid = '12345' \| limit 1` | Row with `finding_info_uid = '12345'` as String (even if API returned integer `12345`; polymorphic ID normalization per EC-016-013-004). Column is `finding_info_uid`, NOT `id` (KF-03 correction). | Column is `finding_info_uid`; value is String type; polymorphic integer ID serializes as string `"12345"`; no `id` column in schema. | DTU+LIVE | P0 |
| QA-007 | Describe the claroty_alerts table schema before writing queries. | `prism_describe claroty_alerts` (MCP tool, not PrismQL) | 6 Tier-1 `ColumnDescriptor` entries: `finding_info_uid`, `status`, `time`, `finding_info_modified_time`, `message`, `finding_info_title`. Plus 1 `raw_extensions` Json entry enumerating source keys. The `example_query` in response uses `finding_info_uid` (POST-ROUTING-001 name). | Schema reflects POST-ROUTING-001 names; `example_query` executes without `E-QUERY-038`; no Tier-2 keys surface as individual descriptors. | DTU+LIVE | P0 |

---

## claroty_audit_logs (QA-008..QA-014)

| QA-ID | Analyst Question | Exact Query | Expected Data Returned | Pass Criterion | DTU/LIVE | Priority |
|---|---|---|---|---|---|---|
| QA-008 | Who made changes in the system in the last week? Show the audit trail. | `FROM claroty_audit_logs \| where time > '2026-08-14T00:00:00Z' \| order by time desc \| limit 50` | Rows from the last 7 days, most recent first. Fields: `activity_name`, `actor_user_name`, `time`, `message`. Push-down fires: POST body to `/api/v1/audit_log/get` contains `"filter_by": {"field": "timestamp", "operation": "greater_or_equal", ...}`. | `activity_name` present (not `action`); `actor_user_name` present (not `actor`); push-down POST body contains `filter_by` with `"field": "timestamp"`. | DTU+LIVE | P0 |
| QA-009 | Show me all actions performed by user 'jsmith@example.com'. | `FROM claroty_audit_logs \| where actor_user_uid = 'jsmith@example.com' \| order by time desc \| limit 30` | Rows where `actor_user_uid` (from `username` TOML col) matches the specified value. Column is `actor_user_uid`, NOT `actor`. | Column name `actor_user_uid`; filtering works; no `actor` column in schema. | DTU+LIVE | P0 |
| QA-010 | What happened on 2026-08-01? Show the full audit trail for that day. | `FROM claroty_audit_logs \| where time > '2026-08-01T00:00:00Z' \| where time < '2026-08-02T00:00:00Z' \| order by time asc \| limit 500` | Rows from 2026-08-01 (UTC) only, ascending order. Push-down fires with AND compound filter: POST body has `"operation": "and"`, `"operands"` key (NOT `"conditions"`) with both `greater_or_equal` and `less_or_equal` bounds. | All rows have `time` within the 24h window; POST body has `"operation": "and"` with `"operands"` (not `"conditions"`); both bounds present. | DTU+LIVE | P0 |
| QA-011 | Show me login activity with user display names. | `FROM claroty_audit_logs \| where activity_name = 'Login' \| fields actor_user_name, actor_user_uid, time \| limit 20` | Rows with `actor_user_name` (display name, from `user_display_name` TOML) and `actor_user_uid` (username, from `username` TOML). Both fields populate correctly. | Both `actor_user_name` and `actor_user_uid` present; no old `actor` column. | DTU+LIVE | P1 |
| QA-012 | How many audit events occurred per activity type this month? | `FROM claroty_audit_logs \| where time > '2026-08-01T00:00:00Z' \| group by activity_name \| count(*) as event_count \| order by event_count desc` | One row per distinct `activity_name` with count. Push-down fires on the `time` predicate (single `greater_or_equal` bound in POST body). | `activity_name` present; GROUP BY works; push-down fires; counts correct. | DTU+LIVE | P1 |
| QA-013 | What notes were recorded for configuration change activities? | `FROM claroty_audit_logs \| where activity_name = 'Configuration Change' \| fields activity_name, actor_user_name, time, comment \| limit 10` | Rows with `comment` field (from `note` TOML col). Column is `comment`, NOT `note`. | Column name `comment` (not `note`); data returns. | DTU+LIVE | P1 |
| QA-014 | Access audit record IDs for deduplication or direct lookup (Tier-1 `metadata_uid`). | `FROM claroty_audit_logs \| where metadata_uid = '<audit-record-id>' \| limit 1` | **KF-05 RESOLVED (2026-08-21):** `audit_logs.id` is now Tier-1 as `metadata_uid` (ocsf_field = "metadata.uid"). The audit record ID is a first-class Arrow String column, directly queryable via `WHERE metadata_uid = '<id>'`. No need to extract from `raw_extensions`. `raw_extensions` for `audit_logs` rows contains only `"category"` key (no `"id"` key). | `metadata_uid` column present in schema; `WHERE metadata_uid = '<id>'` executes without `E-QUERY-038`; `prism_describe claroty_audit_logs` includes a `metadata_uid` String descriptor; no `"id"` key in `raw_extensions`. | DTU+LIVE | P1 |

---

## claroty_devices (QA-015..QA-021)

| QA-ID | Analyst Question | Exact Query | Expected Data Returned | Pass Criterion | DTU/LIVE | Priority |
|---|---|---|---|---|---|---|
| QA-015 | How many OT devices are in our environment? | `FROM claroty_devices \| where device_type = 'OT Device' \| count(*) as ot_count` | Count of devices where `device_type` (from `device_category` TOML col) = 'OT Device'. Column is `device_type`, NOT `device_category`. | Column name `device_type`; filter works; count returns integer; no `device_category` column. | DTU+LIVE | P0 |
| QA-016 | Show me high-risk devices (risk_score above threshold). | `FROM claroty_devices \| where risk_score > '70' \| fields device_uid, device_name, risk_score, device_type \| order by risk_score desc \| limit 20` | Devices with `risk_score` > 70. `risk_score` is String type; comparison behavior documented. | Query executes without column errors; `risk_score` column present; behavior of string-compare vs numeric-compare documented in test notes. | DTU+LIVE | P1 |
| QA-017 | Show me the complete profile of device DEVICE-UID-001. | `FROM claroty_devices \| where device_uid = 'DEVICE-UID-001' \| limit 1` | Full device row. Both `device_type` (category, from `device_category`) and `device_type_label` (type-within-category, from `device_type`, KF-06 correction) are present and distinct. `raw_extensions` blob present with Tier-2 fields. | Both `device_type` AND `device_type_label` present (distinct); no `device_type_name` column (old pre-KF-06 name); `raw_extensions` blob contains Tier-2 keys. | DTU+LIVE | P0 |
| QA-018 | List all device categories and their counts. | `FROM claroty_devices \| group by device_type \| count(*) as device_count \| order by device_count desc` | One row per distinct `device_type` value (device category). Column is `device_type`, NOT `device_category`. | Column name `device_type`; GROUP BY executes; counts computed. | DTU+LIVE | P1 |
| QA-019 | Find which devices have multiple IP addresses. (Tier-2 raw_extensions access — read path) | `FROM claroty_devices \| fields device_uid, device_name, raw_extensions \| limit 20` then parse `raw_extensions["ip_list"]`. | `raw_extensions` JSON blob present. Key `"ip_list"` contains a compact JSON-list string: `"[\"10.0.1.1\",\"10.0.1.2\"]"`, NOT a nested JSON array (EC-016-013-028). Also `"mac_list"`, `"network_list"`, `"vlan_list"` keys present in blob. **Tier-2 WHERE-filtering note (OQ-002 RESOLVED-GATED):** A query like `WHERE json_extract_string(raw_extensions, '$.ip_list') LIKE '%10.0.1.1%'` will be possible once the `json_extract_string` DataFusion ScalarUDF story ships (S-JSON-EXTRACT-UDF-001, depends_on ROUTING-001, v1-chain delivery). Until then, filtering on `raw_extensions` keys is not supported; SELECT access is available. | `raw_extensions` present; `"ip_list"` value IS a String (not a JSON array node); compact-string format confirmed. **Test that `WHERE json_extract_string(raw_extensions, '$.ip_list')` returns an appropriate error pre-UDF** (not a silent incorrect result). | DTU+LIVE | P0 |
| QA-020 | Which devices are retired or decommissioned? | `FROM claroty_devices \| where status_code = true \| fields device_uid, device_name, device_type \| limit 20` | Devices where `status_code` (from `retired` TOML col, Boolean type) = true. Column is `status_code`, NOT `retired`. | Column name `status_code`; boolean filtering works; no `retired` column in schema. | DTU+LIVE | P1 |
| QA-021 | Find devices running Windows 10. | `FROM claroty_devices \| where device_os_name = 'Windows 10' \| fields device_uid, device_name, device_os_name \| limit 20` | Devices where `device_os_name` (from `os_category` TOML col) = 'Windows 10'. Column is `device_os_name`, NOT `os_category`. | Column name `device_os_name`; filter works; no `os_category` column. | DTU+LIVE | P1 |

---

## claroty_device_alert_relations (QA-022..QA-024)

| QA-ID | Analyst Question | Exact Query | Expected Data Returned | Pass Criterion | DTU/LIVE | Priority |
|---|---|---|---|---|---|---|
| QA-022 | Which devices have the most active (unresolved) alerts? | `FROM claroty_device_alert_relations \| where status = 'Unresolved' \| group by device_uid \| count(*) as alert_count \| order by alert_count desc \| limit 20` | Devices with unresolved alerts, ranked by count. `device_uid` and `status` are Tier-1 columns. | `device_uid` and `status` column names present; GROUP BY works; counts correct; no `E-QUERY-038`. | DTU+LIVE | P0 |
| QA-023 | Show me all alerts associated with device DEVICE-UID-001. | `FROM claroty_device_alert_relations \| where device_uid = 'DEVICE-UID-001' \| fields finding_info_uid, status, time, risk_score \| limit 20` | Alert-device relation rows. `finding_info_uid` is the alert ID (KF-07: was `finding_uid` pre-ROUTING-001, was `alert_id` in TOML). Column is `finding_info_uid`, NOT `alert_id` or `finding_uid`. | Column `finding_info_uid` present (not `alert_id` or `finding_uid`); filter by `device_uid` works. | DTU+LIVE | P0 |
| QA-024 | Show the risk score and notes for device-alert associations of alert ALERT-UID-001. | `FROM claroty_device_alert_relations \| where finding_info_uid = 'ALERT-UID-001' \| fields device_uid, risk_score, status, comment \| limit 10` | Relation rows for the specified alert. `risk_score` (from `device_risk_score` TOML) and `comment` (from `alert_note` TOML) populated. | Column names `risk_score` and `comment` present (not `device_risk_score` or `alert_note`); query executes. | DTU+LIVE | P1 |

---

## Cross-Table Investigation Flows (QA-025..QA-027)

These entries model multi-step SOC investigation workflows that span two or three Claroty tables. Each step's query is listed inline. The key validation point is that join keys (`finding_info_uid`, `device_uid`) are consistent across tables.

| QA-ID | Analyst Question / Flow | Steps (queries in order) | Expected Data Returned | Pass Criterion | DTU/LIVE | Priority |
|---|---|---|---|---|---|---|
| QA-025 | **Alert → Devices.** I have a critical alert. Which devices does it affect, and what are their profiles? | Step 1: `FROM claroty_alerts \| where finding_info_uid = '<ALERT-ID>' \| limit 1`. Step 2: `FROM claroty_device_alert_relations \| where finding_info_uid = '<ALERT-ID>' \| fields device_uid, status, risk_score \| limit 20`. Step 3: `FROM claroty_devices \| where device_uid = '<device_uid-from-step-2>' \| limit 1`. | Step 1: alert details. Step 2: device UIDs for affected devices. Step 3: full device profile. Join key `finding_info_uid` is consistent between `claroty_alerts` (KF-03) and `claroty_device_alert_relations` (KF-07); both expose the same OCSF field name post-ROUTING-001. | All three queries execute without `E-QUERY-038`; `finding_info_uid` join key works across tables; `device_uid` from step 2 yields a matching row in step 3; no column name mismatch. | DTU+LIVE | P0 |
| QA-026 | **Device → Alerts.** I have a suspicious device. What alerts has it triggered historically? | Step 1: `FROM claroty_device_alert_relations \| where device_uid = '<DEVICE-UID>' \| fields finding_info_uid, status, time \| limit 20`. Step 2: `FROM claroty_alerts \| where finding_info_uid = '<finding_info_uid-from-step-1>' \| limit 1`. | Step 1: alert UIDs associated with the device. Step 2: full alert details for each alert. Cross-table `finding_info_uid` join works. | `finding_info_uid` is the consistent join key across `claroty_device_alert_relations` and `claroty_alerts` (KF-03/KF-07 parity confirmed); no column name mismatch; both steps execute cleanly. | DTU+LIVE | P0 |
| QA-027 | **Alert + Audit Correlation.** An alert fired at 14:23. What was happening in the audit trail in that 2-hour window? | Step 1: `FROM claroty_alerts \| where status = 'Unresolved' \| where finding_info_title IIN 'unauthorized' \| order by time desc \| limit 5`. Step 2: Capture `time` from step 1 result. Step 3: `FROM claroty_audit_logs \| where time > '<alert_time - 1h>' \| where time < '<alert_time + 1h>' \| order by time asc \| limit 50`. | Step 1: unresolved alerts with "unauthorized" in title (case-insensitive `IIN` operator, ADR-047). Uses `finding_info_title`. Step 3: audit log entries within ±1h of the alert. Push-down fires on both time bounds (AND compound filter in POST body). | `finding_info_title IIN 'unauthorized'` executes without parse error; push-down on audit_log fires with `"operation": "and"`, `"operands"` (not `"conditions"`), both bounds present; cross-signal correlation flow completes without errors. | DTU+LIVE | P0 |

---

## Column Name Quick Reference (POST-ROUTING-001)

Summary of the TOML → Arrow renames that affect analyst queries. Use to verify queries use
the correct column names.

| Table | Old name (or TOML col.name) | POST-ROUTING-001 Arrow name | KF ref |
|---|---|---|---|
| claroty_alerts | `id` | `finding_info_uid` | KF-03 |
| claroty_alerts | `alert_name` | `finding_info_title` | KF-04 |
| claroty_alerts | `updated_time` | `finding_info_modified_time` | KF-12 |
| claroty_alerts | `category`, `alert_type_name`, `devices_count`, `alert_class`, `ot_devices_count` | (→ `raw_extensions`) | KF-08/KF-09/KF-10 |
| claroty_audit_logs | `action` | `activity_name` | — |
| claroty_audit_logs | `user_display_name` | `actor_user_name` | — |
| claroty_audit_logs | `timestamp` | `time` (INDEX, push-down eligible) | — |
| claroty_audit_logs | `username` | `actor_user_uid` | — |
| claroty_audit_logs | `note` | `comment` | — |
| claroty_audit_logs | `id` | `metadata_uid` (Tier-1, String) | KF-05 RESOLVED 2026-08-21: ocsf_field = "metadata.uid"; directly queryable |
| claroty_audit_logs | `category` | (→ `raw_extensions`) | KF-11: ocsf_field removed |
| claroty_devices | `device_category` | `device_type` | — |
| claroty_devices | `device_type` | `device_type_label` | KF-06 |
| claroty_devices | `retired` | `status_code` | — |
| claroty_devices | `os_category` | `device_os_name` | — |
| claroty_devices | 12 Tier-2 fields (`ip_list`, `mac_list`, etc.) | (→ `raw_extensions`) | — |
| claroty_device_alert_relations | `alert_id` | `finding_info_uid` | KF-07 |
| claroty_device_alert_relations | `device_risk_score` | `risk_score` | — |
| claroty_device_alert_relations | `alert_note` | `comment` | — |
| claroty_device_alert_relations | `device_alert_status` | `status` | — |

---

## Item Count

| Subsection | Count |
|---|---|
| claroty_alerts Q&A | 7 (QA-001..QA-007) |
| claroty_audit_logs Q&A | 7 (QA-008..QA-014) |
| claroty_devices Q&A | 7 (QA-015..QA-021) |
| claroty_device_alert_relations Q&A | 3 (QA-022..QA-024) |
| Cross-table flows | 3 (QA-025..QA-027) |
| **Total** | **27** |
