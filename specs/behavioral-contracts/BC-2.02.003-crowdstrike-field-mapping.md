---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "OCSF Normalization"
capability: "CAP-003"
---

# BC-2.02.003: CrowdStrike Alert Field Mapping to OCSF

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

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-003 |
| L2 Invariants | DI-005 |
| Priority | P0 |
