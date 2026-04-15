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

# BC-2.13.002: Single-Event Detection — Evaluate Rule Predicate Against Each Materialized Record

## Preconditions
- One or more enabled single-event detection rules exist (rules with `match event where <condition>`)
- A scheduled query execution or ad-hoc query has produced materialized OCSF records
- The detection engine has been invoked for the result set (either inline with scheduled query execution or as a post-query evaluation pass)

## Postconditions
- Each materialized OCSF record is evaluated against all enabled single-event rules
- For each rule, the condition tree is evaluated using the four-tier field resolution chain: (1) OCSF proto fields via dotted path, (2) vendor extension fields via `raw_extensions` JSON, (3) virtual fields (`sensor`, `client_id`, `source`), (4) fallback returns null
- Condition evaluation uses short-circuit logic: `and` exits on first false, `or` exits on first true
- **Operator semantics:**
  - `==`, `!=`: exact equality (string comparison; numeric auto-coercion for int/float)
  - `>`, `>=`, `<`, `<=`: numeric comparison (type error if non-numeric)
  - `contains`: case-sensitive substring match
  - `matches`: pre-compiled regex match (cached at rule load time)
  - `cidr`: IP-in-CIDR-range check via `ipnet` crate
  - `in`: set membership against literal list
- For each record that matches a rule's condition: an alert is generated (BC-2.13.005) with the single trigger event UID
- Rules are evaluated in load order; a single record may match multiple rules, producing multiple alerts

## Invariants
- Evaluation is stateless: each record is evaluated independently with no cross-record context
- A null field value compared to any literal returns false (not an error); `!= <value>` on a null field returns true
- Rule evaluation must complete within the query timeout budget (BC-2.15.007)

## Error Cases
| Error | Condition | Behavior |
|-------|-----------|----------|
| `E-DETECT-001` | Field type mismatch (e.g., numeric comparison on string field) | Predicate evaluates to false; warning logged; evaluation continues |
| `E-DETECT-002` | CIDR parse failure at evaluation time | Predicate evaluates to false; warning logged (should not occur if validated at load time) |

## Edge Cases
| ID | Description | Expected Behavior |
|----|-------------|-------------------|
| EC-13-005 | Record matches 5 different single-event rules | 5 separate alerts generated |
| EC-13-006 | Rule condition references field that exists in CrowdStrike but not Claroty records | Null field for Claroty records; condition evaluates to false; no false alert |
| EC-13-007 | 10,000 records evaluated against 50 rules | All 500K evaluations complete within query timeout; no pre-filtering optimization required for MVP |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-020 |
| L2 Invariants | DI-019 |
| Priority | P1 |
