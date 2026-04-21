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
capability: "CAP-020"
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
input-hash: "5b48b9c"
traces_to: ["CAP-020"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.13.001: Detection Rule Loading — Parse PrismQL Predicate, Validate at Load Time, Reject Invalid Rules

## Description

Detection rules are loaded at startup from `.detect` files in the rules directory, built-in string constants, or via the `create_rule` MCP tool. Each rule is parsed through a Pest PEG or Chumsky parser, validated structurally (required meta fields, severity enum, alert block) and security-hardened (16KB max, 16 nesting depth, 1MB compiled regex limit, 32 predicates max). Regex patterns are compiled and cached at load time. Rules with advisory issues (unknown OCSF field paths, window shorter than schedule interval) emit warnings but are accepted. Invalid rules are rejected without affecting other rules in other source files.

## Preconditions
- Detection rules are provided as `.detect` files in a configured rules directory, as built-in rule string constants, or via the `create_rule` MCP tool (BC-2.13.006)
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
- **Correlation window vs schedule interval advisory:** if a correlation or sequence rule specifies a time window shorter than any schedule interval it is bound to (directly or via pack), a warning is emitted at load time: "Rule '{rule_id}' has a {window_duration} window but is bound to schedule '{schedule_name}' with a {interval_duration} interval. Events may be missed between evaluations. Consider increasing the rule window or decreasing the schedule interval." This is advisory only (not a hard error) because the rule may also be evaluated via ad-hoc queries.
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

## Canonical Test Vectors

> See `.factory/specs/prd-supplements/test-vectors.md` for the canonical test vector tables.

| Input | Expected Output | Category |
|-------|----------------|----------|
| Valid single-event rule file with 1 rule | Rule loaded, indexed by type, stored in scope | happy-path |
| Rule file exceeding 16KB | `Err(E-RULE-001)` | error |
| Rule with backtracking regex `(a+)+b` | `Err(E-RULE-003)` regex fails compilation | error |
| Rule referencing vendor extension field | Warning logged; rule loads successfully | edge-case |

## Verification Properties

| VP ID | Property | Proof Method |
|-------|----------|-------------|
| VP-018 | Detection rule validation: rejects invalid rules | proptest |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-020 |
| L2 Invariants | DI-019, DI-024 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial contract |
