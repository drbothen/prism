---
document_type: validation-matrix
producer: product-owner
version: "0.3"
project: prism
timestamp: "2026-09-07"
develop_head: "362e4f85"
naming_regime: POST-ROUTING-001 (ocsf_column_naming=true)
routing_001_status: merged (PR #242, develop@3f1e66179)
tables_in_scope: 14 (all Claroty xDome tables: original 4 + G1-G6 expansion 10)
scope: Claroty xDome v1 release gate — full SOC-analyst path against real xDome tenant, all 14 tables
note: >
  This is an OPEN, human-facing planning artifact. NOT holdout scenarios.
  Do NOT read .factory/holdout-scenarios/ when working with this file.
supersedes: >
  v0.2 (2026-08-21) — 4-table coverage (alerts, audit_logs, devices, device_alert_relations).
  v0.3 (2026-09-07) — AUTHORITATIVE 14-table coverage, all Claroty xDome tables.
supplements:
  - .factory/objectives/xdome-v1-validation/soc-analyst-qa-catalog.md
traces_to:
  - .factory/objectives/xdome-v1-validation/feature-inventory.md
  - .factory/specs/behavioral-contracts/BC-2.16.003-column-to-ocsf-mapping.md
  - .factory/specs/behavioral-contracts/BC-2.16.002-multi-step-fetch-pipeline.md
  - .factory/specs/behavioral-contracts/BC-2.16.015-claroty-vulnerabilities-table.md
  - .factory/specs/behavioral-contracts/BC-2.16.016-claroty-ot-activity-events-table.md
  - .factory/specs/behavioral-contracts/BC-2.16.017-claroty-device-vulnerability-relations-table.md
  - .factory/specs/behavioral-contracts/BC-2.16.018-claroty-servers-table.md
  - .factory/specs/behavioral-contracts/BC-2.16.019-claroty-server-interfaces-table.md
  - .factory/specs/behavioral-contracts/BC-2.16.020-claroty-org-zone-domain.md
  - .factory/specs/behavioral-contracts/BC-2.16.021-claroty-org-firewall-domain.md
  - .factory/specs/behavioral-contracts/BC-2.16.022-claroty-org-acl-policies.md
  - .factory/stories/S-ADR058-OCSF-ROUTING-001-sensor-spec-ocsf-field-name-routing.md
  - .factory/stories/S-CLAROTY-VULNS-001-claroty-vulnerabilities-toml-table.md
  - .factory/stories/S-CLAROTY-OT-EVENTS-001-claroty-ot-activity-events-toml-table.md
  - .factory/stories/S-CLAROTY-DEVVULNREL-001-claroty-device-vulnerability-relations-toml-table.md
  - .factory/stories/S-CLAROTY-SERVERS-001-claroty-servers-and-server-interfaces-toml-tables.md
  - .factory/stories/S-CLAROTY-ORGPOLICY-001-claroty-org-policy-tables.md
  - .factory/stories/S-CLAROTY-ACLPOLICY-001-claroty-org-acl-policies.md
  - crates/prism-sensors/specs/claroty.sensor.toml
---

# Claroty xDome v1 — RELEASE-GATE Live Validation Matrix

> **AUTHORITATIVE 14-TABLE VERSION (v0.3).** This document supersedes v0.2 (4-table coverage).
> All 14 Claroty xDome tables are covered: the original 4 (alerts, audit_logs, devices,
> device_alert_relations) plus the 10 expansion tables from stories G1–G6.

> **Naming regime:** All column names reflect **POST-ROUTING-001** Arrow field names
> (`ocsf_column_naming = true`, merged PR #242, develop@3f1e66179). Running any query
> against a pre-ROUTING-001 binary will produce false failures on Field Mapping and Query
> checks.

> **Story-gate notice:** Expansion table items (FM-021+, QU-013+) are gated on the
> respective story merge:
> - G1 (claroty_vulnerabilities): S-CLAROTY-VULNS-001 (status: merged)
> - G2 (claroty_ot_activity_events): S-CLAROTY-OT-EVENTS-001
> - G3 (claroty_device_vulnerability_relations): S-CLAROTY-DEVVULNREL-001
> - G4 (claroty_servers + claroty_server_interfaces): S-CLAROTY-SERVERS-001
> - G5 (4 org-policy tables): S-CLAROTY-ORGPOLICY-001
> - G6 (claroty_organization_acl_policies): S-CLAROTY-ACLPOLICY-001

> **Credentials:** All live checks require a real Claroty xDome tenant with a valid
> bearer token stored via the credential CLI per AD-017. Credential values MUST NOT appear
> in AI context, logs, or any artifact. Use the opaque reference model exclusively.

> **DTU vs LIVE:** Each item is tagged `DTU` (testable against `prism-dtu-claroty`),
> `LIVE` (requires real xDome tenant), or `DTU+LIVE` (DTU for correctness, LIVE for
> compatibility confirmation). Items tagged `LIVE-REQUIRED` cannot be substituted with DTU.
> G2–G6 expansion tables have no DTU; all checks are `LIVE-REQUIRED`.

---

## Preamble — POST-ROUTING-001 Arrow Field Name Reference

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
| `timestamp` | `time` | Datetime | Tier-1 | INDEX — push-down eligible by both `timestamp` and Arrow `time` |
| `details` | `message` | String | Tier-1 | unchanged |
| `username` | `actor_user_uid` | String | Tier-1 | unchanged |
| `note` | `comment` | String | Tier-1 | unchanged |
| `id` | `metadata_uid` | String | Tier-1 | KF-05 RESOLVED: ocsf_field = "metadata.uid"; Tier-1 retention required |
| `category` | (→ raw_extensions) | — | Tier-2 | KF-11: ocsf_field removed |
| (synthesized) | `raw_extensions` | Json | — | keys: "category" |
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
| `ip_list` | (→ raw_extensions) | — | Tier-2 | compact JSON-list string |
| `mac_list` | (→ raw_extensions) | — | Tier-2 | compact JSON-list string |
| `network_list` | (→ raw_extensions) | — | Tier-2 | compact JSON-list string |
| `vlan_list` | (→ raw_extensions) | — | Tier-2 | compact JSON-list string (integer elements stringified) |
| `purdue_level` | (→ raw_extensions) | — | Tier-2 | |
| `site_name` | (→ raw_extensions) | — | Tier-2 | |
| `device_subcategory` | (→ raw_extensions) | — | Tier-2 | |
| `device_type_family` | (→ raw_extensions) | — | Tier-2 | |
| `criticality` | (→ raw_extensions) | — | Tier-2 | |
| `is_online` | (→ raw_extensions) | — | Tier-2 | Boolean value in JSON |
| `manufacturer` | (→ raw_extensions) | — | Tier-2 | |
| `model` | (→ raw_extensions) | — | Tier-2 | |
| (synthesized) | `raw_extensions` | Json | — | all 12 Tier-2 keys |
| (synthesized) | `class_uid` | Integer | — | = 5001 |
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

### claroty_vulnerabilities (ocsf_class: vulnerability_finding, class_uid: 2002)

**Story gate:** S-CLAROTY-VULNS-001 (Wave A G1). DTU: YES (prism-dtu-claroty vulnerabilities route).

| TOML col.name | Arrow field name (POST-ROUTING-001) | Type | Tier | Note |
|---|---|---|---|---|
| `name` | `finding_info_title` | String | Tier-1 | PK — CVE ID or advisory title; ocsf_field = "finding_info.title" |
| `description` | `message` | String | Tier-1 | ocsf_field = "message" |
| `id` | (→ raw_extensions via source_path) | — | Tier-2 | Opaque Claroty internal ID (CJYASHKR format); NOT in fields_enum; optional |
| `vulnerability_type` | (→ raw_extensions) | — | Tier-2 | "Platform", "Clinical", "Configuration", etc. |
| `cve_ids` | (→ raw_extensions) | Json | Tier-2 | Array of CVE IDs; stored as compact JSON-list string |
| `cvss_v3_score` | (→ raw_extensions) | — | Tier-2 | Primary CVSS v3 base score |
| `cvss_v3_exploitability_subscore` | (→ raw_extensions) | — | Tier-2 | |
| `cvss_v3_vector_string` | (→ raw_extensions) | — | Tier-2 | |
| `cvss_v2_score` | (→ raw_extensions) | — | Tier-2 | |
| `is_known_exploited` | (→ raw_extensions) | — | Tier-2 | CISA KEV indicator |
| `affected_devices_count` | (→ raw_extensions) | — | Tier-2 | |
| `affected_ot_devices_count` | (→ raw_extensions) | — | Tier-2 | |
| `published_date` | (→ raw_extensions) | — | Tier-2 | ISO 8601 datetime string in blob |
| `epss_score` | (→ raw_extensions) | — | Tier-2 | |
| `adjusted_vulnerability_score` | (→ raw_extensions) | — | Tier-2 | |
| `adjusted_vulnerability_score_level` | (→ raw_extensions) | — | Tier-2 | "High"/"Medium"/"Low" |
| `exploits_count` | (→ raw_extensions) | — | Tier-2 | |
| `source_name` | (→ raw_extensions) | — | Tier-2 | NVD, ICS-CERT, etc. |
| `source_url` | (→ raw_extensions) | — | Tier-2 | Advisory URL |
| (synthesized) | `raw_extensions` | Json | — | 17 Tier-2 keys (including optional `id` via source_path) |
| (synthesized) | `class_uid` | Integer | — | = 2002 |
| (synthesized) | `_sensor` | String | — | = "claroty" |

### claroty_ot_activity_events (ocsf_class: detection_finding, class_uid: 2004)

**Story gate:** S-CLAROTY-OT-EVENTS-001 (Wave A G2). DTU: NO — LIVE-REQUIRED.

| TOML col.name | Arrow field name (POST-ROUTING-001) | Type | Tier | Note |
|---|---|---|---|---|
| `event_id` | `finding_info_uid` | String | Tier-1 | Integer field normalized to String; ocsf_field = "finding_info.uid"; REQUIRED |
| `detection_time` | `time` | Datetime | Tier-1 | ocsf_field = "time" |
| `event_type` | `activity_name` | String | Tier-1 | "Configuration Upload", etc.; ocsf_field = "activity_name" |
| `description` | `message` | String | Tier-1 | ocsf_field = "message" |
| `source_ip` | (→ raw_extensions) | — | Tier-2 | |
| `dest_ip` | (→ raw_extensions) | — | Tier-2 | |
| `protocol` | (→ raw_extensions) | — | Tier-2 | OT protocol e.g. "CIP", "Modbus" |
| `dest_port` | (→ raw_extensions) | — | Tier-2 | |
| `source_port` | (→ raw_extensions) | — | Tier-2 | |
| `ip_protocol` | (→ raw_extensions) | — | Tier-2 | "TCP", "UDP" |
| `source_asset_id` | (→ raw_extensions) | — | Tier-2 | |
| `dest_asset_id` | (→ raw_extensions) | — | Tier-2 | |
| `source_device_name` | (→ raw_extensions) | — | Tier-2 | |
| `dest_device_name` | (→ raw_extensions) | — | Tier-2 | |
| `source_device_type` | (→ raw_extensions) | — | Tier-2 | |
| `dest_device_type` | (→ raw_extensions) | — | Tier-2 | |
| `source_site_name` | (→ raw_extensions) | — | Tier-2 | |
| `dest_site_name` | (→ raw_extensions) | — | Tier-2 | |
| `source_username` | (→ raw_extensions) | — | Tier-2 | OT user who initiated event |
| `related_alert_ids` | (→ raw_extensions) | Json | Tier-2 | Array of related Claroty alert IDs; compact JSON-list string |
| `mode` | (→ raw_extensions) | — | Tier-2 | Mode Change event target mode |
| (synthesized) | `raw_extensions` | Json | — | 17 Tier-2 keys |
| (synthesized) | `class_uid` | Integer | — | = 2004 |
| (synthesized) | `_sensor` | String | — | = "claroty" |

### claroty_device_vulnerability_relations (ocsf_class: vulnerability_finding, class_uid: 2002)

**Story gate:** S-CLAROTY-DEVVULNREL-001 (Wave B G3). DTU: NO — LIVE-REQUIRED. Depends on S-CLAROTY-VULNS-001.

| TOML col.name | Arrow field name (POST-ROUTING-001) | Type | Tier | Note |
|---|---|---|---|---|
| `vulnerability_name` | `finding_info_title` | String | Tier-1 | Composite PK element; ocsf_field = "finding_info.title"; REQUIRED; join key to claroty_vulnerabilities |
| `device_vulnerability_detection_date` | `time` | Datetime | Tier-1 | ocsf_field = "time" |
| `device_uid` | (→ raw_extensions) | — | Tier-2 | Composite PK element; join key to claroty_devices |
| `vulnerability_id` | (→ raw_extensions) | — | Tier-2 | Opaque Claroty internal ID; enables vulnerability_id joins |
| `vulnerability_type` | (→ raw_extensions) | — | Tier-2 | |
| `vulnerability_cvss_v3_score` | (→ raw_extensions) | — | Tier-2 | CVSS v3 score for triage |
| `vulnerability_is_known_exploited` | (→ raw_extensions) | — | Tier-2 | CISA KEV indicator per-device |
| `vulnerability_epss_score` | (→ raw_extensions) | — | Tier-2 | Exploit probability |
| `device_vulnerability_resolution_date` | (→ raw_extensions) | — | Tier-2 | ISO 8601 when resolved; nullable |
| `patch_install_date` | (→ raw_extensions) | — | Tier-2 | ISO 8601 when patched; nullable |
| `patch_status` | (→ raw_extensions) | — | Tier-2 | Patch status string |
| (synthesized) | `raw_extensions` | Json | — | 11 Tier-2 keys |
| (synthesized) | `class_uid` | Integer | — | = 2002 |
| (synthesized) | `_sensor` | String | — | = "claroty" |

> **Envelope anomaly:** `response_path = "$.devices_vulnerabilities"` — the envelope key
> DIVERGES from the table name. Verify `$.devices_vulnerabilities` exactly (NOT `$.device_vulnerability_relations`).

### claroty_servers (ocsf_class: inventory_info, class_uid: 5001)

**Story gate:** S-CLAROTY-SERVERS-001 (Wave C G4). DTU: NO — LIVE-REQUIRED.

| TOML col.name | Arrow field name (POST-ROUTING-001) | Type | Tier | Note |
|---|---|---|---|---|
| `server_name` | `device_name` | String | Tier-1 | ocsf_field = "device.name"; REQUIRED |
| `server_status` | `status_code` | String | Tier-1 | ocsf_field = "status_code" |
| `server_location` | (→ raw_extensions) | — | Tier-2 | |
| `site_id` | (→ raw_extensions) | — | Tier-2 | |
| `model` | (→ raw_extensions) | — | Tier-2 | |
| `os_version` | (→ raw_extensions) | — | Tier-2 | |
| `serial_number` | (→ raw_extensions) | — | Tier-2 | |
| `num_of_interfaces` | (→ raw_extensions) | — | Tier-2 | Integer count |
| `management_ip` | (→ raw_extensions) | — | Tier-2 | |
| `idrac_ip` | (→ raw_extensions) | — | Tier-2 | |
| `management_mac` | (→ raw_extensions) | — | Tier-2 | |
| `uptime_days` | (→ raw_extensions) | Float | Tier-2 | Fractional float (e.g. 667.234 confirmed in OpenAPI example) |
| `avg_traffic_past_month_mbps` | (→ raw_extensions) | — | Tier-2 | |
| `avg_traffic_past_week_mbps` | (→ raw_extensions) | — | Tier-2 | |
| `avg_traffic_past_hour_mbps` | (→ raw_extensions) | — | Tier-2 | |
| `num_of_open_incidents` | (→ raw_extensions) | — | Tier-2 | |
| `notes` | (→ raw_extensions) | — | Tier-2 | |
| (synthesized) | `raw_extensions` | Json | — | 15 Tier-2 keys |
| (synthesized) | `class_uid` | Integer | — | = 5001 |
| (synthesized) | `_sensor` | String | — | = "claroty" |

### claroty_server_interfaces (ocsf_class: inventory_info, class_uid: 5001)

**Story gate:** S-CLAROTY-SERVERS-001 (Wave C G4). DTU: NO — LIVE-REQUIRED. SEPARATE endpoint from /api/v1/servers/ — uses /api/v1/server_interfaces/.

| TOML col.name | Arrow field name (POST-ROUTING-001) | Type | Tier | Note |
|---|---|---|---|---|
| `server_name` | `device_name` | String | Tier-1 | ocsf_field = "device.name"; REQUIRED; join key to claroty_servers |
| `interface_status` | `status_code` | String | Tier-1 | ocsf_field = "status_code" |
| `interface_name` | (→ raw_extensions) | — | Tier-2 | Composite PK element with server_name; NOT REQUIRED (nullable = degraded-not-dropped) |
| `interface_type` | (→ raw_extensions) | — | Tier-2 | |
| `ip_address` | (→ raw_extensions) | — | Tier-2 | |
| `mac_address` | (→ raw_extensions) | — | Tier-2 | |
| `subnet` | (→ raw_extensions) | — | Tier-2 | |
| `vlan` | (→ raw_extensions) | — | Tier-2 | |
| `is_monitored` | (→ raw_extensions) | — | Tier-2 | Boolean in JSON |
| `interface_description` | (→ raw_extensions) | — | Tier-2 | |
| (synthesized) | `raw_extensions` | Json | — | 8 Tier-2 keys |
| (synthesized) | `class_uid` | Integer | — | = 5001 |
| (synthesized) | `_sensor` | String | — | = "claroty" |

### claroty_organization_zones (ocsf_class: entity_management, class_uid: 3004)

**Story gate:** S-CLAROTY-ORGPOLICY-001 (Wave C G5). DTU: NO — LIVE-REQUIRED. BC: BC-2.16.020 (Zone Domain).

| TOML col.name | Arrow field name (POST-ROUTING-001) | Type | Tier | Note |
|---|---|---|---|---|
| `zone_name` | `name` | String | Tier-1 | PK; ocsf_field = "name"; REQUIRED |
| `zone_description` | `comment` | String | Tier-1 | ocsf_field = "comment" |
| `enabled` | `status_code` | Boolean | Tier-1 | ocsf_field = "status_code" |
| `updated_by` | `actor_user_name` | String | Tier-1 | ocsf_field = "actor.user.name" |
| `zone_source` | (→ raw_extensions) | — | Tier-2 | "Custom", "Recommended", etc. |
| `priority` | (→ raw_extensions) | — | Tier-2 | Integer zone priority |
| `device_conditions` | (→ raw_extensions) | Json | Tier-2 | Array of device filter condition objects; compact JSON-list string |
| `attributed_devices` | (→ raw_extensions) | — | Tier-2 | Count of devices matched by conditions |
| `exportable_attributed_devices` | (→ raw_extensions) | — | Tier-2 | |
| `created_time` | (→ raw_extensions) | — | Tier-2 | ISO 8601 datetime string in blob |
| `last_update` | (→ raw_extensions) | — | Tier-2 | ISO 8601 datetime string in blob |
| (synthesized) | `raw_extensions` | Json | — | 7 Tier-2 keys |
| (synthesized) | `class_uid` | Integer | — | = 3004 |
| (synthesized) | `_sensor` | String | — | = "claroty" |

### claroty_organization_zone_policies (ocsf_class: entity_management, class_uid: 3004)

**Story gate:** S-CLAROTY-ORGPOLICY-001 (Wave C G5). DTU: NO — LIVE-REQUIRED. BC: BC-2.16.020 (Zone Domain).

| TOML col.name | Arrow field name (POST-ROUTING-001) | Type | Tier | Note |
|---|---|---|---|---|
| `policy_name` | `name` | String | Tier-1 | PK; ocsf_field = "name"; REQUIRED |
| `policy_action` | `activity_name` | String | Tier-1 | "Allow"/"Deny"; ocsf_field = "activity_name" |
| `policy_notes` | `comment` | String | Tier-1 | ocsf_field = "comment" |
| `updated_by` | `actor_user_name` | String | Tier-1 | ocsf_field = "actor.user.name" |
| `policy_source` | (→ raw_extensions) | — | Tier-2 | |
| `communication_conditions` | (→ raw_extensions) | Json | Tier-2 | Array of {src_zone, dst_zone} condition objects; compact JSON-list string |
| `matching_devices` | (→ raw_extensions) | — | Tier-2 | Count of devices matching policy |
| `should_generate_alerts` | (→ raw_extensions) | — | Tier-2 | Boolean in JSON |
| `alert_use_case` | (→ raw_extensions) | — | Tier-2 | |
| `related_alerts_ids` | (→ raw_extensions) | Json | Tier-2 | Array of triggered alert IDs; compact JSON-list string |
| `applied_zone_pairs` | (→ raw_extensions) | Json | Tier-2 | Array of {src_zone, dst_zone} pair objects; compact JSON-list string |
| `created_time` | (→ raw_extensions) | — | Tier-2 | ISO 8601 string in blob |
| `last_updated` | (→ raw_extensions) | — | Tier-2 | ISO 8601 string in blob |
| (synthesized) | `raw_extensions` | Json | — | 9 Tier-2 keys (3 are Json arrays) |
| (synthesized) | `class_uid` | Integer | — | = 3004 |
| (synthesized) | `_sensor` | String | — | = "claroty" |

### claroty_organization_firewall_groups (ocsf_class: entity_management, class_uid: 3004)

**Story gate:** S-CLAROTY-ORGPOLICY-001 (Wave C G5). DTU: NO — LIVE-REQUIRED. BC: BC-2.16.021 (Firewall Domain).

| TOML col.name | Arrow field name (POST-ROUTING-001) | Type | Tier | Note |
|---|---|---|---|---|
| `firewall_group_name` | `name` | String | Tier-1 | PK; ocsf_field = "name"; REQUIRED |
| `firewall_group_description` | `comment` | String | Tier-1 | ocsf_field = "comment" |
| `enabled` | `status_code` | Boolean | Tier-1 | ocsf_field = "status_code" |
| `updated_by` | `actor_user_name` | String | Tier-1 | ocsf_field = "actor.user.name" |
| `firewall_group_source` | (→ raw_extensions) | — | Tier-2 | |
| `priority` | (→ raw_extensions) | — | Tier-2 | Integer group priority |
| `device_conditions` | (→ raw_extensions) | Json | Tier-2 | Array of device filter condition objects; compact JSON-list string |
| `attributed_devices` | (→ raw_extensions) | — | Tier-2 | |
| `exportable_attributed_devices` | (→ raw_extensions) | — | Tier-2 | |
| `created_time` | (→ raw_extensions) | — | Tier-2 | ISO 8601 string in blob |
| `last_update` | (→ raw_extensions) | — | Tier-2 | ISO 8601 string in blob |
| (synthesized) | `raw_extensions` | Json | — | 7 Tier-2 keys |
| (synthesized) | `class_uid` | Integer | — | = 3004 |
| (synthesized) | `_sensor` | String | — | = "claroty" |

### claroty_organization_firewall_policies (ocsf_class: entity_management, class_uid: 3004)

**Story gate:** S-CLAROTY-ORGPOLICY-001 (Wave C G5). DTU: NO — LIVE-REQUIRED. BC: BC-2.16.021 (Firewall Domain).

| TOML col.name | Arrow field name (POST-ROUTING-001) | Type | Tier | Note |
|---|---|---|---|---|
| `policy_name` | `name` | String | Tier-1 | PK; ocsf_field = "name"; REQUIRED |
| `policy_action` | `activity_name` | String | Tier-1 | "Allow"/"Deny"; ocsf_field = "activity_name" |
| `policy_notes` | `comment` | String | Tier-1 | ocsf_field = "comment" |
| `updated_by` | `actor_user_name` | String | Tier-1 | ocsf_field = "actor.user.name" |
| `policy_source` | (→ raw_extensions) | — | Tier-2 | |
| `communication_conditions` | (→ raw_extensions) | Json | Tier-2 | Array of {src_group, dst_group} condition objects; compact JSON-list string |
| `matching_devices` | (→ raw_extensions) | — | Tier-2 | |
| `should_generate_alerts` | (→ raw_extensions) | — | Tier-2 | Boolean in JSON |
| `alert_use_case` | (→ raw_extensions) | — | Tier-2 | |
| `related_alerts_ids` | (→ raw_extensions) | Json | Tier-2 | Array of triggered alert IDs; compact JSON-list string |
| `applied_group_pairs` | (→ raw_extensions) | Json | Tier-2 | Array of {src_group, dst_group} pair objects; compact JSON-list string |
| `created_time` | (→ raw_extensions) | — | Tier-2 | ISO 8601 string in blob |
| `last_updated` | (→ raw_extensions) | — | Tier-2 | ISO 8601 string in blob |
| (synthesized) | `raw_extensions` | Json | — | 9 Tier-2 keys (3 are Json arrays) |
| (synthesized) | `class_uid` | Integer | — | = 3004 |
| (synthesized) | `_sensor` | String | — | = "claroty" |

### claroty_organization_acl_policies (ocsf_class: entity_management, class_uid: 3004)

**Story gate:** S-CLAROTY-ACLPOLICY-001 (Wave C G6). DTU: NO — LIVE-REQUIRED. **Non-paginated:** `type = "none"`; no count field in response; mandatory `policy_acl_syntax = "Cisco dACL"` in POST body. BC: BC-2.16.022.

| TOML col.name | Arrow field name (POST-ROUTING-001) | Type | Tier | Note |
|---|---|---|---|---|
| `policy_id` | `metadata_uid` | String | Tier-1 | UUID-format PK; ocsf_field = "metadata.uid"; REQUIRED |
| `policy_name` | `name` | String | Tier-1 | Human-readable policy name; ocsf_field = "name" |
| `policy_notes` | `comment` | String | Tier-1 | ocsf_field = "comment" |
| `policy_updated_by` | `actor_user_name` | String | Tier-1 | ocsf_field = "actor.user.name" |
| `policy_source` | (→ raw_extensions) | — | Tier-2 | "Custom" or system source |
| `policy_acl_type` | (→ raw_extensions) | — | Tier-2 | ACL syntax ("Cisco dACL" etc.) |
| `policy_acl` | (→ raw_extensions) | — | Tier-2 | Raw ACL text (multi-line) |
| `applied_models` | (→ raw_extensions) | Json | Tier-2 | Array of device model strings; compact JSON-list string |
| `matching_devices` | (→ raw_extensions) | — | Tier-2 | Count of matching devices |
| `policy_creation_date` | (→ raw_extensions) | — | Tier-2 | ISO 8601 string in blob |
| `policy_last_updated` | (→ raw_extensions) | — | Tier-2 | ISO 8601 string in blob |
| (synthesized) | `raw_extensions` | Json | — | 7 Tier-2 keys (including 1 Json array) |
| (synthesized) | `class_uid` | Integer | — | = 3004 |
| (synthesized) | `_sensor` | String | — | = "claroty" |

---

## Gate 1: ONBOARDING

Items cover config/org setup, credential wiring, sensor reachability, and health check against the live tenant.

| ID | What to do | Exact expected result | How to verify | DTU/LIVE | Priority |
|---|---|---|---|---|---|
| OB-001 | Set `CLAROTY_INSTANCE_URL` env var to the xDome instance base URL. Store `bearer_token` for the Claroty org via `prism credential store` CLI. Boot prism. | `prism start` boots without error. Boot log shows Claroty adapter registered for the org. No credential value appears in any log line or MCP response. | Read boot log output; grep for `"claroty"` sensor registration line; grep logs for the actual credential value (must be absent). | LIVE-REQUIRED | P0 |
| OB-002 | Run `prism_describe` with the configured org/client_id after all expansion stories (G1–G6) merge. | Response includes all 14 tables: `claroty_alerts`, `claroty_audit_logs`, `claroty_devices`, `claroty_device_alert_relations`, `claroty_vulnerabilities`, `claroty_ot_activity_events`, `claroty_device_vulnerability_relations`, `claroty_servers`, `claroty_server_interfaces`, `claroty_organization_zones`, `claroty_organization_zone_policies`, `claroty_organization_firewall_groups`, `claroty_organization_firewall_policies`, `claroty_organization_acl_policies`. No extra tables beyond these 14. | `prism_describe` MCP call; assert JSON response `.tables[*].name` contains exactly those 14 sensor-prefixed names. | DTU+LIVE | P0 |
| OB-003 | Credential Tier-1 resolution: store token in env var `CLAROTY_BEARER_TOKEN` (or equivalent env-ref form). Verify prism resolves it at boot. | Prism resolves the credential from env without keyring access. Sensor health shows `auth_valid: true` against live xDome. | Check boot log for `env` resolution path; run `check_sensor_health`; confirm no keyring syscall on macOS (no Keychain prompt). | LIVE-REQUIRED | P1 |
| OB-004 | Credential Tier-3 path: configure org with `org_id` + `keyring` reference. Explicitly remove the keyring entry so it is unavailable. Attempt to boot. | Prism exits with `E-CRED-008` error (not silent fallthrough to empty credentials, not panic). Exit code is non-zero. | Boot output; assert `E-CRED-008` in stderr; assert no sensor registration for the org after boot fails. | DTU | P1 |
| OB-005 | Configure prism with an intentionally invalid or expired Claroty bearer token. Boot and run `check_sensor_health`. | Boot succeeds (credential resolution succeeds; token is opaque at boot). `check_sensor_health` returns `reachable: true`, `auth_valid: false`. No boot crash. Process stays running. | `check_sensor_health` MCP call; assert wire JSON `reachable == true`, `auth_valid == false`. | LIVE-REQUIRED | P0 |
| OB-006 | Trigger boot failure by corrupting the audit-init path (e.g., malformed audit log config). Separately, trigger credential-init failure (missing required credential). | Audit-init failure → `exit(4)`. Credential-init failure → `exit(5)`. Both exit codes match BC-2.08.001 / ADR-022 §A table. | Run prism binary with corrupted config; capture `$?`; assert correct exit codes. | DTU | P1 |
| OB-007 | With prism running and Claroty adapter registered, remove the Claroty sensor spec from the active config (empty snapshot) and trigger hot-reload via `reload_config` MCP call. Then restore the spec and reload again. | On removal: Claroty adapter deregistered; `prism_describe` no longer returns `claroty_*` tables. On restore: adapter re-registered; `prism_describe` returns all delivered tables again. Error surfaced if any — not silent. | `reload_config` MCP call; `prism_describe` before/after each reload; assert table count changes correctly. | DTU+LIVE | P1 |
| OB-008 | After each G1–G6 story merges, run `prism_describe` and confirm the new table(s) appear. Specifically: after S-CLAROTY-VULNS-001 merges, `claroty_vulnerabilities` present. After S-CLAROTY-SERVERS-001 merges, `claroty_servers` and `claroty_server_interfaces` both present. After S-CLAROTY-ORGPOLICY-001 merges, all 4 org-policy tables present. | For each newly-delivered table, `prism_describe` returns a `TableDescriptor` with the correct sensor-prefixed name, `ocsf_class`, and columns matching the story's Tier-1/Tier-2 spec. | `prism_describe` MCP call after each story merge; assert new table descriptor present with correct schema. | DTU+LIVE (DTU for G1; LIVE-REQUIRED for G2–G6) | P0 |

**Gate 1 item count: 8**

---

## Gate 2: FIELD MAPPING

Items verify every column maps to the correct POST-ROUTING-001 Arrow field name and class_uid routes correctly. All assertions must be at the **serialized wire shape** level (Arrow JSON column names), not only pre-serialization Rust structs.

| ID | What to do | Exact expected result | How to verify | DTU/LIVE | Priority |
|---|---|---|---|---|---|
| FM-001 | Run `prism_describe claroty_alerts` (post-ROUTING-001 binary). | Returns exactly 6 Tier-1 `ColumnDescriptor` entries (`finding_info_uid`, `status`, `time`, `finding_info_modified_time`, `message`, `finding_info_title`) plus 1 `raw_extensions` descriptor with `col_type: Json`, `nullable: true`, and description enumerating source keys `alert_type_name`, `category`, `devices_count`, `alert_class`, `ot_devices_count`. **PLUS 2 synthesized descriptors: `class_uid` (Integer, non-nullable) and `_sensor` (String, non-nullable), appended after the raw_extensions descriptor.** No individual descriptor for any Tier-2 col.name. | Assert serialized `prism_describe` JSON against the exact column list; assert absence of `alert_type_name`, `category`, `devices_count` as individual names; assert one `raw_extensions` entry; assert `class_uid` and `_sensor` synthesized descriptors. | DTU+LIVE | P0 |
| FM-002 | Run `prism_describe claroty_audit_logs`. | Returns **7** Tier-1 entries (`activity_name`, `actor_user_name`, `time`, `message`, `actor_user_uid`, `comment`, **`metadata_uid`** [KF-05 RESOLVED]) plus 1 `raw_extensions` entry with description enumerating `category` only. **PLUS 2 synthesized descriptors: `class_uid` (Integer, non-nullable) and `_sensor` (String, non-nullable).** No individual `category` descriptor. | Assert `metadata_uid` descriptor present with String type; assert `raw_extensions` description contains `"category"` but NOT `"id"`; assert `class_uid` and `_sensor` synthesized descriptors. | DTU+LIVE | P0 |
| FM-003 | Run `prism_describe claroty_devices`. | Returns 8 Tier-1 entries (`device_uid`, `device_instance_uid`, `device_type`, `device_type_label`, `risk_score`, `status_code`, `device_name`, `device_os_name`) plus 1 `raw_extensions` entry enumerating the 12 Tier-2 source keys. **PLUS 2 synthesized descriptors: `class_uid` (Integer, non-nullable) and `_sensor` (String, non-nullable).** | Assert `class_uid` and `_sensor` synthesized descriptors. | DTU+LIVE | P0 |
| FM-004 | Run `prism_describe claroty_device_alert_relations`. | Returns 6 Tier-1 entries (`device_uid`, `finding_info_uid`, `time`, `risk_score`, `comment`, `status`) plus 1 `raw_extensions` entry enumerating `network_signature_severity`, `network_signature_confidence`, `malicious_ip_severity`, `external_ip`. **PLUS 2 synthesized descriptors: `class_uid` (Integer, non-nullable) and `_sensor` (String, non-nullable).** | Assert synthesized descriptors present. | DTU+LIVE | P0 |
| FM-005 | Execute `SELECT class_uid FROM claroty_alerts LIMIT 1`. | Wire JSON: `class_uid` column = `2004` (Int32). | Assert serialized Arrow JSON column value. | DTU+LIVE | P0 |
| FM-006 | Execute `SELECT class_uid FROM claroty_audit_logs LIMIT 1`. | Wire JSON: `class_uid` = `3004` (entity_management). NOT `3001`. | Assert `!= 3001` AND `== 3004`. | DTU+LIVE | P0 |
| FM-007 | Execute `SELECT class_uid FROM claroty_devices LIMIT 1`. | Wire JSON: `class_uid` = `5001`. NOT `0`. | Assert `== 5001` AND `!= 0`. | DTU+LIVE | P0 |
| FM-008 | Execute `SELECT class_uid FROM claroty_device_alert_relations LIMIT 1`. | Wire JSON: `class_uid` = `2004`. | Assert column value. | DTU+LIVE | P0 |
| FM-009 | Query `claroty_alerts` from a live tenant where `id` values are known to be returned as JSON integers. Inspect the `finding_info_uid` column in the result. | `finding_info_uid` column is String type; the integer ID is serialized as a string — not as an Integer column, not as null. (Polymorphic ID normalization per EC-016-013-004.) | Check Arrow schema field type is `Utf8`; assert value is the string-encoded integer. | LIVE-REQUIRED | P0 |
| FM-010 | Query `claroty_alerts` with a WHERE clause on `time` against live tenant. | Datetime values parse correctly via implicit `["iso8601"]` default (ADR-028 §D8-B). No `E-SPEC-018` timestamp-parse-failed errors in logs. | Check prism logs for timestamp parse errors (must be absent); assert rows returned with non-null `time`. | LIVE-REQUIRED | P1 |
| FM-011 | Materialize a `claroty_alerts` RecordBatch. Inspect the serialized Arrow JSON. | (a) No first-class Arrow columns named `alert_type_name`, `category`, `devices_count`, `alert_class`, or `ot_devices_count`. (b) A `raw_extensions` column exists. (c) The JSON value of `raw_extensions` for each row contains keys `"alert_type_name"`, `"category"`, `"devices_count"`, `"alert_class"`, `"ot_devices_count"` with the corresponding vendor values preserved. | Wire-level assertion on serialized JSON. | DTU+LIVE | P0 |
| FM-012 | Materialize a `claroty_audit_logs` RecordBatch. Inspect the serialized Arrow JSON. | (a) First-class `metadata_uid` String column IS present (KF-05 RESOLVED; Tier-1). (b) No first-class `category` column. (c) `raw_extensions` JSON blob contains key `"category"` but NOT `"id"` or `"metadata_uid"`. (d) The audit record's ID is accessible directly via `SELECT metadata_uid FROM claroty_audit_logs`. | Wire-level assertion; assert `metadata_uid` column present with String type; assert no `"metadata_uid"` key in `raw_extensions`. | DTU+LIVE | P0 |
| FM-013 | Materialize a `claroty_devices` RecordBatch for a device known to have multiple IP addresses. Inspect `raw_extensions`. | `raw_extensions["ip_list"]` = `"[\"10.0.1.1\",\"10.0.1.2\"]"` — compact JSON-list **string**, NOT a nested JSON array. Same for `mac_list`, `network_list`, `vlan_list` (EC-016-013-028). | Wire-level assertion: assert value IS a String, NOT a JSON array node; assert exact compact-string format. | DTU | P0 |
| FM-014 | Materialize a `claroty_device_alert_relations` RecordBatch for a row with `network_signature_severity` and `external_ip` populated. | (a) No first-class columns for Tier-2 fields. (b) These values present in `raw_extensions` JSON. | Wire-level assertion. | DTU+LIVE | P0 |
| FM-015 | Construct a test SensorSpec with `ocsf_column_naming = true` and two columns whose `ocsf_field` values flatten to the same Arrow name. | `pipeline_result_to_record_batch` returns `Err(ArrowError::SchemaError(...))` — fail-closed. | Unit test; assert `Err` variant. | DTU | P1 |
| FM-016 | Construct a SensorSpec with `ocsf_field = "class_uid"` (or `"raw_extensions"`, `"_sensor"`, `"category_uid"`). | `pipeline_result_to_record_batch` returns `Err(ArrowError::SchemaError(...))` — fail-closed (EC-016-013-029, ADR-058 §J2). | Unit test; assert `Err` on each reserved name. | DTU | P1 |
| FM-017 | Query `claroty_device_alert_relations` and inspect the `alert_id`-derived column. | Arrow column is named `finding_info_uid` (NOT `finding_uid`). Value for a known alert is the alert ID string. | `prism_describe` + actual query; assert column name; assert `finding_uid` column is ABSENT. | DTU+LIVE | P0 |
| FM-018 | Verify `ocsf_field_to_arrow_name` crate location via Cargo.toml dependency graph. | Function resides in `prism-spec-engine::column_mapping`. `prism-mcp` imports it from `prism-spec-engine` (not from `prism-bin`). No `prism-mcp → prism-bin` dependency edge exists. | `grep -r "ocsf_field_to_arrow_name" crates/` confirms definition in `prism-spec-engine`; `cargo tree -p prism-mcp` must NOT show a `prism-bin` dependency. | DTU | P0 |
| FM-019 | After ROUTING-001 merge, run the query `SELECT id FROM claroty_alerts LIMIT 1` (using the OLD pre-ROUTING-001 col.name). | Returns `E-QUERY-038` (ColumnNotFound) with a message naming `id` as invalid and listing `finding_info_uid` as the correct column. | MCP `prism_query` call; assert error code `E-QUERY-038` in response JSON; assert suggestion field names correct column. | DTU+LIVE | P0 |
| FM-020 | Call `prism_describe` for each of the 4 original Claroty tables and run each table's `example_query` verbatim. | Each `example_query` uses POST-ROUTING-001 Arrow column names, executes without parse error, and returns at least one row. | Run each `example_query` via `prism_query` MCP; assert no `E-QUERY-038`; assert non-empty result. | DTU+LIVE | P1 |
| FM-021 | Run `prism_describe claroty_vulnerabilities` (post S-CLAROTY-VULNS-001 merge). | Returns exactly 2 Tier-1 `ColumnDescriptor` entries: `finding_info_title` (String, non-nullable — the PK) and `message` (String). Plus 1 `raw_extensions` entry with `col_type: Json`, `nullable: true`, description enumerating 17 Tier-2 source keys including `vulnerability_type`, `cve_ids`, `cvss_v3_score`, `is_known_exploited`, `epss_score`, `source_name`, `source_url`, and optionally `id`. **PLUS 2 synthesized descriptors: `class_uid` (Integer, non-nullable) and `_sensor` (String, non-nullable).** No individual descriptor for any Tier-2 col.name. | Assert exactly `finding_info_title` and `message` as individual Tier-1 descriptors; assert `raw_extensions` description enumerates Tier-2 keys; assert `class_uid` and `_sensor` synthesized descriptors. | DTU+LIVE | P0 |
| FM-022 | Execute `SELECT class_uid FROM claroty_vulnerabilities LIMIT 1`. | Wire JSON: `class_uid` = `2002` (vulnerability_finding). | Assert `== 2002`. | DTU+LIVE | P0 |
| FM-023 | Materialize a `claroty_vulnerabilities` RecordBatch. Inspect serialized Arrow JSON for a CVE with multiple CVE IDs. | (a) No first-class `name` or `description` columns (they became `finding_info_title` and `message`). (b) `raw_extensions["cve_ids"]` is a compact JSON-list string (e.g., `"[\"CVE-2021-31998\"]"`), NOT a nested JSON array node. (c) `raw_extensions["is_known_exploited"]` contains a boolean-as-string or boolean JSON value. | Wire-level assertion; assert `finding_info_title` Tier-1 column present; assert no `name` column; assert `cve_ids` in `raw_extensions` is String (compact JSON-list). | DTU+LIVE | P0 |
| FM-024 | Run `SELECT finding_info_title FROM claroty_vulnerabilities LIMIT 1` against a live tenant with known CVE records. | `finding_info_title` returns the CVE identifier string (e.g., `"CVE-2021-31998"`) or advisory title. Query executes without `E-QUERY-038`. | Assert column name; assert value is a string CVE or advisory identifier. | LIVE-REQUIRED | P1 |
| FM-025 | Run `prism_describe claroty_ot_activity_events` (post S-CLAROTY-OT-EVENTS-001 merge). | Returns exactly 4 Tier-1 `ColumnDescriptor` entries: `finding_info_uid` (String, `event_id` source), `time` (Datetime, `detection_time` source), `activity_name` (String, `event_type` source), `message` (String, `description` source). Plus 1 `raw_extensions` entry enumerating 17 Tier-2 source keys including `source_ip`, `dest_ip`, `protocol`, `related_alert_ids`. **PLUS 2 synthesized descriptors: `class_uid` (Integer, non-nullable) and `_sensor` (String, non-nullable).** | Assert exact 4 Tier-1 descriptors; assert `raw_extensions` descriptor enumerates all 17 Tier-2 keys; assert `class_uid` and `_sensor` synthesized. | LIVE-REQUIRED | P0 |
| FM-026 | Execute `SELECT class_uid FROM claroty_ot_activity_events LIMIT 1`. | Wire JSON: `class_uid` = `2004` (detection_finding). Same class as `claroty_alerts`. | Assert `== 2004`. | LIVE-REQUIRED | P0 |
| FM-027 | Query `claroty_ot_activity_events` against live tenant. Inspect `finding_info_uid` column type and value. | The xDome `event_id` field is an Integer in the API response. After OCSF routing, `finding_info_uid` Arrow column type is `Utf8` (String). Integer value is serialized as the string `"12345"` — same polymorphic-ID normalization as EC-016-013-004. No `E-SPEC-018` errors. `related_alert_ids` in `raw_extensions` is a compact JSON-list string. | Check Arrow schema for `finding_info_uid` is `Utf8`; assert integer event_id serializes as string; assert `related_alert_ids` in `raw_extensions` is a String (compact JSON-list). | LIVE-REQUIRED | P0 |
| FM-028 | Run `prism_describe claroty_device_vulnerability_relations` (post S-CLAROTY-DEVVULNREL-001 merge). | Returns exactly 2 Tier-1 `ColumnDescriptor` entries: `finding_info_title` (String, `vulnerability_name` source, REQUIRED) and `time` (Datetime, `device_vulnerability_detection_date` source). Plus 1 `raw_extensions` entry enumerating 11 Tier-2 source keys including `device_uid`, `vulnerability_id`, `vulnerability_cvss_v3_score`, `vulnerability_is_known_exploited`. **PLUS 2 synthesized descriptors: `class_uid` (Integer, non-nullable) and `_sensor` (String, non-nullable).** | Assert 2 Tier-1 descriptors; assert `raw_extensions` enumerates `device_uid` as a Tier-2 key (it is NOT a Tier-1 Arrow column despite being the composite PK join key); assert `class_uid` and `_sensor` synthesized. | LIVE-REQUIRED | P0 |
| FM-029 | Execute `SELECT class_uid FROM claroty_device_vulnerability_relations LIMIT 1`. | Wire JSON: `class_uid` = `2002` (vulnerability_finding). Same class as `claroty_vulnerabilities`. | Assert `== 2002`. | LIVE-REQUIRED | P0 |
| FM-030 | Execute `SELECT finding_info_title, time FROM claroty_device_vulnerability_relations LIMIT 5` and verify envelope key. | (a) `finding_info_title` (from `vulnerability_name`) is the vulnerability CVE ID or name string. (b) `time` (from `device_vulnerability_detection_date`) is a parseable ISO-8601 datetime. (c) **Critical envelope check:** prism's fetch log must show POST to `/api/v1/device_vulnerability_relations/` with `response_path = "$.devices_vulnerabilities"` — the envelope key diverges from the table name. No `E-SENSOR-030` due to wrong envelope key. | Query against live sensor; assert `finding_info_title` and `time` columns present; verify prism fetch log shows `devices_vulnerabilities` key is used (not `device_vulnerability_relations`). | LIVE-REQUIRED | P0 |
| FM-031 | Run `prism_describe claroty_servers` (post S-CLAROTY-SERVERS-001 merge). | Returns exactly 2 Tier-1 `ColumnDescriptor` entries: `device_name` (String, `server_name` source, REQUIRED) and `status_code` (String, `server_status` source). Plus 1 `raw_extensions` entry enumerating 15 Tier-2 source keys including `server_location`, `model`, `os_version`, `uptime_days`, `num_of_open_incidents`. **PLUS 2 synthesized descriptors: `class_uid` (Integer, non-nullable) and `_sensor` (String, non-nullable).** No `server_name` individual descriptor. | Assert 2 Tier-1 descriptors; assert `raw_extensions` enumerates 15 Tier-2 keys; assert `class_uid` and `_sensor` synthesized. | LIVE-REQUIRED | P0 |
| FM-032 | Execute `SELECT class_uid FROM claroty_servers LIMIT 1`. | Wire JSON: `class_uid` = `5001` (inventory_info). Same class as `claroty_devices`. | Assert `== 5001`. | LIVE-REQUIRED | P0 |
| FM-033 | Materialize a `claroty_servers` RecordBatch. Inspect `raw_extensions` for `uptime_days`. | `raw_extensions["uptime_days"]` value is a Float (JSON number with decimal point, e.g. `667.233661`) — NOT an Integer. This is confirmed by the xDome OpenAPI example value `667.233661`. No `E-SPEC-018` parse errors. | Wire-level assertion; assert `uptime_days` value in `raw_extensions` is a JSON number with a decimal component. | LIVE-REQUIRED | P1 |
| FM-034 | Run `prism_describe claroty_server_interfaces` (post S-CLAROTY-SERVERS-001 merge). | Returns exactly 2 Tier-1 `ColumnDescriptor` entries: `device_name` (String, `server_name` source, REQUIRED) and `status_code` (String, `interface_status` source). Plus 1 `raw_extensions` entry enumerating 8 Tier-2 source keys including `interface_name`, `interface_type`, `ip_address`, `mac_address`, `subnet`, `vlan`, `is_monitored`, `interface_description`. **PLUS 2 synthesized descriptors: `class_uid` (Integer, non-nullable) and `_sensor` (String, non-nullable).** | Assert 2 Tier-1 descriptors; assert `raw_extensions` enumerates 8 Tier-2 keys including `interface_name`; assert `class_uid` and `_sensor` synthesized. | LIVE-REQUIRED | P0 |
| FM-035 | Execute `SELECT class_uid FROM claroty_server_interfaces LIMIT 1`. | Wire JSON: `class_uid` = `5001` (inventory_info). Same class as `claroty_servers` and `claroty_devices`. | Assert `== 5001`. | LIVE-REQUIRED | P0 |
| FM-036 | Verify the endpoint path for `claroty_server_interfaces`. Inspect prism fetch log for the HTTP POST URL. | HTTP POST goes to `/api/v1/server_interfaces/` — a SEPARATE endpoint from `/api/v1/servers/`. NOT a sub-path. This corrects the initial plan assumption (endpoint-spike-findings §Spike 4 correction: separate operationId `get_servers_api_v1_server_interfaces__post`). No 404 due to path confusion with `/api/v1/servers/`. | Capture prism fetch log; assert POST URL is `/api/v1/server_interfaces/` (not `/api/v1/servers/server_interfaces/` or any sub-path form). | LIVE-REQUIRED | P0 |
| FM-037 | Run `prism_describe claroty_organization_zones` (post S-CLAROTY-ORGPOLICY-001 merge). | Returns exactly 4 Tier-1 `ColumnDescriptor` entries: `name` (String, `zone_name` source, REQUIRED), `comment` (String, `zone_description` source), `status_code` (Boolean, `enabled` source), `actor_user_name` (String, `updated_by` source). Plus 1 `raw_extensions` entry enumerating 7 Tier-2 source keys including `device_conditions`, `attributed_devices`, `created_time`. **PLUS 2 synthesized descriptors: `class_uid` (Integer, non-nullable) and `_sensor` (String, non-nullable).** | Assert 4 Tier-1 descriptors; assert `raw_extensions` enumerates 7 Tier-2 keys; assert `class_uid == 3004` via `SELECT class_uid FROM claroty_organization_zones LIMIT 1`. | LIVE-REQUIRED | P0 |
| FM-038 | Materialize a `claroty_organization_zones` RecordBatch. Inspect `raw_extensions` for `device_conditions`. | `raw_extensions["device_conditions"]` is a compact JSON-list string (array of device filter condition objects serialized as string) — NOT a nested JSON object or array node. Same compact JSON-list string pattern as EC-016-013-028. | Wire-level assertion; assert `device_conditions` value in `raw_extensions` is a String (not a JSON array node). | LIVE-REQUIRED | P1 |
| FM-039 | Run `prism_describe claroty_organization_zone_policies` (post S-CLAROTY-ORGPOLICY-001 merge). | Returns exactly 4 Tier-1 `ColumnDescriptor` entries: `name` (String, `policy_name` source, REQUIRED), `activity_name` (String, `policy_action` source), `comment` (String, `policy_notes` source), `actor_user_name` (String, `updated_by` source). Plus 1 `raw_extensions` entry enumerating 9 Tier-2 source keys including `communication_conditions`, `related_alerts_ids`, `applied_zone_pairs` (all Json), `should_generate_alerts`, `matching_devices`. **PLUS 2 synthesized descriptors: `class_uid` (Integer, non-nullable) and `_sensor` (String, non-nullable).** | Assert 4 Tier-1 descriptors; assert `class_uid == 3004`; assert `raw_extensions` enumerates 9 Tier-2 keys. | LIVE-REQUIRED | P0 |
| FM-040 | Materialize a `claroty_organization_zone_policies` RecordBatch for a policy with zone pairs. Inspect `raw_extensions`. | Three Json columns (`communication_conditions`, `related_alerts_ids`, `applied_zone_pairs`) are stored in `raw_extensions` as compact JSON-list strings — NOT nested JSON array nodes. Pattern consistent with EC-016-013-028. `applied_zone_pairs` is a list of `{src_zone, dst_zone}` objects serialized as a compact string. | Wire-level assertion; assert all three Json fields are String type in `raw_extensions`; assert `applied_zone_pairs` contains parseable JSON-list string. | LIVE-REQUIRED | P1 |
| FM-041 | Run `prism_describe claroty_organization_firewall_groups` (post S-CLAROTY-ORGPOLICY-001 merge). | Returns exactly 4 Tier-1 `ColumnDescriptor` entries: `name` (String, `firewall_group_name` source, REQUIRED), `comment` (String, `firewall_group_description` source), `status_code` (Boolean, `enabled` source), `actor_user_name` (String, `updated_by` source). Plus 1 `raw_extensions` entry enumerating 7 Tier-2 source keys including `device_conditions`. **PLUS 2 synthesized descriptors.** | Assert 4 Tier-1 descriptors; assert `class_uid == 3004`; assert `raw_extensions` enumerates 7 Tier-2 keys. | LIVE-REQUIRED | P0 |
| FM-042 | Execute `SELECT class_uid FROM claroty_organization_firewall_groups LIMIT 1`. | Wire JSON: `class_uid` = `3004` (entity_management). Same class as all other org-policy tables and `claroty_audit_logs`. | Assert `== 3004`. | LIVE-REQUIRED | P0 |
| FM-043 | Run `prism_describe claroty_organization_firewall_policies` (post S-CLAROTY-ORGPOLICY-001 merge). | Returns exactly 4 Tier-1 `ColumnDescriptor` entries: `name` (String, `policy_name` source, REQUIRED), `activity_name` (String, `policy_action` source), `comment` (String, `policy_notes` source), `actor_user_name` (String, `updated_by` source). Plus 1 `raw_extensions` entry enumerating 9 Tier-2 source keys including `communication_conditions`, `related_alerts_ids`, `applied_group_pairs` (all Json), `should_generate_alerts`, `matching_devices`. **PLUS 2 synthesized descriptors.** | Assert 4 Tier-1 descriptors; assert `class_uid == 3004`; assert `raw_extensions` enumerates 9 Tier-2 keys. | LIVE-REQUIRED | P0 |
| FM-044 | Materialize a `claroty_organization_firewall_policies` RecordBatch. Inspect `raw_extensions` for `applied_group_pairs`. | `raw_extensions["applied_group_pairs"]` is a compact JSON-list string of `{src_group, dst_group}` pair objects — NOT a nested JSON array node. Pattern consistent with EC-016-013-028 and zone_policies FM-040. | Wire-level assertion; assert `applied_group_pairs` is a String in `raw_extensions`. | LIVE-REQUIRED | P1 |
| FM-045 | Run `prism_describe claroty_organization_acl_policies` (post S-CLAROTY-ACLPOLICY-001 merge). | Returns exactly 4 Tier-1 `ColumnDescriptor` entries: `metadata_uid` (String, `policy_id` source, REQUIRED — UUID format), `name` (String, `policy_name` source), `comment` (String, `policy_notes` source), `actor_user_name` (String, `policy_updated_by` source). Plus 1 `raw_extensions` entry enumerating 7 Tier-2 source keys including `policy_acl_type`, `policy_acl`, `applied_models` (Json), `matching_devices`, `policy_creation_date`, `policy_last_updated`, `policy_source`. **PLUS 2 synthesized descriptors: `class_uid` (Integer, non-nullable) and `_sensor` (String, non-nullable).** | Assert 4 Tier-1 descriptors; assert `metadata_uid` (not `policy_id`) as the PK column; assert `class_uid == 3004`; assert `raw_extensions` enumerates 7 Tier-2 keys. | LIVE-REQUIRED | P0 |
| FM-046 | Verify `claroty_organization_acl_policies` non-paginated fetch contract. Issue a query against the table and inspect the POST body sent to the ACL policies endpoint. | POST body contains `"policy_acl_syntax": "Cisco dACL"` and a `"fields"` array. **NO `"offset"` or `"limit"` keys.** Response has envelope key `organization_acl_policies` with no `count` field. `type = "none"` in TOML prevents offset/limit injection per BC-2.16.022 §Postconditions §3. This is the anomaly check for the non-standard pagination. | Capture prism fetch log; assert POST body lacks `offset`/`limit` keys; assert `policy_acl_syntax` key present; assert response parses `organization_acl_policies` array correctly. | LIVE-REQUIRED | P0 |

**Gate 2 item count: 46**

---

## Gate 3: QUERIES

Items cover the representative PrismQL query catalog against all 14 Claroty tables. All column references use POST-ROUTING-001 Arrow names.

| ID | What to do | Exact expected result | How to verify | DTU/LIVE | Priority |
|---|---|---|---|---|---|
| QU-001 | `FROM claroty_alerts \| fields finding_info_uid, status, time, message \| limit 5` | Returns rows with exactly those 4 columns populated. `finding_info_uid` is String. `time` is parseable as ISO-8601 datetime. | Inspect column names and types in Arrow result schema. | DTU+LIVE | P0 |
| QU-002 | `FROM claroty_alerts \| where status = 'Unresolved' \| limit 10` | Returns only rows where `status` equals `'Unresolved'` exactly. | Assert all returned rows have `status == 'Unresolved'`. | DTU+LIVE | P0 |
| QU-003 | `FROM claroty_alerts \| where time > '2024-01-01T00:00:00Z' \| where time < '2025-01-01T00:00:00Z' \| limit 20` | Returns only rows where `time` falls within the specified range. POST body to alerts endpoint does NOT contain `filter_by` (no push-down). | Assert all returned rows have `time` within range; verify no `filter_by` in alerts POST body. | DTU+LIVE | P0 |
| QU-004 | `FROM claroty_alerts \| where status IEQ 'unresolved' \| limit 10` | Case-insensitive match: same row set as QU-002. | Compare row count / IDs against QU-002 result. | DTU+LIVE | P1 |
| QU-005 | `FROM claroty_audit_logs \| where activity_name IS NOT NULL \| order by time desc \| limit 20` | Returns 20 most recent audit log entries, ordered by `time` descending. | Assert row order: each row's `time` >= next row's `time`. | DTU+LIVE | P0 |
| QU-006 | `FROM claroty_alerts \| group by status \| count(*) as alert_count \| order by alert_count desc` | Returns one row per distinct `status` value with count. No casing-duplicate buckets. | Assert GROUP BY result; verify no casing-duplicate buckets. | DTU+LIVE | P1 |
| QU-007 | `FROM claroty_devices \| where device_type = 'OT Device' \| fields device_uid, device_type_label, risk_score, device_name \| limit 20` | Returns OT devices with the 4 requested fields. `device_type_label` (from `device_type`, post-KF-06) is distinct from `device_type` (from `device_category`). | Assert both columns present; assert no `device_type_name` column. | DTU+LIVE | P0 |
| QU-008 | `FROM claroty_devices \| fields device_uid, raw_extensions \| limit 5`. Then parse `raw_extensions` JSON for key `"is_online"`. | `raw_extensions` column present as Json/String type. JSON blob parseable; key `"is_online"` present. **WHERE predicates on Tier-2 keys require `json_extract_string` UDF (S-JSON-EXTRACT-UDF-001); SELECT access to `raw_extensions` is available in v1.** | Assert `raw_extensions` column in result; parse JSON; assert `"is_online"` key present. | DTU+LIVE | P1 |
| QU-009 | Configure a test scenario or use a live tenant with more than 1000 audit_log entries. Execute `FROM claroty_audit_logs \| limit 2000`. | All pages fetched via offset_limit POST-body pagination. Pagination sequence: page 1 `offset=0 limit=1000`, page 2 `offset=1000 limit=1000`. | Assert total rows = min(2000, total_count); check prism fetch logs for multiple POST requests with incrementing offsets. | DTU+LIVE | P0 |
| QU-010 | Execute `FROM alerts \| limit 5` (no sensor prefix). | Returns `E-SENSOR-030` or `E-QUERY-037` (table not found). NOT silent empty result set. | Assert error code in MCP response. | DTU | P0 |
| QU-011 | Execute `FROM claroty_alerts \| fields finding_info_uid, nonexistent_column \| limit 1`. | Returns `E-QUERY-038` (ColumnNotFound). Error message names `nonexistent_column` as invalid. Error message lists valid columns for `claroty_alerts`. | Assert `E-QUERY-038` in response; assert suggestion list contains correct column names. | DTU+LIVE | P0 |
| QU-012 | Cross-table investigation: (1) `FROM claroty_alerts \| where status = 'Unresolved' \| fields finding_info_uid \| limit 5`. (2) `FROM claroty_device_alert_relations \| where finding_info_uid = '<id-from-step-1>' \| fields device_uid, status \| limit 10`. (3) `FROM claroty_devices \| where device_uid = '<device_uid-from-step-2>' \| limit 1`. | Each query returns data. `finding_info_uid` join key works across tables (KF-07 correction). `device_uid` from step 2 matches `claroty_devices`. No `E-QUERY-038` column-not-found errors. | Manually execute the 3-step pivot; assert data flows correctly. | DTU+LIVE | P0 |
| QU-013 | `FROM claroty_vulnerabilities \| where finding_info_title LIKE 'CVE-%' \| order by finding_info_title asc \| limit 20` | Rows where `finding_info_title` matches `CVE-*` pattern. Column is `finding_info_title` (not `name`). CVE identifiers appear in standard CVE format (e.g., `CVE-2021-31998`). | Assert column name `finding_info_title`; assert values match CVE format; no `E-QUERY-038`. | DTU+LIVE | P0 |
| QU-014 | `FROM claroty_vulnerabilities \| fields finding_info_title, message, raw_extensions \| limit 10`. Parse `raw_extensions` for CISA KEV indicator. | `raw_extensions` present for each row; key `"is_known_exploited"` present in the JSON blob (may be null or boolean string). Key `"cvss_v3_score"` also accessible. | Assert `raw_extensions` column present; parse JSON; assert `"is_known_exploited"` and `"cvss_v3_score"` keys exist. | DTU+LIVE | P1 |
| QU-015 | `FROM claroty_ot_activity_events \| where activity_name = 'Configuration Upload' \| limit 10` | Rows where OT configuration upload events occurred. Column is `activity_name` (from `event_type` TOML col). | Assert column name `activity_name`; filter works; no `E-QUERY-038`. | LIVE-REQUIRED | P0 |
| QU-016 | `FROM claroty_ot_activity_events \| fields finding_info_uid, time, activity_name, raw_extensions \| limit 5`. Parse `raw_extensions` for network context. | `finding_info_uid` (from integer `event_id`) is String. `raw_extensions` contains `source_ip`, `dest_ip`, `protocol`, `related_alert_ids` (compact JSON-list string). | Assert `finding_info_uid` is String type; assert `raw_extensions` contains `source_ip` and `related_alert_ids` keys; assert `related_alert_ids` is a String (compact JSON-list). | LIVE-REQUIRED | P1 |
| QU-017 | `FROM claroty_device_vulnerability_relations \| where finding_info_title = 'CVE-2021-31998' \| fields finding_info_title, time, raw_extensions \| limit 10` | Rows for devices affected by the specified CVE. `finding_info_title` is `vulnerability_name`. `raw_extensions` contains `device_uid` and `vulnerability_cvss_v3_score`. | Assert column name `finding_info_title`; assert `raw_extensions` contains `device_uid`; no `E-QUERY-038`. | LIVE-REQUIRED | P0 |
| QU-018 | Cross-table vulnerability investigation: (1) `FROM claroty_vulnerabilities \| where finding_info_title = 'CVE-2021-31998' \| limit 1`. (2) `FROM claroty_device_vulnerability_relations \| where finding_info_title = 'CVE-2021-31998' \| fields raw_extensions \| limit 20`. (3) Parse `raw_extensions["device_uid"]` from step 2. (4) `FROM claroty_devices \| where device_uid = '<device_uid>' \| limit 1`. | Step 1: vulnerability details. Step 2: all device relations for the CVE. Step 3: device UIDs from Tier-2. Step 4: full device profile. Cross-table join via `finding_info_title` (Tier-1) and `device_uid` (Tier-2 in `raw_extensions`) works. | All four steps execute without `E-QUERY-038`; data flows across tables; `device_uid` parseable from `raw_extensions` JSON. | LIVE-REQUIRED | P0 |
| QU-019 | `FROM claroty_servers \| where status_code = 'Active' \| fields device_name, status_code, raw_extensions \| limit 10` | Active servers. `device_name` (from `server_name`) is the Tier-1 column. `status_code` (from `server_status`) is the other Tier-1 column. `raw_extensions` contains `server_location`, `model`, `uptime_days`. | Assert column names `device_name` and `status_code`; assert no `server_name` column; assert `raw_extensions` contains Tier-2 server details. | LIVE-REQUIRED | P0 |
| QU-020 | `FROM claroty_server_interfaces \| where device_name = 'SERVER-001' \| fields device_name, status_code, raw_extensions \| limit 10` | All interfaces for the specified server. `device_name` is the join key to `claroty_servers`. `raw_extensions` contains `interface_name`, `ip_address`, `is_monitored`. | Assert column names; assert `raw_extensions` contains `interface_name` and `ip_address`; no `E-QUERY-038`. | LIVE-REQUIRED | P0 |
| QU-021 | `FROM claroty_organization_zones \| where status_code = true \| fields name, comment, raw_extensions \| limit 20` | Active zones. `name` (from `zone_name`) is the PK Tier-1 column. `status_code` (from `enabled`) is Boolean. `raw_extensions` contains `attributed_devices` count and `device_conditions`. | Assert column names `name` and `status_code`; assert Boolean filter on `status_code` works; assert `raw_extensions` contains `attributed_devices`. | LIVE-REQUIRED | P0 |
| QU-022 | `FROM claroty_organization_zone_policies \| where activity_name = 'Deny' \| fields name, activity_name, comment \| limit 20` | Deny zone policies. `activity_name` (from `policy_action`) is the Tier-1 column filtering on 'Deny' vs 'Allow'. | Assert column name `activity_name`; filter works; no `E-QUERY-038`. | LIVE-REQUIRED | P1 |
| QU-023 | `FROM claroty_organization_firewall_groups \| where status_code = true \| fields name, comment \| limit 20` | Active firewall groups. Structurally identical to QU-021 but for firewall groups (BC-2.16.021 symmetric with BC-2.16.020). | Assert column names `name` and `status_code`; filter works. | LIVE-REQUIRED | P1 |
| QU-024 | `FROM claroty_organization_firewall_policies \| where activity_name = 'Allow' \| fields name, activity_name, comment \| limit 20` | Allow firewall policies. Structurally symmetric to QU-022. | Assert column names; filter works; data distinct from zone policies. | LIVE-REQUIRED | P1 |
| QU-025 | `FROM claroty_organization_acl_policies \| fields metadata_uid, name, comment, raw_extensions \| limit 20` | All ACL policies returned (non-paginated: `type = "none"`). `metadata_uid` (from `policy_id`, UUID format) is the Tier-1 PK. `raw_extensions` contains `policy_acl` (raw ACL text) and `applied_models` (compact JSON-list string). **No multi-page fetch** — single response for the entire dataset. If tenant has fewer than 20 ACL policies, returns all available. | Assert `metadata_uid` column present (not `policy_id`); assert `raw_extensions` contains `policy_acl` key; verify only ONE POST was issued (no pagination offset sequence in prism fetch log). | LIVE-REQUIRED | P0 |

**Gate 3 item count: 25**

---

## Gate 4: PUSH-DOWN

Items verify the `audit_logs` time-box push-down works against the live tenant, document the no-push-down tables, and close **ASM-CLAROTY-AUDITLOG-001**.

> **Push-down scope:** Only `claroty_audit_logs` has push-down (INDEX datetime column `timestamp`).
> All other tables — including all 10 expansion tables — use in-engine DataFusion post-filtering.
> The expansion tables (G1–G6) have no INDEX columns in their first-cut column sets.

> **POST-ROUTING-001 naming impact on push-down — RESOLVED (OQ-001, ADR-058 §I6, ROUTING-001 RG-PD-001):**
> After ROUTING-001, `timestamp` (TOML col.name) becomes `time` (Arrow field name).
> Analyst queries use `WHERE time > 'T'`, not `WHERE timestamp > 'T'`. The
> `extract_time_window_from_ast` mechanism registers BOTH `timestamp` (col.name) AND `time`
> (Arrow name) as INDEX-eligible. PD-007 is the live confirmation item.

| ID | What to do | Exact expected result | How to verify | DTU/LIVE | Priority |
|---|---|---|---|---|---|
| PD-001 | Execute `FROM claroty_audit_logs \| limit 100` with NO time predicate. Inspect the POST body sent to `/api/v1/audit_log/get`. | POST body contains `"filter_by": {"field": "timestamp", "operation": "greater_or_equal", "value": "<now-7days-iso8601>"}` (EC-01-030: bounded default). | Capture HTTP request body; assert `filter_by` key present; assert value is within 7 days of current time. | DTU+LIVE | P0 |
| PD-002 | Execute `FROM claroty_audit_logs \| where time > '2024-06-01T00:00:00Z' \| limit 100`. Inspect POST body. | POST body contains `"filter_by": {"field": "timestamp", "operation": "greater_or_equal", "value": "2024-06-01T00:00:00Z"}` — single `greater_or_equal` bound (EC-01-031). | Capture POST body; assert exact field/operation/value. | DTU+LIVE | P0 |
| PD-003 | Execute `FROM claroty_audit_logs \| where time < '2024-06-30T00:00:00Z' \| limit 100`. Inspect POST body. | POST body contains `"filter_by": {"field": "timestamp", "operation": "less_or_equal", "value": "2024-06-30T00:00:00Z"}`. NO synthetic lower bound added (EC-01-032). | Capture POST body; assert single `less_or_equal` predicate only. | DTU+LIVE | P0 |
| PD-004 | Execute `FROM claroty_audit_logs \| where time > '2024-06-01T00:00:00Z' \| where time < '2024-06-30T00:00:00Z' \| limit 100`. Inspect POST body. | POST body contains `"filter_by": {"operation": "and", "operands": [{gte}, {lte}]}` (EC-01-033). Key MUST be `"operands"` NOT `"conditions"`. | Capture POST body; assert `"operation": "and"`, `"operands"` key; assert both bounds present. | DTU+LIVE | P0 |
| PD-005 | **ASM-CLAROTY-AUDITLOG-001 LIVE CONFIRMATION.** Execute PD-001 through PD-004 against the **real xDome tenant** (not DTU). | Real xDome returns HTTP 200 with non-empty results for each filter variant. No 400 or 422 from xDome. | Execute all 4 time-filter variants against live xDome; assert HTTP 200; log confirmation: "ASM-CLAROTY-AUDITLOG-001: CONFIRMED/REFUTED" with actual HTTP status. | LIVE-REQUIRED | P0 |
| PD-006 | Execute `FROM claroty_alerts \| where time > '2024-01-01T00:00:00Z' \| limit 50`. Inspect POST body to `/api/v1/alerts/`. | POST body to alerts endpoint does NOT contain any `filter_by` key. All filtering happens in-engine (DataFusion post-filter). | Capture POST body; assert no `filter_by` key; assert all returned rows satisfy the time predicate. | DTU+LIVE | P0 |
| PD-007 | **[ROUTING-001 push-down live confirmation — RG-PD-001].** Execute `FROM claroty_audit_logs \| where time > '2024-01-01T00:00:00Z' \| limit 10` using `time` (POST-ROUTING-001 Arrow name). | Push-down fires: POST body contains `filter_by.operation = "greater_or_equal"`. This CONFIRMS that `extract_time_window_from_ast` registers Arrow name `time` as INDEX-eligible (OQ-001 RESOLVED, ADR-058 §I6). | Capture POST body; assert `filter_by` present. Confirm "CONFIRMED" or flag "REGRESSION" with actual POST body snippet. | DTU+LIVE | P0 |
| PD-008 | Execute `FROM claroty_audit_logs \| where time < '2020-01-01T00:00:00Z' \| where time > '2024-01-01T00:00:00Z'` (inverted window: start > end). | Prism emits a `push_down.inverted_time_range` WARN log event. Both bounds still passed to xDome in the POST body. DataFusion in-engine backstop ensures zero rows returned. No crash. | Check prism log for inverted-window WARN; assert result is 0 rows; assert POST body still contains both bounds. | DTU | P1 |

**Gate 4 item count: 8**

---

## Gate 5: SOC-Analyst Q&A Catalog

The heart of the release gate: 64 real analyst questions spanning all 14 tables and cross-table investigation flows, each with an exact PrismQL query, expected data, and pass criterion.

**See companion file:** `.factory/objectives/xdome-v1-validation/soc-analyst-qa-catalog.md`

**Item count: 64** (was 27 in v0.2; expanded by 37 entries for G1–G6 expansion tables)

---

## Gate 6: STABILITY AND RESILIENCE

Items cover concurrent queries, fan-out bounds, memory/timeout budgets, error/health behavior, transport hardening, and graceful partial-failure.

| ID | What to do | Exact expected result | How to verify | DTU/LIVE | Priority |
|---|---|---|---|---|---|
| SR-001 | Issue 3 concurrent PrismQL queries against `claroty_devices`, `claroty_alerts`, and `claroty_audit_logs` simultaneously from the same prism server instance. | All 3 queries execute concurrently. ArcSwap lock-free config reads. All 3 return results without deadlock or panic. | Issue all 3 via separate MCP client connections; confirm no lock waits in prism logs; assert all 3 complete successfully. | DTU | P0 |
| SR-002 | Query a large `claroty_alerts` dataset (>5000 rows if available). Monitor memory usage during the query. | Memory per-query stays within 200MB budget (BC-2.01.002). No `E-QUERY-004` timeout on a dataset of typical MSSP tenant size. | Monitor process RSS during query; assert peak < 200MB per-query overhead. | DTU+LIVE | P1 |
| SR-003 | Simulate a slow xDome endpoint (artificially delay responses via DTU). Verify the 30s HTTP timeout fires. | After 30s, prism surfaces `E-SENSOR-002` (timeout error). No indefinite hang. Process continues accepting other queries. | DTU with artificial delay; assert `E-SENSOR-002` in MCP error response within 31s of query start. | DTU | P0 |
| SR-004 | Configure the live xDome sensor with an expired bearer token (or use an invalid token). Run `check_sensor_health`. | `check_sensor_health` returns: `reachable: true`, `auth_valid: false`, `http_status: 403` (or 401), `overall_status: "unhealthy"` or `"auth_invalid"`. NOT `Down`. | Assert `reachable == true`, `auth_valid == false`; assert `overall_status` is not `"healthy"`. | LIVE-REQUIRED | P0 |
| SR-005 | Simulate a Claroty API returning HTTP 5xx (503) via DTU. Run `check_sensor_health`. | `check_sensor_health` returns: `reachable: true`, `auth_valid: true`, `error: "service_unavailable"`, `overall_status: "degraded"`. NOT `Down`. | Assert `overall_status == "degraded"`, `reachable == true`, `auth_valid == true`, `error == "service_unavailable"`. | DTU | P0 |
| SR-006 | Completely stop the DTU clone (no TCP listener). Run `check_sensor_health` for Claroty. | `check_sensor_health` returns `reachable: false`, `overall_status: "down"`. `Down` ONLY when no TCP/HTTP exchange was possible. | Assert `reachable == false`, `overall_status == "down"`. | DTU | P0 |
| SR-007 | Verify TLS transport: capture TLS handshake details when prism connects to the real xDome tenant. | TLS negotiation uses rustls (not native-tls). No macOS Keychain initialization delay. `reqwest` client built with `default-features = false, features = ["rustls-tls"]` (ADR-050 D1/D2). | Check prism binary for native-tls dependency (`cargo tree -p prism-bin \| grep native-tls` must return empty); confirm no 65s Keychain stall. | LIVE-REQUIRED | P0 |
| SR-008 | Execute a query against `claroty_alerts` while simultaneously pulling a 5xx error from the Claroty DTU for `claroty_devices`. | The `claroty_alerts` query completes successfully. The `claroty_devices` fan-out returns a structured `SensorError::HttpError{status}`. The partial-failure propagates to the MCP caller — NOT silently swallowed as an empty Vec (Standing Rule 3 §2). | Assert alerts result is non-empty; assert devices fan-out error is present in response envelope. | DTU | P0 |
| SR-009 | Execute `FROM claroty_alerts \| limit 10` against the live tenant. Verify the HTTP request uses the correct trailing-slash path. | HTTP POST to `/api/v1/alerts/` (WITH trailing slash). HTTP POST to `/api/v1/audit_log/get` (WITHOUT trailing slash). Both accepted by live xDome. | Check prism fetch log for exact URL; assert trailing-slash format for alerts/devices/device_alert_relations; assert no trailing slash for audit_log/get. | LIVE-REQUIRED | P0 |
| SR-010 | Fan-out bounds: configure prism with 15 sensor adapters across multiple orgs (exceeding MAX_FANOUT_CONCURRENCY=10 per fan-out). Execute a cross-sensor query. | At most 10 concurrent fan-out tasks execute simultaneously (bounded semaphore). Total HTTP connections bounded by HTTP_SEMAPHORE_PERMITS=200. No deadlock. | Check prism concurrency-architecture log events; assert concurrent fan-out count <= 10. | DTU | P1 |
| SR-011 | Execute a `claroty_devices` query where one step's pagination returns malformed JSON (DTU-injected error). | Prism returns a structured error to the MCP caller identifying the malformed response. NOT a panic. NOT a silent empty result masking the failure. | DTU with injected malformed response; assert structured error in MCP response; assert no server panic. | DTU | P1 |
| SR-012 | Verify `check_sensor_health` for Claroty uses the `probe_table = "devices"` endpoint. Inspect the LIMIT-0 probe POST request. | Health probe issues `POST /api/v1/devices/` with `{"fields": [...], "offset": 0, "limit": 0}`. NOT any other table path. | Check prism health-probe fetch log; assert POST to the `probe_table = "devices"` path with limit=0. | DTU+LIVE | P1 |

**Gate 6 item count: 12**

---

## Priority-1 Live Risk Checks

These items correspond directly to the top risks for the Claroty xDome v1 release gate.

| RISK-ID | Risk | Live Check | Pass Criterion | Escalation if Fails |
|---|---|---|---|---|
| RISK-1 | **ROUTING-001 not merged — query-breaking column rename.** | Before release gate runs, confirm all 27 Red Gate tests (RG-001..RG-027) pass on the post-ROUTING-001 binary. Run `just iter prism-spec-engine` and `just iter prism-bin` and `just iter prism-mcp` against the merged binary. | Exit 0 on all three crate test runs. All 27 RG tests GREEN. | Block release. Root-cause the failing RG test. Implementer fixes in scope. |
| RISK-2 | **ASM-CLAROTY-AUDITLOG-001 unconfirmed.** `filter_by.field = "timestamp"` and operations `"greater_or_equal"` / `"less_or_equal"` are research-validated but not live-confirmed. | Execute PD-005 (Gate 4) against the real xDome tenant with a known date range that has audit events. Record the actual HTTP status and response shape. | HTTP 200; non-empty results for a known-populated date range; filter_by field/operation names accepted without 400/422. | If 400/422: xDome API uses different parameter names. Implementer must update `build_claroty_audit_filter_by` with the correct field/operation names from the live API error response. Block release until confirmed. |
| RISK-3 | **Tier-2 device columns provenance is OpenAPI-only.** The 12 Tier-2 device columns were verified against xDome OpenAPI 2026-06-20, NOT against live API responses. | Execute QU-008 (Gate 3) against the live tenant. Also: query `claroty_devices` with `limit 5` and inspect raw_extensions JSON keys against the 12 expected Tier-2 keys. | All 12 expected Tier-2 keys present in live `raw_extensions` JSON; data types consistent with spec; no `E-SPEC-018` parse errors. | If keys differ: update TOML `col.name` values to match live field names. |
| RISK-4 | **No push-down on alerts / devices / device_alert_relations — large-tenant risk.** These 3 tables require full-scan fan-out. Large xDome tenants with >10k devices or alerts risk `E-QUERY-004` timeout. | Execute `SELECT * FROM claroty_devices LIMIT 10000` (or equivalent) against the live tenant. Monitor query time and memory. If result_count > 5000: measure time and memory. | Query completes within 30s for typical tenant size (<5000 devices). If timeout: document actual device count; escalate to architect for a scan-limit or count-cap mechanism before release. | If timeout on typical tenant: add scan-limit or circuit-breaker story to the release milestone. Make the failure explicit to the analyst (E-QUERY-004). |
| RISK-5 | **DTU-parity tests remain `#[ignore]`'d post-ROUTING-001.** The `S-ADR058-DTU-PARITY-MIGRATION-001` story is PARKED. | Acknowledge gap: run all NON-ignored Claroty-path tests via `just iter prism-spec-engine` and `just iter prism-dtu-claroty`. Record the count of skipped parity tests. Confirm each `#[ignore]` has a documented gate condition and story anchor (SID-1 rule). | Non-ignored tests all pass. Each `#[ignore]`'d test has a comment citing blocking dependency. | If any `#[ignore]`'d test lacks a comment: add the comment per SID-1 before release. |
| RISK-6 | **No DTU for G2–G6 expansion tables.** Ten of the 14 expansion tables (claroty_ot_activity_events, claroty_device_vulnerability_relations, claroty_servers, claroty_server_interfaces, and all 5 org-policy/acl tables) have NO DTU clone. All validation for these tables depends entirely on the live monroe sensor. If the live tenant has no data for a table or the API schema diverges from the OpenAPI spec, validation gaps will only be discovered at release gate time. | For each expansion table without DTU: run the corresponding structural live test suite (Variant-1 #[ignore]'d tests per the xdome-endpoint-expansion-plan §Per-Story Pipeline). Record which tests pass and which fail due to missing data or schema divergence. Document any live API schema divergences as findings. | All Variant-1 structural tests pass for each G2–G6 table against the live monroe sensor. Schema mismatches are filed as defects before release. | If structural tests fail: diagnose whether the issue is (a) TOML col.name mismatch with live API, (b) missing data in the live tenant for that endpoint, or (c) a schema change in the live API. Route to implementer for (a) and (c); accept (b) as a known test limitation with documented skipped test reason. |
| RISK-7 | **`claroty_organization_acl_policies` non-paginated endpoint — large-tenant memory risk.** The ACL policies endpoint returns all policies in a single HTTP response with no pagination. Large MSSP tenants with hundreds of ACL policies may produce a large response, potentially straining the 200MB per-query memory budget. | Execute QU-025 (Gate 3) against the live tenant. Monitor peak RSS during the query. If the tenant has >50 ACL policies, capture the response size. | Query completes within 30s. Peak memory overhead stays within 200MB budget. If response exceeds 1MB (suggesting >100 complex policies): document the tenant policy count and estimated memory impact; escalate to architect if memory budget is threatened. | If memory budget exceeded: add a result-set circuit-breaker (max_rows limit) for `type = "none"` tables. This is a deferred post-v1 story if typical MSSP tenants have <200 ACL policies. |

---

## Ignored-Test Un-Gate List

Tests that are currently `#[ignore]`'d and the conditions required to un-gate them for v1 or the next release cycle.

**Original 4-table tests:**

| IGN-ID | Test location | Ignore reason | Gate condition to un-ignore | Target story |
|---|---|---|---|---|
| IGN-001 | `prism-dtu-claroty/src/routes/audit_log.rs` — `test_..._pipeline_integration_ac_006` | Requires `prism-bin` full-boot wiring with DTU; `todo!()` body | `S-DEMO-002` merged AND `todo!()` replaced with real test body | S-DEMO-002 |
| IGN-002 | `prism-spec-engine/tests/parity/claroty.rs` §117 — `test_BC_2_16_013_dtu_parity_claroty` | Requires DTU clone + recorded reference OCSF fixtures; blocked on ROUTING-001 + fixture recording pipeline | `S-ADR058-OCSF-ROUTING-001` merged + reference OCSF fixture for Claroty recorded via DTU | `S-ADR058-DTU-PARITY-MIGRATION-001` |
| IGN-003 | `prism-spec-engine/tests/parity/claroty.rs` §188 — second parity variant | Same gate as IGN-002 | Same as IGN-002 | `S-ADR058-DTU-PARITY-MIGRATION-001` |
| IGN-004 | `prism-spec-engine/src/pipeline.rs` — `test_BC_2_16_002_pagination_claroty_alerts_page_2_returns_data` | Requires DTU clone with 102-entry alerts fixture | DTU alerts fixture with >1000 entries recorded | `S-ADR058-DTU-PARITY-MIGRATION-001` or standalone |
| IGN-005 | `prism-bin/tests/e2e_smoke.rs` — 13 `#[ignore]` attrs (E2E-001) | Requires DTU server + prism binary running; ungated via `e2e` nextest profile | Run via `cargo nextest run --profile e2e` with DTU server started. | CI pipeline story |
| IGN-006 | `prism-bin/tests/e2e_multi_org.rs` — 10 `#[ignore]` attrs (E2E-MULTI-001) | Requires multi-org DTU; ungated via `e2e-multi-org` profile | Run via `cargo nextest run --profile e2e-multi-org` with multi-org DTU harness. | CI pipeline story |

**Expansion table live tests (G2–G6, no DTU — all LIVE-REQUIRED):**

All Variant-1 integration tests for the 10 expansion tables are `#[ignore]`'d pending live monroe sensor access. Each must carry a comment of the form:
```
// LIVE-ONLY-001: requires live monroe sensor; no DTU available per D-2200.
// Un-gate after live validation confirmed (xdome-v1-validation Gate 2/3 checks for this table).
```

| IGN-ID | Table | Test pattern | Un-gate condition |
|---|---|---|---|
| IGN-007 | claroty_ot_activity_events | `prism-sensors/tests/bc_2_16_016_*.rs` §live Variant-1 | Live monroe sensor accessible; structural wire-shape assertions pass |
| IGN-008 | claroty_device_vulnerability_relations | `prism-sensors/tests/bc_2_16_017_*.rs` §live Variant-1 | Live monroe sensor accessible; response_path `$.devices_vulnerabilities` confirmed |
| IGN-009 | claroty_servers | `prism-sensors/tests/bc_2_16_018_*.rs` §live Variant-1 | Live monroe sensor accessible; `uptime_days` Float confirmed |
| IGN-010 | claroty_server_interfaces | `prism-sensors/tests/bc_2_16_019_*.rs` §live Variant-1 | Live monroe sensor accessible; separate endpoint `/api/v1/server_interfaces/` confirmed |
| IGN-011 | claroty_organization_zones | `prism-sensors/tests/bc_2_16_020_*.rs` §zones Variant-1 | Live monroe sensor accessible |
| IGN-012 | claroty_organization_zone_policies | `prism-sensors/tests/bc_2_16_020_*.rs` §zone_policies Variant-1 | Live monroe sensor accessible |
| IGN-013 | claroty_organization_firewall_groups | `prism-sensors/tests/bc_2_16_021_*.rs` §groups Variant-1 | Live monroe sensor accessible |
| IGN-014 | claroty_organization_firewall_policies | `prism-sensors/tests/bc_2_16_021_*.rs` §policies Variant-1 | Live monroe sensor accessible |
| IGN-015 | claroty_organization_acl_policies | `prism-sensors/tests/bc_2_16_022_*.rs` §live Variant-1 | Live monroe sensor accessible; non-paginated single-response fetch confirmed |

---

## Open Questions

| OQ-ID | Question | Why it matters | Owner | Blocking? |
|---|---|---|---|---|
| OQ-001 | **RESOLVED-IN-SCOPE (2026-08-21, ADR-058 §I6, ROUTING-001 RG-PD-001).** `extract_time_window_from_ast` registers BOTH `timestamp` (TOML col.name) AND `time` (Arrow field name) as INDEX-eligible. Live confirmation via PD-007. | Push-down fix scoped into ROUTING-001. | Product-owner (RESOLVED) | RESOLVED-IN-SCOPE |
| OQ-002 | **RESOLVED (2026-08-21) — gated on `json_extract_string` DataFusion ScalarUDF story (S-JSON-EXTRACT-UDF-001, depends_on ROUTING-001).** Tier-2 fields WILL be filterable in v1 via `json_extract_string(json_col, '$.path')` ScalarUDF. Until that story ships, WHERE predicates on `raw_extensions` keys produce an error; SELECT access is available immediately. | OQ-002 resolved as a v1-chain story gated on ROUTING-001. | Product-owner (RESOLVED — gated) | RESOLVED-GATED |
| OQ-003 | **RESOLVED (2026-08-21, BC-2.16.003 §postconditions synthesized column discoverability, ADR-058 §G).** `prism_describe` with `ocsf_column_naming = true` emits `ColumnDescriptor` entries for `class_uid` (Integer, non-nullable) and `_sensor` (String, non-nullable), appended after Tier-1 and Tier-2 descriptors. FM-001..FM-004 updated to assert these descriptors. | OQ-003 resolved. | Product-owner (RESOLVED) | RESOLVED |
| OQ-004 | **T13 demo runbook step 6.3 uses pre-ROUTING-001 column names.** Step 6.3 expects `claroty_audit_logs` to return columns `action`, `actor`, `id`, `resource`, `timestamp`. Post-ROUTING-001: `action` → `activity_name`, `id` → `metadata_uid`, `timestamp` → `time`. The runbook needs a targeted update to use POST-ROUTING-001 names before demo recording (T14). | Demo will fail at Step 6.3 if the runbook is run as-written post-ROUTING-001. | Product-owner (update T13-capstone-demo-runbook.md §6.3 post-ROUTING-001 merge) | YES — blocks T14 recording |
| OQ-005 | **RESOLVED (human-directed 2026-08-21).** `audit_logs.id` maps to `ocsf_field = "metadata.uid"` → Arrow `metadata_uid`, **Tier-1**. FM-002 and FM-012 updated to reflect the correct Tier-1 state. | Audit record ID is now Tier-1 as `metadata_uid`. | Product-owner (RESOLVED) | RESOLVED |

---

## Item Count Summary

| Gate | Category | Count |
|---|---|---|
| Gate 1 | ONBOARDING | 8 |
| Gate 2 | FIELD MAPPING | 46 |
| Gate 3 | QUERIES | 25 |
| Gate 4 | PUSH-DOWN | 8 |
| Gate 5 | SOC-ANALYST Q&A (see companion) | 64 |
| Gate 6 | STABILITY / RESILIENCE | 12 |
| — | PRIORITY-1 RISK CHECKS | 7 |
| — | IGNORE-TEST UN-GATE LIST | 15 (6 original + 9 expansion) |
| — | OPEN QUESTIONS | 5 |
| **Total (excl. SOC Q&A)** | | **106** |
| **Total (incl. SOC Q&A)** | | **170** |

---

## Changelog

| Version | Date | Author | Summary |
|---|---|---|---|
| 0.3 | 2026-09-07 | product-owner | Expanded from 4-table to ALL 14-table coverage. Added POST-ROUTING-001 field name tables for 10 expansion tables (G1–G6). Added OB-008, FM-021..FM-046, QU-013..QU-025, RISK-6..RISK-7, IGN-007..IGN-015. Updated item count from 70/97 to 106/170. This is the AUTHORITATIVE 14-table live release-gate matrix. |
| 0.2 | 2026-08-21 | product-owner | Initial 4-table coverage (alerts, audit_logs, devices, device_alert_relations). 70 items (excl. Q&A), 97 total. KF-05 RESOLVED (metadata_uid Tier-1). OQ-001..OQ-005 resolved. |
