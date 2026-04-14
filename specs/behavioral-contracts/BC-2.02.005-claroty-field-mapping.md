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

# BC-2.02.005: Claroty xDome Field Mapping to OCSF (9 Data Sources)

## Preconditions
- A Claroty xDome record has been fetched from one of the 9 endpoints
- Polymorphic IDs have been normalized by the Claroty adapter

## Postconditions
- Claroty `device_name` maps to OCSF `device.hostname`
- Claroty device IP fields map to OCSF `device.ip`
- Claroty alert severity maps to OCSF `severity_id`
- Claroty OT-specific fields (e.g., `zone`, `protocol`, `firmware_version`) are preserved in `raw_extensions`
- Each of the 9 Claroty sources maps to an appropriate OCSF event class (alerts to Security Finding, devices to Inventory Info, vulnerabilities to Vulnerability Finding)

## Invariants
- DI-005: OCSF schema validity

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Warning | Claroty record with polymorphic ID that cannot be stringified | ID placed in `raw_extensions` as raw JSON; OCSF ID field left absent |
| Warning | Claroty source type has no defined OCSF event class mapping | Record normalized to generic OCSF Base Event (class 0); all fields go to `raw_extensions` |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-02-007 | Claroty `device_alert_relations` records (join table, not primary entity) | Mapped to OCSF with both device and alert references in the message; primarily useful for correlation |
| EC-02-008 | Claroty audit_log records (admin actions, not security events) | Mapped to OCSF Audit Activity (class 3001); admin-specific fields in `raw_extensions` |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-003 |
| L2 Invariants | DI-005 |
| Priority | P0 |
