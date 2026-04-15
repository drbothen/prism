---
document_type: behavioral-contract
level: L3
version: "1.0"
status: draft
producer: product-owner
timestamp: 2026-04-13T12:00:00
phase: 1a
origin: greenfield
subsystem: "Detection Engine"
capability: "CAP-020"
---

# BC-2.13.001: Detection Rule Loading — Parse AxiQL Predicate, Validate at Load Time, Reject Invalid Rules

## Preconditions
- Detection rules are provided as `.axd` files in a configured rules directory, as built-in rule string constants, or via the `create_rule` MCP tool (BC-2.13.006)
- Each rule has the mandatory structure: `rule <identifier> { meta { ... } match <match_clause> alert { ... } }`

## Postconditions
- Each rule source is parsed via the detection grammar (Pest PEG or Chumsky-based parser)
- **Structural validation at parse time:**
  - `meta` block must contain `name` (string) and `severity` (info/low/medium/high/critical)
  - `alert` block must contain `title` and `description` (template strings)
  - Sequence rules must have at least one step
  - Unknown meta keys cause hard parse errors
- **Security validation at parse time (differs from axiathon spike -- hardened):**
  - Max rule source size: 16KB
  - Max condition nesting depth: 16
  - Regex patterns validated via `RegexBuilder::size_limit(1_048_576)` (1MB compiled regex limit, CWE-1333)
  - CIDR strings validated at parse time (not deferred to evaluation)
  - Max 32 predicates per condition tree
- **Field path advisory validation:** field paths are checked against the OCSF schema; unknown fields produce warnings (not errors) to allow vendor extension fields
- Valid rules are indexed by rule type (single-event, correlation, sequence) and stored per-scope (BC-2.13.011)
- Invalid rules are rejected with structured errors; all other rules continue loading

## Invariants
- A rule that passes loading will not cause a parse error at evaluation time
- Regex patterns are compiled once at load time and cached for the lifetime of the rule
- Rule loading is atomic per-source file: if a file contains multiple rules and one is invalid, all rules from that file are rejected; all other rules from other source files continue loading

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-RULE-001` | Rule source exceeds 16KB | Structured error with size |
| `E-RULE-002` | Missing required meta field (`name` or `severity`) | Structured error with field name |
| `E-RULE-003` | Invalid regex pattern (fails compilation or exceeds size limit) | Structured error with pattern and regex engine error |
| `E-RULE-004` | Condition nesting exceeds 16 levels | Structured error with nesting depth |
| `E-RULE-005` | Unknown match mode (not `event`, `count`, `sequence`) | Structured error |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-13-001 | Rule references vendor extension field `claroty.risk_score` | Warning logged (field not in OCSF schema); rule loads successfully |
| EC-13-002 | Rules directory does not exist | Warning logged; no rules loaded; detection engine starts with zero rules |
| EC-13-003 | Two rules in the same file have the same identifier | Parse error; both rules rejected |
| EC-13-004 | Rule with `enabled: false` in meta block | Rule is parsed and stored but excluded from active evaluation |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-020 |
| L2 Invariants | DI-019 |
| Priority | P0 |
