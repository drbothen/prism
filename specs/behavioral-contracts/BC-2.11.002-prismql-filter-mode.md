---
document_type: behavioral-contract
level: L3
version: "1.5"
status: active
producer: product-owner
timestamp: 2026-04-14T07:00:00
phase: 1a
origin: greenfield
subsystem: "SS-11"
capability: "CAP-015"
lifecycle_status: active
introduced: cycle-1
modified: 2026-07-06
deprecated: null
deprecated_by: null
replacement: null
retired: null
removed: null
removal_reason: null
inputs: [".factory/specs/prd.md", ".factory/specs/domain-spec/capabilities.md", ".factory/specs/architecture/decisions/ADR-047-prismql-case-sensitivity-policy-ieq-iin-and-adapter-boundary-normalization.md"]
input-hash: "c36ec87"
traces_to: ["CAP-015"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.11.002: PrismQL Filter Mode Parsing

## Description

Filter mode is the default query mode: it activates when a query does not start with `SELECT`/`FROM` and contains no `|` pipe operators outside string literals. The Chumsky parser produces a `FilterExpr` AST representing a boolean predicate over OCSF fields. The grammar supports comparison, membership, containment, regex, CIDR, null-check, and existence operators with standard boolean combinators. Security limits (nesting depth 64, regex max 1024 bytes) are enforced at parse time. The resulting AST is translated to a DataFusion `Expr` for execution over the materialized Arrow table.

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
- **Case-insensitive equality/membership operators (ADR-047 D.2):** `IEQ` (case-insensitive equality), `IIN` (case-insensitive membership), `INE` (case-insensitive inequality) are also supported. These are lowered to `lower(field) OP lower('val')` DataFusion SQL and extend the existing I-prefix family (`ICONTAINS`/`ISTARTSWITH`/`IENDSWITH`). See BC-2.11.024 for full semantics. Default `=`/`!=`/`IN` remain case-sensitive.
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

## Canonical Test Vectors

> See `.factory/specs/prd-supplements/test-vectors.md` for the canonical test vector tables.

| Input | Expected Output | Category |
|-------|----------------|----------|
| `severity = 'critical'` | `FilterExpr{Eq("severity", "critical")}` (case-sensitive, unchanged) | happy-path |
| `severity IEQ 'high'` | `FilterExpr{Eq("severity", "high", case_insensitive: true)}`; DataFusion lowers to `lower(severity) = lower('high')` | happy-path (IEQ) |
| `status IIN ('open', 'new')` | `FilterExpr{In("status", ["open","new"], case_insensitive: true)}`; DataFusion lowers to `lower(status) IN (lower('open'), lower('new'))` | happy-path (IIN) |
| `src_endpoint.ip cidr '10.0.0.0/8'` | `FilterExpr{Cidr("src_endpoint.ip", "10.0.0.0/8")}` | happy-path |
| `""` (empty string) | `Err(E-QUERY-001)` empty query error | error |
| 65 levels of nested `( ... )` | `Err(E-QUERY-003)` nesting depth exceeded | error |
| `severity matches '(a+)+'` | `Err(E-QUERY-001)` invalid regex at parse time | error |

## Execution Validation Requirements (ADR-046 D4)

Filter mode execution is UNVERIFIED unless the following two integration tests exist and pass:

1. **`test_filter_mode_simple_predicate`**: executes `severity='HIGH'` as `Ast::Filter` against a mocked or DTU sensor source via `QueryEngine::execute`; asserts rows matching the predicate are returned and rows NOT matching are absent.

2. **`test_filter_mode_with_source`**: executes a source-qualified filter (e.g., `crowdstrike_detections | severity='HIGH'`) via `QueryEngine::execute`; asserts correct row filtering.

Both tests MUST use `QueryEngine::execute`, not just `PrismQlParser::parse`. Parse-only tests do NOT satisfy the execution validation requirement.

## Verification Properties

| VP ID | Property | Proof Method |
|-------|----------|-------------|
| VP-015 | Query security limits: rejects excessive nesting depth | kani |
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
| 1.5 | S-PRISMQL-CASE-INSENSITIVE-001-bc-burst | 2026-07-06 | product-owner | **ADR-047 D.2 amendment: IEQ/IIN/INE operators added to filter-mode supported operator list.** §Postconditions: added bullet for case-insensitive equality/membership operators (`IEQ`/`IIN`/`INE`), lowering idiom (`lower(field) OP lower('val')`), reference to BC-2.11.024 for full semantics. §Canonical Test Vectors: added IEQ and IIN happy-path vectors. inputs: ADR-047 added. |
| 1.4 | PR-203-post-merge-POL-14 | 2026-06-26 | state-manager | **POL-14 BC auto-promotion: draft → active.** Anchor story S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001 squash-merged via PR #203 to develop@7e60df03 (2026-06-26; CI 43/43 green; 9-round PR-LEVEL 3-CLEAN(strict) cascade on frozen HEAD 356e0573). `status: draft → active`. No behavioral change; frontmatter status field only. |
| 1.4 | demo-readiness-2026-06-24 | 2026-06-24 | product-owner | AMENDMENT: added §Execution Validation Requirements (ADR-046 D4). Filter mode execution was UNVERIFIED — parse-only tests do not satisfy BC-2.11.002 which specifies "Filter mode predicates are applied to the sensor data source." Two mandatory integration tests added: `test_filter_mode_simple_predicate` and `test_filter_mode_with_source`, both using `QueryEngine::execute`. Closes ADR-046 D4 obligation. BC-2.11.023 governs the D7 shared-predicate-grammar invariant as a companion constraint. |
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract |
