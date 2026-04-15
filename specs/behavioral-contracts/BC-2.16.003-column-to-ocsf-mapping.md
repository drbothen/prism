---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "Config-Driven Adapters & Hot Reload"
capability: "CAP-029"
---

# BC-2.16.003: Column-to-OCSF Mapping at Query Time — Map Sensor Columns to OCSF Fields Per Spec

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

## Error Handling
- All coercion failures are non-fatal: the field value is preserved in `raw_extensions` and a warning is logged with the column name, expected type, actual value, and sensor spec file
- If the `ocsf_class` declared in the table spec is not a valid OCSF event class ID, the entire table's records use the generic `base_event` class (OCSF class 0) with a startup warning

## Traces
- CAP-029 (Config-Driven Sensor Adapters)
- CAP-003 (OCSF Normalization)
- BC-2.02.007 (Vendor extension preservation)
- BC-2.02.008 (Three-tier field alias resolution)
- DI-005 (OCSF schema validity)
