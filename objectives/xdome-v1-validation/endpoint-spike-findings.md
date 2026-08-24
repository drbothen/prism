---
document_type: spike-findings
version: "1.0"
created: 2026-08-24
producer: architect
reference_plan: .factory/objectives/xdome-endpoint-expansion-plan.md
reference_extract: .factory/objectives/xdome-v1-validation/endpoint-schema-extract.md
---

# xDome Endpoint Expansion — Pre-Work Spike Findings

Four spikes ran against: (1) the pre-extracted fields_enum data, (2) targeted `jq`
queries against the OpenAPI spec, (3) `claroty.sensor.toml` (canonical TOML pattern),
and (4) `prism-spec-engine/src/spec_parser.rs` + `types.rs` (pagination runtime).

---

## Spike 1 — Vulnerability Primary Key

**Blocks:** S-CLAROTY-VULNS-001 (G1)  
**DTU status:** EXISTS (G1 = YES per plan)

### Finding

The Vulnerability `fields_enum` (32 fields, confirmed via jq against
`Vulnerability__fields_enum`) contains NO `id` field. The fields are:
`name`, `vulnerability_type`, `cve_ids`, `cvss_v2_score`, etc.

The `GetVulnerabilitiesResponse` OpenAPI example DOES include `"id": "CJYASHKR"` in
each row — an opaque 8-character alphanumeric Claroty-internal identifier. This field
is returned by the real API but is outside the queryable fields_enum, meaning:
- It cannot be requested via the `fields` projection in the POST body
- It cannot be used in `filter_by` expressions
- It appears to be an always-returned base field (present in the example regardless of projection)

The `device_vulnerability_relations` DeviceVulnerability fields_enum (214 fields)
includes BOTH `vulnerability_id` (the CJYASHKR-format internal ID) and
`vulnerability_name` (the human-readable name). This means cross-table joins work
via EITHER the opaque ID or the name.

### Decision: `name` is the canonical primary key

**Rationale:**
1. `name` is in the fields_enum (requestable), `id` is not.
2. In the examples: `name = "CVE-2021-31998"` for CVE-based vulnerabilities;
   `name = "ICSMA-21-161-01 (ZOLL Defibrillator Dashboard)"` for advisories.
   These are the industry-standard identifiers for the respective vulnerability.
3. `cve_ids` is an array (Json type) — cannot serve as a scalar PK.
4. Cross-table join via `vulnerability_name` in device_vulnerability_relations is
   available and does not require the opaque ID.
5. OCSF vulnerability_finding: `name` maps to `finding_info.title`.

**Secondary identifier:** The opaque Claroty `id` (CJYASHKR format) appears in the
raw API response outside the fields projection. Add it via `source_path = "$.id"` with
no `ocsf_field` (Tier-2, raw_extensions). This enables joins via `vulnerability_id` in
device_vulnerability_relations if needed. Verify it is present on the live monroe sensor
before making it REQUIRED — mark it optional (no REQUIRED option) until confirmed.

### First-cut column set for `claroty_vulnerabilities`

`ocsf_class = "vulnerability_finding"` (class_uid 2002 — arm ALREADY EXISTS)  
`response_path = "$.vulnerabilities"` (envelope key: `vulnerabilities`, `count` optional-nullable)  
Pagination: `type = "offset_limit"`, `page_size = 1000` (count is nullable but present)

| Column | ColumnType | ocsf_field | Options | Notes |
|--------|-----------|------------|---------|-------|
| `name` | String | `finding_info.title` | REQUIRED | Canonical PK — CVE ID or advisory title |
| `vulnerability_type` | String | — (Tier-2) | | "Platform", "Clinical", "Configuration", etc. |
| `cve_ids` | Json | — (Tier-2) | | Array of CVE IDs; Json for array |
| `cvss_v3_score` | Float | — (Tier-2) | | Primary CVSS v3 base score |
| `cvss_v3_exploitability_subscore` | Float | — (Tier-2) | | |
| `cvss_v3_vector_string` | String | — (Tier-2) | | |
| `cvss_v2_score` | Float | — (Tier-2) | | CVSS v2 fallback |
| `description` | String | `message` | | Vulnerability description text |
| `is_known_exploited` | Boolean | — (Tier-2) | | CISA KEV indicator |
| `affected_devices_count` | Integer | — (Tier-2) | | Total devices affected |
| `affected_ot_devices_count` | Integer | — (Tier-2) | | OT-specific device count |
| `published_date` | Datetime | — (Tier-2) | | ISO 8601; ADR-028 §D8-B implicit iso8601 default |
| `epss_score` | Float | — (Tier-2) | | EPSS exploit probability |
| `adjusted_vulnerability_score` | Float | — (Tier-2) | | Claroty composite risk score |
| `adjusted_vulnerability_score_level` | String | — (Tier-2) | | "High" / "Medium" / "Low" |
| `exploits_count` | Integer | — (Tier-2) | | Known exploit count |
| `source_name` | String | — (Tier-2) | | NVD, ICS-CERT, etc. |
| `source_url` | String | — (Tier-2) | | Advisory URL |

**Optional secondary column (verify on live sensor before making REQUIRED):**

| Column | ColumnType | source_path | Notes |
|--------|-----------|-------------|-------|
| `id` | String | `$.id` | Opaque Claroty internal ID (CJYASHKR format); enables vulnerability_id joins in device_vulnerability_relations; NOT in fields_enum — captured via source_path only, not in fields projection |

---

## Spike 2 — OT-Events OCSF Class Decision

**Blocks:** S-CLAROTY-OT-EVENTS-001 (G2)  
**DTU status:** NONE (G2 deferred; near-term tests run against live monroe sensor)

### Finding

OTActivityEvent fields_enum (23 fields) is network-5-tuple-heavy:
`source_ip`, `dest_ip`, `protocol`, `dest_port`, `source_port`, `ip_protocol` (6 network fields).
Remaining fields: `detection_time`, `event_type`, `event_id`, `description`,
`related_alert_ids`, `dest_asset_id`, `source_asset_id`, `dest_device_type`,
`source_device_type`, `dest_device_name`, `source_device_name`, `dest_site_name`,
`source_site_name`, `dest_network`, `source_network`, `source_username`, `mode`.

`event_type` examples from the API description: "Configuration Upload",
"Configuration Download" — these are monitored OT protocol operations, not just
raw network captures. The `related_alert_ids` field links events to Claroty alerts,
showing these are surfaced as part of the detection/monitoring workflow.

**Option A (network_activity/4001):** Semantically tighter for the network 5-tuple.
Requires a NEW const `CLASS_UID_NETWORK_ACTIVITY: u32 = 4001` in `class_selector.rs`,
plus new match arms in both `select_by_class_name` and `select`.

**Option B (detection_finding/2004):** Pragmatic. Reuses the existing arm.
The 6 network 5-tuple fields become Tier-2 columns (aggregate into `raw_extensions`
under `ocsf_column_naming = true`). Core OCSF field mappings: `detection_time → time`,
`event_id → finding_info.uid`, `event_type → activity_name`, `description → message`.

### Decision: Option B — detection_finding/2004

**Rationale:**
1. The governing plan explicitly states "NO new OCSF `class_selector` arms required
   (pragmatic mappings)" as a design constraint. This is not a suggestion.
2. The events ARE detections: Claroty's OT visibility platform surfaces them as
   "monitored/detected OT activity" — they appear on the OT Activity monitoring page
   alongside alerts. `related_alert_ids` is the definitive signal that these events
   are part of the alert/detection workflow.
3. Under `ocsf_column_naming = true`, Tier-2 columns (those without `ocsf_field`)
   aggregate into `raw_extensions`. All 6 network 5-tuple fields map cleanly as Tier-2.
   They remain queryable via `raw_extensions.source_ip` etc. — no data is lost.
4. Adding a new class_selector arm is not a trivial scope change: it requires a new
   const, two new match arms, and new test coverage in `bc_2_02_012_class_selector.rs`.
   That scope is not justified by an incremental semantic improvement that produces
   no behavioral difference for MSSP PrismQL users.

**No REQUIRES-HUMAN-ADJUDICATION.** Option B is clean and aligns with the plan.

### First-cut column set for `claroty_ot_activity_events`

`ocsf_class = "detection_finding"` (class_uid 2004 — arm ALREADY EXISTS)  
`response_path = "$.ot_activity_events"` (envelope key confirmed in extract)  
Pagination: `type = "offset_limit"`, `page_size = 1000`

| Column | ColumnType | ocsf_field | Options | Notes |
|--------|-----------|------------|---------|-------|
| `event_id` | Integer | `finding_info.uid` | REQUIRED | "Platform unique Event ID"; integer field |
| `detection_time` | Datetime | `time` | | ISO 8601; ADR-028 §D8-B implicit default |
| `event_type` | String | `activity_name` | | "Configuration Upload" etc. |
| `description` | String | `message` | | Event description text |
| `source_ip` | String | — (Tier-2) | | Source IP address |
| `dest_ip` | String | — (Tier-2) | | Destination IP address |
| `protocol` | String | — (Tier-2) | | OT protocol e.g. "CIP", "Modbus" |
| `dest_port` | Integer | — (Tier-2) | | Destination port number |
| `source_port` | Integer | — (Tier-2) | | Source port number |
| `ip_protocol` | String | — (Tier-2) | | IP protocol e.g. "TCP", "UDP" |
| `source_asset_id` | String | — (Tier-2) | | Claroty source asset ID |
| `dest_asset_id` | String | — (Tier-2) | | Claroty dest asset ID |
| `source_device_name` | String | — (Tier-2) | | |
| `dest_device_name` | String | — (Tier-2) | | |
| `source_device_type` | String | — (Tier-2) | | e.g. "Engineering Station" |
| `dest_device_type` | String | — (Tier-2) | | e.g. "PLC" |
| `source_site_name` | String | — (Tier-2) | | Site of source device |
| `dest_site_name` | String | — (Tier-2) | | Site of dest device |
| `source_username` | String | — (Tier-2) | | OT user who initiated event |
| `related_alert_ids` | Json | — (Tier-2) | | Array of related Claroty alert IDs |
| `mode` | String | — (Tier-2) | | Mode Change event target mode |

---

## Spike 3 — Organization-Policy Nested Field Types

**Blocks:** S-CLAROTY-ORGPOLICY-001 (G5)  
**DTU status:** NONE for all 4 tables (deferred post-v1)  
**OCSF class:** `entity_management` (class_uid 3004 — arm ALREADY EXISTS)  
**Pagination:** `type = "offset_limit"`, `page_size = 1000` (all 4 endpoints have `count`)

### Nested-field classification principle

Fields that are arrays-of-scalars (e.g., `related_alerts_ids` = list of integers/UUIDs)
or arrays-of-objects (e.g., `device_conditions`, `communication_conditions`,
`applied_zone_pairs`, `applied_group_pairs`) → **Json**.

Fields that represent a COUNT of devices → **Integer** (same pattern as
`devices_count: Integer` in the existing alerts table).

### Table A: `claroty_organization_zones`

`response_path = "$.organization_zones"`

| Column | ColumnType | ocsf_field | Options | Notes |
|--------|-----------|------------|---------|-------|
| `zone_name` | String | `name` | REQUIRED | PK — unique zone name |
| `zone_description` | String | `comment` | | |
| `zone_source` | String | — (Tier-2) | | "Custom", "Recommended", etc. |
| `priority` | Integer | — (Tier-2) | | Zone priority order |
| `enabled` | Boolean | `status_code` | | Zone active/inactive |
| `device_conditions` | **Json** | — (Tier-2) | | Array of device filter condition objects |
| `attributed_devices` | Integer | — (Tier-2) | | Count of devices matched by conditions |
| `exportable_attributed_devices` | Integer | — (Tier-2) | | Exportable device count subset |
| `created_time` | Datetime | — (Tier-2) | | ISO 8601 |
| `last_update` | Datetime | — (Tier-2) | | ISO 8601 |
| `updated_by` | String | `actor.user.name` | | Email or username |

### Table B: `claroty_organization_zone_policies`

`response_path = "$.organization_zone_policies"`

| Column | ColumnType | ocsf_field | Options | Notes |
|--------|-----------|------------|---------|-------|
| `policy_name` | String | `name` | REQUIRED | PK — unique policy name |
| `policy_source` | String | — (Tier-2) | | "Custom", "Recommended", etc. |
| `policy_action` | String | `activity_name` | | "Allow" / "Deny" |
| `communication_conditions` | **Json** | — (Tier-2) | | Array of src/dst zone condition objects |
| `matching_devices` | Integer | — (Tier-2) | | Count of devices matching policy |
| `should_generate_alerts` | Boolean | — (Tier-2) | | Alert generation flag |
| `alert_use_case` | String | — (Tier-2) | | Alert category when triggered |
| `policy_notes` | String | `comment` | | Analyst notes |
| `related_alerts_ids` | **Json** | — (Tier-2) | | Array of triggered alert IDs |
| `applied_zone_pairs` | **Json** | — (Tier-2) | | Array of {src_zone, dst_zone} pair objects |
| `created_time` | Datetime | — (Tier-2) | | ISO 8601 |
| `last_updated` | Datetime | — (Tier-2) | | ISO 8601 |
| `updated_by` | String | `actor.user.name` | | |

### Table C: `claroty_organization_firewall_groups`

`response_path = "$.organization_firewall_groups"`

| Column | ColumnType | ocsf_field | Options | Notes |
|--------|-----------|------------|---------|-------|
| `firewall_group_name` | String | `name` | REQUIRED | PK — unique firewall group name |
| `firewall_group_description` | String | `comment` | | |
| `firewall_group_source` | String | — (Tier-2) | | "Custom", "Recommended", etc. |
| `priority` | Integer | — (Tier-2) | | Firewall group priority order |
| `enabled` | Boolean | `status_code` | | Group active/inactive |
| `device_conditions` | **Json** | — (Tier-2) | | Array of device filter condition objects |
| `attributed_devices` | Integer | — (Tier-2) | | Count of devices matched by conditions |
| `exportable_attributed_devices` | Integer | — (Tier-2) | | Exportable device count subset |
| `created_time` | Datetime | — (Tier-2) | | ISO 8601 |
| `last_update` | Datetime | — (Tier-2) | | ISO 8601 |
| `updated_by` | String | `actor.user.name` | | |

### Table D: `claroty_organization_firewall_policies`

`response_path = "$.organization_firewall_policies"`

| Column | ColumnType | ocsf_field | Options | Notes |
|--------|-----------|------------|---------|-------|
| `policy_name` | String | `name` | REQUIRED | PK — unique policy name |
| `policy_source` | String | — (Tier-2) | | |
| `policy_action` | String | `activity_name` | | "Allow" / "Deny" |
| `communication_conditions` | **Json** | — (Tier-2) | | Array of src/dst firewall-group condition objects |
| `matching_devices` | Integer | — (Tier-2) | | Count of devices matching policy |
| `should_generate_alerts` | Boolean | — (Tier-2) | | |
| `alert_use_case` | String | — (Tier-2) | | |
| `policy_notes` | String | `comment` | | |
| `related_alerts_ids` | **Json** | — (Tier-2) | | Array of triggered alert IDs |
| `applied_group_pairs` | **Json** | — (Tier-2) | | Array of {src_group, dst_group} pair objects |
| `created_time` | Datetime | — (Tier-2) | | ISO 8601 |
| `last_updated` | Datetime | — (Tier-2) | | ISO 8601 |
| `updated_by` | String | `actor.user.name` | | |

### Json column summary

The following fields are nested arrays/objects and MUST be `column_type = "json"`:

| Table | Json columns |
|-------|-------------|
| organization_zones | `device_conditions` |
| organization_zone_policies | `communication_conditions`, `related_alerts_ids`, `applied_zone_pairs` |
| organization_firewall_groups | `device_conditions` |
| organization_firewall_policies | `communication_conditions`, `related_alerts_ids`, `applied_group_pairs` |

---

## Spike 4 — ACL Pagination Anomaly

**Blocks:** S-CLAROTY-ACLPOLICY-001 (G6)  
**DTU status:** NONE (G6 deferred post-v1)  
**OCSF class:** `entity_management` (class_uid 3004 — arm ALREADY EXISTS)

### Finding: endpoint is non-paginated AND requires a mandatory request parameter

From `GetOrganizationAclPoliciesRequest` schema (confirmed via jq):
1. **No `offset`/`limit` fields** in the request schema. Cannot paginate.
2. **Response has no `count` field** (confirmed in extract). Envelope key only:
   `{"organization_acl_policies": [...]}`.
3. **`policy_acl_syntax` is a REQUIRED request field** (`required: ["policy_acl_syntax"]`).
   Valid values: "Cisco dACL", "AireOS", "ArubaOS-Switch", "ArubaOS-CX".
   This parameter is NOT in the fields_enum — it is a request-level parameter that
   controls the format of the `policy_acl` response field (ACL text syntax).
4. `fields` in the request is optional-nullable (unlike alerts/devices where it is required
   minItems:1). Can be omitted or set to the desired subset.

**Comparison to existing paginated tables:**

Standard pattern (alerts, devices, audit_logs, device_alert_relations):
- POST body: `{"fields": [...], "offset": N, "limit": 1000}`
- Response: `{"<key>": [...], "count": N}`
- TOML: `type = "offset_limit"`, `page_size = 1000`

ACL pattern:
- POST body: `{"policy_acl_syntax": "Cisco dACL", "fields": [...]}`
  (no offset/limit; no pagination possible)
- Response: `{"organization_acl_policies": [...]}` (no count field)
- TOML: `type = "none"` (explicitly declared; documented in spec_parser.rs test fixtures)

The `type = "none"` value is a valid `PaginationConfig` variant in the spec engine
(`PaginationConfig::None` in `prism-spec-engine/src/spec_parser.rs`, decorated with
`#[serde(tag = "type", rename_all = "snake_case")]`). Alternatively, omitting the
`[tables.steps.pagination]` section entirely also produces `PaginationType::None`
(via `.unwrap_or(PaginationType::None)` in `types.rs`). **Decision: explicitly
declare `type = "none"` for clarity and self-documentation.**

**Body template decision for `policy_acl_syntax`:**

The ACL text syntax is hardcoded to `"Cisco dACL"` for v1. Rationale:
- "Cisco dACL" is the API example default and the most common MSSP network equipment
  syntax
- Making it configurable per-table would require new TOML schema extensions not
  planned in scope
- The `policy_acl` column will contain ACL text in Cisco dACL format
- A follow-up story (deferred to post-v1) can introduce a table-level parameter for
  syntax selection if MSSP tenants need ArubaOS or AireOS formats

### First-cut column set for `claroty_organization_acl_policies`

Source: `OrganizationAclPolicyResponseItem` concrete schema (11 fields, all anyOf [type, null]).

`response_path = "$.organization_acl_policies"`  
`body_template = '{"policy_acl_syntax": "Cisco dACL", "fields": ["policy_id", "policy_name", "policy_source", "applied_models", "matching_devices", "policy_acl_type", "policy_acl", "policy_creation_date", "policy_last_updated", "policy_updated_by", "policy_notes"]}'`  
Pagination: `type = "none"` (explicitly declared)

| Column | ColumnType | ocsf_field | Options | Notes |
|--------|-----------|------------|---------|-------|
| `policy_id` | String | `metadata.uid` | REQUIRED | UUID-format PK; anyOf [uuid, null] in schema |
| `policy_name` | String | `name` | | Human-readable policy name |
| `policy_source` | String | — (Tier-2) | | "Custom" or system source |
| `policy_acl_type` | String | — (Tier-2) | | ACL syntax ("Cisco dACL" etc.) |
| `policy_acl` | String | — (Tier-2) | | Raw ACL text (multi-line) |
| `applied_models` | **Json** | — (Tier-2) | | Array of device model strings |
| `matching_devices` | Integer | — (Tier-2) | | Count of matching devices |
| `policy_creation_date` | Datetime | — (Tier-2) | | ISO 8601 (format: "date-time") |
| `policy_last_updated` | Datetime | — (Tier-2) | | ISO 8601 (format: "date-time") |
| `policy_updated_by` | String | `actor.user.name` | | Email or username |
| `policy_notes` | String | `comment` | | |

**Pagination anomaly handling summary:**
- Do NOT inject `offset`/`limit` into the POST body (no `type = "offset_limit"`)
- Do NOT expect `count` in the response
- Response is the full ACL policy list in a single request
- `type = "none"` blocks the OffsetLimit POST-body injection in
  `pipeline.rs::build_request` (BC-2.16.002 §Postconditions)

---

## Overall Verdict

### DTU status per gap

| Gap | Table | DTU exists? | Near-term test target |
|-----|-------|-------------|----------------------|
| G1 | claroty_vulnerabilities | **YES** | Live monroe + DTU |
| G2 | claroty_ot_activity_events | NO | Live monroe only |
| G3 | claroty_device_vulnerability_relations | NO | Live monroe only |
| G4 | claroty_servers | NO | Live monroe only |
| G5 | claroty_org_policy (4 tables) | NO | Live monroe only |
| G6 | claroty_organization_acl_policies | NO | Live monroe only |

SAP-2 DTU-parity probe is N/A for G2–G6 until deferred DTU stories run (per plan D-2200).

### New OCSF class_selector arms required?

**NO. Zero new arms required across all 10 expansion tables.**

All gaps map to existing `select_by_class_name` arms in
`prism-ocsf/src/class_selector.rs`:

| OCSF class | class_uid | Tables using it | Arm status |
|------------|-----------|-----------------|------------|
| `vulnerability_finding` | 2002 | G1 (vulnerabilities), G3 (device_vulnerability_relations) | EXISTS |
| `detection_finding` | 2004 | G2 (ot_activity_events) | EXISTS |
| `inventory_info` | 5001 | G4 (servers) | EXISTS |
| `entity_management` | 3004 | G5 ×4 org-policy tables, G6 (acl_policies) | EXISTS |

### Spike 2 adjudication gate

**REQUIRES-HUMAN-ADJUDICATION: NO.** Spike 2 recommends Option B
(detection_finding/2004). This is consistent with the plan's governing constraint
("NO new OCSF class_selector arms"). Option A was evaluated and rejected. No fork.
