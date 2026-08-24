---
document_type: openapi-schema-extract
version: "1.1"
created: 2026-08-24
producer: orchestrator-preextract
source: .factory/reference/api-specs/xdome_openapi_06.20.2026.json
note: "Mechanical field extraction for xDome endpoint-expansion spikes. Types are raw OpenAPI; architect maps to prism_core::column::ColumnType + OCSF."
---

# xDome Endpoint Row-Item Schema Extract


## vulnerabilities
- path: `/api/v1/vulnerabilities/`
- OCSF: `vulnerability_finding/2002`
- envelope keys: `count, vulnerabilities`
- field count: 0

| field | type | format | nested? |
|-------|------|--------|--------|


## ot_activity_events
- path: `/api/v1/ot_activity_events/`
- OCSF: `DECISION-G2 (A:network_activity/4001 vs B:detection_finding/2004)`
- envelope keys: `count, ot_activity_events`
- field count: 0

| field | type | format | nested? |
|-------|------|--------|--------|


## devices_vulnerabilities
- path: `/api/v1/device_vulnerability_relations/`
- OCSF: `vulnerability_finding/2002`
- envelope keys: `count, devices_vulnerabilities`
- field count: 0

| field | type | format | nested? |
|-------|------|--------|--------|


## servers
- path: `/api/v1/servers/`
- OCSF: `inventory_info/5001`
- envelope keys: `count, servers`
- field count: 0

| field | type | format | nested? |
|-------|------|--------|--------|


## server_interfaces
- path: `/api/v1/servers/ (server_interfaces sub)`
- OCSF: `inventory_info/5001`
- envelope keys: `count, server_interfaces`
- field count: 0

| field | type | format | nested? |
|-------|------|--------|--------|


## organization_zones
- path: `/api/v1/organization_zones/`
- OCSF: `entity_management/3004`
- envelope keys: `count, organization_zones`
- field count: 0

| field | type | format | nested? |
|-------|------|--------|--------|


## organization_zone_policies
- path: `/api/v1/organization_zone_policies/`
- OCSF: `entity_management/3004`
- envelope keys: `count, organization_zone_policies`
- field count: 0

| field | type | format | nested? |
|-------|------|--------|--------|


## organization_firewall_groups
- path: `/api/v1/organization_fw_groups/`
- OCSF: `entity_management/3004`
- envelope keys: `count, organization_firewall_groups`
- field count: 0

| field | type | format | nested? |
|-------|------|--------|--------|


## organization_firewall_policies
- path: `/api/v1/organization_fw_group_policies/`
- OCSF: `entity_management/3004`
- envelope keys: `count, organization_firewall_policies`
- field count: 0

| field | type | format | nested? |
|-------|------|--------|--------|


## organization_acl_policies
- path: `/api/v1/organization_acl_policies/`
- OCSF: `entity_management/3004`
- envelope keys: `organization_acl_policies`  ← **NO count (pagination anomaly)**
- field count: 11

| field | type | format | nested? |
|-------|------|--------|--------|
| policy_id | anyOf |  |  |
| policy_name | anyOf |  |  |
| policy_source | anyOf |  |  |
| applied_models | anyOf |  |  |
| matching_devices | anyOf |  |  |
| policy_acl | anyOf |  |  |
| policy_acl_type | anyOf |  |  |
| policy_creation_date | anyOf |  |  |
| policy_last_updated | anyOf |  |  |
| policy_updated_by | anyOf |  |  |
| policy_notes | anyOf |  |  |

---

# Virtual-Table field enums (authoritative queryable field sets)

> xDome row items are generic objects; the queryable/returnable field set per virtual table is its `*__fields_enum`. These are the field lists the architect must map to TOML columns + OCSF. (ACL uses a concrete item schema — see OrganizationAclPolicyResponseItem above.)

## Vulnerability  (field count: 32)
`name`, `vulnerability_type`, `cve_ids`, `cvss_v2_score`, `cvss_v2_exploitability_subscore`, `cvss_v2_vector_string`, `cvss_v3_score`, `cvss_v3_exploitability_subscore`, `cvss_v3_vector_string`, `sources`, `source_name`, `source_url`, `description`, `is_known_exploited`, `affected_devices_count`, `affected_medical_devices_count`, `affected_iot_devices_count`, `affected_it_devices_count`, `affected_ot_devices_count`, `published_date`, `affected_fixed_devices_count`, `affected_confirmed_devices_count`, `affected_potentially_relevant_devices_count`, `affected_irrelevant_devices_count`, `adjusted_vulnerability_score`, `adjusted_vulnerability_score_level`, `exploits_count`, `vulnerability_labels`, `vulnerability_assignees`, `vulnerability_note`, `vulnerability_priority_group`, `epss_score`

## OTActivityEvent  (field count: 23)
`detection_time`, `event_type`, `related_alert_ids`, `description`, `dest_asset_id`, `dest_ip`, `dest_device_type`, `dest_device_name`, `dest_site_name`, `dest_network`, `protocol`, `dest_port`, `source_port`, `source_asset_id`, `source_ip`, `source_device_type`, `source_username`, `source_device_name`, `source_site_name`, `source_network`, `mode`, `event_id`, `ip_protocol`

## DeviceVulnerability  (field count: 214)
`device_network_list`, `device_category`, `device_subcategory`, `device_type`, `device_uid`, `device_detector_name`, `device_asset_id`, `device_mac_list`, `device_ip_list`, `device_type_family`, `device_model`, `device_model_family`, `device_product_code`, `device_os_category`, `device_serial_number`, `device_vlan_list`, `device_retired`, `device_labels`, `device_assignees`, `device_hw_version`, `device_local_name`, `device_os_name`, `device_os_version`, `device_os_revision`, `device_os_patch_level`, `device_os_subcategory`, `device_combined_os`, `device_endpoint_security_names`, `device_equipment_class`, `device_consequence_of_failure`, `device_management_services`, `device_ad_distinguished_name`, `device_ad_description`, `device_mdm_ownership`, `device_mdm_enrollment_status`, `device_mdm_compliance_status`, `device_last_domain_user`, `device_fda_class`, `device_mobility`, `device_criticality`, `device_purdue_level`, `device_purdue_level_source`, `device_dhcp_hostnames`, `device_http_hostnames`, `device_snmp_hostnames`, `device_windows_hostnames`, `device_other_hostnames`, `device_windows_last_seen_hostname`, `device_dhcp_last_seen_hostname`, `device_http_last_seen_hostname`, `device_snmp_last_seen_hostname`, `device_ae_titles`, `device_dhcp_fingerprint`, `device_note`, `device_domains`, `device_battery_level`, `device_internet_communication`, `device_financial_cost`, `device_handles_pii`, `device_machine_type`, `device_phi`, `device_cmms_state`, `device_cmms_ownership`, `device_cmms_asset_tag`, `device_cmms_campus`, `device_cmms_building`, `device_cmms_location`, `device_cmms_floor`, `device_cmms_department`, `device_cmms_owning_cost_center`, `device_cmms_asset_purchase_cost`, `device_cmms_room`, `device_cmms_manufacturer`, `device_cmms_model`, `device_cmms_serial_number`, `device_cmms_last_pm`, `device_cmms_technician`, `device_edr_is_up_to_date_text`, `device_edr_endpoint_status`, `device_mac_oui_list`, `device_ip_assignment_list`, `device_protocol_location_list`, `device_vlan_qualifier_list`, `device_vlan_name_list`, `device_vlan_description_list`, `device_connection_type_list`, `device_ssid_list`, `device_bssid_list`, `device_wireless_encryption_type_list`, `device_ap_name_list`, `device_ap_location_list`, `device_switch_mac_list`, `device_switch_ip_list`, `device_switch_name_list`, `device_switch_port_list`, `device_switch_location_list`, `device_switch_port_description_list`, `device_wlc_name_list`, `device_wlc_location_list`, `device_applied_acl_list`, `device_applied_acl_type_list`, `device_collection_servers`, `device_edge_locations`, `device_last_edge_location_seen_reported_from`, `device_edge_scan_names_seen_reported_from`, `device_integration_types_reported_from`, `device_integrations_reported_from`, `device_last_project_file_path_reported_from`, `device_number_of_nics`, `device_last_domain_user_activity`, `device_last_scan_time`, `device_edr_last_scan_time`, `device_retired_since`, `device_os_eol_date`, `device_last_seen_list`, `device_first_seen_list`, `device_wifi_last_seen_list`, `device_last_seen_on_switch_list`, `device_is_online`, `device_network_scope_list`, `device_end_of_life_state`, `device_end_of_sale_date`, `device_end_of_life_date`, `device_connection_paths`, `device_ise_authentication_method_list`, `device_ise_endpoint_profile_list`, `device_ise_identity_group_list`, `device_ise_security_group_name_list`, `device_ise_security_group_tag_list`, `device_ise_logical_profile_list`, `device_cppm_authentication_status_list`, `device_cppm_roles_list`, `device_cppm_service_list`, `device_visibility_score`, `device_visibility_score_level`, `device_name`, `device_manufacturer`, `device_site_name`, `device_site_group_name`, `device_ot_criticality`, `device_risk_score`, `device_risk_score_points`, `device_effective_likelihood_subscore`, `device_effective_likelihood_subscore_points`, `device_likelihood_subscore`, `device_likelihood_subscore_points`, `device_impact_subscore`, `device_impact_subscore_points`, `device_exposure_scenarios`, `device_known_vulnerabilities`, `device_known_vulnerabilities_points`, `device_insecure_protocols`, `device_insecure_protocols_points`, `device_suspicious`, `device_switch_group_name_list`, `device_managed_by`, `device_authentication_user_list`, `device_collection_interfaces`, `device_data_sources_seen_reported_from`, `device_collection_servers_seen_reported_from`, `device_collection_interfaces_seen_reported_from`, `device_active_queries_seen_reported_from`, `device_edge_hosts_seen_reported_from`, `device_edge_locations_seen_reported_from`, `device_last_seen_reported`, `device_slot_cards`, `device_cmms_financial_cost`, `device_software_or_firmware_version`, `device_enforcement_or_authorization_profiles_list`, `device_ise_security_group_description_list`, `device_recommended_firewall_group_name`, `device_recommended_trustsec_group_name`, `device_organization_trustsec_group_name`, `device_recommended_zone_name`, `device_recommended_trustsec_policy`, `device_organization_trustsec_policy`, `device_recommended_firewall_policy`, `device_organization_firewall_policy`, `device_recommended_zone_policy`, `device_organization_zone_policy`, `vulnerability_id`, `vulnerability_name`, `vulnerability_type`, `vulnerability_cve_ids`, `vulnerability_cvss_v3_score`, `vulnerability_cvss_v3_vector_string`, `vulnerability_cvss_v3_exploitability_subscore`, `vulnerability_cvss_v2_score`, `vulnerability_cvss_v2_vector_string`, `vulnerability_cvss_v2_exploitability_subscore`, `vulnerability_adjusted_vulnerability_score`, `vulnerability_adjusted_vulnerability_score_level`, `vulnerability_epss_score`, `vulnerability_sources`, `vulnerability_description`, `vulnerability_exploits_count`, `vulnerability_is_known_exploited`, `vulnerability_published_date`, `vulnerability_labels`, `vulnerability_assignees`, `vulnerability_note`, `vulnerability_last_updated`, `vulnerability_relevance`, `vulnerability_relevance_sources`, `vulnerability_manufacturer_remediation_info`, `vulnerability_manufacturer_remediation_info_source`, `vulnerability_manufacturer_remediation_info_required_actions`, `vulnerability_overall_cvss_v3_score`, `device_vulnerability_detection_date`, `device_vulnerability_resolution_date`, `device_vulnerability_days_to_resolution`, `patch_install_date`, `device_vulnerability_note`, `device_vulnerability_relevance_reasons`

## Server  (field count: 17)
`server_name`, `server_location`, `server_status`, `site_id`, `model`, `os_version`, `serial_number`, `num_of_interfaces`, `management_ip`, `idrac_ip`, `management_mac`, `uptime_days`, `avg_traffic_past_month_mbps`, `avg_traffic_past_week_mbps`, `avg_traffic_past_hour_mbps`, `num_of_open_incidents`, `notes`

## ServerInterfaces  (field count: 10)
`server_name`, `interface_name`, `interface_status`, `interface_type`, `interface_connection_type`, `site_id`, `avg_traffic_past_month_mbps`, `avg_traffic_past_week_mbps`, `avg_traffic_past_hour_mbps`, `notes`

## OrganizationZones  (field count: 11)
`priority`, `device_conditions`, `attributed_devices`, `exportable_attributed_devices`, `created_time`, `last_update`, `updated_by`, `enabled`, `zone_source`, `zone_name`, `zone_description`

## OrganizationZonePolicies  (field count: 13)
`policy_source`, `policy_action`, `communication_conditions`, `matching_devices`, `created_time`, `last_updated`, `updated_by`, `policy_notes`, `alert_use_case`, `related_alerts_ids`, `should_generate_alerts`, `policy_name`, `applied_zone_pairs`

## OrganizationFirewallGroups  (field count: 11)
`priority`, `device_conditions`, `attributed_devices`, `exportable_attributed_devices`, `created_time`, `last_update`, `updated_by`, `enabled`, `firewall_group_source`, `firewall_group_name`, `firewall_group_description`

## OrganizationFirewallGroupPolicies  (field count: 13)
`policy_source`, `policy_action`, `communication_conditions`, `matching_devices`, `created_time`, `last_updated`, `updated_by`, `policy_notes`, `alert_use_case`, `related_alerts_ids`, `should_generate_alerts`, `policy_name`, `applied_group_pairs`

## OrganizationAclPolicy  (field count: 11)
`policy_id`, `policy_name`, `policy_source`, `applied_models`, `matching_devices`, `policy_acl_type`, `policy_acl`, `policy_creation_date`, `policy_last_updated`, `policy_updated_by`, `policy_notes`

