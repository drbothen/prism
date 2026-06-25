---
document_type: adversarial-review
story: S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001
pass: prlevel-5
pr: "#203"
pr_head: "216e19fa"
base_develop: "903c8fcb"
result: NOT_CLEAN
clean_strict: false
clean_pr_merge: false
streak_before: 0
streak_after: 0
findings_total: 2
findings_high: 1
findings_obs: 1
date: 2026-06-25
state_decision: D-1351
---

# PR-LEVEL Adversarial Pass 5 — S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001

**PR HEAD reviewed:** `216e19fa`
**Base develop:** `903c8fcb`
**Date:** 2026-06-25
**CLEAN(strict):** NO
**CLEAN(PR-merge):** NO
**3-CLEAN streak:** RESET 0/3 on `b58a1a49`

> Note: This pass was the FIRST clean pass on `216e19fa` per the first-pass
> adversary. A SECOND independent fresh-context pass on the SAME HEAD surfaced
> HIGH-1 below — validating that a single-pass "clean" is insufficient under
> BC-5.39.001 multi-pass strict 3-CLEAN discipline.

---

## Findings

### HIGH-1 — TD-VSDD-060 Systemic Sibling-Sweep: Ast::SqlPipe Not Swept Through Plan-Time AST Match Sites

**Severity:** HIGH (CRIT-equivalent for correctness gates; TD-VSDD-060 mandatory sibling-sweep breach)
**Category:** Missing match arm / fell-through wildcard
**BC anchor:** BC-5.39.001 (production-grade completeness), ADR-033 §T1 (push-down extractors), E-QUERY-011/037/038 (gate invariants)

**Finding:**

The new top-level `Ast::SqlPipe` variant — added to support the SQL-pipe query
mode — was not swept through the plan-time AST extractor and gate functions.
All 11 affected match sites silently fell through their `_ => {}` wildcard arm,
producing the following behavioral gaps:

| Site | File | Consequence |
|------|------|-------------|
| `walk_ast` | `prism-query/src/ast.rs` | SqlPipe nodes not visited by the shared walk visitor |
| `extract_sources_from_ast_for_gate` | `prism-query/src/planner.rs` | E-QUERY-011 audit-capability gate silent bypass for SqlPipe-mode queries (mitigated by Layer-2 scan gate, but degrades the structured error path) |
| `extract_source_names_recursive` | `prism-query/src/planner.rs` | Source-name extraction returns empty set for SqlPipe head |
| `extract_time_window_from_ast_from_query` | `prism-query/src/planner.rs` | ADR-033 T1 time-window push-down extractor returns `None` for SqlPipe head; a SqlPipe head `WHERE timestamp > NOW()-INTERVAL '24h'` does NOT push down → over-fetch vs 200 MB per-query budget |
| `extract_push_down_filters_as_map` | `prism-query/src/planner.rs` | ADR-033 T1 equality-filter push-down extractor returns empty map for SqlPipe head → over-fetch |
| `check_query_column_availability` | `prism-query/src/planner.rs` | E-QUERY-038 column gate not applied to SqlPipe-mode queries |
| `extract_sources_from_ast` (explain.rs) | `prism-query/src/explain.rs` | EXPLAIN source extraction returns empty for SqlPipe |
| `query_mode_str` (explain.rs) | `prism-query/src/explain.rs` | EXPLAIN mode string returns generic fallback instead of "sqlpipe" |
| `post_fetch_operations_from_ast` (explain.rs) | `prism-query/src/explain.rs` | EXPLAIN post-fetch ops omitted for SqlPipe |
| `has_or_predicate` (explain.rs) | `prism-query/src/explain.rs` | OR-predicate detection always returns false for SqlPipe |
| `explain_filter_exprs` (explain.rs) | `prism-query/src/explain.rs` | Filter expression explanation omitted for SqlPipe |

**Demo relevance:** The ADR-033 T1 push-down gap is demo-relevant — a SqlPipe
query with a time-window predicate in the head `WHERE` clause would over-fetch
sensor data, risking the 200 MB per-query memory budget during the live demo.

**No test exercised SqlPipe through any of these 11 gates before this pass.**

**Status: CLOSED**

Implementer addressed `216e19fa → b58a1a49`:

- Added `Ast::SqlPipe` arms to all 11 match sites: `walk_ast`,
  `extract_sources_from_ast_for_gate`, `extract_source_names_recursive`,
  `extract_time_window_from_ast_from_query`, `extract_push_down_filters_as_map`,
  `check_query_column_availability`, and the four `explain.rs` functions
  (`extract_sources_from_ast`, `query_mode_str`, `post_fetch_operations_from_ast`,
  `has_or_predicate`, `explain_filter_exprs`).
- Added 6 load-bearing gate tests:
  - E-QUERY-011 deny path: SqlPipe query against sensor requiring audit capability → denied
  - E-QUERY-011 allow path: SqlPipe query with audit capability present → allowed
  - E-QUERY-037 availability gate: SqlPipe query against unavailable sensor → E-QUERY-037
  - E-QUERY-038 column gate: SqlPipe query referencing non-existent column → E-QUERY-038
  - Push-down filter map: SqlPipe head equality filter extracted correctly
  - Push-down time-window: SqlPipe head time-window predicate extracted correctly
- `just check` EXIT 0 (1135 prism-query tests); non-exhaustive 87; SAP-1 clean.

---

### OBS-1 (MED) — ADR-045 D3: Missing `| tail` NegativeE040 Example in REFERENCE_EXAMPLES

**Severity:** MED (ADR-045 D3 compliance gap — CI-gate obligation requires BOTH `| limit` and `| tail` examples)
**Category:** Spec/test coverage gap
**BC anchor:** ADR-045 D3 (error-taxonomy v2.00 E-QUERY-040 CI-gate obligation)

**Finding:**

The error-taxonomy v2.00 E-QUERY-040 CI-gate obligation requires BOTH a
`| limit` NegativeE040 example AND a `| tail` NegativeE040 example in the
shared `REFERENCE_EXAMPLES` constant (used by the 3-tier gate test that drives
all NegativeE040 entries through `plan_sqlpipe_query`). After the P3 taxonomy
v2.00 update that changed E-QUERY-040 to fire on both `PipeStage::Limit` and
`PipeStage::Tail`, only the `| limit` example was present in `REFERENCE_EXAMPLES`.
The `| tail` variant was absent, leaving the `PipeStage::Tail` firing predicate
path without a live gate-test exercise.

**Status: CLOSED**

Implementer added `SELECT * FROM crowdstrike.detections LIMIT 10 | tail 5`
NegativeE040 entry to `REFERENCE_EXAMPLES`. This entry is auto-exercised by the
3-tier gate test which drives all NegativeE040 entries through
`plan_sqlpipe_query` — confirming the `PipeStage::Tail` firing predicate path
is now covered by a load-bearing test.

---

## Probes Passing

All other probes PASS:

- All 27 ACs verified against `216e19fa` diff
- AC-019 BLOCKER-001 deferred (D-1326 — do not flag)
- AC-020 runbook v1.4 satisfied
- FORBID-BOTH (Limit+Tail): both `PipeStage::Limit` and `PipeStage::Tail` fire
  E-QUERY-040 (verified in prior passes; not regressed)
- Filter-arm apostrophe: `O'Brien` correctly escapes to `'O''Brien'` (P4 OBS-1 CLOSED)
- Demo tape reproducibility: all 11 tapes reproducible with committed driver scripts (P4 HIGH-1 CLOSED)
- E-QUERY-040 neutral wording (`| limit`/`| tail`) per taxonomy v2.00 (P3 OBS-1 CLOSED)
- NYA handler doc comments accurate (P2 OBS-1 CLOSED)
- GRAMMAR-013 table headers verbatim (P2 OBS-2 CLOSED)
- D2-on-bare-ORDER: intentional helpful superset — DO-NOT-FLAG
- Temporal plain-string: D-1335 adjudicated
- E-QUERY-036/037 label distinction: correct
- SAP-1 (tracing emission catalog): clean — no new event_type sites without BC-2.16.002 row
- SAP-2: no TOML sensor spec mutations in diff
- SID-1: no `#[ignore]`'d test rationalization
- TD-VSDD-059: all claimed closures have load-bearing tests (not doc-comment-only)
- Security probes: no new credential exposure, no SQL injection vectors introduced

---

## Post-Close Status

**New FROZEN PR HEAD:** `b58a1a49`
**just check:** EXIT 0 (1135 prism-query tests pass; full workspace GREEN)
**non-exhaustive:** 87 (EXPECTED=87 — unchanged)
**fmt-canonical:** clean
**3-CLEAN streak:** RESET 0/3 on `b58a1a49` (code HEAD moved by HIGH-1 + OBS-1 fix)
**develop_head:** UNCHANGED (`903c8fcb`)

**NEXT:** PR-LEVEL adversary 3 consecutive CLEAN(strict) on UNCHANGED `b58a1a49`
→ CI green → squash-merge (--admin D-1337) → post-merge POL-14 BC promotion
→ pre-flight demo re-audit → T13 capstone → T14 recording.
