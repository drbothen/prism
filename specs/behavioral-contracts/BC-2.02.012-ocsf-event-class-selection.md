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
input-hash: "76729b7"
traces_to: ["CAP-003"]
extracted_from: ".factory/specs/prd.md"
introduced: cycle-1
modified: "2026-06-03"  # v1.4 OCSF-CLASS-MIGRATION-001 Wave-5 Phase-A PO burst
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.02.012: OCSF Event Class Selection Per Sensor Record Type

## Description

Each sensor record type maps deterministically to exactly one OCSF event class, selected by the per-sensor mapper and verified against the pinned OCSF schema at build time. The primary mappings are: detection-type alerts to Detection Finding (2004), devices to Device Inventory Info (5001), vulnerabilities to Vulnerability Finding (2002), and audit logs to Audit Activity (3001). Security Finding (2001) was deprecated in OCSF v1.1.0 and is retained ONLY as a transitional alias in `select_by_class_name` — the four production sensor TOMLs MUST use `detection_finding` (class 2004) after OCSF-CLASS-MIGRATION-001 merges. Record types with no defined OCSF class fall back to Base Event (class 0) with all fields in `raw_extensions`.

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
  - NOTE: Security Finding (class 2001) is DEPRECATED since OCSF v1.1.0 — see class-name mapping rule below
  - Remaining types -> Base Event (class 0) with all fields in `raw_extensions`
- The following launch-day record types have no OCSF class mapping and launch as `raw_extensions` only (Base Event class 0): `claroty_event`, `claroty_server`, `claroty_site`, `claroty_relation`, `armis_activity`, `armis_risk_factor`, `armis_connection`. These are queryable via `raw_extensions` and may receive dedicated OCSF mappings in future releases.
- The `event_class` field on `OcsfEvent` reflects the selected class
- The DynamicMessage is created from the correct protobuf descriptor for that class

### `select()` path (record-type tokens) — deprecated class 2001 prohibition
The `EventClassSelector::select(sensor_id, record_type)` function maps record-type TOKENS
(`"detection"`, `"alert"`, `"device"`, `"audit_log"`, etc.). This path MUST NOT return
class_uid 2001 (Security Finding, deprecated OCSF v1.1.0). Any record-type token that
previously mapped to 2001 must be updated to map to 2004 (Detection Finding) or another
current OCSF class as appropriate for the semantic type. No new record-type token may be
introduced that maps to 2001.

### `select_by_class_name()` path (OCSF class-name strings from TOML) — transitional alias
The `EventClassSelector::select_by_class_name(class_name)` function maps OCSF class-name
STRINGS declared in sensor TOML `ocsf_class` fields. This function MUST implement:

| `class_name` value (TOML `ocsf_class`) | `class_uid` returned | Notes |
|----------------------------------------|---------------------|-------|
| `"detection_finding"` | 2004 | OCSF v1.1 canonical; PRIMARY entry |
| `"security_finding"` | 2004 | Transitional alias — maps to 2004 (NOT 2001) with deprecation WARN emission. External TOML specs not under Prism control may still use this string. Introduced per OCSF-CLASS-MIGRATION-001 (Option A). |
| `"incident_finding"` | 2005 | CrowdStrike incidents, Cyberint incidents |
| `"vulnerability_finding"` | 2002 | Claroty vulnerabilities |
| `"device"` | 5001 | Claroty/Armis devices (InventoryInfo) |
| `"audit_activity"` | 3001 | Claroty/Armis audit logs (AccountChange — closest OCSF v1.7.0 class) |

**Deprecation WARN emission for `"security_finding"` alias:** When
`select_by_class_name("security_finding")` is called, the implementation MUST emit:
`tracing::warn!(event_type = "ocsf.deprecated_class_alias", class_name = "security_finding", resolved_class_uid = 2004, "sensor TOML uses deprecated ocsf_class value 'security_finding'; update to 'detection_finding'")`.
This is a per-invocation WARN — callers should update their TOML specs to use `"detection_finding"` directly.

**Production sensor TOML constraint (post-OCSF-CLASS-MIGRATION-001):** The four production
sensor TOML specs bundled in `crates/prism-sensors/specs/` MUST declare
`ocsf_class = "detection_finding"` (not `"security_finding"`) in all alert/detection tables.
The grep audit `rg 'ocsf_class.*security_finding' crates/prism-sensors/specs/` MUST return
zero results after OCSF-CLASS-MIGRATION-001 merges.

**`select_by_class_name("detection_finding")` postcondition:** Returns `2004` (no WARN
emitted). This is the canonical path post-migration; all production TOML specs use this.

## Invariants
- Each record type has a deterministic, documented OCSF event class mapping
- DI-005: OCSF schema validity
- **INV-NO-2001-SELECT-PATH:** The `select()` path (record-type tokens) MUST NOT map any
  token to class_uid 2001 (Security Finding, deprecated OCSF v1.1.0).
- **INV-PRODUCTION-TOML-NO-SECURITY-FINDING (post-OCSF-CLASS-MIGRATION-001):** No
  production sensor TOML in `crates/prism-sensors/specs/` declares
  `ocsf_class = "security_finding"` after OCSF-CLASS-MIGRATION-001 merges to develop.

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Warning | `record_type` has no defined OCSF class mapping | Falls back to Base Event (class 0); all fields in `raw_extensions`; warning logged |
| WARN (tracing) | `select_by_class_name("security_finding")` called | Emits `event_type = "ocsf.deprecated_class_alias"` WARN; returns 2004; does not fail |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-02-022 | New sensor data source added without OCSF mapping | Falls back to Base Event; the record is still queryable via `raw_extensions` |
| EC-02-023 | Claroty `device_alert_relations` (join table) | Mapped to a relationship-type OCSF class if available, otherwise Base Event; both entity references preserved |
| EC-02-024 | External user-supplied TOML still declares `ocsf_class = "security_finding"` | `select_by_class_name` returns 2004 with deprecation WARN; query executes successfully; no rejection; user should update TOML to `"detection_finding"` |
| EC-02-025 | Conformance test with `ocsf_class = "security_finding"` (pre-migration test fixture) | `select_by_class_name("security_finding")` returns 2004 (Option A alias); test must assert class_uid == 2004, NOT 2001; stale assertions of 2001 are test defects to be corrected by OCSF-CLASS-MIGRATION-001 |

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.02.012-001 | `crowdstrike_detection` record | OCSF event class 2004 (Detection Finding); DynamicMessage from correct descriptor |
| TV-BC-2.02.012-002 | `claroty_device` record | OCSF event class 5001 (Device Inventory Info) |
| TV-BC-2.02.012-003 | `claroty_vulnerability` record | OCSF event class 2002 (Vulnerability Finding) |
| TV-BC-2.02.012-004 | `armis_audit_log` record | OCSF event class 3001 (Audit Activity) |
| TV-BC-2.02.012-005 | `claroty_event` (no OCSF mapping, launch-day) | Base Event class 0; all fields in `raw_extensions`; warning logged |
| TV-BC-2.02.012-006 | Entirely new unrecognized record type | Base Event class 0; `raw_extensions` preserved; warning logged |
| TV-BC-2.02.012-007 | `select_by_class_name("detection_finding")` called | Returns `Some(2004)`; no WARN emitted |
| TV-BC-2.02.012-008 | `select_by_class_name("security_finding")` called (transitional alias) | Returns `Some(2004)` (NOT `Some(2001)`); `event_type = "ocsf.deprecated_class_alias"` WARN emitted |
| TV-BC-2.02.012-009 | Post-OCSF-CLASS-MIGRATION-001: `rg 'ocsf_class.*security_finding' crates/prism-sensors/specs/` | Returns zero results — all four production sensor TOMLs use `"detection_finding"` |

## Verification Properties

| VP | Verification Aspect |
|----|---------------------|
| VP-016 | OCSF normalization: output is valid protobuf (proptest) |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-003 |
| Capability Anchor Justification | CAP-003 ("OCSF normalization") per capabilities.md §CAP-003 — this BC specifies OCSF event class selection, which is exactly the mapping from sensor record types to OCSF schema classes that CAP-003 mandates. The `select_by_class_name` function specified here is a direct artifact of CAP-003's requirement that TOML spec `ocsf_class` fields drive canonical class_uid derivation. |
| L2 Invariants | DI-005 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.4 | Wave-5-Phase-A-PO-burst | 2026-06-03 | product-owner | OCSF-CLASS-MIGRATION-001 (Wave 5) gate: (1) Added `select_by_class_name()` path specification with full mapping table — `"detection_finding"` → 2004 (canonical), `"security_finding"` → 2004 (transitional alias, WARN emitted), and 4 other class names. Transitional alias uses Option A (keep alias, emit deprecation WARN, do not reject) per D-989 PO decision: Option A is production-grade because external TOML specs not under Prism control may use the old string value. (2) Added `select()` path invariant: INV-NO-2001-SELECT-PATH — select() record-type token path MUST NOT return class_uid 2001. (3) Added INV-PRODUCTION-TOML-NO-SECURITY-FINDING invariant (post-migration). (4) Updated §Description to reflect transitional alias semantics. (5) Added EC-02-024/025, TV-BC-2.02.012-007/008/009. (6) Added Capability Anchor Justification (S-7.01). (7) Added `ocsf.deprecated_class_alias` WARN emission spec in §Error Cases. Closes OQ-1 (Option A selected), OQ-2 (BC amended as required by story). BC v1.3 → v1.4. |
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added inputs/input-hash/traces_to/extracted_from frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors; added ## Verification Properties; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract. |
