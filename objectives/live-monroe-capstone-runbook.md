---
document_type: demo-runbook
objective: live-capstone
level: ops
version: "1.0"
producer: product-owner
timestamp: "2026-09-08"
project: prism
status: ready
naming_regime: POST-ROUTING-001 (ocsf_column_naming=true)
supersedes: T13-capstone-demo-runbook.md (v1.12, DTU-based, stale)
client: monroe
sensor: claroty (xDome)
tenant_env: live (direct HTTPS to api.claroty.com)
dtu_dependency: NONE
enrich_scope: "enrich nvd(...) ONLY — enrich threat_intel(...) is DTU-bound, not live-safe"
read_only: true
gates_on:
  - S-ADR058-OCSF-ROUTING-001 MERGED
  - S-CLAROTY-VULNS-001 MERGED
  - S-CLAROTY-OT-EVENTS-001 MERGED (G2)
  - S-CLAROTY-DEVVULNREL-001 MERGED (G3)
  - S-CLAROTY-SERVERS-001 MERGED (G4)
  - S-CLAROTY-ORGPOLICY-001 MERGED (G5)
  - S-CLAROTY-ACLPOLICY-001 MERGED (G6)
related:
  - .factory/objectives/DEMO-SCOPE.md (v2.1, authoritative demo frame)
  - .factory/objectives/live-sensor-runbook.md (v1.0, live-soc deployment ops)
  - .factory/objectives/xdome-v1-validation/soc-analyst-qa-catalog.md (v0.3, 64 questions)
---

# Live Monroe Capstone Runbook — Claroty xDome SOC-Analyst Investigation

> **AUTHORITATIVE LIVE-MONROE CAPSTONE.** This document supersedes
> `.factory/objectives/T13-capstone-demo-runbook.md` (v1.12), which was authored
> against the DTU-EVERYTHING invariant (D-1163, now superseded) and uses pre-ROUTING-001
> column names with the stale DTU fleet. The T13 runbook MUST NOT be used for any live
> walkthrough. Use this document instead.

> **Governing direction:** Live demo runs against the LIVE Claroty xDome tenant
> ("monroe") — a single real tenant, live API, read-only SOC-analyst investigation
> across all 14 Claroty xDome tables. DTU fleet is stale and non-functional for a
> 14-table demo (D-2443 / DEMO-SCOPE.md §Live Demo Target — AUTHORITATIVE, 2026-09-08).

> **AD-017 discipline:** Customer data is structural observables only. This document
> contains ZERO real tenant data values — only wire shapes, field names, column types,
> and OCSF normalizations. Actual query results (IDs, names, counts, timestamps) are
> never captured or quoted.

---

## Prerequisites

| Item | Check |
|------|-------|
| Binary at `test-soc/bin/prism` built from develop HEAD | `test-soc/bin/prism --version` shows current PRISM_VERSION |
| Spec at `test-soc/.prism-live/specs/claroty.sensor.toml` is current | Matches `crates/prism-sensors/specs/claroty.sensor.toml` |
| Credentials loaded in macOS Keychain under the prism binary | Operator-loaded per AD-017; `check_sensor_health` confirms |
| All G1–G6 stories MERGED on develop | See frontmatter `gates_on` |
| Column naming regime | POST-ROUTING-001 (`ocsf_column_naming=true`) |
| Enrich scope | `enrich nvd(...)` ONLY — live-safe via HttpLookup |

**Launch:**
```bash
test-soc/prism-live-mcp-wrapper.sh
# Sets env, exec `bin/prism --config-dir .prism-live start`, MCP stdio
```

**MCP trust gate:** The Claude Code MCP server approval prompt MUST be approved
by a human operator. The AI harness classifier blocks auto-approval for a live
sensor MCP server. The operator approves once per session; subsequent calls are
auto-accepted. This gate is visible in the demo recording as a normal MCP server
registration flow — present it as part of the security model.

---

## 30-Second Product Framing

Prism is a **per-analyst MCP server** inside Claude Code. The analyst asks security
questions in PrismQL — a SQL-flavored language — against named sensor tables.
Prism fans out to vendor APIs, normalizes every result to OCSF, and returns a unified
response the LLM can reason over. In this session: one analyst, one real client
organization (monroe), one live Claroty xDome OT security platform, read-only.

**Read-only loop:** Action / case / rule / containment tools are NOT registered
(`-32003`). The entire session is observation and reasoning only.

---

## Investigation Storyline — The Monroe SOC Arc

**Scenario premise:** Morgan is a SOC analyst at the MSSP managing monroe, an
industrial operator running Claroty xDome for OT asset visibility. An alert just
surfaced in the xDome console. Morgan opens prism to investigate: triage the alert,
trace it to the affected OT device, check what vulnerabilities that device carries,
audit recent activity and configuration changes, then understand the zone/firewall
context to assess blast radius and detection coverage. The entire investigation happens
through PrismQL — no direct xDome UI, no vendor-specific API knowledge required.

**Analyst persona:** Morgan. Driven via the test-soc/CLAUDE.md discovery-first protocol.

---

## Act 1: Discovery and Orientation (Beats 1–3)

**Narrative:** Morgan opens a new Claude Code session with prism-live active.
Before querying anything, Morgan establishes what is available: which clients are
registered, whether the sensor is healthy, and what tables and columns the live
xDome surface exposes.

### Beat 1 — Client Discovery

**Tool:** `ReadMcpResource`
**Resource URI:** `prism://config/clients`
**Server:** `prism-live`

```
ReadMcpResource("prism://config/clients", server="prism-live")
```

**Expected observable shape:**
- Returns a JSON resource listing registered clients
- At minimum, entry with `client_id: "monroe"` is present
- Each entry has `enabled_sensors` listing `claroty` as active
- Never guess client_id — this step anchors everything that follows

**Why this beat:** Demonstrates zero-hardcoded-client-ID discipline. The discovery
resource is the single source of truth.

---

### Beat 2 — Sensor Health Check

**Tool:** `check_sensor_health`
**Arguments:** `{ "clients": ["monroe"] }`

```json
{ "tool": "check_sensor_health", "arguments": { "clients": ["monroe"] } }
```

**Expected observable shape:**
- Response contains a health entry for `monroe / claroty`
- `status` field: one of `Healthy`, `Degraded`, `Unavailable`
- For a fully-live run: `status == "Healthy"`, `tables_available` list is present
- `_source_type == "live"` confirms the upstream Claroty xDome API is reachable
- If Degraded: `error_details` and `E-SENSOR-030` diagnostic present; investigate
  before proceeding

**Why this beat:** Health gate. A failed health check stops the investigation before
any data queries. Visible proof that prism has a live connection to api.claroty.com.

---

### Beat 3 — Schema Discovery

**Tool:** `prism_describe`
**Arguments:** `{ "clients": ["monroe"] }`

```json
{ "tool": "prism_describe", "arguments": { "clients": ["monroe"] } }
```

**Expected observable shape:**
- Returns all 14 Claroty xDome table descriptors available for monroe
- Each table shows: Tier-1 column names with Arrow types, `raw_extensions` blob
  descriptor, `class_uid` synthesized field, `_sensor` synthesized field
- Notable Tier-1 column spot-check (from POST-ROUTING-001):
  - `claroty_alerts`: `finding_info_uid` (String), `finding_info_title` (String),
    `status` (String), `time` (Datetime), `finding_info_modified_time` (Datetime),
    `message` (String)
  - `claroty_devices`: `device_uid` (String), `device_name` (String),
    `device_type` (String), `device_type_label` (String), `risk_score` (String),
    `status_code` (Boolean), `device_os_name` (String), `device_instance_uid` (String)
  - `claroty_audit_logs`: `metadata_uid` (String), `activity_name` (String),
    `time` (Datetime), `actor_user_uid` (String), `actor_user_name` (String),
    `comment` (String)
- No stale pre-ROUTING-001 names (`id`, `detected_time`, `alert_name`,
  `device_category`, `device_type` (as label)) in the schema output
- `class_uid == 2004` for alerts (finding_info), `class_uid == 5001` for devices
  (inventory_info), `class_uid == 3006` for audit_logs (account_change)

**Why this beat:** Schema-first protocol. Morgan knows the full surface before writing
any query. Demonstrates that prism teaches its own API.

---

## Act 2: Initial Alert Triage — "What Fired?" (Beats 4–7)

**Narrative:** Morgan now knows the surface. The xDome console flagged an unresolved
alert. Morgan pulls current unresolved alerts, identifies one of interest, retrieves
full details, and checks for analyst commentary on prior analyst touches.

### Beat 4 — Unresolved Alert Triage

**Table:** `claroty_alerts`

```sql
FROM claroty_alerts
| where status = 'Unresolved'
| order by time desc
| limit 20
```

**Expected observable shape:**
- All rows have `status == 'Unresolved'`
- `time` column is a parseable ISO-8601 Datetime (POST-ROUTING-001 Arrow name,
  sourced from Claroty `detected_time`)
- `finding_info_uid` is the unique alert identifier (String — NOT the old `id` column)
- `finding_info_title` contains alert name strings (e.g., "Unauthorized Communication",
  "New Device Detected", "Configuration Upload Detected")
- Result is most-recent-first
- No `E-QUERY-038` column-not-found errors
- `_source_type == "live"`

**OCSF context:** `class_uid == 2004` (finding_info). `status` is OCSF Title-case.

---

### Beat 5 — Alert Title Search

**Table:** `claroty_alerts`
**Drives:** QA-003 (finding_info_title LIKE filter)

```sql
FROM claroty_alerts
| where finding_info_title LIKE '%Unauthorized Communication%'
| order by time desc
| limit 10
```

**Expected observable shape:**
- Returns rows where `finding_info_title` contains the search phrase
- `finding_info_title` is a first-class Tier-1 column — no `E-QUERY-038`
- Filter executes at DataFusion (in-engine post-filter; not pushed to xDome API for
  the LIKE predicate)
- Row schema: `finding_info_uid`, `finding_info_title`, `status`, `time`,
  `finding_info_modified_time`, `message`, `raw_extensions`

**Enrich opportunity (DEFERRED):** Morgan notes that `raw_extensions["ot_devices_count"]`
in the returned rows would give the OT device count per alert (QA-004 pattern). In v1,
access is via client-side parse of the `raw_extensions` JSON blob.

---

### Beat 6 — Alert Full Detail Fetch

**Table:** `claroty_alerts`
**Drives:** QA-006 (full alert detail), QA-007 (modified > detected)

Step 1 (capture UID from Beat 4/5 result):
```sql
FROM claroty_alerts
| where finding_info_title LIKE '%Unauthorized Communication%'
| fields finding_info_uid
| limit 1
```

Step 2 (full record — use UID from Step 1):
```sql
FROM claroty_alerts
| where finding_info_uid = '<uid-from-step-1>'
| limit 1
```

**Expected observable shape (Step 2):**
- Single row returned
- All six Tier-1 columns present: `finding_info_uid`, `status`, `time`,
  `finding_info_modified_time`, `message`, `finding_info_title`
- `finding_info_modified_time > time`: analyst has interacted with this alert
  (OCSF update time is after detection time) — confirms active investigation
- `raw_extensions` is a parseable JSON blob containing Tier-2 keys:
  `"ot_devices_count"`, `"severity_score"`, and other xDome-native fields
- `_source_type == "live"`

**OCSF note:** `finding_info_modified_time` sourced from Claroty `updated_time`.
The modified > detected comparison (QA-007) is a live-meaningful signal: it shows
analyst touches in the upstream platform.

---

### Beat 7 — Alert Status Distribution

**Table:** `claroty_alerts`
**Drives:** QA-002

```sql
FROM claroty_alerts
| group by status
| count(*) as alert_count
| order by alert_count desc
```

**Expected observable shape:**
- One row per distinct `status` value (Title-case OCSF): `'Unresolved'`,
  `'Resolved'`, `'In Progress'`, or other xDome-native status values
- No casing-duplicate buckets (OCSF Title-case normalization at adapter boundary,
  per ADR-058 + S-ADR058-OCSF-ROUTING-001)
- Total counts are observable but NEVER quoted in any artifact (AD-017)

**Why this beat:** Gives Morgan a situational-awareness snapshot before deep-diving
on one alert. Demonstrates GROUP BY + COUNT aggregate on a live dataset.

---

## Act 3: Device Identification and Profile (Beats 8–10)

**Narrative:** Morgan has a specific alert UID. Now pivot: which OT device does this
alert involve? What type of device is it, what's its risk posture, is it active?

### Beat 8 — Device-Alert Pivot

**Tables:** `claroty_device_alert_relations` (primary), `claroty_alerts` (context)
**Drives:** QA-022, QA-023

```sql
FROM claroty_device_alert_relations
| where finding_info_uid = '<uid-from-beat-6>'
| fields device_uid, risk_score, status, comment
| limit 10
```

**Expected observable shape:**
- `finding_info_uid` is the join key (POST-ROUTING-001 name — old name was `alert_id`,
  corrected by KF-07)
- `device_uid` is a Tier-1 column on `claroty_device_alert_relations`
- `risk_score` (from `device_risk_score`) is a String
- `status` (from `device_alert_status`) is the device-alert relationship status
- No `E-QUERY-038` on `finding_info_uid` or `device_uid`
- If `comment IS NOT NULL`: analyst has annotated the device-alert relationship

**Analyst note (QA-024 pattern):** `comment` (from `alert_note`) shows any analyst
commentary on this specific device-alert pairing.

---

### Beat 9 — Device Full Profile

**Table:** `claroty_devices`
**Drives:** QA-015, QA-016, QA-025 (step 3)

```sql
FROM claroty_devices
| where device_uid = '<device-uid-from-beat-8>'
| fields device_uid, device_name, device_type, device_type_label, risk_score, status_code, device_os_name
| limit 1
```

**Expected observable shape:**
- `device_type` (from `device_category`): category string such as `'OT Device'`,
  `'IT Device'`, `'Network Equipment'`
- `device_type_label` (from `device_type`, KF-06 corrected): the specific device
  type label (e.g., `'PLC'`, `'HMI'`, `'Engineering Workstation'`)
- Both columns present and distinct — no confusion between category and label
- `risk_score` (String): numeric string representing the xDome risk score
- `status_code` (Boolean): `false` = active device, `true` = retired/decommissioned
- `device_os_name` (from `os_category`): OS category string
- `class_uid == 5001` (inventory_info)

**Secondary query — check if device is online:**
```sql
FROM claroty_devices
| where device_uid = '<device-uid-from-beat-8>'
| fields device_uid, raw_extensions
| limit 1
```
Parse `raw_extensions["is_online"]` (Boolean in blob) and `raw_extensions["ip_list"]`
(compact JSON-list String — e.g., `"[\"10.x.x.x\"]"`) client-side.
**AD-017:** Actual IP values are never quoted in any record.

---

### Beat 10 — All High-Risk Devices (context)

**Table:** `claroty_devices`
**Drives:** QA-016

```sql
FROM claroty_devices
| where risk_score > '80'
| fields device_uid, device_name, device_type, device_type_label, risk_score
| order by risk_score desc
| limit 20
```

**Expected observable shape:**
- No `E-QUERY-038` on `risk_score`
- String comparison semantics (risk_score is String in v1; numeric ordering for
  precise ranking requires `json_extract_string` UDF, S-JSON-EXTRACT-UDF-001)
- Returns devices whose risk_score string sorts above '80'
- Confirms the device from Beat 9 is likely in this set if it's a high-risk asset

---

## Act 4: Audit Trail — "What Changed?" (Beats 11–12)

**Narrative:** An alert for unauthorized OT communication raises the question: was there
a configuration change in xDome before this alert was detected? Morgan queries the
audit log with a time-window push-down to find activity around the alert's detection
time. This demonstrates the `filter_by` push-down path to the xDome API.

### Beat 11 — Audit Trail Time-Window Query

**Table:** `claroty_audit_logs`
**Drives:** QA-012 (PD-004 push-down), QA-026 (time correlation)

*(Morgan computes `alert_time - 30min` and `alert_time + 30min` client-side from the
`time` value retrieved in Beat 6.)*

```sql
FROM claroty_audit_logs
| where time > '<alert-time-minus-30min>'
| where time < '<alert-time-plus-30min>'
| order by time asc
| limit 50
```

**Expected observable shape:**
- Both time bounds are pushed down to xDome via `filter_by.operation = "and"` with
  `"operands": [{gte}, {lte}]` (PD-004 pattern — verifiable in prism fetch log)
- `time` (from `timestamp`, INDEX column): ISO-8601 Datetime
- `activity_name` (from `action`): action type strings
- `actor_user_uid` (from `username`): the acting user's identity
- `actor_user_name` (from `user_display_name`): display name
- `metadata_uid` (from `id`, KF-05 RESOLVED): the audit record ID — a Tier-1
  first-class column, NOT accessed via `raw_extensions`
- `comment` (from `note`): any analyst notes attached to the audit entry
- No `E-QUERY-038` on any column
- `_source_type == "live"`

**MSSP value demonstration:** The time-window push-down means prism asks xDome to
filter server-side — only the relevant window is returned. For large tenants this is
essential for response time and rate-limit budget.

---

### Beat 12 — Configuration Change Search

**Table:** `claroty_audit_logs`
**Drives:** QA-012 (IEQ operator for case-insensitive match)

```sql
FROM claroty_audit_logs
| where time > '<alert-time-minus-30min>'
| where time < '<alert-time-plus-30min>'
| where activity_name IEQ 'configuration change'
| order by time asc
| limit 50
```

**Expected observable shape:**
- `IEQ` operator provides case-insensitive match — matches `'Configuration Change'`,
  `'configuration change'`, `'CONFIGURATION CHANGE'` regardless of xDome casing
- `activity_name` is Tier-1, directly filterable
- If any rows return: Morgan identifies who made a configuration change and when,
  relative to the alert detection time
- Parse `raw_extensions["category"]` client-side for the action category (Tier-2)
- `metadata_uid` in each row enables cross-reference with external ticketing systems
  (QA-011 pattern)

---

## Act 5: OT Activity — "What Happened on the Wire?" (Beats 13–14)

**Narrative:** The audit log shows what happened in xDome. But what happened on the
OT network itself? Morgan queries OT activity events to find unauthorized communication
patterns, checking for suspicious protocol usage and correlating events back to alerts.

*Note: `claroty_ot_activity_events` is LIVE-REQUIRED — no DTU coverage (G2 table).*
*Story gate: S-CLAROTY-OT-EVENTS-001.*

### Beat 13 — OT Configuration Upload Events

**Table:** `claroty_ot_activity_events`
**Drives:** QA-032

```sql
FROM claroty_ot_activity_events
| where activity_name = 'Configuration Upload'
| order by time desc
| limit 20
```

**Expected observable shape:**
- `activity_name` (from `event_type`): filter on `'Configuration Upload'`
- `time` (from `detection_time`): Datetime column — note that unlike `audit_logs`,
  this table has NO push-down support; the `time` filter is in-engine (DataFusion
  post-filter on the full page result)
- `finding_info_uid` (from integer `event_id`, normalized to String at adapter
  boundary): unique event identifier
- `raw_extensions` blob present
- No `E-QUERY-038`
- `_source_type == "live"`

---

### Beat 14 — OT Event Protocol and Alert Correlation

**Table:** `claroty_ot_activity_events`
**Drives:** QA-033, QA-034, QA-063 (cross-table correlation)

```sql
FROM claroty_ot_activity_events
| fields finding_info_uid, time, activity_name, raw_extensions
| limit 50
```

**Expected observable shape:**
- `raw_extensions["protocol"]` (String): OT protocol name — e.g., `"Modbus"`,
  `"CIP"`, `"DNP3"`, `"S7"`, `"EtherNet/IP"`
- `raw_extensions["source_ip"]` (String): source IP address
- `raw_extensions["dest_ip"]` (String): destination IP address
- `raw_extensions["dest_port"]` (Integer or integer String): destination port
- `raw_extensions["related_alert_ids"]` (String — compact JSON-list):
  e.g., `"[\"ALERT-001\",\"ALERT-002\"]"` — NOT a nested JSON array node

**Cross-table correlation (QA-063 pattern):** Parse `raw_extensions["related_alert_ids"]`
from events of interest. Use the extracted alert UID to look up the matching alert:
```sql
FROM claroty_alerts
| where finding_info_uid = '<alert-id-from-raw-extensions>'
| fields finding_info_uid, finding_info_title, status, message
| limit 1
```

This demonstrates the OT event → alert correlation pivot: a network-layer event
links back to a high-level alert finding.

---

## Act 6: Vulnerability Scope — "What CVEs Are Exposed?" (Beats 15–17)

**Narrative:** Morgan knows the affected device. What known vulnerabilities does it
carry? Are any on the CISA KEV list? Morgan queries the vulnerability catalog, enriches
CVE findings with NVD CVSS data, then checks the device-vulnerability relationships
to count affected devices and assess patch status.

*Note: G1 (`claroty_vulnerabilities`) is merged. G3 (`claroty_device_vulnerability_relations`)
is LIVE-REQUIRED. Story gates: S-CLAROTY-VULNS-001 (G1, merged), S-CLAROTY-DEVVULNREL-001 (G3).*

### Beat 15 — CVE Catalog Query

**Table:** `claroty_vulnerabilities`
**Drives:** QA-029, QA-030

```sql
FROM claroty_vulnerabilities
| where finding_info_title LIKE 'CVE-%'
| order by finding_info_title asc
| limit 50
```

**Expected observable shape:**
- `finding_info_title` (from `name`): CVE ID string, e.g., `'CVE-YYYY-NNNNN'`
- `message` (from `description`): human-readable CVE description
- Filter on `finding_info_title` works — Tier-1 column, no `E-QUERY-038`
- `raw_extensions["source_name"]`: advisory source (e.g., `"NVD"`, `"ICS-CERT"`)
- `raw_extensions["cvss_v3_score"]`: numeric CVSS v3 base score
- `raw_extensions["epss_score"]`: EPSS exploit probability (fractional 0.0–1.0)
- `raw_extensions["is_known_exploited"]`: Boolean CISA KEV indicator

**CISA KEV triage (QA-028 pattern):** Parse `raw_extensions["is_known_exploited"]`
client-side; filter for `true` to identify KEV-listed CVEs in the tenant environment.

---

### Beat 16 — NVD Enrichment

**Table:** `claroty_vulnerabilities`
**Enrich:** `enrich nvd(...)` — LIVE-SAFE via HttpLookup path
**Drives:** QA-031 + NVD enrichment UDF

```sql
FROM claroty_vulnerabilities
| where finding_info_title = '<cve-id-of-interest>'
| fields finding_info_title, message, raw_extensions
| enrich nvd(finding_info_title)
| limit 5
```

**Expected observable shape:**
- `finding_info_title` is the NVD lookup key (CVE ID string)
- After enrichment: additional columns present — `cvss_base_score`,
  `cvss_severity`, `cvss_vector` — sourced from the real NVD API via
  `nvd.infusion.toml` HttpLookup path
- `cvss_severity` returns OCSF Title-case string (e.g., `'High'`, `'Critical'`)
- The enrichment UDF executes live against the NVD API (no DTU involved)
- `_source_type == "live"` on the enriched rows

**Why `enrich nvd(...)` and not `enrich threat_intel(...)`:**
`enrich threat_intel(...)` routes through the prism-threatintel-infusion WASM plugin
to the ThreatIntel DTU endpoint. The DTU fleet is stale and non-functional for the
live demo path. `enrich nvd(...)` uses the HttpLookup path (`nvd.infusion.toml`) with
no DTU dependency — fully live-safe. Always use `enrich nvd(...)` in live scripts.

---

### Beat 17 — Device-Vulnerability Relations and Patch Status

**Table:** `claroty_device_vulnerability_relations`
**Drives:** QA-036, QA-037, QA-038, QA-039, QA-062

Step 1 — Count affected devices:
```sql
FROM claroty_device_vulnerability_relations
| where finding_info_title = '<cve-id-of-interest>'
| count(*) as affected_device_count
```

Step 2 — Detail rows with CISA KEV and patch status:
```sql
FROM claroty_device_vulnerability_relations
| where finding_info_title = '<cve-id-of-interest>'
| fields finding_info_title, time, raw_extensions
| limit 50
```

Step 3 — Full triage: for each device UID from raw_extensions, look up device profile:
```sql
FROM claroty_devices
| where device_uid = '<device-uid-from-raw-extensions>'
| fields device_uid, device_name, device_type_label, risk_score
| limit 1
```

**Expected observable shape (Steps 1–2):**
- `finding_info_title` (from `vulnerability_name`): CVE ID join key
- `time` (from `device_vulnerability_detection_date`): when this device-CVE pair
  was first detected
- `count(*)` returns a single-row aggregate: observable count (never quoted in artifacts)
- `raw_extensions["device_uid"]` (String): affected device's UID — composite PK element
- `raw_extensions["vulnerability_is_known_exploited"]` (Boolean): CISA KEV per
  device-vulnerability pair
- `raw_extensions["vulnerability_cvss_v3_score"]` (numeric): CVSS score per pair
- `raw_extensions["patch_status"]` (String): e.g., `"Patched"`, `"Not Patched"`
- `raw_extensions["patch_install_date"]` (ISO-8601 String or null)
- `raw_extensions["device_vulnerability_resolution_date"]` (ISO-8601 String or null)

---

## Act 7: Infrastructure Awareness — Servers and Network Coverage (Beats 18–19)

**Narrative:** Morgan needs to understand the collection infrastructure. Which xDome
servers are monitoring the network segment containing the affected device? Are the right
interfaces active? This tells Morgan whether prism has full visibility into the traffic
that generated the alert.

*Note: G4 tables are LIVE-REQUIRED. Story gate: S-CLAROTY-SERVERS-001.*

### Beat 18 — Server Inventory and Health

**Table:** `claroty_servers`
**Drives:** QA-040, QA-041, QA-042, QA-043

```sql
FROM claroty_servers
| fields device_name, status_code
| order by device_name asc
| limit 50
```

**Expected observable shape:**
- `device_name` (from `server_name`): collection server name
- `status_code` (from `server_status`): server operational status (Boolean or String —
  per spec; actual value semantics are xDome-native)
- No `server_name` column in result (retired pre-ROUTING-001 name)
- `raw_extensions["num_of_open_incidents"]` (Integer): parseable open incident count
- `raw_extensions["uptime_days"]` (Float with decimal component): e.g., `667.233661` —
  confirming it is a JSON number, NOT an integer
- `raw_extensions["site_id"]` (String): site identifier for cross-reference

---

### Beat 19 — Network Interface Coverage

**Table:** `claroty_server_interfaces`
**Drives:** QA-044, QA-045, QA-046, QA-064

Step 1 — List interfaces for the relevant server:
```sql
FROM claroty_server_interfaces
| where device_name = '<server-name-from-beat-18>'
| fields device_name, status_code, raw_extensions
| limit 20
```

Step 2 — Find interface by IP (client-side filter on raw_extensions["ip_address"]):
```sql
FROM claroty_server_interfaces
| fields device_name, status_code, raw_extensions
| limit 500
```

**Expected observable shape:**
- `device_name` (from `server_name`): join key back to `claroty_servers`
- `status_code` (from `interface_status`): interface operational status
- `raw_extensions["interface_name"]` (String): interface label
- `raw_extensions["is_monitored"]` (Boolean): `true` = actively capturing traffic
- `raw_extensions["ip_address"]` (String): interface IP address
- `raw_extensions["ip_address"]` values never quoted (AD-017)

**Coverage check:** Active monitoring on the interface covering the affected device's
network segment = full visibility. An interface with `is_monitored == false` covering
that segment = blind spot — a detection gap to escalate.

---

## Act 8: Zone and Policy Context — "What's the Segmentation?" (Beats 20–22)

**Narrative:** Morgan now understands the physical infrastructure. What about logical
segmentation? What zones is the affected device attributed to? What policies govern
cross-zone communication? Are DENY policies in place that should have blocked the
unauthorized communication? This act maps the policy landscape.

*Note: G5 tables are LIVE-REQUIRED. Story gate: S-CLAROTY-ORGPOLICY-001.*

### Beat 20 — Active Zone Map

**Table:** `claroty_organization_zones`
**Drives:** QA-047, QA-048, QA-049

```sql
FROM claroty_organization_zones
| where status_code = true
| fields name, comment, actor_user_name, raw_extensions
| order by name asc
| limit 50
```

**Expected observable shape:**
- `name` (from `zone_name`, PK, REQUIRED): zone identifier
- `comment` (from `zone_description`): zone description text
- `status_code` (from `enabled`, Boolean): `true` = active zone
- `actor_user_name` (from `updated_by`): last modifier — compliance audit trail
- `raw_extensions["attributed_devices"]` (Integer): device count attributed to zone
- `raw_extensions["device_conditions"]` (String — compact JSON-list): device filter
  conditions that determine zone membership — parseable as an array of condition objects
- No `E-QUERY-038`

---

### Beat 21 — Zone Policy — DENY Rules

**Table:** `claroty_organization_zone_policies`
**Drives:** QA-050, QA-051, QA-052

```sql
FROM claroty_organization_zone_policies
| where activity_name = 'Deny'
| fields name, activity_name, comment, actor_user_name
| limit 50
```

**Expected observable shape:**
- `name` (from `policy_name`, PK, REQUIRED): policy identifier
- `activity_name` (from `policy_action`): `'Deny'` — represents a blocked path
- `comment` (from `policy_notes`): policy rationale
- `actor_user_name` (from `updated_by`): last modifier

Secondary — alerting coverage and zone pairs:
```sql
FROM claroty_organization_zone_policies
| fields name, activity_name, raw_extensions
| limit 50
```
- `raw_extensions["should_generate_alerts"]` (Boolean): `true` = this policy
  generates alerts when triggered — confirms detection coverage
- `raw_extensions["alert_use_case"]` (String): alert scenario description
- `raw_extensions["applied_zone_pairs"]` (String — compact JSON-list): `{src_zone, dst_zone}`
  pair objects defining which zone-to-zone paths this policy covers
- `raw_extensions["communication_conditions"]` (String — compact JSON-list): traffic
  conditions (protocol, port, etc.)

**Investigative pivot:** If the unauthorized communication alert's source and
destination zones are NOT covered by a DENY policy with `should_generate_alerts == true`,
Morgan has found a detection gap.

---

### Beat 22 — Firewall Group Coverage

**Tables:** `claroty_organization_firewall_groups`, `claroty_organization_firewall_policies`
**Drives:** QA-053, QA-054, QA-055, QA-056, QA-057, QA-058

Firewall groups:
```sql
FROM claroty_organization_firewall_groups
| where status_code = true
| fields name, comment, actor_user_name, raw_extensions
| order by name asc
| limit 50
```

Firewall DENY policies:
```sql
FROM claroty_organization_firewall_policies
| where activity_name = 'Deny'
| fields name, activity_name, comment
| limit 50
```

**Expected observable shape (both tables):**
- Structurally symmetric to `claroty_organization_zones` and
  `claroty_organization_zone_policies` (same Tier-1 column set: `name`, `comment`,
  `status_code`, `actor_user_name`)
- `class_uid == 3004` for both firewall tables (network_activity)
- `raw_extensions["attributed_devices"]` (Integer): device count per firewall group
- `raw_extensions["device_conditions"]` (String — compact JSON-list): NOT a nested
  JSON array node — same compact-JSON-list-string pattern as zone device_conditions
- `raw_extensions["applied_group_pairs"]` on firewall policies: `{src_group, dst_group}`
  object pairs — structurally symmetric to zone `applied_zone_pairs`
- `raw_extensions["related_alerts_ids"]` (String — compact JSON-list): alert IDs
  that have triggered against this firewall policy — direct connection back to the
  initial alert from Beat 4

---

## Act 9: Access Control Review — "What Rules Govern This Device?" (Beat 23)

**Narrative:** Final layer: ACL policies. These are the device-level access rules —
Cisco dACL syntax controlling which devices can reach what. Morgan checks if an ACL
policy covers the affected device's model, and gets the raw ACL rule text for
escalation to the network team.

*Note: G6 table is LIVE-REQUIRED. Story gate: S-CLAROTY-ACLPOLICY-001.
Non-paginated endpoint: a single HTTP POST returns the entire policy set.*

### Beat 23 — ACL Policy Inventory and Rule Text

**Table:** `claroty_organization_acl_policies`
**Drives:** QA-059, QA-060, QA-061

Step 1 — Full inventory:
```sql
FROM claroty_organization_acl_policies
| fields metadata_uid, name, comment
| limit 50
```

Step 2 — Look up a specific policy for rule text and applied models:
```sql
FROM claroty_organization_acl_policies
| where name = '<policy-name-of-interest>'
| fields metadata_uid, name, raw_extensions
| limit 1
```

**Expected observable shape:**
- `metadata_uid` (from `policy_id`, UUID format, REQUIRED): UUID-format policy ID —
  Tier-1 first-class column, NOT in `raw_extensions` blob
- `name` (from `policy_name`): policy identifier
- `comment` (from `policy_notes`): policy notes
- `actor_user_name` (from `policy_updated_by`): last modifier
- Non-paginated: a single HTTP POST is issued (no offset/limit pagination visible
  in prism fetch log)
- `raw_extensions["policy_acl"]` (String): multi-line ACL rule text
- `raw_extensions["policy_acl_type"]` (String): ACL syntax format —
  `"Cisco dACL"` is the expected value per BC-2.16.022 §postconditions
- `raw_extensions["applied_models"]` (String — compact JSON-list): device model
  names this policy applies to — `"[\"Model-A\",\"Model-B\"]"` format; NOT a
  nested JSON array node

---

## Act 10: Investigation Wrap-Up (Beat 24)

**Narrative:** Morgan has completed the investigation arc across all 14 tables.
The final beat confirms the system-level health snapshot and surfaces what prism
would normally report back to the analyst as the investigation summary.

### Beat 24 — Environmental Health Snapshot

**Tables:** `claroty_alerts`, `claroty_audit_logs`, `claroty_devices`
**Drives:** QA-027 (three-query count pattern)

```sql
FROM claroty_alerts
| count(*) as total_alerts
```

```sql
FROM claroty_audit_logs
| count(*) as total_audit_events
```

```sql
FROM claroty_devices
| count(*) as total_devices
```

**Expected observable shape:**
- Three separate queries, each returning exactly one row with an integer count
- No errors on any of the three
- Counts are observable but NEVER quoted in any artifact (AD-017)

This closes the loop: Morgan now has a complete picture — from initial alert triage,
through device identification, audit trail, OT event correlation, vulnerability scope,
infrastructure coverage, zone/firewall policy context, and ACL rule text — all from
a single read-only PrismQL session against the live monroe Claroty xDome tenant.

---

## Beat-by-Beat Table Coverage Summary

| Beat | Act | Tables Exercised | Q&A Catalog |
|------|-----|-----------------|-------------|
| 1 | 1 — Discovery | (prism://config/clients resource) | — |
| 2 | 1 — Discovery | (check_sensor_health tool) | — |
| 3 | 1 — Discovery | (prism_describe — all 14 tables) | QA-019 |
| 4 | 2 — Alert Triage | claroty_alerts (1) | QA-001 |
| 5 | 2 — Alert Triage | claroty_alerts (1) | QA-003 |
| 6 | 2 — Alert Triage | claroty_alerts (1) | QA-006, QA-007 |
| 7 | 2 — Alert Triage | claroty_alerts (1) | QA-002 |
| 8 | 3 — Device ID | claroty_device_alert_relations (4) | QA-022, QA-023 |
| 9 | 3 — Device ID | claroty_devices (2) | QA-015, QA-016, QA-025 |
| 10 | 3 — Device ID | claroty_devices (2) | QA-016 |
| 11 | 4 — Audit Trail | claroty_audit_logs (3) | QA-012, QA-026 |
| 12 | 4 — Audit Trail | claroty_audit_logs (3) | QA-012 |
| 13 | 5 — OT Activity | claroty_ot_activity_events (5) | QA-032 |
| 14 | 5 — OT Activity | claroty_ot_activity_events (5) + claroty_alerts (1) | QA-033, QA-034, QA-063 |
| 15 | 6 — Vulns | claroty_vulnerabilities (6) | QA-029, QA-030 |
| 16 | 6 — Vulns | claroty_vulnerabilities (6) + NVD enrich | QA-031 |
| 17 | 6 — Vulns | claroty_device_vulnerability_relations (7) + claroty_devices (2) | QA-036, QA-037, QA-062 |
| 18 | 7 — Servers | claroty_servers (8) | QA-040, QA-041, QA-042 |
| 19 | 7 — Servers | claroty_server_interfaces (9) | QA-044, QA-045, QA-046 |
| 20 | 8 — Zones | claroty_organization_zones (10) | QA-047, QA-048, QA-049 |
| 21 | 8 — Zones/Policy | claroty_organization_zone_policies (11) | QA-050, QA-051, QA-052 |
| 22 | 8 — Firewall | claroty_organization_firewall_groups (12) + claroty_organization_firewall_policies (13) | QA-053, QA-056, QA-058 |
| 23 | 9 — ACL | claroty_organization_acl_policies (14) | QA-059, QA-060, QA-061 |
| 24 | 10 — Wrap | claroty_alerts (1) + claroty_audit_logs (3) + claroty_devices (2) | QA-027 |

**All 14 tables exercised:** claroty_alerts (1), claroty_devices (2),
claroty_audit_logs (3), claroty_device_alert_relations (4),
claroty_ot_activity_events (5), claroty_vulnerabilities (6),
claroty_device_vulnerability_relations (7), claroty_servers (8),
claroty_server_interfaces (9), claroty_organization_zones (10),
claroty_organization_zone_policies (11), claroty_organization_firewall_groups (12),
claroty_organization_firewall_policies (13), claroty_organization_acl_policies (14).

---

## Enrich Decision

**`enrich nvd(finding_info_title)` — INCLUDED (Beat 16)**
- Live-safe: uses HttpLookup path (`nvd.infusion.toml`), no DTU dependency
- Real NVD API call executed in-query
- Returns `cvss_base_score`, `cvss_severity`, `cvss_vector` columns
- Demonstrates the `| enrich` pipe stage against a live external service

**`enrich threat_intel(...)` — EXCLUDED from all live scripts**
- Routes through prism-threatintel-infusion WASM plugin to the ThreatIntel DTU endpoint
- DTU fleet is stale and non-functional for the live demo path (D-2443)
- Will produce `E-SENSOR-030 AllTargetsFailed` if attempted against the live binary
  without the ThreatIntel DTU running
- NEVER script this in a live walkthrough unless ThreatIntel DTU is explicitly
  confirmed functional

---

## Read-Only Discipline

The entire investigation is read-only:

| Capability | Status |
|-----------|--------|
| `query` | REGISTERED — full PrismQL query surface |
| `prism_describe` | REGISTERED — schema discovery |
| `check_sensor_health` | REGISTERED — health check |
| `list_capabilities` | REGISTERED — capability listing |
| `explain_query` | REGISTERED — query plan inspection |
| `ReadMcpResource prism://config/clients` | REGISTERED — client discovery |
| Case management tools | NOT REGISTERED (-32003) |
| Rule / detection management | NOT REGISTERED (-32003) |
| Containment / action tools | NOT REGISTERED (-32003) |
| Credential management | NOT REGISTERED (-32003) |
| Write-back to sensor | NOT REGISTERED (-32003) |

`_source_type: "live"` on all results means results carry `untrusted_external` trust
level. They are evidence for analyst reasoning, not conclusions.

---

## Recorded-Walkthrough Acceptance Criteria

The Phase-2 demo-recorder capture (T14) MUST demonstrate the following for this
capstone to be considered complete:

| AC | Requirement | Evidence |
|----|-------------|---------|
| RW-AC-001 | MCP trust gate shown — human operator approval visible before first query | Screen capture of the MCP server registration prompt |
| RW-AC-002 | Client discovery via `prism://config/clients` — not hardcoded | Resource read shown; `monroe` client_id visible |
| RW-AC-003 | `check_sensor_health` returns `Healthy` for `monroe/claroty` | Health response with status field visible |
| RW-AC-004 | `prism_describe` shows all 14 Claroty table descriptors | Descriptor list scrollable; at minimum 14 table entries visible |
| RW-AC-005 | At least one alert query with POST-ROUTING-001 column names (`finding_info_uid`, `finding_info_title`) — no pre-ROUTING-001 names | Query result columns shown |
| RW-AC-006 | Device-alert pivot demonstrated across `claroty_device_alert_relations` | `finding_info_uid` used as join key; `device_uid` returned |
| RW-AC-007 | `claroty_audit_logs` time-window push-down executed | Prism fetch log or health metadata shows `filter_by` issued |
| RW-AC-008 | `claroty_ot_activity_events` queried — LIVE-REQUIRED table | Results shown; `finding_info_uid` is String type |
| RW-AC-009 | `claroty_vulnerabilities` queried with CVE pattern | `finding_info_title LIKE 'CVE-%'` filter shown |
| RW-AC-010 | `enrich nvd(finding_info_title)` executed live | Enriched rows show `cvss_base_score`, `cvss_severity` columns |
| RW-AC-011 | `claroty_device_vulnerability_relations` queried | `count(*)` aggregate + `raw_extensions["patch_status"]` access |
| RW-AC-012 | `claroty_servers` and `claroty_server_interfaces` queried | Both table schemas shown; `raw_extensions["is_monitored"]` accessed |
| RW-AC-013 | Zone segmentation tables queried (`claroty_organization_zones`, `claroty_organization_zone_policies`) | Boolean filter on `status_code`; `activity_name = 'Deny'` filter shown |
| RW-AC-014 | Firewall tables queried (`claroty_organization_firewall_groups`, `claroty_organization_firewall_policies`) | Both tables executed; `raw_extensions["applied_group_pairs"]` accessible |
| RW-AC-015 | `claroty_organization_acl_policies` queried | `metadata_uid` as Tier-1 column; `raw_extensions["policy_acl_type"]` = `"Cisco dACL"` |
| RW-AC-016 | All 14 tables exercised within the walkthrough | Beat-by-beat summary shown or narrated |
| RW-AC-017 | Zero real tenant data values visible in any screenshot or narration (AD-017) | Operator confirms; analyst persona never reads aloud specific IDs, IPs, or names |
| RW-AC-018 | No DTU infrastructure launched — walkthrough is fully live | `prism-live-mcp-wrapper.sh` is the ONLY launcher used |
| RW-AC-019 | `enrich threat_intel(...)` NOT invoked at any point | Walkthrough script review; no threat_intel UDF call |
| RW-AC-020 | Investigative arc flows naturally: alert → device → audit → OT event → vuln → infra → policy → ACL | Acts 2–9 presented in coherent sequence |

**Operator note on MCP trust gate (RW-AC-001):** The Claude Code AI harness classifier
blocks auto-approval for a live sensor MCP server — this is intentional security
behaviour. A human operator must manually approve the prism-live MCP server before
any queries can run. This gate is presented in the recording as part of the security
model: "prism requires explicit human approval before connecting to a live OT security
platform." Do NOT attempt to work around or skip this gate.

---

## Session Quick-Reference

| Step | Command / Call | Notes |
|------|---------------|-------|
| Launch | `test-soc/prism-live-mcp-wrapper.sh` | Sets env, starts MCP stdio |
| Approve gate | Operator approves prism-live MCP server | Human-only; classifier blocks AI auto-approval |
| Discovery | `ReadMcpResource("prism://config/clients")` | Never guess client_id |
| Health | `check_sensor_health({ clients: ["monroe"] })` | Gate before querying |
| Schema | `prism_describe({ clients: ["monroe"] })` | All 14 tables and Tier-1 columns |
| Query | `query({ clients: ["monroe"], query: "<PQL>", limit: 1000 })` | Max 1000 rows; pipe or SQL mode |
| Fresh | Add `force_refresh: true` to bypass sensor-fetch cache | Use when staleness suspected |
| Enrich | `\| enrich nvd(finding_info_title)` — live-safe only | Never `enrich threat_intel(...)` in live |
| Explain | `explain_query({ clients: ["monroe"], query: "<PQL>" })` | Shows push-down plan |

**Column naming:** All queries in this runbook use POST-ROUTING-001 Arrow field names.
Pre-ROUTING-001 names (`id`, `detected_time`, `alert_name`, `device_category`,
`device_type` as label, `finding_uid`, `alert_id`, etc.) produce `E-QUERY-038`
column-not-found errors. Use the Q&A Catalog column quick-reference for migration.

---

## Changelog

| Version | Date | Author | Summary |
|---------|------|--------|---------|
| 1.0 | 2026-09-08 | product-owner | Initial authoring. Live-monroe capstone replacing stale DTU-based T13 runbook. 24-beat investigation arc across all 14 Claroty xDome tables. POST-ROUTING-001 column names throughout. enrich nvd(...) only. AD-017 compliant. Zero DTU dependency. |
