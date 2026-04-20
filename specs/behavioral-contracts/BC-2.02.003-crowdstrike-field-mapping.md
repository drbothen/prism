---
document_type: behavioral-contract
level: L3
version: "1.1"
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
input-hash: "365fb25"
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

The CrowdStrike normalizer maps CrowdStrike alert fields to their canonical OCSF Detection Finding (class 2004) equivalents. Key mappings include `hostname` → `device.hostname`, `local_ip` → `device.ip`, severity integer (1-5) → `severity_id` enum, `created_timestamp` → OCSF `time` (RFC 3339), and MITRE ATT&CK fields to `attacks[].technique`. Any CrowdStrike-specific fields lacking an OCSF equivalent (e.g., `agent_id`, `cid`) are preserved in `raw_extensions` without loss.

## Preconditions
- A CrowdStrike alert record has been fetched via the two-step QueryV2/PostEntities flow
- The record contains CrowdStrike-specific field names (e.g., `hostname`, `local_ip`, `severity`, `created_timestamp`)

## Postconditions
- CrowdStrike `hostname` maps to OCSF `device.hostname`
- CrowdStrike `local_ip` maps to OCSF `device.ip`
- CrowdStrike `severity` (1-5 integer) maps to OCSF `severity_id` enum
- CrowdStrike `created_timestamp` maps to OCSF `time` (RFC 3339)
- CrowdStrike `technique` / `tactic` maps to OCSF `attacks[].technique` (MITRE ATT&CK alignment)
- CrowdStrike-specific fields with no OCSF equivalent (e.g., `agent_id`, `cid`) are preserved in `raw_extensions`

## Invariants
- DI-005: OCSF schema validity

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Warning | CrowdStrike severity value outside 1-5 range | Mapped to OCSF `severity_id: 99` (Other); warning logged with the raw value |
| Warning | `created_timestamp` is null or missing | OCSF `time` set to fetch timestamp; warning logged |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-007 | Custom CrowdStrike field `custom_tags` with no OCSF mapping | Preserved in `raw_extensions`; debug-level log records the unmapped field |
| EC-02-005 | CrowdStrike alert with 32+ fields (full hydration) | All 32 known fields mapped; any additional fields go to `raw_extensions` |

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.02.003-001 | CrowdStrike alert with severity=4, hostname, local_ip, created_timestamp | OCSF `severity_id: 4`, `device.hostname`, `device.ip`, `time` set correctly |
| TV-BC-2.02.003-002 | Severity value = 7 (out of 1-5 range) | `severity_id: 99` (Other); warning logged with raw value 7 |
| TV-BC-2.02.003-003 | `created_timestamp` is null | OCSF `time` = fetch timestamp; warning logged |
| TV-BC-2.02.003-004 | Record contains `custom_tags` (unmapped field) | `raw_extensions.custom_tags` preserved; debug log entry |
| TV-BC-2.02.003-005 | Full hydration: 32 known fields + 3 unknown | All 32 mapped; 3 in `raw_extensions` |

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

| Version | Burst | Date | Author | Changes |
|---------|-------|------|--------|---------|
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added inputs/input-hash/traces_to/extracted_from frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors; added ## Verification Properties with VP-016/VP-017; added ## Changelog. |
