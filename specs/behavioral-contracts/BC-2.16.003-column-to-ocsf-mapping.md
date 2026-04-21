---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "SS-16"
capability: "CAP-029"
lifecycle_status: active
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "ac6b633"
traces_to:
  - "CAP-029"
extracted_from: ".factory/specs/prd.md"
---

# BC-2.16.003: Column-to-OCSF Mapping at Query Time — Map Sensor Columns to OCSF Fields Per Spec

## Description

After a spec-driven table's multi-step fetch pipeline returns raw records, columns with
`ocsf_field` mappings are translated to the corresponding OCSF protobuf fields using
the standard four-tier resolution from BC-2.02.008. Columns without mappings are
preserved in the `raw_extensions` JSON blob. Type coercion is applied for mismatched
types with non-fatal fallback to `raw_extensions` on failure.

The resulting `OcsfEvent` is uniform across all sensors: downstream consumers
(detection rules, cross-sensor correlation, decorators) cannot distinguish spec-driven
data from built-in adapter data. Invalid OCSF field paths produce a warning at spec
load time (not a hard error) because OCSF schema extensions may introduce fields not
in the compiled schema.

## Preconditions
- A spec-driven table has been fetched via the multi-step pipeline (BC-2.16.002) and raw records are available
- The table's `ColumnSpec` entries include `ocsf_field` mappings (some columns may have `ocsf_field: None`, meaning no OCSF mapping)
- The OCSF normalizer (CAP-003) is available

## Postconditions
- For each record fetched from the spec-driven sensor:
  - Columns with an `ocsf_field` value are mapped to the corresponding OCSF field in the DynamicMessage protobuf representation
  - The mapping follows the standard four-tier field resolution (BC-2.02.008): Prism metadata fields, proto descriptor fields, unmapped JSON blob, None
  - Columns without an `ocsf_field` mapping are preserved in the `raw_extensions` JSON blob (consistent with BC-2.02.007)
  - The `ocsf_class` declared at the table level determines which OCSF event class the DynamicMessage uses (e.g., `security_finding`, `device_inventory`, `network_activity`)
- Type coercion is applied when the column's declared type differs from the OCSF field's expected type:
  - `string` -> OCSF integer field: parse as integer, fall back to `raw_extensions` if parsing fails (with warning)
  - `integer` -> OCSF string field: convert to string representation
  - `datetime` -> OCSF timestamp field: parse using ISO 8601 with fallback to Unix epoch seconds/milliseconds
  - Coercion failures are logged at warning level but do not drop the record (the field is placed in `raw_extensions` instead)
- The resulting `OcsfEvent` is uniform across all sensors — downstream consumers (query engine, detection rules, decorators) cannot tell which spec file produced the data
- Cross-sensor correlation works identically: any sensor's `device.ip` column mapped to `ocsf_field = "device.ip"` correlates with any other sensor's `device.ip` OCSF field

## OCSF Field Validation
- At spec load time (BC-2.16.009), each `ocsf_field` value is validated against the compiled OCSF protobuf schema
- Invalid OCSF field paths produce a warning at load time but do not reject the spec (the mapping is skipped at runtime, and the column goes to `raw_extensions`)
- This is a warning, not an error, because OCSF schema extensions may introduce fields not in the compiled schema

## Invariants
- Coercion failures are non-fatal: the field value is preserved in `raw_extensions` (record is never dropped due to type mismatch)
- The `ocsf_class` at table level determines the OCSF event class for all records in that table
- Spec-driven OcsfEvents are indistinguishable from built-in adapter OcsfEvents to downstream consumers

## Error Conditions
| Error | Condition | Behavior |
|-------|-----------|----------|
| (warning only) | Coercion failure for a column value | Field placed in `raw_extensions`; warning logged with column name, expected type, actual value, spec file; record not dropped |
| (warning only) | Invalid `ocsf_class` in table spec | All records use generic `base_event` class (OCSF class 0) with startup warning |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| Column with no ocsf_field | column has `ocsf_field: None` | Column value placed in `raw_extensions` |
| String to OCSF int coercion | `"42"` -> int field | Parsed as 42; succeeds |
| Non-parseable string to int | `"not-a-number"` -> int field | Value placed in `raw_extensions`; warning logged; record included |
| Invalid ocsf_class | `ocsf_class: "made_up_class"` | Records use base_event (class 0); startup warning |
| Cross-sensor correlation | two sensors both map `device_ip` -> `ocsf_field = "device.ip"` | Both queryable as `device.ip`; JOIN works transparently |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for full canonical vectors.

| Scenario | Input | Expected Output |
|----------|-------|-----------------|
| Happy path — full mapping | all columns have ocsf_field; all types match | OcsfEvent with all fields mapped; raw_extensions empty |
| Mixed mapping | some columns have ocsf_field, some don't | Mapped columns in OCSF proto; unmapped in raw_extensions |
| Coercion failure | string value for int field | Field in raw_extensions; warning; record included |
| Invalid ocsf_class | table has unknown ocsf_class | base_event class used; warning at load |

## Verification Properties

| VP ID | Description |
|-------|-------------|
| (none) | Coercion-failure record preservation is semantically identical to VP-017 (unmapped fields preserved in raw_extensions); cross-sensor correlation requires full query engine integration test; no additional formal VP. |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-029 |
| L2 Invariants | DI-005 |
| Related BCs | CAP-003 (OCSF Normalization), BC-2.02.007 (vendor extension preservation), BC-2.02.008 (four-tier field resolution) |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-74-fix | 2026-04-20 | product-owner | Resolved (placeholder) row in ## Verification Properties per pass-74 VP-TBD decision matrix extension. |
| 1.2 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description; added ## Invariants; added ## Error Conditions (from inline Error Handling); converted ## Traces → ## Traceability table; added ## Canonical Test Vectors; added ## Verification Properties; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial draft (used ## Traces section) |
