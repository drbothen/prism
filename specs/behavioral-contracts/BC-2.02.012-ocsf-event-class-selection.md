---
document_type: behavioral-contract
level: L3
version: "1.3"
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

# BC-2.02.012: OCSF Event Class Selection Per Sensor Record Type

## Description

Each sensor record type maps deterministically to exactly one OCSF event class, selected by the per-sensor mapper and verified against the pinned OCSF schema at build time. The primary mappings are: detection-type alerts to Detection Finding (2004), devices to Device Inventory Info (5001), vulnerabilities to Vulnerability Finding (2002), and audit logs to Audit Activity (3001). Security Finding (2001) is deprecated since OCSF v1.1.0 and must not be used. Record types with no defined OCSF class fall back to Base Event (class 0) with all fields in `raw_extensions`.

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

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.02.012-001 | `crowdstrike_detection` record | OCSF event class 2004 (Detection Finding); DynamicMessage from correct descriptor |
| TV-BC-2.02.012-002 | `claroty_device` record | OCSF event class 5001 (Device Inventory Info) |
| TV-BC-2.02.012-003 | `claroty_vulnerability` record | OCSF event class 2002 (Vulnerability Finding) |
| TV-BC-2.02.012-004 | `armis_audit_log` record | OCSF event class 3001 (Audit Activity) |
| TV-BC-2.02.012-005 | `claroty_event` (no OCSF mapping, launch-day) | Base Event class 0; all fields in `raw_extensions`; warning logged |
| TV-BC-2.02.012-006 | Entirely new unrecognized record type | Base Event class 0; `raw_extensions` preserved; warning logged |

## Verification Properties

| VP | Verification Aspect |
|----|---------------------|
| VP-016 | OCSF normalization: output is valid protobuf (proptest) |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-003 |
| L2 Invariants | DI-005 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added inputs/input-hash/traces_to/extracted_from frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors; added ## Verification Properties; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract. |
