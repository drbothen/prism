---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-14T07:00:00
phase: 1a
origin: greenfield
subsystem: "Query Engine & Aliases"
capability: "CAP-015"
---

# BC-2.11.004: AxiQL Pipe Mode Parsing

## Preconditions
- A query string contains `|` outside of string literals, indicating pipe mode
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
| `PrismError::QueryParse` | Unknown pipe stage keyword | Error with list of supported stages |
| `PrismError::QuerySecurityLimit` | More than 32 pipe stages | Structured error: "Query has N pipe stages (max 32). Simplify the query pipeline." |
| `PrismError::QueryParse` | `stats` with invalid aggregation function | Error listing supported aggregation functions |
| `PrismError::QueryParse` | `head` or `tail` with non-integer argument | Error: "head/tail requires a positive integer argument" |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-11-009 | Pipe mode with no source prefix (starts with `| where ...`) | Valid; operates on the full materialized events table |
| EC-11-010 | `head 0` | Returns empty result set (valid but unusual) |
| EC-11-011 | `dedup` on a field with all unique values | Returns all rows (no deduplication occurs) |
| EC-11-012 | Multiple `where` stages in sequence | Valid; equivalent to AND-ing the conditions. Each `where` narrows the previous result. |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-015 |
| L2 Invariants | DI-019 |
| Priority | P0 |
