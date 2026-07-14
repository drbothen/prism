---
document_type: adversarial-review
scope: LOCAL
fix_pr: DEFECT-PQL-FNCALL-LHS-001
passes: [26]
feature_head_at_review: b55c7708
date: 2026-07-14
clean_strict: false
clean_pr_merge: true
finding_counts:
  total: 3
  crit: 0
  high: 0
  med: 0
  low: 0
  obs: 3
  process_gap: 0
code_behavior_defects: 0
streak_after: 0/3
convergence: IN_PROGRESS
authored_by: orchestrator-relay
---

# LOCAL Adversary Pass 26 — DEFECT-PQL-FNCALL-LHS-001

---

## Pass 26 (frozen b55c7708; fresh-context adversary; fn-call-LHS PrismQL grammar + ADR-048 §D.7 aggregate gate + expr_to_sql FuncCall arm + SqlPipe stage span offset translation; LOCAL cascade; streak RESET 1/3 → 0/3)

**Pass result:** CLEAN(strict)=NO, CLEAN(PR-merge)=YES

**Findings:** 3 (all OBS); 2 additional out-of-scope items noted

Note: adversary recovered from a transient API interruption mid-pass and completed the review on frozen b55c7708 (2026-07-14).

---

## Findings

### F-PQLFN-P26-OBS-001 [OBS][grammar-observation]

**BC LOW-002 enumerated 6 compare operators; grammar ships 7 (pre-existing == alias)**

BC-2.11.004 §Error Cases LOW-002 enumerated the six compare operators `=, !=, <, <=, >, >=`. The PrismQL grammar ships a seventh production: `==` as a pre-existing alias for `=`. The BC text understated the grammar surface by one operator.

**Status:** CLOSED by fix-burst-20 — BC-2.11.004 v1.42 LOW-002 corrected to 7-alternative enumeration with `==` documented in parenthetical as "note: == is a pre-existing synonym for =".

---

### F-PQLFN-P26-OBS-002 [OBS][ux-edge-case]

**Keyword-shaped fn-names (NOT/AND/OR/...) admitted as Unknown fn-calls at parse layer**

The parser admitted keyword-shaped identifiers (e.g., `NOT(x)`, `AND(x, y)`) as `ScalarFunc::Unknown` calls. At the grammar level these parsed without error, producing a call expression with a keyword-form function name. RED evidence revealed the pre-fix behavior was WORSE than analyzed: keyword fn-names silently parsed, executed, and returned 0 rows at execution (silent-failure mode — no E-QUERY-001 grammar rejection, no plan-time error at any layer).

**Status:** CLOSED by fix-burst-20 — BC-2.11.004 v1.42 LOW-006 ADJUDICATED option (b): reserved-keyword fn-name exclusion REQUIRED. 20-keyword case-insensitive list (NOT/AND/OR/XOR/IN/LIKE/ILIKE/BETWEEN/IS/NULL/TRUE/FALSE/CAST/CASE/WHEN/THEN/ELSE/END/EXTRACT/INTERVAL) + exact E-QUERY-001 template + canonical test vector. Code @1a07a5f9: `.validate()` call at end of `fn_call_comparison` production (try_map rejected — Chumsky choice() error-priority would swallow the keyword message). IN-SCOPE DISCOVERY: `parse_sql_with_limits` F-MEDIUM-001 recovery path was swallowing `.validate()` semantic errors — guarded with E-QUERY-001-prefix check.

---

### F-PQLFN-P26-OBS-003 [OBS][documentation-parity]

**`sql_parser` `scalar_call` `Span::ZERO` lacked inline comment**

In `sql_parser.rs`, the `scalar_call` production constructed spans with `Span::ZERO` without an inline comment explaining why (span tracking not available at parse-text level in the SQL parser path; normalization happens in `shift_scalar_spans_in_stages` post-parse). This created an unexplained divergence between the pipe parser (which does track spans) and the SQL parser path.

**Status:** CLOSED by fix-burst-20 code @1a07a5f9 — inline comment added on the `Span::ZERO` construction site.

---

## Out-of-Scope Items Noted

The following items were observed but classified out-of-scope for DEFECT-PQL-FNCALL-LHS-001:

1. **ast.rs `PqlNormalizer` `_ => "func"` wildcard:** The `PqlNormalizer` retains a catch-all `_ => "func"` wildcard for unrecognized function names in one internal path. This is a pre-existing pattern predating this defect; not a regression introduced by this branch.

2. **`==` alias pre-existing:** The `==` → `=` alias in the grammar predates this branch. Documented in BC-2.11.004 v1.42 LOW-002 (OBS-001 closure) but not a new defect introduced by this branch.

---

## SAP-1 Verification

SAP-1 (tracing emission catalog completeness) PASS:
- `rg 'event_type\s*=' crates/ --type rust` executed across workspace
- All `event_type =` emissions pre-existing and catalogued in BC-2.16.002 §Postconditions Canonical Structured Event Catalog
- Zero new `event_type =` emissions introduced in this branch at frozen HEAD b55c7708

---

## Fix-Burst 20 Summary

All 3 OBS findings CLOSED. Three components:

**Spec layer — PO commit @04b0fa19:**
- BC-2.11.004 v1.41→v1.42: LOW-002 corrected to 7-alternative enumeration (== alias documented in parenthetical); LOW-006 ADJUDICATED option (b): reserved-keyword fn-name exclusion REQUIRED — 20-keyword case-insensitive list + exact E-QUERY-001 template + canonical test vector
- POL-23 sweep: S-PRISMQL-CASE-INSENSITIVE-001 v1.66→v1.67 (4 BC-2.11.004 v1.41→v1.42 pin sites updated)

**MCP lane sync — PO commit @9d1f6b5e:**
- BC-2.10.007 v1.16→v1.17: §RETRYABLE-503 §Implementer Code Follow-Up updated — `.as_u16()` form in snippet corrected to `.code()` form
- POL-23 sweep: S-MCP-E003 v0.6→v0.7 (9 pins) + S-TEST-WIRESHAPE v0.15→v0.16 (11-12 pins) + S-MCP-THREATINTEL-PROD-ENDPOINT-001 v0.2→v0.3 (security_review: required added)

**Code layer — implementer commit @1a07a5f9:**
- LOW-006: `.validate()` at end of `fn_call_comparison` production (try_map rejected — Chumsky choice() error-priority would swallow the keyword error message)
- IN-SCOPE DISCOVERY: `parse_sql_with_limits` F-MEDIUM-001 recovery path was swallowing `.validate()` semantic errors — guarded with E-QUERY-001-prefix check
- OBS-003: `scalar_call` `Span::ZERO` inline comment added
- 7 tests: 5 RED→GREEN (pipe/SQL-WHERE/SqlPipe-stage × case-insensitivity + lower() form) + 2 positive locks (NOT-space-form + lower())
- just iter 1627/1627; just check 5563/5563 GREEN; non-exhaustive 91/91

---

## Frozen-HEAD Rule and Streak Status

Per BC-5.39.001 (3-CLEAN convergence protocol) and DRIFT-ORCH-PRLEVEL-PUSH-001 (frozen-HEAD streak rule):

- Pass-25 was CLEAN(strict)=YES on frozen b55c7708 → streak was 1/3
- Pass-26 has 3 OBS findings → CLEAN(strict)=NO → streak RESETS to 0/3
- Fix-burst-20 advanced HEAD from b55c7708 to 1a07a5f9; new frozen HEAD for streak counting is 1a07a5f9
- DRIFT-ORCH-PRLEVEL-PUSH-001: NO pushes to branch until 3/3 achieved on 1a07a5f9

---

## Cascade Status

- Total passes: 26 (LOCAL cascade only; pre-push)
- Fix-bursts: 20 completed
- LOCAL 3-CLEAN streak: 0/3 on NEW frozen HEAD 1a07a5f9
- Previous frozen HEAD: b55c7708 (LOCAL-ONLY, NOT pushed; pass-25 was 1/3 on this head)
- New frozen HEAD: 1a07a5f9 (LOCAL-ONLY, NOT pushed; streak starts 0/3)
- Next: LOCAL pass 27 on frozen 1a07a5f9 (dispatched)
