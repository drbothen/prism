---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "Sensor Query Pipeline"
capability: "CAP-001"
---

# BC-2.01.015: MCP Tool Response Envelope Structure

## Preconditions
- A sensor query tool has completed execution (success or partial success)

## Postconditions
- Response includes `_meta` object with: `tool`, `data_source`, `query_time`, `trust_level`, `safety_flags`, `pagination` fields
- `_meta.trust_level` is always `"untrusted_external"` for sensor data
- `_meta.pagination` includes `cursor` (opaque string), `has_more` (bool), and `total_count` (nullable int)
- `results` array contains sensor records with both raw and OCSF-normalized representations
- `content_summary` provides an LLM-consumable text summary (e.g., "Found 23 CrowdStrike alerts for client acme-corp")
- Sensor data values appear only in `structuredContent`, never interpolated into `content[].text` prose

## Invariants
- DI-006: Prompt injection sanitization -- untrusted sensor data never in prose text
- DI-004: Audit completeness

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| N/A | Response serialization failure | Internal error logged; generic error response returned to MCP client |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-008 | Hostname contains potential prompt injection text | Hostname placed in `structuredContent.hostname`; parallel `hostname_safety_flag` field populated; `content[].text` does not include the hostname |
| EC-01-024 | Response exceeds MCP message size limits | Results truncated to fit; `truncated: true` with `truncation_reason: "response_size_limit"` |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 |
| L2 Invariants | DI-004, DI-006 |
| Priority | P0 |
