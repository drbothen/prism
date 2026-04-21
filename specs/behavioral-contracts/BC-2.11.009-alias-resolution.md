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
capability: "CAP-016"
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
traces_to: ["CAP-016"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.11.009: Alias Resolution — Pre-Parse Expansion, Composition, Cycle Detection

## Description

Alias resolution runs before Chumsky parsing: identifiers matching known alias names are detected in the query string, resolved by scope (per-client overrides global), parameter-substituted (with strict single-token validation to prevent injection), and recursively expanded up to depth 3. The fully expanded query is then checked against the 64KB limit and passed to the parser. Both the original and expanded queries are recorded in `query_context` for transparency. Cycles cannot occur at runtime because they are detected at alias creation time (DI-020); the runtime cycle check is a belt-and-suspenders guard.

## Preconditions
- An PrismQL query string has been received that may contain alias references
- Alias configuration has been loaded and validated at startup (cycles detected, depth validated)
- The query's client scope is known (from tool parameters)

## Postconditions
- Alias references in the query string are detected and expanded before Chumsky parsing:
  1. **Detection**: Identifiers in the query that match known alias names are identified
  2. **Scope resolution**: For each alias reference, scope is resolved: per-client alias (if querying a specific client and alias exists for that client) overrides global alias of the same name
  3. **Parameter substitution**: If the alias is parameterized and invoked with arguments (e.g., `recent_alerts(severity="critical", hours=4)`), parameters are substituted into the template. Missing parameters use defaults.
  4. **Recursive expansion**: If the expanded alias contains references to other aliases, those are expanded recursively (up to depth 3)
  5. **Security check**: The fully expanded query is checked against the 64KB length limit
  6. **Final parse**: The expanded query string is passed to the Chumsky parser
- The original query and expanded query are both recorded in `query_context` for transparency
- Resolution order is inner-to-outer: innermost alias references are expanded first

## Invariants
- DI-020: Composition depth max 3, cycles impossible (detected at config load time)
- Alias expansion cannot widen the query scope beyond what tool parameters allow (intersection semantics still apply)
- Parameter substitution produces a string that is re-parsed; injected values are validated by the same parser with the same security limits
- Parameter values must parse as a single PrismQL literal token (StringLiteral, NumericLiteral, Identifier, or DurationLiteral) per the grammar reference in prismql-grammar.md. Specifically, a valid parameter value is one of:
  - **StringLiteral**: `"quoted string"` (with standard PrismQL escape sequences)
  - **IntegerLiteral**: optional `-` followed by digits (i64 range)
  - **FloatLiteral**: optional `-` followed by digits `.` digits (f64)
  - **BooleanLiteral**: `TRUE` or `FALSE` (case-insensitive)
  - **DurationLiteral**: digits followed by `s`, `m`, `h`, or `d`
  - **Identifier**: `(letter | "_") { letter | digit | "_" }` (e.g., field names, enum values)
  Values that parse as expressions, operators, or compound constructs (e.g., `"x" OR "y"`, `field = value`, `a AND b`, anything containing `|`, `(`, `)`, `=`, `!=`, `>`, `<`, `>=`, `<=`) are rejected with `E-ALIAS-004`. This prevents query injection through parameterized alias values by restricting substitution to atomic tokens that cannot alter query structure.

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-ALIAS-001` | Query references an alias that does not exist in any scope | Structured error with the alias name and available aliases for the current scope |
| `E-ALIAS-001` | Cross-client query uses per-client alias not defined for all queried clients | Structured error with `defined_in` and `missing_in` client lists (DEC-025) |
| `E-QUERY-003` | Expanded query exceeds 64KB | Error noting alias expansion caused the limit to be exceeded |
| `E-ALIAS-004` | Parameterized alias called with unknown parameter name | Error listing valid parameter names for the alias |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| DEC-024 | Alias references undefined alias (should be caught at config load) | If somehow reached at runtime, error with exact alias chain |
| DEC-025 | Cross-client query with per-client alias missing for some clients | Error with list of clients that define and lack the alias |
| EC-11-023 | Alias name is a substring of a field name (e.g., alias `ip` and field `device.ip`) | Only standalone identifiers are matched as aliases; dotted field names are not alias candidates |
| EC-11-024 | Parameterized alias called with zero arguments | Valid; all defaults are used |

## Canonical Test Vectors

> See `.factory/specs/prd-supplements/test-vectors.md` for the canonical test vector tables.

| Input | Expected Output | Category |
|-------|----------------|----------|
| Query `high_sev` where `high_sev` = `severity = 'high'` | Expanded to `severity = 'high'`; recorded in query_context | happy-path |
| Query `recent_alerts(hours=2)` with parameterized alias | Parameters substituted; expansion validated | happy-path |
| Query `unknown_alias` with no matching alias | `Err(E-ALIAS-001)` with available aliases | error |
| Parameter value `"x OR y"` in parameterized call | `Err(E-ALIAS-004)` injection attempt rejected | error |

## Verification Properties

| VP ID | Property | Proof Method |
|-------|----------|-------------|
| VP-012 | Alias depth: rejects composition beyond depth 3 | kani |
| VP-013 | Alias cycles: detects and rejects cyclic references | proptest |
| VP-037 | Alias expansion: never panics on arbitrary alias graphs | fuzz |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-016 |
| L2 Invariants | DI-020 |
| L2 Edge Cases | DEC-024, DEC-025 |
| Priority | P1 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-14 | product-owner | Initial contract |
