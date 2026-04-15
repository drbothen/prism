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
capability: "CAP-016"
---

# BC-2.11.009: Alias Resolution -- Pre-Parse Expansion, Composition, Cycle Detection

## Preconditions
- An AxiQL query string has been received that may contain alias references
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

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-016 |
| L2 Invariants | DI-020 |
| L2 Edge Cases | DEC-024, DEC-025 |
| Priority | P1 |
