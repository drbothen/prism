---
document_type: behavioral-contract
level: L3
version: "1.7"
status: active
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
input-hash: "4a1f396"
traces_to: ["CAP-003"]
extracted_from: ".factory/specs/prd.md"
scheduled_amendment_in: null
amendment_lifecycle: null
introduced: cycle-1
modified: "2026-08-12"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.02.005: Claroty xDome Field Mapping to OCSF (9 Data Sources)

## Description

> **Amendment — ADR-023 (PLUGIN-MIGRATION-001-G):** This BC previously described a
> hardcoded Rust mapper module (`prism-ocsf/src/mappers/claroty.rs`). That
> implementation was deleted in PLUGIN-MIGRATION-001-C (PR #158). The field-mapping behavior
> described here is now delivered by `SpecDrivenMapper` reading `ocsf_field` column
> annotations from the Claroty TOML sensor spec. The behavioral contract itself
> is unchanged — the same OCSF field mappings must be produced; they are now
> data-driven via TOML annotations per ADR-023 Rule 1.

`SpecDrivenMapper` reads `ocsf_field` column annotations from the Claroty TOML sensor spec and handles 9 distinct xDome data sources, mapping each to an appropriate OCSF event class: alerts to Security Finding (2004), devices to Device Inventory Info (5001), vulnerabilities to Vulnerability Finding (2002), and audit logs to Audit Activity (3001). Polymorphic IDs are pre-normalized before field mapping occurs. OT-specific fields (e.g., `zone`, `protocol`, `firmware_version`) with no OCSF equivalent are preserved in `raw_extensions`.

## Preconditions
- A Claroty xDome record has been fetched from one of the 9 endpoints
- Polymorphic IDs have been normalized by the Claroty adapter

## Postconditions
- Claroty `device_name` maps to OCSF `device.name` (grounded: `claroty.sensor.toml` `devices` table `device_name` column, `ocsf_field = "device.name"`, per `fix/claroty-live-api-fidelity` / F-CLARO-P2-HIGH-001)
- Claroty `ip_list` (array column) is extracted via `source_path = "$.ip_list[*]"` (ENRICH-1 wildcard); no `ocsf_field` is declared — extracted values land in `raw_extensions`, not OCSF `device.ip`. Routing `ip_list` → `device.ip` is a follow-up pending array→ocsf_field column-grammar support (ENRICH-1 scope; see §ip_list→device.ip follow-up below).
- Claroty alerts carry no severity column: the `severity` field was removed in the Gap-CL-005 fix (2026-05-29); no `severity` field exists in the Claroty xDome alerts API or DTU `ClarotyAlert` struct. Severity signal for Claroty resides on `device_alert_relations` rows via `device_risk_score`, `network_signature_severity`, `network_signature_confidence`, and `malicious_ip_severity` (BC-2.16.013 §Postconditions §1).
- Claroty OT-specific fields (e.g., `zone`, `protocol`, `firmware_version`) are preserved in `raw_extensions`
- Each of the 9 Claroty sources maps to an appropriate OCSF event class (alerts to Security Finding, devices to Inventory Info, vulnerabilities to Vulnerability Finding)

### ip_list → device.ip follow-up

The `ip_list` column uses `source_path = "$.ip_list[*]"` (ENRICH-1 wildcard array extraction) and deliberately omits an `ocsf_field` value. This is the correct behavior for the current column grammar, which does not yet support mapping an array-extracted column to an OCSF field path (the `source_path` + `ocsf_field` combination is not supported by `SpecDrivenMapper`). The OCSF field `device.ip` is the semantically correct target for IP address data. Routing `ip_list` → `device.ip` should be addressed in a follow-up story targeting ENRICH-1 / array→ocsf_field grammar support. Until then, the raw IP array is accessible via `raw_extensions`. This gap does NOT block the field-mapping contract for the other Claroty device columns.

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

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.02.005-001 | Claroty device record with `device_name` and `ip_list` | Mapped to Device Inventory Info 5001; OCSF `device.name` set from `device_name`; `ip_list` array values in `raw_extensions` (ENRICH-1 `source_path` extraction, no `ocsf_field`) |
| TV-BC-2.02.005-002 | Claroty device record with OT fields (zone, protocol) | Mapped to Device Inventory Info 5001; `zone` and `protocol` in `raw_extensions` |
| TV-BC-2.02.005-003 | Claroty vulnerability record | Mapped to Vulnerability Finding 2002; CVE fields mapped |
| TV-BC-2.02.005-004 | Claroty audit_log record | Mapped to Audit Activity 3001; admin action fields in `raw_extensions` |
| TV-BC-2.02.005-005 | Unknown Claroty source type | Falls back to Base Event class 0; all fields in `raw_extensions`; warning logged |

## Verification Properties

| VP | Verification Aspect |
|----|---------------------|
| VP-016 | OCSF normalization: output is valid protobuf (proptest) |
| VP-017 | OCSF normalization: unmapped fields preserved (proptest) |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-003 |
| Capability Anchor Justification | CAP-003 ("OCSF Normalization") per capabilities.md §CAP-003 — this BC specifies the OCSF field-mapping contract for the Claroty xDome sensor's 9 data sources, defining how vendor-specific fields (`device_name` → `device.name`, `ip_list` via ENRICH-1 `source_path`) map to OCSF equivalents via `SpecDrivenMapper`. This is exactly what CAP-003 ("OCSF Normalization") defines: normalize all sensor records to OCSF v1.x via the DynamicMessage protobuf pattern, mapping vendor-specific fields to a common schema. |
| L2 Invariants | DI-005 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.7 | claroty-live-api-fidelity-bc-amendment | 2026-08-12 | product-owner | F-CLARO-P2-HIGH-001 closure (human-authorized spec-amendment-to-match-code per CLAUDE.md §Source-of-Truth Precedence rule 7). §Postconditions: (1) `device_name` → `device.hostname` corrected to `device_name` → `device.name`, grounded in `claroty.sensor.toml` `devices` table `device_name` column `ocsf_field = "device.name"` on `fix/claroty-live-api-fidelity`. (2) `device IP fields → device.ip` corrected to `ip_list` extracted via `source_path = "$.ip_list[*]"` (ENRICH-1 wildcard) landing in `raw_extensions` (no `ocsf_field` declared); follow-up subsection added documenting the `ip_list` → `device.ip` gap and ENRICH-1 scope. (3) `alert severity → severity_id` corrected: severity column absent since Gap-CL-005 fix (2026-05-29); severity signal resides on `device_alert_relations` rows. §Canonical Test Vectors: TV-BC-2.02.005-001 updated from stale alert-record/`device.hostname`/`severity_id` scenario to device-record scenario asserting `device.name` set from `device_name` and `ip_list` in `raw_extensions`. §Traceability: added Capability Anchor Justification row (CAP-003 "OCSF Normalization", verbatim per capabilities.md §CAP-003). Origin: adversary PR-LEVEL pass 2 finding F-CLARO-P2-HIGH-001; backport row S-DEMO-CLAROTY-LIVE-DRIFT-BACKPORT-001. |
| 1.6 | wave-a-spec-evolution-fix-burst-38 | 2026-07-24 | product-owner | F-WASE-P49-LOW-001 sibling-sweep extension: `scheduled_amendment_in` cleared (ADR-023 amendment completed in v1.5 PLUGIN-MIGRATION-001-G, 2026-05-27); set to `null`; added `amendment_lifecycle: null` per BC-2.01.006 cleared-state convention. |
| 1.5 | PLUGIN-MIGRATION-001-G | 2026-05-27 | product-owner | AC-002 amendment: removed PENDING AMENDMENT banner; added Amendment Note to Description; updated mechanism language from deleted `prism-ocsf/src/mappers/claroty.rs` to SpecDrivenMapper + ocsf_field TOML annotations; updated Description to remove adapter reference; bumped status draft→active; removed amendment_lifecycle: pending. |
| 1.4 | prereq-f | 2026-05-11 | product-owner | PREREQ-F prefix note: added PENDING AMENDMENT — ADR-023 callout under H1 per ADR-023 L370 wording; added scheduled_amendment_in: ADR-023 and amendment_lifecycle: pending to frontmatter. No semantic change to BC body. Full amendment in Wave 2/G. |
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added inputs/input-hash/traces_to/extracted_from frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors; added ## Verification Properties; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract. |
