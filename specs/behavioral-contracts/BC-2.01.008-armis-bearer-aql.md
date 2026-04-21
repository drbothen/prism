---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-14T05:00:00
phase: 1a
origin: greenfield
subsystem: "SS-01"
capability: "CAP-001"
lifecycle_status: active
inputs:
  - ".factory/specs/prd.md"
  - ".factory/specs/domain-spec/capabilities.md"
input-hash: "85d7741"
traces_to: ["CAP-001"]
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

# BC-2.01.008: Armis Bearer Token Auth with AQL Query Forwarding and Timestamp Fallback

## Description

The Armis Centrix adapter authenticates via a static API key (bearer token) and forwards queries as AQL (Armis Query Language) to the GetSearch endpoint. Because Armis records use inconsistent timestamp and ID field names across its 7 data sources, the adapter employs per-source fallback chains (1-3 candidate timestamp fields, 2-4 candidate ID fields) to reliably construct a `(Timestamp, TypeSpecificID)` cursor. Records with no valid timestamp in any fallback field are included in results but do not advance the cursor.

## Preconditions
- Armis Centrix sensor is configured with a static API key (bearer token)
- The target data source is one of the 7 Armis sources (alerts, activities, audit_logs, risk_factors, connections, devices, vulnerabilities)

## Postconditions
- All Armis API requests use the bearer token via the SDK's GetSearch endpoint
- Queries are expressed in AQL (Armis Query Language) and forwarded to the GetSearch API
- Timestamp extraction uses the per-source fallback chain (1-3 candidate fields)
- ID extraction uses the per-source fallback chain (2-4 candidate fields)
- Cursor is a `(Timestamp, TypeSpecificID)` 2-tuple

## Invariants
- DI-012: Sealed auth trait
- DI-001: Cursor forward progress

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `PrismError::Sensor` | Armis API key rejected (HTTP 401) | `category: "authentication"`, suggestion: "Verify Armis API key in credential store" |
| `PrismError::Sensor` | AQL syntax error (HTTP 400) | `category: "api_contract"`, include the AQL query and Armis error message in the structured error |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-013 | Record has no valid timestamp in any of the fallback fields | Warning logged identifying the record; record included in response but with null cursor contribution; does not advance cursor |
| EC-01-011 | All records in a page lack valid timestamps | Page treated as having no cursor advancement; pagination halts to prevent infinite loops |
| EC-01-012 | ID fallback chain exhausted (no valid ID field found) | Record logged as warning and skipped; cursor does not account for this record |

## Canonical Test Vectors

| Test Vector ID | Description | Expected |
|----------------|-------------|----------|
| TV-BC-2.01.008-001 | Armis alerts source with valid API key; all records have primary timestamp field | Records fetched; cursor advanced with `(Timestamp, AlertID)` 2-tuple |
| TV-BC-2.01.008-002 | Record missing primary timestamp; secondary fallback field present | Secondary timestamp used; cursor correctly set; warning logged |
| TV-BC-2.01.008-003 | Record has no timestamp in any fallback field (DEC-013) | Record included; cursor not advanced for this record; warning logged |
| TV-BC-2.01.008-004 | HTTP 401 API key rejection | `PrismError::Sensor` with `category: "authentication"` |
| TV-BC-2.01.008-005 | AQL syntax error (HTTP 400) | `PrismError::Sensor` with `category: "api_contract"` including AQL query text |

## Verification Properties

| VP | Verification Aspect |
|----|---------------------|
| (none) | No VP directly verifies this BC — see VP-INDEX.md for full map |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-001 |
| L2 Invariants | DI-001, DI-012 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added inputs/input-hash/traces_to/extracted_from frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors; added ## Verification Properties; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract. |
