---
document_type: behavioral-contract
level: L3
version: "1.0"
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
---

# BC-2.11.002: PrismQL Filter Mode Parsing

## Preconditions
- A query string is provided and mode auto-detection has resolved to filter mode
- **Mode auto-detection precedence** (applied in order, first match wins):
  1. If the query contains `|` outside string literals -> **pipe mode** (BC-2.11.004)
  2. If the query starts with `SELECT` (case-insensitive) -> **SQL mode** (BC-2.11.003)
  3. If the query starts with `FROM` (case-insensitive) and has no `|` outside string literals -> **SQL mode** (BC-2.11.003)
  4. Otherwise -> **filter mode** (this BC)
- Pipe mode wins over SQL mode when both could match (e.g., `SELECT ... | where ...` is pipe mode)
- The query string has passed the 64KB length check

## Postconditions
- The Chumsky parser produces a `FilterExpr` AST representing the boolean expression
- Supported operators: `=`, `!=`, `>`, `>=`, `<`, `<=`, `in`, `contains`, `matches` (regex), `between`, `is null`, `is not null`, `exists`, `cidr` (IP range)
- Boolean combinators: `AND`, `OR`, `NOT` with standard precedence (NOT > AND > OR); parentheses for grouping
- Value types: string literals (double-quoted), integers, floats, booleans, null, IP addresses, CIDR notation
- Field names support dot-notation for nested OCSF fields (e.g., `device.ip`, `src_endpoint.port`)
- Alias references in filter position are detected and expanded before parsing (see BC-2.11.009)
- The `FilterExpr` AST is translated to a DataFusion `Expr` for execution

## Invariants
- DI-019: Nesting depth tracked during recursive parsing; exceeding 64 returns structured error
- Regex patterns validated at parse time using Rust `regex` crate (finite automaton, immune to catastrophic backtracking); max pattern length 1024 bytes (CWE-1333)
- CIDR notation validated at parse time (CWE-20)
- Integer arithmetic uses i128 intermediate representation to detect overflow (CWE-190)

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-QUERY-001` | Unexpected token in filter expression | Error with position, the unexpected token, and syntax help: `"Filter mode syntax: field op value [AND\|OR field op value ...]"` |
| `E-QUERY-001` | Unknown field name | Error with `similar_fields` suggestions based on OCSF field name similarity |
| `E-QUERY-002` | Type mismatch (e.g., `severity >= 42` when severity is string) | Error with field type info and correct usage example |
| `E-QUERY-003` | Nesting depth exceeds 64 | Structured error identifying the limit exceeded |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-11-003 | Empty query string | Error: "Query string is empty. Provide a filter expression, SQL query, or pipe expression." |
| EC-11-004 | Query is just an alias name with no operators | Expand alias, parse expanded result as filter expression |
| EC-11-005 | Field name matches a reserved keyword (e.g., `select`, `from`) | In filter mode, treat as field name (no keyword reservation in filter mode) |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-015 |
| L2 Invariants | DI-019 |
| Priority | P0 |
