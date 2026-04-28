---
document_type: behavioral-contract
level: L3
version: "1.3"
status: draft
producer: product-owner
timestamp: 2026-04-14T07:00:00
phase: 1a
origin: greenfield
subsystem: "SS-11"
capability: "CAP-015"
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
input-hash: "412c872"
traces_to: ["CAP-015"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.11.004: PrismQL Pipe Mode Parsing

## Description

Pipe mode is the highest-priority query mode: it activates whenever the query contains a `|` operator outside string literals, overriding SQL or filter mode detection. The Chumsky parser produces a `PipeExpr` AST representing a linear chain of transformation stages applied left-to-right. Each stage (`where`, `sort`, `head`, `tail`, `stats`, `dedup`, `fields`) is translated to a DataFusion `DataFrame` API call in sequence. The 32-stage limit and security limits from DI-019 apply. Pipe mode is the recommended mode for multi-step analysis workflows.

## Preconditions
- A query string contains `|` outside of string literals -- pipe mode has highest precedence in mode auto-detection (see BC-2.11.002 for full precedence rules). Pipe mode wins even if the query also starts with `SELECT` or `FROM`.
- The query string has passed the 64KB length check

## Postconditions
- The Chumsky parser produces a `PipeExpr` AST consisting of an optional source followed by a chain of pipe stages
- Supported pipe stages:
  - `where <filter_expr>` -- filter rows using the same filter grammar as filter mode
  - `sort <field> [asc|desc] [, <field> [asc|desc]]` -- sort results
  - `head <N>` -- take first N rows (equivalent to `LIMIT N`)
  - `tail <N>` -- take last N rows (reverse sort + limit + reverse)
  - `stats <agg_func> [by <field>]` -- aggregation (count, sum, avg, min, max) with optional GROUP BY
  - `dedup <field> [, <field>]` -- deduplicate by specified fields
  - `fields [+|-] <field> [, <field>]` -- include (`+`) or exclude (`-`) specific fields from output
- Each pipe stage is translated to a DataFusion `DataFrame` API call in sequence
- Pipe stages are applied left-to-right (first stage operates on the full dataset, each subsequent stage operates on the previous stage's output)

## Invariants
- DI-019: Maximum 32 pipe stages enforced; exceeding returns structured error
- DI-019: Nesting depth within `where` expressions tracked against the 64 depth limit
- Each pipe stage produces a valid intermediate `DataFrame`; stage ordering errors (e.g., `head` before `stats`) are not compile errors -- DataFusion evaluates them in order

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-QUERY-001` | Unknown pipe stage keyword | Error with list of supported stages |
| `E-QUERY-003` | More than 32 pipe stages | Structured error: "Query has N pipe stages (max 32). Simplify the query pipeline." |
| `E-QUERY-001` | `stats` with invalid aggregation function | Error listing supported aggregation functions |
| `E-QUERY-001` | `head` or `tail` with non-integer argument | Error: "head/tail requires a positive integer argument" |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-11-009 | Pipe mode with no source prefix (starts with `| where ...`) | Valid; operates on the full materialized events table |
| EC-11-010 | `head 0` | Returns empty result set (valid but unusual) |
| EC-11-011 | `dedup` on a field with all unique values | Returns all rows (no deduplication occurs) |
| EC-11-012 | Multiple `where` stages in sequence | Valid; equivalent to AND-ing the conditions. Each `where` narrows the previous result. |

## Canonical Test Vectors

> See `.factory/specs/prd-supplements/test-vectors.md` for the canonical test vector tables.

| Input | Expected Output | Category |
|-------|----------------|----------|
| `| where severity = 'critical' \| stats count by _sensor` | Aggregate counts per sensor for critical events | happy-path |
| `| where severity = 'high' \| sort event_time desc \| head 10` | Top 10 recent high-severity events | happy-path |
| 33 pipe stages chained | `Err(E-QUERY-003)` pipe stage limit exceeded | error |
| `| stats invalid_func by severity` | `Err(E-QUERY-001)` invalid aggregation function | error |
| `| head 0` | Empty result set (valid) | edge-case |

## Verification Properties

| VP ID | Property | Proof Method |
|-------|----------|-------------|
| VP-021 | PrismQL parser: never panics on arbitrary input | fuzz |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-015 |
| L2 Invariants | DI-019 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract |
