---
document_type: soc-qa-catalog
producer: product-owner
version: "0.3"
project: prism
timestamp: "2026-09-07"
develop_head: "362e4f85"
naming_regime: POST-ROUTING-001 (ocsf_column_naming=true)
scope: >
  Gate 5 companion — 64 real SOC-analyst questions spanning all 14 Claroty xDome tables.
  Supersedes v0.2 (27 questions for 4 tables).
  AUTHORITATIVE 14-table SOC Q&A catalog.
supersedes: >
  v0.2 (2026-08-21) — 27 questions for 4 tables (alerts, audit_logs, devices, device_alert_relations).
  v0.3 (2026-09-07) — 64 questions for all 14 tables.
note: >
  This is an OPEN, human-facing planning artifact. NOT holdout scenarios.
  Do NOT read .factory/holdout-scenarios/ when working with this file.
  All queries use POST-ROUTING-001 Arrow field names.
  Story-gate notice: G2–G6 queries require the respective story merge.
  Enrich safety: enrich nvd(...) is LIVE-safe.
  enrich threat_intel(...) is DTU-bound — NOT live-safe. Omit from live scripts.
traces_to:
  - .factory/objectives/xdome-v1-validation/live-validation-matrix.md
  - .factory/specs/behavioral-contracts/BC-2.16.003-column-to-ocsf-mapping.md
---

# Claroty xDome — SOC-Analyst Q&A Catalog

> **AUTHORITATIVE 14-TABLE VERSION (v0.3).** This document supersedes v0.2 (4-table, 27 questions).
> All 14 Claroty xDome tables covered: 64 analyst questions total.

> **Column names:** All queries use **POST-ROUTING-001** Arrow field names. Running these
> queries against a pre-ROUTING-001 binary will produce `E-QUERY-038` column-not-found errors.

> **Story-gate notice:** Questions for expansion tables (QA-028+) require the respective
> story merge (G1: S-CLAROTY-VULNS-001, G2: S-CLAROTY-OT-EVENTS-001, G3: S-CLAROTY-DEVVULNREL-001,
> G4: S-CLAROTY-SERVERS-001, G5: S-CLAROTY-ORGPOLICY-001, G6: S-CLAROTY-ACLPOLICY-001).

> **Enrich safety:**
> - `enrich nvd(...)` — LIVE-SAFE. Can be included in live SOC scripts.
> - `enrich threat_intel(...)` — DTU-BOUND. **NOT live-safe.** Never include in live scripts.

> **Tier-2 column access:** Direct WHERE predicates on `raw_extensions` sub-keys require the
> `json_extract_string` DataFusion UDF (S-JSON-EXTRACT-UDF-001, gated on ROUTING-001 merge).
> Until that story ships, Tier-2 key access is SELECT-only. Multi-step flows (run query,
> parse `raw_extensions` client-side, run next query) are available in v1.

---

## Distribution by Table

| Table | Q&A entries | IDs | Story gate |
|---|---|---|---|
| claroty_alerts | 7 | QA-001..QA-007 | — (already delivered) |
| claroty_audit_logs | 7 | QA-008..QA-014 | — (already delivered) |
| claroty_devices | 7 | QA-015..QA-021 | — (already delivered) |
| claroty_device_alert_relations | 3 | QA-022..QA-024 | — (already delivered) |
| Cross-table (original 4) | 3 | QA-025..QA-027 | — (already delivered) |
| claroty_vulnerabilities | 4 | QA-028..QA-031 | S-CLAROTY-VULNS-001 (merged) |
| claroty_ot_activity_events | 4 | QA-032..QA-035 | S-CLAROTY-OT-EVENTS-001 |
| claroty_device_vulnerability_relations | 4 | QA-036..QA-039 | S-CLAROTY-DEVVULNREL-001 |
| claroty_servers | 4 | QA-040..QA-043 | S-CLAROTY-SERVERS-001 |
| claroty_server_interfaces | 3 | QA-044..QA-046 | S-CLAROTY-SERVERS-001 |
| claroty_organization_zones | 3 | QA-047..QA-049 | S-CLAROTY-ORGPOLICY-001 |
| claroty_organization_zone_policies | 3 | QA-050..QA-052 | S-CLAROTY-ORGPOLICY-001 |
| claroty_organization_firewall_groups | 3 | QA-053..QA-055 | S-CLAROTY-ORGPOLICY-001 |
| claroty_organization_firewall_policies | 3 | QA-056..QA-058 | S-CLAROTY-ORGPOLICY-001 |
| claroty_organization_acl_policies | 3 | QA-059..QA-061 | S-CLAROTY-ACLPOLICY-001 |
| Cross-table (expansion: vuln/device joins) | 3 | QA-062..QA-064 | S-CLAROTY-DEVVULNREL-001 |
| **Total** | **64** | | |

---

## Section 1: claroty_alerts

### QA-001

**Analyst question:** "Show me all unresolved alerts from the last 24 hours."

```sql
FROM claroty_alerts
| where status = 'Unresolved'
| where time > '2024-07-01T00:00:00Z'
| order by time desc
| limit 20
```

**Expected result:** Rows where `status == 'Unresolved'` and `time` falls within the last 24h window. Ordered most-recent first. `time` is a parseable ISO-8601 datetime (POST-ROUTING-001). `finding_info_uid` (NOT `id`) is the unique alert identifier.

**Pass criterion:** All returned rows have `status == 'Unresolved'`. `time` is within range. No column named `id` in the result schema.

**Sensor:** claroty_alerts. **DTU:** DTU+LIVE.

---

### QA-002

**Analyst question:** "What's the breakdown of alert statuses across the tenant?"

```sql
FROM claroty_alerts
| group by status
| count(*) as alert_count
| order by alert_count desc
```

**Expected result:** One row per distinct `status` value. No casing-duplicate buckets.

**Pass criterion:** Multiple distinct status values returned; total row count matches total distinct statuses in tenant.

**Sensor:** claroty_alerts. **DTU:** DTU+LIVE.

---

### QA-003

**Analyst question:** "Find all alerts matching 'Unauthorized Communication' in the alert title."

```sql
FROM claroty_alerts
| where finding_info_title LIKE '%Unauthorized Communication%'
| order by time desc
| limit 10
```

**Expected result:** Rows where `finding_info_title` (from `alert_name`, KF-04 corrected) contains the search phrase. `finding_info_title` is a Tier-1 first-class column — directly filterable.

**Pass criterion:** No `E-QUERY-038` on `finding_info_title`; filter executes successfully; values match search phrase.

**Sensor:** claroty_alerts. **DTU:** DTU+LIVE.

---

### QA-004

**Analyst question:** "How many OT devices were affected by alerts in the last 7 days?"

```sql
FROM claroty_alerts
| where time > '2024-06-24T00:00:00Z'
| fields finding_info_uid, raw_extensions
| limit 50
```

Then parse `raw_extensions["ot_devices_count"]` client-side and aggregate.

**Expected result:** `raw_extensions` column present for each row. JSON blob contains `"ot_devices_count"` key. (Direct GROUP BY on raw_extensions sub-keys requires `json_extract_string` UDF, S-JSON-EXTRACT-UDF-001.)

**Pass criterion:** `raw_extensions` present; `"ot_devices_count"` key accessible in JSON blob.

**Sensor:** claroty_alerts. **DTU:** DTU+LIVE.

---

### QA-005

**Analyst question:** "What are the most recent alert titles and their current statuses?"

```sql
FROM claroty_alerts
| fields finding_info_title, status, time
| order by time desc
| limit 20
```

**Expected result:** Three-column result: `finding_info_title` (from `alert_name`), `status`, `time` (from `detected_time`). Most recent alerts listed first.

**Pass criterion:** Three columns present with correct POST-ROUTING-001 names; ordered correctly.

**Sensor:** claroty_alerts. **DTU:** DTU+LIVE.

---

### QA-006

**Analyst question:** "Show me the full alert details for a specific alert ID."

Step 1 (obtain ID): `FROM claroty_alerts | fields finding_info_uid | limit 1`
Step 2 (fetch full record): `FROM claroty_alerts | where finding_info_uid = '<id-from-step-1>' | limit 1`

**Expected result:** Single row returned. All Tier-1 columns populated: `finding_info_uid`, `status`, `time`, `finding_info_modified_time`, `message`, `finding_info_title`. `raw_extensions` present with Tier-2 keys.

**Pass criterion:** Step 2 returns exactly 1 row; all 6 Tier-1 columns present; `raw_extensions` parseable JSON.

**Sensor:** claroty_alerts. **DTU:** DTU+LIVE.

---

### QA-007

**Analyst question:** "Are there any alerts updated more recently than they were detected (indicating analyst activity)?"

```sql
FROM claroty_alerts
| where finding_info_modified_time > time
| fields finding_info_uid, time, finding_info_modified_time, status
| limit 20
```

**Expected result:** Rows where `finding_info_modified_time` (from `updated_time`, KF-12) is after `time` (from `detected_time`), indicating analyst interaction.

**Pass criterion:** Both `time` and `finding_info_modified_time` columns present; filter executes; returned rows satisfy the condition.

**Sensor:** claroty_alerts. **DTU:** DTU+LIVE.

---

## Section 2: claroty_audit_logs

### QA-008

**Analyst question:** "Show me all admin actions in the last 7 days."

```sql
FROM claroty_audit_logs
| where time > '2024-06-24T00:00:00Z'
| where actor_user_uid LIKE '%admin%'
| order by time desc
| limit 20
```

**Expected result:** Rows where `actor_user_uid` (from `username`) contains 'admin'. `time` (from `timestamp`, INDEX column — push-down eligible) is within range. Push-down fires for the `time` predicate.

**Pass criterion:** Rows returned with `time` within range; prism fetch log shows `filter_by.field = "timestamp"` in POST body to `/api/v1/audit_log/get`.

**Sensor:** claroty_audit_logs. **DTU:** DTU+LIVE.

---

### QA-009

**Analyst question:** "What types of actions are being taken in the system? Give me a breakdown."

```sql
FROM claroty_audit_logs
| group by activity_name
| count(*) as action_count
| order by action_count desc
| limit 20
```

**Expected result:** One row per distinct action type. `activity_name` (from `action`) is the Tier-1 column.

**Pass criterion:** No `E-QUERY-038` on `activity_name`; grouped result with action counts.

**Sensor:** claroty_audit_logs. **DTU:** DTU+LIVE.

---

### QA-010

**Analyst question:** "Find all audit events for a specific user to investigate potential insider threat activity."

```sql
FROM claroty_audit_logs
| where actor_user_uid = 'suspect.user@example.com'
| order by time desc
| limit 50
```

**Expected result:** All audit events for the specified user, ordered by time descending. `actor_user_uid` (from `username`) and `actor_user_name` (from `user_display_name`) are both Tier-1 columns.

**Pass criterion:** Filter works on `actor_user_uid`; results ordered correctly.

**Sensor:** claroty_audit_logs. **DTU:** DTU+LIVE.

---

### QA-011

**Analyst question:** "Retrieve the audit record ID so I can cross-reference it with an external ticket."

```sql
FROM claroty_audit_logs
| fields metadata_uid, activity_name, time, actor_user_name
| order by time desc
| limit 10
```

**Expected result:** `metadata_uid` (from `id`, KF-05 RESOLVED: Tier-1 column) present as first-class column in the result. Directly queryable without `raw_extensions` access.

**Pass criterion:** `metadata_uid` column present; value is a non-empty string audit record ID; no `E-QUERY-038`.

**Sensor:** claroty_audit_logs. **DTU:** DTU+LIVE.

---

### QA-012

**Analyst question:** "Did any configuration changes happen between midnight and 6am last night? (Suspicious off-hours window.)"

```sql
FROM claroty_audit_logs
| where time > '2024-07-01T00:00:00Z'
| where time < '2024-07-01T06:00:00Z'
| where activity_name IEQ 'configuration change'
| order by time asc
| limit 50
```

**Expected result:** Audit events in the 00:00–06:00 window with configuration change activity. Push-down fires (both time bounds pushed to xDome `filter_by.operation = "and"` with `"operands": [{gte}, {lte}]`).

**Pass criterion:** Prism fetch log shows `"operation": "and"` with `"operands"` key (PD-004 pattern). Rows within time range.

**Sensor:** claroty_audit_logs. **DTU:** DTU+LIVE.

---

### QA-013

**Analyst question:** "What was the audit note (comment) for a recent sensitive action?"

```sql
FROM claroty_audit_logs
| where comment IS NOT NULL
| fields metadata_uid, activity_name, comment, actor_user_name, time
| limit 10
```

**Expected result:** `comment` (from `note`) is a Tier-1 column. Rows where comment is populated.

**Pass criterion:** `comment` column present; IS NOT NULL filter works; `metadata_uid` column also present in result (Tier-1, KF-05 resolved).

**Sensor:** claroty_audit_logs. **DTU:** DTU+LIVE.

---

### QA-014

**Analyst question:** "Show me all audit log details available for the same action — see what category the system assigned."

```sql
FROM claroty_audit_logs
| fields metadata_uid, activity_name, time, actor_user_name, comment, raw_extensions
| limit 5
```

Then parse `raw_extensions["category"]` client-side.

**Expected result:** `raw_extensions` present; JSON blob contains `"category"` key. No `"metadata_uid"` key in `raw_extensions` (it is a Tier-1 column, not in the blob).

**Pass criterion:** `raw_extensions` column present; `"category"` key accessible in blob; `"metadata_uid"` key NOT in raw_extensions blob; `metadata_uid` first-class column present.

**Sensor:** claroty_audit_logs. **DTU:** DTU+LIVE.

---

## Section 3: claroty_devices

### QA-015

**Analyst question:** "Show me all OT devices — I need to know what's in the environment."

```sql
FROM claroty_devices
| where device_type = 'OT Device'
| fields device_uid, device_name, device_type_label, risk_score
| limit 50
```

**Expected result:** Rows with `device_type == 'OT Device'`. `device_type_label` (from `device_type`, KF-06 corrected) is distinct from `device_type` (from `device_category`).

**Pass criterion:** Both `device_type` and `device_type_label` columns present; no `device_type_name` column.

**Sensor:** claroty_devices. **DTU:** DTU+LIVE.

---

### QA-016

**Analyst question:** "Find all devices with a risk score above 80. Who are my high-risk assets?"

```sql
FROM claroty_devices
| where risk_score > '80'
| fields device_uid, device_name, device_type, risk_score
| order by risk_score desc
| limit 20
```

**Expected result:** Devices filtered by `risk_score` (String column — comparison semantics depend on string ordering; numeric string comparison may be needed via `json_extract_string` for proper numeric ordering in v1+). In v1, returns rows where `risk_score` sorts above '80' as a string.

**Pass criterion:** No `E-QUERY-038` on `risk_score`; filter executes; result contains `device_uid` and `device_name`.

**Sensor:** claroty_devices. **DTU:** DTU+LIVE.

---

### QA-017

**Analyst question:** "Are there devices marked as retired? I want to confirm decommissioning."

```sql
FROM claroty_devices
| where status_code = true
| fields device_uid, device_name, device_type, status_code
| limit 20
```

**Expected result:** `status_code` (from `retired`, Boolean) = true indicates a retired/decommissioned device (Claroty semantic: `retired = true` → `status_code = true`).

**Pass criterion:** Boolean filter on `status_code` works; returned rows have `status_code == true`.

**Sensor:** claroty_devices. **DTU:** DTU+LIVE.

---

### QA-018

**Analyst question:** "List the IP addresses associated with a specific device."

Step 1: `FROM claroty_devices | where device_name = 'TARGET-PLC-001' | fields device_uid, raw_extensions | limit 1`
Step 2: Parse `raw_extensions["ip_list"]` client-side.

**Expected result:** `raw_extensions["ip_list"]` is a compact JSON-list string: `"[\"10.0.1.1\",\"10.0.1.2\"]"`. NOT a nested JSON array node.

**Pass criterion:** `raw_extensions` present; `"ip_list"` key accessible; value IS a String (compact JSON-list), not a nested array.

**Sensor:** claroty_devices. **DTU:** DTU+LIVE.

---

### QA-019

**Analyst question:** "Describe the devices table schema so I know what columns are available."

```sql
prism_describe claroty_devices
```

**Expected result:** 8 Tier-1 column descriptors + 1 `raw_extensions` descriptor + 2 synthesized descriptors (`class_uid`, `_sensor`). `device_type_label` (KF-06 corrected) is a Tier-1 column. `class_uid == 5001` (inventory_info). No individual Tier-2 column descriptors.

**Pass criterion:** 11 total descriptors (8 + 1 + 2); `device_type_label` present; `device_type_name` absent; `class_uid` and `_sensor` synthesized descriptors present.

**Sensor:** claroty_devices. **DTU:** DTU+LIVE.

---

### QA-020

**Analyst question:** "How many devices are online right now?"

Step 1: `FROM claroty_devices | fields device_uid, raw_extensions | limit 5000`
Step 2: Parse `raw_extensions["is_online"]` client-side; count `true` values.

**Expected result:** `raw_extensions["is_online"]` is a boolean value in the JSON blob. Direct WHERE filter requires `json_extract_string` UDF. Multi-step approach works in v1.

**Pass criterion:** `raw_extensions` present; `"is_online"` key accessible in blob; boolean value parseable.

**Sensor:** claroty_devices. **DTU:** DTU+LIVE.

---

### QA-021

**Analyst question:** "Show me all devices on a specific site."

Step 1: `FROM claroty_devices | fields device_uid, device_name, raw_extensions | limit 200`
Step 2: Parse `raw_extensions["site_name"]` client-side; filter for target site.

**Expected result:** `raw_extensions["site_name"]` accessible for each device. Multi-step works in v1 (direct WHERE on Tier-2 key requires json_extract_string UDF).

**Pass criterion:** `raw_extensions` present; `"site_name"` key accessible in blob.

**Sensor:** claroty_devices. **DTU:** DTU+LIVE.

---

## Section 4: claroty_device_alert_relations

### QA-022

**Analyst question:** "Which devices are associated with active high-risk alerts?"

Step 1: `FROM claroty_alerts | where status = 'Unresolved' | fields finding_info_uid | limit 20`
Step 2: For an alert ID from step 1: `FROM claroty_device_alert_relations | where finding_info_uid = '<id>' | fields device_uid, risk_score | limit 10`
Step 3: `FROM claroty_devices | where device_uid = '<device_uid>' | limit 1`

**Expected result:** Step 2 uses `finding_info_uid` (KF-07 corrected; was `finding_uid`). `device_uid` is a Tier-1 column on `claroty_device_alert_relations`.

**Pass criterion:** No `E-QUERY-038` on `finding_info_uid` in step 2; `device_uid` from step 2 resolves in step 3.

**Sensor:** claroty_device_alert_relations + claroty_alerts + claroty_devices. **DTU:** DTU+LIVE.

---

### QA-023

**Analyst question:** "What's the risk score of the device involved in this specific alert?"

```sql
FROM claroty_device_alert_relations
| where finding_info_uid = 'ALERT-123'
| fields device_uid, risk_score, status
| limit 5
```

**Expected result:** `risk_score` (from `device_risk_score`) and `status` (from `device_alert_status`) are both Tier-1 columns directly accessible.

**Pass criterion:** `risk_score` and `status` columns present; no `E-QUERY-038`.

**Sensor:** claroty_device_alert_relations. **DTU:** DTU+LIVE.

---

### QA-024

**Analyst question:** "Has there been any analyst commentary on this device-alert relationship?"

```sql
FROM claroty_device_alert_relations
| where finding_info_uid = 'ALERT-123'
| where comment IS NOT NULL
| fields device_uid, comment, status
| limit 5
```

**Expected result:** `comment` (from `alert_note`) is a Tier-1 column. IS NOT NULL filter works.

**Pass criterion:** `comment` column present; IS NOT NULL filter executes; no `E-QUERY-038`.

**Sensor:** claroty_device_alert_relations. **DTU:** DTU+LIVE.

---

## Section 5: Cross-Table Flows (Original 4 Tables)

### QA-025

**Analyst question:** "Walk me through a full incident investigation: go from an unresolved alert → find affected devices → get device details."

Step 1: `FROM claroty_alerts | where status = 'Unresolved' | fields finding_info_uid, finding_info_title | limit 1`
Step 2: `FROM claroty_device_alert_relations | where finding_info_uid = '<id>' | fields device_uid, risk_score | limit 5`
Step 3: `FROM claroty_devices | where device_uid = '<device_uid>' | fields device_name, device_type, device_os_name | limit 1`

**Expected result:** All three steps execute without `E-QUERY-038`. `finding_info_uid` join key works (KF-07 corrected). Device profile retrieved in step 3.

**Pass criterion:** Three-step flow completes; data threads correctly across tables.

**Sensor:** All 3 tables. **DTU:** DTU+LIVE.

---

### QA-026

**Analyst question:** "Find all audit events around the time an alert was detected — looking for who might have made a change that caused it."

Step 1: `FROM claroty_alerts | where finding_info_uid = 'ALERT-123' | fields time | limit 1`
Step 2: Compute `[time - 30min, time + 30min]` window client-side.
Step 3: `FROM claroty_audit_logs | where time > '<t-30min>' | where time < '<t+30min>' | order by time asc | limit 50`

**Expected result:** Step 3 triggers push-down for both time bounds (PD-004 pattern). Audit events in the ±30-minute window around the alert.

**Pass criterion:** Step 3 prism fetch log shows `"operation": "and"` with `"operands"` containing both bounds.

**Sensor:** claroty_alerts + claroty_audit_logs. **DTU:** DTU+LIVE.

---

### QA-027

**Analyst question:** "What's the overall health picture? Give me alert count, audit event count, and device count."

Step 1: `FROM claroty_alerts | count(*) as total_alerts`
Step 2: `FROM claroty_audit_logs | count(*) as total_audit_events`
Step 3: `FROM claroty_devices | count(*) as total_devices`

**Expected result:** Three separate count queries, each returning a single-row aggregate. All execute without error.

**Pass criterion:** Three count queries each return exactly 1 row with an integer count; no errors.

**Sensor:** All 3 tables. **DTU:** DTU+LIVE.

---

## Section 6: claroty_vulnerabilities

*Story gate: S-CLAROTY-VULNS-001 (merged). DTU: YES (prism-dtu-claroty). LIVE: YES.*

### QA-028

**Analyst question:** "Show me all vulnerabilities in our environment that are on the CISA Known Exploited Vulnerabilities list."

```sql
FROM claroty_vulnerabilities
| fields finding_info_title, message, raw_extensions
| limit 100
```

Then parse `raw_extensions["is_known_exploited"]` client-side; filter for `true` values.

**Expected result:** `finding_info_title` (from `name`) is the CVE ID or advisory title. `raw_extensions["is_known_exploited"]` is accessible in the JSON blob. (Direct WHERE on `is_known_exploited` requires `json_extract_string` UDF, S-JSON-EXTRACT-UDF-001.)

**Pass criterion:** `finding_info_title` and `raw_extensions` columns present; `"is_known_exploited"` key accessible in blob; no `E-QUERY-038`.

**Sensor:** claroty_vulnerabilities. **DTU:** DTU+LIVE.

---

### QA-029

**Analyst question:** "Find all CVE-identified vulnerabilities and sort them by name."

```sql
FROM claroty_vulnerabilities
| where finding_info_title LIKE 'CVE-%'
| order by finding_info_title asc
| limit 50
```

**Expected result:** Rows where `finding_info_title` matches CVE pattern. Sorted alphabetically (which approximates CVE chronological order within the same year-prefix).

**Pass criterion:** Filter on `finding_info_title` works; results ordered; values match CVE format (e.g., `CVE-2021-31998`).

**Sensor:** claroty_vulnerabilities. **DTU:** DTU+LIVE.

---

### QA-030

**Analyst question:** "What's the description for a specific CVE that was flagged?"

```sql
FROM claroty_vulnerabilities
| where finding_info_title = 'CVE-2021-31998'
| fields finding_info_title, message
| limit 1
```

**Expected result:** `message` (from `description`) contains the human-readable vulnerability description. Single row returned.

**Pass criterion:** `message` column present with non-empty description; `finding_info_title` matches query value.

**Sensor:** claroty_vulnerabilities. **DTU:** DTU+LIVE.

---

### QA-031

**Analyst question:** "Give me the raw vulnerability data for NVD-sourced findings — I want the CVSSv3 score and exploit probability."

```sql
FROM claroty_vulnerabilities
| fields finding_info_title, raw_extensions
| limit 20
```

Then parse `raw_extensions["source_name"]`, `raw_extensions["cvss_v3_score"]`, `raw_extensions["epss_score"]` client-side; filter for `source_name == "NVD"`.

**Expected result:** `raw_extensions` JSON blob contains `"source_name"` (e.g., "NVD"), `"cvss_v3_score"` (numeric), and `"epss_score"` (numeric 0.0–1.0) keys.

**Pass criterion:** All three keys accessible in `raw_extensions` blob; CVSS score is a numeric value; EPSS score is a fractional value.

**Sensor:** claroty_vulnerabilities. **DTU:** DTU+LIVE.

---

## Section 7: claroty_ot_activity_events

*Story gate: S-CLAROTY-OT-EVENTS-001. DTU: NO — LIVE-REQUIRED.*

### QA-032

**Analyst question:** "What OT configuration changes happened in the last 24 hours? I'm looking for unauthorized firmware or config uploads."

```sql
FROM claroty_ot_activity_events
| where activity_name = 'Configuration Upload'
| where time > '2024-07-01T00:00:00Z'
| order by time desc
| limit 20
```

**Expected result:** `activity_name` (from `event_type`) filters on 'Configuration Upload'. `time` (from `detection_time`) is a parseable datetime. Note: unlike `audit_logs`, `ot_activity_events` has no push-down — `time` filter is in-engine (DataFusion post-filter).

**Pass criterion:** `activity_name` and `time` columns present; no `E-QUERY-038`; results filtered correctly.

**Sensor:** claroty_ot_activity_events. **DTU:** LIVE-REQUIRED.

---

### QA-033

**Analyst question:** "Find all Modbus protocol events — we suspect unauthorized Modbus communication."

```sql
FROM claroty_ot_activity_events
| fields finding_info_uid, time, activity_name, raw_extensions
| limit 50
```

Then parse `raw_extensions["protocol"]` client-side; filter for `protocol == "Modbus"`.

**Expected result:** `raw_extensions["protocol"]` contains the OT protocol string ("Modbus", "CIP", "DNP3", etc.). `finding_info_uid` (from integer `event_id`, normalized to String) is present as Tier-1.

**Pass criterion:** `finding_info_uid` is String type; `raw_extensions` blob contains `"protocol"` key; value is a string OT protocol name.

**Sensor:** claroty_ot_activity_events. **DTU:** LIVE-REQUIRED.

---

### QA-034

**Analyst question:** "Which OT events have related alerts? I need to understand the correlation."

```sql
FROM claroty_ot_activity_events
| fields finding_info_uid, time, activity_name, raw_extensions
| limit 30
```

Then parse `raw_extensions["related_alert_ids"]` client-side; filter for non-empty arrays.

**Expected result:** `raw_extensions["related_alert_ids"]` is a compact JSON-list string (e.g., `"[\"ALERT-001\",\"ALERT-002\"]"`). NOT a nested JSON array node.

**Pass criterion:** `"related_alert_ids"` key accessible in blob; value IS a String (compact JSON-list); parseable as a list of alert IDs.

**Sensor:** claroty_ot_activity_events. **DTU:** LIVE-REQUIRED.

---

### QA-035

**Analyst question:** "Show me network-layer context for an OT event — source/dest IPs and protocol."

```sql
FROM claroty_ot_activity_events
| where finding_info_uid = '<event-id>'
| fields finding_info_uid, time, activity_name, raw_extensions
| limit 1
```

Then parse `raw_extensions["source_ip"]`, `raw_extensions["dest_ip"]`, `raw_extensions["protocol"]`, `raw_extensions["dest_port"]`.

**Expected result:** All four network-context fields accessible in `raw_extensions` blob. `source_ip` and `dest_ip` are IP address strings. `protocol` is a string. `dest_port` is an integer or integer string.

**Pass criterion:** All four keys present in `raw_extensions` blob with non-null values for a network event.

**Sensor:** claroty_ot_activity_events. **DTU:** LIVE-REQUIRED.

---

## Section 8: claroty_device_vulnerability_relations

*Story gate: S-CLAROTY-DEVVULNREL-001. DTU: NO — LIVE-REQUIRED. Depends on S-CLAROTY-VULNS-001.*

### QA-036

**Analyst question:** "Which devices in our environment are affected by CVE-2021-31998?"

```sql
FROM claroty_device_vulnerability_relations
| where finding_info_title = 'CVE-2021-31998'
| fields finding_info_title, time, raw_extensions
| limit 20
```

Then parse `raw_extensions["device_uid"]` client-side for each affected device.

**Expected result:** `finding_info_title` (from `vulnerability_name`) is the join key. `raw_extensions["device_uid"]` contains the affected device's UID (composite PK element, Tier-2). `time` (from `device_vulnerability_detection_date`) is when this device-CVE relationship was detected.

**Pass criterion:** `finding_info_title` column present; filter works; `raw_extensions` contains `"device_uid"` key.

**Sensor:** claroty_device_vulnerability_relations. **DTU:** LIVE-REQUIRED.

---

### QA-037

**Analyst question:** "How many devices are affected by this vulnerability? Give me a quick triage count."

```sql
FROM claroty_device_vulnerability_relations
| where finding_info_title = 'CVE-2021-31998'
| count(*) as affected_device_count
```

**Expected result:** Single-row aggregate. Count reflects the number of device-vulnerability relationship rows for this CVE.

**Pass criterion:** count(*) executes on `claroty_device_vulnerability_relations`; single row returned with integer count.

**Sensor:** claroty_device_vulnerability_relations. **DTU:** LIVE-REQUIRED.

---

### QA-038

**Analyst question:** "Is this CVE on the CISA KEV list for the specific devices in our environment?"

```sql
FROM claroty_device_vulnerability_relations
| fields finding_info_title, raw_extensions
| limit 30
```

Then parse `raw_extensions["vulnerability_is_known_exploited"]` client-side.

**Expected result:** `raw_extensions["vulnerability_is_known_exploited"]` is the CISA KEV indicator per device-vulnerability pair. Also parse `raw_extensions["vulnerability_cvss_v3_score"]` for severity triage.

**Pass criterion:** Both `"vulnerability_is_known_exploited"` and `"vulnerability_cvss_v3_score"` keys accessible in `raw_extensions` blob.

**Sensor:** claroty_device_vulnerability_relations. **DTU:** LIVE-REQUIRED.

---

### QA-039

**Analyst question:** "Has the patch been applied for this device-vulnerability pair?"

```sql
FROM claroty_device_vulnerability_relations
| where finding_info_title = 'CVE-2021-31998'
| fields finding_info_title, time, raw_extensions
| limit 5
```

Then parse `raw_extensions["patch_status"]`, `raw_extensions["patch_install_date"]`, `raw_extensions["device_vulnerability_resolution_date"]`.

**Expected result:** `raw_extensions["patch_status"]` is a string (e.g., "Patched", "Not Patched"). `raw_extensions["patch_install_date"]` and `raw_extensions["device_vulnerability_resolution_date"]` are ISO-8601 datetime strings or null.

**Pass criterion:** Three keys accessible in `raw_extensions` blob; `patch_status` is a string; date fields are datetime strings or null (not absent entirely).

**Sensor:** claroty_device_vulnerability_relations. **DTU:** LIVE-REQUIRED.

---

## Section 9: claroty_servers

*Story gate: S-CLAROTY-SERVERS-001. DTU: NO — LIVE-REQUIRED.*

### QA-040

**Analyst question:** "List all collection servers and their current operational status."

```sql
FROM claroty_servers
| fields device_name, status_code
| order by device_name asc
| limit 50
```

**Expected result:** `device_name` (from `server_name`) and `status_code` (from `server_status`) are the two Tier-1 columns. No `server_name` column in result.

**Pass criterion:** `device_name` and `status_code` columns present; no `server_name` column; no `E-QUERY-038`.

**Sensor:** claroty_servers. **DTU:** LIVE-REQUIRED.

---

### QA-041

**Analyst question:** "Which servers have the most open incidents? I need to prioritize maintenance."

```sql
FROM claroty_servers
| fields device_name, raw_extensions
| limit 50
```

Then parse `raw_extensions["num_of_open_incidents"]` client-side; sort descending.

**Expected result:** `raw_extensions["num_of_open_incidents"]` is an integer count per server.

**Pass criterion:** `"num_of_open_incidents"` key accessible in `raw_extensions` blob; value is a parseable integer.

**Sensor:** claroty_servers. **DTU:** LIVE-REQUIRED.

---

### QA-042

**Analyst question:** "What server has been running the longest without a restart? High uptime can mean delayed security patching."

```sql
FROM claroty_servers
| fields device_name, raw_extensions
| limit 50
```

Then parse `raw_extensions["uptime_days"]` client-side; sort descending.

**Expected result:** `raw_extensions["uptime_days"]` is a Float (e.g., `667.233661`) — fractional days, not integer. Confirmed by OpenAPI spec example value.

**Pass criterion:** `"uptime_days"` key accessible in `raw_extensions` blob; value IS a JSON number with a decimal component (Float), not an integer.

**Sensor:** claroty_servers. **DTU:** LIVE-REQUIRED.

---

### QA-043

**Analyst question:** "Find all servers at site 'SITE-001' for a site-specific investigation."

```sql
FROM claroty_servers
| fields device_name, status_code, raw_extensions
| limit 100
```

Then parse `raw_extensions["site_id"]` client-side; filter for target site.

**Expected result:** `raw_extensions["site_id"]` contains the site identifier string.

**Pass criterion:** `"site_id"` key accessible in `raw_extensions` blob; value is a site identifier string.

**Sensor:** claroty_servers. **DTU:** LIVE-REQUIRED.

---

## Section 10: claroty_server_interfaces

*Story gate: S-CLAROTY-SERVERS-001. DTU: NO — LIVE-REQUIRED. Separate endpoint from claroty_servers.*

### QA-044

**Analyst question:** "List all network interfaces for server 'SERVER-001'."

```sql
FROM claroty_server_interfaces
| where device_name = 'SERVER-001'
| fields device_name, status_code, raw_extensions
| limit 20
```

**Expected result:** `device_name` (from `server_name`) is the join key to `claroty_servers`. Rows represent individual interfaces for the specified server. `raw_extensions["interface_name"]` identifies each interface.

**Pass criterion:** `device_name` column present; filter works; `raw_extensions` contains `"interface_name"` key.

**Sensor:** claroty_server_interfaces. **DTU:** LIVE-REQUIRED.

---

### QA-045

**Analyst question:** "Which interfaces are actively monitoring traffic? I need to audit coverage."

```sql
FROM claroty_server_interfaces
| fields device_name, status_code, raw_extensions
| limit 100
```

Then parse `raw_extensions["is_monitored"]` client-side; filter for `true`.

**Expected result:** `raw_extensions["is_monitored"]` is a Boolean value in the JSON blob. Interfaces with `is_monitored == true` are actively capturing traffic.

**Pass criterion:** `"is_monitored"` key accessible in `raw_extensions` blob; value is a Boolean (true/false), not a string.

**Sensor:** claroty_server_interfaces. **DTU:** LIVE-REQUIRED.

---

### QA-046

**Analyst question:** "Find the interface with IP address 10.0.1.100 — which server is it on?"

```sql
FROM claroty_server_interfaces
| fields device_name, status_code, raw_extensions
| limit 500
```

Then parse `raw_extensions["ip_address"]` client-side; filter for target IP.

**Expected result:** `raw_extensions["ip_address"]` contains the interface IP address string. `device_name` identifies the owning server for cross-reference with `claroty_servers`.

**Pass criterion:** `"ip_address"` key accessible in `raw_extensions` blob; `device_name` present for server cross-reference.

**Sensor:** claroty_server_interfaces. **DTU:** LIVE-REQUIRED.

---

## Section 11: claroty_organization_zones

*Story gate: S-CLAROTY-ORGPOLICY-001. DTU: NO — LIVE-REQUIRED. BC: BC-2.16.020 (Zone Domain).*

### QA-047

**Analyst question:** "Show me all active network zones — I want a map of the segmented environment."

```sql
FROM claroty_organization_zones
| where status_code = true
| fields name, comment, actor_user_name, raw_extensions
| order by name asc
| limit 50
```

**Expected result:** `name` (from `zone_name`, PK, REQUIRED), `comment` (from `zone_description`), `status_code` (from `enabled`, Boolean), `actor_user_name` (from `updated_by`) are all Tier-1 columns. Boolean filter on `status_code` works.

**Pass criterion:** All four Tier-1 columns present; Boolean filter works; no `E-QUERY-038`.

**Sensor:** claroty_organization_zones. **DTU:** LIVE-REQUIRED.

---

### QA-048

**Analyst question:** "How many devices are attributed to each network zone? I need coverage visibility."

```sql
FROM claroty_organization_zones
| fields name, raw_extensions
| limit 50
```

Then parse `raw_extensions["attributed_devices"]` client-side.

**Expected result:** `raw_extensions["attributed_devices"]` is an integer count of devices matched by the zone's device conditions. `raw_extensions["device_conditions"]` is a compact JSON-list string of device filter condition objects.

**Pass criterion:** `"attributed_devices"` key accessible in `raw_extensions` blob; value is a numeric count; `"device_conditions"` key also accessible and is a String (compact JSON-list).

**Sensor:** claroty_organization_zones. **DTU:** LIVE-REQUIRED.

---

### QA-049

**Analyst question:** "Who last modified each zone? I need an audit trail for compliance."

```sql
FROM claroty_organization_zones
| fields name, actor_user_name
| order by name asc
| limit 50
```

**Expected result:** `actor_user_name` (from `updated_by`) is a Tier-1 column with the last modifier's display name.

**Pass criterion:** `actor_user_name` column present; values are non-empty usernames.

**Sensor:** claroty_organization_zones. **DTU:** LIVE-REQUIRED.

---

## Section 12: claroty_organization_zone_policies

*Story gate: S-CLAROTY-ORGPOLICY-001. DTU: NO — LIVE-REQUIRED. BC: BC-2.16.020 (Zone Domain).*

### QA-050

**Analyst question:** "List all DENY zone policies — these represent blocked communication paths."

```sql
FROM claroty_organization_zone_policies
| where activity_name = 'Deny'
| fields name, activity_name, comment, actor_user_name
| limit 50
```

**Expected result:** `activity_name` (from `policy_action`) = 'Deny'. `name` (from `policy_name`, PK, REQUIRED) identifies the policy. All four Tier-1 columns present.

**Pass criterion:** Filter on `activity_name` works; results have 'Deny' value; no `E-QUERY-038`.

**Sensor:** claroty_organization_zone_policies. **DTU:** LIVE-REQUIRED.

---

### QA-051

**Analyst question:** "Which zone policies are configured to generate alerts? I want to know my detection coverage."

```sql
FROM claroty_organization_zone_policies
| fields name, activity_name, raw_extensions
| limit 50
```

Then parse `raw_extensions["should_generate_alerts"]` client-side; filter for `true`.

**Expected result:** `raw_extensions["should_generate_alerts"]` is a Boolean in the JSON blob. `raw_extensions["alert_use_case"]` describes the alert scenario.

**Pass criterion:** `"should_generate_alerts"` and `"alert_use_case"` keys accessible in `raw_extensions` blob.

**Sensor:** claroty_organization_zone_policies. **DTU:** LIVE-REQUIRED.

---

### QA-052

**Analyst question:** "What zone pairs does a specific policy cover? I need to understand traffic flow restrictions."

```sql
FROM claroty_organization_zone_policies
| where name = 'POLICY-BLOCK-IT-OT'
| fields name, activity_name, raw_extensions
| limit 1
```

Then parse `raw_extensions["applied_zone_pairs"]` and `raw_extensions["communication_conditions"]` client-side.

**Expected result:** `raw_extensions["applied_zone_pairs"]` is a compact JSON-list string of `{src_zone, dst_zone}` pair objects. `raw_extensions["communication_conditions"]` is also a compact JSON-list string.

**Pass criterion:** Both keys accessible in `raw_extensions` blob; values are String (compact JSON-list), parseable as arrays of objects.

**Sensor:** claroty_organization_zone_policies. **DTU:** LIVE-REQUIRED.

---

## Section 13: claroty_organization_firewall_groups

*Story gate: S-CLAROTY-ORGPOLICY-001. DTU: NO — LIVE-REQUIRED. BC: BC-2.16.021 (Firewall Domain).*

### QA-053

**Analyst question:** "List all active firewall groups — what device groups have firewall policies applied?"

```sql
FROM claroty_organization_firewall_groups
| where status_code = true
| fields name, comment, actor_user_name
| order by name asc
| limit 50
```

**Expected result:** Structurally symmetric to `claroty_organization_zones` (QA-047). `name` (from `firewall_group_name`, REQUIRED), `comment` (from `firewall_group_description`), `status_code` (from `enabled`, Boolean), `actor_user_name` (from `updated_by`) are Tier-1 columns.

**Pass criterion:** Four Tier-1 columns present; Boolean filter works; no `E-QUERY-038`. `class_uid == 3004`.

**Sensor:** claroty_organization_firewall_groups. **DTU:** LIVE-REQUIRED.

---

### QA-054

**Analyst question:** "How many devices are in each firewall group? Coverage audit."

```sql
FROM claroty_organization_firewall_groups
| fields name, raw_extensions
| limit 50
```

Then parse `raw_extensions["attributed_devices"]` client-side.

**Expected result:** `raw_extensions["attributed_devices"]` is an integer device count. Same pattern as zone attributed_devices (QA-048).

**Pass criterion:** `"attributed_devices"` key accessible in `raw_extensions` blob; integer value parseable.

**Sensor:** claroty_organization_firewall_groups. **DTU:** LIVE-REQUIRED.

---

### QA-055

**Analyst question:** "What device conditions define membership in this firewall group?"

```sql
FROM claroty_organization_firewall_groups
| where name = 'FW-GROUP-CRITICAL-OT'
| fields name, raw_extensions
| limit 1
```

Then parse `raw_extensions["device_conditions"]` client-side.

**Expected result:** `raw_extensions["device_conditions"]` is a compact JSON-list string of device filter condition objects — same pattern as `claroty_organization_zones` device_conditions (FM-038).

**Pass criterion:** `"device_conditions"` key accessible in `raw_extensions` blob; value IS a String (compact JSON-list), NOT a nested JSON array.

**Sensor:** claroty_organization_firewall_groups. **DTU:** LIVE-REQUIRED.

---

## Section 14: claroty_organization_firewall_policies

*Story gate: S-CLAROTY-ORGPOLICY-001. DTU: NO — LIVE-REQUIRED. BC: BC-2.16.021 (Firewall Domain).*

### QA-056

**Analyst question:** "Show me all DENY firewall policies — map the blocked communication paths."

```sql
FROM claroty_organization_firewall_policies
| where activity_name = 'Deny'
| fields name, activity_name, comment
| limit 50
```

**Expected result:** Structurally symmetric to zone_policies deny query (QA-050). `activity_name` (from `policy_action`) = 'Deny'.

**Pass criterion:** Filter on `activity_name` works; no `E-QUERY-038`; `class_uid == 3004`.

**Sensor:** claroty_organization_firewall_policies. **DTU:** LIVE-REQUIRED.

---

### QA-057

**Analyst question:** "Which firewall policies are linked to active alerts? I need to understand what policies triggered detections."

```sql
FROM claroty_organization_firewall_policies
| fields name, activity_name, raw_extensions
| limit 50
```

Then parse `raw_extensions["related_alerts_ids"]` client-side; filter for non-empty arrays.

**Expected result:** `raw_extensions["related_alerts_ids"]` is a compact JSON-list string of alert IDs — same pattern as zone_policies related_alerts_ids (FM-044).

**Pass criterion:** `"related_alerts_ids"` key accessible in `raw_extensions` blob; value IS a String (compact JSON-list).

**Sensor:** claroty_organization_firewall_policies. **DTU:** LIVE-REQUIRED.

---

### QA-058

**Analyst question:** "What firewall group pairs does this policy control? I need to trace the policy coverage."

```sql
FROM claroty_organization_firewall_policies
| where name = 'FW-POLICY-BLOCK-EXTERNAL'
| fields name, activity_name, raw_extensions
| limit 1
```

Then parse `raw_extensions["applied_group_pairs"]` and `raw_extensions["communication_conditions"]` client-side.

**Expected result:** `raw_extensions["applied_group_pairs"]` is a compact JSON-list string of `{src_group, dst_group}` pair objects. Structurally symmetric to zone_policies `applied_zone_pairs` (QA-052).

**Pass criterion:** Both keys accessible in `raw_extensions` blob; `applied_group_pairs` IS a String (compact JSON-list) parseable as array of `{src_group, dst_group}` objects.

**Sensor:** claroty_organization_firewall_policies. **DTU:** LIVE-REQUIRED.

---

## Section 15: claroty_organization_acl_policies

*Story gate: S-CLAROTY-ACLPOLICY-001. DTU: NO — LIVE-REQUIRED. Non-paginated endpoint. BC: BC-2.16.022.*

### QA-059

**Analyst question:** "List all ACL policies by ID and name — I need an inventory for a compliance review."

```sql
FROM claroty_organization_acl_policies
| fields metadata_uid, name, comment
| limit 50
```

**Expected result:** `metadata_uid` (from `policy_id`, UUID format, REQUIRED), `name` (from `policy_name`), `comment` (from `policy_notes`) are Tier-1 columns. Single HTTP response (no pagination). If the tenant has fewer than 50 ACL policies, all are returned.

**Pass criterion:** `metadata_uid` column present (NOT `policy_id`); no `E-QUERY-038`; single POST issued (no offset/limit pagination in prism fetch log).

**Sensor:** claroty_organization_acl_policies. **DTU:** LIVE-REQUIRED.

---

### QA-060

**Analyst question:** "Look up a specific ACL policy by name to get its UUID for an external ticket."

```sql
FROM claroty_organization_acl_policies
| where name = 'ACL-CRITICAL-ASSETS'
| fields metadata_uid, name, comment, actor_user_name
| limit 1
```

**Expected result:** `metadata_uid` is the UUID-format policy ID. `actor_user_name` (from `policy_updated_by`) is the last modifier.

**Pass criterion:** Filter on `name` works; `metadata_uid` value is a UUID-format string; `actor_user_name` present.

**Sensor:** claroty_organization_acl_policies. **DTU:** LIVE-REQUIRED.

---

### QA-061

**Analyst question:** "Show me the raw ACL syntax and which device models it applies to — I need to verify the rule text."

```sql
FROM claroty_organization_acl_policies
| where name = 'ACL-CRITICAL-ASSETS'
| fields metadata_uid, name, raw_extensions
| limit 1
```

Then parse `raw_extensions["policy_acl"]` (raw ACL rule text), `raw_extensions["policy_acl_type"]` (syntax format, e.g., "Cisco dACL"), and `raw_extensions["applied_models"]` (compact JSON-list string of device model names).

**Expected result:** `raw_extensions["policy_acl"]` contains multi-line ACL rule text. `raw_extensions["policy_acl_type"]` = "Cisco dACL" (matches the mandatory body parameter from BC-2.16.022 §postconditions). `raw_extensions["applied_models"]` is a compact JSON-list string of model names.

**Pass criterion:** All three keys accessible in `raw_extensions` blob; `policy_acl_type` == "Cisco dACL"; `applied_models` IS a String (compact JSON-list).

**Sensor:** claroty_organization_acl_policies. **DTU:** LIVE-REQUIRED.

---

## Section 16: Cross-Table Flows (Expansion Tables)

### QA-062

**Analyst question:** "Full vulnerability triage: from a CVE → get all affected devices → look up full device profiles."

Step 1: `FROM claroty_vulnerabilities | where finding_info_title = 'CVE-2021-31998' | fields finding_info_title, message | limit 1` — verify CVE exists and get description.

Step 2: `FROM claroty_device_vulnerability_relations | where finding_info_title = 'CVE-2021-31998' | fields finding_info_title, time, raw_extensions | limit 50` — get all device relations.

Step 3: Parse `raw_extensions["device_uid"]` from step 2; for each device UID: `FROM claroty_devices | where device_uid = '<uid>' | fields device_uid, device_name, device_type, risk_score | limit 1`.

**Expected result:** Three-step flow threads correctly. `finding_info_title` is the join key between `claroty_vulnerabilities` and `claroty_device_vulnerability_relations`. `device_uid` (Tier-2 in device_vulnerability_relations) is the join key to `claroty_devices`.

**Pass criterion:** All three steps execute without `E-QUERY-038`; CVE data flows from vulnerabilities → device relations → device profiles; `device_uid` from raw_extensions resolves in `claroty_devices`.

**Sensor:** claroty_vulnerabilities + claroty_device_vulnerability_relations + claroty_devices. **DTU:** LIVE-REQUIRED (G2-G3 gated on story merges).

---

### QA-063

**Analyst question:** "OT event correlation: find recent OT events and check which of those have related alerts — then look up those alerts."

Step 1: `FROM claroty_ot_activity_events | where time > '2024-07-01T00:00:00Z' | fields finding_info_uid, activity_name, time, raw_extensions | limit 20` — get recent OT events.

Step 2: Parse `raw_extensions["related_alert_ids"]` from step 1; for each alert ID in the compact JSON-list string.

Step 3: `FROM claroty_alerts | where finding_info_uid = '<alert-id>' | fields finding_info_uid, finding_info_title, status, message | limit 1` — get alert details.

**Expected result:** Step 2 parses the compact JSON-list string to extract alert IDs. Step 3 uses `finding_info_uid` (POST-ROUTING-001 Arrow name for the alert ID column) to look up the correlated alert.

**Pass criterion:** Step 1 returns rows with `raw_extensions` containing `"related_alert_ids"` as a String; alert ID parsed from compact JSON-list string; step 3 returns the matching alert record via `finding_info_uid`.

**Sensor:** claroty_ot_activity_events + claroty_alerts. **DTU:** LIVE-REQUIRED for OT events; DTU+LIVE for alerts.

---

### QA-064

**Analyst question:** "Server → interface → zone coverage check: for a specific server, list its interfaces, then check if those interfaces' IPs appear in any zone's device conditions."

Step 1: `FROM claroty_servers | where device_name = 'SERVER-001' | fields device_name, status_code | limit 1` — confirm server exists.

Step 2: `FROM claroty_server_interfaces | where device_name = 'SERVER-001' | fields device_name, raw_extensions | limit 10` — get all interfaces.

Step 3: Parse `raw_extensions["ip_address"]` for each interface. Parse `raw_extensions["interface_name"]` for label.

Step 4: `FROM claroty_organization_zones | where status_code = true | fields name, raw_extensions | limit 50` — get zone configs.

Step 5: Parse `raw_extensions["device_conditions"]` (compact JSON-list string) for each zone; check if any condition references the interface IP addresses from step 3.

**Expected result:** `device_name` joins `claroty_servers` → `claroty_server_interfaces`. `ip_address` from server_interfaces feeds into zone device condition check. Both `claroty_server_interfaces` and `claroty_organization_zones` use the same `raw_extensions` compact-JSON-list-string pattern for their respective array fields.

**Pass criterion:** Steps 1–4 execute without error; IP addresses accessible from `raw_extensions["ip_address"]` in step 3; `raw_extensions["device_conditions"]` accessible as compact JSON-list string in step 5; both are String type (not nested JSON array nodes).

**Sensor:** claroty_servers + claroty_server_interfaces + claroty_organization_zones. **DTU:** LIVE-REQUIRED (G4 and G5 gated on story merges).

---

## Column Name Quick Reference (POST-ROUTING-001)

This table maps the most common analyst-facing column names from the old TOML col.name convention to the POST-ROUTING-001 Arrow field names. Use this when migrating existing scripts.

### Original 4 tables

| Old col.name (pre-ROUTING-001 or TOML) | Arrow field name (POST-ROUTING-001) | Table |
|---|---|---|
| `id` | `finding_info_uid` | claroty_alerts |
| `detected_time` | `time` | claroty_alerts |
| `updated_time` | `finding_info_modified_time` | claroty_alerts |
| `alert_name` | `finding_info_title` | claroty_alerts |
| `action` | `activity_name` | claroty_audit_logs |
| `user_display_name` | `actor_user_name` | claroty_audit_logs |
| `timestamp` | `time` | claroty_audit_logs |
| `username` | `actor_user_uid` | claroty_audit_logs |
| `note` | `comment` | claroty_audit_logs |
| `id` | `metadata_uid` | claroty_audit_logs |
| `uid` | `device_uid` | claroty_devices |
| `asset_id` | `device_instance_uid` | claroty_devices |
| `device_category` | `device_type` | claroty_devices |
| `device_type` | `device_type_label` | claroty_devices |
| `retired` | `status_code` | claroty_devices |
| `os_category` | `device_os_name` | claroty_devices |
| `alert_id` | `finding_info_uid` | claroty_device_alert_relations |
| `alert_note` | `comment` | claroty_device_alert_relations |
| `device_alert_detected_time` | `time` | claroty_device_alert_relations |
| `device_risk_score` | `risk_score` | claroty_device_alert_relations |
| `device_alert_status` | `status` | claroty_device_alert_relations |

### Expansion tables (G1–G6)

| TOML col.name | Arrow field name (POST-ROUTING-001) | Table |
|---|---|---|
| `name` | `finding_info_title` | claroty_vulnerabilities |
| `description` | `message` | claroty_vulnerabilities |
| `event_id` | `finding_info_uid` | claroty_ot_activity_events |
| `detection_time` | `time` | claroty_ot_activity_events |
| `event_type` | `activity_name` | claroty_ot_activity_events |
| `vulnerability_name` | `finding_info_title` | claroty_device_vulnerability_relations |
| `device_vulnerability_detection_date` | `time` | claroty_device_vulnerability_relations |
| `server_name` | `device_name` | claroty_servers |
| `server_status` | `status_code` | claroty_servers |
| `server_name` | `device_name` | claroty_server_interfaces |
| `interface_status` | `status_code` | claroty_server_interfaces |
| `zone_name` | `name` | claroty_organization_zones |
| `zone_description` | `comment` | claroty_organization_zones |
| `enabled` | `status_code` | claroty_organization_zones |
| `updated_by` | `actor_user_name` | claroty_organization_zones |
| `policy_name` | `name` | claroty_organization_zone_policies |
| `policy_action` | `activity_name` | claroty_organization_zone_policies |
| `policy_notes` | `comment` | claroty_organization_zone_policies |
| `updated_by` | `actor_user_name` | claroty_organization_zone_policies |
| `firewall_group_name` | `name` | claroty_organization_firewall_groups |
| `firewall_group_description` | `comment` | claroty_organization_firewall_groups |
| `enabled` | `status_code` | claroty_organization_firewall_groups |
| `updated_by` | `actor_user_name` | claroty_organization_firewall_groups |
| `policy_name` | `name` | claroty_organization_firewall_policies |
| `policy_action` | `activity_name` | claroty_organization_firewall_policies |
| `policy_notes` | `comment` | claroty_organization_firewall_policies |
| `updated_by` | `actor_user_name` | claroty_organization_firewall_policies |
| `policy_id` | `metadata_uid` | claroty_organization_acl_policies |
| `policy_name` | `name` | claroty_organization_acl_policies |
| `policy_notes` | `comment` | claroty_organization_acl_policies |
| `policy_updated_by` | `actor_user_name` | claroty_organization_acl_policies |

---

## Item Count

| Metric | Count |
|---|---|
| Total Q&A entries | 64 |
| Original 4-table entries (QA-001..QA-027) | 27 |
| Expansion G1–G6 entries (QA-028..QA-064) | 37 |
| Tables covered | 14 |
| Cross-table flow entries | 6 (QA-025..QA-027 original; QA-062..QA-064 expansion) |
| LIVE-REQUIRED entries (G2–G6 expansion) | 37 |
| DTU+LIVE entries (original 4 + G1) | 27 |

---

## Changelog

| Version | Date | Author | Summary |
|---|---|---|---|
| 0.3 | 2026-09-07 | product-owner | Expanded from 27 questions (4 tables) to 64 questions (14 tables). Added QA-028..QA-064 for G1–G6 expansion tables. Updated distribution table, column quick reference, and item count. AUTHORITATIVE 14-table catalog. |
| 0.2 | 2026-08-21 | product-owner | Initial catalog with 27 questions for claroty_alerts, claroty_audit_logs, claroty_devices, claroty_device_alert_relations, and 3 cross-table flows. |
