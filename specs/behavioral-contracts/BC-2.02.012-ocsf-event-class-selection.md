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

# BC-2.02.012: OCSF Event Class Selection Per Sensor Record Type

## Preconditions
- A sensor record with a known `record_type` (e.g., `crowdstrike_alert`, `claroty_device`) is being normalized
- The per-sensor mapper has a defined mapping from `record_type` to OCSF event class

## Postconditions
- Each `record_type` maps to exactly one OCSF event class (verified against pinned
  OCSF schema version via ocsf-proto-gen at build time):
  - `crowdstrike_detection` -> Detection Finding (class 2004)
  - `crowdstrike_incident` -> Incident Finding (class 2005)
  - `cyberint_alert` -> Detection Finding (class 2004) + OSINT profile
  - `claroty_alert` -> Detection Finding (class 2004)
  - `armis_alert` -> Detection Finding (class 2004)
  - `claroty_device`, `armis_device` -> Device Inventory Info (class 5001)
  - `claroty_vulnerability` -> Vulnerability Finding (class 2002)
  - `claroty_audit_log`, `armis_audit_log` -> Audit Activity (class 3001)
  - NOTE: Security Finding (class 2001) is DEPRECATED since OCSF v1.1.0 — do not use
  - Remaining types -> Base Event (class 0) with all fields in `raw_extensions`
- The following launch-day record types have no OCSF class mapping and launch as `raw_extensions` only (Base Event class 0): `claroty_event`, `claroty_server`, `claroty_site`, `claroty_relation`, `armis_activity`, `armis_risk_factor`, `armis_connection`. These are queryable via `raw_extensions` and may receive dedicated OCSF mappings in future releases.
- The `event_class` field on `OcsfEvent` reflects the selected class
- The DynamicMessage is created from the correct protobuf descriptor for that class

## Invariants
- Each record type has a deterministic, documented OCSF event class mapping
- DI-005: OCSF schema validity

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Warning | `record_type` has no defined OCSF class mapping | Falls back to Base Event (class 0); all fields in `raw_extensions`; warning logged |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-02-022 | New sensor data source added without OCSF mapping | Falls back to Base Event; the record is still queryable via `raw_extensions` |
| EC-02-023 | Claroty `device_alert_relations` (join table) | Mapped to a relationship-type OCSF class if available, otherwise Base Event; both entity references preserved |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-003 |
| L2 Invariants | DI-005 |
| Priority | P0 |
