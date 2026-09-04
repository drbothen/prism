---
document_type: ops-playbook
title: Claroty xDome Live Demo Playbook — monroe Tenant
version: "1.0"
producer: technical-writer
created: 2026-09-04
status: active
portability: project-specific (prism + test-soc live-soc)
grounding: RC1 live validation 2026-09-04 (rc1-live-validation-evidence.json)
changelog:
  - version: "1.0"
    date: 2026-09-04
    author: technical-writer
    summary: >
      Initial authoring from RC1 live validation ground truth. 14 tables confirmed,
      13 live-data tables, 2 valid-0-row tables, 1 LRG_TBL. All steps verified
      against rc1-live-validation-evidence.json and live-tenant-validation-runbook.md §8.
---

# Claroty xDome Live Demo Playbook — monroe Tenant

**Binary**: prism 1.0.0-rc.1  
**Client**: monroe  
**Sensor**: claroty (xDome)  
**MCP server name** (as registered in `.mcp.json`): `prism-live`  
**MCP wrapper**: `/Users/jmagady/Dev/test-soc/prism-live-mcp-wrapper.sh`  
**Config dir**: `/Users/jmagady/Dev/test-soc/.prism-live`  
**Grounding**: RC1 live validation run 2026-09-04 —
evidence at `/Users/jmagady/Dev/test-soc/live-soc/monroe/state/rc1-live-validation-evidence.json`

**AD-017**: No credentials, tokens, bearer values, or secrets appear anywhere in this playbook.  
**D-2410**: All expected response shapes use structural placeholders only — no live row values.

---

## Prerequisites

Before running the demo, verify:

1. `prism-live` is listed in `/Users/jmagady/Dev/test-soc/.mcp.json` pointing to the wrapper.
2. Monroe credentials are in the OS keyring (`prism/monroe/claroty/bearer_token`). Run
   `security find-generic-password -s "prism/monroe/claroty/bearer_token" -w` to confirm
   the entry exists (value is not printed here per AD-017).
3. The binary is the current RC release:
   ```bash
   /Users/jmagady/Dev/test-soc/bin/prism --version
   # Expected: prism 1.0.0-rc.1
   ```
4. The live spec is current:
   ```bash
   grep -c '^\[\[tables\]\]' /Users/jmagady/Dev/test-soc/.prism-live/specs/claroty.sensor.toml
   # Expected: 14
   ```

The wrapper starts Prism with `PRISM_DTU_MODE` unset — this instance talks to the real
Claroty xDome API at `https://api.claroty.com` via the monroe customer overlay at
`.prism-live/specs/customers/monroe/claroty.sensor.toml`.

---

## Tables in Scope

All 14 Claroty xDome tables, confirmed present and schema-correct via `prism_describe`:

| Table | Columns | RC1 live result | Notes |
|---|---|---|---|
| claroty_alerts | 11 | PASS — live rows | Active alert feed |
| claroty_audit_logs | 8 | PASS — live rows | Admin audit trail |
| claroty_devices | 20 | PASS — live rows | OT/IoT device inventory |
| claroty_device_alert_relations | 10 | PASS — live rows | Device↔alert join |
| claroty_device_vulnerability_relations | 13 | PASS — live rows | Device↔vuln join |
| claroty_servers | 17 | PASS — live rows | IT server inventory |
| claroty_server_interfaces | 10 | PASS — live rows | Server network interfaces |
| claroty_organization_zones | 11 | PASS — live rows | Network zone definitions |
| claroty_organization_zone_policies | 13 | PASS — live rows | Zone-level policies |
| claroty_organization_firewall_groups | 11 | PASS — live rows | FW group definitions |
| claroty_organization_firewall_policies | 13 | PASS — live rows | FW policy rules |
| claroty_ot_activity_events | 21 | PASS — **0 rows** | Monroe tenant has no OT activity events |
| claroty_organization_acl_policies | 11 | PASS — **0 rows** | Monroe tenant has no ACL policies configured |
| claroty_vulnerabilities | 19 | **LRG_TBL** — schema only | Large-tenant offset-pagination timeout (see §Known Limitations) |

---

## Step 1: Onboarding and Health Check

**What this shows**: Prism has the monroe client registered with claroty enabled, and the sensor
is live-reachable with valid credentials.

### 1a — Client registry

**Claude Code prompt** (invoke with `prism-live` MCP loaded):
> "Read the prism://config/clients resource and show me what clients are registered."

**Underlying MCP call**:
```json
{
  "method": "resources/read",
  "params": { "uri": "prism://config/clients" }
}
```

**Expected response shape** (D-2410 — structural only):
```json
[
  {
    "client_id": "monroe",
    "enabled_sensors": ["claroty"],
    "sensor_count": 1
  }
]
```
Check: `client_id = "monroe"`, `"claroty"` appears in `enabled_sensors`.  
Note: the field is `enabled_sensors` (array of sensor_id strings). Do not confuse with `sensors`
(a different field not present in this response).

### 1b — Sensor health

**Claude Code prompt**:
> "Run check_sensor_health for the monroe client."

**Underlying MCP call**:
```json
{
  "method": "tools/call",
  "params": {
    "name": "check_sensor_health",
    "arguments": { "client_id": "monroe" }
  }
}
```

**Expected structuredContent shape** (D-2410):
```json
{
  "overall_status": "healthy",
  "summary": "<human-readable summary string>",
  "sensors": [
    {
      "sensor_id": "claroty",
      "reachable": true,
      "auth_valid": true,
      "probe_level": "<string>",
      "latency_ms": <number>,
      "error": null
    }
  ]
}
```
Check: `overall_status = "healthy"`, `reachable = true`, `auth_valid = true`.  
If `isError: true` with `E-SENSOR-030` in the text, the keyring entry is missing or
the Claroty API is unreachable — this is an upstream issue, not a Prism bug.

---

## Step 2: Schema Discovery — All 14 Tables

**What this shows**: Prism knows the full Claroty xDome schema for this client, including column
names, OCSF field mappings, and table descriptions.

**Claude Code prompt**:
> "Run prism_describe for the monroe client and list all available tables."

**Underlying MCP call**:
```json
{
  "method": "tools/call",
  "params": {
    "name": "prism_describe",
    "arguments": { "client_id": "monroe" }
  }
}
```

**Expected structuredContent shape** (D-2410):
```json
{
  "results": {
    "tables": [
      {
        "name": "claroty_alerts",
        "columns": [ /* 11 column descriptors */ ],
        "description": "<table description string>",
        "sensor_type": "claroty"
      },
      {
        "name": "claroty_audit_logs",
        ...
      }
      /* ... 14 entries total ... */
    ]
  },
  "_meta": { "total_results": 0 }
}
```
Note: `_meta.total_results` is the query row count, not the table count — it is 0 here
because `prism_describe` does not execute a row-returning query.

Check: all 14 table names appear under `results.tables[].name`. Each entry carries a `columns`
array and a `sensor_type: "claroty"` field.

**Full validation script** (faster — runs describe + all 14 smoke queries in parallel):
```bash
python3 /Users/jmagady/Dev/test-soc/live-validate-sensor.py \
  --client monroe --sensor claroty
```

---

## Step 3: Query Patterns

### 3a — Pipe form (PrismQL pipe mode)

**What this shows**: PrismQL pipe syntax; live data returned from Claroty xDome.

**Claude Code prompt**:
> "Query claroty_alerts for the monroe client using pipe mode, limit 3 rows."

**Query** (copy-paste):
```
FROM claroty_alerts | limit 3
```
**MCP call arguments**: `{ "query": "FROM claroty_alerts | limit 3", "clients": ["monroe"] }`

**Expected shape** (D-2410 — shape only, not values):
```json
{
  "results": {
    "rows": [
      {
        "_source_type": "live",
        "class_uid": <integer>,
        "raw_extensions": { /* Claroty-specific fields */ },
        /* OCSF Tier-1 columns as top-level keys */
      }
    ],
    "total_results": <integer>
  }
}
```
Check: `rows` is non-empty, `_source_type = "live"` on each row, `class_uid` present
(integer, OCSF class identifier), `raw_extensions` object present.

### 3b — SQL form

**What this shows**: SQL SELECT syntax produces identical results to pipe mode.

**Query**:
```sql
SELECT * FROM claroty_alerts LIMIT 3
```

Expected shape identical to 3a. Both query forms reach the same execution path after parsing.

### 3c — WHERE filter (pipe form)

**What this shows**: Predicate filtering over live data.

**Query**:
```
FROM claroty_devices | where risk_level = 'High' | limit 5
```
Note: `risk_level` is a Tier-2 column — it appears in `raw_extensions` at the wire level and
as a top-level key in the projected row. Use the column name as declared in the spec (no
dot-notation prefix needed for Tier-2 columns).

Alternative using an OCSF Tier-1 column (projected as standalone Arrow field):
```
FROM claroty_devices | where severity_id = 3 | limit 5
```

**SQL form equivalent**:
```sql
SELECT * FROM claroty_devices WHERE risk_level = 'High' LIMIT 5
```

Expected: rows where `risk_level` equals `'High'`; may return 0 rows if no High-risk devices
exist — that is valid, not an error.

### 3d — Sort and limit

**What this shows**: Sort stage, deterministic ordering.

**Query** (pipe form):
```
FROM claroty_audit_logs | sort time asc | limit 5
```
The `time` column is an OCSF Tier-1 datetime column (`ocsf_field = "time"`), projected as
`time` in the Arrow output.

### 3e — Pushdown-eligible query (vulnerabilities name column)

**What this shows**: The `name` column on `claroty_vulnerabilities` carries `options = ["REQUIRED"]`,
making it index-eligible for the Claroty API request body. However, note the large-table
limitation described in §Known Limitations before running this step.

```sql
SELECT finding_info_title, message, cvss_v3_score FROM claroty_vulnerabilities
WHERE finding_info_title = '<exact_vuln_name>' LIMIT 1
```
Note: under `ocsf_column_naming = true`, the OCSF Arrow name for the `name`/`finding_info.title`
column is `finding_info_title`. The raw spec column name `name` is also accepted in predicates.

**Caution**: this query still invokes the offset-pagination fetch on a large dataset. See
§Known Limitations for the expected outcome on the monroe tenant.

---

## Step 4: OCSF Correctness

**What this shows**: Every live row carries `_source_type: "live"`, an OCSF `class_uid`, and
`raw_extensions` — confirming end-to-end OCSF normalization from the Claroty xDome API.

### 4a — OCSF class_uid on alerts (detection_finding)

**Query**:
```sql
SELECT class_uid, finding_info_title, finding_info_uid, time FROM claroty_alerts LIMIT 3
```
Expected: `class_uid` is a non-null integer (OCSF detection_finding class, e.g. 2004).
`finding_info_title` is a string (mapped from the Claroty `subject` field).
`time` is a datetime value.

### 4b — raw_extensions on devices (network_activity)

**Query**:
```
FROM claroty_devices | fields class_uid, raw_extensions | limit 3
```
Expected: `class_uid` is present (integer), `raw_extensions` is a JSON object containing
Claroty-specific fields that are not mapped to OCSF Tier-1 columns. The object is non-null
on live rows. The `fields` pipe stage projects only the named columns.

### 4c — Wire-level assertion (use the Python script for structural evidence)

The validation script records structural evidence — `_source_type`, `class_uid` presence,
`raw_extensions` presence — per row across all 14 tables, without logging any row values
(D-2410 compliance). Evidence written to:
```
/Users/jmagady/Dev/test-soc/live-soc/monroe/state/live-validation-claroty.json
```

---

## Step 5: Cross-Table SOC Workflow

**What this shows**: A realistic multi-table analyst investigation loop — start from an active
alert, find the affected device(s), then surface the device's vulnerability profile.

This is the core SOC use case: Prism federates three API calls into a single in-session query
chain without any ETL or persistent storage.

### 5a — Start: pull recent alerts

**Query**:
```
FROM claroty_alerts | sort time desc | limit 5
```
Note one `finding_info_uid` value from the results (the alert identifier).
Call it `<ALERT_ID>` in subsequent steps. **Do not paste the actual value into any
screen-recording transcript or shared document per D-2410.**

### 5b — Find devices associated with that alert

**Query** (substitute `<ALERT_ID>` from step 5a):
```
FROM claroty_device_alert_relations | where alert_id = '<ALERT_ID>' | limit 10
```
Note one `device_uid` value from the results (the Claroty device UUID).
Call it `<DEVICE_UID>`.

Column reference:
- `alert_id` — maps to OCSF `finding_info.uid`, projected as `alert_id` in the output
- `device_uid` — the Claroty device UUID (OCSF `device.uid`)
- `device_alert_detected_time` — when the alert was associated with this device

### 5c — Surface that device's vulnerability profile

**Query** (substitute `<DEVICE_UID>` from step 5b):
```
FROM claroty_device_vulnerability_relations | where device_uid = '<DEVICE_UID>' | limit 10
```
Expected: rows showing the vulnerability names, CVE IDs, severity scores, and CVSS data for
the specific device involved in the alert.

Column reference (key columns for the demo narrative):
- `device_uid` — device identifier (matches 5b output)
- `finding_info_title` — vulnerability name (OCSF Tier-1)
- `cvss_v3_base_score` — CVSS v3 base score (float)
- `is_known_exploited` — CISA KEV membership (boolean)

### 5d — Pivot to device context

**Query** (substitute `<DEVICE_UID>` from step 5b):
```sql
SELECT device_name, device_type, device_category, risk_level, class_uid, raw_extensions
FROM claroty_devices
WHERE device_uid = '<DEVICE_UID>'
LIMIT 1
```
Note: `device_uid` on the `claroty_devices` table is a Tier-2 column — it appears in
`raw_extensions` but is also queryable by name in WHERE predicates.

**Narrative for the demo**: Steps 5a through 5d demonstrate that an analyst can start from
an OCSF-normalized alert, trace it to a physical OT device, examine the device's vulnerability
burden, and pull the device's full context — all from a single Prism session, against live
Claroty xDome data, without any pre-built dashboards or persistent data stores.

---

## Step 6: Enrichment

### Current state on monroe (2026-09-04)

The live environment file `/Users/jmagady/Dev/test-soc/live-soc/prism-live-env.sh` is **absent**.
The wrapper's fallback block sets dummy values:
- `PRISM_THREATINTEL_BASE_URL=http://127.0.0.1:0`
- `PRISM_NVD_BASE_URL=http://127.0.0.1:0`

Additionally, the infusions directory `~/.prism-live/infusions/` is absent from the live config,
so no infusion UDFs are registered for the monroe client.

### 6a — Graceful-fail behavior (demonstrate, do not hide)

**What this shows**: Prism returns a structured, actionable error when an unknown enrichment UDF
is invoked — it does not crash or return an opaque internal error.

**Query**:
```
FROM claroty_device_vulnerability_relations | enrich nvd_cvss(cve_ids) | limit 3
```

**Expected error response** (E-QUERY-039 — EnrichUdfNotFound):
```json
{
  "isError": true,
  "content": [{
    "type": "text",
    "text": "... E-QUERY-039 ... Enrichment UDF 'nvd_cvss' not found. Available UDFs for this client: []"
  }]
}
```
The error carries the available-UDFs list (empty because no infusion files are deployed).
The message may also include a `did_you_mean` suggestion if a registered name is within
Levenshtein distance 3 of the queried name.

This is the correct graceful-fail path. The sensor query layer completed successfully;
the enrichment layer reported the missing UDF as a plan-time error rather than returning
silently empty columns.

### 6b — If enrichment keys are available (for future demos)

To enable live enrichment, deploy the infusion TOML files and set environment variables
in `/Users/jmagady/Dev/test-soc/live-soc/prism-live-env.sh`:

```bash
# Copy infusion specs to live config
cp /Users/jmagady/Dev/prism/specs/infusions/nvd.infusion.toml \
   /Users/jmagady/Dev/test-soc/.prism-live/infusions/
cp /Users/jmagady/Dev/prism/specs/infusions/threatintel.infusion.toml \
   /Users/jmagady/Dev/test-soc/.prism-live/infusions/
```

Then set real API keys in `prism-live-env.sh` (AD-017 — values never enter this document).

Once deployed, the NVD enrichment query:
```
FROM claroty_device_vulnerability_relations | enrich cvss_base_score(cve_ids) | limit 3
```
would add columns `cvss_base_score` (float), `cvss_severity` (string), `cvss_vector` (string)
sourced from the NVD CVSS API per `nvd.infusion.toml`.

The threat intel enrichment (requires the ThreatIntel plugin `threatintel-lookup.prx`):
```
FROM claroty_alerts | enrich threat_score(finding_info_uid) | limit 3
```
would add columns `threat_is_known_malicious` (boolean), `threat_score` (integer),
`threat_sources` (JSON array) per `threatintel.infusion.toml`.

---

## Step 7: Stability Check and Wrap

Run the full parallel validation script to confirm all 14 tables remain healthy at the end
of the demo session:

```bash
python3 /Users/jmagady/Dev/test-soc/live-validate-sensor.py \
  --client monroe --sensor claroty \
  --limit 3 --timeout 45
```

**Expected terminal output** (D-2410 — structural only):
```
========================================================================
VALIDATION SUMMARY
========================================================================
  Client:               monroe
  Sensor:               claroty
  Client registration:  PASS
  Sensor health:        PASS
  prism_describe:       PASS (14 tables found, 14 expected)
  Smoke queries:        12 PASS, 1 LRG_TBL, 0 KNW_LIM, 0 TIMEOUT, 0 FAIL

  OVERALL VERDICT: PASS
  → All 14 claroty tables resolve live, health OK.
  → 1 large-volume table(s) confirmed present but timed out on row retrieval (not blocking).
========================================================================
```

The 12 PASS count is the expected outcome: 11 tables with live rows + 2 tables with
0 rows (claroty_ot_activity_events, claroty_organization_acl_policies) + the 1 LRG_TBL
(claroty_vulnerabilities) = 14 total, 0 blocking failures.

Evidence is written to:
```
/Users/jmagady/Dev/test-soc/live-soc/monroe/state/live-validation-claroty.json
```

---

## Known Limitations

### KL-1: claroty_vulnerabilities — large-tenant offset-pagination timeout

**Symptom**: Queries against `claroty_vulnerabilities` return a prism-side timeout error
with `[transient]` in the message text. Status in the validation script: `LRG_TBL`.

**Root cause**: The Claroty xDome `/api/v1/vulnerabilities/` endpoint uses POST-for-read
with offset pagination (`page_size = 1000`). On the monroe tenant, the full CVE dataset
exceeds the per-query timeout before all pages are fetched. This is a large-tenant data
volume issue, not a schema or credential failure.

**What works**: Schema inspection via `prism_describe` succeeds — the table appears with all
19 columns correctly described. The LRG_TBL status in the validation script is non-blocking
by definition (the table is confirmed present in the schema).

**What does not work**: Row retrieval via any `query` call. A WHERE clause on cvss score,
date, or vulnerability name does **not** avoid the timeout. The timeout occurs at the sensor
API fetch layer (before DataFusion filters are applied); PrismQL WHERE predicates are not
pushed down to the Claroty API request body for this table. The body_template is a fixed
fields-and-sort specification with no filter injection path.

**Demo script for this table** (schema only):
```
# Show the schema — this completes immediately
# Claude Code prompt: "Describe the claroty_vulnerabilities table for the monroe client."
```
Then show the 19-column describe output. Narrate that row retrieval is blocked by tenant
data volume, and that the fix path (API-level filter_by pushdown, or streaming early-stop
pagination) is a known follow-up item, not a deployment regression.

### KL-2: claroty_ot_activity_events — tenant has no data

**Symptom**: Queries return 0 rows with `isError: false` and `total_results: 0`.

**Root cause**: The monroe deployment of Claroty xDome does not have the OT Activity Events
feature populated. This is a tenant configuration state, not a Prism issue.

**Correct presentation**: Show the query returning `[]` with status `PASS`. Narrate that
0 rows with no error is the correct behavior — Prism faithfully reports what the API returns.

### KL-3: claroty_organization_acl_policies — tenant has no data

Same behavior as KL-2. The monroe tenant has no ACL policies configured.
Queries return 0 rows, `isError: false`. Status: PASS.

### KL-4: Array-type columns (ip_list, mac_list) not queryable

The `claroty_devices` table has array-type columns (`ip_list`, `mac_list`, `network_list`,
`vlan_list`) that appear in `raw_extensions` but cannot be used in WHERE predicates.
Attempting to do so returns a known-limitation error. These columns are readable as JSON
values in SELECT projections; they cannot be the subject of equality or range predicates.

### KL-5: MCP serverInfo version is hardcoded "0.1.0"

The `serverInfo.version` field in the MCP initialize response is `"0.1.0"` regardless of
the actual binary version. To confirm the deployed binary version use:
```bash
/Users/jmagady/Dev/test-soc/bin/prism --version
# prism 1.0.0-rc.1
```

---

## Quick Reference — MCP Tool Signatures

| Tool | Required arguments | Notes |
|---|---|---|
| `prism_describe` | `client_id: string` | Returns `results.tables[]` with column descriptors |
| `check_sensor_health` | `client_id: string` | Returns `structuredContent.overall_status` |
| `query` | `query: string`, `clients: [string]` | PrismQL pipe or SQL form; `clients` is an array |
| `resources/read` | `uri: "prism://config/clients"` | Returns registered client list |

PrismQL pipe syntax summary:
```
FROM <table>
  | where <field> <op> '<value>'
  | sort <field> asc|desc
  | fields <col1>, <col2>
  | limit N
  | enrich <udf_name>(<field>)
```

SQL syntax summary:
```sql
SELECT <columns> FROM <table>
  WHERE <predicate>
  ORDER BY <col> ASC|DESC
  LIMIT N
```

Both forms accept the same `clients: ["monroe"]` argument in the MCP `query` tool call.

---

## Appendix: Repeatable Full-Run Script

```bash
#!/usr/bin/env bash
# Repeat this at the start and end of any demo session to confirm no regression.
set -euo pipefail

echo "=== Binary version ==="
/Users/jmagady/Dev/test-soc/bin/prism --version

echo "=== Full live validation ==="
python3 /Users/jmagady/Dev/test-soc/live-validate-sensor.py \
  --client monroe \
  --sensor claroty \
  --limit 3 \
  --timeout 45

echo "=== Evidence location ==="
echo "/Users/jmagady/Dev/test-soc/live-soc/monroe/state/live-validation-claroty.json"
```

Save as `/Users/jmagady/Dev/test-soc/live-soc/demo-preflight.sh` and run before screen-recording.
Expected exit code: 0. Any exit 1 indicates a blocking failure (credential, connectivity, or
schema regression) that must be resolved before the demo proceeds.
