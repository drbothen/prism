---
document_type: behavioral-contract
level: L3
version: "1.5"
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
  - ".factory/specs/architecture/decisions/ADR-047-prismql-case-sensitivity-policy-ieq-iin-and-adapter-boundary-normalization.md"
input-hash: "76729b7"
traces_to: ["CAP-003"]
extracted_from: ".factory/specs/prd.md"
introduced: cycle-1
modified: 2026-07-06
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
---

# BC-2.02.002: DynamicMessage Creation from Sensor Records

## Description

When normalizing a raw sensor record to OCSF, the normalizer creates a `DynamicMessage` wrapping the target OCSF event class protobuf descriptor, then sets mapped fields via `prost-reflect` runtime field access. The resulting `OcsfEvent` bundles the `DynamicMessage`, `raw_extensions`, `source_sensor`, and `source_record_type`. Records that fail protobuf encoding are skipped with a logged error, but do not halt batch processing and do not prevent cursor advancement.

## Preconditions
- A sensor adapter has returned a raw sensor record (JSON)
- The OCSF event class for this record type is known (e.g., CrowdStrike alerts map to OCSF Detection Finding, class 2004 (Security Finding 2001 is deprecated))

## Postconditions
- **Adapter-boundary canonical-case normalization (ADR-047 §D.3):** Before the `DynamicMessage` is populated, OCSF enum-label string fields in the normalized record are rewritten to their canonical OCSF Title-case casing per `enum_map.rs` (e.g., `'HIGH'` → `'High'`, `'critical'` → `'Critical'`). The full normalization contract is in BC-2.02.013. The `DynamicMessage` therefore contains canonical-cased enum-label values — downstream case-sensitive PrismQL `=` predicates operate against this canonical data without requiring `IEQ`.
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

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.02.002-001 | Well-formed CrowdStrike alert record | `DynamicMessage` created with all mapped fields set; `OcsfEvent` valid |
| TV-BC-2.02.002-002 | Empty JSON record `{}` | `DynamicMessage` created with no fields; `raw_extensions: {}`; warning logged |
| TV-BC-2.02.002-003 | Field type mismatch: string "42" for severity_id (integer) | Coercion succeeds; field set on DynamicMessage |
| TV-BC-2.02.002-004 | Field type mismatch: non-numeric string for severity_id | Coercion fails; field placed in `raw_extensions`; warning logged |
| TV-BC-2.02.002-005 | DynamicMessage encoding fails (malformed proto) | Record skipped; error logged; cursor advances; batch continues |
| TV-BC-2.02.002-006 | Armis record with `severity='HIGH'` (all-caps) | Normalization applied before DynamicMessage creation; `severity='High'` in DynamicMessage (ADR-047 §D.3, BC-2.02.013) |
| TV-BC-2.02.002-007 | CrowdStrike record with `severity='High'` (already canonical) | Normalization is idempotent; `severity='High'` unchanged in DynamicMessage |

## Verification Properties

| VP | Verification Aspect |
|----|---------------------|
| VP-016 | OCSF normalization: output is valid protobuf (proptest) |
| VP-022 | OCSF normalizer: never panics on arbitrary input (fuzz) |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-003 |
| L2 Invariants | DI-005 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.5 | S-PRISMQL-CASE-INSENSITIVE-001-bc-burst | 2026-07-06 | product-owner | **ADR-047 §D.3 amendment: adapter-boundary canonical-case normalization made explicit in §Postconditions.** Added first postcondition bullet: OCSF enum-label string fields are normalized to canonical OCSF Title-case casing per `enum_map.rs` before `DynamicMessage` is populated; references BC-2.02.013 for full contract. §Canonical Test Vectors: TV-006 (Armis `'HIGH'` → `'High'` after normalization) and TV-007 (idempotent normalization on already-canonical `'High'`) added. inputs: ADR-047 added. |
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added inputs/input-hash/traces_to/extracted_from frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors; added ## Verification Properties with VP-016/VP-022; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract. |
