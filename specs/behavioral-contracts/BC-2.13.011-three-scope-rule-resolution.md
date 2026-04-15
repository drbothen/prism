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

# BC-2.13.011: Three-Scope Rule Resolution — Global Baseline + Per-Client Overrides + Analyst Ad-Hoc

## Preconditions
- Detection rules exist at one or more scopes: global, client, analyst
- A scheduled execution or rule listing targets a specific client_id

## Postconditions
- The effective rule set for a client is computed by merging three scopes:
  1. **Global baseline**: rules that apply to all clients (managed by MSSP provider)
  2. **Client overrides**: rules specific to a client_id; a client rule with the same rule_id as a global rule **replaces** the global rule for that client (not both fire)
  3. **Analyst ad-hoc**: session-scoped rules that supplement the merged set; analyst rules never override global or client rules (they are additive only)
- Merge semantics:
  - Start with all global rules
  - For each client rule: if its rule_id matches a global rule, replace the global rule; otherwise, add it
  - Add all analyst rules (must have unique rule_ids; duplicate IDs with existing rules are rejected at creation time)
- The effective rule set is recomputed when: a rule is created/deleted at any scope, or a different client_id is being evaluated
- For `list_rules` with `client_id`: each rule in the response is annotated with `effective_scope` indicating which scope it came from and whether it overrides another

## Invariants
- A client never evaluates the same rule_id from two different scopes (override semantics prevent duplicates)
- Analyst rules are additive only: they cannot suppress global or client rules
- Global rule updates propagate to all clients that have not overridden that rule
- Client-scope rules with no matching global rule are standalone (not considered "overrides")

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-RULE-010` | Analyst creates a rule with an ID that already exists at global or client scope | Structured error: "Rule ID '{id}' already exists at {scope} scope; analyst rules must use unique IDs" |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-13-038 | Client A overrides global rule "brute_force" with a higher threshold | Client A evaluates with threshold 10; Client B evaluates with global threshold 5 |
| EC-13-039 | Global rule deleted while client override exists | Client override becomes standalone; other clients lose the rule entirely |
| EC-13-040 | Analyst creates rule during session, then session ends | Rule is lost; no persistence; detection engine rebuilt without it |
| EC-13-041 | 50 clients, each with 5 client-specific rules, plus 20 global rules | Each client evaluates 25 rules (20 global + 5 client); memory for 50 x 25 = 1250 effective rules |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-020 |
| L2 Invariants | DI-008 |
| Priority | P0 |
