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

# BC-2.02.002: DynamicMessage Creation from Sensor Records

## Preconditions
- A sensor adapter has returned a raw sensor record (JSON)
- The OCSF event class for this record type is known (e.g., CrowdStrike alerts map to OCSF Security Finding, class 2001)

## Postconditions
- A `DynamicMessage` is created wrapping the target OCSF event class protobuf descriptor
- Mapped fields from the sensor record are set on the `DynamicMessage` via `prost-reflect` runtime field access
- The `DynamicMessage` is valid according to the OCSF protobuf schema (all set fields have correct types)
- The resulting `OcsfEvent` includes: `event_class`, `message` (DynamicMessage), `raw_extensions`, `source_sensor`, `source_record_type`

## Invariants
- DI-005: OCSF schema validity -- the DynamicMessage conforms to the compiled protobuf descriptor

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Warning (non-fatal) | A required OCSF field cannot be mapped from the sensor record | DynamicMessage is created with the field absent (OCSF fields are optional by design); warning logged with field name and record type |
| Fatal (record skipped) | Protobuf encoding of the DynamicMessage fails | Error logged; record skipped; not delivered downstream; cursor still advances past it |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-02-003 | Sensor record is empty JSON `{}` | DynamicMessage created with no mapped fields; all OCSF fields absent; `raw_extensions` is `{}`; valid but minimally useful |
| EC-02-004 | Sensor record field has wrong type (e.g., string where number expected) | Type coercion attempted (string "42" to int 42); if coercion fails, field placed in `raw_extensions` instead of OCSF message |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-003 |
| L2 Invariants | DI-005 |
| Priority | P0 |
