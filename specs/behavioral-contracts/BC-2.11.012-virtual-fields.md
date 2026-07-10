---
document_type: behavioral-contract
level: L3
version: "1.5"
status: draft
producer: product-owner
timestamp: 2026-04-14T07:00:00
phase: 1a
origin: greenfield
subsystem: "SS-11"
capability: "CAP-015"
lifecycle_status: active
introduced: cycle-1
modified: "2026-07-10"
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md"]
input-hash: "c36ec87"
traces_to: ["CAP-015"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.11.012: Virtual Fields in Queries — `_sensor`, `_client`, `_source_table`

## Description

Three virtual fields — `_sensor`, `_client`, and `_source_table` — are injected as additional Arrow columns into every materialized RecordBatch, making sensor provenance queryable in all PrismQL modes. The underscore prefix distinguishes them from OCSF data fields. Virtual field predicates participate in scope intersection (same semantics as tool-level `sensors`/`clients` parameters). Virtual fields are string-typed; numeric comparisons are type errors. Their names are verified at build time to not collide with OCSF field names.

## Preconditions
- A PrismQL query references `_sensor`, `_client`, or `_source_table` as field names in filter expressions, WHERE clauses, or pipe stages

## Postconditions
- Three virtual fields are available in all PrismQL query modes:
  - **`_sensor`**: The sensor type that produced the event (values: `"crowdstrike"`, `"cyberint"`, `"claroty"`, `"armis"` for external tables; `"prism"` for internal RocksDB-backed tables). Underscore prefix distinguishes virtual fields from OCSF data fields (per BC-2.15.009).
  - **`_client`**: The client ID (OrgSlug value; formerly TenantId, renamed per ADR-006) that owns the sensor instance or the Prism record.
  - **`_source_table`**: The data source table name within the sensor (e.g., `"alerts"`, `"devices"`, `"vulnerabilities"` for external tables; `"alerts"`, `"cases"`, `"rules"`, `"schedules"`, `"diff_results"`, `"audit"`, `"aliases"` for internal tables). Injected during OCSF normalization (external) or during internal table materialization (internal).
- Virtual fields are usable in all positions where regular OCSF fields are usable:
  - Filter mode: `_sensor = "crowdstrike" AND severity >= "high"`
  - SQL mode: `SELECT _sensor, count(*) FROM events GROUP BY _sensor`
  - Pipe mode: `| where _sensor = "claroty" | stats count by _client`
- Virtual fields are implemented as additional Arrow columns in the materialized RecordBatch
- Virtual field predicates participate in scope intersection:
  - `_sensor = "crowdstrike"` in the query intersects with `sensors` tool parameter
  - `_client = "acme"` in the query intersects with `clients` tool parameter
- Virtual field values are strings; comparison operators (`=`, `!=`, `in`) are supported; numeric comparisons (`>`, `<`) on virtual fields are type errors

## Invariants
- Virtual fields are not part of the OCSF schema; they are Prism-specific metadata fields
- Virtual field names cannot collide with OCSF field names (verified at build time against the OCSF proto schema)
- **Empty MemTable schema parity (F-CSD-P14-001 / F-CSD-P16-001):** Every registered table schema — whether produced by the populated RecordBatch path (`inject_virtual_fields`, nullable=false) or the empty-table pre-registration path (`virtual_fields::append_virtual_fields_to_schema` called from `pre_register_empty_tables`, nullable=true) — MUST include all three virtual-field columns (`_sensor` Utf8, `_client` Utf8, `_source_table` Utf8). The nullable=true divergence on the empty path is intentional and required: when the empty table occupies the right side of a LEFT JOIN, DataFusion propagates NULL for those columns rather than producing a schema mismatch error. The spoofed-column dedup guard (`remove_spoofed_virtual_columns` on the populated path; strip-and-append logic inside `append_virtual_fields_to_schema` on the empty path) applies on both paths, preventing duplicate virtual columns if a sensor response already includes a field of the same name. Cross-reference: BC-2.11.005 §Postconditions DEC-022 bullet and §Edge Cases DEC-022 row.

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-QUERY-002` | Numeric comparison on virtual field (e.g., `_sensor > "armis"`) | Type error: "Field '_sensor' is a string virtual field. Use = or != for comparison." |
| `E-QUERY-002` | Invalid sensor name in predicate | Error with list of valid sensor names (including `"prism"` for internal tables) |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-11-029 | `_sensor = "unknown_sensor"` | No records match; empty result set (not an error -- the filter simply excludes everything) |
| EC-11-030 | `SELECT _sensor, _client, _source_table FROM events` | Valid projection; returns only virtual fields for each event |
| EC-11-031 | Virtual field used in `GROUP BY` | Valid; DataFusion groups by the string column normally |

## Canonical Test Vectors

> See `.factory/specs/prd-supplements/test-vectors.md` for the canonical test vector tables.

| Input | Expected Output | Category |
|-------|----------------|----------|
| `SELECT _sensor, count(*) FROM events GROUP BY _sensor` | One row per sensor type with count | happy-path |
| `_sensor = 'crowdstrike' AND severity = 'critical'` | Events from CrowdStrike with critical severity only | happy-path |
| `_sensor > 'armis'` | `Err(E-QUERY-002)` numeric comparison on string virtual field | error |
| `_sensor = 'unknown_sensor'` | Empty result set; not an error | edge-case |
| LEFT JOIN with empty right table; SELECT `_sensor` from right side | Right-side `_sensor` column present in schema; value is NULL (not absent); no schema-mismatch error; empty-path nullable=true enables NULL propagation | edge-case (F-CSD-P14-001) — `test_BC_2_11_012_F_CSD_P14_001_T32_left_join_empty_side_virtual_fields_return_null_not_error` |
| SELECT * with populated-left / zero-batch-right; compare schema of both sides | Both sides include all three virtual columns; empty path nullable=true, populated path nullable=false; schemas are parity-compatible for DataFusion planning | edge-case (F-CSD-P14-001) — `test_BC_2_11_012_F_CSD_P14_001_T33_select_star_empty_side_schema_includes_virtual_fields_parity` |
| LEFT JOIN populated-left / populated-right; SELECT `_sensor`, `_client`, `_source_table` from both sides | Virtual fields present and non-null on both sides (nullable=false); actual string values returned; green-lock confirming populated path unaffected by empty-path changes | green-lock (F-CSD-P14-001) — `test_BC_2_11_012_F_CSD_P14_001_T34_left_join_populated_side_virtual_fields_green_lock` |

## Verification Properties

| VP ID | Property | Proof Method |
|-------|----------|-------------|
| VP-021 | PrismQL parser: never panics on arbitrary input | fuzz |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-015 |
| L2 Invariants | DI-008 (client data separation — `_client` virtual field carries OrgSlug provenance) |
| Related BCs | BC-2.11.005 (virtual fields injected during materialization; §DEC-022 empty-path `append_virtual_fields_to_schema` — cross-reference for empty MemTable schema parity invariant), BC-2.11.011 (scope intersection) |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.5 | DEFECT-CSDEVICES-EMPTY-PIPELINE-001 / F-CSD-P16-001 virtual-field spec anchor | 2026-07-10 | product-owner | **Closes F-CSD-P16-001 (MED) — empty-MemTable virtual-field behavior has no spec anchor.** (1) **§Invariants:** added "Empty MemTable schema parity" invariant: every registered table schema (populated or empty) includes `_sensor`, `_client`, `_source_table`; empty path uses nullable=true for LEFT JOIN NULL propagation; spoofed-column dedup guard applies on both paths; cites F-CSD-P14-001 + F-CSD-P16-001; cross-references BC-2.11.005 §DEC-022. (2) **§Canonical Test Vectors:** added T32 (left-join empty-side virtual fields return NULL), T33 (schema parity between empty and populated paths), T34 (populated-path green-lock). (3) **§Traceability:** added L2 Invariants row (DI-008); expanded Related BCs to explicitly reference BC-2.11.005 §DEC-022. (4) **Frontmatter:** fixed `modified: null` → `modified: "2026-07-10"` (POL-27). No change to Preconditions, Postconditions, Error Cases, Verification Properties. |
| 1.4 | pass-15-remediation | 2026-04-27 | product-owner | `_client` virtual field description updated TenantId → OrgSlug (ADR-006). |
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract |
