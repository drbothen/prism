---
document_type: behavioral-contract
level: L3
version: "1.1"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-05"
capability: "CAP-007"
lifecycle_status: active
introduced: cycle-1
modified: null
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "67e5667"
traces_to: ["CAP-007"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.05.007: Audit Entries Are Compatible with the Vector Pipeline

## Description

Audit entries are emitted as single-line JSON objects on stderr using `tracing-subscriber`'s
JSON layer format. Field names use `snake_case`. Standard `tracing-subscriber` fields
(`timestamp`, `level`, `target`, `span`, `fields`) are used as-is; Prism-specific audit fields
are nested consistently for Vector's JSON parser. Timestamps use ISO 8601 with UTC timezone,
parseable by Vector's `parse_timestamp` transform. Entries never use multi-line
pretty-printing.

Stderr is strictly separated from stdout (MCP JSON-RPC), ensuring audit entries never
interfere with the MCP protocol stream.

## Preconditions
- The `tracing` subscriber is configured with a JSON output format
- The downstream Vector pipeline expects structured JSON log lines on stderr (or a configured output)

## Postconditions
- Each audit entry is emitted as a single-line JSON object (no multi-line pretty-printing)
- The JSON structure uses `snake_case` field names consistent with convention-reconciliation.md
- Standard fields (`timestamp`, `level`, `target`, `span`, `fields`) follow `tracing-subscriber` JSON layer conventions
- Prism-specific audit fields are nested under a consistent key (e.g., `fields.audit.*` or top-level structured fields) parseable by Vector's JSON parser
- The `timestamp` field uses ISO 8601 format with UTC timezone, parseable by Vector's `parse_timestamp` transform

## Invariants
- DI-004: Audit completeness -- Vector compatibility ensures the pipeline can ingest all entries

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| Serialization failure | A field value cannot be serialized to JSON (e.g., non-UTF-8 string) | The field value is replaced with a placeholder string `"<serialization_error>"` and the entry is still emitted |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-05-011 | Audit entry contains very large `parameters` or `result_summary` (e.g., a bulk query with thousands of filter terms) | The entry is emitted as-is; truncation is not applied at the Prism layer. Vector pipeline configuration handles size limits if needed. |
| EC-05-012 | Tracing subscriber outputs to stderr which is captured by the MCP host process | Audit entries on stderr do not interfere with MCP JSON-RPC on stdout; the two streams are strictly separated |

## Canonical Test Vectors

See `.factory/specs/prd-supplements/test-vectors.md` for canonical test vectors for BC-2.05.007.

| Scenario | Input | Expected Output |
|----------|-------|----------------|
| Normal audit entry | Single tool invocation | Single-line JSON on stderr; valid JSON parseable by `jq .` |
| Timestamp format | Any invocation | `timestamp` field matches ISO 8601 UTC pattern, e.g., `"2026-04-20T12:34:56.789Z"` |
| Non-UTF-8 field | Parameter containing non-UTF-8 bytes | Field replaced with `"<serialization_error>"`; rest of entry intact |

## Verification Properties

No VPs in VP-INDEX v1.5 directly verify Vector pipeline compatibility. Placeholder for future VP (integration test: emit N audit entries, verify all parseable by Vector's JSON parser).

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-007 |
| L2 Invariants | DI-004 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
