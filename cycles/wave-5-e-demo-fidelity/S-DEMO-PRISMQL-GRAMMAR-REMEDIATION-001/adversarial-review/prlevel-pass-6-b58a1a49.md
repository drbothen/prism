---
document_type: adversarial-review
story: S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001
pass: prlevel-6
pr: "#203"
pr_head: "b58a1a49"
base_develop: "903c8fcb"
result: NOT_CLEAN
clean_strict: false
clean_pr_merge: true
streak_before: 0
streak_after: 0
findings_total: 1
findings_low: 1
date: 2026-06-25
state_decision: D-1352
---

# PR-LEVEL Adversarial Pass 6 — S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001

**PR HEAD reviewed:** `b58a1a49`
**Base develop:** `903c8fcb`
**Date:** 2026-06-25
**CLEAN(strict):** NO
**CLEAN(PR-merge):** YES
**3-CLEAN streak:** RESET 0/3 on `9eab936b`

---

## Findings

### OBS-1 (LOW) — TD-VSDD-060 Parity: SqlPipe Extractor Arms Did Not Iterate `PipeStage::Join` Sources

**Severity:** LOW (TD-VSDD-060 parity gap — latent, no live defect)
**Category:** Incomplete match-arm coverage / parity gap vs Ast::Pipe arms
**BC anchor:** TD-VSDD-060 (sibling-site sweep), E-QUERY-011/037 (gate invariants)

**Finding:**

The P5 fix sweep added `Ast::SqlPipe` arms to all 11 plan-time AST match sites,
correctly walking `spq.head` (the SQL `SELECT` head). However, none of the four
SqlPipe extractor arms iterated `spq.stages` to collect sources from
`PipeStage::Join` stage entries — a parity gap vs the `Ast::Pipe` arms, which
do walk join sources.

The gap affects four extractor functions:

| Function | File | Gap |
|----------|------|-----|
| `extract_source_names_shallow` | `prism-query/src/planner.rs` | `PipeStage::Join` source not collected from SqlPipe stages |
| `extract_source_names_recursive` | `prism-query/src/planner.rs` | `PipeStage::Join` source not collected from SqlPipe stages |
| `extract_sources_from_ast_for_gate` | `prism-query/src/table_registry.rs` | `PipeStage::Join` source not collected from SqlPipe stages |
| `extract_sources_from_ast` | `prism-query/src/explain.rs` | `PipeStage::Join` source not collected from SqlPipe stages |

**Practical impact:** A SqlPipe query with a `| join prism_audit` stage would
not have the `prism_audit` source reach the E-QUERY-011 audit-capability gate or
the E-QUERY-037 availability gate. This is a **latent parity gap, not a live
defect** — the pipe-mode `JOIN` emitter currently returns `Err("not yet
supported")` before any table read occurs, so no production query exercises this
path today.

**Status: CLOSED**

Implementer addressed `b58a1a49 → 9eab936b`:

- Added `PipeStage::Join` source collection to all 4 SqlPipe extractor arms:
  `extract_source_names_shallow` and `extract_source_names_recursive` in
  `materialization.rs`; `extract_sources_from_ast_for_gate` in
  `table_registry.rs`; `extract_sources_from_ast` in `explain.rs`.
- Confirmed SqlPipe-with-join parses and reaches the gates before the emitter
  `Err` fires.
- Added 2 load-bearing tests:
  - Layer-1 audit-gate source discovery: SqlPipe `| join` stage source collected
    by `extract_sources_from_ast_for_gate`
  - Availability-gate source discovery: SqlPipe `| join` stage source collected
    by `extract_source_names_recursive`
- `just check` EXIT 0 (4938 tests); non-exhaustive 87; SAP-1 clean.

---

## Probes Passing

All other probes PASS:

- All 27 ACs verified against `b58a1a49` diff
- AC-019 BLOCKER-001 deferred (D-1326 — do not flag)
- AC-020 runbook v1.4 satisfied
- Ast::SqlPipe gate-sweep COMPLETE (all 11 plan-time gates mode-agnostic for SqlPipe — P5 HIGH-1 CLOSED)
- SqlPipe `| tail` NegativeE040 example present in REFERENCE_EXAMPLES (P5 OBS-1 CLOSED)
- FORBID-BOTH (Limit+Tail): both `PipeStage::Limit` and `PipeStage::Tail` fire E-QUERY-040
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

**New FROZEN PR HEAD:** `9eab936b`
**just check:** EXIT 0 (4938 tests pass; full workspace GREEN)
**non-exhaustive:** 87 (EXPECTED=87 — unchanged)
**fmt-canonical:** clean
**3-CLEAN streak:** RESET 0/3 on `9eab936b` (code HEAD moved by OBS-1 fix)
**develop_head:** UNCHANGED (`903c8fcb`)

**NEXT:** PR-LEVEL adversary 3 consecutive CLEAN(strict) on UNCHANGED `9eab936b`
→ CI green → squash-merge (--admin D-1337) → post-merge POL-14 BC promotion
→ pre-flight demo re-audit → T13 capstone → T14 recording.
