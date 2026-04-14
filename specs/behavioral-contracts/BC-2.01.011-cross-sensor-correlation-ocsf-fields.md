---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "Sensor Query Pipeline"
capability: "CAP-012"
---

# BC-2.01.011: Cross-Sensor Correlation via OCSF Field Alignment

## Preconditions
- At least two different sensors are configured for the same client
- Both sensors have records that map to OCSF fields suitable for correlation (e.g., `device.ip`, `device.hostname`, `time`)

## Postconditions
- All sensor records include an `ocsf` field containing the OCSF-normalized representation
- CrowdStrike `hostname` and Claroty `device_name` both map to OCSF `device.hostname`
- CrowdStrike `local_ip` and Armis `ipaddress` both map to OCSF `device.ip`
- Timestamp fields from all sensors map to OCSF `time` in a consistent format (RFC 3339)
- The AI agent can join results from different sensors using these common OCSF fields

## Invariants
- DI-005: OCSF schema validity -- all returned OcsfEvents conform to the compiled schema
- DI-008: Client data separation -- correlation is within a single client's data

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| N/A | A sensor record has no mappable IP or hostname | OCSF fields left absent (OCSF fields are optional by design); `raw_extensions` preserves original fields |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-01-016 | CrowdStrike reports hostname "WORKSTATION-01" and Claroty reports "workstation-01" for the same device | OCSF `device.hostname` preserves original casing from each sensor; case-insensitive matching is the AI agent's responsibility |
| EC-01-017 | Armis reports an IPv6 address while CrowdStrike reports IPv4 for the same dual-stack device | Both addresses mapped to their respective OCSF IP fields; the agent must handle dual-stack correlation |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-012 |
| L2 Invariants | DI-005, DI-008 |
| Priority | P1 |
