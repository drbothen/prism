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

# BC-2.02.006: Armis Centrix Field Mapping to OCSF (7 Data Sources)

## Preconditions
- An Armis record has been fetched via AQL GetSearch from one of the 7 sources
- Timestamp extraction used the per-source fallback chain successfully (or fell back to fetch timestamp)

## Postconditions
- Armis `ipaddress` maps to OCSF `device.ip`
- Armis `name` (device name) maps to OCSF `device.hostname`
- Armis alert severity maps to OCSF `severity_id`
- Armis `riskLevel` maps to OCSF risk score fields
- Armis-specific fields (e.g., `aqlResults`, `connectionType`, `riskFactors`) are preserved in `raw_extensions`
- Each of the 7 Armis sources maps to an appropriate OCSF event class

## Invariants
- DI-005: OCSF schema validity

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Warning | Armis record missing all timestamp fallback fields | OCSF `time` set to fetch timestamp; warning logged (DEC-013) |
| Warning | Armis severity/risk value in unexpected format | Best-effort mapping; unrecognized values go to `raw_extensions` |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-02-009 | Armis `connections` records (network flow data) | Mapped to OCSF Network Activity class; source/destination IPs extracted from connection fields |
| EC-02-010 | Armis `risk_factors` records (metadata about risk scoring) | Mapped to generic OCSF event; risk factor details in `raw_extensions` for agent consumption |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-003 |
| L2 Invariants | DI-005 |
| Priority | P0 |
