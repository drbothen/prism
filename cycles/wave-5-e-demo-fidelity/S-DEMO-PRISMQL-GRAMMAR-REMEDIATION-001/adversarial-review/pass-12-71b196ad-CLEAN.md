---
pass: 12
story: S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001
frozen_head: 71b196ad
base_head: 903c8fcb
date: 2026-06-25
adversary_pass_type: LOCAL
clean_strict: true
clean_pr_merge: true
findings_count: 0
streak: "1/3"
---

# LOCAL Adversarial Pass 12 — S-DEMO-PRISMQL-GRAMMAR-REMEDIATION-001

**Frozen HEAD:** `71b196ad`
**Base (develop):** `903c8fcb`
**CLEAN(strict):** YES — zero findings of ANY severity
**CLEAN(PR-merge):** YES
**Streak:** 1/3 (first of three consecutive clean passes on UNCHANGED HEAD `71b196ad`)

## Pass Summary

Adversary reviewed the full diff `903c8fcb..71b196ad` with no prior context from passes 1–11.

### Highest-Risk Surface: FORBID-BOTH Completeness

The primary concern entering this pass was whether the D-1346 Tail+Limit fix was genuinely complete.

Verified:
- `plan_sqlpipe_query` now checks both `PipeStage::Limit` and `PipeStage::Tail` for the FORBID-BOTH invariant (INV-FORBID-BOTH-PERMANENT / ADR-043 D4).
- `SELECT * FROM t LIMIT 5 | tail 3` → `Err(RedundantRowLimit)` / E-QUERY-040 — confirmed by `test_bc_2_11_007_tail_after_limit_rejected`.
- `SELECT * FROM t LIMIT 5 | limit 3` (limit-after-limit) → `Err(RedundantRowLimit)` — regression test present and load-bearing.
- `SELECT * FROM t | sort ts desc` (non-capping stage, positive control) → not triggered — discriminating negative control confirmed.
- No third PipeStage variant caps rows (Join does not, Where/Sort/Stats/Fields/Dedup/Enrich do not). Sibling sweep of PipeStage arms complete per TD-VSDD-060. No gap found.

### Filter Arm Temporal Guard

- `Ast::Filter` execute arm carries `predicate_has_unfolded_temporal_pub` guard.
- Bare `Expr::Interval` comparison RHS now returns structured `QueryExecutionFailed` (E-QUERY-038 / E-QUERY-039), not empty SQL / redacted generic failure.
- Load-bearing tests: `test_bc_2_11_filter_temporal_guard_bare_interval_rhs` + sibling positive control confirmed.

### All 27 ACs Verified

AC-001 through AC-027 inspected against diff. All acceptance criteria met.

Notable AC surface coverage:
- AC-002 (FORBID-BOTH gate for `| limit`): load-bearing test against plan level.
- AC-002 extension (Tail path): new load-bearing test `test_bc_2_11_007_tail_after_limit_rejected`.
- AC-004/AC-005 (NOW()/INTERVAL inject_now): all 4 AST arms confirmed wired (SQL, SqlPipe head, filter, pipe stage).
- AC-007 (FORBID-BOTH E-QUERY-040 verbatim): message string confirmed unchanged.
- AC-009 (D1 mode-bridge message verbatim): all 3 mandated substrings present and tested.
- AC-010 (normalize_pql round-trip): `engine.rs::normalize_pql` present, no regression.
- AC-015/AC-016 (MCP investigate_host / missing-arg via real rmcp duplex): 2 load-bearing tests in `mcp_infrastructure.rs`.
- AC-019 (AC-019 deferral): BLOCKER-001 root-cause = connect-timeout, not KV staleness; reset_token_cache removed from production path; unit test present, scope-bounded per D-1329.
- AC-020 (runbook v1.4 §5.5): satisfied prior burst; LOCAL do-not-flag.
- AC-021 (did_you_mean suggestion): `available_tables + Some(suggestion)` wired.
- AC-022 (pure-pipe guided errors): `rewrite_enrich_parse_errors` + `rewrite_d2_sql_keyword_in_pipe_position` applied.
- AC-023 (IS-NOT-NULL JSON-list semantics note): note present in `resources.rs`; load-bearing test present.
- AC-024 (PR description GRAMMAR-013 table): PR-LEVEL deliverable — LOCAL do-not-flag.
- AC-025 (SqlPipe error path guided messages): unified SqlPipe pipe-stage error path via shared helper; 2 load-bearing tests.
- AC-026/AC-027 (D2 mode-bridge `rewrite_d2_sql_keyword_in_pipe_position`): verbatim message verified; WHERE/LIMIT uppercase excluded.

### SAP-1 (Tracing Emission Catalog)

Grep `rg 'event_type\s*=' crates/ --type rust` run across full workspace. No new `event_type` emission sites introduced in `71b196ad` diff. All pre-existing catalog rows verified present in BC-2.16.002 §Postconditions. SAP-1 PASS.

### SAP-2 (DTU↔TOML Schema Parity)

No `.prism/specs/sensors/*.toml` modifications in diff. SAP-2 N/A.

### SID-1 (No-Ignored-Test Rationalization)

No `#[ignore]`'d tests introduced in diff. All new tests are non-ignored, in-process unit tests. SID-1 PASS.

### Paper-Fix Detection (TD-VSDD-059)

Every closed finding has load-bearing tests. No doc-comment-only or rename-only closures. TD-VSDD-059 PASS.

### Production-Grade Invariants

- No `unwrap()` / `expect()` in production code paths introduced.
- No `println!` in production code introduced.
- Arc-DI plumbing unchanged (no regressions).
- Non-exhaustive count: 87 (CLAUDE.md worktree value; within scope).
- Error taxonomy variants used: E-QUERY-040 (FORBID-BOTH), E-QUERY-038/039 (temporal). No new variants introduced without taxonomy registration.

## Conclusion

**CLEAN(strict) = YES. CLEAN(PR-merge) = YES. Zero findings.**

Implementation is "unusually complete" per adversary assessment. Every high-risk surface was verified closed with load-bearing tests. The FORBID-BOTH Limit+Tail coverage, the temporal guard, and all 27 ACs are satisfied against frozen HEAD `71b196ad`.

**Streak: 1/3.** Second clean pass required on UNCHANGED HEAD `71b196ad`.
