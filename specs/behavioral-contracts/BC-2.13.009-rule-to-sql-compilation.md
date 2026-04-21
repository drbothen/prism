---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "SS-13"
capability: "CAP-027"
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
input-hash: "abc4070"
traces_to: ["CAP-027"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.13.009: Rule-to-SQL Compilation — Translate Detection Predicates to DataFusion WHERE Clauses

## Description

Detection rule conditions are compiled to DataFusion SQL or LogicalPlan at rule load time, with compiled plans cached per rule_id. Single-event rules produce a flat WHERE clause; correlation rules produce GROUP BY + HAVING with a COUNT; sequence rules produce windowed self-joins or correlated subqueries ordered by event_time. Dotted OCSF field paths are translated to flattened Arrow column names. Vendor extension fields use the `json_extract_string` UDF. Compiled plans respect DI-019 security limits. Compilation failures cause the rule to be rejected at load time — no runtime compilation errors.

## Preconditions
- A detection rule has been parsed and validated (BC-2.13.001)
- The DataFusion SessionContext is available with registered OCSF schema and security UDFs (BC-2.13.010)

## Postconditions
- **Single-event rules** are compiled to: `SELECT * FROM events WHERE <condition_tree>` where each predicate is translated to a DataFusion expression:
  - `field == value` -> `"field" = 'value'` (string) or `"field" = N` (numeric)
  - `field contains value` -> `"field" LIKE '%value%'`
  - `field matches pattern` -> `regexp_match("field", 'pattern') IS NOT NULL`
  - `field cidr range` -> `subnet_contains('range', "field")` (custom UDF)
  - `field in (values)` -> `"field" IN ('v1', 'v2', ...)`
  - `and`/`or`/`not` -> SQL `AND`/`OR`/`NOT`
- **Correlation rules** are compiled to: `SELECT <group_by_fields>, COUNT(*) as match_count, ARRAY_AGG(event_uid) as trigger_uids FROM events WHERE <condition_tree> AND event_time >= <window_start> GROUP BY <group_by_fields> HAVING COUNT(*) <op> <threshold>`
- **Sequence rules** are compiled to windowed self-joins or correlated subqueries ordered by event_time, joined on the key field
- Dotted field paths (e.g., `src_endpoint.ip`) are translated to the flattened Arrow column name using the OCSF schema mapping
- Compiled plans are cached per rule_id; recompilation only occurs when the rule source changes
- Push-down optimization: compiled WHERE clauses are pushed down to sensor API filters where possible (BC-2.11.007)

## Invariants
- The compiled SQL produces semantically identical results to interpretive condition evaluation (BC-2.13.002)
- Compilation failures are detected at rule load time, not at execution time
- Compiled plans respect the same security limits as PrismQL queries (DI-019)

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-RULE-008` | Condition references field not present in OCSF schema or vendor extensions | Compilation succeeds with advisory warning; at execution time, the column resolves to NULL |
| `E-RULE-009` | Sequence rule too complex for SQL compilation (exceeds join depth) | Fallback to interpretive evaluation with performance warning |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-13-030 | Rule with 32 predicates (max allowed) | Compiles to a deeply nested WHERE clause; DataFusion handles expression tree natively |
| EC-13-031 | Correlation rule with multiple group_by fields | Compiles to multi-column GROUP BY; group key in result matches concatenation semantics |
| EC-13-032 | Rule references vendor extension field in `raw_extensions` | Compiled to `json_extract_string(raw_extensions, '$.field_name')` expression |

## Canonical Test Vectors

> See `.factory/specs/prd-supplements/test-vectors.md` for the canonical test vector tables.

| Input | Expected Output | Category |
|-------|----------------|----------|
| Single-event rule with `field == 'value'` | Compiled to `"field" = 'value'` DataFusion expr | happy-path |
| Correlation rule with `count >= 5 group_by src_endpoint.ip within 5m` | GROUP BY + HAVING + window_start clause | happy-path |
| Rule with unknown OCSF field | Warning; column resolves to NULL at execution | edge-case |
| Sequence rule exceeding join depth | Fallback to interpretive evaluation with warning | edge-case |

## Verification Properties

| VP ID | Property | Proof Method |
|-------|----------|-------------|
| VP-014 | Query security limits: rejects oversized queries | kani |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-027 |
| L2 Invariants | DI-019 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial contract |
