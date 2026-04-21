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
input-hash: "abc4070"
traces_to: ["CAP-020"]
extracted_from: ".factory/specs/prd.md"
---

# BC-2.13.002: Single-Event Detection — Evaluate Rule Predicate Against Each Differential Record

## Description

Single-event detection evaluates each new/added record from differential results against all enabled single-event rules (`match event where <condition>`). Evaluation is stateless and independent per record; a null field compared against any literal returns false. The four-tier field resolution chain (OCSF proto fields, vendor extensions via raw_extensions, virtual fields, null fallback) is used for every predicate. Short-circuit logic applies. A matching record generates one alert per matching rule. Evaluation must complete within the query timeout budget.

## Preconditions
- One or more enabled single-event detection rules exist (rules with `match event where <condition>`)
- A scheduled query execution has produced differential results (CAP-018) containing added records
- The detection engine has been invoked for the differential result set

## Postconditions
- Each record in the differential results (new/added records only, NOT the full materialized result set) is evaluated against all enabled single-event rules
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

## Canonical Test Vectors

> See `.factory/specs/prd-supplements/test-vectors.md` for the canonical test vector tables.

| Input | Expected Output | Category |
|-------|----------------|----------|
| 1 record matching 1 single-event rule | 1 alert generated with trigger event UID | happy-path |
| Record with null `severity` field against rule `severity == 'critical'` | Evaluates to false; no alert | edge-case |
| Record matching 3 different rules | 3 alerts generated | edge-case |
| Numeric comparison `>` on string field | Predicate false; warning logged; evaluation continues | error |

## Verification Properties

| VP ID | Property | Proof Method |
|-------|----------|-------------|
| VP-018 | Detection rule validation: rejects invalid rules | proptest |
| VP-024 | Injection scanner: detects known injection patterns | proptest |

## Traceability
| Field | Value |
|-------|-------|
| L2 Capability | CAP-020 |
| L2 Invariants | DI-019 |
| Priority | P0 |

## Changelog

| Version | Burst | Date | Author | Change |
|---------|-------|------|--------|--------|
| 1.3 | pass-73-fix | 2026-04-20 | state-manager | Deterministic changelog reorder: sorted all rows to descending version order (pass-73 bash script). |
| 1.2 | pass-69-housekeeping | 2026-04-20 | product-owner | Normalized changelog schema to canonical 5-col schema. |
| 1.1 | pre-build-sweep | 2026-04-20 | product-owner | Template-compliance sweep: added extracted_from/inputs/input-hash/traces_to frontmatter; added ## Description synthesized from body; added ## Canonical Test Vectors scaffolding; added ## Verification Properties cross-ref; added ## Changelog. |
| 1.0 | cycle-1 | 2026-04-13 | product-owner | Initial contract |
