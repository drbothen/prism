---
document_type: behavioral-contract
level: L3
version: "1.1"
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
input-hash: "[pending-recompute]"
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

# BC-2.02.011: Graceful Normalization Error Handling (No Silent Data Loss)

## Description

When OCSF normalization encounters an error (type mismatch, encoding failure, missing required context), the system handles it gracefully without silent data loss. Missing OCSF fields produce valid-but-partial messages (OCSF fields are optional). Type coercion failures divert the field to `raw_extensions`. Protobuf encoding failures skip the record entirely with a logged error, while the cursor still advances past it to prevent reprocessing loops. A warning-level log entry is emitted for every normalization issue.

## Preconditions
- A sensor record is being normalized to OCSF
- The normalization process encounters an error (type mismatch, encoding failure, missing required context)

## Postconditions
- Missing OCSF fields produce valid OCSF messages with those fields absent (OCSF fields are optional by design)
- Type coercion failures result in the field being placed in `raw_extensions` instead of the OCSF message
- Protobuf encoding failures cause the record to be skipped with a logged error, but do not halt the batch
- A warning-level log entry is emitted for each normalization issue, identifying the record, field, and issue
- The cursor advances past skipped records to prevent re-processing infinite loops

## Invariants
- DI-005: Invalid protobuf messages are never delivered downstream
- No sensor data is silently dropped without a log entry

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Warning (non-fatal) | OCSF field mapping produces a type mismatch | Field diverted to `raw_extensions`; warning logged |
| Error (record skipped) | DynamicMessage fails to encode to valid protobuf bytes | Record skipped; error logged with record ID and sensor; cursor advances past it |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-02-020 | Entire batch of records fails normalization (e.g., sensor API changed response format) | All records skipped with errors logged; empty OCSF result set returned; cursor advances; alert-level log for "all records in batch failed normalization" |
| EC-02-021 | Single field causes normalization to take >1s (deeply nested JSON) | No per-field timeout; but total normalization time is included in response `_meta.query_time` for observability |

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.02.011-001 | Record with type mismatch on severity_id | Field in `raw_extensions`; warning logged; rest of record normalized; not skipped |
| TV-BC-2.02.011-002 | DynamicMessage encoding fails (malformed proto state) | Record skipped; error logged; cursor advances; batch continues |
| TV-BC-2.02.011-003 | Entire batch of 50 records fails normalization | All 50 skipped; 50 error log entries; empty result set; alert-level log for full batch failure |
| TV-BC-2.02.011-004 | Record with missing optional OCSF field | Valid DynamicMessage created without that field; not an error; no warning |

## Verification Properties

| VP | Verification Aspect |
|----|---------------------|
| VP-016 | OCSF normalization: output is valid protobuf — proptest verifies encoding never produces invalid messages |
| VP-022 | OCSF normalizer: never panics on arbitrary input (fuzz) |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-003 |
| L2 Invariants | DI-005 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Changes |
|---------|-------|------|--------|---------|
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added inputs/input-hash/traces_to/extracted_from frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors; added ## Verification Properties with VP-016/VP-022; added ## Changelog. |
