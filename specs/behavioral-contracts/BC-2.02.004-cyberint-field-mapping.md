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
input-hash: "85d7741"
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

# BC-2.02.004: Cyberint Alert Field Mapping to OCSF

## Description

The Cyberint normalizer maps alert and asset records fetched from the Cyberint Argos API to OCSF Detection Finding (class 2004) or other appropriate event classes. Timestamps are pre-processed by the CyberintTime 4-format parser before OCSF mapping. Severity string values ("high", "medium", "low") are mapped to OCSF `severity_id` enum integers, with unrecognized values mapped to 99 (Other). Cyberint-specific fields (e.g., `threat_type`, `digital_asset_type`) are preserved in `raw_extensions`.

## Preconditions
- A Cyberint alert or asset record has been fetched via the Cyberint Argos API
- Timestamps have been parsed through the CyberintTime 4-format parser

## Postconditions
- Cyberint alert fields map to OCSF Detection Finding (class 2004, Security Finding 2001 deprecated) or appropriate event class
- Cyberint severity string (e.g., "high", "medium", "low") maps to OCSF `severity_id` enum values
- Cyberint timestamp (parsed via CyberintTime) maps to OCSF `time` in RFC 3339 format
- Cyberint-specific fields (e.g., `threat_type`, `digital_asset_type`) are preserved in `raw_extensions`

## Invariants
- DI-005: OCSF schema validity

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Warning | Unknown Cyberint severity string (not in known set) | Mapped to OCSF `severity_id: 99` (Other); warning logged |
| Warning | CyberintTime parser fails on all 4 formats | OCSF `time` set to fetch timestamp; raw string preserved in `raw_extensions`; warning logged (DEC-015) |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-015 | Timestamp in unexpected 5th format | Parse fails gracefully; fetch timestamp used as fallback; record not dropped |
| EC-02-006 | Cyberint asset record (not alert) -- different field structure | Separate field mapping for assets; maps to appropriate OCSF event class |

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.02.004-001 | Cyberint alert with severity="high" and ISO 8601 timestamp | `severity_id: 4` (Critical), `time` in RFC 3339; alert mapped to Detection Finding 2004 |
| TV-BC-2.02.004-002 | Unknown severity string "extreme" | `severity_id: 99` (Other); warning logged with raw value |
| TV-BC-2.02.004-003 | Timestamp in unknown 5th format (DEC-015) | Fetch timestamp used; raw string in `raw_extensions`; warning logged; record not dropped |
| TV-BC-2.02.004-004 | Cyberint asset record (different schema) | Asset-specific mapper applies; maps to appropriate OCSF class |

## Verification Properties

| VP | Verification Aspect |
|----|---------------------|
| VP-016 | OCSF normalization: output is valid protobuf (proptest) |
| VP-017 | OCSF normalization: unmapped fields preserved (proptest) |

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
