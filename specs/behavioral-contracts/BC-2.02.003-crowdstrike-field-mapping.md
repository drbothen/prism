---
document_type: behavioral-contract
level: L3
version: "1.4"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-02"
capability: "CAP-003"
lifecycle_status: active
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "572c2a9"
traces_to: ["CAP-003"]
extracted_from: ".factory/specs/prd.md"
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.02.003: CrowdStrike Alert Field Mapping to OCSF

## Description

The CrowdStrike normalizer maps CrowdStrike alert fields to their canonical OCSF Detection Finding (class 2004) equivalents. Key mappings include `device.hostname` → `device.name`, `device.local_ip` → `device.ip`, severity string (e.g., `"High"`) → `severity_id` integer via OCSF v1.x name-to-id mapping, `created_timestamp` → OCSF `time` (RFC 3339), and MITRE ATT&CK `behaviors[*].tactic`/`technique` to `attacks[*].tactic.name`/`technique.name`. The original CrowdStrike `severity_name` string is preserved in `raw_extensions["crowdstrike_severity_name"]`. Any CrowdStrike-specific fields lacking an OCSF equivalent (e.g., `agent_id`, `cid`) are preserved in `raw_extensions` without loss.

## Preconditions
- A CrowdStrike alert record has been fetched via the two-step QueryV2/PostEntities flow
- The record contains CrowdStrike-specific field names (e.g., `device.hostname`, `device.local_ip`, `severity`, `severity_name`, `created_timestamp`)

## Postconditions
- CrowdStrike `device.hostname` maps to OCSF `device.name`
- CrowdStrike `device.device_id` maps to OCSF `device.uid`
- CrowdStrike `device.local_ip` maps to OCSF `device.ip`
- CrowdStrike `device.os_version` maps to OCSF `device.os.version`
- CrowdStrike `severity` (string, e.g., `"High"`) maps to OCSF `severity_id` integer using OCSF v1.x name-to-id mapping: `"Informational"`→1, `"Low"`→2, `"Medium"`→3, `"High"`→4, `"Critical"`→5; unrecognized strings map to `severity_id: 99` (Other)
- CrowdStrike `severity_name` string is preserved in `raw_extensions["crowdstrike_severity_name"]` (not used for `severity_id` derivation — `severity` field drives the mapping)
- CrowdStrike `detection_id` maps to OCSF `finding_info.uid`
- CrowdStrike `created_timestamp` maps to OCSF `time` (RFC 3339)
- CrowdStrike `behaviors[*].tactic` maps to OCSF `attacks[*].tactic.name`
- CrowdStrike `behaviors[*].technique` maps to OCSF `attacks[*].technique.name`
- CrowdStrike `ioc_type` + `ioc_value` map to OCSF `evidences[0].data.type` + `evidences[0].data.value`
- CrowdStrike-specific fields with no OCSF equivalent (e.g., `agent_id`, `cid`) are preserved in `raw_extensions`

## Invariants
- DI-005: OCSF schema validity

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Warning | CrowdStrike `severity` string does not match any OCSF v1.x severity name (unrecognized string) | Mapped to OCSF `severity_id: 99` (Other); warning logged with the raw string value |
| Warning | `created_timestamp` is null or missing | OCSF `time` set to fetch timestamp; warning logged |
| Error | `detection_id` field is missing from a detection record | Returns `Err(PrismError::OcsfNormalizationFailed)` containing field name and source context |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-007 | Custom CrowdStrike field `custom_tags` with no OCSF mapping | Preserved in `raw_extensions`; debug-level log records the unmapped field |
| EC-02-005 | CrowdStrike alert with 32+ fields (full hydration) | All 32 known fields mapped; any additional fields go to `raw_extensions` |

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.02.003-001 | CrowdStrike detection with `"severity": "High"`, `"detection_id": "ldt:abc123"`, hostname, local_ip, created_timestamp | OCSF `severity_id: 4`, `finding_info.uid: "ldt:abc123"`, `device.name`, `device.ip`, `time` set correctly; `raw_extensions["crowdstrike_severity_name"]` preserved |
| TV-BC-2.02.003-002 | `"severity": "Critical"` | `severity_id: 5` (Critical per OCSF v1.x) |
| TV-BC-2.02.003-003 | `"severity": "UNKNOWN_VENDOR_LEVEL"` (unrecognized string) | `severity_id: 99` (Other); warning logged with raw string |
| TV-BC-2.02.003-004 | `created_timestamp` is null | OCSF `time` = fetch timestamp; warning logged |
| TV-BC-2.02.003-005 | Detection with `"behaviors": [{"tactic": "Discovery", "technique": "T1082"}]` | `attacks[0].tactic.name == "Discovery"`, `attacks[0].technique.name == "T1082"` |
| TV-BC-2.02.003-006 | Record contains `custom_tags` (unmapped field) | `raw_extensions.custom_tags` preserved; debug log entry |
| TV-BC-2.02.003-007 | Full hydration: 32 known fields + 3 unknown | All 32 mapped; 3 in `raw_extensions` |

## Verification Properties

| VP | Verification Aspect |
|----|---------------------|
| VP-016 | OCSF normalization: output is valid protobuf (proptest) |
| VP-017 | OCSF normalization: unmapped fields preserved (proptest) |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-003 |
| L2 Invariants | DI-005 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.4 | BLOCK-WV1-04 | 2026-04-22 | product-owner | Severity format fix: corrected CrowdStrike severity from integer (1-5) to string (e.g., "High") with OCSF v1.x name-to-id mapping; added severity_name preservation in raw_extensions; expanded postconditions with full field list per S-1.05 Task 2; updated test vectors to use string severity; added missing-detection_id error case. |
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added inputs/input-hash/traces_to/extracted_from frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors; added ## Verification Properties with VP-016/VP-017; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract. |
