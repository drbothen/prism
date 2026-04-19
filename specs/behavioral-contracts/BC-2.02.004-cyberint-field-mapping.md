---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-02"
capability: "CAP-003"
lifecycle_status: active
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

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-003 |
| L2 Invariants | DI-005 |
| Priority | P0 |
